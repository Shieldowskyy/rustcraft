//! Background chunk meshing worker pool (`AsyncMesher`) built on PAL threading hooks.
//! Falls back seamlessly to synchronous meshing when `worker_thread_count` returns 0 (e.g. WASM).

use std::sync::mpsc::{Receiver, Sender};

use crate::platform::capabilities;
use crate::platform::threading::{configure_worker_thread_affinity, worker_thread_count};

use super::culling::{build_chunk_mesh_from_job, build_chunk_mesh_from_job_limited};
use super::types::{ChunkMeshJob, Triangle};

pub struct AsyncMesher {
    req_tx: Sender<(u64, ChunkMeshJob)>,
    res_rx: Receiver<(u64, (i32, i32), Vec<Triangle>)>,
    res_tx: Sender<(u64, (i32, i32), Vec<Triangle>)>,
    worker_count: usize,
    max_triangles: Option<usize>,
    generation: u64,
}

impl Default for AsyncMesher {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn build_mesh_for_job(job: &ChunkMeshJob, max_triangles: Option<usize>) -> Vec<Triangle> {
    match max_triangles {
        Some(limit) => build_chunk_mesh_from_job_limited(job, limit),
        None => build_chunk_mesh_from_job(job),
    }
}

impl AsyncMesher {
    pub fn new() -> Self {
        Self::with_max_triangles(capabilities().max_chunk_mesh_triangles)
    }

    pub fn with_max_triangles(max_triangles: Option<usize>) -> Self {
        let (req_tx, req_rx) = std::sync::mpsc::channel::<(u64, ChunkMeshJob)>();
        let (res_tx, res_rx) = std::sync::mpsc::channel::<(u64, (i32, i32), Vec<Triangle>)>();

        let req_rx = std::sync::Arc::new(std::sync::Mutex::new(req_rx));
        let num_threads = worker_thread_count(1, 4, 1);

        let mut worker_count = 0usize;
        for i in 0..num_threads {
            let req_rx_clone = std::sync::Arc::clone(&req_rx);
            let res_tx_clone = res_tx.clone();
            let spawn_result = std::thread::Builder::new()
                .name(format!("chunk_mesh_{}", i))
                .spawn(move || {
                    configure_worker_thread_affinity(i, true);

                    loop {
                        let task = if let Ok(rx) = req_rx_clone.lock() {
                            rx.recv().ok()
                        } else {
                            None
                        };
                        match task {
                            Some((job_gen, job)) => {
                                let key = job.key;
                                let mesh = build_mesh_for_job(&job, max_triangles);
                                if res_tx_clone.send((job_gen, key, mesh)).is_err() {
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                });
            if spawn_result.is_ok() {
                worker_count += 1;
            }
        }

        Self {
            req_tx,
            res_rx,
            res_tx,
            worker_count,
            max_triangles,
            generation: 0,
        }
    }

    pub fn submit_job(&self, job: ChunkMeshJob) -> bool {
        if self.worker_count == 0 {
            let key = job.key;
            let mesh = build_mesh_for_job(&job, self.max_triangles);
            self.res_tx.send((self.generation, key, mesh)).is_ok()
        } else {
            self.req_tx.send((self.generation, job)).is_ok()
        }
    }

    pub fn drain_completed(&self) -> Vec<((i32, i32), Vec<Triangle>)> {
        let mut completed = Vec::new();
        while let Ok((job_gen, key, mesh)) = self.res_rx.try_recv() {
            if job_gen == self.generation {
                completed.push((key, mesh));
            }
        }
        completed
    }

    pub fn invalidate_pending(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        while self.res_rx.try_recv().is_ok() {}
    }

    /// Creates a zero-thread inert mesher when 3D world rendering uses an external GPU pipeline.
    pub fn disabled() -> Self {
        let (req_tx, _req_rx) = std::sync::mpsc::channel::<(u64, ChunkMeshJob)>();
        let (res_tx, res_rx) = std::sync::mpsc::channel::<(u64, (i32, i32), Vec<Triangle>)>();
        Self {
            req_tx,
            res_rx,
            res_tx,
            worker_count: 0,
            max_triangles: None,
            generation: 0,
        }
    }
}
