//! Voxel blocks & items domain facade.
//! Exposes `BlockId`, `Face`, `ArmorSlot`, `BlockSpec`, `BlockProperties`, and `BlockRegistry`.

mod colors;
pub mod constants;
mod id;
mod registry;
mod spec;
mod textures;
mod tools;

#[cfg(test)]
mod tests;

pub use constants::{CUSTOM_BLOCK_ID_START, VANILLA_BLOCK_COUNT};
pub use id::{ArmorSlot, BlockId, Face};
pub use registry::{BlockProperties, BlockRegistry};
pub use spec::{BlockSpec, MiningCategory};
