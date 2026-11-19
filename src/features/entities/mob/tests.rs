//! Unit tests covering mob AI targeting, husbandry, spawning rules, network snapshots, and sounds.

use super::kind::MobKind;
use super::manager::{MobManager, MobPlayerContext};
use super::state::Mob;
use crate::audio::{SoundCategory, SoundId};
use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::BlockId;
use crate::features::world::{Chunk, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

#[test]
fn test_network_snapshot_replaces_mobs_and_preserves_state() {
    let mut manager = MobManager::new();
    manager.spawn_mob(MobKind::Pig, Vec3::zero());
    let snapshot = vec![crate::net::protocol::MobSyncEntry {
        id: 17,
        kind: MobKind::Sheep as u8,
        x: 4.0,
        y: 65.0,
        z: -2.0,
        yaw: 1.25,
        health: 3.0,
        is_dead: false,
        walk_phase: 2.5,
        is_baby: true,
        wool_color: 6,
        is_sheared: true,
        is_puffed: false,
        fuse: 0.0,
        is_ignited: false,
    }];

    manager.apply_network_snapshot(&snapshot).unwrap();

    assert_eq!(manager.mobs.len(), 1);
    assert_eq!(manager.mobs[0].id, 17);
    assert_eq!(manager.mobs[0].kind, MobKind::Sheep);
    assert_eq!(manager.mobs[0].pos, Vec3::new(4.0, 65.0, -2.0));
    assert_eq!(manager.mobs[0].health, 3.0);
    assert!(manager.mobs[0].is_baby);
    assert_eq!(manager.mobs[0].wool_color, 6);
    assert!(manager.mobs[0].is_sheared);

    assert_eq!(manager.spawn_mob(MobKind::Pig, Vec3::zero()), 18);
}

#[test]
fn test_mob_sounds_damage_and_death() {
    let mut pig = Mob::new(1, MobKind::Pig, Vec3::new(0.0, 10.0, 0.0));
    assert_eq!(pig.pending_sound, None);

    pig.take_damage(3.0, Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(pig.pending_sound, Some(SoundId::PigHurt));
    assert!(!pig.is_dead);
    let _ = pig.pending_sound.take();

    pig.take_damage(100.0, Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(pig.pending_sound, Some(SoundId::PigDeath));
    assert!(pig.is_dead);
}

#[test]
fn test_all_mob_kinds_sound_mapping() {
    let kinds = [
        MobKind::Pig,
        MobKind::Cow,
        MobKind::Chicken,
        MobKind::Sheep,
        MobKind::Zombie,
        MobKind::Skeleton,
        MobKind::Reaper,
        MobKind::Spider,
    ];
    for kind in kinds {
        let mut mob = Mob::new(42, kind, Vec3::zero());
        if let Some(snd) = kind.idle_sound() {
            assert_eq!(snd.category(), SoundCategory::Sfx);
        }
        assert_eq!(kind.hurt_sound().category(), SoundCategory::Sfx);
        assert_eq!(kind.death_sound().category(), SoundCategory::Sfx);

        mob.take_damage(1.0, Vec3::zero());
        assert_eq!(mob.pending_sound, Some(kind.hurt_sound()));

        mob.take_damage(999.0, Vec3::zero());
        assert_eq!(mob.pending_sound, Some(kind.death_sound()));
    }
}

#[test]
fn test_husbandry_tempting_items_and_baby_scaling() {
    assert!(MobKind::Cow.is_tempting_item(BlockId::Wheat));
    assert!(MobKind::Sheep.is_tempting_item(BlockId::Wheat));
    assert!(!MobKind::Cow.is_tempting_item(BlockId::Carrot));

    assert!(MobKind::Pig.is_tempting_item(BlockId::Carrot));
    assert!(MobKind::Pig.is_tempting_item(BlockId::Potato));
    assert!(MobKind::Pig.is_tempting_item(BlockId::BakedPotato));
    assert!(!MobKind::Pig.is_tempting_item(BlockId::Wheat));

    assert!(MobKind::Chicken.is_tempting_item(BlockId::WheatSeeds));
    assert!(!MobKind::Chicken.is_tempting_item(BlockId::Bread));

    let adult = Mob::new(1, MobKind::Cow, Vec3::new(0.0, 0.0, 0.0));
    let mut baby = Mob::new(2, MobKind::Cow, Vec3::new(0.0, 0.0, 0.0));
    baby.is_baby = true;
    let adult_aabb = adult.aabb();
    let baby_aabb = baby.aabb();
    assert!(baby_aabb.1.x - baby_aabb.0.x < adult_aabb.1.x - adult_aabb.0.x);
}

#[test]
fn test_sheep_shearing_and_grazing_regrowth() {
    let mut sheep = Mob::new(100, MobKind::Sheep, Vec3::zero());
    assert!(!sheep.is_sheared);
    assert!(sheep.wool_color <= 15);

    sheep.is_sheared = true;
    assert!(sheep.is_sheared);

    sheep.eat_grass_timer = 1.0;
    let world = World::new_with_seed(42);
    let mut rng = 12345;
    sheep.update(0.5, &world, &mut rng, None, None, None);
    assert!(sheep.is_sheared);
    assert!(sheep.eat_grass_timer > 0.0);

    sheep.update(0.6, &world, &mut rng, None, None, None);
    assert_eq!(sheep.eat_grass_timer, 0.0);
    assert!(!sheep.is_sheared, "Wool should regrow after grazing");
}

#[test]
fn test_mobs_do_not_see_player_in_creative() {
    let mut mob_manager = MobManager::new();
    let world = World::new_with_seed(12345);
    let player_pos = Vec3::new(5.0, 60.0, 5.0);

    let reaper_id = mob_manager.spawn_mob(MobKind::Reaper, Vec3::new(5.0, 60.0, 7.0));
    let zombie_id = mob_manager.spawn_mob(MobKind::Zombie, Vec3::new(5.0, 60.0, 8.0));

    mob_manager.update(0.05, &world, player_pos, None, true);

    let reaper = mob_manager.mobs.iter().find(|m| m.id == reaper_id).unwrap();
    assert!(!reaper.is_ignited, "Reaper must not ignite or see player in Creative mode");
    assert_eq!(reaper.fuse, 0.0, "Reaper fuse must stay 0 in Creative mode");

    let zombie = mob_manager.mobs.iter().find(|m| m.id == zombie_id).unwrap();
    let angle_to_player = (player_pos.x - zombie.pos.x).atan2(player_pos.z - zombie.pos.z);
    assert_ne!(zombie.target_yaw, angle_to_player, "Zombie must not target player in Creative mode");

    mob_manager.update(0.05, &world, player_pos, None, false);

    let reaper = mob_manager.mobs.iter().find(|m| m.id == reaper_id).unwrap();
    assert!(reaper.is_ignited, "Reaper should ignite when seeing player in Survival mode");

    let zombie = mob_manager.mobs.iter().find(|m| m.id == zombie_id).unwrap();
    let angle_to_player = (player_pos.x - zombie.pos.x).atan2(player_pos.z - zombie.pos.z);
    assert!((zombie.target_yaw - angle_to_player).abs() < 1e-4, "Zombie should target player in Survival mode");
}

#[test]
fn distant_player_keeps_nearby_mobs_simulated_and_targeted() {
    let mut manager = MobManager::new();
    let world = World::new_with_seed(1);
    let mob_id = manager.spawn_mob(MobKind::Zombie, Vec3::new(100.0, 60.0, 100.0));
    manager.update_for_players(
        0.05,
        &world,
        &[
            MobPlayerContext {
                position: Vec3::new(0.0, 60.0, 0.0),
                held_item: None,
                is_creative: false,
            },
            MobPlayerContext {
                position: Vec3::new(100.0, 60.0, 100.5),
                held_item: None,
                is_creative: true,
            },
            MobPlayerContext {
                position: Vec3::new(102.0, 60.0, 100.0),
                held_item: None,
                is_creative: false,
            },
        ],
    );

    let zombie = manager.mobs.iter().find(|mob| mob.id == mob_id).unwrap();
    assert!(!zombie.despawn_pending, "mob near the distant player must not despawn");
    assert!(
        (zombie.target_yaw - std::f32::consts::FRAC_PI_2).abs() < 1e-4,
        "hostile mob should target the nearest player"
    );
}

#[test]
fn test_golems_anger_and_pursue_player_who_attacked_villager() {
    let mut manager = MobManager::new();
    let world = World::new_with_seed(1);
    let villager_pos = Vec3::new(0.0, 60.0, 0.0);
    let golem_id = manager.spawn_mob(MobKind::IronGolem, Vec3::new(2.0, 60.0, 0.0));
    let distant_golem_id = manager.spawn_mob(MobKind::IronGolem, Vec3::new(30.0, 60.0, 0.0));

    manager.anger_golems_at_villager_attack(villager_pos);
    manager.update(0.05, &world, Vec3::new(10.0, 60.0, 0.0), None, false);

    let golem = manager.mobs.iter().find(|mob| mob.id == golem_id).unwrap();
    assert!(golem.player_anger_timer > 0.0);
    assert!((golem.target_yaw - std::f32::consts::FRAC_PI_2).abs() < 0.01);

    let distant_golem = manager.mobs.iter().find(|mob| mob.id == distant_golem_id).unwrap();
    assert_eq!(distant_golem.player_anger_timer, 0.0);
}

#[test]
fn test_zombies_attack_nearby_villagers() {
    let mut manager = MobManager::new();
    let world = World::new_with_seed(1);
    let villager_id = manager.spawn_mob(MobKind::Villager, Vec3::new(1.0, 60.0, 0.0));
    manager.spawn_mob(MobKind::Zombie, Vec3::new(0.0, 60.0, 0.0));

    manager.update(0.1, &world, Vec3::new(20.0, 60.0, 0.0), None, false);

    let villager = manager.mobs.iter().find(|mob| mob.id == villager_id).unwrap();
    assert_eq!(villager.health, MobKind::Villager.max_health() - 3.0);
}

#[test]
fn test_zombies_ignore_villagers_outside_target_range() {
    let mut manager = MobManager::new();
    let world = World::new_with_seed(1);
    let villager_id = manager.spawn_mob(MobKind::Villager, Vec3::new(30.0, 60.0, 0.0));
    manager.spawn_mob(MobKind::Zombie, Vec3::new(0.0, 60.0, 0.0));

    manager.update(0.1, &world, Vec3::new(40.0, 60.0, 0.0), None, false);

    let villager = manager.mobs.iter().find(|mob| mob.id == villager_id).unwrap();
    assert_eq!(villager.health, MobKind::Villager.max_health());
}

#[test]
fn test_ground_mob_steers_around_walls() {
    let mut world = World::new_with_seed(1);
    world.chunks.clear();
    world.chunks.insert(
        (0, 0),
        Chunk::new(
            0,
            0,
            vec![BlockId::Air.to_u8(); (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
            false,
        ),
    );
    for x in 0..CHUNK_SIZE_X {
        for z in 0..CHUNK_SIZE_Z {
            world.set_block(IVec3::new(x, 10, z), BlockId::Grass);
        }
    }
    for x in 7..=9 {
        for y in 11..=12 {
            world.set_block(IVec3::new(x, y, 9), BlockId::Stone);
        }
    }

    let mob = Mob::new(1, MobKind::Pig, Vec3::new(8.5, 11.0, 8.5));
    let detour_yaw = mob.navigation_yaw(&world, 0.0).expect("a clear route around the wall should exist");
    assert!(detour_yaw.abs() > 0.5, "mob should steer away from the blocked forward direction");

    let direct = Vec3::new(8.5, 11.0, 9.4);
    assert!(world.box_collides(
        Vec3::new(direct.x - 0.4, direct.y, direct.z - 0.4),
        Vec3::new(direct.x + 0.4, direct.y + MobKind::Pig.height(), direct.z + 0.4),
    ));
}

#[test]
fn test_skeleton_does_not_shoot_through_walls() {
    let mut world = World::new_with_seed(1);
    world.chunks.clear();
    world.chunks.insert(
        (0, 0),
        Chunk::new(
            0,
            0,
            vec![BlockId::Air.to_u8(); (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
            false,
        ),
    );
    for x in 0..CHUNK_SIZE_X {
        for z in 0..CHUNK_SIZE_Z {
            world.set_block(IVec3::new(x, 0, z), BlockId::Stone);
        }
    }
    for y in 1..=3 {
        world.set_block(IVec3::new(4, y, 2), BlockId::Stone);
    }

    let mut manager = MobManager::new();
    manager.spawn_mob(MobKind::Skeleton, Vec3::new(2.5, 1.0, 2.5));
    manager.update(0.05, &world, Vec3::new(8.5, 1.0, 2.5), None, false);

    assert!(manager.pending_arrows.is_empty());
    assert_eq!(manager.mobs[0].attack_cooldown, 0.0);
}

#[test]
fn test_villagers_return_to_their_home_after_wandering() {
    let mut world = World::new_with_seed(1);
    world.chunks.clear();
    world.chunks.insert(
        (0, 0),
        Chunk::new(
            0,
            0,
            vec![BlockId::Air.to_u8(); (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
            false,
        ),
    );
    for x in 0..CHUNK_SIZE_X {
        for z in 0..CHUNK_SIZE_Z {
            world.set_block(IVec3::new(x, 10, z), BlockId::Grass);
        }
    }

    let home = Vec3::new(8.5, 11.0, 8.5);
    let mut villager = Mob::new(1, MobKind::Villager, home);
    let mut rng = 12345;
    let mut max_distance_sq: f32 = 0.0;
    for _ in 0..600 {
        villager.update(0.05, &world, &mut rng, None, None, None);
        let (dx, dz) = (villager.pos.x - home.x, villager.pos.z - home.z);
        max_distance_sq = max_distance_sq.max(dx * dx + dz * dz);
    }

    assert!(max_distance_sq < 17.0 * 17.0);
}

#[test]
fn test_daytime_natural_spawns_stay_underground() {
    let mut world = World::new_with_seed(1);
    world.chunks.clear();
    world.time_of_day = 120.0;
    for cx in -3..=2 {
        for cz in -3..=2 {
            let mut chunk = Chunk::new(
                cx,
                cz,
                vec![0; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
                false,
            );
            for x in 0..CHUNK_SIZE_X {
                for z in 0..CHUNK_SIZE_Z {
                    chunk.set_local(x, 20, z, BlockId::Grass);
                }
            }
            world.chunks.insert((cx, cz), chunk);
        }
    }

    let mut manager = MobManager::new();
    let mut rng = 12345;
    for _ in 0..100 {
        manager.try_natural_spawn(&world, Vec3::new(0.0, 30.0, 0.0), &mut rng);
    }

    assert!(manager.mobs.is_empty(), "Natural daytime spawns must not appear on the surface");
}
