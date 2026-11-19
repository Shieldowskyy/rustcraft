//! Biome generation, ruststone circuit, village torch lighting, and superflat unit tests.
//! Migrated from `src/world/mod.rs`.

use std::collections::HashSet;

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::meshing::ChunkNeighborCache;

use super::gen::hash_coords;
use super::*;

    #[test]
    fn test_biome_name_at_all_biomes() {
        assert!(!World::new_with_seed(123).biome_name_at(0, 0).is_empty());
    }

    #[test]
    fn test_all_biomes_have_distinct_color_filters() {
        let all = [
            Biome::DeepOcean, Biome::Ocean, Biome::SnowyTaiga, Biome::SnowyTundra, Biome::AutumnForest, Biome::Forest,
            Biome::CherryGrove, Biome::Plains, Biome::Swamp, Biome::Savanna, Biome::Desert, Biome::Mountain,
            Biome::Jungle, Biome::Badlands, Biome::DarkForest, Biome::Taiga, Biome::BirchForest, Biome::IceSpikes,
            Biome::MushroomIsland, Biome::Meadow, Biome::JaggedPeaks, Biome::FrozenPeaks, Biome::SnowySlopes,
            Biome::River, Biome::FrozenRiver, Biome::Beach, Biome::ColdBeach, Biome::StoneBeach,
            Biome::SunflowerPlains, Biome::FlowerForest, Biome::MegaTaiga, Biome::JungleEdge,
            Biome::SavannaPlateau, Biome::Mesa, Biome::MesaPlateau, Biome::MesaPlateauForest, Biome::MesaBryce,
        ];
        let mut seen = HashSet::new();
        for b in all {
            let f = b.color_filter();
            assert!(seen.insert(((f[0] * 1000.0).round() as i32, (f[1] * 1000.0).round() as i32, (f[2] * 1000.0).round() as i32)));
        }
        assert_eq!(seen.len(), 37);
    }

    #[test]
    fn test_smooth_mountain_transitions_no_sheer_biome_cliffs() {
        let seed = 424242;
        for x in (-1500..=1500).step_by(10) {
            for z in (-1500..=1500).step_by(10) {
                let ((b1, c1), (b2, c2)) = (get_biome_at(x, z, seed), get_biome_at(x + 1, z, seed));
                let is_m = |b| matches!(b, Biome::Meadow | Biome::SnowySlopes | Biome::Mountain | Biome::JaggedPeaks | Biome::FrozenPeaks);
                if is_m(b1) && is_m(b2) {
                    assert!((get_height_at(x, z, seed, b1, c1) - get_height_at(x + 1, z, seed, b2, c2)).abs() <= 3);
                }
            }
        }
    }

    #[test]
    fn test_mountain_biome_block_continuity_no_checkerboard() {
        let seed = 2858965613;
        let chunk = Chunk::generate(29, 41, seed);
        for lx in 0..15 {
            for lz in 0..15 {
                let (wx, wz) = (chunk.cx * CHUNK_SIZE_X + lx, chunk.cz * CHUNK_SIZE_Z + lz);
                let gb = |dx, dz| {
                    let (b, c) = get_biome_at(wx + dx, wz + dz, seed);
                    chunk.get_local(lx + dx, get_height_at(wx + dx, wz + dz, seed, b, c), lz + dz)
                };
                let (b00, b10, b01, b11) = (gb(0, 0), gb(1, 0), gb(0, 1), gb(1, 1));
                assert!(!(b00 == b11 && b10 == b01 && b00 != b10));
            }
        }
    }

    #[test]
    fn test_ruststone_circuit_simulation() {
        let mut world = World::new_with_seed(12345);
        let (lever_pos, wire_pos, lamp_pos) = (IVec3::new(0, 51, 0), IVec3::new(1, 51, 0), IVec3::new(2, 51, 0));
        for x in 0..=2 { world.set_block(IVec3::new(x, 50, 0), BlockId::Stone); }
        world.set_block(lever_pos, BlockId::Lever);
        world.set_block(wire_pos, BlockId::RustStone);
        world.set_block(lamp_pos, BlockId::RustStoneLamp);

        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);

        world.set_block(lever_pos, BlockId::LeverOn);
        world.update_ruststone_network(lever_pos, 10);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);

        world.set_block(lever_pos, BlockId::ButtonPressed);
        world.button_timers.insert(lever_pos, 0.5);
        world.update_ruststone(0.6, Vec3::new(100.0, 50.0, 100.0), &[]);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLamp);

        world.set_block(wire_pos, BlockId::PressurePlate);
        world.update_ruststone(0.1, Vec3::new(1.2, 51.0, 0.2), &[]);
        assert_eq!(world.get_block(lamp_pos), BlockId::RustStoneLampLit);
    }

    #[test]
    fn tnt_placed_on_ruststone_block_is_primed() {
        let mut world = World::new_with_seed(12345);
        let source = IVec3::new(4, 51, 4);
        let tnt = source + IVec3::new(0, 1, 0);
        world.set_block(source, BlockId::RustStoneBlock);
        world.set_block(tnt, BlockId::Tnt);

        world.update_ruststone_network(tnt, 16);

        assert_eq!(world.get_block(tnt), BlockId::Air);
        assert_eq!(world.primed_tnts.len(), 1);
    }

    #[test]
    fn test_spaced_villages_and_torch_lighting() {
        let seed: u32 = 1337;
        let mut found = None;
        for rx in -2..=2 {
            for rz in -2..=2 {
                let vh = hash_coords(rx, rz, seed.wrapping_add(8881337));
                let (vcx, vcz) = (rx * 20 + (vh % 16) as i32 + 2, rz * 20 + ((vh >> 16) % 16) as i32 + 2);
                let (vb, vc) = get_biome_at(vcx * 16 + 8, vcz * 16 + 8, seed);
                let vh_y = get_height_at(vcx * 16 + 8, vcz * 16 + 8, seed, vb, vc);
                if matches!(vb, Biome::Plains | Biome::Meadow | Biome::Savanna | Biome::Desert | Biome::Taiga) && (23..CHUNK_SIZE_Y - 14).contains(&vh_y) {
                    found = Some((vcx, vcz));
                    break;
                }
            }
        }
        let (vcx, vcz) = found.unwrap();
        let center_chunk = Chunk::generate(vcx, vcz, seed);
        assert!(center_chunk.blocks.contains(&BlockId::Torch.to_u8()));
        assert!(Chunk::generate(vcx + 1, vcz, seed).blocks.contains(&BlockId::Bed.to_u8()));
        assert!(Chunk::generate(vcx - 1, vcz, seed).blocks.contains(&BlockId::Furnace.to_u8()));
    }

    #[test]
    fn changing_torch_dirties_neighbor_chunk_within_light_radius() {
        let mut world = World::new();
        for cx in 0..=1 {
            for cz in 0..=1 {
                world.chunks.insert(
                    (cx, cz),
                                    Chunk {
                                        cx,
                                        cz,
                                        blocks: vec![0; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
                                    dirty: false,
                                    },
                );
            }
        }

        let torch_pos = IVec3::new(14, 32, 8);
        world.set_block(torch_pos, BlockId::Torch);
        assert!(
            world.chunks.get(&(1, 0)).unwrap().dirty,
                "placing a torch near the border should rebuild the lit neighbor chunk"
        );
        let target_job = world.create_mesh_job((1, 0)).unwrap();
        assert!(target_job.lights.iter().any(|light| light.pos == torch_pos));
        let target_cache = ChunkNeighborCache::from_job(&target_job);
        assert!(
            target_cache.sample_block_light(16.5, 32.5, 8.5) > 0.0,
                "the neighboring chunk should receive light from the torch"
        );

        world.chunks.get_mut(&(1, 0)).unwrap().dirty = false;
        world.set_block(torch_pos, BlockId::Air);
        assert!(
            world.chunks.get(&(1, 0)).unwrap().dirty,
                "removing a torch should rebuild the neighbor chunk to remove stale light"
        );

        world.chunks.get_mut(&(1, 1)).unwrap().dirty = false;
        let mut streamed_chunk = Chunk {
            cx: 0,
            cz: 0,
            blocks: vec![0; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
            dirty: false,
        };
        streamed_chunk.set_local(14, 32, 14, BlockId::Torch);
        world.index_chunk_lights(&streamed_chunk);
        assert!(
            world.chunks.get(&(1, 1)).unwrap().dirty,
                "indexing a streamed torch should rebuild diagonal chunks in its light radius"
        );
    }

    #[test]
    fn test_superflat_chunk_layering() {
        let chunk = Chunk::generate_superflat(2, -4);
        assert_eq!(chunk.cx, 2);
        assert_eq!(chunk.cz, -4);

        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                assert_eq!(chunk.get_local(lx, 0, lz), BlockId::Bedrock);
                assert_eq!(chunk.get_local(lx, 1, lz), BlockId::Dirt);
                assert_eq!(chunk.get_local(lx, 2, lz), BlockId::Dirt);
                assert_eq!(chunk.get_local(lx, 3, lz), BlockId::Grass);
                for y in 4..CHUNK_SIZE_Y {
                    assert_eq!(chunk.get_local(lx, y, lz), BlockId::Air);
                }
            }
        }

        let via_type = Chunk::generate_with_type(0, 0, 9999, WorldGeneratorType::Superflat);
        assert_eq!(via_type.get_local(7, 3, 7), BlockId::Grass);
        assert_eq!(via_type.get_local(7, 4, 7), BlockId::Air);
    }

    #[test]
    fn test_superflat_world_creation_and_spawn() {
        let world = World::new_with_generator(42, WorldGeneratorType::Superflat);
        assert_eq!(world.generator_type, WorldGeneratorType::Superflat);
        assert_eq!(world.get_biome_at(0, 0), Biome::Plains);
        assert_eq!(world.get_biome_at(1234, -5678), Biome::Plains);
        assert_eq!(world.biome_name_at(10, 10), "PLAINS");

        // Spawn position should be safely standing on top of the grass layer (y=3 block, surface=4)
        assert!(world.player_position.y >= 5.6, "player spawn eye position must be above grass");

        // Center chunks must be superflat
        assert_eq!(world.get_block(IVec3::new(8, 0, 8)), BlockId::Bedrock);
        assert_eq!(world.get_block(IVec3::new(8, 1, 8)), BlockId::Dirt);
        assert_eq!(world.get_block(IVec3::new(8, 2, 8)), BlockId::Dirt);
        assert_eq!(world.get_block(IVec3::new(8, 3, 8)), BlockId::Grass);
        assert_eq!(world.get_block(IVec3::new(8, 4, 8)), BlockId::Air);
    }

    #[test]
    fn test_superflat_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let save_file = temp_dir.join("rustcraft_test_superflat_save.bin");
        let path_str = save_file.to_str().unwrap();

        let mut world = World::new_with_generator(777, WorldGeneratorType::Superflat);
        world.name = "Flatland Test".to_string();
        world.save(path_str).expect("saving superflat world failed");

        let loaded = World::load(path_str).expect("loading superflat world failed");
        assert_eq!(loaded.name, "Flatland Test");
        assert_eq!(loaded.generator_type, WorldGeneratorType::Superflat);
        assert_eq!(loaded.seed, 777);
        assert_eq!(loaded.get_biome_at(0, 0), Biome::Plains);
        assert_eq!(loaded.get_block(IVec3::new(8, 0, 8)), BlockId::Bedrock);
        assert_eq!(loaded.get_block(IVec3::new(8, 3, 8)), BlockId::Grass);
        assert_eq!(loaded.get_block(IVec3::new(8, 4, 8)), BlockId::Air);

        let _ = std::fs::remove_file(save_file);
    }

    #[test]
    fn test_world_generator_type_and_ui_rects() {
        assert_eq!(WorldGeneratorType::Default.name(), "DEFAULT");
        assert_eq!(WorldGeneratorType::Superflat.name(), "SUPERFLAT");

        let rects = crate::ui::create_world_rects(960, 540);
        assert_eq!(rects.len(), 8);
        for &(x, y, w, h) in &rects {
            assert!(w > 0);
            assert!(h > 0);
            assert!(x >= 0);
            assert!(y >= 0);
        }
    }
