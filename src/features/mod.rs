//! Core gameplay domains strictly isolated from platform-specific `#[cfg]` code.
//! Re-exports refactored voxel engine modules (`audio`, `autotest`, `blocks`, `entities`, `meshing`, `modding`, `net`, `physics`, `player`, `rendering`, `ui`, `world`).

pub mod audio;
pub mod autotest;
pub mod blocks;
pub mod entities;
pub mod meshing;
pub mod modding;
pub mod net;
pub mod physics;
pub mod player;
pub mod rendering;
pub mod ui;
pub mod world;
