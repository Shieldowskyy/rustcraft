//! Core `World` unit tests covering chunk indexing, streaming, fluids, spawn rules, and save/load roundtrips.
//! Migrated from `src/world/mod.rs`.

use std::collections::HashMap;

use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::physics::STANDING_EYE_OFFSET;
use crate::features::player::ItemStack;

use super::*;

    #[test]
    fn test_chunk_index_and_local_blocks() {
        let mut chunk = Chunk { cx: 0, cz: 0, blocks: vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize], dirty: false };
        chunk.set_local(0, 0, 0, BlockId::Stone);
        assert_eq!(chunk.get_local(0, 0, 0), BlockId::Stone);
        assert!(chunk.dirty);
        chunk.set_local(15, 63, 15, BlockId::IronOre);
        assert_eq!(chunk.get_local(15, 63, 15), BlockId::IronOre);
        assert_eq!(chunk.get_local(-1, 0, 0), BlockId::Air);
        assert_eq!(chunk.get_local(16, 0, 0), BlockId::Air);
    }

    #[test]
    fn simulation_chunk_streaming_includes_distant_players() {
        let mut world = World::new_with_seed(1);
        let local = Vec3::new(8.0, 60.0, 8.0);
        let remote = Vec3::new(328.0, 60.0, -184.0);

        world.update_with_simulation_centers(local, 2, &[remote]);

        let remote_chunk = (20, -12);
        assert!(
            world.chunks.contains_key(&remote_chunk)
            || world.pending_chunks.contains(&remote_chunk),
                "chunk around remote player should be generated or queued for mob simulation"
        );
    }

    #[test]
    fn test_bed_pair_uses_two_horizontal_blocks() {
        let mut world = World::new_with_seed(1);
        world.chunks.clear();
        world.chunks.insert(
            (0, 0),
                            Chunk {
                                cx: 0,
                                cz: 0,
                                blocks: vec![
                                    BlockId::Air.to_u8();
                                    (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize
                                ],
                                dirty: false,
                            },
        );
        let foot = IVec3::new(4, 20, 4);
        let head = IVec3::new(4, 20, 5);
        world.set_block(foot, BlockId::Bed);
        assert_eq!(world.bed_pair_at(foot), None);

        world.set_block(head, BlockId::Bed);
        assert_eq!(world.bed_pair_at(foot), Some((foot, head)));
        assert_eq!(world.bed_pair_at(head), Some((head, foot)));

        world.set_block(head, BlockId::Air);
        assert_eq!(world.bed_pair_at(foot), None);
    }

    #[test]
    fn falling_fluid_preserves_its_spread_level() {
        for (fluid, ticks) in [(BlockId::Water, 1), (BlockId::Lava, 3)] {
            let mut world = World::new_with_seed(1);
            world.chunks.clear();
            world.chunks.insert(
                (0, 0),
                                Chunk {
                                    cx: 0,
                                    cz: 0,
                                    blocks: vec![
                                        BlockId::Air.to_u8();
                                        (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize
                                    ],
                                    dirty: false,
                                },
            );
            let source = IVec3::new(8, 48, 8);
            world.set_block(source, fluid);
            world.set_fluid_level(source, 4);

            for _ in 0..ticks {
                world.simulate_materials(Vec3::new(8.5, 48.0, 8.5), true);
            }

            let below = IVec3::new(source.x, source.y - 1, source.z);
            assert_eq!(world.get_block(below), fluid);
            assert_eq!(world.fluid_level(below), 4);
        }
    }

    #[test]
    fn test_acoustic_reverb_tracks_enclosure_and_surface_material() {
        let mut world = World::new_with_seed(1);
        world.chunks.clear();
        world.chunks.insert(
            (0, 0),
                            Chunk {
                                cx: 0,
                                cz: 0,
                                blocks: vec![
                                    BlockId::Air.to_u8();
                                    (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize
                                ],
                                dirty: false,
                            },
        );
        let listener = Vec3::new(8.5, 32.5, 8.5);
        let open_air = world.calculate_acoustic_reverb(listener);
        assert_eq!(open_air.0, 0.0);

        for x in 4..=12 {
            for z in 4..=12 {
                for y in 28..=36 {
                    if x == 4 || x == 12 || z == 4 || z == 12 || y == 28 || y == 36 {
                        world.set_block(IVec3::new(x, y, z), BlockId::Stone);
                    }
                }
            }
        }
        let stone_room = world.calculate_acoustic_reverb(listener);
        assert!(stone_room.0 > open_air.0);

        for x in 4..=12 {
            for z in 4..=12 {
                for y in 28..=36 {
                    if x == 4 || x == 12 || z == 4 || z == 12 || y == 28 || y == 36 {
                        world.set_block(IVec3::new(x, y, z), BlockId::Planks);
                    }
                }
            }
        }
        let wooden_room = world.calculate_acoustic_reverb(listener);
        assert!(wooden_room.0 < stone_room.0);
    }

    #[test]
    fn test_find_spawn_position_disallowed_biomes() {
        for seed in [1337, 42, 999999, 1234567, 777] {
            let spawn = World::find_spawn_position(seed);
            let (biome, _) = get_biome_at(spawn.x.floor() as i32, spawn.z.floor() as i32, seed);
            assert!(!matches!(biome, Biome::DeepOcean | Biome::Ocean | Biome::Swamp));
            assert!(spawn.y >= 24.0);
        }
    }

    #[test]
    fn test_obstructed_spawn_moves_to_clear_nearby_position() {
        let mut world = World::new_with_seed(12345);
        let spawn = world.player_position;
        let obstruction = IVec3::new(
            spawn.x.floor() as i32,
                                     (spawn.y - STANDING_EYE_OFFSET).floor() as i32,
                                     spawn.z.floor() as i32,
        );
        world.set_block(obstruction, BlockId::Stone);

        let safe_spawn = world
        .find_clear_spawn_position(spawn)
        .expect("a nearby spawn position should be clear");

        assert_ne!((safe_spawn.x, safe_spawn.z), (spawn.x, spawn.z));
        assert!(world.player_position_is_clear(safe_spawn));
    }

    #[test]
    fn test_leaf_support_and_decay_after_trunk_removal() {
        let mut world = World::new();
        world.chunks.insert((0, 0), Chunk { cx: 0, cz: 0, blocks: vec![0u8; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize], dirty: false });
        for pos in [IVec3::new(5, 10, 5), IVec3::new(5, 11, 5)] { world.set_block(pos, BlockId::Wood); }
        for pos in [IVec3::new(5, 12, 5), IVec3::new(6, 11, 5), IVec3::new(4, 11, 5)] { world.set_block(pos, BlockId::Leaves); }

        assert!(world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert!(world.find_unsupported_leaves_around(IVec3::new(5, 10, 5)).is_empty());

        world.set_block(IVec3::new(5, 10, 5), BlockId::Air);
        assert!(world.is_leaf_supported(IVec3::new(5, 12, 5), 5));

        world.set_block(IVec3::new(5, 11, 5), BlockId::Air);
        assert!(!world.is_leaf_supported(IVec3::new(5, 12, 5), 5));
        assert_eq!(world.find_unsupported_leaves_around(IVec3::new(5, 11, 5)).len(), 3);
    }

    #[test]
    fn test_world_save_and_load_with_player_stats() {
        let path_str = std::env::temp_dir().join("rustcraft_test_player_world.bin").to_string_lossy().to_string();
        let mut world = World::new_with_seed(12345);
        world.name = "PlayerSaveTest".to_string();

        let mut player = crate::features::player::Player::new();
        player.health = 13.0;
        player.hunger = 15.0;
        player.saturation = 4.0;
        player.exhaustion = 2.0;
        player.air = 18.0;
        player.selected_hotbar_slot = 4;
        player.inventory.hotbar[4] = Some(ItemStack::new(BlockId::Torch, 16));
        player.inventory.armor[1] = Some(ItemStack::new(BlockId::IronChestplate, 1));
        player.inventory.offhand = Some(ItemStack::new(BlockId::Shield, 1));

        world.saved_player = Some(player.save_state());
        assert!(world.save(&path_str).is_ok());

        let loaded = World::load(&path_str).expect("Should load world");
        assert_eq!(loaded.name, "PlayerSaveTest");
        let lp = loaded.saved_player.expect("Should load player data");
        assert_eq!((lp.health, lp.hunger, lp.saturation, lp.selected_hotbar_slot), (13.0, 15.0, 4.0, 4));
        assert_eq!(lp.hotbar[4].unwrap().block, BlockId::Torch);
        assert_eq!(lp.armor[1].unwrap().block, BlockId::IronChestplate);
        assert_eq!(lp.offhand.unwrap().block, BlockId::Shield);
        let _ = std::fs::remove_file(path_str);
    }

    #[test]
    fn test_block_changes_survive_chunk_unload_and_world_reload() {
        let path = std::env::temp_dir().join(format!(
            "rustcraft_test_block_overrides_{}.bin",
            std::process::id()
        ));
        let path_str = path.to_string_lossy().to_string();
        let mut world = World::new_with_seed(54321);
        world.chunks.insert((0, 0), Chunk::generate(0, 0, world.seed));

        let first = IVec3::new(2, 63, 3);
        let second = IVec3::new(4, 63, 5);
        world.set_block(first, BlockId::Stone);
        world.chunks.remove(&(0, 0));
        world.set_network_block(second, BlockId::Dirt);
        assert_eq!(world.get_block(first), BlockId::Stone);

        world.save(&path_str).expect("Should save world block overrides");
        let mut loaded = World::load(&path_str).expect("Should load world block overrides");
        loaded.chunks.remove(&(0, 0));
        loaded.set_network_block(IVec3::new(6, 63, 7), BlockId::Planks);

        assert_eq!(loaded.get_block(first), BlockId::Stone);
        assert_eq!(loaded.get_block(second), BlockId::Dirt);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn network_block_changes_track_local_edits_without_echoing_server_updates() {
        let mut world = World::new_with_seed(54321);
        world.chunks.insert((0, 0), Chunk::generate(0, 0, world.seed));
        let pos = IVec3::new(2, 63, 3);

        world.set_block(pos, BlockId::Stone);
        assert_eq!(
            world.take_network_changes().collect::<Vec<_>>(),
                   vec![(pos, BlockId::Stone.to_u8())]
        );

        world.set_block(pos, BlockId::Stone);
        assert_eq!(world.take_network_changes().count(), 0);

        world.set_network_block(pos, BlockId::Dirt);
        assert_eq!(world.take_network_changes().count(), 0);

        world.set_block(pos, BlockId::Air);
        assert_eq!(
            world.take_network_changes().collect::<Vec<_>>(),
                   vec![(pos, BlockId::Air.to_u8())]
        );
    }

    #[test]
    fn test_all_new_biomes_generation_and_rarity() {
        let mut counts = HashMap::new();
        for x in (-2000..=2000).step_by(40) {
            for z in (-2000..=2000).step_by(40) {
                *counts.entry(get_biome_at(x, z, 424242).0).or_insert(0) += 1;
            }
        }
        for b in [
            Biome::Jungle, Biome::Badlands, Biome::DarkForest, Biome::Taiga,
            Biome::BirchForest, Biome::IceSpikes, Biome::MushroomIsland,
            Biome::Meadow, Biome::JaggedPeaks, Biome::FrozenPeaks,
            Biome::SnowySlopes, Biome::River, Biome::FrozenRiver, Biome::Beach,
            Biome::ColdBeach, Biome::StoneBeach, Biome::SunflowerPlains,
            Biome::FlowerForest, Biome::MegaTaiga, Biome::JungleEdge,
            Biome::SavannaPlateau, Biome::Mesa, Biome::MesaPlateau, Biome::MesaPlateauForest,
            Biome::MesaBryce,
        ] {
            assert!(counts.contains_key(&b), "{:?} must generate", b);
        }
        assert!(counts[&Biome::MushroomIsland] < counts[&Biome::Plains]);
        assert!(counts[&Biome::IceSpikes] < counts[&Biome::Taiga]);
    }

    #[test]
    fn test_terrain_has_rolling_lowlands_high_peaks_and_carved_rivers() {
        let seed = 424242;
        let mut min_lowland = CHUNK_SIZE_Y;
        let mut max_lowland = 0;
        let mut max_highland = 0;
        let mut max_river = 0;
        let mut found_river = false;

        for x in (-2000..=2000).step_by(20) {
            for z in (-2000..=2000).step_by(20) {
                let (biome, continentalness) = get_biome_at(x, z, seed);
                let height = get_height_at(x, z, seed, biome, continentalness);
                if continentalness > 0.34 {
                    max_highland = max_highland.max(height);
                } else if continentalness >= -0.105 && !matches!(biome, Biome::River | Biome::FrozenRiver) {
                    min_lowland = min_lowland.min(height);
                    max_lowland = max_lowland.max(height);
                }
                if matches!(biome, Biome::River | Biome::FrozenRiver) {
                    found_river = true;
                    max_river = max_river.max(height);
                }
            }
        }

        assert!(max_lowland - min_lowland >= 8, "lowlands should have broad rolling relief");
        assert!(max_highland >= 45, "mountain ranges should rise above the lowlands");
        assert!(found_river, "river biomes should occur in the sampled region");
        assert!(max_river <= 20, "river channels should reach the sea-level basin");
    }

