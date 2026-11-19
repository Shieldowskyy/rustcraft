//! Multiplayer world-snapshot serialization and per-frame network event synchronization.
//! Mutates world blocks, entities, player hurt state, chat log, and mod sync from incoming network events.
//! Sends throttled position updates, block diffs, mob snapshots, and player persistence data.

use crate::core::math::{IVec3, Vec3};
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::item::ItemEntity;
use crate::world::block::BlockId;

use super::super::{AppOutput, RumbleRequest, RustCraftApp};

impl RustCraftApp {
    pub(super) fn network_world_snapshot(&self) -> crate::net::protocol::WorldSnapshot {
        use crate::net::protocol::*;

        let network_item = |item: Option<ItemStack>| {
            item.map(|item| PlayerItemStack {
                block: item.block.to_u8(),
                count: item.count,
                durability: item.durability,
            })
        };
        let weather = &self.world.weather;
        let lightning = weather
            .active_lightning
            .iter()
            .map(|bolt| LightningEntry {
                x: bolt.pos.x,
                y: bolt.pos.y,
                z: bolt.pos.z,
                timer: bolt.timer,
                max_duration: bolt.max_duration,
                seed: bolt.seed,
            })
            .collect();

        WorldSnapshot {
            items: self
                .items
                .iter()
                .map(|item| WorldItemEntry {
                    x: item.position.x,
                    y: item.position.y,
                    z: item.position.z,
                    velocity_x: item.velocity.x,
                    velocity_y: item.velocity.y,
                    velocity_z: item.velocity.z,
                    block: item.block.to_u8(),
                    count: item.count,
                    pickup_delay: item.pickup_delay,
                    age: item.age,
                })
                .collect(),
            chests: self
                .world
                .chests
                .iter()
                .map(|(pos, chest)| ChestSnapshot {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    items: chest.items.map(network_item),
                })
                .collect(),
            furnaces: self
                .world
                .furnaces
                .iter()
                .map(|(pos, furnace)| FurnaceSnapshot {
                    x: pos.x,
                    y: pos.y,
                    z: pos.z,
                    input: network_item(furnace.input),
                    fuel: network_item(furnace.fuel),
                    output: network_item(furnace.output),
                    burn_time_remaining: furnace.burn_time_remaining,
                    burn_time_total: furnace.burn_time_total,
                    cook_progress: furnace.cook_progress,
                })
                .collect(),
            primed_tnts: self
                .world
                .primed_tnts
                .iter()
                .map(|tnt| PrimedTntEntry {
                    x: tnt.pos.x,
                    y: tnt.pos.y,
                    z: tnt.pos.z,
                    velocity_x: tnt.velocity.x,
                    velocity_y: tnt.velocity.y,
                    velocity_z: tnt.velocity.z,
                    fuse: tnt.fuse,
                })
                .collect(),
            arrows: self
                .arrows
                .iter()
                .map(|arrow| ArrowEntry {
                    x: arrow.position.x,
                    y: arrow.position.y,
                    z: arrow.position.z,
                    velocity_x: arrow.velocity.x,
                    velocity_y: arrow.velocity.y,
                    velocity_z: arrow.velocity.z,
                    shooter_is_player: arrow.shooter_is_player,
                    in_ground: arrow.in_ground,
                    lifetime: arrow.lifetime,
                    damage: arrow.damage,
                })
                .collect(),
            weather: WeatherSnapshot {
                current_type: match weather.current_type {
                    crate::world::WeatherType::Clear => 0,
                    crate::world::WeatherType::Rain => 1,
                    crate::world::WeatherType::Thunder => 2,
                },
                timer: weather.timer,
                rain_strength: weather.rain_strength,
                thunder_strength: weather.thunder_strength,
                lightning_timer: weather.lightning_timer,
                lightning_flash: weather.lightning_flash,
                rng_state: weather.network_rng_state(),
                active_lightning: lightning,
            },
        }
    }

