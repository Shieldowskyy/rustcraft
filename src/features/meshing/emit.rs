//! Low-level CCW quad emission, per-vertex AO calculation, smooth light averaging, and face emitter.
//! Enforces counter-clockwise winding viewed from outside the voxel and AO diagonal flipping.

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::{BlockId, Face};
use crate::features::rendering::atlas::TextureId;
use crate::features::world::Biome;

use super::cache::ChunkNeighborCache;
use super::types::{Triangle, Vertex};

#[inline(always)]
pub(super) fn push_quad_uv(
    tris: &mut Vec<Triangle>,
    a: (Vec3, (f32, f32)),
    b: (Vec3, (f32, f32)),
    c: (Vec3, (f32, f32)),
    d: (Vec3, (f32, f32)),
    texture: TextureId,
    ao: f32,
    sky_light: f32,
    block_light: f32,
) {
    let va = Vertex::lit(a.0, a.1, ao, sky_light, block_light);
    let vb = Vertex::lit(b.0, b.1, ao, sky_light, block_light);
    let vc = Vertex::lit(c.0, c.1, ao, sky_light, block_light);
    let vd = Vertex::lit(d.0, d.1, ao, sky_light, block_light);
    tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
    tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
}

#[inline(always)]
pub(super) fn push_quad(
    tris: &mut Vec<Triangle>,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
    texture: TextureId,
    ao: f32,
    sky_light: f32,
    block_light: f32,
) {
    push_quad_uv(
        tris,
        (a, (0.0, 1.0)),
        (b, (1.0, 1.0)),
        (c, (1.0, 0.0)),
        (d, (0.0, 0.0)),
        texture,
        ao,
        sky_light,
        block_light,
    );
}


#[inline(always)]
pub(super) fn vertex_ao(side1: bool, side2: bool, corner: bool) -> f32 {
    let count = side1 as u8 + side2 as u8 + (corner && !(side1 && side2)) as u8;
    match count {
        0 => 1.00,
        1 => 0.80,
        2 => 0.65,
        _ => 0.50,
    }
}

#[inline(always)]
pub(super) fn vertex_light_and_ao(
    center_sky: f32,
    center_block: f32,
    s1: (bool, f32, f32),
    s2: (bool, f32, f32),
    corner: (bool, f32, f32),
) -> (f32, f32, f32) {
    let ao = vertex_ao(s1.0, s2.0, corner.0);
    if s1.0 && s2.0 {
        (ao, center_sky, center_block)
    } else {
        let mut count = 1.0f32;
        let mut sum_sky = center_sky;
        let mut sum_block = center_block;

        if !s1.0 {
            sum_sky += s1.1;
            sum_block += s1.2;
            count += 1.0;
        }
        if !s2.0 {
            sum_sky += s2.1;
            sum_block += s2.2;
            count += 1.0;
        }
        if !corner.0 && (!s1.0 || !s2.0) {
            sum_sky += corner.1;
            sum_block += corner.2;
            count += 1.0;
        }

        (ao, sum_sky / count, sum_block / count)
    }
}

