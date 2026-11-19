//! Główna klatka aktualizacji i renderowania (`update_and_render`, `update`, `resize`, obsługa toastów i zrzutów profilera).

use crate::features::audio::SoundtrackCategory;
use crate::features::blocks::BlockId;
use crate::features::net::NetworkEvent;
use crate::features::player::GameMode;

use super::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
    pub fn pixel_buffer(&self) -> &[u32] {
        self.renderer.pixel_buffer()
    }

    pub fn pixel_buffer_mut(&mut self) -> &mut [u32] {
        self.renderer.pixel_buffer_mut()
    }

    pub fn update(&mut self, dt: f32, input: &InputState) -> AppOutput {
        self.update_and_render(dt, input)
    }

    pub fn update_and_render(&mut self, dt: f32, input: &InputState) -> AppOutput {
        let mut bot_input = input.clone();
        if let Some(mut runner) = self.autotest.take() {
            runner.update(self, dt, &mut bot_input);
            self.autotest = Some(runner);
        }
        let input = &bot_input;

        self.total_time += dt;
        if let Some((_, remaining, _)) = &mut self.multiplayer_toast {
            *remaining = (*remaining - dt).max(0.0);
            if *remaining == 0.0 {
                self.multiplayer_toast = None;
            }
        }
        if let Some((_, remaining)) = &mut self.achievement_toast {
            *remaining = (*remaining - dt).max(0.0);
            if *remaining == 0.0 {
                self.achievement_toast = self
                    .pending_achievement_toasts
                    .pop_front()
                    .map(|id| (id, 6.0));
            }
        }
        if let Some((_, remaining)) = &mut self.quest_toast {
            *remaining = (*remaining - dt).max(0.0);
            if *remaining == 0.0 {
                self.quest_toast = self.pending_quest_toasts.pop_front().map(|id| (id, 5.5));
            }
        }
        self.renderer.total_time = self.total_time;
        self.renderer.chunk_anim = self.settings.chunk_anim;
        self.hand_animation = (self.hand_animation - dt * 3.5).max(0.0);
        self.profiler_copied_timer = (self.profiler_copied_timer - dt).max(0.0);
        self.controls.update(input);

        let mut out = AppOutput {
            request_capture_mouse: false,
            request_release_mouse: false,
            quit_requested: false,
            is_playing: matches!(self.state, GameState::Playing),
            title: String::new(),
            rumble_request: None,
            cursor_visible: None,
        };

        let mouse_moved = match (input.mouse_pos, self.last_mouse_pos) {
            (Some((mx, my)), Some((lx, ly))) => (mx - lx).abs() > 1.5 || (my - ly).abs() > 1.5,
            (Some(_), None) => true,
            _ => false,
        };
        self.last_mouse_pos = input.mouse_pos;

        let kb_mouse_activity = mouse_moved
            || input.mouse_left
            || input.mouse_right
            || input.mouse_dx.abs() > 1.0
            || input.mouse_dy.abs() > 1.0
            || input.scroll_delta.abs() > 0.1
            || input.key_w
            || input.key_s
            || input.key_a
            || input.key_d
            || input.key_space
            || input.key_shift
            || input.key_ctrl
            || input.key_escape
            || input.key_e
            || input.key_q
            || input.key_enter
            || input.key_up
            || input.key_down
            || input.key_left
            || input.key_right
            || !input.typed_chars.is_empty();

        let gp_stick_moved = input.gp_left_stick_x.abs() > 0.25
            || input.gp_left_stick_y.abs() > 0.25
            || input.gp_right_stick_x.abs() > 0.25
            || input.gp_right_stick_y.abs() > 0.25
            || input.gp_left_trigger > 0.25
            || input.gp_right_trigger > 0.25;

        let gp_activity = input.gp_connected
            && (gp_stick_moved
                || input.gp_btn_south
                || input.gp_btn_east
                || input.gp_btn_west
                || input.gp_btn_north
                || input.gp_btn_lb
                || input.gp_btn_rb
                || input.gp_btn_l3
                || input.gp_btn_r3
                || input.gp_btn_start
                || input.gp_btn_select
                || input.gp_dpad_up
                || input.gp_dpad_down
                || input.gp_dpad_left
                || input.gp_dpad_right
                || input.gp_south_just_pressed
                || input.gp_east_just_pressed
                || input.gp_west_just_pressed
                || input.gp_north_just_pressed
                || input.gp_lb_just_pressed
                || input.gp_rb_just_pressed
                || input.gp_r3_just_pressed
                || input.gp_start_just_pressed
                || input.gp_select_just_pressed
                || input.gp_dpad_up_just_pressed
                || input.gp_dpad_down_just_pressed
                || input.gp_dpad_left_just_pressed
                || input.gp_dpad_right_just_pressed
                || input.gp_menu_up
                || input.gp_menu_down
                || input.gp_menu_left
                || input.gp_menu_right
                || input.gp_right_trigger_just_pressed
                || input.gp_left_trigger_just_pressed);

        if gp_activity {
            self.is_gamepad_active = true;
        } else if kb_mouse_activity || !input.gp_connected {
            self.is_gamepad_active = false;
        }

        let showing_logo = self.update_and_render_dev_logo(dt, input);
        if !showing_logo {
            match self.state {
                GameState::MainMenu => self.update_and_render_main_menu(input, mouse_moved, &mut out),
                GameState::MultiplayerMenu => {
                    self.update_and_render_multiplayer_menu(dt, input, &mut out)
                }
                GameState::EditNickname(prev) => {
                    self.update_and_render_nickname_dialog(input, prev, &mut out)
                }
                GameState::MultiplayerConnecting { .. } => {
                    self.update_and_render_multiplayer_connecting(dt, input, &mut out)
                }
                GameState::SaveSelect => {
                    self.update_and_render_save_select(dt, input, mouse_moved, &mut out)
                }
                GameState::CreateWorld => {
                    self.update_and_render_create_world(input, mouse_moved, &mut out)
                }
                GameState::Settings(prev) => {
                    self.update_and_render_settings(input, prev, mouse_moved, &mut out)
                }
                GameState::ControlsRebind(prev) => {
                    self.update_and_render_controls_rebind(prev, dt, input, mouse_moved, &mut out)
                }
                GameState::Loading(ref rx_opt) => {
                    let is_sync = rx_opt.is_none();
                    let res = rx_opt.as_ref().and_then(|rx| rx.try_recv().ok());
                    self.update_and_render_loading(res, is_sync);
                }
                GameState::Playing => self.update_and_render_playing(dt, input, &mut out),
                GameState::Paused => {
                    self.update_and_render_paused(dt, input, mouse_moved, &mut out)
                }
                GameState::AchievementTree => {
                    self.update_and_render_achievement_tree(input, &mut out)
                }
                GameState::Questbook => self.update_and_render_questbook(input, &mut out),
                GameState::Inventory => {
                    self.update_and_render_inventory(dt, input, mouse_moved, &mut out)
                }
                GameState::Chest { pos } => {
                    self.update_and_render_chest(dt, input, pos, &mut out)
                }
                GameState::Furnace { pos } => {
                    self.update_and_render_furnace(dt, input, pos, &mut out)
                }
                GameState::VillagerTrade {
                    mob_id,
                    selected_trade,
                } => self.update_and_render_villager_trade(
                    dt,
                    input,
                    mob_id,
                    selected_trade,
                    &mut out,
                ),
                GameState::DeathScreen => {
                    self.update_and_render_death_screen(input, mouse_moved, &mut out)
                }
                GameState::WorldMap => {
                    self.update_and_render_worldmap(dt, input, mouse_moved, &mut out)
                }
            }
        }

        if !matches!(
            self.state,
            GameState::Playing | GameState::MultiplayerConnecting { .. }
        ) {
            let events = self
                .net_client
                .as_mut()
                .filter(|client| client.is_connected)
                .map(|client| client.update(dt));
            if let Some(events) = events {
                if let Some(reason) = events.iter().find_map(|event| match event {
                    NetworkEvent::Disconnected { reason } => Some(reason.clone()),
                    _ => None,
                }) {
                    self.handle_multiplayer_disconnect(reason);
                    out.request_release_mouse = true;
                } else {
                    self.pending_network_events.extend(events);
                }
            }
        }

        let soundtrack_category = match self.state {
            GameState::Playing
            | GameState::Paused
            | GameState::AchievementTree
            | GameState::Questbook
            | GameState::Inventory
            | GameState::Chest { .. }
            | GameState::Furnace { .. }
            | GameState::VillagerTrade { .. }
            | GameState::DeathScreen
            | GameState::WorldMap => match self.player.gamemode {
                GameMode::Creative => SoundtrackCategory::Creative,
                GameMode::Survival => SoundtrackCategory::Survival,
            },
            _ => SoundtrackCategory::Menu,
        };

        self.soundtrack.update(
            dt,
            soundtrack_category,
            &mut self.audio,
            self.settings.music_volume,
            self.settings.mute_audio,
        );

        self.record_prev_inputs(input);

        out.title = format!(
            "RustCraft [{} renderer] - {} FPS - block: {}{}{}",
            self.renderer.backend_name(),
            self.fps_display,
            BlockId::from_u8(self.renderer.selected_block()).display_name(),
            if self.fly_mode { " - [FLYING]" } else { "" },
            if matches!(
                self.state,
                GameState::Paused | GameState::AchievementTree | GameState::Questbook
            ) {
                " - [PAUSED]"
            } else {
                ""
            }
        );

        self.handle_profiler_screenshot();
        self.render_post_frame_overlays(dt);

        out.is_playing = matches!(self.state, GameState::Playing) && !self.chat.is_open;
        out.rumble_request = self.pending_rumble.take();
        if !input.is_mouse_captured && !matches!(self.state, GameState::Playing) {
            out.cursor_visible = Some(!self.is_gamepad_active);
        }

        out
    }

    fn record_prev_inputs(&mut self, input: &InputState) {
        self.prev_left_down = input.mouse_left;
        self.prev_right_down = input.mouse_right;
        self.prev_middle_down = input.mouse_middle;
        self.prev_escape_down = input.key_escape;
        self.prev_p_down = input.key_p;
        self.prev_e_down = input.key_e;
        self.prev_f_down = input.key_f;
        self.prev_q_down = input.key_q;
        self.prev_m_down = input.key_m;
        self.prev_g_down = input.key_g;
        self.prev_f5_down = input.key_f5;
        self.prev_enter_down = input.key_enter;
        self.prev_up_arrow_down = input.key_up;
        self.prev_down_arrow_down = input.key_down;
        self.prev_left_arrow_down = input.key_left;
        self.prev_right_arrow_down = input.key_right;
        self.prev_n_down = input.key_n;
        self.prev_del_down = input.key_del;
        self.prev_backspace_down = input.key_backspace;
        self.prev_tab_down = input.key_tab;
        self.prev_a_down = input.key_a;
        self.prev_c_down = input.key_c;
        self.prev_v_down = input.key_v;
        self.prev_x_down = input.key_x;
        self.prev_key_digits = input.key_digits;
        self.prev_t_down = input.key_t;
        self.prev_slash_down = input.key_slash;
        self.prev_b_down = input.key_b;
        self.prev_u_down = input.key_u;
        self.prev_space_down = input.key_space;
    }

    /// Returns true when the current game state renders the live 3D world (Playing or in-game menu overlay).
    pub fn uses_live_3d_scene(&self) -> bool {
        self.dev_logo.is_none()
            && matches!(
                self.state,
                GameState::Playing
                    | GameState::Paused
                    | GameState::AchievementTree
                    | GameState::Questbook
                    | GameState::Inventory
                    | GameState::Chest { .. }
                    | GameState::Furnace { .. }
                    | GameState::VillagerTrade { .. }
                    | GameState::DeathScreen
                    | GameState::Settings(super::PreviousMenu::Paused)
            )
    }

    /// Encodes and delivers a requested profiler screenshot from an `XRGB` pixel buffer.
    pub fn submit_profiler_screenshot(&self, xrgb_pixels: &[u32]) {
        let Some(ref server) = self.profiler_server else {
            return;
        };
        if !server.is_screenshot_requested() {
            return;
        }
        let bmp_bytes = crate::core::profiler_server::encode_bmp(
            xrgb_pixels,
            self.width,
            self.height,
        );
        let bpos = self.camera.position.floor_to_ivec();
        let biome = self.world.biome_name_at(bpos.x, bpos.z).to_string();
        let game_state_str = match &self.state {
            GameState::MainMenu => "Main Menu",
            GameState::SaveSelect => "Save Select",
            GameState::CreateWorld => "Create World",
            GameState::Settings(_) => "Settings",
            GameState::ControlsRebind(_) => "Controls Rebind",
            GameState::EditNickname(_) => "Change Nickname",
            GameState::Loading(_) => "Loading",
            GameState::Playing => "Playing",
            GameState::Paused => "Paused",
            GameState::AchievementTree => "Achievement Tree",
            GameState::Questbook => "Questbook",
            GameState::Inventory => "Inventory",
            GameState::Chest { .. } => "Chest",
            GameState::Furnace { .. } => "Furnace",
            GameState::VillagerTrade { .. } => "Villager Trade",
            GameState::DeathScreen => "Death Screen",
            GameState::WorldMap => "World Map",
            GameState::MultiplayerMenu => "Multiplayer Menu",
            GameState::MultiplayerConnecting { .. } => "Multiplayer Connecting",
        };
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        server.set_screenshot(crate::core::profiler_server::ScreenshotData {
            bmp_bytes,
            width: self.width,
            height: self.height,
            timestamp_ms: now_ms,
            fps: self.fps_display,
            player_pos: [
                self.camera.position.x,
                self.camera.position.y,
                self.camera.position.z,
            ],
            biome,
            game_state: game_state_str.to_string(),
            gamemode: match self.player.gamemode {
                GameMode::Survival => "Survival".to_string(),
                GameMode::Creative => "Creative".to_string(),
            },
        });
    }

    fn handle_profiler_screenshot(&self) {
        if crate::platform::capabilities().uses_hardware_gxm && self.uses_live_3d_scene() {
            return;
        }
        self.submit_profiler_screenshot(self.renderer.pixel_buffer());
    }

    fn render_post_frame_overlays(&mut self, dt: f32) {
        if let Some(ref runner) = self.autotest {
            runner.render_overlay(self.renderer.pixel_buffer_mut(), self.width, self.height);
        }

        if self.menu_fade_timer > 0.0 {
            self.menu_fade_timer = (self.menu_fade_timer - dt.min(0.05)).max(0.0);
            let a = (self.menu_fade_timer / crate::ui::dev_logo::MENU_FADE_IN).clamp(0.0, 1.0);
            crate::ui::fill_rect_alpha(
                self.renderer.pixel_buffer_mut(),
                self.width,
                self.height,
                0,
                0,
                self.width as i32,
                self.height as i32,
                0,
                a,
            );
        }

        let is_paused = matches!(
            self.state,
            GameState::Paused | GameState::AchievementTree | GameState::Questbook
        );
        if self.settings.show_music_toast {
            self.soundtrack.draw_toast(
                self.renderer.pixel_buffer_mut(),
                self.width,
                self.height,
                is_paused,
                &self.audio,
            );
        }
        if let Some((message, _, is_error)) = &self.multiplayer_toast {
            crate::ui::draw_multiplayer_toast(
                self.renderer.pixel_buffer_mut(),
                self.width,
                self.height,
                message,
                *is_error,
            );
        }
        if let Some((id, _)) = self.achievement_toast {
            crate::ui::draw_achievement_toast(
                self.renderer.pixel_buffer_mut(),
                self.width,
                self.height,
                crate::core::i18n::t(id.name_key()),
                crate::core::i18n::t(id.description_key()),
            );
        }
        if let Some((quest_idx, _)) = self.quest_toast {
            let key_hint = self.questbook_key_hint();
            crate::ui::draw_quest_toast(
                self.renderer.pixel_buffer_mut(),
                self.width,
                self.height,
                quest_idx,
                key_hint,
            );
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        if width == 0 || height == 0 || (self.width == width && self.height == height) {
            return;
        }
        self.width = width;
        self.height = height;
        self.renderer.resize(width, height);

        self.main_menu_rects = self.renderer.main_menu_button_rects();
        self.main_menu_nick_rect = self.renderer.main_menu_nickname_rect();
        self.nickname_dialog_rects = self.renderer.nickname_dialog_rects();
        self.multiplayer_menu_rects = self.renderer.multiplayer_menu_rects();
        self.save_card_rects = self.renderer.save_card_rects();
        self.save_btn_rects = self.renderer.save_bottom_button_rects();
        self.create_world_rects = self.renderer.create_world_rects();
        self.settings_tab_rects = self.renderer.settings_tab_rects();
        self.settings_rects = self.renderer.settings_graphics_rects();
        self.settings_audio_rects = self.renderer.settings_audio_rects();
        self.settings_controls_rects = self.renderer.settings_controls_rects();
        self.settings_debug_rects = self.renderer.settings_debug_rects();
        self.pause_menu_rects = self.renderer.pause_menu_button_rects();
        self.death_screen_rects = self.renderer.death_screen_button_rects();

        let aspect = width as f32 / height as f32;
        self.last_aspect = aspect;
        self.proj_matrix = self.camera.projection_matrix(aspect);
    }
}
