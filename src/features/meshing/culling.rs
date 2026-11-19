//! Pure CPU chunk & subchunk face-culling mesh builders (`build_subchunk_mesh*`, `build_chunk_mesh*`).
//! Emits opaque/cutout geometry followed by translucent geometry for chunks or worker `ChunkMeshJob`s.

use crate::core::math::IVec3;
use crate::features::blocks::{BlockId, Face};
use crate::features::world::{Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Z};

use super::cache::ChunkNeighborCache;
use super::constants::{NUM_SUBCHUNKS, SUBCHUNK_SIZE};
use super::emit::emit_face;
use super::shapes::{
    emit_bed, emit_button, emit_cross, emit_door, emit_ladder, emit_lever, emit_pressure_plate,
    emit_ruststone_wire, emit_slab, emit_stairs, emit_torch, emit_waterlogged_faces,
};
use super::types::{ChunkMeshJob, Triangle};

pub fn build_subchunk_mesh_from_cache(
    sub_y: usize,
    cache: &ChunkNeighborCache,
) -> (Vec<Triangle>, Vec<Triangle>) {
    let y_start = (sub_y as i32) * SUBCHUNK_SIZE;
    let y_end = y_start + SUBCHUNK_SIZE;

    // --- OPTYMALIZACJA: Fast-path dla pustych subchunków ---
    // Sprawdzamy, czy maksymalna wysokość terenu w tym subchunku jest niższa niż y_start.
    // Jeśli tak, to w tym przedziale Y nie ma żadnych pełnych bloków ziemi/kamienia.
    let mut max_h_in_subchunk = 0u8;
    for lx in 0..16 {
        for lz in 0..16 {
            let idx = ((lx + 1) * 18 + (lz + 1)) as usize;
            let h = cache.heightmap[idx];
            if h > max_h_in_subchunk {
                max_h_in_subchunk = h;
            }
        }
    }
    // Jeśli cały teren jest poniżej tego subchunka oraz brak świateł/płynów mogących generować geometrię,
    // możemy pominąć całą iterację po blokach (zabezpieczenie: dla sub_y > 0 sprawdzamy czy y_start >= max_h_in_subchunk).
    if (y_start as u8) >= max_h_in_subchunk && cache.lights.is_empty() {
        // Dodatkowo upewniamy się, że nie ma tam płynów (np. oceanu powyżej dna)
        // Jeśli fluid_map jest pusta, możemy bezpiecznie zwrócić puste wektory.
        if cache.fluid_map.map_or(true, |fm| fm.is_empty()) {
            return (Vec::new(), Vec::new());
        }
    }
    // ------------------------------------------------------

    let mut solid_tris = Vec::with_capacity(384);
    let mut trans_tris = Vec::with_capacity(64);

    let base_x = cache.base_x;
    let base_z = cache.base_z;
    let blocks = cache.center_blocks;

    for lx in 0..CHUNK_SIZE_X {
        for ly in y_start..y_end {
            for lz in 0..CHUNK_SIZE_Z {
                let block_u8 = blocks[((lx << 10) | (ly << 4) | lz) as usize];
                if block_u8 == 0 {
                    continue;
                }
                let block = BlockId::from_u8(block_u8);
                let wx = base_x + lx;
                let wy = ly;
                let wz = base_z + lz;

                if block == BlockId::Torch {
                    emit_torch(&mut solid_tris, wx, wy, wz, cache);
                    continue;
                }
                if block == BlockId::Door {
                    emit_door(&mut solid_tris, wx, wy, wz, false, cache);
                    continue;
                }
                if block == BlockId::DoorOpen {
                    emit_door(&mut solid_tris, wx, wy, wz, true, cache);
                    continue;
                }
                if block == BlockId::Bed {
                    emit_bed(&mut solid_tris, wx, wy, wz, cache);
                    continue;
                }
                if block == BlockId::WoodenSlab || block == BlockId::StoneSlab {
                    emit_slab(&mut solid_tris, wx, wy, wz, block, cache);
                    continue;
                }
                if block == BlockId::WoodenStairs || block == BlockId::StoneStairs {
                    emit_stairs(&mut solid_tris, wx, wy, wz, block, cache);
                    continue;
                }
                if block == BlockId::Ladder {
                    emit_ladder(&mut solid_tris, wx, wy, wz, cache);
                    continue;
                }
                if block == BlockId::RustStone || block == BlockId::RustStoneWireLit {
                    emit_ruststone_wire(&mut solid_tris, wx, wy, wz, block == BlockId::RustStoneWireLit, cache);
                    continue;
                }
                if block == BlockId::Lever || block == BlockId::LeverOn {
                    emit_lever(&mut solid_tris, wx, wy, wz, block == BlockId::LeverOn, cache);
                    continue;
                }
                if block == BlockId::Button || block == BlockId::ButtonPressed {
                    emit_button(&mut solid_tris, wx, wy, wz, block == BlockId::ButtonPressed, cache);
                    continue;
                }
                if block == BlockId::PressurePlate || block == BlockId::PressurePlatePressed {
                    emit_pressure_plate(&mut solid_tris, wx, wy, wz, block == BlockId::PressurePlatePressed, cache);
                    continue;
                }
                if matches!(block, BlockId::WheatCrop | BlockId::CarrotCrop | BlockId::PotatoCrop | BlockId::Sapling | BlockId::Kelp | BlockId::Seagrass) {
                    emit_cross(&mut solid_tris, wx, wy, wz, block.texture_id(Face::Top), cache);
                    if matches!(block, BlockId::Kelp | BlockId::Seagrass) {
                        emit_waterlogged_faces(&mut trans_tris, wx, wy, wz, lx, ly, lz, cache);
                    }
                    continue;
                }

                let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);
                let is_trans = matches!(block, BlockId::Water | BlockId::Glass);

                for face in Face::ALL {
                    let n = face.normal();
                    let neighbor = cache.sample_block(lx + n.x, ly + n.y, lz + n.z);

                    let visible = if is_fluid {
                        let neighbor_loaded = match face {
                            Face::East => lx < CHUNK_SIZE_X - 1 || cache.east_blocks.is_some(),
                            Face::West => lx > 0 || cache.west_blocks.is_some(),
                            Face::South => lz < CHUNK_SIZE_Z - 1 || cache.south_blocks.is_some(),
                            Face::North => lz > 0 || cache.north_blocks.is_some(),
                            Face::Top | Face::Bottom => true,
                        };

                        if !neighbor_loaded {
                            false
                        } else {
                            let same_fluid = if block == BlockId::Water {
                                matches!(neighbor, BlockId::Water | BlockId::Kelp | BlockId::Seagrass)
                            } else {
                                neighbor == block
                            };

                            match face {
                                Face::Top => !same_fluid,
                                Face::Bottom => wy > 0 && !same_fluid && (neighbor == BlockId::Air || (neighbor.is_transparent() && !same_fluid)),
                                _ => {
                                    if !same_fluid {
                                        neighbor == BlockId::Air || (neighbor.is_transparent() && !same_fluid)
                                    } else {
                                        let above_neighbor = cache.sample_block(lx + n.x, ly + 1, lz + n.z);
                                        let above_same_fluid = if block == BlockId::Water {
                                            matches!(above_neighbor, BlockId::Water | BlockId::Kelp | BlockId::Seagrass)
                                        } else {
                                            above_neighbor == block
                                        };
                                        if !above_same_fluid {
                                            let my_lvl = cache.fluid_level(IVec3::new(wx, wy, wz));
                                            let neighbor_lvl = cache.fluid_level(IVec3::new(wx + n.x, wy, wz + n.z));
                                            my_lvl > neighbor_lvl
                                        } else {
                                            false
                                        }
                                    }
                                }
                            }
                        }
                    } else if face == Face::Bottom && wy == 0 {
                        false
                    } else {
                        neighbor == BlockId::Air
                        || (neighbor.is_transparent() && neighbor != block)
                    };

                    if visible {
                        if is_trans {
                            emit_face(&mut trans_tris, wx, wy, wz, face, block, cache);
                        } else {
                            emit_face(&mut solid_tris, wx, wy, wz, face, block, cache);
                        }
                    }
                }
            }
        }
    }

    (solid_tris, trans_tris)
}

