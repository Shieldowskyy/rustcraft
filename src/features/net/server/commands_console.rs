//! Obsługa poleceń konsoli terminalowej serwera dedykowanego (`help`, `list`, `say`, `tell`, `kick`, `op`, `time`, `tps`, `tp`, `stop`).

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::features::net::protocol::ServerPacket;

use super::config::{kick_client, set_op, ConnectedClient};

#[allow(clippy::too_many_arguments)]
pub(super) fn handle_console_command(
    cmd_line: &str,
    clients: &mut HashMap<u32, ConnectedClient>,
    modified_blocks: &mut HashMap<(i32, i32, i32), u8>,
    time_of_day: &mut f32,
    ops: &mut HashSet<String>,
    persist_ops: bool,
    seed: u32,
    tps: f32,
    mspt: f32,
    uptime: Duration,
    running: &Arc<AtomicBool>,
) {
    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0].to_lowercase().as_str() {
        "help" => {
            crate::server_info!("------------------- Server Commands -------------------");
            crate::server_info!("  help                     - Show this list of console commands");
            crate::server_info!(
                "  list                     - List all connected players and their ping"
            );
            crate::server_info!(
                "  say <message>            - Broadcast chat message to all players"
            );
            crate::server_info!(
                "  tell <player> <message>  - Whisper a private message to a player"
            );
            crate::server_info!(
                "  kick <player> [reason]   - Disconnect a player from the server"
            );
            crate::server_info!(
                "  op <player> | deop <player> - Grant or revoke operator status"
            );
            crate::server_info!(
                "  time [day|night|set <v>] - Display or change world time of day"
            );
            crate::server_info!(
                "  tps                      - Display server TPS and tick time"
            );
            crate::server_info!(
                "  seed                     - Display the world generation seed"
            );
            crate::server_info!(
                "  status                   - Display server TPS, uptime, and stats"
            );
            crate::server_info!("  tp <p1> <p2> / tp <p> x y z - Teleport a player");
            crate::server_info!("  stop | exit              - Gracefully shut down the server");
            crate::server_info!("-------------------------------------------------------");
        }
        "list" => {
            crate::server_info!("[SERVER] Connected Players ({}/16):", clients.len());
            if clients.is_empty() {
                crate::server_info!("  (No players online)");
            } else {
                for client in clients.values() {
                    crate::server_info!(
                        "  - {} (ID: {}, IP: {}, Pos: ({:.1}, {:.1}, {:.1}), Ping: {}ms)",
                        client.nickname,
                        client.id,
                        client.addr,
                        client.x,
                        client.y,
                        client.z,
                        client.ping_ms
                    );
                }
            }
        }
        "say" => {
            if parts.len() < 2 {
                crate::server_info!("[SERVER] Usage: say <message>");
            } else {
                let msg = parts[1..].join(" ");
                crate::server_info!("[SERVER SAY] {}", msg);
                let packet = ServerPacket::ChatMessage {
                    sender: "[Server]".to_string(),
                    text: msg,
                    is_system: true,
                };
                for client in clients.values_mut() {
                    let _ = client.send_packet(&packet);
                }
            }
        }
        "tell" | "msg" | "w" => {
            if parts.len() < 3 {
                crate::server_info!("[SERVER] Usage: tell <player> <message>");
            } else {
                let target_name = parts[1].to_lowercase();
                let msg = parts[2..].join(" ");
                let mut found = false;
                for client in clients.values_mut() {
                    if client.nickname.to_lowercase() == target_name {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: "[Server Whisper]".to_string(),
                            text: msg.clone(),
                            is_system: true,
                        });
                        crate::server_info!(
                            "[SERVER] Whispered to {}: {}",
                            client.nickname,
                            msg
                        );
                        found = true;
                        break;
                    }
                }
                if !found {
                    crate::server_info!("[SERVER] Player '{}' not found", parts[1]);
                }
            }
        }
        "kick" => {
            if parts.len() < 2 {
                crate::server_info!("[SERVER] Usage: kick <player> [reason]");
            } else {
                let target_name = parts[1].to_lowercase();
                let reason = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    "Kicked by server operator".to_string()
                };
                let kick_id = clients
                    .values()
                    .find(|c| c.nickname.to_lowercase() == target_name)
                    .map(|c| c.id);
                if let Some(id) = kick_id {
                    kick_client(clients, id, &reason);
                } else {
                    crate::server_info!("[SERVER] Player '{}' not found", parts[1]);
                }
            }
        }
        "op" | "deop" => {
            let grant = parts[0].eq_ignore_ascii_case("op");
            if let Some(name) = parts.get(1) {
                if set_op(ops, name, grant, persist_ops) {
                    crate::server_info!(
                        "[SERVER] {} operator status for {}",
                        if grant { "Granted" } else { "Revoked" },
                        name
                    );
                    for client in clients
                        .values_mut()
                        .filter(|c| c.nickname.eq_ignore_ascii_case(name))
                    {
                        let _ = client.send_packet(&ServerPacket::ChatMessage {
                            sender: "Server".to_string(),
                            text: if grant {
                                "You are now an operator"
                            } else {
                                "You are no longer an operator"
                            }
                            .to_string(),
                            is_system: true,
                        });
                    }
                } else {
                    crate::server_info!("[SERVER] Nothing changed for '{}'", name);
                }
            } else {
                crate::server_info!("[SERVER] Usage: op <player> | deop <player>");
            }
        }
        "seed" => {
            crate::server_info!("[SERVER] World Seed: {}", seed);
        }
        "time" => {
            if parts.len() == 1 {
                crate::server_info!("[SERVER] Current time of day: {:.1} / 480.0", *time_of_day);
            } else {
                match parts[1].to_lowercase().as_str() {
                    "day" => {
                        *time_of_day = 120.0;
                        crate::server_info!("[SERVER] Time set to Day (120.0)");
                    }
                    "night" => {
                        *time_of_day = 360.0;
                        crate::server_info!("[SERVER] Time set to Night (360.0)");
                    }
                    "set" if parts.len() >= 3 => {
                        if let Ok(val) = parts[2].parse::<f32>() {
                            *time_of_day = val.rem_euclid(480.0);
                            crate::server_info!("[SERVER] Time set to {:.1}", *time_of_day);
                        } else {
                            crate::server_info!("[SERVER] Invalid time value: {}", parts[2]);
                        }
                    }
                    _ => {
                        crate::server_info!("[SERVER] Usage: time [day|night|set <number>]");
                        return;
                    }
                }
                let sync = ServerPacket::TimeSync {
                    time_of_day: *time_of_day,
                };
                for client in clients.values_mut() {
                    let _ = client.send_packet(&sync);
                }
            }
        }
        "status" => {
            let secs = uptime.as_secs();
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let s = secs % 60;
            crate::server_info!("------------------- Server Status -------------------");
            crate::server_info!("  TPS:             {:.1} / 20.0", tps);
            crate::server_info!("  Uptime:          {:02}:{:02}:{:02}", hours, mins, s);
            crate::server_info!("  Online Players:  {}/16", clients.len());
            crate::server_info!("  Modified Blocks: {}", modified_blocks.len());
            crate::server_info!("  Time of Day:     {:.1}s", *time_of_day);
            crate::server_info!("-----------------------------------------------------");
        }
        "tps" => {
            crate::server_info!("[SERVER] TPS: {:.1} / 20.0 | MSPT: {:.2} ms", tps, mspt);
        }
        "tp" => {
            if parts.len() == 5 {
                let p_name = parts[1].to_lowercase();
                if let (Ok(x), Ok(y), Ok(z)) = (
                    parts[2].parse::<f32>(),
                    parts[3].parse::<f32>(),
                    parts[4].parse::<f32>(),
                ) {
                    let mut found = false;
                    for client in clients.values_mut() {
                        if client.nickname.to_lowercase() == p_name {
                            client.x = x;
                            client.y = y;
                            client.z = z;
                            let _ = client.send_packet(&ServerPacket::Teleport { x, y, z });
                            crate::server_info!(
                                "[SERVER] Teleported {} to ({:.1}, {:.1}, {:.1})",
                                client.nickname,
                                x,
                                y,
                                z
                            );
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        crate::server_info!("[SERVER] Player '{}' not found", parts[1]);
                    }
                } else {
                    crate::server_info!(
                        "[SERVER] Coordinates must be numbers: tp <player> <x> <y> <z>"
                    );
                }
            } else if parts.len() == 3 {
                let p1_name = parts[1].to_lowercase();
                let p2_name = parts[2].to_lowercase();
                let target_pos = clients
                    .values()
                    .find(|c| c.nickname.to_lowercase() == p2_name)
                    .map(|c| (c.x, c.y, c.z));
                if let Some((x, y, z)) = target_pos {
                    if let Some(c) = clients
                        .values_mut()
                        .find(|c| c.nickname.to_lowercase() == p1_name)
                    {
                        c.x = x;
                        c.y = y;
                        c.z = z;
                        let _ = c.send_packet(&ServerPacket::Teleport { x, y, z });
                        crate::server_info!("[SERVER] Teleported {} to {}", c.nickname, parts[2]);
                    } else {
                        crate::server_info!("[SERVER] Player '{}' not found", parts[1]);
                    }
                } else {
                    crate::server_info!("[SERVER] Target player '{}' not found", parts[2]);
                }
            } else {
                crate::server_info!(
                    "[SERVER] Usage: tp <player> <x> <y> <z>  OR  tp <player1> <player2>"
                );
            }
        }
        "stop" | "exit" | "quit" => {
            crate::server_info!("[SERVER] Initiating server shutdown...");
            running.store(false, Ordering::Relaxed);
        }
        unknown => {
            crate::server_info!(
                "[SERVER] Unknown command: '{}'. Type 'help' for commands.",
                unknown
            );
        }
    }
}
