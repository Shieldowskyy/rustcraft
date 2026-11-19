//! Mob domain facade (`MobKind`, `Mob`, `MobManager`, `MobBox`, `emit_mob_boxes`).
//! Pure simulation & geometric model emission with zero platform `#[cfg]`.

mod ai;
mod ai_environment;
mod combat;
mod constants;
mod drops;
mod kind;
mod manager;
mod models;
mod models_hostile;
mod models_passive;
mod spawning;
mod state;

#[cfg(test)]
mod tests;

pub use kind::MobKind;
pub use manager::{MobDrop, MobManager, MobPlayerContext};
pub use models::{emit_mob_boxes, MobBox};
pub use state::Mob;
