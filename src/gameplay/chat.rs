//! Minecraft-style in-game chat and command console.
//!
//! Opened via `T` (chat mode) or `/` (command mode with pre-filled slash).
//! Supports command history, standard commands (`/gamemode`, `/time`, `/give`, `/tp`, etc.),
//! and visual text fade-out.

use crate::core::math::Vec3;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::{MobKind, MobManager};
use crate::gameplay::particles::ParticleSystem;
use crate::gameplay::player::{GameMode, Player};
use crate::graphics::camera::Camera;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::block::BlockId;
use crate::world::World;

/// Single chat line in history.
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub text: String,
    pub color: u32,
    pub age: f32,
}

pub enum ChatAction {
    Execute(String),
    Close,
}

pub struct CommandContext<'a> {
    pub player: &'a mut Player,
    pub camera: &'a mut Camera,
    pub world: &'a mut World,
    pub mob_manager: &'a mut MobManager,
    pub items: &'a mut Vec<ItemEntity>,
    pub particles: &'a mut ParticleSystem,
    pub is_flying: &'a mut bool,
    pub fly_velocity: &'a mut Vec3,
    pub horiz_velocity: &'a mut Vec3,
    pub reload_chunks_requested: &'a mut bool,
}

pub struct ChatState {
    pub is_open: bool,
    pub input_text: String,
    pub cursor_pos: usize,
    pub messages: Vec<ChatMessage>,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
    pub cursor_blink: f32,
    pub saved_draft: String,

    // Key edge trackers
    was_backspace: bool,
    was_delete: bool,
    was_left: bool,
    was_right: bool,
    was_up: bool,
    was_down: bool,
    was_enter: bool,
    was_escape: bool,
    pub ignore_slash_until_release: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            input_text: String::new(),
            cursor_pos: 0,
            messages: Vec::new(),
            history: Vec::new(),
            history_idx: None,
            cursor_blink: 0.0,
            saved_draft: String::new(),

