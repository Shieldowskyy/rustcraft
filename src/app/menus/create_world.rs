//! Ekran tworzenia nowego świata (`CreateWorld`): nazwa, seed, tryb gry, generator i opcja questów.

use std::sync::mpsc;
use std::thread;

use super::find_hovered_rect;
use crate::app::{AppOutput, GameState, InputState, RustCraftApp, WorldLoadResult};
use crate::cli::generate_random_seed;
use crate::features::entities::MobManager;
use crate::features::player::{Camera, GameMode};
use crate::features::world::{World, WorldGeneratorType};

impl RustCraftApp {
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
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
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
        let up_just = (up_arrow_down && !self.prev_up_arrow_down)
            || input.gp_dpad_up_just_pressed
            || input.gp_menu_up;
        let down_just = (down_arrow_down && !self.prev_down_arrow_down)
            || input.gp_dpad_down_just_pressed
            || input.gp_menu_down;
        let left_just = (left_arrow_down && !self.prev_left_arrow_down)
            || input.gp_dpad_left_just_pressed
            || input.gp_menu_left;
        let right_just = (right_arrow_down && !self.prev_right_arrow_down)
            || input.gp_dpad_right_just_pressed
            || input.gp_menu_right;

        if tab_just || down_just {
            self.create_world_focus = (self.create_world_focus + 1) % 8;
            self.create_world_selected = false;
        } else if up_just {
            self.create_world_focus = if self.create_world_focus == 0 {
                7
            } else {
                self.create_world_focus - 1
            };
            self.create_world_selected = false;
        } else if left_just && self.create_world_focus == 7 {
            self.create_world_focus = 6;
            self.create_world_selected = false;
        } else if right_just && self.create_world_focus == 6 {
            self.create_world_focus = 7;
            self.create_world_selected = false;
        } else if left_just && self.create_world_focus == 2 {
            self.create_world_focus = 1;
            self.create_world_selected = false;
        } else if right_just && self.create_world_focus == 1 {
            self.create_world_focus = 2;
            self.create_world_selected = false;
        }

        let a_just = a_down && !self.prev_a_down;
        let c_just = c_down && !self.prev_c_down;
        let v_just = v_down && !self.prev_v_down;
        let x_just = x_down && !self.prev_x_down;

        if ctrl_down {
            if a_just {
                if (self.create_world_focus == 0 && !self.create_world_name.is_empty())
                    || (self.create_world_focus == 1 && !self.create_world_seed.is_empty())
                {
                    self.create_world_selected = true;
                }
            } else if c_just {
                if self.create_world_focus == 0 {
                    crate::platform::clipboard::set_clipboard(&self.create_world_name);
                } else if self.create_world_focus == 1 {
                    crate::platform::clipboard::set_clipboard(&self.create_world_seed);
                }
            } else if x_just {
                if self.create_world_focus == 0 {
                    crate::platform::clipboard::set_clipboard(&self.create_world_name);
                    self.create_world_name.clear();
                    self.create_world_selected = false;
                } else if self.create_world_focus == 1 {
                    crate::platform::clipboard::set_clipboard(&self.create_world_seed);
                    self.create_world_seed.clear();
                    self.create_world_selected = false;
                }
            } else if v_just {
                let clip = crate::platform::clipboard::get_clipboard();
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

        let backspace_just =
            (backspace_down && !self.prev_backspace_down) || (del_down && !self.prev_del_down);
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
            || click_elem == Some(7)
            || (self.create_world_focus == 7 && (enter_just || south_just));

        let do_create = click_elem == Some(6)
            || (enter_just
                && (self.create_world_focus == 0
                    || self.create_world_focus == 1
                    || self.create_world_focus == 6))
            || (south_just && self.create_world_focus == 6);

        let do_rand_seed =
            (self.create_world_focus == 2 && (enter_just || south_just)) || click_elem == Some(2);
        let do_toggle_mode =
            (self.create_world_focus == 3 && (enter_just || south_just)) || click_elem == Some(3);
        let do_toggle_world_type =
            (self.create_world_focus == 4 && (enter_just || south_just)) || click_elem == Some(4);
        let do_toggle_quests =
            (self.create_world_focus == 5 && (enter_just || south_just)) || click_elem == Some(5);

        if click_elem.is_none()
            && (do_cancel
                || do_create
                || do_rand_seed
                || do_toggle_mode
                || do_toggle_world_type
                || do_toggle_quests)
        {
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
        }

        if do_rand_seed {
            self.create_world_seed = format!("{}", generate_random_seed());
            self.create_world_selected = false;
            crate::log_debug!(
                "MENU",
                "Generated random seed for new world: {}",
                self.create_world_seed
            );
        } else if do_toggle_mode {
            self.create_world_gamemode = match self.create_world_gamemode {
                GameMode::Survival => GameMode::Creative,
                GameMode::Creative => GameMode::Survival,
            };
            crate::log_debug!(
                "MENU",
                "Toggled world creation mode to {:?}",
                self.create_world_gamemode
            );
        } else if do_toggle_world_type {
            self.create_world_generator_type = match self.create_world_generator_type {
                WorldGeneratorType::Default => WorldGeneratorType::Superflat,
                WorldGeneratorType::Superflat => WorldGeneratorType::Default,
            };
            crate::log_debug!(
                "MENU",
                "Toggled world creation generator type to {:?}",
                self.create_world_generator_type
            );
        } else if do_toggle_quests {
            self.create_world_quests_enabled = !self.create_world_quests_enabled;
            crate::log_debug!(
                "MENU",
                "Toggled world creation quests_enabled to {}",
                self.create_world_quests_enabled
            );
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
                if s == 0 {
                    generate_random_seed()
                } else {
                    s
                }
            };

            let name = if self.create_world_name.trim().is_empty() {
                format!("New World {:02}", self.create_world_slot)
            } else {
                self.create_world_name.trim().to_string()
            };

            let gamemode = self.create_world_gamemode;
            let generator_type = self.create_world_generator_type;
            self.save_slot = self.create_world_slot;
            self.seed = seed;
            self.player.reset(gamemode);
            self.reset_achievements_for_slot(self.save_slot);
            self.reset_quests_for_slot(self.save_slot, self.create_world_quests_enabled);
            crate::log_info!(
                "MENU",
                "Creating new world '{}' (seed: {}, mode: {:?}, type: {:?}) in slot {}",
                name,
                seed,
                gamemode,
                generator_type,
                self.save_slot
            );

            if crate::platform::capabilities().supports_async_world_load {
                let (tx, rx) = mpsc::channel();
                let thread_seed = self.seed;
                let thread_name = name;
                let thread_gamemode = gamemode;
                let thread_gen_type = generator_type;
                thread::spawn(move || {
                    let mut new_world = World::new_with_generator(thread_seed, thread_gen_type);
                    new_world.name = thread_name;
                    new_world.gamemode = thread_gamemode;
                    let _ = tx.send(WorldLoadResult::Success(new_world, None));
                });
                self.state = GameState::Loading(Some(rx));
            } else {
                self.world = World::new_with_generator(self.seed, generator_type);
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
                self.create_world_generator_type,
                self.create_world_quests_enabled,
                self.create_world_focus,
                hovered_elem,
                self.total_time,
                self.create_world_selected,
                self.is_gamepad_active,
            );
        }
    }
}
