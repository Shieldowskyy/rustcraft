//! Dyspozytor komend czatu, komendy gracza (`/gamemode`, `/give`, `/tp`, `/heal`, `/kill`, `/clear`, `/fly`) oraz parsowanie nazw bloków.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::player::GameMode;

use super::state::{ChatState, CommandContext};

impl ChatState {
    pub fn dispatch(&mut self, text: &str, ctx: &mut CommandContext) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        if text.starts_with('/') {
            self.execute_command(text, ctx);
        } else {
            self.add_player_chat("Player", text);
        }
    }

    pub fn execute_command(&mut self, full_cmd: &str, ctx: &mut CommandContext) {
        crate::log_info!("Command", "Executing: {}", full_cmd);
        let trimmed = full_cmd.trim_start_matches('/');
        let mut parts = trimmed.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c.to_ascii_lowercase(),
            None => return,
        };

        if self.execute_world_or_mob_command(&cmd, &mut parts, ctx) {
            return;
        }

        match cmd.as_str() {
            "help" | "?" => {
                self.add_info("--- Showing help page (Minecraft Commands) ---");
                self.add_info("/gamemode <survival|creative> - Change game mode");
                self.add_info("/time set <day|noon|night|midnight|number> - Set time");
                self.add_info("/tps - Show multiplayer server tick rate and tick time");
                self.add_info("/give <item> [count] - Add items to inventory");
                self.add_info("/tp <x> <y> <z> or /tp <player> - Teleport");
                self.add_info("/spawn <pig|cow|chicken|zombie|skeleton|reaper> [count] - Spawn mob");
                self.add_info("/mobs - List all currently spawned mobs in the world");
                self.add_info("/heal - Restore health and hunger");
                self.add_info("/kill - Kill the player");
                self.add_info("/clear [chat] - Clear inventory or chat");
                self.add_info("/seed - Display world generation seed");
                self.add_info("/reloadchunks - Reload chunks and rebuild mesh cache");
                self.add_info("/fly - Toggle flight mode");
                self.add_info("/weather <clear|rain|thunder|query> [duration] - Control weather");
                self.add_info("/difficulty <peaceful|easy|normal|hard>");

                let mod_cmds = crate::engine::modding::get_all_mod_commands();
                if !mod_cmds.is_empty() {
                    self.add_info("--- Mod Commands ---");
                    for (c_name, c_desc) in mod_cmds {
                        if c_desc.is_empty() {
                            self.add_info(format!("/{}", c_name));
                        } else {
                            self.add_info(format!("/{} - {}", c_name, c_desc));
                        }
                    }
                }
            }
            "gamemode" | "gm" => {
                let mode_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                match mode_arg.as_str() {
                    "creative" | "c" | "1" => {
                        ctx.player.gamemode = GameMode::Creative;
                        self.add_success("Set own game mode to Creative Mode");
                        crate::log_debug!("Command", "Gamemode changed to Creative");
                    }
                    "survival" | "s" | "0" => {
                        ctx.player.gamemode = GameMode::Survival;
                        *ctx.is_flying = false;
                        self.add_success("Set own game mode to Survival Mode");
                        crate::log_debug!("Command", "Gamemode changed to Survival");
                    }
                    _ => self.add_error("Usage: /gamemode <survival|creative>"),
                }
            }
            "give" => {
                let Some(item_arg) = parts.next() else {
                    self.add_error("Usage: /give <item> [count]");
                    return;
                };
                let count: u8 = parts
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(1)
                    .clamp(1, 64);

                if let Some(block) = parse_block_id(item_arg) {
                    let leftover = ctx.player.inventory.add_item(block, count);
                    let given = count - leftover;
                    if given > 0 {
                        self.add_success(format!("Gave {} [{}] to Player", given, block.name()));
                        crate::log_debug!("Command", "Gave {} {:?} to Player", given, block);
                    }
                    if leftover > 0 {
                        self.add_error(format!(
                            "Inventory full! Could not give {} items.",
                            leftover
                        ));
                    }
                } else {
                    self.add_error(format!("Unknown item: '{}'", item_arg));
                }
            }
            "tp" | "teleport" => {
                let x_str = parts.next();
                let y_str = parts.next();
                let z_str = parts.next();

                if let (Some(xs), Some(ys), Some(zs)) = (x_str, y_str, z_str) {
                    let parse_coord = |s: &str, current: f32| -> Option<f32> {
                        if let Some(rel) = s.strip_prefix('~') {
                            if rel.is_empty() {
                                Some(current)
                            } else {
                                rel.parse::<f32>().ok().map(|d| current + d)
                            }
                        } else {
                            s.parse::<f32>().ok()
                        }
                    };

                    let px = parse_coord(xs, ctx.camera.position.x);
                    let py = parse_coord(ys, ctx.camera.position.y);
                    let pz = parse_coord(zs, ctx.camera.position.z);

                    if let (Some(x), Some(y), Some(z)) = (px, py, pz) {
                        ctx.camera.position = Vec3::new(x, y, z);
                        *ctx.horiz_velocity = Vec3::zero();
                        *ctx.fly_velocity = Vec3::zero();
                        self.add_success(format!("Teleported Player to {:.1}, {:.1}, {:.1}", x, y, z));
                        crate::log_debug!("Command", "Teleported Player to ({:.1}, {:.1}, {:.1})", x, y, z);
                    } else {
                        self.add_error("Invalid coordinates. Example: /tp 0 70 0 or /tp ~ ~10 ~");
                    }
                } else {
                    self.add_error("Usage: /tp <x> <y> <z> or /tp <player>");
                }
            }
            "heal" => {
                ctx.player.health = ctx.player.max_health;
                ctx.player.hunger = ctx.player.max_hunger;
                ctx.player.saturation = 20.0;
                ctx.player.air = ctx.player.max_air;
                self.add_success("Healed Player to full health and hunger");
                crate::log_debug!("Command", "Healed Player to full health and hunger");
            }
            "kill" => {
                ctx.player.take_damage(999.0);
                self.add_error("Ouch! That looks like it hurt.");
                crate::log_debug!("Command", "Killed Player via command");
            }
            "clear" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                if sub == "chat" {
                    self.messages.clear();
                } else {
                    ctx.player.inventory.hotbar = [None; 10];
                    ctx.player.inventory.main = [None; 27];
                    ctx.player.inventory.held_item = None;
                    self.add_success("Cleared the inventory of Player");
                    crate::log_debug!("Command", "Cleared player inventory");
                }
            }
            "fly" => {
                *ctx.is_flying = !*ctx.is_flying;
                if *ctx.is_flying {
                    self.add_success("Flight mode: ENABLED");
                } else {
                    self.add_info("Flight mode: DISABLED");
                }
                crate::log_debug!("Command", "Flight mode set to {}", *ctx.is_flying);
            }
            "difficulty" => {
                let d = parts.next().unwrap_or("normal");
                self.add_info(format!("Set difficulty to {}", d));
            }
            unknown => {
                if let Some(cmd_def) = crate::engine::modding::find_mod_command(&cmd) {
                    crate::engine::modding::execute_mod_command(&cmd_def, self, ctx);
                } else {
                    self.add_error(format!(
                        "Unknown command: '/{}'. Type /help for help.",
                        unknown
                    ));
                }
            }
        }
    }
}

