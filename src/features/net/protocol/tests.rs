use super::*;
use crate::features::entities::MobKind;

#[test]
fn test_client_packets_roundtrip() {
    let handshake = ClientPacket::Handshake {
        protocol_version: PROTOCOL_VERSION,
        nickname: "TestMiner123".to_string(),
    };
    let mut framed = handshake.serialize();
    let packets = extract_packets(&mut framed).unwrap();
    assert_eq!(packets.len(), 1);
    let decoded = ClientPacket::deserialize(&packets[0]).unwrap();
    assert_eq!(decoded, handshake);

    let pmove = ClientPacket::PlayerMove {
        x: 10.5,
        y: 64.0,
        z: -20.25,
        yaw: 1.57,
        pitch: -0.2,
        is_sneaking: true,
        is_sprinting: false,
    };
    let mut framed2 = pmove.serialize();
    let packets2 = extract_packets(&mut framed2).unwrap();
    assert_eq!(ClientPacket::deserialize(&packets2[0]).unwrap(), pmove);

    let attack = ClientPacket::AttackPlayer {
        target_id: 101,
        damage: 6.5,
    };
    let mut framed_atk = attack.serialize();
    let packets_atk = extract_packets(&mut framed_atk).unwrap();
    assert_eq!(ClientPacket::deserialize(&packets_atk[0]).unwrap(), attack);

    let attack_mob = ClientPacket::AttackMob {
        mob_id: 8,
        damage: 6.5,
    };
    let mut framed_mob_attack = attack_mob.serialize();
    let packets_mob_attack = extract_packets(&mut framed_mob_attack).unwrap();
    assert_eq!(
        ClientPacket::deserialize(&packets_mob_attack[0]).unwrap(),
        attack_mob
    );

    let spawn_mob = ClientPacket::SpawnMob {
        kind: MobKind::Zombie as u8,
        x: 12.5,
        y: 64.0,
        z: -7.25,
    };
    let mut framed_spawn = spawn_mob.serialize();
    let packets_spawn = extract_packets(&mut framed_spawn).unwrap();
    assert_eq!(
        ClientPacket::deserialize(&packets_spawn[0]).unwrap(),
        spawn_mob
    );

    let mob_attack = ServerPacket::MobAttack {
        attacker_id: 2,
        mob_id: 8,
        damage: 6.5,
        direction_x: 0.0,
        direction_y: 0.1,
        direction_z: 1.0,
    };
    let mut framed_server_mob_attack = mob_attack.serialize();
    let packets_server_mob_attack = extract_packets(&mut framed_server_mob_attack).unwrap();
    assert_eq!(
        ServerPacket::deserialize(&packets_server_mob_attack[0]).unwrap(),
        mob_attack
    );

    let mob_spawn = ServerPacket::MobSpawn {
        kind: MobKind::Zombie as u8,
        x: 12.5,
        y: 64.0,
        z: -7.25,
    };
    let mut framed_server_spawn = mob_spawn.serialize();
    let packets_server_spawn = extract_packets(&mut framed_server_spawn).unwrap();
    assert_eq!(
        ServerPacket::deserialize(&packets_server_spawn[0]).unwrap(),
        mob_spawn
    );

    let mob_snapshot = ClientPacket::MobSnapshot {
        mobs: vec![MobSyncEntry {
            id: 8,
            kind: 2,
            x: 1.0,
            y: 64.0,
            z: -3.0,
            yaw: 0.5,
            health: 4.0,
            is_dead: false,
            walk_phase: 1.25,
            is_baby: false,
            wool_color: 0,
            is_sheared: false,
            is_puffed: false,
            fuse: 0.0,
            is_ignited: false,
        }],
    };
    let mut framed_mobs = mob_snapshot.serialize();
    let packets_mobs = extract_packets(&mut framed_mobs).unwrap();
    assert_eq!(
        ClientPacket::deserialize(&packets_mobs[0]).unwrap(),
        mob_snapshot
    );

    let mut framed_death = ClientPacket::PlayerDeath.serialize();
    let packets_death = extract_packets(&mut framed_death).unwrap();
    assert_eq!(
        ClientPacket::deserialize(&packets_death[0]).unwrap(),
        ClientPacket::PlayerDeath
    );
    let pong = ClientPacket::Pong { seq: 42 };
    let mut framed_pong = pong.serialize();
    let packets_pong = extract_packets(&mut framed_pong).unwrap();
    assert_eq!(ClientPacket::deserialize(&packets_pong[0]).unwrap(), pong);

    let player_data = PlayerData {
        x: 2.0,
        y: 70.0,
        z: -4.0,
        health: 17.0,
        max_health: 20.0,
        hunger: 13.0,
        max_hunger: 20.0,
        saturation: 2.5,
        exhaustion: 0.5,
        air: 20.0,
        selected_hotbar_slot: 3,
        hotbar: [Some(PlayerItemStack {
            block: 1,
            count: 32,
            durability: 12,
        }); 10],
        main: [None; 27],
        armor: [None; 4],
        offhand: None,
        spawn_point: Some((10, 64, -8)),
    };
    let packet = ClientPacket::PlayerData {
        data: player_data.clone(),
    };
    let mut framed_data = packet.serialize();
    let packets_data = extract_packets(&mut framed_data).unwrap();
    assert_eq!(ClientPacket::deserialize(&packets_data[0]).unwrap(), packet);
}

