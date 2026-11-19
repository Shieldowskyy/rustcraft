//! Transitional compatibility facade re-exporting `crate::features::blocks`.
//! Allows existing callers of `crate::world::block::*` to remain unbroken during domain migration.

pub use crate::features::blocks::{ArmorSlot, BlockId, Face};
