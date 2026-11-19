//! Voxel-aware acoustic occlusion and room reverb estimators.
//! Samples world block absorption/reflection properties along DDA rays around the listener.

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::world::World;

use super::raycast::traverse_voxels;

/// Calculates sound attenuation (`0.0..=1.0`) between `listener` and `emitter`.
pub fn calculate_acoustic_occlusion(world: &World, listener: Vec3, emitter: Vec3) -> f32 {
    let delta = emitter - listener;
    let dist = delta.length();
    if dist <= 0.6 {
        return 0.0;
    }

    let start_v = listener.floor_to_ivec();
    let end_v = emitter.floor_to_ivec();
    let mut occlusion_accum = 0.0f32;
    let mut steps = 0;

    traverse_voxels(listener, delta * (1.0 / dist), dist, |pos, _, traveled| {
        if traveled >= dist || steps >= 64 {
            return Some(());
        }
        steps += 1;
        if pos != start_v && pos != end_v {
            let block = world.get_block(pos);
            if block != BlockId::Air {
                occlusion_accum += match block {
                    BlockId::Glass => 0.15,
                    BlockId::Water => 0.25,
                    BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves | BlockId::SnowLeaves => 0.12,
                    BlockId::Wood | BlockId::Planks | BlockId::Door | BlockId::DoorOpen => 0.35,
                    _ => 0.50,
                };
                if occlusion_accum >= 1.0 {
                    return Some(());
                }
            }
        }
        None
    });
    occlusion_accum.min(1.0)
}

/// Estimates room reverb (`(wet_mix, rt60_seconds)`) from nearby solid surface materials.
pub fn calculate_acoustic_reverb(world: &World, listener: Vec3) -> (f32, f32) {
    const MAX_DISTANCE: i32 = 12;
    const DIRECTIONS: [(f32, f32, f32); 10] = [
        (1.0, 0.0, 0.0),
        (-1.0, 0.0, 0.0),
        (0.0, 0.0, 1.0),
        (0.0, 0.0, -1.0),
        (0.707, 0.0, 0.707),
        (-0.707, 0.0, 0.707),
        (0.707, 0.0, -0.707),
        (-0.707, 0.0, -0.707),
        (0.0, 1.0, 0.0),
        (0.0, -1.0, 0.0),
    ];

    let mut wall_count = 0;
    let mut distance_sum = 0.0;
    let mut reflection_sum = 0.0;

    for (dx, dy, dz) in DIRECTIONS {
        for distance in 1..=MAX_DISTANCE {
            let pos = IVec3::new(
                (listener.x + dx * distance as f32).floor() as i32,
                (listener.y + dy * distance as f32).floor() as i32,
                (listener.z + dz * distance as f32).floor() as i32,
            );
            let block = world.get_block(pos);
            if !block.is_solid() {
                continue;
            }

            let reflection = if block.is_leaves() {
                0.45
            } else if block.is_wood()
                || matches!(
                    block,
                    BlockId::Planks
                        | BlockId::CraftingTable
                        | BlockId::Chest
                        | BlockId::Door
                        | BlockId::WoodenSlab
                        | BlockId::WoodenStairs
                )
            {
                0.68
            } else if block == BlockId::Glass {
                0.9
            } else {
                0.88
            };
            wall_count += 1;
            distance_sum += distance as f32;
            reflection_sum += reflection;
            break;
        }
    }

    if wall_count == 0 {
        return (0.0, 0.35);
    }

    let enclosure = wall_count as f32 / DIRECTIONS.len() as f32;
    let room_scale = (distance_sum / wall_count as f32 / MAX_DISTANCE as f32).clamp(0.0, 1.0);
    let surface_reflection = reflection_sum / wall_count as f32;
    let wet = (enclosure * surface_reflection * (0.22 + 0.42 * room_scale)).clamp(0.0, 0.55);
    let decay_seconds = (0.35 + room_scale * 2.7).clamp(0.35, 3.5);
    (wet, decay_seconds)
}
