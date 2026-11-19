//! Shared numeric constants for block IDs, custom mod ranges, and base combat damage.
//! Consumed across `blocks/` specs, tools, and tests without magic literals.

/// Total number of built-in vanilla block/item IDs (`0..=185`).
pub const VANILLA_BLOCK_COUNT: usize = 186;

/// First numerical block ID reserved for dynamically registered custom mod blocks.
pub const CUSTOM_BLOCK_ID_START: u8 = 186;

/// Base unarmed melee attack damage.
pub const DEFAULT_BARE_HAND_DAMAGE: f32 = 1.5;

/// Break time assigned to any dyed wool variant.
pub const WOOL_BREAK_TIME_SECS: f32 = 0.8;