pub fn build_subchunk_mesh_internal(
    _chunk: &Chunk,
    _world: &World,
    sub_y: usize,
    cache: &ChunkNeighborCache,
) -> (Vec<Triangle>, Vec<Triangle>) {
    build_subchunk_mesh_from_cache(sub_y, cache)
}

pub fn build_subchunk_mesh(chunk: &Chunk, world: &World, sub_y: usize) -> Vec<Triangle> {
    if sub_y >= NUM_SUBCHUNKS {
        return Vec::new();
    }
    let cache = ChunkNeighborCache::new(chunk, world);
    let (mut solid, trans) = build_subchunk_mesh_from_cache(sub_y, &cache);
    solid.extend(trans);
    solid
}

fn build_chunk_mesh_from_cache(cache: &ChunkNeighborCache) -> Vec<Triangle> {
    let mut solid_tris = Vec::with_capacity(3072);
    let mut trans_tris = Vec::with_capacity(512);

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = build_subchunk_mesh_from_cache(sub_y, cache);
        solid_tris.extend(sub_solid);
        trans_tris.extend(sub_trans);
    }

    solid_tris.extend(trans_tris);
    solid_tris
}

pub fn build_chunk_mesh(chunk: &Chunk, world: &World) -> Vec<Triangle> {
    let cache = ChunkNeighborCache::new(chunk, world);
    build_chunk_mesh_from_cache(&cache)
}

pub fn build_chunk_mesh_from_job(job: &ChunkMeshJob) -> Vec<Triangle> {
    let cache = ChunkNeighborCache::from_job(job);
    build_chunk_mesh_from_cache(&cache)
}

pub fn build_chunk_mesh_from_job_limited(job: &ChunkMeshJob, max_triangles: usize) -> Vec<Triangle> {
    if max_triangles == 0 {
        return Vec::new();
    }
    let cache = ChunkNeighborCache::from_job(job);
    let mut solid_tris = Vec::with_capacity(3072.min(max_triangles));
    let mut trans_tris = Vec::with_capacity(512.min(max_triangles));

    for sub_y in 0..NUM_SUBCHUNKS {
        let (sub_solid, sub_trans) = build_subchunk_mesh_from_cache(sub_y, &cache);
        if solid_tris.len() < max_triangles {
            let remaining = max_triangles - solid_tris.len();
            solid_tris.extend(sub_solid.into_iter().take(remaining));
        }
        if solid_tris.len() >= max_triangles {
            return solid_tris;
        }
        if trans_tris.len() < max_triangles {
            let remaining = max_triangles - trans_tris.len();
            trans_tris.extend(sub_trans.into_iter().take(remaining));
        }
    }

    trans_tris.truncate(max_triangles.saturating_sub(solid_tris.len()));
    solid_tris.extend(trans_tris);
    solid_tris
}
