use super::{find_hovered_rect, point_in_rect};
use crate::app::{AppOutput, GameState, InputState, PreviousMenu, RustCraftApp, MAX_SAVE_SLOTS};
use crate::ui::VISIBLE_SAVE_ROWS;

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
            let nick = point_in_rect(px, py, self.main_menu_nick_rect);
            let btn = find_hovered_rect(mouse_pos, &self.main_menu_rects);
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

        // Easter egg: clicking splash text cycles to next text
        if left_down && !self.prev_left_down {
            if let Some((mx, my)) = mouse_pos {
                let cx = self.width as i32 / 2;
                let title_w = crate::graphics::font::measure_text("RUSTCRAFT", 4, 3);
                let splash_cx = cx + title_w / 2 + 10;
                let splash_cy = 138;
                let px = mx as i32;
                let py = my as i32;
                if px >= splash_cx - 70 && px <= splash_cx + 110 && py >= splash_cy - 25 && py <= splash_cy + 25 {
                    self.splash_index = (self.splash_index + 1) % crate::ui::splashes::SPLASH_TEXTS.len();
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                }
            }
        }

        let splash_text = crate::ui::splashes::SPLASH_TEXTS[self.splash_index % crate::ui::splashes::SPLASH_TEXTS.len()];
        self.renderer.draw_main_menu(
            hovered_idx,
            &self.settings.nickname,
            self.main_menu_nick_hovered,
            self.is_gamepad_active,
            splash_text,
            self.total_time,
        );
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

        if let Some(hovered) = find_hovered_rect(mouse_pos, &rects) {
            hovered_idx = Some(hovered);
            self.nickname_dialog_hovered_btn = Some(hovered);
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
                let splash_text = crate::ui::splashes::SPLASH_TEXTS[self.splash_index % crate::ui::splashes::SPLASH_TEXTS.len()];
                self.renderer.draw_main_menu(None, &self.settings.nickname, false, self.is_gamepad_active, splash_text, self.total_time);
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
            || matches!(self.state, GameState::ControlsRebind(_))
            || self.is_ime_active
    }
}