#[inline]
pub(super) fn emit_face(
    tris: &mut Vec<Triangle>,
    wx: i32, wy: i32, wz: i32,
    face: Face,
    block: BlockId,
    cache: &ChunkNeighborCache,
) {
    let shade = face.shade();
    let lx = wx - cache.base_x;
    let lz = wz - cache.base_z;

    let texture = if block == BlockId::Door {
        let below = cache.sample_block(lx, wy - 1, lz);
        if below == BlockId::Door {
            TextureId::DoorTop
        } else {
            TextureId::Door
        }
    } else if block == BlockId::Grass {
        let biome = cache.biome_at_cached(wx, wz);
        match face {
            Face::Top => match biome {
                Biome::Swamp | Biome::DarkForest => TextureId::GrassTopSwamp,
                Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::GrassTopSavanna,
                Biome::CherryGrove | Biome::Meadow => TextureId::GrassTopCherry,
                Biome::AutumnForest => TextureId::GrassTopAutumn,
                Biome::SnowyTaiga | Biome::Mountain | Biome::Taiga | Biome::IceSpikes | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes => TextureId::GrassTopTaiga,
                _ => TextureId::GrassTop,
            },
            Face::Bottom => TextureId::Dirt,
            _ => match biome {
                Biome::Swamp | Biome::DarkForest => TextureId::GrassSideSwamp,
                Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::GrassSideSavanna,
                Biome::CherryGrove | Biome::Meadow => TextureId::GrassSideCherry,
                Biome::AutumnForest => TextureId::GrassSideAutumn,
                Biome::SnowyTaiga | Biome::Mountain | Biome::Taiga | Biome::IceSpikes | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes => TextureId::GrassSideTaiga,
                _ => TextureId::GrassSide,
            },
        }
    } else if block == BlockId::Leaves {
        let biome = cache.biome_at_cached(wx, wz);
        match biome {
            Biome::Swamp | Biome::DarkForest => TextureId::LeavesSwamp,
            Biome::Savanna | Biome::Desert | Biome::Badlands => TextureId::LeavesSavanna,
            Biome::CherryGrove => TextureId::CherryLeaves,
            Biome::AutumnForest => TextureId::AutumnLeaves,
            Biome::SnowyTaiga | Biome::IceSpikes | Biome::JaggedPeaks | Biome::FrozenPeaks | Biome::SnowySlopes => TextureId::SnowLeaves,
            _ => TextureId::Leaves,
        }
    } else {
        block.texture_id(face)
    };

    let x = wx as f32;
    let y = wy as f32;
    let z = wz as f32;
    let is_fluid = matches!(block, BlockId::Water | BlockId::Lava);

    let top_y = if is_fluid {
        let above = cache.sample_block(lx, wy + 1, lz);
        if above == block || matches!(above, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
            y + 1.0
        } else {
            let lvl = cache.fluid_level(IVec3::new(wx, wy, wz));
            if lvl >= 8 {
                y + 1.0
            } else {
                y + (lvl as f32 / 8.0) * 0.85 + 0.05
            }
        }
    } else {
        y + 1.0
    };

    let c000 = Vec3::new(x,       y,     z);
    let c100 = Vec3::new(x + 1.0, y,     z);
    let c010 = Vec3::new(x,       top_y, z);
    let c110 = Vec3::new(x + 1.0, top_y, z);
    let c001 = Vec3::new(x,       y,     z + 1.0);
    let c101 = Vec3::new(x + 1.0, y,     z + 1.0);
    let c011 = Vec3::new(x,       top_y, z + 1.0);
    let c111 = Vec3::new(x + 1.0, top_y, z + 1.0);

    let sample_cell = |dx: i32, dy: i32, dz: i32| -> (bool, f32, f32) {
        let is_solid = cache.solid_at(lx + dx, wy + dy, lz + dz);
        let n_lx = lx + dx;
        let n_ly = wy + dy;
        let n_lz = lz + dz;
        let sky = if is_solid {
            0.0
        } else {
            cache.sample_sky_light(n_lx, n_ly, n_lz)
        };
        let block_l = if is_solid {
            0.0
        } else {
            cache.sample_block_light(
                wx as f32 + dx as f32 + 0.5,
                wy as f32 + dy as f32 + 0.5,
                wz as f32 + dz as f32 + 0.5,
            )
        };
        (is_solid, sky, block_l)
    };

    let n = face.normal();
    let center_cell = sample_cell(n.x, n.y, n.z);
    let center_sky = center_cell.1;
    let center_block = if block == BlockId::Lava {
        1.0
    } else {
        center_cell.2
    };

    let (a, (ao_a, sky_a, blk_a), b, (ao_b, sky_b, blk_b), c, (ao_c, sky_c, blk_c), d, (ao_d, sky_d, blk_d)) = match face {
        Face::Top => (
            c010, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,1,0), sample_cell(0,1,-1), sample_cell(-1,1,-1)),
            c011, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,1,0), sample_cell(0,1, 1), sample_cell(-1,1, 1)),
            c111, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,1,0), sample_cell(0,1, 1), sample_cell( 1,1, 1)),
            c110, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,1,0), sample_cell(0,1,-1), sample_cell( 1,1,-1)),
        ),
        Face::Bottom => (
            c001, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,-1,0), sample_cell(0,-1, 1), sample_cell(-1,-1, 1)),
            c000, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,-1,0), sample_cell(0,-1,-1), sample_cell(-1,-1,-1)),
            c100, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,-1,0), sample_cell(0,-1,-1), sample_cell( 1,-1,-1)),
            c101, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,-1,0), sample_cell(0,-1, 1), sample_cell( 1,-1, 1)),
        ),
        Face::North => (
            c100, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,-1), sample_cell(0,-1,-1), sample_cell( 1,-1,-1)),
            c000, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(0,-1,-1), sample_cell(-1,-1,-1)),
            c010, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(0, 1,-1), sample_cell(-1, 1,-1)),
            c110, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,-1), sample_cell(0, 1,-1), sample_cell( 1, 1,-1)),
        ),
        Face::South => (
            c001, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,1), sample_cell(0,-1,1), sample_cell(-1,-1,1)),
            c101, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,1), sample_cell(0,-1,1), sample_cell( 1,-1,1)),
            c111, vertex_light_and_ao(center_sky, center_block, sample_cell( 1,0,1), sample_cell(0, 1,1), sample_cell( 1, 1,1)),
            c011, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,1), sample_cell(0, 1,1), sample_cell(-1, 1,1)),
        ),
        Face::East => (
            c101, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0, 1), sample_cell(1,-1,0), sample_cell(1,-1, 1)),
            c100, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0,-1), sample_cell(1,-1,0), sample_cell(1,-1,-1)),
            c110, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0,-1), sample_cell(1, 1,0), sample_cell(1, 1,-1)),
            c111, vertex_light_and_ao(center_sky, center_block, sample_cell(1,0, 1), sample_cell(1, 1,0), sample_cell(1, 1, 1)),
        ),
        Face::West => (
            c000, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(-1,-1,0), sample_cell(-1,-1,-1)),
            c001, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0, 1), sample_cell(-1,-1,0), sample_cell(-1,-1, 1)),
            c011, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0, 1), sample_cell(-1, 1,0), sample_cell(-1, 1, 1)),
            c010, vertex_light_and_ao(center_sky, center_block, sample_cell(-1,0,-1), sample_cell(-1, 1,0), sample_cell(-1, 1,-1)),
        ),
    };

    let va = Vertex { pos: a, uv: (0.0, 1.0), ao: shade * ao_a, sky_light: sky_a, block_light: blk_a };
    let vb = Vertex { pos: b, uv: (1.0, 1.0), ao: shade * ao_b, sky_light: sky_b, block_light: blk_b };
    let vc = Vertex { pos: c, uv: (1.0, 0.0), ao: shade * ao_c, sky_light: sky_c, block_light: blk_c };
    let vd = Vertex { pos: d, uv: (0.0, 0.0), ao: shade * ao_d, sky_light: sky_d, block_light: blk_d };

    // DLACZEGO: Flip quad diagonal along the brighter corner pair so AO gradients stay smooth across triangles.
    let light_ac = ao_a + ao_c;
    let light_bd = ao_b + ao_d;
    if light_ac < light_bd {
        tris.push(Triangle { v0: vb, v1: vc, v2: vd, texture });
        tris.push(Triangle { v0: vb, v1: vd, v2: va, texture });
    } else {
        tris.push(Triangle { v0: va, v1: vb, v2: vc, texture });
        tris.push(Triangle { v0: va, v1: vc, v2: vd, texture });
    }
}
