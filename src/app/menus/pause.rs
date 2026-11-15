use super::find_hovered_rect;
use crate::app::{AppOutput, GameState, InputState, PreviousMenu, RustCraftApp};
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use crate::app::WorldLoadResult;
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use crate::gameplay::mob::MobManager;
#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use crate::graphics::camera::Camera;
use crate::core::controls::Action;
use crate::core::math::Vec3;
use crate::core::profiler::FrameProfiler;
use crate::world::storage::get_save_metadata;

impl RustCraftApp {
    #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
    pub(crate) fn update_and_render_loading(&mut self, result: Option<WorldLoadResult>) {
        let splash_text = crate::ui::splashes::splash_text(self.splash_index);
        self.renderer.draw_main_menu(None, &self.settings.nickname, false, self.is_gamepad_active, splash_text, self.total_time);

        if let Some(result) = result {
            match result {
                WorldLoadResult::Success(loaded_world, loaded_pos) => {
                    self.load_achievements_for_slot(self.save_slot);
                    self.load_quests_for_slot(self.save_slot);
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
                    self.minimap.waypoints = crate::ui::minimap::load_waypoints(self.save_slot, self.world.seed);
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
        let splash_text = crate::ui::splashes::splash_text(self.splash_index);
        self.renderer.draw_main_menu(None, &self.settings.nickname, false, self.is_gamepad_active, splash_text, self.total_time);
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
        } else {
            find_hovered_rect(mouse_pos, &self.pause_menu_rects)
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
                Some(0) | None => 6,
                Some(i) => i - 1,
            };
            self.pause_hovered_btn = Some(next);
            hovered_idx = Some(next);
        }
        if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
            let next = match self.pause_hovered_btn {
                Some(6) | None => 0,
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
                    if self.is_hosting_multiplayer {
                        if let Some(client) = &mut self.net_client {
                            client.disconnect();
                        }
                        self.stop_host_multiplayer();
                        self.is_multiplayer = false;
                        self.multiplayer_status = Some("Hosting stopped".to_string());
                        self.show_multiplayer_toast(crate::core::i18n::t("multiplayer.host_stopped_toast"), false);
                    } else if !self.is_multiplayer {
                        match self.start_host_multiplayer() {
                            Ok(()) => {
                                crate::log_info!("NETWORK", "Started hosting the current world");
                            }
                            Err(error) => {
                                self.multiplayer_status = Some(error.clone());
                                self.show_multiplayer_toast(error.clone(), true);
                                crate::log_error!("NETWORK", "Could not host current world: {}", error);
                            }
                        }
                    }
                }
                4 => {
                    self.state = GameState::AchievementTree;
                    crate::log_info!("MENU", "Opened Field Notes from pause menu");
                }
                5 => {
                    if self.is_multiplayer {
                        if let Some(client) = &mut self.net_client {
                            client.disconnect();
                        }
                        let was_hosting = self.is_hosting_multiplayer;
                        self.stop_host_multiplayer();
                        self.is_multiplayer = false;
                        if was_hosting {
                            let _ = self.save_world_progress();
                            crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
                            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                        }
                    } else {
                        let _ = self.save_world_progress();
                        crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
                        self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    }
                    self.state = GameState::MainMenu;
                    out.request_release_mouse = true;
                    crate::log_info!("STATE", "Returned to Main Menu from pause menu");
                }
                6 => {
                    if self.is_multiplayer {
                        if let Some(client) = &mut self.net_client {
                            client.disconnect();
                        }
                        let was_hosting = self.is_hosting_multiplayer;
                        self.stop_host_multiplayer();
                        self.is_multiplayer = false;
                        if was_hosting {
                            let _ = self.save_world_progress();
                            crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
                        }
                    } else {
                        let _ = self.save_world_progress();
                        crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
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
        if self.is_hosting_multiplayer
            && input.key_ctrl
            && input.key_c
            && !self.prev_c_down
        {
            if let Some(invite) = &self.multiplayer_invite {
                crate::ui::clipboard::set_clipboard(invite);
                self.show_multiplayer_toast(crate::core::i18n::t("multiplayer.host_copy_toast"), false);
            }
        }
        if q_down && !self.prev_q_down {
            if self.is_hosting_multiplayer {
                if let Some(client) = &mut self.net_client {
                    client.disconnect();
                }
                self.stop_host_multiplayer();
                self.is_multiplayer = false;
            }
            let _ = self.save_world_progress();
            crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
            out.quit_requested = true;
            crate::log_info!("STATE", "Saved and quitting game via shortcut");
        }
        if m_down && !self.prev_m_down {
            if self.is_hosting_multiplayer {
                if let Some(client) = &mut self.net_client {
                    client.disconnect();
                }
                self.stop_host_multiplayer();
                self.is_multiplayer = false;
            }
            let _ = self.save_world_progress();
            crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
            self.state = GameState::MainMenu;
            out.request_release_mouse = true;
            crate::log_info!("STATE", "Saved and returned to Main Menu via shortcut");
        }

        self.renderer.fog_enabled = self.settings.fog;
        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
        self.renderer.draw_pause_menu(
            self.fly_mode,
            hovered_idx,
            self.is_gamepad_active,
            !self.is_multiplayer || self.is_hosting_multiplayer,
            self.is_hosting_multiplayer,
            self.multiplayer_status.as_deref(),
        );

        if let Some(ref server) = self.profiler_server {
            let mut cur_profiler = FrameProfiler::default();
            cur_profiler.fps = (1.0 / dt.max(0.001)).round() as u32;
            let snap = self.build_profiler_snapshot(&cur_profiler);
            server.update_snapshot(snap);
        }
    }

    pub(crate) fn update_and_render_achievement_tree(
        &mut self,
        input: &InputState,
        _out: &mut AppOutput,
    ) {
        if input.key_escape
            || self.controls.is_just_pressed(Action::Pause)
            || input.gp_east_just_pressed
        {
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            self.state = GameState::Paused;
        }

        let previous_tab = self.field_guide_tab;
        if input.key_left && !self.prev_left_arrow_down || input.gp_dpad_left_just_pressed {
            self.field_guide_tab = (self.field_guide_tab + 2) % 3;
        } else if input.key_right && !self.prev_right_arrow_down
            || input.gp_dpad_right_just_pressed
        {
            self.field_guide_tab = (self.field_guide_tab + 1) % 3;
        }
        if input.mouse_left && !self.prev_left_down {
            if let Some(tab) = crate::ui::field_guide_tab_at(self.width, input.mouse_pos) {
                self.field_guide_tab = tab;
            }
        }
        if self.field_guide_tab != previous_tab {
            self.field_guide_scroll = 0;
        } else if self.field_guide_tab == 2
            && (input.scroll_delta != 0.0
                || input.key_up && !self.prev_up_arrow_down
                || input.key_down && !self.prev_down_arrow_down
                || input.gp_dpad_up_just_pressed
                || input.gp_dpad_down_just_pressed)
        {
            let page_size = crate::ui::block_catalog_page_size(self.width, self.height);
            let max_scroll =
                crate::ui::block_catalog_entry_count().saturating_sub(page_size);
            let scroll_up = input.scroll_delta > 0.0
                || input.key_up && !self.prev_up_arrow_down
                || input.gp_dpad_up_just_pressed;
            self.field_guide_scroll = if scroll_up {
                self.field_guide_scroll.saturating_sub(page_size)
            } else {
                (self.field_guide_scroll + page_size).min(max_scroll)
            };
        }

        self.renderer.fog_enabled = self.settings.fog;
        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
        crate::ui::draw_field_guide(
            self.renderer.pixel_buffer_mut(),
            self.width,
            self.height,
            &self.achievements,
            self.field_guide_tab,
            self.field_guide_scroll,
        );
    }

    pub(crate) fn update_and_render_questbook(
        &mut self,
        input: &InputState,
        out: &mut AppOutput,
    ) {
        let just_clicked = input.mouse_left && !self.prev_left_down;

        if input.key_escape
            || self.controls.is_just_pressed(Action::Pause)
            || self.controls.is_just_pressed(Action::Questbook)
            || input.gp_east_just_pressed
            || (just_clicked && crate::ui::questbook_close_hit(self.width, self.height, input.mouse_pos))
        {
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            self.state = GameState::Playing;
            out.request_capture_mouse = true;
            return;
        }

        let cat_count = crate::gameplay::quests::QuestCategory::ALL.len();
        let prev_tab = self.questbook_tab;
        if (input.key_left && !self.prev_left_arrow_down) || input.gp_dpad_left_just_pressed {
            self.questbook_tab = (self.questbook_tab + cat_count - 1) % cat_count;
        } else if (input.key_right && !self.prev_right_arrow_down) || input.gp_dpad_right_just_pressed {
            self.questbook_tab = (self.questbook_tab + 1) % cat_count;
        }
        if just_clicked {
            if let Some(tab) = crate::ui::questbook_tab_at(self.width, self.height, input.mouse_pos) {
                self.questbook_tab = tab;
            }
        }

        let active_cat = crate::gameplay::quests::QuestCategory::ALL
            .get(self.questbook_tab)
            .copied()
            .unwrap_or(crate::gameplay::quests::QuestCategory::Basics);
        let cat_quests = crate::gameplay::quests::quests_in_category(active_cat);

        if self.questbook_tab != prev_tab || !cat_quests.contains(&self.questbook_selected) {
            self.questbook_selected = cat_quests.first().copied().unwrap_or(0);
            self.questbook_scroll = 0;
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.5, 1.05);
        }

        let max_scroll = cat_quests.len().saturating_sub(crate::ui::QUESTBOOK_VISIBLE_CARDS);
        if input.scroll_delta > 0.0 {
            self.questbook_scroll = self.questbook_scroll.saturating_sub(1);
        } else if input.scroll_delta < 0.0 {
            self.questbook_scroll = (self.questbook_scroll + 1).min(max_scroll);
        }

        if let Some(pos_in_cat) = cat_quests.iter().position(|&q| q == self.questbook_selected) {
            let mut moved = false;
            let mut new_pos = pos_in_cat;
            if (input.key_up && !self.prev_up_arrow_down) || input.gp_dpad_up_just_pressed {
                new_pos = (pos_in_cat + cat_quests.len() - 1) % cat_quests.len();
                moved = true;
            } else if (input.key_down && !self.prev_down_arrow_down) || input.gp_dpad_down_just_pressed {
                new_pos = (pos_in_cat + 1) % cat_quests.len();
                moved = true;
            }
            if moved {
                self.questbook_selected = cat_quests[new_pos];
                if new_pos < self.questbook_scroll {
                    self.questbook_scroll = new_pos;
                } else if new_pos >= self.questbook_scroll + crate::ui::QUESTBOOK_VISIBLE_CARDS {
                    self.questbook_scroll = new_pos + 1 - crate::ui::QUESTBOOK_VISIBLE_CARDS;
                }
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.4, 1.1);
            }
        }
        self.questbook_scroll = self.questbook_scroll.min(max_scroll);

        if just_clicked {
            if let Some(q_idx) = crate::ui::questbook_card_at(
                self.width,
                self.height,
                self.questbook_tab,
                self.questbook_scroll,
                input.mouse_pos,
            ) {
                self.questbook_selected = q_idx;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.5, 1.0);
            } else if crate::ui::questbook_claim_hit(self.width, self.height, input.mouse_pos) {
                if self.quests.claim_reward(self.questbook_selected, &mut self.player.inventory) {
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.3);
                    let extra = self.quests.sync_inventory(&self.player.inventory);
                    self.process_quest_completions(extra);
                    let _ = crate::world::storage::save_quest_progress(self.save_slot, &self.quests);
                }
            }
        }

