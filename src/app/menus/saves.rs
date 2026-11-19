//! Ekran wyboru zapisu świata (`SaveSelect`): przewijanie slotów, usuwanie i uruchamianie ładowania świata.

use std::sync::mpsc;
use std::thread;

use super::find_hovered_rect;
use crate::app::{AppOutput, GameState, InputState, RustCraftApp, WorldLoadResult, MAX_SAVE_SLOTS};
use crate::features::entities::MobManager;
use crate::features::player::{Camera, GameMode};
use crate::features::ui::VISIBLE_SAVE_ROWS;
use crate::features::world::{delete_save, get_save_metadata, save_path, World, WorldGeneratorType};

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
            self.save_select_scroll =
                (self.save_select_scroll + VISIBLE_SAVE_ROWS).min(max_scroll);
            self.save_select_slot = (self.save_select_scroll + 1) as u8;
        }

        if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
            self.save_select_slot = if self.save_select_slot <= 1 {
                MAX_SAVE_SLOTS
            } else {
                self.save_select_slot - 1
            };
            let slot_idx = self.save_select_slot as usize - 1;
            if slot_idx < self.save_select_scroll {
                self.save_select_scroll = slot_idx;
            } else if slot_idx >= self.save_select_scroll + VISIBLE_SAVE_ROWS {
                self.save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
            }
        }
        if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
            self.save_select_slot = if self.save_select_slot >= MAX_SAVE_SLOTS {
                1
            } else {
                self.save_select_slot + 1
            };
            let slot_idx = self.save_select_slot as usize - 1;
            if slot_idx >= self.save_select_scroll + VISIBLE_SAVE_ROWS {
                self.save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
            } else if slot_idx < self.save_select_scroll {
                self.save_select_scroll = slot_idx;
            }
        }

        let (save_select_hovered_card, save_select_hovered_btn) =
            if self.is_gamepad_active && !mouse_moved {
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

            if left_down && !self.prev_left_down {
                for (row, &(bx, by, bw, bh)) in self.save_card_rects.iter().enumerate() {
                    if px >= bx && px < bx + bw && py >= by && py < by + bh {
                        let target_slot = (self.save_select_scroll + row + 1) as u8;
                        self.audio
                            .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        if self.save_card_last_clicked_slot == Some(target_slot)
                            && self.save_card_double_click_timer > 0.0
                        {
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

            let cx = self.width as i32 / 2;
            let sb_x = cx + 340;
            let sb_w = 40;
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
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
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

        if mouse_clicked_btn.is_none()
            && (play_action || new_action || delete_action || back_action)
        {
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
        }

        if play_action {
            self.save_slot = self.save_select_slot;
            let meta = &self.save_metadata_cache[self.save_slot as usize - 1];
            if meta.exists {
                let path = save_path(self.save_slot);
                crate::log_info!(
                    "MENU",
                    "Selected existing world in slot {} ('{}', seed: {}) to load",
                    self.save_slot,
                    meta.name,
                    meta.seed
                );
                if crate::platform::capabilities().supports_async_world_load {
                    let (tx, rx) = mpsc::channel();
                    thread::spawn(move || match World::load(&path) {
                        Ok(loaded_world) => {
                            let pos = loaded_world.player_position;
                            let _ = tx.send(WorldLoadResult::Success(loaded_world, Some(pos)));
                        }
                        Err(error) => {
                            let _ = tx.send(WorldLoadResult::Error(error.to_string()));
                        }
                    });
                    self.state = GameState::Loading(Some(rx));
                } else {
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
                            self.load_achievements_for_slot(self.save_slot);
                            self.load_quests_for_slot(self.save_slot);
                            self.minimap.waypoints =
                                crate::ui::minimap::load_waypoints(self.save_slot, self.world.seed);
                            self.state = GameState::Playing;
                        }
                        Err(_) => {
                            self.world = World::new_with_seed(self.seed);
                            self.player.reset(self.world.gamemode);
                            self.camera = Camera::new(self.world.player_position);
                            self.mob_manager = MobManager::new();
                            self.reset_achievements_for_slot(self.save_slot);
                            self.reset_quests_for_slot(
                                self.save_slot,
                                self.create_world_quests_enabled,
                            );
                            self.minimap.waypoints =
                                crate::ui::minimap::load_waypoints(self.save_slot, self.world.seed);
                            self.state = GameState::Playing;
                        }
                    }
                }
            } else {
                self.open_create_world_for_slot(self.save_select_slot, true);
            }
        } else if new_action {
            self.open_create_world_for_slot(self.save_select_slot, false);
        } else if delete_action {
            crate::log_info!(
                "MENU",
                "Requested deletion of save slot {}",
                self.save_select_slot
            );
            let _ = delete_save(self.save_select_slot);
            self.save_metadata_cache[self.save_select_slot as usize - 1] =
                get_save_metadata(self.save_select_slot);
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

    fn open_create_world_for_slot(&mut self, slot: u8, empty: bool) {
        self.create_world_slot = slot;
        self.create_world_name = format!("New World {:02}", slot);
        self.create_world_seed = String::new();
        self.create_world_gamemode = GameMode::Survival;
        self.create_world_generator_type = WorldGeneratorType::Default;
        self.create_world_quests_enabled = true;
        self.create_world_focus = 0;
        self.create_world_selected = false;
        self.state = GameState::CreateWorld;
        crate::log_info!(
            "MENU",
            "Opened Create World menu for {}slot {}",
            if empty { "empty " } else { "" },
            self.create_world_slot
        );
    }
}
