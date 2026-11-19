//! Platform threading, rayon parallel dispatch, and PS Vita CPU core affinity helpers.
//! Keeps all `rayon` / Vita `sceKernel*` / single-threaded WASM branching out of `features/`.
//! Inputs: slices or worker indices; Side effects: configures global thread pool or thread affinity.

/// Initializes the global parallel worker pool on targets supporting `rayon`.
pub fn init_global_thread_pool() {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_os = "xbox360"),
        not(target_os = "xenon"),
        not(target_arch = "powerpc"),
        not(feature = "xbox360")
    ))]
    {
        let logical_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let worker_threads = if logical_threads <= 4 {
            (logical_threads / 2).max(1)
        } else {
            logical_threads
        };
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(worker_threads)
            .build_global();
    }
}

/// Returns active worker thread count used by parallel rasterization / UI passes.
#[inline]
pub fn active_raster_threads() -> usize {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_arch = "powerpc"),
        not(feature = "xbox360")
    ))]
    {
        rayon::current_num_threads()
    }
    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
    {
        1
    }
}

/// Returns whether background chunk generation/meshing threads should be spawned.
#[inline]
pub const fn supports_background_workers() -> bool {
    !cfg!(any(
        target_arch = "wasm32",
        target_os = "xbox360",
        target_os = "xenon",
        target_arch = "powerpc",
        feature = "xbox360"
    ))
}

/// Returns recommended worker thread count for chunk meshing or chunk generation.
pub fn worker_thread_count(min: usize, max: usize, vita_count: usize) -> usize {
    #[cfg(target_os = "vita")]
    {
        let _ = (min, max);
        vita_count
    }
    #[cfg(any(target_arch = "wasm32", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
    {
        let _ = (min, max, vita_count);
        0
    }
    #[cfg(not(any(
        target_os = "vita",
        target_arch = "wasm32",
        target_os = "xbox360",
        target_os = "xenon",
        target_arch = "powerpc",
        feature = "xbox360"
    )))]
    {
        let _ = vita_count;
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(min.max(2))
            .clamp(min, max)
    }
}

/// Returns active software rasterizer thread count (`rayon::current_num_threads()` where enabled, `1` on Vita/WASM).
pub fn raster_thread_count() -> usize {
    #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
    {
        rayon::current_num_threads()
    }
    #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
    {
        1
    }
}

/// Pins background worker threads to non-main user cores on Sony PS Vita.
pub fn configure_worker_thread_affinity(worker_idx: usize, is_mesher: bool) {
    #[cfg(target_os = "vita")]
    unsafe {
        extern "C" {
            fn sceKernelGetThreadId() -> i32;
            fn sceKernelChangeThreadCpuAffinityMask(thid: i32, bitmask: i32) -> i32;
        }
        // DLACZEGO: Main thread runs on user cores (mask 0x7 / core 0 priority); pinning
        // background chunk workers to cores 1 and 2 prevents frame-time stalls on PS Vita.
        let mask = if is_mesher {
            if worker_idx % 2 == 0 { 0x0002_0000 } else { 0x0004_0000 }
        } else if worker_idx % 2 == 0 {
            0x2
        } else {
            0x4
        };
        let _ = sceKernelChangeThreadCpuAffinityMask(sceKernelGetThreadId(), mask);
    }
    #[cfg(not(target_os = "vita"))]
    {
        let _ = (worker_idx, is_mesher);
    }
}

/// Pins a background audio/soundtrack decoding thread to idle user cores 1 & 2 on PS Vita.
pub fn configure_background_decoder_affinity() {
    #[cfg(target_os = "vita")]
    unsafe {
        extern "C" {
            fn sceKernelChangeThreadCpuAffinityMask(thid: i32, mask: i32) -> i32;
        }
        // DLACZEGO: Main game loop runs on Core 0 (0x00010000); moving background Opus/Vorbis
        // decoding to cores 1 & 2 (0x00060000) prevents audio decode starvation and frame stalls.
        let _ = sceKernelChangeThreadCpuAffinityMask(0, 0x0006_0000);
    }
}

/// Executes `f(chunk_index, chunk_slice)` in parallel on multi-core hosts, sequentially on Vita/WASM.
pub fn parallel_for_chunks_mut<T, F>(slice: &mut [T], chunk_size: usize, f: F)
where
    T: Send,
    F: Fn(usize, &mut [T]) + Sync + Send,
{
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_arch = "powerpc"),
        not(feature = "xbox360")
    ))]
    {
        use rayon::prelude::*;
        slice
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(idx, chunk)| f(idx, chunk));
    }
    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
    {
        for (idx, chunk) in slice.chunks_mut(chunk_size).enumerate() {
            f(idx, chunk);
        }
    }
}

/// Filters and maps `items` in parallel when `rayon` is active, sequentially otherwise.
pub fn parallel_filter_map<T, R, F>(items: &[T], f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> Option<R> + Sync + Send,
{
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_arch = "powerpc"),
        not(feature = "xbox360")
    ))]
    {
        use rayon::prelude::*;
        items.par_iter().filter_map(f).collect()
    }
    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
    {
        items.iter().filter_map(f).collect()
    }
}

/// Maps `items` into `(Vec<A>, Vec<B>)` in parallel on multi-core hosts and flattens both into output buffers.
pub fn parallel_map_unzip_flatten<T, A, B, F>(
    items: &[T],
    out_a: &mut Vec<A>,
    out_b: &mut Vec<B>,
    f: F,
) where
    T: Sync,
    A: Send,
    B: Send,
    F: Fn(&T) -> (Vec<A>, Vec<B>) + Sync + Send,
{
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_arch = "powerpc"),
        not(feature = "xbox360")
    ))]
    {
        use rayon::prelude::*;
        let (chunks_a, chunks_b): (Vec<Vec<A>>, Vec<Vec<B>>) = items.par_iter().map(f).unzip();
        *out_a = chunks_a.into_par_iter().flatten().collect();
        *out_b = chunks_b.into_par_iter().flatten().collect();
    }
    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
    {
        out_a.clear();
        out_b.clear();
        for item in items {
            let (mut a, mut b) = f(item);
            out_a.append(&mut a);
            out_b.append(&mut b);
        }
    }
}

/// Executes `f(tile_idx, chunk_a, chunk_b)` across paired mutable slices in parallel on multi-core hosts.
pub fn parallel_for_two_chunks_mut<A, B, F>(
    slice_a: &mut [A],
    slice_b: &mut [B],
    chunk_size: usize,
    f: F,
) where
    A: Send,
    B: Send,
    F: Fn(usize, &mut [A], &mut [B]) + Sync + Send,
{
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "vita"),
        not(target_arch = "powerpc"),
        not(feature = "xbox360")
    ))]
    {
        use rayon::prelude::*;
        slice_a
            .par_chunks_mut(chunk_size)
            .zip(slice_b.par_chunks_mut(chunk_size))
            .enumerate()
            .for_each(|(idx, (ca, cb))| f(idx, ca, cb));
    }
    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
    {
        for (idx, (ca, cb)) in slice_a
            .chunks_mut(chunk_size)
            .zip(slice_b.chunks_mut(chunk_size))
            .enumerate()
        {
            f(idx, ca, cb);
        }
    }
}