        if (input.key_enter && !self.prev_enter_down) || input.gp_south_just_pressed {
            if self.quests.claim_reward(self.questbook_selected, &mut self.player.inventory) {
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.3);
                let extra = self.quests.sync_inventory(&self.player.inventory);
                self.process_quest_completions(extra);
                let _ = crate::world::storage::save_quest_progress(self.save_slot, &self.quests);
            }
        }

        self.renderer.fog_enabled = self.settings.fog;
        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
        let key_hint = self.questbook_key_hint();
        crate::ui::draw_questbook(
            self.renderer.pixel_buffer_mut(),
            self.width,
            self.height,
            &self.quests,
            self.questbook_tab,
            self.questbook_selected,
            self.questbook_scroll,
            key_hint,
            input.mouse_pos,
        );
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
        } else {
            find_hovered_rect(mouse_pos, &self.death_screen_rects)
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
                        Vec3::new(
                            sp.x as f32 + 0.5,
                            sp.y as f32 + 1.0 + crate::gameplay::physics::STANDING_EYE_OFFSET,
                            sp.z as f32 + 0.5,
                        )
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
                    self.death_message_sent = false;
                    self.damage_tilt_timer = 0.0;
                    let (spawn_biome, _) = crate::world::get_biome_at(spawn_pos.x.floor() as i32, spawn_pos.z.floor() as i32, self.world.seed);
                    self.biome_filter_tint = spawn_biome.color_filter();
                    crate::log_info!("PLAYER", "Player respawned at ({:.1}, {:.1}, {:.1})", self.camera.position.x, self.camera.position.y, self.camera.position.z);
                    self.state = GameState::Playing;
                    out.request_capture_mouse = true;
                }
                1 => {
                    self.player.reset(self.player.gamemode);
                    self.death_message_sent = false;
                    self.camera.position = self.world.player_position;
                    let _ = self.save_world_progress();
                    crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
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
            let _ = self.save_world_progress();
            crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
            crate::log_info!("STATE", "Quit to Main Menu from Death Screen via shortcut, game saved");
            self.state = GameState::MainMenu;
            out.request_release_mouse = true;
        }

        self.renderer.fog_enabled = self.settings.fog;
        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
        self.renderer.draw_death_screen(hovered_idx);
    }
}
