//! Pure CPU voxel meshing domain facade (no GPU dependencies, zero platform `#[cfg]`).
//! Exposes vertex/triangle types, `ChunkNeighborCache`, `AsyncMesher`, and chunk/subchunk mesh builders.

mod cache;
pub mod constants;
mod culling;
mod emit;
mod queue;
mod shapes;
mod types;

pub use cache::ChunkNeighborCache;
pub use constants::{NUM_SUBCHUNKS, SUBCHUNK_SIZE};
pub use culling::{
    build_chunk_mesh, build_chunk_mesh_from_job, build_chunk_mesh_from_job_limited,
    build_subchunk_mesh, build_subchunk_mesh_from_cache, build_subchunk_mesh_internal,
};
pub use queue::AsyncMesher;
pub use types::{ChunkMeshJob, LightSource, Triangle, Vertex};