pub fn parse_vanilla_block_id(s: &str) -> Option<BlockId> {
    let s = s.trim().to_ascii_lowercase().replace('-', "_");
    match s.as_str() {
        "rock" | "cobblestone" => return Some(BlockId::Stone),
        "log" | "oak_log" => return Some(BlockId::Wood),
        "wood_planks" => return Some(BlockId::Planks),
        "workbench" => return Some(BlockId::CraftingTable),
        "coal" => return Some(BlockId::CoalOre),
        "zloto" => return Some(BlockId::GoldOre),
        "ingot" => return Some(BlockId::IronIngot),
        "porkchop" => return Some(BlockId::RawPorkchop),
        "beef" => return Some(BlockId::RawBeef),
        "chicken" => return Some(BlockId::RawChicken),
        "sword" | "diamond_sword" => return Some(BlockId::IronSword),
        "pickaxe" | "diamond_pickaxe" => return Some(BlockId::IronPickaxe),
        "axe" => return Some(BlockId::IronAxe),
        "shovel" => return Some(BlockId::IronShovel),
        "pig_egg" => return Some(BlockId::PigSpawnEgg),
        "cow_egg" => return Some(BlockId::CowSpawnEgg),
        "chicken_egg" => return Some(BlockId::ChickenSpawnEgg),
        "redstone" => return Some(BlockId::RustStone),
        "redstone_ore" => return Some(BlockId::RustStoneOre),
        "ruststone_block" | "redstone_block" => return Some(BlockId::RustStoneBlock),
        "lampa" | "redstone_lamp" => return Some(BlockId::RustStoneLamp),
        "plytka" => return Some(BlockId::PressurePlate),
        "sheep_egg" | "owca_jajko" => return Some(BlockId::SheepSpawnEgg),
        "wool" | "welna" => return Some(BlockId::WhiteWool),
        "jasnofioletowa_welna" => return Some(BlockId::MagentaWool),
        "turkusowa_welna" => return Some(BlockId::CyanWool),
        "wodorosty" => return Some(BlockId::Kelp),
        "cod" | "dorsz" => return Some(BlockId::RawCod),
        "salmon" | "losos" => return Some(BlockId::RawSalmon),
        "dolphin_egg" => return Some(BlockId::DolphinSpawnEgg),
        "fish_egg" => return Some(BlockId::FishSpawnEgg),
        "villager_egg" | "jajko_osadnika" => return Some(BlockId::VillagerSpawnEgg),
        "nici" => return Some(BlockId::StringItem),
        "spider_egg" | "jajko_pajaka" => return Some(BlockId::SpiderSpawnEgg),
        _ => {}
    }
    for &b in &BlockId::VANILLA_ALL[1..] {
        let mut snake = String::with_capacity(24);
        for (i, ch) in b.variant_name().chars().enumerate() {
            if ch.is_ascii_uppercase() && i > 0 {
                snake.push('_');
            }
            snake.push(ch.to_ascii_lowercase());
        }
        if s == snake
            || s == b.name().to_ascii_lowercase().replace(' ', "_")
            || s == b.name_en().to_ascii_lowercase().replace(' ', "_")
        {
            return Some(b);
        }
    }
    if let Ok(id) = s.parse::<u8>() {
        if id > 0 {
            return Some(BlockId::from_u8(id));
        }
    }
    None
}

pub fn parse_block_id(s: &str) -> Option<BlockId> {
    if let Some(b) = parse_vanilla_block_id(s) {
        return Some(b);
    }
    let s = s.trim().to_ascii_lowercase().replace('-', "_");
    crate::engine::modding::find_custom_block_by_name(&s)
}
