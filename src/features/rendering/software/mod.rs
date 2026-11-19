//! Software CPU rasterizer & hybrid `GpuBridge` 3D/2D renderer facade.
//! Zero platform `#[cfg]` and zero per-pixel heap allocations in hot loops.

mod clip;
mod entities;
mod held_items;
mod items_3d;
mod overlays;
mod player_model;
mod raster;
mod sky;
mod state;
mod types;
mod ui_bridge;
mod weather;
mod world;

#[cfg(test)]
mod tests;

pub use types::{Particle, Renderer};
