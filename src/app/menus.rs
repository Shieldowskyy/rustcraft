#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use std::sync::mpsc;
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use std::thread;

use crate::cli::generate_random_seed;
use crate::core::controls::Action;
use crate::core::math::Vec3;
use crate::core::profiler::FrameProfiler;
use crate::gameplay::mob::MobManager;
use crate::gameplay::player::GameMode;
use crate::graphics::camera::Camera;
use crate::ui::VISIBLE_SAVE_ROWS;
use crate::world::storage::{delete_save, get_save_metadata, save_current_world, save_path};
use crate::world::World;
use super::{AppOutput, GameState, InputState, PreviousMenu, RustCraftApp, MAX_SAVE_SLOTS};
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use super::WorldLoadResult;

impl RustCraftApp {
    pub(crate) fn update_and_render_main_menu(
        &mut self,
        input: &InputState,
        mouse_moved: bool,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let q_down = input.key_q;
        let enter_down = input.key_enter;
        let n_down = input.key_n;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                let (mouse_nick_hovered, mouse_hovered_btn) = if self.is_gamepad_active && !mouse_moved {
                    (false, None)
                } else if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    let (nx, ny, nw, nh) = self.main_menu_nick_rect;
                    let nick = px >= nx && px < nx + nw && py >= ny && py < ny + nh;
                    let btn = self.main_menu_rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    (nick, btn)
                } else {
                    (false, None)
                };

                self.main_menu_nick_hovered = mouse_nick_hovered;
                let mut hovered_idx = self.menu_hovered_btn;
                if let Some(btn) = mouse_hovered_btn {
                    hovered_idx = Some(btn);
                    self.menu_hovered_btn = Some(btn);
                } else if !self.is_gamepad_active && mouse_pos.is_some() {
                    hovered_idx = None;
                }

