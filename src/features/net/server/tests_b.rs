use std::thread;
use std::time::{Duration, Instant};

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::entities::MobKind;
use crate::features::net::client::{NetworkClient, NetworkEvent};
use crate::features::net::protocol::{ClientPacket, MobSyncEntry, WorldItemEntry, WorldSnapshot};

use super::runner::{EmbeddedServer, EmbeddedServerState};

#[test]
fn authoritative_world_snapshots_reach_other_clients_and_late_joiners() {
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
    let mut host = NetworkClient::connect(&address, "WorldHost").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline
        && (!host.is_connected || host.mob_authority_id != host.local_id)
    {
        host.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(host.is_connected);
    assert_eq!(host.mob_authority_id, host.local_id);

    let mut observer = NetworkClient::connect(&address, "WorldObserver").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline
        && (!host.is_connected
            || !observer.is_connected
            || observer.mob_authority_id != host.local_id)
    {
        host.update(0.05);
        observer.update(0.05);
        thread::sleep(Duration::from_millis(10));
    }
    assert!(host.is_connected && observer.is_connected);
    assert_eq!(observer.mob_authority_id, host.local_id);

    let snapshot = WorldSnapshot {
        items: vec![WorldItemEntry {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            velocity_x: 0.0,
            velocity_y: 1.0,
            velocity_z: 0.0,
            block: BlockId::Stone.to_u8(),
            count: 2,
            pickup_delay: 0.5,
            age: 1.0,
        }],
        ..WorldSnapshot::default()
    };

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut received = false;
    host.send_packet(&ClientPacket::WorldSnapshot {
        snapshot: snapshot.clone(),
    })
    .unwrap();
    while Instant::now() < deadline && !received {
        host.update(0.05);
        received |= observer.update(0.05).into_iter().any(|event| {
            matches!(event, NetworkEvent::WorldSnapshot { snapshot: actual } if actual == snapshot)
        });
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        received,
        "observer should receive the authority's world snapshot"
    );

    observer.disconnect();
    thread::sleep(Duration::from_millis(200));
    let mut late_joiner = NetworkClient::connect(&address, "WorldLateJoiner").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut late_joiner_received = false;
    while Instant::now() < deadline && !late_joiner_received {
        host.update(0.05);
        late_joiner_received |= late_joiner.update(0.05).into_iter().any(|event| {
            matches!(event, NetworkEvent::WorldSnapshot { snapshot: actual } if actual == snapshot)
        });
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        late_joiner_received,
        "late joiner should receive the latest snapshot"
    );

    late_joiner.disconnect();
    host.disconnect();
    drop(server);
}

#[test]
fn embedded_server_uses_world_snapshot_and_stops_cleanly() {
    let initial_mob = MobSyncEntry {
        id: 7,
        kind: MobKind::Zombie as u8,
        x: 4.0,
        y: 65.0,
        z: -2.0,
        yaw: 0.5,
        health: 20.0,
        is_dead: false,
        walk_phase: 1.0,
        is_baby: false,
        wool_color: 0,
        is_sheared: false,
        is_puffed: false,
        fuse: 0.0,
        is_ignited: false,
    };
    let server = EmbeddedServer::start(
        0,
        EmbeddedServerState {
            seed: 98765,
            time_of_day: 42.0,
            spawn_pos: (3.5, 65.0, -2.5),
            blocks: vec![(3, 10, -2, BlockId::Stone.to_u8())],
            mobs: vec![initial_mob.clone()],
        },
    )
    .unwrap();
    let mut client = NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "Host").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut connected = None;
    let mut host_received_mob_snapshot = false;
    while Instant::now() < deadline {
        for event in client.update(0.05) {
            match event {
                NetworkEvent::Connected {
                    seed,
                    time_of_day,
                    modified_blocks,
                    ..
                } => connected = Some((seed, time_of_day, modified_blocks)),
                NetworkEvent::MobSnapshot { .. } => {
                    host_received_mob_snapshot = true;
                }
                _ => {}
            }
        }
        if connected.is_some() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let (seed, time_of_day, modified_blocks) = connected.expect("host client should log in");
    assert_eq!(seed, 98765);
    assert_eq!(time_of_day, 42.0);
    assert_eq!(modified_blocks, vec![(3, 10, -2, BlockId::Stone.to_u8())]);

    assert!(!host_received_mob_snapshot);
    let host_position = Vec3::new(31.0, 72.0, -8.0);
    client.send_position(host_position, 0.3, -0.1, false, true);
    thread::sleep(Duration::from_millis(100));
    client.update(0.05);
    let mut observer =
        NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "Observer").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut observer_received_mobs = false;
    while Instant::now() < deadline && !observer_received_mobs {
        client.update(0.05);
        for event in observer.update(0.05) {
            if let NetworkEvent::MobSnapshot { mobs } = event {
                observer_received_mobs = mobs == vec![initial_mob.clone()];
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(observer_received_mobs);
    let host_id = client.local_id.expect("host should have an assigned ID");
    let replicated_host = observer
        .remote_players
        .get(&host_id)
        .expect("late joiner should receive existing players");
    assert_eq!(replicated_host.target_pos, host_position);
    assert_eq!(replicated_host.yaw, 0.3);

    client
        .send_packet(&ClientPacket::MobSnapshot {
            mobs: vec![initial_mob.clone()],
        })
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut observer_received_update = false;
    let mut authority_received_own_update = false;
    while Instant::now() < deadline && !observer_received_update {
        for event in client.update(0.05) {
            authority_received_own_update |= matches!(event, NetworkEvent::MobSnapshot { .. });
        }
        for event in observer.update(0.05) {
            if let NetworkEvent::MobSnapshot { mobs } = event {
                observer_received_update = mobs == vec![initial_mob.clone()];
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(observer_received_update);
    assert!(!authority_received_own_update);

    let changed_block = (6, 20, -7, BlockId::Dirt.to_u8());
    client.send_block_change(
        changed_block.0,
        changed_block.1,
        changed_block.2,
        changed_block.3,
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut observer_received_block = false;
    while Instant::now() < deadline && !observer_received_block {
        client.update(0.05);
        for event in observer.update(0.05) {
            if let NetworkEvent::BlockChanged { x, y, z, block } = event {
                observer_received_block = (x, y, z, block) == changed_block;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(observer_received_block);

    let mut late_joiner =
        NetworkClient::connect(&format!("127.0.0.1:{}", server.port), "LateJoiner").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut late_joiner_blocks = None;
    while Instant::now() < deadline && late_joiner_blocks.is_none() {
        client.update(0.05);
        observer.update(0.05);
        for event in late_joiner.update(0.05) {
            if let NetworkEvent::Connected {
                modified_blocks, ..
            } = event
            {
                late_joiner_blocks = Some(modified_blocks);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    let late_joiner_blocks = late_joiner_blocks.expect("late joiner should receive world changes");
    assert!(late_joiner_blocks.contains(&changed_block));

    late_joiner.disconnect();
    observer.disconnect();
    client.disconnect();
    drop(server);
}
