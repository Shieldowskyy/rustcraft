//! User configuration, graphics options, RAM budget limits, and persistent serialization.

mod options;
mod persistence;
mod types;

#[cfg(test)]
mod tests;

pub use persistence::round_to_5_pct;
pub use types::{
    apply_hard_ram_limit, generate_default_nickname, max_ram_mb, max_render_distance,
    set_max_ram_mb, GameSettings, JumpMode, LeafQuality, SneakMode, SprintMode,
};