#[test]
fn test_server_packets_roundtrip() {
    let login = ServerPacket::LoginSuccess {
        player_id: 42,
        seed: 987654321,
        spawn_x: 8.5,
        spawn_y: 65.0,
        spawn_z: 8.5,
        time_of_day: 120.0,
        modified_blocks: vec![(1, 2, 3, 4), (10, 20, 30, 0)],
    };
    let mut framed = login.serialize();
    let packets = extract_packets(&mut framed).unwrap();
    assert_eq!(packets.len(), 1);
    assert_eq!(ServerPacket::deserialize(&packets[0]).unwrap(), login);

    let hurt = ServerPacket::PlayerHurt {
        victim_id: 7,
        attacker_id: 3,
        damage: 4.5,
        knockback_x: 0.3,
        knockback_z: -0.2,
    };
    let mut framed_hurt = hurt.serialize();
    let packets_hurt = extract_packets(&mut framed_hurt).unwrap();
    assert_eq!(ServerPacket::deserialize(&packets_hurt[0]).unwrap(), hurt);

    let chat = ServerPacket::ChatMessage {
        sender: "SwiftMiner".to_string(),
        text: "Hello RustCraft multiplayer!".to_string(),
        is_system: false,
    };
    let mut framed_chat = chat.serialize();
    let packets_chat = extract_packets(&mut framed_chat).unwrap();
    assert_eq!(ServerPacket::deserialize(&packets_chat[0]).unwrap(), chat);

    let mod_sync = ServerPacket::ModSync {
        mods: vec![ModSyncEntry {
            mod_id: "emerald_expansion".to_string(),
            content: r#"{"id":"emerald_expansion","name":"Emerald Expansion"}"#.to_string(),
        }],
    };
    let mut framed_mod_sync = mod_sync.serialize();
    let packets_mod_sync = extract_packets(&mut framed_mod_sync).unwrap();
    assert_eq!(
        ServerPacket::deserialize(&packets_mod_sync[0]).unwrap(),
        mod_sync
    );

    let authority = ServerPacket::MobAuthority { id: 3 };
    let mut framed_authority = authority.serialize();
    let packets_authority = extract_packets(&mut framed_authority).unwrap();
    assert_eq!(
        ServerPacket::deserialize(&packets_authority[0]).unwrap(),
        authority
    );
    let ping = ServerPacket::Ping { seq: 42 };
    let mut framed_ping = ping.serialize();
    let packets_ping = extract_packets(&mut framed_ping).unwrap();
    assert_eq!(ServerPacket::deserialize(&packets_ping[0]).unwrap(), ping);

    let mob_snapshot = ServerPacket::MobSnapshot {
        mobs: vec![MobSyncEntry {
            id: 8,
            kind: 2,
            x: 1.0,
            y: 64.0,
            z: -3.0,
            yaw: 0.5,
            health: 4.0,
            is_dead: false,
            walk_phase: 1.25,
            is_baby: false,
            wool_color: 0,
            is_sheared: false,
            is_puffed: false,
            fuse: 0.0,
            is_ignited: false,
        }],
    };
    let mut framed_mobs = mob_snapshot.serialize();
    let packets_mobs = extract_packets(&mut framed_mobs).unwrap();
    assert_eq!(
        ServerPacket::deserialize(&packets_mobs[0]).unwrap(),
        mob_snapshot
    );

    let player_data = ServerPacket::PlayerData {
        data: Some(PlayerData {
            x: 2.0,
            y: 70.0,
            z: -4.0,
            health: 17.0,
            max_health: 20.0,
            hunger: 13.0,
            max_hunger: 20.0,
            saturation: 2.5,
            exhaustion: 0.5,
            air: 20.0,
            selected_hotbar_slot: 3,
            hotbar: [Some(PlayerItemStack {
                block: 1,
                count: 32,
                durability: 12,
            }); 10],
            main: [None; 27],
            armor: [None; 4],
            offhand: None,
            spawn_point: Some((10, 64, -8)),
        }),
    };
    let mut framed_data = player_data.serialize();
    let packets_data = extract_packets(&mut framed_data).unwrap();
    assert_eq!(
        ServerPacket::deserialize(&packets_data[0]).unwrap(),
        player_data
    );
}

