//! Obsługa komend czatu wewnątrz gry wieloosobowej (`/op`, `/deop`, `/kick`, `/tp`, `/time`, `/tps`, `/tell`, `/spawn`).

use std::collections::{HashMap, HashSet};

use crate::features::net::protocol::ServerPacket;

use super::config::{kick_client, set_op, ConnectedClient, ServerConfig};

pub(super) fn parse_time_value(value: &str) -> Option<f32> {
    match value.to_ascii_lowercase().as_str() {
        "day" | "1000" => Some(120.0),
        "noon" | "6000" => Some(240.0),
        "night" | "13000" => Some(360.0),
        "midnight" | "18000" => Some(0.0),
        ticks => {
            let ticks = ticks.parse::<f32>().ok()?;
            ticks
                .is_finite()
                .then(|| ticks.rem_euclid(24_000.0) / 24_000.0 * 480.0)
        }
    }
}

fn send_system(clients: &mut HashMap<u32, ConnectedClient>, id: u32, text: &str) {
    if let Some(client) = clients.get_mut(&id) {
        let _ = client.send_packet(&ServerPacket::ChatMessage {
            sender: "Server".to_string(),
            text: text.to_string(),
            is_system: true,
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_in_game_command(
    cmd_line: &str,
    sender_id: u32,
    embedded_host_id: Option<u32>,
    ops: &mut HashSet<String>,
    persist_ops: bool,
    clients: &mut HashMap<u32, ConnectedClient>,
    broadcast_packets: &mut Vec<ServerPacket>,
    time_of_day: &mut f32,
    config: &ServerConfig,
    tps: f32,
    mspt: f32,
) {
    let sender_nick = clients
        .get(&sender_id)
        .map(|c| c.nickname.clone())
        .unwrap_or_default();
    let trimmed = cmd_line.trim_start_matches('/');
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let is_op = embedded_host_id == Some(sender_id) || ops.contains(&sender_nick.to_lowercase());
    let command = parts[0].to_lowercase();
    let needs_op = match command.as_str() {
        "kick" | "tp" | "teleport" | "op" | "deop" => true,
        "time" => !parts
            .get(1)
            .is_some_and(|arg| arg.eq_ignore_ascii_case("query")),
        _ => false,
    };
    if needs_op && !is_op {
        send_system(
            clients,
            sender_id,
            "You do not have permission to use this command.",
        );
        return;
    }

    match command.as_str() {
        "op" | "deop" => {
            let grant = command == "op";
            let Some(target) = parts.get(1) else {
                send_system(clients, sender_id, &format!("Usage: /{} <player>", command));
                return;
            };
            let online_name = clients
                .values()
                .find(|c| c.nickname.eq_ignore_ascii_case(target))
                .map(|c| (c.id, c.nickname.clone()));
            if !grant
                && embedded_host_id.is_some_and(|id| {
                    clients
                        .get(&id)
                        .is_some_and(|c| c.nickname.eq_ignore_ascii_case(target))
                })
            {
                send_system(clients, sender_id, "The host cannot be de-opped.");
                return;
            }
            let name = online_name
                .as_ref()
                .map(|(_, n)| n.clone())
                .unwrap_or_else(|| target.to_string());
            if set_op(ops, &name, grant, persist_ops) {
                send_system(
                    clients,
                    sender_id,
                    &format!(
                        "{} {} operator status",
                        if grant { "Made" } else { "Removed" },
                        name
                    ),
                );
                if let Some((id, _)) = online_name {
                    send_system(
                        clients,
                        id,
                        if grant {
                            "You are now an operator"
                        } else {
                            "You are no longer an operator"
                        },
                    );
                }
            } else {
                send_system(
                    clients,
                    sender_id,
                    if grant {
                        "Nothing changed. That player is already an operator"
                    } else {
                        "Nothing changed. That player is not an operator"
                    },
                );
            }
        }
        "help" => {
            send_system(
                clients,
                sender_id,
                "Available commands: /help, /list, /time set <day|noon|night|midnight|number>, /tps, /seed, /spawn, /tp <player>, /tell <player> <msg>, /kick <player> [reason] (op), /op <player> (op), /deop <player> (op)",
            );
        }
        "kick" => {
            if parts.len() < 2 {
                send_system(clients, sender_id, "Usage: /kick <player> [reason]");
                return;
            }

            let target = clients
                .values()
                .find(|client| client.nickname.eq_ignore_ascii_case(parts[1]))
                .map(|client| client.id);
            let reason = if parts.len() > 2 {
                parts[2..].join(" ")
            } else {
                format!("Kicked by host ({sender_nick})")
            };
            if let Some(target_id) = target {
                if let Some(target_name) = kick_client(clients, target_id, &reason) {
                    send_system(clients, sender_id, &format!("Kicked {target_name}: {reason}"));
                }
            } else {
                send_system(
                    clients,
                    sender_id,
                    &format!("Player '{}' not found", parts[1]),
                );
            }
        }
        "list" => {
            let names: Vec<String> = clients
                .values()
                .map(|c| format!("{} ({}ms)", c.nickname, c.ping_ms))
                .collect();
            let msg = format!(
                "Online ({}/{}): {}",
                clients.len(),
                config.max_players,
                names.join(", ")
            );
            send_system(clients, sender_id, &msg);
        }
        "seed" => {
            send_system(
                clients,
                sender_id,
                &format!("World Seed: {}", config.level_seed),
            );
        }
        "spawn" => {
            if let Some(client) = clients.get_mut(&sender_id) {
                client.x = config.spawn_pos.0;
                client.y = config.spawn_pos.1;
                client.z = config.spawn_pos.2;
                let _ = client.send_packet(&ServerPacket::Teleport {
                    x: config.spawn_pos.0,
                    y: config.spawn_pos.1,
                    z: config.spawn_pos.2,
                });
                let _ = client.send_packet(&ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: "Teleported to spawn point".to_string(),
                    is_system: true,
                });
            }
        }
        "tp" | "teleport" => {
            if parts.len() != 2 {
                send_system(clients, sender_id, "Usage: /tp <player>");
                return;
            }

            let target = clients
                .values()
                .find(|client| client.nickname.eq_ignore_ascii_case(parts[1]))
                .map(|client| (client.x, client.y, client.z, client.nickname.clone()));
            if let Some((x, y, z, target_name)) = target {
                if let Some(client) = clients.get_mut(&sender_id) {
                    client.x = x;
                    client.y = y;
                    client.z = z;
                    let _ = client.send_packet(&ServerPacket::Teleport { x, y, z });
                    let _ = client.send_packet(&ServerPacket::ChatMessage {
                        sender: "Server".to_string(),
                        text: format!("Teleported to {}", target_name),
                        is_system: true,
                    });
                }
            } else {
                send_system(
                    clients,
                    sender_id,
                    &format!("Player '{}' not found", parts[1]),
                );
            }
        }
        "time" => {
            let args = &parts[1..];
            if args
                .first()
                .is_some_and(|arg| arg.eq_ignore_ascii_case("query"))
            {
                let ticks = (*time_of_day / 480.0 * 24_000.0).round() as u32;
                send_system(
                    clients,
                    sender_id,
                    &format!("The time is {} ticks", ticks),
                );
                return;
            }

            let value = match args {
                [value] if !value.eq_ignore_ascii_case("set") => Some(*value),
                [set, value] if set.eq_ignore_ascii_case("set") => Some(*value),
                _ => None,
            };
            if let Some(value) = value.and_then(parse_time_value) {
                *time_of_day = value;
                broadcast_packets.push(ServerPacket::TimeSync { time_of_day: value });
                broadcast_packets.push(ServerPacket::ChatMessage {
                    sender: "Server".to_string(),
                    text: format!(
                        "{} set the time to {:.0} ticks",
                        sender_nick,
                        value / 480.0 * 24_000.0
                    ),
                    is_system: true,
                });
            } else {
                send_system(
                    clients,
                    sender_id,
                    "Usage: /time set <day|noon|night|midnight|number>",
                );
            }
        }
        "tps" => {
            send_system(
                clients,
                sender_id,
                &format!("Server TPS: {:.1} / 20.0 | MSPT: {:.2} ms", tps, mspt),
            );
        }
        "tell" | "w" | "msg" => {
            if parts.len() >= 3 {
                let target_name = parts[1].to_lowercase();
                let whisper = parts[2..].join(" ");
                let mut found = false;
                for (other_id, other) in clients.iter_mut() {
                    if *other_id != sender_id && other.nickname.to_lowercase() == target_name {
                        let _ = other.send_packet(&ServerPacket::ChatMessage {
                            sender: format!("{} -> you", sender_nick),
                            text: whisper.clone(),
                            is_system: false,
                        });
                        found = true;
                        break;
                    }
                }

                if let Some(client) = clients.get_mut(&sender_id) {
                    if found {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: format!("you -> {}", parts[1]),
                            text: whisper,
                            is_system: false,
                        });
                    } else {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: "Server".to_string(),
                            text: format!("Player '{}' not found", parts[1]),
                            is_system: true,
                        });
                    }
                }
            } else {
                send_system(clients, sender_id, "Usage: /tell <player> <message>");
            }
        }
        unknown => {
            send_system(
                clients,
                sender_id,
                &format!(
                    "Unknown command: /{}. Type /help for a list of commands.",
                    unknown
                ),
            );
        }
    }
}
