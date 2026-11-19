//! Ekran pauzy i ekran ładowania świata (`update_and_render_loading`, `update_and_render_paused`).

use super::find_hovered_rect;
use crate::app::{AppOutput, GameState, InputState, PreviousMenu, RustCraftApp, WorldLoadResult};
use crate::core::controls::Action;
use crate::core::profiler::FrameProfiler;
use crate::features::entities::MobManager;
use crate::features::player::Camera;
use crate::features::world::get_save_metadata;

impl RustCraftApp {
    pub(crate) fn update_and_render_loading(
        &mut self,
        result: Option<WorldLoadResult>,
        is_sync: bool,
    ) {
        let splash_text = crate::ui::splashes::splash_text(self.splash_index);
        self.renderer.draw_main_menu(
            None,
            &self.settings.nickname,
            false,
            self.is_gamepad_active,
            splash_text,
            self.total_time,
        );

        if is_sync {
            self.state = GameState::Playing;
            return;
        }

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
                    self.minimap.waypoints =
                        crate::ui::minimap::load_waypoints(self.save_slot, self.world.seed);
                    self.state = GameState::Playing;
                    crate::log_info!(
                        "STATE",
                        "Entered world '{}' (seed: {}, mode: {:?}) at position ({:.1}, {:.1}, {:.1})",
                        self.world.name,
                        self.world.seed,
                        self.player.gamemode,
                        start_pos.x,
                        start_pos.y,
                        start_pos.z
                    );
                }
                WorldLoadResult::Error(e) => {
                    eprintln!("Failed to load world: {e}");
                    crate::log_error!(
                        "STATE",
                        "Failed to load world: {}, returning to Main Menu",
                        e
                    );
                    self.state = GameState::MainMenu;
                }
            }
        }
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
        self.camera.fov_y += ((self.settings.fov_deg as f32).to_radians() - self.camera.fov_y) * t_fov;
        let aspect = self.width as f32 / self.height as f32;
        if (self.camera.fov_y - self.last_fov).abs() > 1e-4
            || (aspect - self.last_aspect).abs() > 1e-4
        {
            self.proj_matrix = self.camera.projection_matrix(aspect);
            self.last_fov = self.camera.fov_y;
            self.last_aspect = aspect;
        }

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

        if input.gp_dpad_up_just_pressed
            || input.gp_menu_up
            || (up_arrow_down && !self.prev_up_arrow_down)
        {
            let next = match self.pause_hovered_btn {
                Some(0) | None => 6,
                Some(i) => i - 1,
            };
            self.pause_hovered_btn = Some(next);
            hovered_idx = Some(next);
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.5, 1.15);
        }
        if input.gp_dpad_down_just_pressed
            || input.gp_menu_down
            || (down_arrow_down && !self.prev_down_arrow_down)
        {
            let next = match self.pause_hovered_btn {
                Some(6) | None => 0,
                Some(i) => i + 1,
            };
            self.pause_hovered_btn = Some(next);
            hovered_idx = Some(next);
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.5, 1.15);
        }

        let click_pause_btn = if left_down && !self.prev_left_down {
            mouse_hovered_btn
        } else if input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
            hovered_idx
        } else {
            None
        };

        if let Some(btn) = click_pause_btn {
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            match btn {
                0 => {
                    self.state = GameState::Playing;
                    out.request_capture_mouse = true;
                    crate::log_info!("STATE", "Resumed game from Pause Menu");
                }
                1 => {
                    self.settings_hovered_opt = Some(0);
                    self.state = GameState::Settings(PreviousMenu::Paused);
                    crate::log_info!("MENU", "Opened Settings from Pause Menu");
                }
                2 => {
                    self.fly_mode = !self.fly_mode;
                    self.vertical_velocity = 0.0;
                    crate::log_info!("GAMEPLAY", "Toggled Fly Mode: {}", self.fly_mode);
                }
                3 => {
                    self.state = GameState::AchievementTree;
                    crate::log_info!("MENU", "Opened Achievement Tree from Pause Menu");
                }
                4 => {
                    if self.is_hosting_multiplayer {
                        if let Some(invite) = &self.multiplayer_invite {
                            crate::platform::clipboard::set_clipboard(invite);
                            self.multiplayer_status = Some(
                                crate::core::i18n::t("multiplayer.invite_copied_status")
                                    .to_string(),
                            );
                            self.show_multiplayer_toast(
                                crate::core::i18n::t("multiplayer.invite_copied_toast"),
                                false,
                            );
                        }
                    } else if !self.is_multiplayer {
                        if let Err(error) = self.start_host_multiplayer() {
                            self.multiplayer_status = Some(error.clone());
                            self.show_multiplayer_toast(error, true);
                        }
                    }
                }
                5 => {
                    if self.is_multiplayer {
                        if let Some(client) = &mut self.net_client {
                            client.disconnect();
                        }
                        self.net_client = None;
                        self.is_multiplayer = false;
                        self.stop_host_multiplayer();
                        self.pending_network_events.clear();
                    }
                    let _ = self.save_world_progress();
                    crate::ui::minimap::save_waypoints(
                        self.save_slot,
                        self.world.seed,
                        &self.minimap.waypoints,
                    );
                    self.save_metadata_cache =
                        std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    self.state = GameState::MainMenu;
                    out.request_release_mouse = true;
                    crate::log_info!("STATE", "Saved world and returned to Main Menu");
                }
                6 => {
                    let _ = self.save_world_progress();
                    crate::ui::minimap::save_waypoints(
                        self.save_slot,
                        self.world.seed,
                        &self.minimap.waypoints,
                    );
                    let _ = self.settings.save();
                    out.quit_requested = true;
                    crate::log_info!("STATE", "Saved world and requested application exit");
                }
                _ => {}
            }
        }

        if self.controls.is_just_pressed(Action::Pause)
            || input.gp_east_just_pressed
            || input.gp_start_just_pressed
        {
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            self.state = GameState::Playing;
            out.request_capture_mouse = true;
            crate::log_info!("STATE", "Resumed game via Pause/Back shortcut");
        }
        if q_down && !self.prev_q_down {
            let _ = self.save_world_progress();
            crate::ui::minimap::save_waypoints(
                self.save_slot,
                self.world.seed,
                &self.minimap.waypoints,
            );
            let _ = self.settings.save();
            out.quit_requested = true;
            crate::log_info!("STATE", "Saved and requested exit via Q shortcut");
        }
        if m_down && !self.prev_m_down {
            if self.is_multiplayer {
                if let Some(client) = &mut self.net_client {
                    client.disconnect();
                }
                self.net_client = None;
                self.is_multiplayer = false;
                self.stop_host_multiplayer();
                self.pending_network_events.clear();
            }
            let _ = self.save_world_progress();
            crate::ui::minimap::save_waypoints(
                self.save_slot,
                self.world.seed,
                &self.minimap.waypoints,
            );
            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
            self.state = GameState::MainMenu;
            out.request_release_mouse = true;
            crate::log_info!("STATE", "Saved and returned to Main Menu via shortcut");
        }

        self.renderer.fog_enabled = self.settings.fog;
        self.render_menu_world_backdrop();
        self.renderer.draw_pause_menu(
            self.fly_mode,
            hovered_idx,
            self.is_gamepad_active,
            !self.is_multiplayer || self.is_hosting_multiplayer,
            self.is_hosting_multiplayer,
            self.multiplayer_status.as_deref(),
        );

        if !crate::platform::capabilities().uses_hardware_gxm {
            if let Some(ref server) = self.profiler_server {
                let mut cur_profiler = FrameProfiler::default();
                cur_profiler.fps = (1.0 / dt.max(0.001)).round() as u32;
                let snap = self.build_profiler_snapshot(&cur_profiler);
                server.update_snapshot(snap);
            }
        }
    }
}