            was_backspace: false,
            was_delete: false,
            was_left: false,
            was_right: false,
            was_up: false,
            was_down: false,
            was_enter: false,
            was_escape: false,
            ignore_slash_until_release: false,
        }
    }

    pub fn open(&mut self, initial_text: &str) {
        self.is_open = true;
        self.input_text = initial_text.to_string();
        self.cursor_pos = self.input_text.len();
        self.history_idx = None;
        self.cursor_blink = 0.0;
        self.saved_draft.clear();
        self.ignore_slash_until_release = initial_text.starts_with('/');
        crate::log_debug!("Chat", "Chat opened with prefix: '{}'", initial_text);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.input_text.clear();
        self.cursor_pos = 0;
        self.history_idx = None;
        self.ignore_slash_until_release = false;
        crate::log_debug!("Chat", "Chat closed");
    }

    pub fn add_message(&mut self, text: impl Into<String>, color: u32) {
        let text = text.into();
        crate::log_info!("Chat", "{}", text);
        self.messages.push(ChatMessage {
            text,
            color,
            age: 0.0,
        });
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn add_info(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(1.0, 1.0, 0.4)); // Minecraft yellow/gold
    }

    pub fn add_success(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(0.4, 1.0, 0.4)); // Minecraft green
    }

    pub fn add_error(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(1.0, 0.35, 0.35)); // Minecraft red
    }

    pub fn add_player_chat(&mut self, player_name: &str, msg: &str) {
        let line = format!("<{}> {}", player_name, msg);
        self.add_message(line, pack_color(1.0, 1.0, 1.0));
    }

    pub fn update(&mut self, dt: f32) {
        self.cursor_blink += dt;
        for msg in &mut self.messages {
            msg.age += dt;
        }
    }

    /// Handles keyboard input when chat panel is open.
    pub fn handle_input(&mut self, input: &crate::app::InputState) -> Option<ChatAction> {
        if !self.is_open {
            return None;
        }

        // 1. Enter key -> submit message / command
        let enter_pressed = input.key_enter && !self.was_enter;
        self.was_enter = input.key_enter;
        if enter_pressed {
            let submitted = self.input_text.trim().to_string();
            if !submitted.is_empty() {
                self.history.push(submitted.clone());
                if self.history.len() > 50 {
                    self.history.remove(0);
                }
                self.close();
                return Some(ChatAction::Execute(submitted));
            } else {
                self.close();
                return Some(ChatAction::Close);
            }
        }

        // 2. Escape key -> cancel and close
        let esc_pressed = input.key_escape && !self.was_escape;
        self.was_escape = input.key_escape;
        if esc_pressed {
            self.close();
            return Some(ChatAction::Close);
        }

        // 3. Arrow keys: Up/Down for command history
        let up_pressed = input.key_up && !self.was_up;
        self.was_up = input.key_up;
        let down_pressed = input.key_down && !self.was_down;
        self.was_down = input.key_down;

        if up_pressed && !self.history.is_empty() {
            if self.history_idx.is_none() {
                self.saved_draft = self.input_text.clone();
                let new_idx = self.history.len() - 1;
                self.history_idx = Some(new_idx);
                self.input_text = self.history[new_idx].clone();
                self.cursor_pos = self.input_text.len();
            } else if let Some(idx) = self.history_idx {
                if idx > 0 {
                    let new_idx = idx - 1;
                    self.history_idx = Some(new_idx);
                    self.input_text = self.history[new_idx].clone();
                    self.cursor_pos = self.input_text.len();
                }
            }
            self.cursor_blink = 0.0;
        } else if down_pressed && self.history_idx.is_some() {
            let idx = self.history_idx.unwrap();
            if idx + 1 < self.history.len() {
                let new_idx = idx + 1;
                self.history_idx = Some(new_idx);
                self.input_text = self.history[new_idx].clone();
                self.cursor_pos = self.input_text.len();
            } else {
                self.history_idx = None;
                self.input_text = self.saved_draft.clone();
                self.cursor_pos = self.input_text.len();
            }
            self.cursor_blink = 0.0;
        }

        // 4. Cursor Navigation (Left / Right)
        let left_pressed = input.key_left && !self.was_left;
        self.was_left = input.key_left;
        if left_pressed && self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.cursor_blink = 0.0;
        }

        let right_pressed = input.key_right && !self.was_right;
        self.was_right = input.key_right;
        if right_pressed && self.cursor_pos < self.input_text.len() {
            self.cursor_pos += 1;
            self.cursor_blink = 0.0;
        }

        // 5. Backspace & Delete
        let bs_pressed = input.key_backspace && !self.was_backspace;
        self.was_backspace = input.key_backspace;
        if bs_pressed && self.cursor_pos > 0 {
            self.input_text.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.cursor_blink = 0.0;
        }

        let del_pressed = input.key_del && !self.was_delete;
        self.was_delete = input.key_del;
        if del_pressed && self.cursor_pos < self.input_text.len() {
            self.input_text.remove(self.cursor_pos);
            self.cursor_blink = 0.0;
        }

        // 6. Character typing
        if self.ignore_slash_until_release && !input.key_slash {
            self.ignore_slash_until_release = false;
        }

        for &ch in &input.typed_chars {
            if self.ignore_slash_until_release && ch == '/' {
                continue;
            }
            if !ch.is_control() {
                self.input_text.insert(self.cursor_pos, ch);
                self.cursor_pos += 1;
                self.cursor_blink = 0.0;
            }
        }

        None
    }

    /// Renders chat messages and active command prompt overlay.
    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize) {
        let base_x = 8i32;
        let line_height = 11i32;
        let max_visible_lines = 12;
        let input_y = height as i32 - 18;

        // 1. Draw message history (from bottom to top)
        let mut y = input_y - 3;
        let mut rendered = 0;

        for msg in self.messages.iter().rev() {
            if rendered >= max_visible_lines {
                break;
            }

            // In closed state, messages fade out after 10s
            if !self.is_open && msg.age > 10.0 {
                continue;
            }

            let alpha = if self.is_open {
                0.70f32
            } else {
                (1.0 - (msg.age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0) * 0.70
            };

            if alpha <= 0.02 {
                continue;
            }

            y -= line_height;
            let text_w = crate::graphics::font::measure_text(&msg.text, 1, 1);
            let bar_w = (text_w + 6).min(width as i32 - base_x * 2);

            fill_rect_alpha(
                buffer,
                width,
                height,
                base_x - 3,
                y - 2,
                bar_w,
                line_height,
                pack_color(0.0, 0.0, 0.0),
                alpha,
            );

            let text_alpha = if self.is_open {
                1.0
            } else {
                (1.0 - (msg.age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0)
            };

            if text_alpha > 0.05 {
                draw_text_colored_shadowed(
                    buffer,
                    width,
                    height,
                    &msg.text,
                    base_x,
                    y,
                    1,
                    1,
                    msg.color,
                );
            }

            rendered += 1;
        }

        // 2. Draw chat input bar when open
        if self.is_open {
            let bar_w = (width as i32 - base_x * 2).max(100);
            let bar_h = 14i32;

            fill_rect_alpha(
                buffer,
                width,
                height,
                base_x - 3,
                input_y - 2,
                bar_w + 6,
                bar_h,
                pack_color(0.0, 0.0, 0.0),
                0.80,
            );
            fill_rect_alpha(
                buffer,
                width,
                height,
                base_x - 3,
                input_y + bar_h - 2,
                bar_w + 6,
                1,
                pack_color(0.45, 0.45, 0.45),
                0.90,
            );

            let prefix = "> ";
            let display_str = format!("{}{}", prefix, self.input_text);
            let white = pack_color(1.0, 1.0, 1.0);
            draw_text_colored_shadowed(
                buffer,
                width,
                height,
                &display_str,
                base_x,
                input_y + 1,
                1,
                1,
                white,
            );

            // Blinking cursor
            let show_cursor = (self.cursor_blink % 0.8) < 0.4;
            if show_cursor {
                let cursor_sub = format!("{}{}", prefix, &self.input_text[..self.cursor_pos]);
                let cursor_x = base_x + crate::graphics::font::measure_text(&cursor_sub, 1, 1);
                fill_rect(buffer, width, height, cursor_x, input_y + 1, 2, 7, white);
            }
        }
    }

    /// Dispatches and executes chat command or regular text message.
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

    /// Parses and executes slash commands like `/gamemode`, `/time`, `/give`, `/tp`, etc.
    pub fn execute_command(&mut self, full_cmd: &str, ctx: &mut CommandContext) {
        crate::log_info!("Command", "Executing: {}", full_cmd);
        let trimmed = full_cmd.trim_start_matches('/');
        let mut parts = trimmed.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c.to_ascii_lowercase(),
            None => return,
        };

        match cmd.as_str() {
            "help" | "?" => {
                self.add_info("--- Showing help page (Minecraft Commands) ---");
                self.add_info("/gamemode <survival|creative> - Change game mode");
                self.add_info("/time set <day|noon|night|midnight|number> - Set time");
                self.add_info("/give <item> [count] - Add items to inventory");
                self.add_info("/tp <x> <y> <z> - Teleport player");
                self.add_info("/spawn <pig|cow|chicken|zombie|skeleton|creeper> [count] - Spawn mob");
                self.add_info("/mobs - List all currently spawned mobs in the world");
                self.add_info("/heal - Restore health and hunger");
                self.add_info("/kill - Kill the player");
                self.add_info("/clear [chat] - Clear inventory or chat");
                self.add_info("/seed - Display world generation seed");
                self.add_info("/reloadchunks - Reload chunks and rebuild mesh cache");
                self.add_info("/fly - Toggle flight mode");
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
                    _ => {
                        self.add_error("Usage: /gamemode <survival|creative>");
                    }
                }
            }

            "time" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "set" => {
                        let val = parts.next().unwrap_or("").to_ascii_lowercase();
                        match val.as_str() {
                            "day" | "1000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.25;
                                self.add_success("Set the time to 1000 (Day)");
                            }
                            "noon" | "6000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.50;
                                self.add_success("Set the time to 6000 (Noon)");
                            }
                            "night" | "13000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.75;
                                self.add_success("Set the time to 13000 (Night)");
                            }
                            "midnight" | "18000" => {
                                ctx.world.time_of_day = 0.0;
                                self.add_success("Set the time to 18000 (Midnight)");
                            }
                            other => {
                                if let Ok(ticks) = other.parse::<f32>() {
                                    let frac = (ticks % 24000.0) / 24000.0;
                                    ctx.world.time_of_day = World::DAY_DURATION * frac;
                                    self.add_success(format!("Set the time to {:.0}", ticks));
                                } else {
                                    self.add_error("Usage: /time set <day|noon|night|midnight|number>");
                                }
                            }
                        }
                    }
                    "query" => {
                        let ticks = (ctx.world.day_fraction() * 24000.0) as u32;
                        self.add_info(format!("The time is {} ticks", ticks));
                    }
                    _ => {
                        self.add_error("Usage: /time <set|query> <value>");
                    }
                }
            }

            "give" => {
                let item_arg = match parts.next() {
                    Some(a) => a,
                    None => {
                        self.add_error("Usage: /give <item> [count]");
                        return;
                    }
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
                        self.add_error(format!("Inventory full! Could not give {} items.", leftover));
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
                    self.add_error("Usage: /tp <x> <y> <z>");
                }
            }

            "spawn" => {
                let mut kind_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                if kind_arg == "mob" {
                    kind_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                }
                let count: usize = parts.next().and_then(|c| c.parse().ok()).unwrap_or(1).clamp(1, 20);

                let mob_kind = match kind_arg.as_str() {
                    "pig" | "swinia" => Some(MobKind::Pig),
                    "cow" | "krowa" => Some(MobKind::Cow),
                    "chicken" | "kurczak" => Some(MobKind::Chicken),
                    "sheep" | "owca" => Some(MobKind::Sheep),
                    "zombie" | "zombi" => Some(MobKind::Zombie),
                    "skeleton" | "szkielet" => Some(MobKind::Skeleton),
                    "creeper" => Some(MobKind::Creeper),
                    "dolphin" | "delfin" => Some(MobKind::Dolphin),
                    "cod" | "dorsz" => Some(MobKind::Cod),
                    "salmon" | "losos" => Some(MobKind::Salmon),
                    "pufferfish" | "rozdymka" => Some(MobKind::Pufferfish),
                    "tropical_fish" | "tropikalna_ryba" => Some(MobKind::TropicalFish),
                    "villager" | "osadnik" | "wiesniak" => Some(MobKind::Villager),
                    _ => None,
                };

                if let Some(kind) = mob_kind {
                    let forward = ctx.camera.forward_flat();
                    for i in 0..count {
                        let offset = forward * (2.0 + (i as f32 * 0.8));
                        let spawn_pos = ctx.camera.position + offset;
                        ctx.mob_manager.spawn_mob(kind, spawn_pos);
                        ctx.particles.spawn_mob_hit(spawn_pos);
                    }
                    self.add_success(format!("Spawned {} {:?}(s)", count, kind));
                    crate::log_debug!("Command", "Spawned {} {:?} mobs", count, kind);
                } else {
                    self.add_error("Usage: /spawn <pig|cow|chicken|sheep|zombie|skeleton|creeper|dolphin|cod|salmon|pufferfish|tropical_fish|villager> [count]");
                }
            }

            "mobs" | "moblist" | "listmobs" => {
                self.list_mobs_debug(ctx);
            }

            "debug" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "mobs" | "moblist" => {
                        self.list_mobs_debug(ctx);
                    }
                    _ => {
                        self.add_info("Debug commands: /mobs, /debug mobs");
                    }
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

            "seed" => {
                let seed = ctx.world.seed;
                self.add_info(format!("Seed: [{}]", seed));
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

            "reload" | "reloadchunks" => {
                ctx.world.mark_all_chunks_dirty();
                *ctx.reload_chunks_requested = true;
                self.add_success("Reloading all chunks and rebuilding mesh cache...");
                crate::log_info!("WORLD", "Chunk reload triggered via chat command");
            }

            "weather" => {
                let w = parts.next().unwrap_or("clear");
                self.add_info(format!("Set weather to {}", w));
            }

            "difficulty" => {
                let d = parts.next().unwrap_or("normal");
                self.add_info(format!("Set difficulty to {}", d));
            }

            unknown => {
                if let Some(cmd_def) = crate::engine::modding::find_mod_command(&cmd) {
                    crate::engine::modding::execute_mod_command(&cmd_def, self, ctx);
                } else {
                    self.add_error(format!("Unknown command: '/{}'. Type /help for help.", unknown));
                }
            }
        }
    }

    /// Debug helper to display all currently spawned mobs in the world.
    fn list_mobs_debug(&mut self, ctx: &CommandContext) {
        let total = ctx.mob_manager.mobs.len();
        if total == 0 {
            self.add_info("No mobs currently spawned in the world.");
            return;
        }

        let player_pos = ctx.camera.position;
        self.add_info(format!("=== Spawned Mobs (Total: {}) ===", total));

        let mut pigs = 0;
        let mut cows = 0;
        let mut chickens = 0;
        let mut sheep = 0;
        let mut zombies = 0;
        let mut skeletons = 0;
        let mut creepers = 0;
        let mut dolphins = 0;
        let mut cod = 0;
        let mut salmon = 0;
        let mut pufferfish = 0;
        let mut tropical_fish = 0;
        let mut villagers = 0;

        for mob in &ctx.mob_manager.mobs {
            match mob.kind {
                MobKind::Pig => pigs += 1,
                MobKind::Cow => cows += 1,
                MobKind::Chicken => chickens += 1,
                MobKind::Sheep => sheep += 1,
                MobKind::Zombie => zombies += 1,
                MobKind::Skeleton => skeletons += 1,
                MobKind::Creeper => creepers += 1,
                MobKind::Dolphin => dolphins += 1,
                MobKind::Cod => cod += 1,
                MobKind::Salmon => salmon += 1,
                MobKind::Pufferfish => pufferfish += 1,
                MobKind::TropicalFish => tropical_fish += 1,
                MobKind::Villager => villagers += 1,
            }
            let dist = (mob.pos - player_pos).length();
            let hostile_tag = if mob.kind.is_hostile() { " [HOSTILE]" } else { "" };
            self.add_message(
                format!(
                    "#{:02} {} at ({:.1}, {:.1}, {:.1}) | Dist: {:.1}m | HP: {:.0}/{:.0}{}",
                    mob.id,
                    mob.kind.name(),
                    mob.pos.x,
                    mob.pos.y,
                    mob.pos.z,
                    dist,
                    mob.health.max(0.0),
                    mob.kind.max_health(),
                    hostile_tag,
                ),
                if mob.kind.is_hostile() {
                    pack_color(1.0, 0.45, 0.45)
                } else {
                    pack_color(0.55, 1.0, 0.55)
                },
            );
        }

        self.add_info(format!(
            "Summary: Pigs: {}, Cows: {}, Chickens: {}, Sheep: {}, Zombies: {}, Skeletons: {}, Creepers: {}, Dolphins: {}, Fish: {}, Villagers: {}",
            pigs, cows, chickens, sheep, zombies, skeletons, creepers, dolphins, cod + salmon + pufferfish + tropical_fish, villagers
        ));
    }
}

