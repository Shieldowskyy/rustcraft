//! World entities domain facade (`ItemEntity`, `ArrowEntity`, `ParticleSystem`, `Mob`, `MobManager`).
//! Strictly isolated from platform-specific `#[cfg]` code.

pub mod item;
pub mod mob;
pub mod particles;

pub use item::{
    emit_arrow_boxes, emit_feather_boxes, emit_meat_boxes, emit_procedural_tool_boxes,
    emit_torch_boxes, ArrowEntity, ItemEntity,
};
pub use mob::{
    emit_mob_boxes, Mob, MobBox, MobDrop, MobKind, MobManager, MobPlayerContext,
};
pub use particles::{Particle, ParticleSystem};
