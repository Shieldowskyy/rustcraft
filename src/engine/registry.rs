//! Transitional compatibility facade re-exporting `BlockProperties` and `BlockRegistry`.
//! Full block registry metadata now lives in `crate::features::blocks`.

pub use crate::features::blocks::{BlockProperties, BlockRegistry};
