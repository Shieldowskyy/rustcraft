//! Unit tests for chunk/subchunk meshing, lighting/AO, fluids, non-opaque subchunk culling, and `AsyncMesher`.

use std::time::{Duration, Instant};

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::rendering::atlas::TextureId;
use crate::features::world::{Chunk, World, WorldGeneratorType, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

use super::shapes::emit_ladder;
use super::{
    build_chunk_mesh, build_chunk_mesh_from_job_limited, build_subchunk_mesh, AsyncMesher,
    ChunkMeshJob, ChunkNeighborCache, Triangle,
};

fn empty_superflat_world() -> World {
    let mut world = World::new_with_generator(1, WorldGeneratorType::Superflat);
    world.chunks.clear();
    world
}

fn empty_chunk(cx: i32, cz: i32) -> Chunk {
    Chunk {
        cx,
        cz,
        blocks: vec![0; CHUNK_SIZE_X as usize * CHUNK_SIZE_Y as usize * CHUNK_SIZE_Z as usize],
        dirty: false,
    }
}

fn fill_dense_caves(chunk: &mut Chunk) {
    for x in 0..CHUNK_SIZE_X {
        for y in 1..15 {
            for z in 0..CHUNK_SIZE_Z {
                if (x + y + z) % 2 == 0 {
                    chunk.set_local(x, y, z, BlockId::Stone);
                }
            }
        }
    }
}

fn drain_one_result(mesher: &AsyncMesher) -> ((i32, i32), Vec<Triangle>) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        let mut results = mesher.drain_completed();
        if !results.is_empty() {
            return results.remove(0);
        }
        std::thread::yield_now();
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("timed out waiting for AsyncMesher result");
}

#[test]
fn ladder_mesh_is_double_sided_and_faces_away_from_east_support() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    chunk.set_local(2, 5, 2, BlockId::Ladder);
    chunk.set_local(3, 5, 2, BlockId::Stone);
    world.chunks.insert((0, 0), chunk);

    let cache = ChunkNeighborCache::new(world.chunks.get(&(0, 0)).unwrap(), &world);
    let mut tris = Vec::new();
    emit_ladder(&mut tris, 2, 5, 2, &cache);

    assert_eq!(tris.len(), 4);
    assert!(tris.iter().all(|tri| (tri.v0.pos.x - 2.95).abs() < 1e-4));

    let first = &tris[0];
    let ab = Vec3::new(
        first.v1.pos.x - first.v0.pos.x,
        first.v1.pos.y - first.v0.pos.y,
        first.v1.pos.z - first.v0.pos.z,
    );
    let ac = Vec3::new(
        first.v2.pos.x - first.v0.pos.x,
        first.v2.pos.y - first.v0.pos.y,
        first.v2.pos.z - first.v0.pos.z,
    );
    let normal_x = ab.y * ac.z - ab.z * ac.y;
    assert!(normal_x < 0.0, "ladder front should face away from east support");
}

#[test]
fn test_chunk_bottom_face_culling_at_y0() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    chunk.set_local(0, 0, 0, BlockId::Stone);
    chunk.set_local(5, 10, 5, BlockId::Stone);

    world.chunks.insert((0, 0), chunk);
    let mesh = build_chunk_mesh(world.chunks.get(&(0, 0)).unwrap(), &world);

    let has_y0_bottom_face = mesh.iter().any(|tri| {
        tri.v0.pos.y == 0.0
            && tri.v1.pos.y == 0.0
            && tri.v2.pos.y == 0.0
            && tri.v0.pos.x <= 1.0
            && tri.v0.pos.z <= 1.0
    });
    assert!(!has_y0_bottom_face, "Bottom face at y=0 should be culled");

    let has_y10_bottom_face = mesh
        .iter()
        .any(|tri| tri.v0.pos.y == 10.0 && tri.v1.pos.y == 10.0 && tri.v2.pos.y == 10.0);
    assert!(has_y10_bottom_face, "Bottom face at y=10 with air below should be generated");
}

#[test]
fn test_subchunk_mesh_building() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    chunk.set_local(2, 5, 2, BlockId::Stone);
    chunk.set_local(2, 20, 2, BlockId::Stone);

    world.chunks.insert((0, 0), chunk);
    let c = world.chunks.get(&(0, 0)).unwrap();

    let sub0 = build_subchunk_mesh(c, &world, 0);
    let sub1 = build_subchunk_mesh(c, &world, 1);
    let sub2 = build_subchunk_mesh(c, &world, 2);

    assert!(!sub0.is_empty());
    assert!(!sub1.is_empty());
    assert!(sub2.is_empty());

    for tri in sub0.iter().chain(sub1.iter()) {
        for v in [&tri.v0, &tri.v1, &tri.v2] {
            assert!(v.uv.0 >= 0.0 && v.uv.0 <= 1.0);
            assert!(v.uv.1 >= 0.0 && v.uv.1 <= 1.0);
        }
    }
}