    pub(super) fn apply_network_world_snapshot(&mut self, snapshot: crate::net::protocol::WorldSnapshot) {
        use crate::net::protocol::PlayerItemStack;

        let world_item = |item: Option<PlayerItemStack>| {
            item.map(|item| ItemStack {
                block: BlockId::from_u8(item.block),
                count: item.count,
                durability: item.durability,
            })
        };
        self.items = snapshot
            .items
            .into_iter()
            .map(|item| {
                let mut entity = ItemEntity::new(
                    Vec3::new(item.x, item.y, item.z),
                    Vec3::new(item.velocity_x, item.velocity_y, item.velocity_z),
                    BlockId::from_u8(item.block),
                    item.count,
                );
                entity.pickup_delay = item.pickup_delay;
                entity.age = item.age;
                entity
            })
            .collect();
        self.world.chests = snapshot
            .chests
            .into_iter()
            .map(|chest| {
                (
                    IVec3::new(chest.x, chest.y, chest.z),
                    crate::world::ChestData {
                        items: chest.items.map(world_item),
                    },
                )
            })
            .collect();
        self.world.furnaces = snapshot
            .furnaces
            .into_iter()
            .map(|furnace| {
                (
                    IVec3::new(furnace.x, furnace.y, furnace.z),
                    crate::world::FurnaceData {
                        input: world_item(furnace.input),
                        fuel: world_item(furnace.fuel),
                        output: world_item(furnace.output),
                        burn_time_remaining: furnace.burn_time_remaining,
                        burn_time_total: furnace.burn_time_total,
                        cook_progress: furnace.cook_progress,
                    },
                )
            })
            .collect();
        self.world.primed_tnts = snapshot
            .primed_tnts
            .into_iter()
            .map(|tnt| crate::world::PrimedTnt {
                pos: Vec3::new(tnt.x, tnt.y, tnt.z),
                velocity: Vec3::new(tnt.velocity_x, tnt.velocity_y, tnt.velocity_z),
                fuse: tnt.fuse,
            })
            .collect();
        self.arrows = snapshot
            .arrows
            .into_iter()
            .map(|arrow| {
                let mut entity = crate::gameplay::ArrowEntity::new(
                    Vec3::new(arrow.x, arrow.y, arrow.z),
                    Vec3::new(arrow.velocity_x, arrow.velocity_y, arrow.velocity_z),
                    arrow.shooter_is_player,
                    arrow.damage,
                );
                entity.in_ground = arrow.in_ground;
                entity.lifetime = arrow.lifetime;
                entity
            })
            .collect();
        let weather = snapshot.weather;
        self.world.weather.set_network_state(
            match weather.current_type {
                0 => crate::world::WeatherType::Clear,
                1 => crate::world::WeatherType::Rain,
                _ => crate::world::WeatherType::Thunder,
            },
            weather.timer,
            weather.rain_strength,
            weather.thunder_strength,
            weather.lightning_timer,
            weather.lightning_flash,
            weather.rng_state,
            weather
                .active_lightning
                .into_iter()
                .map(|bolt| {
                    let mut strike = crate::world::LightningStrike::new(
                        Vec3::new(bolt.x, bolt.y, bolt.z),
                        bolt.max_duration,
                        bolt.seed,
                    );
                    strike.timer = bolt.timer;
                    strike
                })
                .collect(),
        );
    }

