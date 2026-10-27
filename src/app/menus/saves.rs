#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use std::sync::mpsc;
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use std::thread;

use super::find_hovered_rect;
use crate::app::{AppOutput, GameState, InputState, RustCraftApp, MAX_SAVE_SLOTS};
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use crate::app::WorldLoadResult;
use crate::cli::generate_random_seed;
#[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
use crate::gameplay::mob::MobManager;
use crate::gameplay::player::GameMode;
#[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
use crate::graphics::camera::Camera;
use crate::ui::VISIBLE_SAVE_ROWS;
use crate::world::storage::{delete_save, get_save_metadata, save_path};
use crate::world::World;

impl RustCraftApp {
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
        } else if mouse_pos.is_some() {
            let card = find_hovered_rect(mouse_pos, &self.save_card_rects);
            let btn = find_hovered_rect(mouse_pos, &self.save_btn_rects);
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
        } else {
            find_hovered_rect(mouse_pos, &self.create_world_rects)
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
}
