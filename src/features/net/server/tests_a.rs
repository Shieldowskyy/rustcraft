use std::thread;
use std::time::{Duration, Instant};

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::entities::{MobKind, MobManager};
use crate::features::net::client::{NetworkClient, NetworkEvent};
use crate::features::net::protocol::{MobSyncEntry, PlayerData, PlayerItemStack};

use super::commands_chat::parse_time_value;
use super::config::{load_player_data, player_data_path, save_player_data};
use super::runner::{EmbeddedServer, EmbeddedServerState};

#[test]
fn player_data_is_persisted_by_case_insensitive_nickname() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let data_dir = std::env::temp_dir().join(format!(
        "rustcraft_playerdata_{}_{}",
        std::process::id(),
        unique
    ));
    let data = PlayerData {
        x: 12.5,
        y: 70.0,
        z: -6.0,
        health: 16.0,
        max_health: 20.0,
        hunger: 14.0,
        max_hunger: 20.0,
        saturation: 2.0,
        exhaustion: 1.0,
        air: 18.0,
        selected_hotbar_slot: 2,
        hotbar: [Some(PlayerItemStack {
            block: BlockId::Diamond.to_u8(),
            count: 5,
            durability: 42,
        }); 10],
        main: [None; 27],
        armor: [None; 4],
        offhand: None,
        spawn_point: Some((3, 64, 8)),
    };

    save_player_data(&data_dir, "RelayUser", &data).unwrap();
    assert_eq!(load_player_data(&data_dir, "relayuser").unwrap(), Some(data));
    assert_eq!(load_player_data(&data_dir, "AnotherUser").unwrap(), None);
    std::fs::remove_file(player_data_path(&data_dir, "RelayUser")).unwrap();
    std::fs::remove_dir(&data_dir).unwrap();
}

#[test]
fn server_updates_player_list_with_measured_ping() {
    let server = EmbeddedServer::start(
        0,
        EmbeddedServerState {
            seed: 1,
            time_of_day: 120.0,
            spawn_pos: (8.5, 65.0, 8.5),
            blocks: Vec::new(),
            mobs: Vec::new(),
        },
    )
    .unwrap();
    let mut client =
        NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "PingTest").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline && client.ping_ms == 0 {
        client.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }

    assert!(client.is_connected);
    assert!(client.ping_ms > 0);
    client.disconnect();
    drop(server);
}