#[test]
fn world_snapshot_packets_roundtrip_and_reject_non_finite_values() {
    let snapshot = WorldSnapshot {
        items: vec![WorldItemEntry {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            velocity_x: 0.1,
            velocity_y: 0.2,
            velocity_z: 0.3,
            block: 7,
            count: 4,
            pickup_delay: 0.5,
            age: 12.0,
        }],
        chests: vec![ChestSnapshot {
            x: 4,
            y: 5,
            z: 6,
            items: [None; 27],
        }],
        furnaces: vec![FurnaceSnapshot {
            x: 7,
            y: 8,
            z: 9,
            input: Some(PlayerItemStack {
                block: 10,
                count: 2,
                durability: 30,
            }),
            fuel: None,
            output: None,
            burn_time_remaining: 3.0,
            burn_time_total: 4.0,
            cook_progress: 5.0,
        }],
        primed_tnts: vec![PrimedTntEntry {
            x: 1.5,
            y: 2.5,
            z: 3.5,
            velocity_x: 0.0,
            velocity_y: 2.0,
            velocity_z: 0.0,
            fuse: 1.25,
        }],
        arrows: vec![ArrowEntry {
            x: 6.0,
            y: 7.0,
            z: 8.0,
            velocity_x: 1.0,
            velocity_y: 2.0,
            velocity_z: 3.0,
            shooter_is_player: true,
            in_ground: false,
            lifetime: 0.75,
            damage: 6.0,
        }],
        weather: WeatherSnapshot {
            current_type: 2,
            timer: 20.0,
            rain_strength: 0.8,
            thunder_strength: 0.6,
            lightning_timer: 1.5,
            lightning_flash: 0.25,
            rng_state: 123456789,
            active_lightning: vec![LightningEntry {
                x: 10.0,
                y: 64.0,
                z: -5.0,
                timer: 0.1,
                max_duration: 0.2,
                seed: 42,
            }],
        },
    };
    let client_packet = ClientPacket::WorldSnapshot {
        snapshot: snapshot.clone(),
    };
    let mut frame = client_packet.serialize();
    let payload = extract_packets(&mut frame).unwrap().pop().unwrap();
    assert_eq!(ClientPacket::deserialize(&payload).unwrap(), client_packet);

    let server_packet = ServerPacket::WorldSnapshot { snapshot };
    let mut frame = server_packet.serialize();
    let payload = extract_packets(&mut frame).unwrap().pop().unwrap();
    assert_eq!(ServerPacket::deserialize(&payload).unwrap(), server_packet);

    let invalid = ClientPacket::WorldSnapshot {
        snapshot: WorldSnapshot {
            items: vec![WorldItemEntry {
                x: f32::NAN,
                y: 0.0,
                z: 0.0,
                velocity_x: 0.0,
                velocity_y: 0.0,
                velocity_z: 0.0,
                block: 1,
                count: 1,
                pickup_delay: 0.0,
                age: 0.0,
            }],
            ..WorldSnapshot::default()
        },
    };
    let mut frame = invalid.serialize();
    let payload = extract_packets(&mut frame).unwrap().pop().unwrap();
    assert_eq!(
        ClientPacket::deserialize(&payload),
        Err("World snapshot contains non-finite values")
    );
}

#[test]
fn packet_extraction_handles_fragmented_and_oversized_frames() {
    let mut frame = ClientPacket::Ping { seq: 42 }.serialize();
    let suffix = frame.split_off(6);
    assert!(extract_packets(&mut frame).unwrap().is_empty());
    frame.extend_from_slice(&suffix);
    assert_eq!(
        ClientPacket::deserialize(&extract_packets(&mut frame).unwrap()[0]).unwrap(),
        ClientPacket::Ping { seq: 42 }
    );

    let mut oversized = ((MAX_PACKET_SIZE as u32) + 1).to_le_bytes().to_vec();
    assert_eq!(
        extract_packets(&mut oversized),
        Err("Invalid packet frame length")
    );
    assert!(oversized.is_empty());
}
