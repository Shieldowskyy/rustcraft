use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::net::client::{NetworkClient, NetworkEvent};
use crate::features::net::protocol::ClientPacket;

use super::config::ServerConfig;
use super::runner::{EmbeddedServer, EmbeddedServerState};

#[test]
fn unauthenticated_clients_cannot_change_replicated_blocks() {
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
    let mut unauthenticated = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    unauthenticated
        .write_all(
            &ClientPacket::BlockChange {
                x: 4,
                y: 20,
                z: 8,
                block: BlockId::DiamondBlock.to_u8(),
            }
            .serialize(),
        )
        .unwrap();
    drop(unauthenticated);
    thread::sleep(Duration::from_millis(100));

    let mut observer =
        NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "Observer").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut modified_blocks = None;
    while Instant::now() < deadline && modified_blocks.is_none() {
        for event in observer.update(0.05) {
            if let NetworkEvent::Connected {
                modified_blocks: blocks,
                ..
            } = event
            {
                modified_blocks = Some(blocks);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        modified_blocks.expect("observer should connect"),
        Vec::<(i32, i32, i32, u8)>::new()
    );
    observer.disconnect();
    drop(server);
}

#[test]
fn in_game_teleport_command_moves_player_to_named_target() {
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
    let mut host = NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "Host").unwrap();
    let mut guest = NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "Guest").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && (!host.is_connected || !guest.is_connected) {
        host.update(0.05);
        guest.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(host.is_connected && guest.is_connected);

    host.send_position(Vec3::new(31.0, 72.0, -8.0), 0.0, 0.0, false, false);
    thread::sleep(Duration::from_millis(100));
    host.send_chat("/op Guest");
    thread::sleep(Duration::from_millis(200));
    guest.send_chat("/tp Host");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut teleport = None;
    while Instant::now() < deadline && teleport.is_none() {
        for event in guest.update(0.05) {
            if let NetworkEvent::Teleport { x, y, z } = event {
                teleport = Some((x, y, z));
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(teleport, Some((31.0, 72.0, -8.0)));
    host.disconnect();
    guest.disconnect();
    drop(server);
}

#[test]
fn embedded_host_can_kick_a_player_with_a_visible_reason() {
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
    let mut host = NetworkClient::connect(&address, "Host").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && !host.is_connected {
        host.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(host.is_connected);

    let mut guest = NetworkClient::connect(&address, "Guest").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && !guest.is_connected {
        host.update(0.05);
        guest.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(guest.is_connected);

    guest.send_chat("/kick Host unauthorized");
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut permission_denied = false;
    while Instant::now() < deadline && !permission_denied {
        host.update(0.05);
        for event in guest.update(0.05) {
            if let NetworkEvent::ChatMessage { text, .. } = event {
                permission_denied = text.contains("do not have permission");
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(permission_denied);
    assert!(host.is_connected && guest.is_connected);

    host.send_chat("/kick Guest Testing kick reason");
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut disconnect_reason = None;
    while Instant::now() < deadline && disconnect_reason.is_none() {
        host.update(0.05);
        for event in guest.update(0.05) {
            if let NetworkEvent::Disconnected { reason } = event {
                disconnect_reason = Some(reason);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert_eq!(disconnect_reason.as_deref(), Some("Testing kick reason"));
    host.disconnect();
    drop(server);
}

#[test]
fn player_death_is_broadcast_to_all_connected_clients() {
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
    let mut player = NetworkClient::connect(&address, "PlayerOne").unwrap();
    let mut observer = NetworkClient::connect(&address, "Observer").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline && (!player.is_connected || !observer.is_connected) {
        player.update(0.05);
        observer.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(player.is_connected && observer.is_connected);

    player.send_player_death();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut player_notified = false;
    let mut observer_notified = false;
    while Instant::now() < deadline && !(player_notified && observer_notified) {
        for event in player.update(0.05) {
            player_notified |= matches!(
                event,
                NetworkEvent::ChatMessage { ref text, is_system: true, .. }
                    if text == "PlayerOne died"
            );
        }
        for event in observer.update(0.05) {
            observer_notified |= matches!(
                event,
                NetworkEvent::ChatMessage { ref text, is_system: true, .. }
                    if text == "PlayerOne died"
            );
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(player_notified && observer_notified);
    player.disconnect();
    observer.disconnect();
    drop(server);
}

#[test]
fn test_server_config_defaults_and_parse() {
    let default_cfg = ServerConfig::default();
    assert_eq!(default_cfg.server_port, 26656);
    assert_eq!(default_cfg.max_players, 20);
    assert!(default_cfg.pvp);
    assert_eq!(default_cfg.gamemode, "survival");

    let test_props = "\
# Test server properties
server-port=25565
level-seed=424242
motd=Custom Test MOTD
max-players=64
pvp=false
gamemode=creative
difficulty=hard
view-distance=12
spawn-x=10.0
spawn-y=70.0
spawn-z=20.0
";
    let config_path = std::env::temp_dir().join(format!(
        "test_server_props_{}.properties",
        std::process::id()
    ));
    std::fs::write(&config_path, test_props).unwrap();

    let loaded = ServerConfig::load_or_create(config_path.to_str().unwrap());
    assert_eq!(loaded.server_port, 25565);
    assert_eq!(loaded.level_seed, 424242);
    assert_eq!(loaded.motd, "Custom Test MOTD");
    assert_eq!(loaded.max_players, 64);
    assert!(!loaded.pvp);
    assert_eq!(loaded.gamemode, "creative");
    assert_eq!(loaded.difficulty, "hard");
    assert_eq!(loaded.view_distance, 12);
    assert_eq!(loaded.spawn_pos, (10.0, 70.0, 20.0));

    let _ = std::fs::remove_file(&config_path);
}

#[test]
fn test_server_config_creates_missing_file() {
    let config_path = std::env::temp_dir().join(format!(
        "test_missing_props_{}.properties",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&config_path);
    assert!(!config_path.exists());

    let created = ServerConfig::load_or_create(config_path.to_str().unwrap());
    assert!(config_path.exists());
    assert_eq!(created.server_port, 26656);
    assert_eq!(created.max_players, 20);

    let _ = std::fs::remove_file(&config_path);
}