/// Parses a vanilla item/block string or numeric ID into a `BlockId`.
pub fn parse_vanilla_block_id(s: &str) -> Option<BlockId> {
    let s = s.trim().to_ascii_lowercase().replace('-', "_");
    match s.as_str() {
        "grass" | "grass_block" | "trawa" => Some(BlockId::Grass),
        "dirt" | "ziemia" => Some(BlockId::Dirt),
        "stone" | "kamien" | "rock" | "cobblestone" => Some(BlockId::Stone),
        "sand" | "piasek" => Some(BlockId::Sand),
        "wood" | "log" | "drewno" | "oak_log" => Some(BlockId::Wood),
        "planks" | "deski" | "wood_planks" => Some(BlockId::Planks),
        "leaves" | "liscie" => Some(BlockId::Leaves),
        "glass" | "szyba" => Some(BlockId::Glass),
        "ice" | "lod" => Some(BlockId::Ice),
        "tnt" => Some(BlockId::Tnt),
        "crafting_table" | "workbench" => Some(BlockId::CraftingTable),
        "chest" | "skrzynia" => Some(BlockId::Chest),
        "furnace" | "piec" => Some(BlockId::Furnace),
        "torch" | "pochodnia" => Some(BlockId::Torch),
        "bedrock" => Some(BlockId::Bedrock),
        "water" | "woda" => Some(BlockId::Water),
        "lava" | "lawa" => Some(BlockId::Lava),
        "mud" | "bloto" => Some(BlockId::Mud),
        "coal_ore" | "coal" | "wegiel" => Some(BlockId::CoalOre),
        "iron_ore" | "zelazo" => Some(BlockId::IronOre),
        "gold_ore" | "zloto" => Some(BlockId::GoldOre),
        "iron_ingot" | "ingot" => Some(BlockId::IronIngot),
        "gold_ingot" => Some(BlockId::GoldIngot),
        "iron_block" => Some(BlockId::IronBlock),
        "gold_block" => Some(BlockId::GoldBlock),
        "coal_block" => Some(BlockId::CoalBlock),
        "stick" | "patyk" => Some(BlockId::Stick),
        "apple" | "jablko" => Some(BlockId::Apple),
        "raw_porkchop" | "porkchop" => Some(BlockId::RawPorkchop),
        "cooked_porkchop" => Some(BlockId::CookedPorkchop),
        "raw_beef" | "beef" => Some(BlockId::RawBeef),
        "cooked_beef" | "steak" => Some(BlockId::CookedBeef),
        "raw_chicken" | "chicken" => Some(BlockId::RawChicken),
        "cooked_chicken" => Some(BlockId::CookedChicken),
        "feather" | "pioro" => Some(BlockId::Feather),
        "wooden_sword" => Some(BlockId::WoodenSword),
        "stone_sword" => Some(BlockId::StoneSword),
        "iron_sword" | "sword" | "diamond_sword" => Some(BlockId::IronSword),
        "wooden_pickaxe" => Some(BlockId::WoodenPickaxe),
        "stone_pickaxe" => Some(BlockId::StonePickaxe),
        "iron_pickaxe" | "pickaxe" | "diamond_pickaxe" => Some(BlockId::IronPickaxe),
        "wooden_axe" => Some(BlockId::WoodenAxe),
        "stone_axe" => Some(BlockId::StoneAxe),
        "iron_axe" | "axe" => Some(BlockId::IronAxe),
        "wooden_shovel" => Some(BlockId::WoodenShovel),
        "stone_shovel" => Some(BlockId::StoneShovel),
        "iron_shovel" | "shovel" => Some(BlockId::IronShovel),
        "pig_spawn_egg" | "pig_egg" => Some(BlockId::PigSpawnEgg),
        "cow_spawn_egg" | "cow_egg" => Some(BlockId::CowSpawnEgg),
        "chicken_spawn_egg" | "chicken_egg" => Some(BlockId::ChickenSpawnEgg),
        "bone" | "kosc" => Some(BlockId::Bone),
        "gunpowder" | "proch" => Some(BlockId::Gunpowder),
        "ruststone" | "rdzawnik" | "redstone" => Some(BlockId::RustStone),
        "ruststone_ore" | "ruda_rdzawnika" | "redstone_ore" => Some(BlockId::RustStoneOre),
        "ruststone_block" | "blok_rdzawnika" | "redstone_block" => Some(BlockId::RustStoneBlock),
        "ruststone_lamp" | "lampa_rdzawnikowa" | "lampa" | "redstone_lamp" => Some(BlockId::RustStoneLamp),
        "lever" | "dzwignia" => Some(BlockId::Lever),
        "button" | "przycisk" => Some(BlockId::Button),
        "pressure_plate" | "plytka" | "plytka_naciskowa" => Some(BlockId::PressurePlate),
        "shears" | "nozyce" => Some(BlockId::Shears),
        "sheep_spawn_egg" | "sheep_egg" | "owca_jajko" => Some(BlockId::SheepSpawnEgg),
        "wool" | "welna" | "white_wool" | "biala_welna" => Some(BlockId::WhiteWool),
        "orange_wool" | "pomaranczowa_welna" => Some(BlockId::OrangeWool),
        "magenta_wool" | "jasnofioletowa_welna" => Some(BlockId::MagentaWool),
        "light_blue_wool" | "jasnoniebieska_welna" => Some(BlockId::LightBlueWool),
        "yellow_wool" | "zolta_welna" => Some(BlockId::YellowWool),
        "lime_wool" | "jasnozielona_welna" => Some(BlockId::LimeWool),
        "pink_wool" | "rozowa_welna" => Some(BlockId::PinkWool),
        "gray_wool" | "szara_welna" => Some(BlockId::GrayWool),
        "light_gray_wool" | "jasnoszara_welna" => Some(BlockId::LightGrayWool),
        "cyan_wool" | "turkusowa_welna" => Some(BlockId::CyanWool),
        "purple_wool" | "fioletowa_welna" => Some(BlockId::PurpleWool),
        "blue_wool" | "niebieska_welna" => Some(BlockId::BlueWool),
        "brown_wool" | "brazowa_welna" => Some(BlockId::BrownWool),
        "green_wool" | "zielona_welna" => Some(BlockId::GreenWool),
        "red_wool" | "czerwona_welna" => Some(BlockId::RedWool),
        "black_wool" | "czarna_welna" => Some(BlockId::BlackWool),
        "kelp" | "wodorosty" => Some(BlockId::Kelp),
        "seagrass" | "trawa_morska" => Some(BlockId::Seagrass),
        "raw_cod" | "cod" | "dorsz" => Some(BlockId::RawCod),
        "cooked_cod" | "pieczony_dorsz" => Some(BlockId::CookedCod),
        "raw_salmon" | "salmon" | "losos" => Some(BlockId::RawSalmon),
        "cooked_salmon" | "pieczony_losos" => Some(BlockId::CookedSalmon),
        "pufferfish" | "rozdymka" => Some(BlockId::Pufferfish),
        "tropical_fish" | "tropikalna_ryba" => Some(BlockId::TropicalFish),
        "dried_kelp" | "suszone_wodorosty" => Some(BlockId::DriedKelp),
        "dolphin_spawn_egg" | "dolphin_egg" | "jajko_delfina" => Some(BlockId::DolphinSpawnEgg),
        "fish_spawn_egg" | "fish_egg" | "jajko_ryby" => Some(BlockId::FishSpawnEgg),
        "villager_spawn_egg" | "villager_egg" | "jajko_osadnika" | "jajko_wiesniaka" => Some(BlockId::VillagerSpawnEgg),
        "emerald" | "szmaragd" => Some(BlockId::Emerald),
        "emerald_ore" | "ruda_szmaragdu" => Some(BlockId::EmeraldOre),
        "emerald_block" | "blok_szmaragdu" => Some(BlockId::EmeraldBlock),
        "fishing_rod" | "wedka" => Some(BlockId::FishingRod),
        _ => {
            if let Ok(id) = s.parse::<u8>() {
                if id > 0 {
                    return Some(BlockId::from_u8(id));
                }
            }
            None
        }
    }
}