                if input.gp_dpad_up_just_pressed || input.gp_menu_up || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.menu_hovered_btn {
                        Some(0) | None => 3,
                        Some(i) => i - 1,
                    };
                    self.menu_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if input.gp_dpad_down_just_pressed || input.gp_menu_down || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.menu_hovered_btn {
                        Some(3) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.menu_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_button = if left_down && !self.prev_left_down {
                    mouse_hovered_btn
                } else if input.gp_south_just_pressed {
                    hovered_idx
                } else {
                    None
                };

                let click_nick = (left_down && !self.prev_left_down && self.main_menu_nick_hovered)
                    || (n_down && !self.prev_n_down)
                    || input.gp_north_just_pressed;
                if click_nick {
                    self.open_edit_nickname(PreviousMenu::MainMenu);
                }

                if (enter_down && !self.prev_enter_down && (hovered_idx == Some(0) || hovered_idx.is_none())) || click_button == Some(0) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    self.state = GameState::SaveSelect;
                    self.save_select_slot = self.save_slot;
                    self.save_select_scroll = ((self.save_slot as usize - 1).saturating_sub(1))
                        .min(MAX_SAVE_SLOTS as usize - VISIBLE_SAVE_ROWS);
                    crate::log_info!("MENU", "Opened Save Selection menu from Main Menu");
                } else if click_button == Some(1) || (enter_down && !self.prev_enter_down && hovered_idx == Some(1)) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    self.state = GameState::MultiplayerMenu;
                    self.multiplayer_status = None;
                    self.multiplayer_is_connecting = false;
                    self.selected_lan_server = None;
                    self.lan_discovery.refresh();
                    crate::log_info!("MENU", "Opened Multiplayer menu from Main Menu");
                } else if click_button == Some(2) || (enter_down && !self.prev_enter_down && hovered_idx == Some(2)) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    self.state = GameState::Settings(PreviousMenu::MainMenu);
                    self.settings_hovered_opt = Some(0);
                    crate::log_info!("MENU", "Opened Settings menu from Main Menu");
                } else if (q_down && !self.prev_q_down) || click_button == Some(3) || input.gp_east_just_pressed || (enter_down && !self.prev_enter_down && hovered_idx == Some(3)) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    out.quit_requested = true;
                    crate::log_info!("STATE", "Quit requested from Main Menu");
                }

                self.renderer.draw_main_menu(hovered_idx, &self.settings.nickname, self.main_menu_nick_hovered, self.is_gamepad_active);

    }

    pub(crate) fn update_and_render_nickname_dialog(
        &mut self,
        input: &InputState,
        prev: PreviousMenu,
        _out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let escape_down = input.key_escape;
        let q_down = input.key_q;
        let a_down = input.key_a;
        let c_down = input.key_c;
        let v_down = input.key_v;
        let ctrl_down = input.key_ctrl;
        let enter_down = input.key_enter;
        let del_down = input.key_del;
        let backspace_down = input.key_backspace;
        let tab_down = input.key_tab;
        let left_arrow_down = input.key_left;
        let right_arrow_down = input.key_right;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                let rects = self.nickname_dialog_rects;
                let mut hovered_idx = self.nickname_dialog_hovered_btn;

                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            self.nickname_dialog_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                // D-Pad / arrows navigation between elements
                let left_just = (left_arrow_down && !self.prev_left_arrow_down) || input.gp_dpad_left_just_pressed || input.gp_menu_left;
                let right_just = (right_arrow_down && !self.prev_right_arrow_down) || input.gp_dpad_right_just_pressed || input.gp_menu_right;
                let up_just = (up_arrow_down && !self.prev_up_arrow_down) || input.gp_dpad_up_just_pressed || input.gp_menu_up;
                let down_just = (down_arrow_down && !self.prev_down_arrow_down) || input.gp_dpad_down_just_pressed || input.gp_menu_down;
                let tab_just = tab_down && !self.prev_tab_down;

                if left_just {
                    let cur = hovered_idx.unwrap_or(1);
                    let next = if cur <= 1 { 3 } else { cur - 1 };
                    hovered_idx = Some(next);
                    self.nickname_dialog_hovered_btn = Some(next);
                } else if right_just || tab_just {
                    let cur = hovered_idx.unwrap_or(0);
                    let next = if cur >= 3 { 1 } else { cur + 1 };
                    hovered_idx = Some(next);
                    self.nickname_dialog_hovered_btn = Some(next);
                } else if up_just {
                    hovered_idx = Some(0);
                    self.nickname_dialog_hovered_btn = Some(0);
                } else if down_just {
                    if hovered_idx == Some(0) || hovered_idx.is_none() {
                        hovered_idx = Some(1);
                        self.nickname_dialog_hovered_btn = Some(1);
                    }
                }

                // Typing handling
                if !ctrl_down {
                    for &ch in &input.typed_chars {
                        if !ch.is_control() && (ch.is_alphanumeric() || ch == '_' || ch == '-') {
                            if self.nickname_selected {
                                self.nickname_input.clear();
                                self.nickname_selected = false;
                            }
                            if self.nickname_input.len() < 24 {
                                self.nickname_input.push(ch);
                            }
                        }
                    }
                }

                let backspace_just = (backspace_down && !self.prev_backspace_down) || (del_down && !self.prev_del_down);
                if backspace_just {
                    if self.nickname_selected {
                        self.nickname_input.clear();
                        self.nickname_selected = false;
                    } else {
                        self.nickname_input.pop();
                    }
                }

                // Clipboard shortcuts: Ctrl+A, Ctrl+C, Ctrl+V
                if ctrl_down {
                    if a_down && !self.prev_a_down {
                        self.nickname_selected = true;
                    } else if c_down && !self.prev_c_down {
                        crate::ui::clipboard::set_clipboard(&self.nickname_input);
                    } else if v_down && !self.prev_v_down {
                        let clip = crate::ui::clipboard::get_clipboard();
                        let filtered: String = clip.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();
                        if self.nickname_selected {
                            self.nickname_input.clear();
                            self.nickname_selected = false;
                        }
                        for ch in filtered.chars() {
                            if self.nickname_input.len() < 24 {
                                self.nickname_input.push(ch);
                            }
                        }
                    }
                }

                // Randomize shortcut: Gamepad West (X)
                let r_just = input.gp_west_just_pressed;
                if r_just {
                    self.nickname_input = crate::core::settings::generate_default_nickname();
                    self.nickname_selected = false;
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                }

                let click = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
                let enter_just = enter_down && !self.prev_enter_down;
                let esc_just = (escape_down && !self.prev_escape_down) || (q_down && !self.prev_q_down) || input.gp_east_just_pressed;

                if esc_just {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    self.state = match prev {
                        PreviousMenu::MainMenu => GameState::MainMenu,
                        PreviousMenu::Paused => GameState::Paused,
                        PreviousMenu::Settings => GameState::Settings(PreviousMenu::MainMenu),
                        PreviousMenu::MultiplayerMenu => GameState::MultiplayerMenu,
                    };
                } else if enter_just {
                    let trimmed = self.nickname_input.trim();
                    if !trimmed.is_empty() {
                        self.settings.nickname = trimmed.to_string();
                    } else {
                        self.settings.nickname = crate::core::settings::generate_default_nickname();
                    }
                    let _ = self.settings.save();
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    crate::log_info!("SETTINGS", "Nickname saved as {}", self.settings.nickname);
                    self.state = match prev {
                        PreviousMenu::MainMenu => GameState::MainMenu,
                        PreviousMenu::Paused => GameState::Paused,
                        PreviousMenu::Settings => GameState::Settings(PreviousMenu::MainMenu),
                        PreviousMenu::MultiplayerMenu => GameState::MultiplayerMenu,
                    };
                } else if click {
                    if let Some(idx) = hovered_idx {
                        match idx {
                            0 => {
                                self.nickname_selected = false;
                                let cur = self.nickname_input.clone();
                                self.request_ime("Change Nickname", &cur);
                            }
                            1 => {
                                let trimmed = self.nickname_input.trim();
                                if !trimmed.is_empty() {
                                    self.settings.nickname = trimmed.to_string();
                                } else {
                                    self.settings.nickname = crate::core::settings::generate_default_nickname();
                                }
                                let _ = self.settings.save();
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "Nickname saved as {}", self.settings.nickname);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    PreviousMenu::Settings => GameState::Settings(PreviousMenu::MainMenu),
                                    PreviousMenu::MultiplayerMenu => GameState::MultiplayerMenu,
                                };
                            }
                            2 => {
                                self.nickname_input = crate::core::settings::generate_default_nickname();
                                self.nickname_selected = false;
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            }
                            3 => {
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    PreviousMenu::Settings => GameState::Settings(PreviousMenu::MainMenu),
                                    PreviousMenu::MultiplayerMenu => GameState::MultiplayerMenu,
                                };
                            }
                            _ => {}
                        }
                    }
                }

                // Render background screen based on previous menu
                match prev {
                    PreviousMenu::MainMenu => {
                        self.renderer.draw_main_menu(None, &self.settings.nickname, false, self.is_gamepad_active);
                    }
                    PreviousMenu::Paused => {
                        self.renderer.draw_pause_menu(self.fly_mode, None, self.is_gamepad_active);
                    }
                    PreviousMenu::Settings => {
                        let server_ip = self.profiler_server.as_ref().map(|s| s.ip_address.as_str()).unwrap_or("127.0.0.1");
                        self.renderer.draw_settings_menu(&self.settings, self.settings_tab, None, None, server_ip, false, mouse_pos, self.is_gamepad_active);
                    }
                    PreviousMenu::MultiplayerMenu => {
                        self.renderer.draw_multiplayer_menu(&self.settings.nickname, &self.multiplayer_server_addr, self.multiplayer_focus, None, None, false, &self.lan_discovery.servers, self.selected_lan_server, false);
                    }
                }

                let cursor_visible = (self.total_time * 2.5).fract() < 0.5;
                self.renderer.draw_nickname_dialog(
                    &self.nickname_input,
                    self.nickname_selected,
                    hovered_idx,
                    cursor_visible,
                    self.is_gamepad_active,
                );

    }

    pub(crate) fn update_and_render_save_select(
        &mut self,
        dt: f32,
        input: &InputState,
        mouse_moved: bool,
        _out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let escape_down = input.key_escape;
        let enter_down = input.key_enter;
        let n_down = input.key_n;
        let del_down = input.key_del;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                if self.save_card_double_click_timer > 0.0 {
                    self.save_card_double_click_timer = (self.save_card_double_click_timer - dt).max(0.0);
                }

                if input.scroll_delta > 0.0 {
                    self.save_select_scroll = self.save_select_scroll.saturating_sub(1);
                } else if input.scroll_delta < 0.0 {
                    let max_scroll = (MAX_SAVE_SLOTS as usize).saturating_sub(VISIBLE_SAVE_ROWS);
                    self.save_select_scroll = (self.save_select_scroll + 1).min(max_scroll);
                } else if input.gp_lb_just_pressed {
                    self.save_select_scroll = self.save_select_scroll.saturating_sub(VISIBLE_SAVE_ROWS);
                    self.save_select_slot = (self.save_select_scroll + 1) as u8;
                } else if input.gp_rb_just_pressed {
                    let max_scroll = (MAX_SAVE_SLOTS as usize).saturating_sub(VISIBLE_SAVE_ROWS);
                    self.save_select_scroll = (self.save_select_scroll + VISIBLE_SAVE_ROWS).min(max_scroll);
                    self.save_select_slot = (self.save_select_scroll + 1) as u8;
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    self.save_select_slot = if self.save_select_slot <= 1 { MAX_SAVE_SLOTS } else { self.save_select_slot - 1 };
                    let slot_idx = self.save_select_slot as usize - 1;
                    if slot_idx < self.save_select_scroll {
                        self.save_select_scroll = slot_idx;
                    } else if slot_idx >= self.save_select_scroll + VISIBLE_SAVE_ROWS {
                        self.save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
                    }
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    self.save_select_slot = if self.save_select_slot >= MAX_SAVE_SLOTS { 1 } else { self.save_select_slot + 1 };
                    let slot_idx = self.save_select_slot as usize - 1;
                    if slot_idx >= self.save_select_scroll + VISIBLE_SAVE_ROWS {
                        self.save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
                    } else if slot_idx < self.save_select_scroll {
                        self.save_select_scroll = slot_idx;
                    }
                }

                let (save_select_hovered_card, save_select_hovered_btn) = if self.is_gamepad_active && !mouse_moved {
                    (None, None)
                } else if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;

                    // Save cards hover
                    let card = self.save_card_rects.iter().enumerate().find_map(|(row, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(row)
                        } else {
                            None
                        }
                    });

                    // Bottom buttons hover
                    let btn = self.save_btn_rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(i)
                        } else {
                            None
                        }
                    });

                    (card, btn)
                } else {
                    (None, None)
                };

                let mut card_launch_action = false;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;

                    // Save cards touch/mouse click
                    if left_down && !self.prev_left_down {
                        for (row, &(bx, by, bw, bh)) in self.save_card_rects.iter().enumerate() {
                            if px >= bx && px < bx + bw && py >= by && py < by + bh {
                                let target_slot = (self.save_select_scroll + row + 1) as u8;
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                if self.save_card_last_clicked_slot == Some(target_slot) && self.save_card_double_click_timer > 0.0 {
                                    card_launch_action = true;
                                    self.save_card_double_click_timer = 0.0;
                                } else {
                                    self.save_card_last_clicked_slot = Some(target_slot);
                                    self.save_card_double_click_timer = 0.35;
                                }
                                self.save_select_slot = target_slot;
                                break;
                            }
                        }
                    }

                    // Scrollbar touch and drag
                    let cx = self.width as i32 / 2;
                    let sb_x = cx + 340;
                    let sb_w = 40; // enlarged touch target for scrollbar
                    let sb_y = 75;
                    let sb_h = (VISIBLE_SAVE_ROWS as i32) * 80 - 10;
                    if px >= sb_x && px <= sb_x + sb_w && py >= sb_y && py <= sb_y + sb_h && left_down {
                        let rel_y = ((py - sb_y) as f32 / sb_h as f32).clamp(0.0, 1.0);
                        let max_scroll = (MAX_SAVE_SLOTS as usize).saturating_sub(VISIBLE_SAVE_ROWS);
                        self.save_select_scroll = (rel_y * max_scroll as f32).round() as usize;
                    }
                }

                let mouse_clicked_btn = if left_down && !self.prev_left_down {
                    save_select_hovered_btn
                } else {
                    None
                };

                if mouse_clicked_btn.is_some() {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                }

                let play_action = (enter_down && !self.prev_enter_down)
                    || mouse_clicked_btn == Some(0)
                    || card_launch_action
                    || input.gp_south_just_pressed;
                let new_action = (n_down && !self.prev_n_down)
                    || mouse_clicked_btn == Some(1)
                    || input.gp_north_just_pressed;
                let delete_action = (del_down && !self.prev_del_down)
                    || mouse_clicked_btn == Some(2)
                    || input.gp_west_just_pressed;
                let back_action = (escape_down && !self.prev_escape_down)
                    || mouse_clicked_btn == Some(3)
                    || input.gp_east_just_pressed;

                if mouse_clicked_btn.is_none() && (play_action || new_action || delete_action || back_action) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                }

                if play_action {
                    self.save_slot = self.save_select_slot;
                    let meta = &self.save_metadata_cache[self.save_slot as usize - 1];
                    if meta.exists {
                        let path = save_path(self.save_slot);
                        crate::log_info!("MENU", "Selected existing world in slot {} ('{}', seed: {}) to load", self.save_slot, meta.name, meta.seed);
                        #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
                        {
                            let (tx, rx) = mpsc::channel();
                            thread::spawn(move || {
                                match World::load(&path) {
                                    Ok(loaded_world) => {
                                        let pos = loaded_world.player_position;
                                        let _ = tx.send(WorldLoadResult::Success(loaded_world, Some(pos)));
                                    }
                                    Err(error) => {
                                        let _ = tx.send(WorldLoadResult::Error(error.to_string()));
                                    }
                                }
                            });
                            self.state = GameState::Loading(rx);
                        }
                        #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
                        {
                            match World::load(&path) {
                                Ok(loaded_world) => {
                                    let pos = loaded_world.player_position;
                                    self.player.gamemode = loaded_world.gamemode;
                                    if let Some(ref saved) = loaded_world.saved_player {
                                        self.player.apply_saved_state(saved);
                                    } else {
                                        self.player.reset(loaded_world.gamemode);
                                    }
                                    self.world = loaded_world;
                                    self.camera = Camera::new(pos);
                                    self.mob_manager = MobManager::new();
                                    self.minimap.waypoints = crate::ui::minimap::load_waypoints(self.save_slot);
                                    self.state = GameState::Playing;
                                }
                                Err(_) => {
                                    self.world = World::new_with_seed(self.seed);
                                    self.player.reset(self.world.gamemode);
                                    self.camera = Camera::new(self.world.player_position);
                                    self.mob_manager = MobManager::new();
                                    self.minimap.waypoints = crate::ui::minimap::load_waypoints(self.save_slot);
                                    self.state = GameState::Playing;
                                }
                            }
                        }
                    } else {
                        // Open world creation menu for empty slot
                        self.create_world_slot = self.save_select_slot;
                        self.create_world_name = format!("New World {:02}", self.save_select_slot);
                        self.create_world_seed = String::new();
                        self.create_world_gamemode = GameMode::Survival;
                        self.create_world_focus = 0;
                        self.create_world_selected = false;
                        self.state = GameState::CreateWorld;
                        crate::log_info!("MENU", "Opened Create World menu for empty slot {}", self.create_world_slot);
                    }
                } else if new_action {
                    // Open world creation menu for selected slot
                    self.create_world_slot = self.save_select_slot;
                    self.create_world_name = format!("New World {:02}", self.save_select_slot);
                    self.create_world_seed = String::new();
                    self.create_world_gamemode = GameMode::Survival;
                    self.create_world_focus = 0;
                    self.create_world_selected = false;
                    self.state = GameState::CreateWorld;
                    crate::log_info!("MENU", "Opened Create World menu for slot {}", self.create_world_slot);
                } else if delete_action {
                    crate::log_info!("MENU", "Requested deletion of save slot {}", self.save_select_slot);
                    let _ = delete_save(self.save_select_slot);
                    self.save_metadata_cache[self.save_select_slot as usize - 1] = get_save_metadata(self.save_select_slot);
                } else if back_action {
                    self.state = GameState::MainMenu;
                    crate::log_info!("MENU", "Returned to Main Menu from Save Selection");
                }

                if matches!(self.state, GameState::SaveSelect) {
                    self.renderer.draw_save_select_menu(
                        self.save_select_slot,
                        self.save_select_scroll,
                        &self.save_metadata_cache,
                        save_select_hovered_card,
                        save_select_hovered_btn,
                        self.is_gamepad_active,
                    );
                }

    }

    pub(crate) fn update_and_render_create_world(
        &mut self,
        input: &InputState,
        mouse_moved: bool,
        _out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let escape_down = input.key_escape;
        let a_down = input.key_a;
        let c_down = input.key_c;
        let v_down = input.key_v;
        let x_down = input.key_x;
        let ctrl_down = input.key_ctrl;
        let enter_down = input.key_enter;
        let del_down = input.key_del;
        let backspace_down = input.key_backspace;
        let tab_down = input.key_tab;
        let left_arrow_down = input.key_left;
        let right_arrow_down = input.key_right;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                let hovered_elem = if self.is_gamepad_active && !mouse_moved {
                    None
                } else if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    self.create_world_rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(i)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let click_elem = if left_down && !self.prev_left_down {
                    if let Some(i) = hovered_elem {
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        if self.create_world_focus != i {
                            self.create_world_selected = false;
                        }
                        self.create_world_focus = i;
                        if i == 0 {
                            let name = self.create_world_name.clone();
                            self.request_ime("World Name", &name);
                        } else if i == 1 {
                            let seed = self.create_world_seed.clone();
                            self.request_ime("World Seed", &seed);
                        }
                    }
                    hovered_elem
                } else {
                    None
                };

                let tab_just = tab_down && !self.prev_tab_down;
                let up_just = (up_arrow_down && !self.prev_up_arrow_down) || input.gp_dpad_up_just_pressed || input.gp_menu_up;
                let down_just = (down_arrow_down && !self.prev_down_arrow_down) || input.gp_dpad_down_just_pressed || input.gp_menu_down;
                let left_just = (left_arrow_down && !self.prev_left_arrow_down) || input.gp_dpad_left_just_pressed || input.gp_menu_left;
                let right_just = (right_arrow_down && !self.prev_right_arrow_down) || input.gp_dpad_right_just_pressed || input.gp_menu_right;

                if tab_just || down_just {
                    self.create_world_focus = (self.create_world_focus + 1) % 6;
                    self.create_world_selected = false;
                } else if up_just {
                    self.create_world_focus = if self.create_world_focus == 0 { 5 } else { self.create_world_focus - 1 };
                    self.create_world_selected = false;
                } else if left_just && self.create_world_focus == 5 {
                    self.create_world_focus = 4;
                    self.create_world_selected = false;
                } else if right_just && self.create_world_focus == 4 {
                    self.create_world_focus = 5;
                    self.create_world_selected = false;
                }

                // Keyboard shortcuts: Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X
                let a_just = a_down && !self.prev_a_down;
                let c_just = c_down && !self.prev_c_down;
                let v_just = v_down && !self.prev_v_down;
                let x_just = x_down && !self.prev_x_down;

                if ctrl_down {
                    if a_just {
                        if self.create_world_focus == 0 && !self.create_world_name.is_empty() {
                            self.create_world_selected = true;
                        } else if self.create_world_focus == 1 && !self.create_world_seed.is_empty() {
                            self.create_world_selected = true;
                        }
                    } else if c_just {
                        if self.create_world_focus == 0 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_name);
                        } else if self.create_world_focus == 1 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_seed);
                        }
                    } else if x_just {
                        if self.create_world_focus == 0 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_name);
                            self.create_world_name.clear();
                            self.create_world_selected = false;
                        } else if self.create_world_focus == 1 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_seed);
                            self.create_world_seed.clear();
                            self.create_world_selected = false;
                        }
                    } else if v_just {
                        let clip = crate::ui::clipboard::get_clipboard();
                        let filtered: String = clip.chars().filter(|c| !c.is_control()).collect();
                        if self.create_world_focus == 0 {
                            if self.create_world_selected {
                                self.create_world_name.clear();
                                self.create_world_selected = false;
                            }
                            for ch in filtered.chars() {
                                if self.create_world_name.len() < 24 {
                                    self.create_world_name.push(ch);
                                }
                            }
                        } else if self.create_world_focus == 1 {
                            if self.create_world_selected {
                                self.create_world_seed.clear();
                                self.create_world_selected = false;
                            }
                            for ch in filtered.chars() {
                                if self.create_world_seed.len() < 16 {
                                    self.create_world_seed.push(ch);
                                }
                            }
                        }
                    }
                }

                // Append typed characters to active text input field
                if !ctrl_down {
                    for &ch in &input.typed_chars {
                        if !ch.is_control() {
                            if self.create_world_focus == 0 {
                                if self.create_world_selected {
                                    self.create_world_name.clear();
                                    self.create_world_selected = false;
                                }
                                if self.create_world_name.len() < 24 {
                                    self.create_world_name.push(ch);
                                }
                            } else if self.create_world_focus == 1 {
                                if self.create_world_selected {
                                    self.create_world_seed.clear();
                                    self.create_world_selected = false;
                                }
                                if self.create_world_seed.len() < 16 {
                                    self.create_world_seed.push(ch);
                                }
                            }
                        }
                    }
                }

                let backspace_just = (backspace_down && !self.prev_backspace_down) || (del_down && !self.prev_del_down);
                if backspace_just {
                    if self.create_world_focus == 0 {
                        if self.create_world_selected {
                            self.create_world_name.clear();
                            self.create_world_selected = false;
                        } else {
                            self.create_world_name.pop();
                        }
                    } else if self.create_world_focus == 1 {
                        if self.create_world_selected {
                            self.create_world_seed.clear();
                            self.create_world_selected = false;
                        } else {
                            self.create_world_seed.pop();
                        }
                    }
                }

                if input.gp_north_just_pressed {
                    self.create_world_seed = format!("{}", generate_random_seed());
                    self.create_world_selected = false;
                }
                if input.gp_west_just_pressed {
                    self.create_world_gamemode = match self.create_world_gamemode {
                        GameMode::Survival => GameMode::Creative,
                        GameMode::Creative => GameMode::Survival,
                    };
                }

                let enter_just = enter_down && !self.prev_enter_down;
                let south_just = input.gp_south_just_pressed;

                let do_cancel = (escape_down && !self.prev_escape_down)
                    || input.gp_east_just_pressed
                    || click_elem == Some(5)
                    || (self.create_world_focus == 5 && (enter_just || south_just));

                let do_create = click_elem == Some(4)
                    || (enter_just && (self.create_world_focus == 0 || self.create_world_focus == 1 || self.create_world_focus == 4))
                    || (south_just && self.create_world_focus == 4);

                let do_rand_seed = (self.create_world_focus == 2 && (enter_just || south_just)) || click_elem == Some(2);
                let do_toggle_mode = (self.create_world_focus == 3 && (enter_just || south_just)) || click_elem == Some(3);

                if click_elem.is_none() && (do_cancel || do_create || do_rand_seed || do_toggle_mode) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                }

                if do_rand_seed {
                    self.create_world_seed = format!("{}", generate_random_seed());
                    self.create_world_selected = false;
                    crate::log_debug!("MENU", "Generated random seed for new world: {}", self.create_world_seed);
                } else if do_toggle_mode {
                    self.create_world_gamemode = match self.create_world_gamemode {
                        GameMode::Survival => GameMode::Creative,
                        GameMode::Creative => GameMode::Survival,
                    };
                    crate::log_debug!("MENU", "Toggled world creation mode to {:?}", self.create_world_gamemode);
                } else if do_cancel {
                    self.state = GameState::SaveSelect;
                    crate::log_info!("MENU", "Cancelled world creation, returning to Save Selection");
                } else if do_create {
                    let seed = if self.create_world_seed.trim().is_empty() {
                        generate_random_seed()
                    } else if let Ok(s) = self.create_world_seed.trim().parse::<u32>() {
                        s
                    } else {
                        let mut s = 0u32;
                        for b in self.create_world_seed.trim().bytes() {
                            s = s.wrapping_mul(31).wrapping_add(b as u32);
                        }
                        if s == 0 { generate_random_seed() } else { s }
                    };

                    let name = if self.create_world_name.trim().is_empty() {
                        format!("New World {:02}", self.create_world_slot)
                    } else {
                        self.create_world_name.trim().to_string()
                    };

                    let gamemode = self.create_world_gamemode;
                    self.save_slot = self.create_world_slot;
                    self.seed = seed;
                    self.player.reset(gamemode);
                    crate::log_info!("MENU", "Creating new world '{}' (seed: {}, mode: {:?}) in slot {}", name, seed, gamemode, self.save_slot);

                    #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
                    {
                        let (tx, rx) = mpsc::channel();
                        let thread_seed = self.seed;
                        let thread_name = name;
                        let thread_gamemode = gamemode;
                        thread::spawn(move || {
                            let mut new_world = World::new_with_seed(thread_seed);
                            new_world.name = thread_name;
                            new_world.gamemode = thread_gamemode;
                            let _ = tx.send(WorldLoadResult::Success(new_world, None));
                        });
                        self.state = GameState::Loading(rx);
                    }
                    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
                    {
                        self.world = World::new_with_seed(self.seed);
                        self.world.name = name;
                        self.world.gamemode = gamemode;
                        self.camera = Camera::new(self.world.player_position);
                        self.mob_manager = MobManager::new();
                        self.state = GameState::Playing;
                    }
                }

                if matches!(self.state, GameState::CreateWorld) {
                    self.renderer.draw_create_world_menu(
                        self.create_world_slot,
                        &self.create_world_name,
                        &self.create_world_seed,
                        self.create_world_gamemode,
                        self.create_world_focus,
                        hovered_elem,
                        self.total_time,
                        self.create_world_selected,
                        self.is_gamepad_active,
                    );
                }

    }

    pub(crate) fn update_and_render_settings(
        &mut self,
        input: &InputState,
        prev: PreviousMenu,
        mouse_moved: bool,
        _out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let escape_down = input.key_escape;
        let p_down = input.key_p;
        let enter_down = input.key_enter;
        let tab_down = input.key_tab;
        let left_arrow_down = input.key_left;
        let right_arrow_down = input.key_right;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                let (hovered_tab, mouse_hovered_opt) = if self.is_gamepad_active && !mouse_moved {
                    (None, None)
                } else if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;

                    let tab = self.settings_tab_rects.iter().enumerate().find_map(|(t_idx, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(t_idx)
                        } else {
                            None
                        }
                    });

                    let opt = if tab.is_none() {
                        let rects: &[(i32, i32, i32, i32)] = match self.settings_tab {
                            0 => &self.settings_rects,
                            1 => &self.settings_audio_rects,
                            2 => &self.settings_controls_rects,
                            _ => &self.settings_debug_rects,
                        };
                        rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
                            if px >= bx && px < bx + bw && py >= by && py < by + bh {
                                Some(i)
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    };

                    (tab, opt)
                } else {
                    (None, None)
                };

                let mut hovered_opt = self.settings_hovered_opt;
                if let Some(opt) = mouse_hovered_opt {
                    hovered_opt = Some(opt);
                    self.settings_hovered_opt = Some(opt);
                } else if !self.is_gamepad_active && mouse_pos.is_some() && hovered_tab.is_none() {
                    hovered_opt = None;
                }

                // Tab switching via keyboard (Tab / 1 / 2 / 3 / 4) or Gamepad LB / RB
                let tab_key_just = (tab_down && !self.prev_tab_down)
                    || (input.key_digits[1] && !self.prev_key_digits[1])
                    || (input.key_digits[2] && !self.prev_key_digits[2])
                    || (input.key_digits[3] && !self.prev_key_digits[3])
                    || (input.key_digits[4] && !self.prev_key_digits[4]);
                let gp_l1_just = input.gp_btn_lb && !self.prev_l1_btn;
                let gp_r1_just = input.gp_btn_rb && !self.prev_r1_btn;
                self.prev_l1_btn = input.gp_btn_lb;
                self.prev_r1_btn = input.gp_btn_rb;

                let prev_tab = self.settings_tab;
                if input.key_digits[1] && !self.prev_key_digits[1] {
                    self.settings_tab = 0;
                    self.settings_hovered_opt = Some(0);
                    hovered_opt = Some(0);
                } else if input.key_digits[2] && !self.prev_key_digits[2] {
                    self.settings_tab = 1;
                    self.settings_hovered_opt = Some(0);
                    hovered_opt = Some(0);
                } else if input.key_digits[3] && !self.prev_key_digits[3] {
                    self.settings_tab = 2;
                    self.settings_hovered_opt = Some(0);
                    hovered_opt = Some(0);
                } else if input.key_digits[4] && !self.prev_key_digits[4] {
                    self.settings_tab = 3;
                    self.settings_hovered_opt = Some(0);
                    hovered_opt = Some(0);
                } else if tab_key_just || gp_r1_just {
                    self.settings_tab = (self.settings_tab + 1) % 4;
                    self.settings_hovered_opt = Some(0);
                    hovered_opt = Some(0);
                } else if gp_l1_just {
                    self.settings_tab = (self.settings_tab + 3) % 4;
                    self.settings_hovered_opt = Some(0);
                    hovered_opt = Some(0);
                }

                let max_opts = match self.settings_tab {
                    0 => 13,
                    1 => 7,
                    2 => 8,
                    _ => 5,
                };

                if input.gp_dpad_up_just_pressed || input.gp_menu_up || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.settings_hovered_opt {
                        Some(0) | None => max_opts - 1,
                        Some(i) => (i + max_opts - 1) % max_opts,
                    };
                    self.settings_hovered_opt = Some(next);
                    hovered_opt = Some(next);
                }
                if input.gp_dpad_down_just_pressed || input.gp_menu_down || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.settings_hovered_opt {
                        Some(i) => (i + 1) % max_opts,
                        None => 0,
                    };
                    self.settings_hovered_opt = Some(next);
                    hovered_opt = Some(next);
                }

                let left_click_just = left_down && !self.prev_left_down;
                if let Some(t_idx) = hovered_tab {
                    if left_click_just {
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        self.settings_tab = t_idx;
                        self.settings_hovered_opt = Some(0);
                        hovered_opt = Some(0);
                    }
                }
                if self.settings_tab != prev_tab {
                    crate::log_debug!("SETTINGS", "Switched to tab {} ({})", self.settings_tab, match self.settings_tab { 0 => "Graphics", 1 => "Audio", 2 => "Controls", _ => "Debug" });
                }

                let click_opt = if (left_click_just && hovered_tab.is_none()) || input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
                    hovered_opt
                } else {
                    None
                };

                let click_direction_forward = if left_click_just {
                    if let (Some((mx, _)), Some(opt_idx)) = (mouse_pos, hovered_opt) {
                        let rect = match self.settings_tab {
                            0 => self.settings_rects.get(opt_idx),
                            1 => self.settings_audio_rects.get(opt_idx),
                            2 => self.settings_controls_rects.get(opt_idx),
                            _ => self.settings_debug_rects.get(opt_idx),
                        };
                        if let Some(&(bx, _, bw, _)) = rect {
                            mx >= (bx as f32 + bw as f32 / 2.0)
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

                let left_key_just = (left_arrow_down && !self.prev_left_arrow_down) || input.gp_menu_left;
                let right_key_just = (right_arrow_down && !self.prev_right_arrow_down) || input.gp_menu_right;

                if self.settings_tab == 0 {
                    // GRAPHICS TAB
                    if let Some(opt_idx) = click_opt {
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        match opt_idx {
                            0 => {
                                self.settings.cycle_language(click_direction_forward);
                                crate::core::i18n::set_language(self.settings.language);
                                crate::log_info!("SETTINGS", "Language changed to {:?}", self.settings.language);
                            }
                            1 => {
                                self.settings.cycle_render_distance(click_direction_forward);
                                self.renderer.render_distance = self.settings.render_distance;
                                self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                                self.renderer.clear_mesh_cache();
                                crate::log_info!("SETTINGS", "Render distance changed to {} chunks", self.settings.render_distance);
                            }
                            2 => {
                                self.settings.cycle_leaf_quality(click_direction_forward);
                                self.renderer.update_textures(self.settings.fancy_leaves);
                                self.world.mark_all_chunks_dirty();
                                self.renderer.clear_mesh_cache();
                                crate::log_info!("SETTINGS", "Fancy leaves changed to {}", self.settings.fancy_leaves);
                            }
                            3 => {
                                self.settings.fog = !self.settings.fog;
                                crate::log_info!("SETTINGS", "Fog toggled to {}", self.settings.fog);
                            }
                            4 => {
                                self.settings.clouds = !self.settings.clouds;
                                crate::log_info!("SETTINGS", "Clouds toggled to {}", self.settings.clouds);
                            }
                            5 => {
                                self.settings.particles = !self.settings.particles;
                                crate::log_info!("SETTINGS", "Particles toggled to {}", self.settings.particles);
                            }
                            6 => {
                                self.settings.chunk_anim = !self.settings.chunk_anim;
                                crate::log_info!("SETTINGS", "Chunk animation toggled to {}", self.settings.chunk_anim);
                            }
                            7 => {
                                self.settings.cycle_fov(click_direction_forward);
                                crate::log_info!("SETTINGS", "FOV changed to {} deg", self.settings.fov_deg);
                            }
                            8 => {
                                self.settings.cycle_brightness(click_direction_forward);
                                self.renderer.brightness = self.settings.brightness;
                                crate::log_info!("SETTINGS", "Brightness changed to {:.2}", self.settings.brightness);
                            }
                            9 => {
                                self.settings.cycle_msaa(click_direction_forward);
                                crate::log_info!("SETTINGS", "MSAA changed to {}", self.settings.msaa);
                            }
                            10 => {
                                self.settings.cycle_fps_limit(click_direction_forward);
                                crate::log_info!("SETTINGS", "FPS limit changed to {}", self.settings.fps_limit);
                            }
                            11 => {
                                self.settings.underwater_fx = !self.settings.underwater_fx;
                                crate::log_info!("SETTINGS", "Underwater FX toggled to {}", self.settings.underwater_fx);
                            }
                            12 => {
                                let _ = self.settings.save();
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                            _ => {}
                        }
                    } else if input.gp_dpad_left_just_pressed || left_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            match opt_idx {
                                0 => {
                                    self.settings.cycle_language(false);
                                    crate::core::i18n::set_language(self.settings.language);
                                    crate::log_info!("SETTINGS", "Language changed to {:?}", self.settings.language);
                                }
                                1 => {
                                    self.settings.cycle_render_distance(false);
                                    self.renderer.render_distance = self.settings.render_distance;
                                    self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                                    self.renderer.clear_mesh_cache();
                                    crate::log_info!("SETTINGS", "Render distance changed to {} chunks", self.settings.render_distance);
                                }
                                2 => {
                                    self.settings.cycle_leaf_quality(false);
                                    self.renderer.update_textures(self.settings.fancy_leaves);
                                    self.world.mark_all_chunks_dirty();
                                    self.renderer.clear_mesh_cache();
                                    crate::log_info!("SETTINGS", "Fancy leaves changed to {}", self.settings.fancy_leaves);
                                }
                                3 => {
                                    self.settings.fog = !self.settings.fog;
                                    crate::log_info!("SETTINGS", "Fog toggled to {}", self.settings.fog);
                                }
                                4 => {
                                    self.settings.clouds = !self.settings.clouds;
                                    crate::log_info!("SETTINGS", "Clouds toggled to {}", self.settings.clouds);
                                }
                                5 => {
                                    self.settings.particles = !self.settings.particles;
                                    crate::log_info!("SETTINGS", "Particles toggled to {}", self.settings.particles);
                                }
                                6 => {
                                    self.settings.chunk_anim = !self.settings.chunk_anim;
                                    crate::log_info!("SETTINGS", "Chunk animation toggled to {}", self.settings.chunk_anim);
                                }
                                7 => {
                                    self.settings.cycle_fov(false);
                                    crate::log_info!("SETTINGS", "FOV changed to {} deg", self.settings.fov_deg);
                                }
                                8 => {
                                    self.settings.cycle_brightness(false);
                                    self.renderer.brightness = self.settings.brightness;
                                    crate::log_info!("SETTINGS", "Brightness changed to {:.2}", self.settings.brightness);
                                }
                                9 => {
                                    self.settings.cycle_msaa(false);
                                    crate::log_info!("SETTINGS", "MSAA changed to {}", self.settings.msaa);
                                }
                                10 => {
                                    self.settings.cycle_fps_limit(false);
                                    crate::log_info!("SETTINGS", "FPS limit changed to {}", self.settings.fps_limit);
                                }
                                11 => {
                                    self.settings.underwater_fx = !self.settings.underwater_fx;
                                    crate::log_info!("SETTINGS", "Underwater FX toggled to {}", self.settings.underwater_fx);
                                }
                                _ => {}
                            }
                        }
                    } else if input.gp_dpad_right_just_pressed || right_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            match opt_idx {
                                0 => {
                                    self.settings.cycle_language(true);
                                    crate::core::i18n::set_language(self.settings.language);
                                    crate::log_info!("SETTINGS", "Language changed to {:?}", self.settings.language);
                                }
                                1 => {
                                    self.settings.cycle_render_distance(true);
                                    self.renderer.render_distance = self.settings.render_distance;
                                    self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                                    self.renderer.clear_mesh_cache();
                                    crate::log_info!("SETTINGS", "Render distance changed to {} chunks", self.settings.render_distance);
                                }
                                2 => {
                                    self.settings.cycle_leaf_quality(true);
                                    self.renderer.update_textures(self.settings.fancy_leaves);
                                    self.world.mark_all_chunks_dirty();
                                    self.renderer.clear_mesh_cache();
                                    crate::log_info!("SETTINGS", "Fancy leaves changed to {}", self.settings.fancy_leaves);
                                }
                                3 => {
                                    self.settings.fog = !self.settings.fog;
                                    crate::log_info!("SETTINGS", "Fog toggled to {}", self.settings.fog);
                                }
                                4 => {
                                    self.settings.clouds = !self.settings.clouds;
                                    crate::log_info!("SETTINGS", "Clouds toggled to {}", self.settings.clouds);
                                }
                                5 => {
                                    self.settings.particles = !self.settings.particles;
                                    crate::log_info!("SETTINGS", "Particles toggled to {}", self.settings.particles);
                                }
                                6 => {
                                    self.settings.chunk_anim = !self.settings.chunk_anim;
                                    crate::log_info!("SETTINGS", "Chunk animation toggled to {}", self.settings.chunk_anim);
                                }
                                7 => {
                                    self.settings.cycle_fov(true);
                                    crate::log_info!("SETTINGS", "FOV changed to {} deg", self.settings.fov_deg);
                                }
                                8 => {
                                    self.settings.cycle_brightness(true);
                                    self.renderer.brightness = self.settings.brightness;
                                    crate::log_info!("SETTINGS", "Brightness changed to {:.2}", self.settings.brightness);
                                }
                                9 => {
                                    self.settings.cycle_msaa(true);
                                    crate::log_info!("SETTINGS", "MSAA changed to {}", self.settings.msaa);
                                }
                                10 => {
                                    self.settings.cycle_fps_limit(true);
                                    crate::log_info!("SETTINGS", "FPS limit changed to {}", self.settings.fps_limit);
                                }
                                11 => {
                                    self.settings.underwater_fx = !self.settings.underwater_fx;
                                    crate::log_info!("SETTINGS", "Underwater FX toggled to {}", self.settings.underwater_fx);
                                }
                                _ => {}
                            }
                        }
                    }
                } else if self.settings_tab == 1 {
                    // AUDIO TAB
                    if let Some(opt_idx) = click_opt {
                        match opt_idx {
                            0 => {
                                self.settings.cycle_master_volume(click_direction_forward);
                                self.audio.set_master_volume(self.settings.master_volume);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "Master volume changed to {:.0}%", self.settings.master_volume * 100.0);
                            }
                            1 => {
                                self.settings.cycle_sfx_volume(click_direction_forward);
                                self.audio.set_sfx_volume(self.settings.sfx_volume);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "SFX volume changed to {:.0}%", self.settings.sfx_volume * 100.0);
                            }
                            2 => {
                                self.settings.cycle_music_volume(click_direction_forward);
                                self.audio.set_music_volume(self.settings.music_volume);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "Music volume changed to {:.0}%", self.settings.music_volume * 100.0);
                            }
                            3 => {
                                self.settings.cycle_ambient_volume(click_direction_forward);
                                self.audio.set_ambient_volume(self.settings.ambient_volume);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "Ambient volume changed to {:.0}%", self.settings.ambient_volume * 100.0);
                            }
                            4 => {
                                self.settings.cycle_ui_volume(click_direction_forward);
                                self.audio.set_ui_volume(self.settings.ui_volume);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "UI volume changed to {:.0}%", self.settings.ui_volume * 100.0);
                            }
                            5 => {
                                self.settings.toggle_mute_audio();
                                self.audio.set_muted(self.settings.mute_audio);
                                if !self.settings.mute_audio {
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                }
                                crate::log_info!("SETTINGS", "Mute audio changed to {}", self.settings.mute_audio);
                            }
                            6 => {
                                let _ = self.settings.save();
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                            _ => {}
                        }
                    } else if input.gp_dpad_left_just_pressed || left_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            match opt_idx {
                                0 => {
                                    self.settings.cycle_master_volume(false);
                                    self.audio.set_master_volume(self.settings.master_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "Master volume changed to {:.0}%", self.settings.master_volume * 100.0);
                                }
                                1 => {
                                    self.settings.cycle_sfx_volume(false);
                                    self.audio.set_sfx_volume(self.settings.sfx_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "SFX volume changed to {:.0}%", self.settings.sfx_volume * 100.0);
                                }
                                2 => {
                                    self.settings.cycle_music_volume(false);
                                    self.audio.set_music_volume(self.settings.music_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "Music volume changed to {:.0}%", self.settings.music_volume * 100.0);
                                }
                                3 => {
                                    self.settings.cycle_ambient_volume(false);
                                    self.audio.set_ambient_volume(self.settings.ambient_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "Ambient volume changed to {:.0}%", self.settings.ambient_volume * 100.0);
                                }
                                4 => {
                                    self.settings.cycle_ui_volume(false);
                                    self.audio.set_ui_volume(self.settings.ui_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "UI volume changed to {:.0}%", self.settings.ui_volume * 100.0);
                                }
                                5 => {
                                    self.settings.toggle_mute_audio();
                                    self.audio.set_muted(self.settings.mute_audio);
                                    if !self.settings.mute_audio {
                                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    }
                                    crate::log_info!("SETTINGS", "Mute audio changed to {}", self.settings.mute_audio);
                                }
                                _ => {}
                            }
                        }
                    } else if input.gp_dpad_right_just_pressed || right_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            match opt_idx {
                                0 => {
                                    self.settings.cycle_master_volume(true);
                                    self.audio.set_master_volume(self.settings.master_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "Master volume changed to {:.0}%", self.settings.master_volume * 100.0);
                                }
                                1 => {
                                    self.settings.cycle_sfx_volume(true);
                                    self.audio.set_sfx_volume(self.settings.sfx_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "SFX volume changed to {:.0}%", self.settings.sfx_volume * 100.0);
                                }
                                2 => {
                                    self.settings.cycle_music_volume(true);
                                    self.audio.set_music_volume(self.settings.music_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "Music volume changed to {:.0}%", self.settings.music_volume * 100.0);
                                }
                                3 => {
                                    self.settings.cycle_ambient_volume(true);
                                    self.audio.set_ambient_volume(self.settings.ambient_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "Ambient volume changed to {:.0}%", self.settings.ambient_volume * 100.0);
                                }
                                4 => {
                                    self.settings.cycle_ui_volume(true);
                                    self.audio.set_ui_volume(self.settings.ui_volume);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    crate::log_info!("SETTINGS", "UI volume changed to {:.0}%", self.settings.ui_volume * 100.0);
                                }
                                5 => {
                                    self.settings.toggle_mute_audio();
                                    self.audio.set_muted(self.settings.mute_audio);
                                    if !self.settings.mute_audio {
                                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                    }
                                    crate::log_info!("SETTINGS", "Mute audio changed to {}", self.settings.mute_audio);
                                }
                                _ => {}
                            }
                        }
                    }
                } else if self.settings_tab == 2 {
                    // CONTROLS TAB
                    if let Some(opt_idx) = click_opt {
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        match opt_idx {
                            0 => {
                                self.settings.cycle_sprint_mode(click_direction_forward);
                                crate::log_info!("SETTINGS", "Sprint mode changed to {:?}", self.settings.sprint_mode);
                            }
                            1 => {
                                self.settings.cycle_sneak_mode(click_direction_forward);
                                crate::log_info!("SETTINGS", "Sneak mode changed to {:?}", self.settings.sneak_mode);
                            }
                            2 => {
                                self.settings.cycle_jump_mode(click_direction_forward);
                                crate::log_info!("SETTINGS", "Jump mode changed to {:?}", self.settings.jump_mode);
                            }
                            3 => {
                                self.settings.toggle_invert_y();
                                crate::log_info!("SETTINGS", "Invert Y changed to {}", self.settings.invert_y);
                            }
                            4 => {
                                self.settings.toggle_invert_z();
                                crate::log_info!("SETTINGS", "Invert Z changed to {}", self.settings.invert_z);
                            }
                            5 => {
                                self.settings.toggle_vibration();
                                if self.settings.vibration {
                                    self.request_rumble(0.5, 0.5, 120);
                                }
                                crate::log_info!("SETTINGS", "Gamepad vibration changed to {}", self.settings.vibration);
                            }
                            6 => {
                                self.settings.toggle_damage_tilt();
                                crate::log_info!("SETTINGS", "Damage tilt changed to {}", self.settings.damage_tilt);
                            }
                            7 => {
                                let _ = self.settings.save();
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                            _ => {}
                        }
                    } else if input.gp_dpad_left_just_pressed || left_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            match opt_idx {
                                0 => {
                                    self.settings.cycle_sprint_mode(false);
                                    crate::log_info!("SETTINGS", "Sprint mode changed to {:?}", self.settings.sprint_mode);
                                }
                                1 => {
                                    self.settings.cycle_sneak_mode(false);
                                    crate::log_info!("SETTINGS", "Sneak mode changed to {:?}", self.settings.sneak_mode);
                                }
                                2 => {
                                    self.settings.cycle_jump_mode(false);
                                    crate::log_info!("SETTINGS", "Jump mode changed to {:?}", self.settings.jump_mode);
                                }
                                3 => {
                                    self.settings.toggle_invert_y();
                                    crate::log_info!("SETTINGS", "Invert Y changed to {}", self.settings.invert_y);
                                }
                                4 => {
                                    self.settings.toggle_invert_z();
                                    crate::log_info!("SETTINGS", "Invert Z changed to {}", self.settings.invert_z);
                                }
                                5 => {
                                    self.settings.toggle_vibration();
                                    if self.settings.vibration {
                                        self.request_rumble(0.5, 0.5, 120);
                                    }
                                    crate::log_info!("SETTINGS", "Gamepad vibration changed to {}", self.settings.vibration);
                                }
                                6 => {
                                    self.settings.toggle_damage_tilt();
                                    crate::log_info!("SETTINGS", "Damage tilt changed to {}", self.settings.damage_tilt);
                                }
                                _ => {}
                            }
                        }
                    } else if input.gp_dpad_right_just_pressed || right_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            match opt_idx {
                                0 => {
                                    self.settings.cycle_sprint_mode(true);
                                    crate::log_info!("SETTINGS", "Sprint mode changed to {:?}", self.settings.sprint_mode);
                                }
                                1 => {
                                    self.settings.cycle_sneak_mode(true);
                                    crate::log_info!("SETTINGS", "Sneak mode changed to {:?}", self.settings.sneak_mode);
                                }
                                2 => {
                                    self.settings.cycle_jump_mode(true);
                                    crate::log_info!("SETTINGS", "Jump mode changed to {:?}", self.settings.jump_mode);
                                }
                                3 => {
                                    self.settings.toggle_invert_y();
                                    crate::log_info!("SETTINGS", "Invert Y changed to {}", self.settings.invert_y);
                                }
                                4 => {
                                    self.settings.toggle_invert_z();
                                    crate::log_info!("SETTINGS", "Invert Z changed to {}", self.settings.invert_z);
                                }
                                5 => {
                                    self.settings.toggle_vibration();
                                    if self.settings.vibration {
                                        self.request_rumble(0.5, 0.5, 120);
                                    }
                                    crate::log_info!("SETTINGS", "Gamepad vibration changed to {}", self.settings.vibration);
                                }
                                6 => {
                                    self.settings.toggle_damage_tilt();
                                    crate::log_info!("SETTINGS", "Damage tilt changed to {}", self.settings.damage_tilt);
                                }
                                _ => {}
                            }
                        }
                    }
                } else {
                    // DEBUG TAB
                    if let Some(opt_idx) = click_opt {
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        match opt_idx {
                            0 => {
                                self.settings.toggle_profiler_server();
                                crate::log_info!("SETTINGS", "Profiler server toggled: {}", self.settings.profiler_server);
                                if self.settings.profiler_server {
                                    self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                                } else if let Some(mut s) = self.profiler_server.take() {
                                    s.stop();
                                }
                            }
                            1 => {
                                if self.settings.profiler_server {
                                    let server_ip = self.profiler_server.as_ref().map(|s| s.ip_address.as_str()).unwrap_or("127.0.0.1");
                                    let url = format!("http://{}:{}", server_ip, self.settings.profiler_server_port);
                                    crate::ui::clipboard::set_clipboard(&url);
                                    self.profiler_copied_timer = 2.5;
                                    crate::log_info!("SETTINGS", "Copied profiler server URL: {}", url);
                                }
                            }
                            2 => {
                                self.settings.cycle_profiler_server_port(click_direction_forward);
                                crate::log_info!("SETTINGS", "Profiler server port changed to {}", self.settings.profiler_server_port);
                                if self.settings.profiler_server {
                                    if let Some(mut s) = self.profiler_server.take() {
                                        s.stop();
                                    }
                                    self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                                }
                            }
                            3 => {
                                self.settings.debug_overlay = !self.settings.debug_overlay;
                                crate::log_info!("SETTINGS", "Debug overlay toggled to {}", self.settings.debug_overlay);
                            }
                            4 => {
                                let _ = self.settings.save();
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                            _ => {}
                        }
                    } else if input.gp_dpad_left_just_pressed || left_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            match opt_idx {
                                0 => {
                                    self.settings.toggle_profiler_server();
                                    crate::log_info!("SETTINGS", "Profiler server toggled: {}", self.settings.profiler_server);
                                    if self.settings.profiler_server {
                                        self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                                    } else if let Some(mut s) = self.profiler_server.take() {
                                        s.stop();
                                    }
                                }
                                2 => {
                                    self.settings.cycle_profiler_server_port(false);
                                    crate::log_info!("SETTINGS", "Profiler server port changed to {}", self.settings.profiler_server_port);
                                    if self.settings.profiler_server {
                                        if let Some(mut s) = self.profiler_server.take() {
                                            s.stop();
                                        }
                                        self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                                    }
                                }
                                3 => {
                                    self.settings.debug_overlay = !self.settings.debug_overlay;
                                    crate::log_info!("SETTINGS", "Debug overlay toggled to {}", self.settings.debug_overlay);
                                }
                                _ => {}
                            }
                        }
                    } else if input.gp_dpad_right_just_pressed || right_key_just {
                        if let Some(opt_idx) = hovered_opt {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            match opt_idx {
                                0 => {
                                    self.settings.toggle_profiler_server();
                                    crate::log_info!("SETTINGS", "Profiler server toggled: {}", self.settings.profiler_server);
                                    if self.settings.profiler_server {
                                        self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                                    } else if let Some(mut s) = self.profiler_server.take() {
                                        s.stop();
                                    }
                                }
                                2 => {
                                    self.settings.cycle_profiler_server_port(true);
                                    crate::log_info!("SETTINGS", "Profiler server port changed to {}", self.settings.profiler_server_port);
                                    if self.settings.profiler_server {
                                        if let Some(mut s) = self.profiler_server.take() {
                                            s.stop();
                                        }
                                        self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                                    }
                                }
                                3 => {
                                    self.settings.debug_overlay = !self.settings.debug_overlay;
                                    crate::log_info!("SETTINGS", "Debug overlay toggled to {}", self.settings.debug_overlay);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if (escape_down && !self.prev_escape_down) || (p_down && !self.prev_p_down) || input.gp_east_just_pressed {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    let _ = self.settings.save();
                    crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                    self.state = match prev {
                        PreviousMenu::MainMenu => GameState::MainMenu,
                        PreviousMenu::Paused => GameState::Paused,
                        _ => GameState::MainMenu,
                    };
                }

                self.renderer.fog_enabled = self.settings.fog;
                if prev == PreviousMenu::Paused {
                    #[cfg(not(target_os = "vita"))]
                    self.renderer.render_frame(&mut self.world, &self.camera);
                    #[cfg(target_os = "vita")]
                    {
                        self.renderer.color_buffer.fill(0x0C0C10);
                    }
                }
                let server_ip = self.profiler_server.as_ref().map(|s| s.ip_address.as_str()).unwrap_or("127.0.0.1");
                self.renderer.draw_settings_menu(&self.settings, self.settings_tab, hovered_tab, hovered_opt, server_ip, self.profiler_copied_timer > 0.0, mouse_pos, self.is_gamepad_active);

    }

    #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
    pub(crate) fn update_and_render_loading(&mut self, result: Option<WorldLoadResult>) {
        self.renderer.draw_main_menu(None, &self.settings.nickname, false, self.is_gamepad_active);

        if let Some(result) = result {
            match result {
                WorldLoadResult::Success(loaded_world, loaded_pos) => {
                    self.player.gamemode = loaded_world.gamemode;
                    if let Some(ref saved) = loaded_world.saved_player {
                        self.player.apply_saved_state(saved);
                    } else {
                        self.player.reset(loaded_world.gamemode);
                    }
                    self.world = loaded_world;
                    let start_pos = loaded_pos.unwrap_or(self.world.player_position);
                    self.camera = Camera::new(start_pos);
                    self.mob_manager = MobManager::new();
                    self.minimap.waypoints = crate::ui::minimap::load_waypoints(self.save_slot);
                    self.state = GameState::Playing;
                    crate::log_info!("STATE", "Entered world '{}' (seed: {}, mode: {:?}) at position ({:.1}, {:.1}, {:.1})", self.world.name, self.world.seed, self.player.gamemode, start_pos.x, start_pos.y, start_pos.z);
                }
                WorldLoadResult::Error(e) => {
                    eprintln!("Failed to load world: {e}");
                    crate::log_error!("STATE", "Failed to load world: {}, returning to Main Menu", e);
                    self.state = GameState::MainMenu;
                }
            }
        }
    }

    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
    pub(crate) fn update_and_render_loading(&mut self) {
        self.renderer.draw_main_menu(None, &self.settings.nickname, false, self.is_gamepad_active);
        self.state = GameState::Playing;
    }

    pub(crate) fn update_and_render_paused(
        &mut self,
        dt: f32,
        input: &InputState,
        mouse_moved: bool,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let q_down = input.key_q;
        let m_down = input.key_m;
        let enter_down = input.key_enter;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                let t_fov = 1.0 - (-8.0 * dt).exp();
                self.camera.fov_y += (self.settings.fov_deg.to_radians() - self.camera.fov_y) * t_fov;

                let mouse_hovered_btn = if self.is_gamepad_active && !mouse_moved {
                    None
                } else if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    self.pause_menu_rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(i)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let mut hovered_idx = self.pause_hovered_btn;
                if let Some(btn) = mouse_hovered_btn {
                    hovered_idx = Some(btn);
                    self.pause_hovered_btn = Some(btn);
                } else if !self.is_gamepad_active && mouse_pos.is_some() {
                    hovered_idx = None;
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.pause_hovered_btn {
                        Some(0) | None => 4,
                        Some(i) => i - 1,
                    };
                    self.pause_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.pause_hovered_btn {
                        Some(4) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.pause_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_pause_btn = if left_down && !self.prev_left_down {
                    mouse_hovered_btn
                } else if input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
                    hovered_idx
                } else {
                    None
                };

                if let Some(btn) = click_pause_btn {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    match btn {
                        0 => {
                            self.state = GameState::Playing;
                            out.request_capture_mouse = true;
                            crate::log_info!("STATE", "Game resumed from pause menu");
                        }
                        1 => {
                            self.state = GameState::Settings(PreviousMenu::Paused);
                            self.settings_hovered_opt = Some(0);
                            crate::log_info!("MENU", "Opened Settings from pause menu");
                        }
                        2 => {
                            self.fly_mode = !self.fly_mode;
                            crate::log_info!("PLAYER", "Flight mode toggled to: {}", self.fly_mode);
                        }
                        3 => {
                            if self.is_multiplayer {
                                if let Some(client) = &mut self.net_client {
                                    client.disconnect();
                                }
                                self.net_client = None;
                                self.is_multiplayer = false;
                            } else {
                                let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                                crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                                self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                            }
                            self.state = GameState::MainMenu;
                            out.request_release_mouse = true;
                            crate::log_info!("STATE", "Returned to Main Menu from pause menu");
                        }
                        4 => {
                            if self.is_multiplayer {
                                if let Some(client) = &mut self.net_client {
                                    client.disconnect();
                                }
                                self.net_client = None;
                                self.is_multiplayer = false;
                            } else {
                                let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                                crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                            }
                            out.quit_requested = true;
                            crate::log_info!("STATE", "Saved and quitting game from pause menu");
                        }
                        _ => {}
                    }
                }

                if self.controls.is_just_pressed(Action::Pause) || input.gp_east_just_pressed {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    self.state = GameState::Playing;
                    out.request_capture_mouse = true;
                    crate::log_info!("STATE", "Game resumed via pause shortcut");
                }
                if self.controls.is_just_pressed(Action::ToggleFly) {
                    self.fly_mode = !self.fly_mode;
                    crate::log_info!("PLAYER", "Flight mode toggled to: {}", self.fly_mode);
                }
                if q_down && !self.prev_q_down {
                    let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                    crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                    out.quit_requested = true;
                    crate::log_info!("STATE", "Saved and quitting game via shortcut");
                }
                if m_down && !self.prev_m_down {
                    let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                    crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                    self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    self.state = GameState::MainMenu;
                    out.request_release_mouse = true;
                    crate::log_info!("STATE", "Saved and returned to Main Menu via shortcut");
                }

                self.renderer.fog_enabled = self.settings.fog;
                #[cfg(not(target_os = "vita"))]
                self.renderer.render_frame(&mut self.world, &self.camera);
                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0x0C0C10);
                }
                self.renderer.draw_pause_menu(self.fly_mode, hovered_idx, self.is_gamepad_active);

                if let Some(ref server) = self.profiler_server {
                    let mut cur_profiler = FrameProfiler::default();
                    cur_profiler.fps = (1.0 / dt.max(0.001)).round() as u32;
                    let snap = self.build_profiler_snapshot(&cur_profiler);
                    server.update_snapshot(snap);
                }

    }

    pub(crate) fn update_and_render_death_screen(
        &mut self,
        input: &InputState,
        mouse_moved: bool,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let escape_down = input.key_escape;
        let p_down = input.key_p;
        let enter_down = input.key_enter;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;
                let mouse_hovered_btn = if self.is_gamepad_active && !mouse_moved {
                    None
                } else if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    self.death_screen_rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            Some(i)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                };

                let mut hovered_idx = self.death_hovered_btn;
                if let Some(btn) = mouse_hovered_btn {
                    hovered_idx = Some(btn);
                    self.death_hovered_btn = Some(btn);
                } else if !self.is_gamepad_active && mouse_pos.is_some() {
                    hovered_idx = None;
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.death_hovered_btn {
                        Some(0) | None => 1,
                        Some(i) => i - 1,
                    };
                    self.death_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.death_hovered_btn {
                        Some(1) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.death_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_death_btn = if left_down && !self.prev_left_down {
                    mouse_hovered_btn
                } else if input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
                    hovered_idx
                } else {
                    None
                };

                if let Some(btn) = click_death_btn {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    match btn {
                        0 => {
                            self.player.health = self.player.max_health;
                            self.player.hunger = self.player.max_hunger;
                            self.player.saturation = 5.0;
                            self.player.exhaustion = 0.0;
                            self.player.starvation_timer = 0.0;
                            self.player.heal_timer = 0.0;
                            self.player.air = self.player.max_air;
                            self.player.head_underwater = false;
                            self.drown_damage_timer = 0.0;
                            let spawn_pos = if let Some(sp) = self.player.spawn_point {
                                Vec3::new(sp.x as f32 + 0.5, sp.y as f32 + 1.0, sp.z as f32 + 0.5)
                            } else {
                                self.world.player_position
                            };
                            self.camera.position = spawn_pos;
                            self.camera.pitch = 0.0;
                            self.vertical_velocity = 0.0;
                            self.horiz_velocity = Vec3::zero();
                            self.player.fall_distance = 0.0;
                            self.player.invulnerability_timer = 3.0;
                            self.player.has_landed = false;
                            self.damage_tilt_timer = 0.0;
                            crate::log_info!("PLAYER", "Player respawned at ({:.1}, {:.1}, {:.1})", self.camera.position.x, self.camera.position.y, self.camera.position.z);
                            self.state = GameState::Playing;
                            out.request_capture_mouse = true;
                        }
                        1 => {
                            self.player.reset(self.player.gamemode);
                            self.camera.position = self.world.player_position;
                            let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                            crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                            crate::log_info!("STATE", "Player chose Title Screen from Death Screen, game saved");
                            self.state = GameState::MainMenu;
                            out.request_release_mouse = true;
                        }
                        _ => {}
                    }
                }

                if (escape_down && !self.prev_escape_down) || (p_down && !self.prev_p_down) || input.gp_east_just_pressed {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    self.player.reset(self.player.gamemode);
                    self.camera.position = self.world.player_position;
                    let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                    crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                    self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    crate::log_info!("STATE", "Quit to Main Menu from Death Screen via shortcut, game saved");
                    self.state = GameState::MainMenu;
                    out.request_release_mouse = true;
                }

                self.renderer.fog_enabled = self.settings.fog;
                #[cfg(not(target_os = "vita"))]
                self.renderer.render_frame(&mut self.world, &self.camera);
                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0x0C0C10);
                }
                self.renderer.draw_death_screen(hovered_idx);

    }

    pub fn open_edit_nickname(&mut self, prev: PreviousMenu) {
        let current_nick = self.settings.nickname.clone();
        self.nickname_input = current_nick.clone();
        self.nickname_selected = false;
        self.nickname_dialog_hovered_btn = Some(0);
        self.state = GameState::EditNickname(prev);
        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
        crate::log_info!("MENU", "Opened Change Nickname dialogue");
        self.request_ime("Change Nickname", &current_nick);
    }

    /// Requests opening the platform on-screen keyboard (IME) on systems like PS Vita / Android.
    pub fn request_ime(&mut self, title: &str, initial_text: &str) {
        self.ime_requested = Some((title.to_string(), initial_text.to_string()));
        self.is_ime_active = true;
    }

    /// Applies text entered from an IME on-screen keyboard (e.g. from Vita or Android).
    pub fn apply_ime_text(&mut self, text: &str) {
        self.is_ime_active = false;
        self.ime_requested = None;
        let filtered: String = text
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .take(24)
            .collect();
        if !filtered.is_empty() {
            self.settings.nickname = filtered;
            let _ = self.settings.save();
            crate::log_info!("SETTINGS", "Nickname updated via IME to {}", self.settings.nickname);
        }
        if let GameState::EditNickname(prev) = self.state {
            self.state = match prev {
                PreviousMenu::MainMenu => GameState::MainMenu,
                PreviousMenu::Paused => GameState::Paused,
                PreviousMenu::Settings => GameState::Settings(PreviousMenu::MainMenu),
                PreviousMenu::MultiplayerMenu => GameState::MultiplayerMenu,
            };
        }
    }

    /// Cancels active IME dialog without changing state.
    pub fn cancel_ime(&mut self) {
        self.is_ime_active = false;
        self.ime_requested = None;
    }

    /// Returns true when any text input field or chat prompt is actively receiving keystrokes,
    /// triggering the soft keyboard IME on platforms like Android and PS Vita.
    pub fn is_text_input_active(&self) -> bool {
        self.chat.is_open
            || matches!(self.state, GameState::EditNickname(_))
            || (matches!(self.state, GameState::MultiplayerMenu) && (self.multiplayer_focus == 0 || self.multiplayer_focus == 1))
            || (matches!(self.state, GameState::CreateWorld) && (self.create_world_focus == 0 || self.create_world_focus == 1))
            || (matches!(self.state, GameState::WorldMap) && self.worldmap.creating_waypoint)
            || self.is_ime_active
    }

    pub(crate) fn update_and_render_worldmap(
        &mut self,
        dt: f32,
        input: &InputState,
        _mouse_moved: bool,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let left_click = left_down && !self.prev_left_down;
        let right_click = input.mouse_right && !self.prev_right_down;
        let escape_down = input.key_escape && !self.prev_escape_down;
        let m_down = input.key_m && !self.prev_m_down;
        let b_down = input.key_b && !self.prev_b_down;
        let u_down = input.key_u && !self.prev_u_down;
        let c_down = input.key_c && !self.prev_c_down;
        let space_down = input.key_space && !self.prev_space_down;
        let enter_down = input.key_enter && !self.prev_enter_down;
        let backspace_down = input.key_backspace && !self.prev_backspace_down;

        if self.worldmap.creating_waypoint {
            if escape_down {
                self.worldmap.creating_waypoint = false;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
            } else if enter_down {
                if !self.worldmap.waypoint_name_input.trim().is_empty() {
                    let color = crate::ui::minimap::WAYPOINT_COLORS[self.worldmap.waypoint_color_idx].0;
                    self.minimap.waypoints.push(crate::ui::minimap::Waypoint {
                        name: self.worldmap.waypoint_name_input.trim().to_string(),
                        x: self.worldmap.waypoint_coords_input.0,
                        y: self.worldmap.waypoint_coords_input.1,
                        z: self.worldmap.waypoint_coords_input.2,
                        color,
                        enabled: true,
                    });
                    crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.2);
                }
                self.worldmap.creating_waypoint = false;
            } else {
                if backspace_down {
                    self.worldmap.waypoint_name_input.pop();
                }
                for &ch in &input.typed_chars {
                    if !ch.is_control() && self.worldmap.waypoint_name_input.len() < 20 {
                        self.worldmap.waypoint_name_input.push(ch);
                    }
                }
                let dia_w = 320i32;
                let dia_h = 180i32;
                let dx = (self.width as i32 - dia_w) / 2;
                let dy = (self.height as i32 - dia_h) / 2;
                let (save_rect, cancel_rect) = crate::ui::minimap::waypoint_dialog_buttons(dx, dy, dia_w, dia_h);

                if left_click {
                    if let Some((mx, my)) = mouse_pos {
                        let px = mx as i32;
                        let py = my as i32;
                        for (i, _) in crate::ui::minimap::WAYPOINT_COLORS.iter().enumerate() {
                            let sw_x = dx + 70 + (i as i32) * 28;
                            let sw_y = dy + 95;
                            if px >= sw_x && px < sw_x + 20 && py >= sw_y && py < sw_y + 18 {
                                self.worldmap.waypoint_color_idx = i;
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                            }
                        }
                        if px >= save_rect.0 && px < save_rect.0 + save_rect.2 && py >= save_rect.1 && py < save_rect.1 + save_rect.3 {
                            if !self.worldmap.waypoint_name_input.trim().is_empty() {
                                let color = crate::ui::minimap::WAYPOINT_COLORS[self.worldmap.waypoint_color_idx].0;
                                self.minimap.waypoints.push(crate::ui::minimap::Waypoint {
                                    name: self.worldmap.waypoint_name_input.trim().to_string(),
                                    x: self.worldmap.waypoint_coords_input.0,
                                    y: self.worldmap.waypoint_coords_input.1,
                                    z: self.worldmap.waypoint_coords_input.2,
                                    color,
                                    enabled: true,
                                });
                                crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.2);
                            }
                            self.worldmap.creating_waypoint = false;
                        } else if px >= cancel_rect.0 && px < cancel_rect.0 + cancel_rect.2 && py >= cancel_rect.1 && py < cancel_rect.1 + cancel_rect.3 {
                            self.worldmap.creating_waypoint = false;
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
                        }
                    }
                }
            }
        } else if self.worldmap.showing_waypoint_manager {
            if escape_down || u_down {
                self.worldmap.showing_waypoint_manager = false;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
            } else if b_down {
                let coords = if let Some((hx, hy, hz)) = self.worldmap.hovered_world_pos {
                    (hx, hy, hz)
                } else {
                    (self.camera.position.x.round() as i32, self.camera.position.y.round() as i32, self.camera.position.z.round() as i32)
                };
                self.worldmap.showing_waypoint_manager = false;
                self.worldmap.creating_waypoint = true;
                self.worldmap.waypoint_name_input = format!("Waypoint {}", self.minimap.waypoints.len() + 1);
                self.worldmap.waypoint_coords_input = coords;
                self.worldmap.waypoint_color_idx = (self.minimap.waypoints.len()) % crate::ui::minimap::WAYPOINT_COLORS.len();
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
            } else {
                if input.scroll_delta > 0.0 && self.worldmap.waypoint_manager_scroll > 0 {
                    self.worldmap.waypoint_manager_scroll -= 1;
                } else if input.scroll_delta < 0.0 && self.worldmap.waypoint_manager_scroll + 6 < self.minimap.waypoints.len() {
                    self.worldmap.waypoint_manager_scroll += 1;
                }

                if left_click {
                    if let Some((mx, my)) = mouse_pos {
                        let px = mx as i32;
                        let py = my as i32;
                        let card_w = 540i32;
                        let card_h = 360i32;
                        let dx = (self.width as i32 - card_w) / 2;
                        let dy = (self.height as i32 - card_h) / 2;
                        let (header_close, new_wp_btn, bottom_close, scroll_up, scroll_down) =
                            crate::ui::minimap::waypoint_manager_buttons(dx, dy, card_w, card_h);

                        if px >= header_close.0 && px < header_close.0 + header_close.2 && py >= header_close.1 && py < header_close.1 + header_close.3 {
                            self.worldmap.showing_waypoint_manager = false;
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
                        } else if px >= bottom_close.0 && px < bottom_close.0 + bottom_close.2 && py >= bottom_close.1 && py < bottom_close.1 + bottom_close.3 {
                            self.worldmap.showing_waypoint_manager = false;
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
                        } else if px >= new_wp_btn.0 && px < new_wp_btn.0 + new_wp_btn.2 && py >= new_wp_btn.1 && py < new_wp_btn.1 + new_wp_btn.3 {
                            let coords = if let Some((hx, hy, hz)) = self.worldmap.hovered_world_pos {
                                (hx, hy, hz)
                            } else {
                                (self.camera.position.x.round() as i32, self.camera.position.y.round() as i32, self.camera.position.z.round() as i32)
                            };
                            self.worldmap.showing_waypoint_manager = false;
                            self.worldmap.creating_waypoint = true;
                            self.worldmap.waypoint_name_input = format!("Waypoint {}", self.minimap.waypoints.len() + 1);
                            self.worldmap.waypoint_coords_input = coords;
                            self.worldmap.waypoint_color_idx = (self.minimap.waypoints.len()) % crate::ui::minimap::WAYPOINT_COLORS.len();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                        } else if px >= scroll_up.0 && px < scroll_up.0 + scroll_up.2 && py >= scroll_up.1 && py < scroll_up.1 + scroll_up.3 {
                            if self.worldmap.waypoint_manager_scroll > 0 {
                                self.worldmap.waypoint_manager_scroll -= 1;
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.5, 1.2);
                            }
                        } else if px >= scroll_down.0 && px < scroll_down.0 + scroll_down.2 && py >= scroll_down.1 && py < scroll_down.1 + scroll_down.3 {
                            if self.worldmap.waypoint_manager_scroll + 6 < self.minimap.waypoints.len() {
                                self.worldmap.waypoint_manager_scroll += 1;
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.5, 1.2);
                            }
                        } else {
                            let list_y = dy + 46;
                            let max_rows = 6usize;
                            let start = self.worldmap.waypoint_manager_scroll;
                            let end = (start + max_rows).min(self.minimap.waypoints.len());
                            let mut to_remove = None;
                            let mut to_teleport = None;

                            for i in start..end {
                                let row_idx = i - start;
                                let (vis_btn, tp_btn, del_btn) = crate::ui::minimap::waypoint_row_buttons(dx, list_y, card_w, row_idx);

                                if px >= vis_btn.0 && px < vis_btn.0 + vis_btn.2 && py >= vis_btn.1 && py < vis_btn.1 + vis_btn.3 {
                                    self.minimap.waypoints[i].enabled = !self.minimap.waypoints[i].enabled;
                                    crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                                    break;
                                } else if px >= tp_btn.0 && px < tp_btn.0 + tp_btn.2 && py >= tp_btn.1 && py < tp_btn.1 + tp_btn.3 {
                                    to_teleport = Some(self.minimap.waypoints[i].clone());
                                    break;
                                } else if px >= del_btn.0 && px < del_btn.0 + del_btn.2 && py >= del_btn.1 && py < del_btn.1 + del_btn.3 {
                                    to_remove = Some(i);
                                    break;
                                }
                            }

                            if let Some(wp) = to_teleport {
                                self.camera.position = Vec3::new(wp.x as f32 + 0.5, wp.y as f32 + 1.62, wp.z as f32 + 0.5);
                                self.state = GameState::Playing;
                                out.request_capture_mouse = true;
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.4);
                                return;
                            }

                            if let Some(del_idx) = to_remove {
                                self.minimap.waypoints.remove(del_idx);
                                if self.worldmap.waypoint_manager_scroll > 0 && self.worldmap.waypoint_manager_scroll >= self.minimap.waypoints.len() {
                                    self.worldmap.waypoint_manager_scroll = self.worldmap.waypoint_manager_scroll.saturating_sub(1);
                                }
                                crate::ui::minimap::save_waypoints(self.save_slot, &self.minimap.waypoints);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 0.9);
                            }
                        }
                    }
                }
            }
        } else {
            if escape_down || m_down {
                self.state = GameState::Playing;
                out.request_capture_mouse = true;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
                return;
            }

            if u_down {
                self.worldmap.showing_waypoint_manager = true;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.1);
            }

            if c_down {
                self.worldmap.selected_chunks.clear();
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
            }

            if space_down {
                self.worldmap.center_x = self.camera.position.x;
                self.worldmap.center_z = self.camera.position.z;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.1);
            }

            if b_down {
                let coords = if let Some((hx, hy, hz)) = self.worldmap.hovered_world_pos {
                    (hx, hy, hz)
                } else {
                    (self.camera.position.x.round() as i32, self.camera.position.y.round() as i32, self.camera.position.z.round() as i32)
                };
                self.worldmap.creating_waypoint = true;
                self.worldmap.waypoint_name_input = format!("Waypoint {}", self.minimap.waypoints.len() + 1);
                self.worldmap.waypoint_coords_input = coords;
                self.worldmap.waypoint_color_idx = (self.minimap.waypoints.len()) % crate::ui::minimap::WAYPOINT_COLORS.len();
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
            }

            let pan_speed = (80.0 / self.worldmap.zoom) * dt;
            if input.key_w { self.worldmap.center_z -= pan_speed; }
            if input.key_s { self.worldmap.center_z += pan_speed; }
            if input.key_a { self.worldmap.center_x -= pan_speed; }
            if input.key_d { self.worldmap.center_x += pan_speed; }

            if input.scroll_delta.abs() > 0.01 {
                let factor = if input.scroll_delta > 0.0 { 1.2 } else { 0.833 };
                self.worldmap.zoom = (self.worldmap.zoom * factor).clamp(0.5, 8.0);
            }

            // Top bar button clicks
            let (wp_btn, new_wp_btn, clear_btn) = crate::ui::minimap::worldmap_top_bar_buttons();
            let mut top_bar_clicked = false;

            if left_click {
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    if py < 28 {
                        if px >= wp_btn.0 && px < wp_btn.0 + wp_btn.2 && py >= wp_btn.1 && py < wp_btn.1 + wp_btn.3 {
                            self.worldmap.showing_waypoint_manager = true;
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.1);
                            top_bar_clicked = true;
                        } else if px >= new_wp_btn.0 && px < new_wp_btn.0 + new_wp_btn.2 && py >= new_wp_btn.1 && py < new_wp_btn.1 + new_wp_btn.3 {
                            let coords = if let Some((hx, hy, hz)) = self.worldmap.hovered_world_pos {
                                (hx, hy, hz)
                            } else {
                                (self.camera.position.x.round() as i32, self.camera.position.y.round() as i32, self.camera.position.z.round() as i32)
                            };
                            self.worldmap.creating_waypoint = true;
                            self.worldmap.waypoint_name_input = format!("Waypoint {}", self.minimap.waypoints.len() + 1);
                            self.worldmap.waypoint_coords_input = coords;
                            self.worldmap.waypoint_color_idx = (self.minimap.waypoints.len()) % crate::ui::minimap::WAYPOINT_COLORS.len();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                            top_bar_clicked = true;
                        } else if px >= clear_btn.0 && px < clear_btn.0 + clear_btn.2 && py >= clear_btn.1 && py < clear_btn.1 + clear_btn.3 {
                            self.worldmap.selected_chunks.clear();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
                            top_bar_clicked = true;
                        }
                    }
                }
            }

            // Chunk selection toggling: Right-Click or Shift + Left-Click
            let toggle_chunk = right_click || (input.key_shift && left_click);
            if toggle_chunk && !top_bar_clicked {
                if let Some((_mx, my)) = mouse_pos {
                    if my >= 28.0 && my < (self.height as f32 - 26.0) {
                        if let Some(ch) = self.worldmap.hovered_chunk {
                            if self.worldmap.selected_chunks.contains(&ch) {
                                self.worldmap.selected_chunks.remove(&ch);
                            } else {
                                self.worldmap.selected_chunks.insert(ch);
                            }
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                        }
                    }
                }
            }

            // Map panning via mouse drag (only if not shift-clicking or clicking top bar)
            if let Some((mx, my)) = mouse_pos {
                if left_down && !input.key_shift && !top_bar_clicked && my >= 28.0 && my < (self.height as f32 - 26.0) {
                    if !self.worldmap.is_dragging {
                        self.worldmap.is_dragging = true;
                        self.worldmap.drag_start_mouse = (mx, my);
                        self.worldmap.drag_start_center = (self.worldmap.center_x, self.worldmap.center_z);
                    } else {
                        let dx = mx - self.worldmap.drag_start_mouse.0;
                        let dy = my - self.worldmap.drag_start_mouse.1;
                        self.worldmap.center_x = self.worldmap.drag_start_center.0 - dx / self.worldmap.zoom;
                        self.worldmap.center_z = self.worldmap.drag_start_center.1 - dy / self.worldmap.zoom;
                    }
                } else if !left_down {
                    self.worldmap.is_dragging = false;
                }
            }
        }

        let player_yaw = self.camera.yaw;
        let player_pos = self.camera.position;
        let waypoints = &self.minimap.waypoints;
        crate::ui::minimap::draw_worldmap(
            &mut self.renderer.color_buffer,
            self.width,
            self.height,
            &self.world,
            player_pos,
            player_yaw,
            &self.mob_manager.mobs,
            &mut self.worldmap,
            waypoints,
            mouse_pos,
        );
    }
}