    pub(super) fn sync_playing_network(&mut self, dt: f32, out: &mut AppOutput) {
        let player_data = self.net_client.is_some().then(|| self.multiplayer_player_data());
        let network_block_changes = self.world.take_network_changes().collect::<Vec<_>>();
        let mut received_player_data = None;
        let mut received_world_snapshot = None;
        let outgoing_world_snapshot = self
            .net_client
            .as_ref()
            .filter(|client| client.can_send_world_snapshot())
            .map(|_| self.network_world_snapshot());

        if let Some(client) = &mut self.net_client {
            if client.last_move_sent.elapsed() >= std::time::Duration::from_millis(50) {
                client.send_position(
                    self.camera.position,
                    self.camera.yaw,
                    self.camera.pitch,
                    self.crouch_toggled,
                    self.sprint_toggled,
                );
                client.last_move_sent = std::time::Instant::now();
            }
            let mut events = std::mem::take(&mut self.pending_network_events);
            events.extend(client.update(dt));
            let mut should_disconnect = None;
            let mut pending_mods = Vec::new();
            let mut spawned_authoritative_mob = false;
            for ev in events {
                match ev {
                    crate::net::NetworkEvent::BlockChanged { x, y, z, block } => {
                        self.world
                            .set_network_block(IVec3::new(x, y, z), BlockId::from_u8(block));
                    }
                    crate::net::NetworkEvent::ChatMessage {
                        sender,
                        text,
                        is_system,
                    } => {
                        if is_system {
                            self.chat.add_info(format!("[{}] {}", sender, text));
                        } else {
                            self.chat.add_message(
                                format!("<{}> {}", sender, text),
                                crate::ui::pack_color(1.0, 1.0, 1.0),
                            );
                        }
                    }
                    crate::net::NetworkEvent::TimeSync { time_of_day } => {
                        self.world.time_of_day = time_of_day;
                    }
                    crate::net::NetworkEvent::Teleport { x, y, z } => {
                        self.camera.position = Vec3::new(x, y, z);
                    }
                    crate::net::NetworkEvent::PlayerHurt {
                        victim_id,
                        attacker_id,
                        damage,
                        knockback_x,
                        knockback_z,
                    } => {
                        let is_local = client.local_id == Some(victim_id);
                        if is_local {
                            if self.player.take_damage(damage) {
                                self.damage_tilt_timer = 0.4;
                                self.damage_tilt_dir = if knockback_x > 0.0 { -1.0 } else { 1.0 };
                                self.horiz_velocity.x += knockback_x * 8.0;
                                self.horiz_velocity.z += knockback_z * 8.0;
                                self.vertical_velocity = 3.5;
                                self.particles.spawn_mob_hit(self.camera.position);
                                if self.settings.vibration {
                                    self.pending_rumble = Some(RumbleRequest {
                                        strong: 0.85,
                                        weak: 0.6,
                                        duration_ms: 220,
                                    });
                                }
                                crate::log_info!(
                                    "PVP",
                                    "Player took {:.1} damage from player ID {}",
                                    damage,
                                    attacker_id
                                );
                            }
                        } else if let Some(rp) = client.remote_players.get(&victim_id) {
                            self.particles.spawn_mob_hit(rp.pos);
                        }
                    }
                    crate::net::NetworkEvent::MobSnapshot { mobs } => {
                        if let Err(error) = self.mob_manager.apply_network_snapshot(&mobs) {
                            crate::log_warn!("NETWORK", "Rejected mob snapshot: {}", error);
                        }
                    }
                    crate::net::NetworkEvent::MobAttack {
                        attacker_id,
                        mob_id,
                        damage,
                        direction,
                    } => {
                        if client.local_id == client.mob_authority_id
                            && client.local_id != Some(attacker_id)
                        {
                            if let Some(mob) = self
                                .mob_manager
                                .mobs
                                .iter_mut()
                                .find(|mob| mob.id == mob_id)
                            {
                                let hit_pos = mob.center_pos();
                                mob.take_damage(damage, direction);
                                self.particles.spawn_mob_hit(hit_pos);
                            }
                        }
                    }
                    crate::net::NetworkEvent::MobSpawn { kind, pos } => {
                        if client.local_id == client.mob_authority_id {
                            if let Some(kind) =
                                crate::gameplay::mob::MobKind::ALL.get(kind as usize)
                            {
                                self.mob_manager.spawn_mob(*kind, pos);
                                spawned_authoritative_mob = true;
                            } else {
                                crate::log_warn!(
                                    "NETWORK",
                                    "Rejected mob spawn with unknown kind {}",
                                    kind
                                );
                            }
                        }
                    }
                    crate::net::NetworkEvent::WorldSnapshot { snapshot } => {
                        received_world_snapshot = Some(snapshot);
                    }
                    crate::net::NetworkEvent::PlayerDataLoaded { data } => {
                        received_player_data = Some(data);
                    }
                    crate::net::NetworkEvent::ModsSynchronized { mods } => {
                        pending_mods.extend(mods);
                    }
                    crate::net::NetworkEvent::Disconnected { reason } => {
                        should_disconnect = Some(reason);
                    }
                    _ => {}
                }
            }
            if spawned_authoritative_mob {
                client.send_mob_snapshot_now(&self.mob_manager);
            } else {
                client.send_mob_snapshot(&self.mob_manager);
            }
            if let Some(snapshot) = outgoing_world_snapshot {
                client.send_world_snapshot(snapshot);
            }
            for (pos, block) in network_block_changes {
                client.send_block_change(pos.x, pos.y, pos.z, block);
            }
            if let Some(data) = player_data.clone() {
                client.send_player_data(data, false);
            }
            if !pending_mods.is_empty() {
                self.sync_server_mods(pending_mods);
            }
            if let Some(reason) = should_disconnect {
                self.handle_multiplayer_disconnect(reason);
                out.request_release_mouse = true;
            }
        }
        if let Some(snapshot) = received_world_snapshot {
            self.apply_network_world_snapshot(snapshot);
        }
        if self.is_multiplayer {
            if let Some(data) = received_player_data {
                match data {
                    Some(data) => self.apply_multiplayer_player_data(data),
                    None => {
                        if let (Some(client), Some(data)) = (&mut self.net_client, player_data) {
                            client.send_player_data(data, true);
                        }
                    }
                }
            }
        }
    }
}
