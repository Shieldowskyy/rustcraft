//! Transitional compatibility facade preserving legacy `crate::world::*` paths.

pub mod block;
pub mod mesher;
pub mod storage;
pub mod tile_entity;
pub mod weather;

pub use block::BlockId;
pub use crate::features::world::{
    fuel_burn_time, get_biome_at, get_height_at, smelting_result, Biome, ChestData, Chunk,
    FurnaceData, LightningStrike, PrecipitationKind, PrimedTnt, VillageComponent, WeatherState,
    WeatherType, World, WorldGeneratorType, WorldType, CARDINAL_OFFSETS, CHUNK_SIZE_X,
    CHUNK_SIZE_Y, CHUNK_SIZE_Z, DEFAULT_WORLD_SEED, MAX_BLOCK_LIGHT_RADIUS, SMELT_DURATION,
    VIEW_DISTANCE,
};
pub use mesher::{
    build_chunk_mesh, build_chunk_mesh_from_job, build_subchunk_mesh, AsyncMesher, ChunkMeshJob,
    LightSource, Triangle, Vertex, NUM_SUBCHUNKS, SUBCHUNK_SIZE,
};
pub use storage::{save_current_world, save_path};
