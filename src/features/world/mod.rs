//! Voxel world domain facade (`World`, `Chunk`, biomes, generation, streaming, lighting, simulation, weather, tile entities, storage).
//! Zero platform `#[cfg]` inside `features::world`; worker threading & storage paths delegate to `platform`.

mod biome;
mod blocks_access;
mod chunk;
mod codec;
pub mod constants;
pub mod gen;
mod lighting;
mod redstone;
pub mod save_meta;
mod simulation;
pub mod storage;
mod streaming;
pub mod tile_entity;
pub mod weather;
mod world_state;

#[cfg(test)]
mod tests_simulation;
#[cfg(test)]
mod tests_world;
#[cfg(test)]
mod weather_tests;

pub use biome::{Biome, VillageComponent, WorldGeneratorType, WorldType};
pub use chunk::Chunk;
pub use constants::{
    CARDINAL_OFFSETS, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, DEFAULT_WORLD_SEED,
    MAX_BLOCK_LIGHT_RADIUS, VIEW_DISTANCE,
};
pub use gen::{get_biome_at, get_height_at};
pub use save_meta::{get_save_metadata, SaveMetadata};
pub use storage::{
    achievement_progress_path, app_files_dir, capture_thumbnail, delete_save,
    load_achievement_progress, load_quest_progress, load_thumbnail, quest_progress_path,
    save_achievement_progress, save_current_world, save_path, save_quest_progress, save_thumbnail,
    set_custom_save_dir, settings_file_path, soundtracks_dir, thumbnail_path, THUMB_SIZE,
};
pub use tile_entity::{fuel_burn_time, smelting_result, ChestData, FurnaceData, SMELT_DURATION};
pub use weather::{LightningStrike, PrecipitationKind, WeatherState, WeatherType};
pub use world_state::{PrimedTnt, World};