#[test]
fn test_light_occlusion_and_smooth_lighting() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    for x in 0..16 {
        for z in 0..16 {
            chunk.set_local(x, 0, z, BlockId::Stone);
        }
    }
    for y in 1..=4 {
        for z in 0..16 {
            chunk.set_local(5, y, z, BlockId::Stone);
        }
    }
    world.chunks.insert((0, 0), chunk);
    world.torches.insert(IVec3::new(3, 2, 8));

    let cache = ChunkNeighborCache::new(world.chunks.get(&(0, 0)).unwrap(), &world);
    let light_near = cache.sample_block_light(4.5, 2.5, 8.5);
    assert!(light_near > 0.5, "Light near torch should be bright, got {light_near}");

    let light_behind_wall = cache.sample_block_light(6.5, 2.5, 8.5);
    assert_eq!(light_behind_wall, 0.0, "Light behind wall should be occluded");
}

#[test]
fn test_chunk_boundary_sky_light() {
    let mut world = empty_superflat_world();
    let mut chunk_west = empty_chunk(-1, 0);
    for y in 0..=20 {
        chunk_west.set_local(15, y, 5, BlockId::Stone);
    }
    world.chunks.insert((-1, 0), chunk_west);
    world.chunks.insert((0, 0), empty_chunk(0, 0));

    let cache = ChunkNeighborCache::new(world.chunks.get(&(0, 0)).unwrap(), &world);
    assert_eq!(cache.sample_sky_light(-1, 5, 5), 0.0);
    assert_eq!(cache.sample_sky_light(-1, 25, 5), 1.0);
}

#[test]
fn test_kelp_and_seagrass_cross_mesh() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    chunk.set_local(5, 9, 5, BlockId::Sand);
    chunk.set_local(5, 10, 5, BlockId::Kelp);
    chunk.set_local(5, 11, 5, BlockId::Water);
    chunk.set_local(4, 10, 5, BlockId::Water);
    chunk.set_local(6, 10, 5, BlockId::Water);
    chunk.set_local(5, 10, 4, BlockId::Water);
    chunk.set_local(5, 10, 6, BlockId::Water);

    chunk.set_local(10, 9, 10, BlockId::Sand);
    chunk.set_local(10, 10, 10, BlockId::Seagrass);
    chunk.set_local(10, 11, 10, BlockId::Water);

    world.chunks.insert((0, 0), chunk);
    let mesh = build_chunk_mesh(world.chunks.get(&(0, 0)).unwrap(), &world);

    assert_eq!(mesh.iter().filter(|t| t.texture == TextureId::Kelp).count(), 8);
    assert_eq!(mesh.iter().filter(|t| t.texture == TextureId::Seagrass).count(), 8);

    let face_west_of_kelp = mesh.iter().filter(|t| {
        t.texture == TextureId::Water
            && (t.v0.pos.x - 5.0).abs() < 1e-4
            && (t.v1.pos.x - 5.0).abs() < 1e-4
            && (t.v2.pos.x - 5.0).abs() < 1e-4
            && (t.v0.pos.y + t.v1.pos.y + t.v2.pos.y) / 3.0 < 11.0
            && (t.v0.pos.z + t.v1.pos.z + t.v2.pos.z) / 3.0 > 5.0
            && (t.v0.pos.z + t.v1.pos.z + t.v2.pos.z) / 3.0 < 6.0
    }).count();
    assert_eq!(face_west_of_kelp, 0);
}

#[test]
fn test_water_surface_is_emitted_at_block_top() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    chunk.set_local(5, 10, 5, BlockId::Water);
    world.chunks.insert((0, 0), chunk);

    let mesh = build_chunk_mesh(world.chunks.get(&(0, 0)).unwrap(), &world);
    let surface_triangles = mesh
        .iter()
        .filter(|tri| {
            tri.texture == TextureId::Water
                && (tri.v0.pos.y - 11.0).abs() < 1e-4
                && (tri.v1.pos.y - 11.0).abs() < 1e-4
                && (tri.v2.pos.y - 11.0).abs() < 1e-4
        })
        .count();
    assert_eq!(surface_triangles, 2);
}