/// Parses an item/block string into a `BlockId` (supports both vanilla and registered mod items).
pub fn parse_block_id(s: &str) -> Option<BlockId> {
    if let Some(b) = parse_vanilla_block_id(s) {
        return Some(b);
    }
    let s = s.trim().to_ascii_lowercase().replace('-', "_");
    crate::engine::modding::find_custom_block_by_name(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_open_close_and_history() {
        let mut chat = ChatState::new();
        assert!(!chat.is_open);

        chat.open("hello");
        assert!(chat.is_open);
        assert_eq!(chat.input_text, "hello");
        assert_eq!(chat.cursor_pos, 5);

        chat.close();
        assert!(!chat.is_open);
        assert_eq!(chat.input_text, "");
    }

    #[test]
    fn test_parse_block_id_items() {
        assert_eq!(parse_block_id("diamond_sword"), Some(BlockId::IronSword));
        assert_eq!(parse_block_id("wood"), Some(BlockId::Wood));
        assert_eq!(parse_block_id("raw_porkchop"), Some(BlockId::RawPorkchop));
        assert_eq!(parse_block_id("cooked_beef"), Some(BlockId::CookedBeef));
        assert_eq!(parse_block_id("apple"), Some(BlockId::Apple));
        assert_eq!(parse_block_id("torch"), Some(BlockId::Torch));
        assert_eq!(parse_block_id("bone"), Some(BlockId::Bone));
        assert_eq!(parse_block_id("kosc"), Some(BlockId::Bone));
        assert_eq!(parse_block_id("gunpowder"), Some(BlockId::Gunpowder));
        assert_eq!(parse_block_id("proch"), Some(BlockId::Gunpowder));
    }

    #[test]
    fn test_chat_mobs_debug_command() {
        let mut chat = ChatState::new();
        let mut mob_manager = MobManager::new();
        mob_manager.spawn_mob(MobKind::Zombie, Vec3::new(10.0, 64.0, 10.0));
        mob_manager.spawn_mob(MobKind::Skeleton, Vec3::new(15.0, 64.0, 15.0));
        mob_manager.spawn_mob(MobKind::Creeper, Vec3::new(8.0, 64.0, 8.0));

        let mut player = Player::new();
        let mut camera = Camera::new(Vec3::new(0.0, 64.0, 0.0));
        let mut world = World::new();
        let mut items = Vec::new();
        let mut particles = ParticleSystem::new();
        let mut is_flying = false;
        let mut fly_vel = Vec3::zero();
        let mut horiz_vel = Vec3::zero();
        let mut reload_chunks = false;

        let mut ctx = CommandContext {
            player: &mut player,
            camera: &mut camera,
            world: &mut world,
            mob_manager: &mut mob_manager,
            items: &mut items,
            particles: &mut particles,
            is_flying: &mut is_flying,
            fly_velocity: &mut fly_vel,
            horiz_velocity: &mut horiz_vel,
            reload_chunks_requested: &mut reload_chunks,
        };

        chat.execute_command("/mobs", &mut ctx);
        assert!(!chat.messages.is_empty());
        assert!(chat.messages.iter().any(|m| m.text.contains("Spawned Mobs")));
        assert!(chat.messages.iter().any(|m| m.text.contains("Zombie")));
        assert!(chat.messages.iter().any(|m| m.text.contains("Skeleton")));
    }
}
