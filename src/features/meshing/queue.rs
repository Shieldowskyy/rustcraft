//! Background chunk meshing worker pool (`AsyncMesher`) built on PAL threading hooks.
//! Falls back seamlessly to synchronous meshing when `worker_thread_count` returns 0 (e.g. WASM).

use std::sync::mpsc::{Receiver, Sender};

use crate::platform::threading::{configure_worker_thread_affinity, worker_thread_count};

use super::constants::DEFAULT_MAX_MESH_TRIANGLES;
use super::culling::build_chunk_mesh_from_job_limited;
use super::types::{ChunkMeshJob, Triangle};

pub struct AsyncMesher {
    req_tx: Sender<ChunkMeshJob>,
    res_rx: Receiver<((i32, i32), Vec<Triangle>)>,
    res_tx: Sender<((i32, i32), Vec<Triangle>)>,
    worker_count: usize,
}

impl Default for AsyncMesher {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncMesher {
    pub fn new() -> Self {
        let (req_tx, req_rx) = std::sync::mpsc::channel::<ChunkMeshJob>();
        let (res_tx, res_rx) = std::sync::mpsc::channel::<((i32, i32), Vec<Triangle>)>();

        let req_rx = std::sync::Arc::new(std::sync::Mutex::new(req_rx));
        let num_threads = worker_thread_count(1, 4, 2);

        let mut worker_count = 0usize;
        for i in 0..num_threads {
            let req_rx_clone = std::sync::Arc::clone(&req_rx);
            let res_tx_clone = res_tx.clone();
            let spawn_result = std::thread::Builder::new()
                .name(format!("chunk_mesh_{}", i))
                .spawn(move || {
                    configure_worker_thread_affinity(i, true);

                    loop {
                        let job = if let Ok(rx) = req_rx_clone.lock() {
                            rx.recv().ok()
                        } else {
                            None
                        };
                        match job {
                            Some(job) => {
                                let key = job.key;
                                let mesh = build_chunk_mesh_from_job_limited(&job, DEFAULT_MAX_MESH_TRIANGLES);
                                if res_tx_clone.send((key, mesh)).is_err() {
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

        Self { req_tx, res_rx, res_tx, worker_count }
    }

    pub fn submit_job(&self, job: ChunkMeshJob) -> bool {
        if self.worker_count == 0 {
            let key = job.key;
            let mesh = build_chunk_mesh_from_job_limited(&job, DEFAULT_MAX_MESH_TRIANGLES);
            self.res_tx.send((key, mesh)).is_ok()
        } else {
            self.req_tx.send(job).is_ok()
        }
    }

    pub fn drain_completed(&self) -> Vec<((i32, i32), Vec<Triangle>)> {
        let mut completed = Vec::new();
        while let Ok(res) = self.res_rx.try_recv() {
            completed.push(res);
        }
        completed
    }
}