#[test]
fn test_non_opaque_blocks_above_opaque_subchunk_are_not_skipped() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    // Opaque floor below subchunk 1 (y <= 10)
    chunk.set_local(2, 10, 2, BlockId::Sand);
    for y in 11..=20 {
        chunk.set_local(2, y, 2, BlockId::Water);
    }
    // Trunk stops at y=31; canopy lives solely inside subchunk 2 (y=32..35)
    chunk.set_local(8, 31, 8, BlockId::Wood);
    for y in 32..=34 {
        chunk.set_local(8, y, 8, BlockId::Leaves);
    }
    // Non-opaque blocks inside subchunk 3 (y=48)
    chunk.set_local(12, 48, 12, BlockId::Glass);
    chunk.set_local(14, 48, 14, BlockId::WoodenSlab);

    world.chunks.insert((0, 0), chunk);
    let job = world.create_mesh_job((0, 0)).unwrap();
    let cache = ChunkNeighborCache::from_job(&job);

    let (_, sub1_trans) = super::build_subchunk_mesh_from_cache(1, &cache);
    let (sub2_solid, _) = super::build_subchunk_mesh_from_cache(2, &cache);
    let (sub3_solid, sub3_trans) = super::build_subchunk_mesh_from_cache(3, &cache);

    assert!(sub1_trans.iter().any(|t| t.texture == TextureId::Water));
    assert!(sub2_solid.iter().any(|t| t.texture == TextureId::Leaves));
    assert!(sub3_trans.iter().any(|t| t.texture == TextureId::Glass));
    assert!(sub3_solid.iter().any(|t| t.texture == TextureId::Planks));

    // Truly empty upper subchunks still skip cleanly.
    let mut flat_world = empty_superflat_world();
    let mut flat_chunk = empty_chunk(0, 0);
    flat_chunk.set_local(0, 10, 0, BlockId::Stone);
    flat_world.chunks.insert((0, 0), flat_chunk);
    let empty_sub = build_subchunk_mesh(flat_world.chunks.get(&(0, 0)).unwrap(), &flat_world, 2);
    assert!(empty_sub.is_empty());
}

#[test]
fn test_async_mesher_does_not_truncate_dense_cave_chunks_on_desktop() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    fill_dense_caves(&mut chunk);
    for x in 0..CHUNK_SIZE_X {
        for z in 0..CHUNK_SIZE_Z {
            chunk.set_local(x, 40, z, BlockId::Grass);
            chunk.set_local(x, 41, z, BlockId::Water);
        }
    }
    world.chunks.insert((0, 0), chunk);

    let job = world.create_mesh_job((0, 0)).unwrap();
    let mesher = AsyncMesher::new();
    assert!(mesher.submit_job(job));
    let (key, mesh) = drain_one_result(&mesher);

    assert_eq!(key, (0, 0));
    assert!(mesh.len() > 4096, "expected full unclipped mesh, got {} tris", mesh.len());

    let grass_top = mesh
        .iter()
        .filter(|t| {
            t.texture == TextureId::GrassTop
                && (t.v0.pos.y - 41.0).abs() < 1e-4
                && (t.v1.pos.y - 41.0).abs() < 1e-4
                && (t.v2.pos.y - 41.0).abs() < 1e-4
        })
        .count();
    let water_top = mesh
        .iter()
        .filter(|t| {
            t.texture == TextureId::Water
                && (t.v0.pos.y - 42.0).abs() < 1e-4
                && (t.v1.pos.y - 42.0).abs() < 1e-4
                && (t.v2.pos.y - 42.0).abs() < 1e-4
        })
        .count();
    assert_eq!(grass_top, 512);
    assert_eq!(water_top, 512);
}

#[test]
fn test_limited_mesher_prioritizes_surface_and_water_over_deep_caves() {
    let mut world = empty_superflat_world();
    let mut chunk = empty_chunk(0, 0);
    fill_dense_caves(&mut chunk);
    for x in 4..12 {
        for z in 4..12 {
            chunk.set_local(x, 40, z, BlockId::Grass);
            chunk.set_local(x, 41, z, BlockId::Water);
        }
    }
    world.chunks.insert((0, 0), chunk);

    let job: ChunkMeshJob = world.create_mesh_job((0, 0)).unwrap();
    let mesh = build_chunk_mesh_from_job_limited(&job, 1024);
    assert!(mesh.len() <= 1024);

    let grass_top = mesh
        .iter()
        .filter(|t| t.texture == TextureId::GrassTop && (t.v0.pos.y - 41.0).abs() < 1e-4)
        .count();
    let water_top = mesh
        .iter()
        .filter(|t| t.texture == TextureId::Water && (t.v0.pos.y - 42.0).abs() < 1e-4)
        .count();
    assert_eq!(grass_top, 128);
    assert_eq!(water_top, 128);
}

#[test]
fn test_async_mesher_invalidate_pending_drops_stale_results() {
    let mut world = empty_superflat_world();
    let mut c0 = empty_chunk(0, 0);
    c0.set_local(4, 10, 4, BlockId::Stone);
    let mut c1 = empty_chunk(1, 0);
    c1.set_local(4, 10, 4, BlockId::Stone);
    world.chunks.insert((0, 0), c0);
    world.chunks.insert((1, 0), c1);

    let mut mesher = AsyncMesher::with_max_triangles(None);
    assert!(mesher.submit_job(world.create_mesh_job((0, 0)).unwrap()));
    mesher.invalidate_pending();
    std::thread::sleep(Duration::from_millis(50));
    assert!(mesher.drain_completed().is_empty());

    assert!(mesher.submit_job(world.create_mesh_job((1, 0)).unwrap()));
    let (key, mesh) = drain_one_result(&mesher);
    assert_eq!(key, (1, 0));
    assert!(!mesh.is_empty());
}
