//! Specialized non-cube voxel geometry emitters organized by block family.
//! Re-exports block, foliage, and redstone mesh builders exclusively inside `features::meshing`.

mod blocks;
mod foliage;
mod redstone;

pub(super) use blocks::{emit_bed, emit_door, emit_ladder, emit_slab, emit_stairs, emit_torch};
pub(super) use foliage::{emit_cross, emit_waterlogged_faces};
pub(super) use redstone::{emit_button, emit_lever, emit_pressure_plate, emit_ruststone_wire};
