//! Procedural 16x16 texture atlas facade (`Texture`, `TextureId`, `TextureAtlas`, `TEX_SIZE`).
//! Exposes `222` built-in texture layers + `34` mod layers (`256` total) with zero platform `#[cfg]`.

mod blocks_containers;
mod blocks_craft;
mod farming;
mod foliage;
mod items_food;
mod items_gear;
mod mobs_hostile;
mod mobs_npcs;
mod mobs_passive;
mod ores_blocks;
mod registry;
mod ruststone;
mod spawn_eggs;
mod terrain;
mod terrain_mineral;
mod types;

#[cfg(test)]
mod tests;

pub use registry::{
    TextureAtlas, TextureId, TextureRenderPass, BUILTIN_TEXTURE_COUNT, CUSTOM_TEXTURE_LAYERS,
};
pub use types::{downsample_texels_rgba, texel_to_rgba8, Texture, TEX_SIZE};