#[test]
fn non_authority_mob_attacks_are_sent_to_the_mob_authority() {
    let server = EmbeddedServer::start(
        0,
        EmbeddedServerState {
            seed: 1,
            time_of_day: 120.0,
            spawn_pos: (8.5, 48.0, 8.5),
            blocks: Vec::new(),
            mobs: vec![MobSyncEntry {
                id: 7,
                kind: MobKind::Zombie as u8,
                x: 9.0,
                y: 48.0,
                z: 8.5,
                yaw: 0.0,
                health: 20.0,
                is_dead: false,
                walk_phase: 0.0,
                is_baby: false,
                wool_color: 0,
                is_sheared: false,
                is_puffed: false,
                fuse: 0.0,
                is_ignited: false,
            }],
        },
    )
    .unwrap();
    let address = format!("127.0.0.1:{}", server.port);
    let mut authority = NetworkClient::connect(&address, "Authority").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline
        && (!authority.is_connected || authority.mob_authority_id != authority.local_id)
    {
        authority.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(authority.is_connected);
    assert_eq!(authority.mob_authority_id, authority.local_id);

    let mut attacker = NetworkClient::connect(&address, "Attacker").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline
        && (!authority.is_connected
            || !attacker.is_connected
            || attacker.mob_authority_id != authority.local_id)
    {
        authority.update(0.05);
        attacker.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(authority.is_connected && attacker.is_connected);
    assert_eq!(attacker.mob_authority_id, authority.local_id);

    attacker.send_attack_mob(7, 6.5);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut replicated = false;
    while Instant::now() < deadline && !replicated {
        replicated |= authority.update(0.05).into_iter().any(|event| {
            matches!(
                event,
                NetworkEvent::MobAttack {
                    attacker_id,
                    mob_id: 7,
                    damage: 6.5,
                    ..
                } if Some(attacker_id) == attacker.local_id
            )
        });
        attacker.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        replicated,
        "mob authority should receive the remote attack"
    );

    authority.disconnect();
    attacker.disconnect();
    drop(server);
}

#[test]
fn non_authority_mob_spawns_are_sent_to_the_mob_authority() {
    let server = EmbeddedServer::start(
        0,
        EmbeddedServerState {
            seed: 1,
            time_of_day: 120.0,
            spawn_pos: (8.5, 65.0, 8.5),
            blocks: Vec::new(),
            mobs: Vec::new(),
        },
    )
    .unwrap();
    let address = format!("127.0.0.1:{}", server.port);
    let mut authority = NetworkClient::connect(&address, "SpawnAuthority").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline
        && (!authority.is_connected || authority.mob_authority_id != authority.local_id)
    {
        authority.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(authority.is_connected);
    assert_eq!(authority.mob_authority_id, authority.local_id);

    let mut guest = NetworkClient::connect(&address, "SpawnGuest").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline
        && (!authority.is_connected
            || !guest.is_connected
            || guest.mob_authority_id != authority.local_id)
    {
        authority.update(0.05);
        guest.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(authority.is_connected && guest.is_connected);
    assert_eq!(guest.mob_authority_id, authority.local_id);

    let spawn_pos = Vec3::new(9.0, 65.0, 8.5);
    guest.send_spawn_mob(MobKind::Zombie, spawn_pos);
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut received = false;
    let mut replicated = false;
    let mut mob_manager = MobManager::new();
    while Instant::now() < deadline && !replicated {
        for event in authority.update(0.05) {
            if let NetworkEvent::MobSpawn { kind, pos } = event {
                if kind == MobKind::Zombie as u8 && pos == spawn_pos {
                    mob_manager.spawn_mob(MobKind::Zombie, pos);
                    received = true;
                    authority.send_mob_snapshot_now(&mob_manager);
                }
            }
        }
        replicated |= guest.update(0.05).into_iter().any(|event| {
            matches!(
                event,
                NetworkEvent::MobSnapshot { mobs }
                    if mobs.iter().any(|mob| {
                        mob.kind == MobKind::Zombie as u8
                            && mob.x == spawn_pos.x
                            && mob.y == spawn_pos.y
                            && mob.z == spawn_pos.z
                    })
            )
        });
        thread::sleep(Duration::from_millis(10));
    }
    assert!(received, "mob authority should receive the client's spawn request");
    assert!(replicated, "other clients should receive the authoritative mob snapshot");

    authority.disconnect();
    guest.disconnect();
    drop(server);
}

#[test]
fn in_game_time_set_command_syncs_all_clients() {
    let server = EmbeddedServer::start(
        0,
        EmbeddedServerState {
            seed: 1,
            time_of_day: 42.0,
            spawn_pos: (8.5, 48.0, 8.5),
            blocks: Vec::new(),
            mobs: Vec::new(),
        },
    )
    .unwrap();
    let address = format!("127.0.0.1:{}", server.port);
    let mut host = NetworkClient::connect(&address, "TimeHost").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && !host.is_connected {
        host.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    let mut guest = NetworkClient::connect(&address, "TimeGuest").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && (!host.is_connected || !guest.is_connected) {
        host.update(0.05);
        guest.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(host.is_connected && guest.is_connected);

    host.send_chat("/time set day");
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut host_synced = false;
    let mut guest_synced = false;
    while Instant::now() < deadline && !(host_synced && guest_synced) {
        host_synced |= host.update(0.05).into_iter().any(|event| {
            matches!(event, NetworkEvent::TimeSync { time_of_day } if time_of_day == 120.0)
        });
        guest_synced |= guest.update(0.05).into_iter().any(|event| {
            matches!(event, NetworkEvent::TimeSync { time_of_day } if time_of_day == 120.0)
        });
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        host_synced && guest_synced,
        "server should broadcast the requested time"
    );

    host.disconnect();
    guest.disconnect();
    drop(server);
}

#[test]
fn in_game_tps_command_reports_server_metrics_to_requesting_client() {
    let server = EmbeddedServer::start(
        0,
        EmbeddedServerState {
            seed: 1,
            time_of_day: 42.0,
            spawn_pos: (8.5, 48.0, 8.5),
            blocks: Vec::new(),
            mobs: Vec::new(),
        },
    )
    .unwrap();
    let address = format!("127.0.0.1:{}", server.port);
    let mut client = NetworkClient::connect(&address, "TpsGuest").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && !client.is_connected {
        client.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(client.is_connected);

    client.send_chat("/tps");
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut received_metrics = false;
    while Instant::now() < deadline && !received_metrics {
        received_metrics = client.update(0.05).into_iter().any(|event| {
            matches!(
                event,
                NetworkEvent::ChatMessage { text, is_system: true, .. }
                    if text.contains("Server TPS:") && text.contains("MSPT:")
            )
        });
        thread::sleep(Duration::from_millis(10));
    }
    assert!(received_metrics, "server should respond with TPS and MSPT");

    client.disconnect();
    drop(server);
}

#[test]
fn time_values_accept_named_and_numeric_ticks() {
    assert_eq!(parse_time_value("day"), Some(120.0));
    assert_eq!(parse_time_value("noon"), Some(240.0));
    assert_eq!(parse_time_value("night"), Some(360.0));
    assert_eq!(parse_time_value("midnight"), Some(0.0));
    assert_eq!(parse_time_value("12000"), Some(240.0));
    assert_eq!(parse_time_value("NaN"), None);
}
