use super::find_hovered_rect;
use crate::app::{AppOutput, GameState, InputState, PreviousMenu, RustCraftApp};

impl RustCraftApp {
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
        } else if mouse_pos.is_some() {
            let tab = find_hovered_rect(mouse_pos, &self.settings_tab_rects);
            let opt = if tab.is_none() {
                let rects: &[(i32, i32, i32, i32)] = match self.settings_tab {
                    0 => &self.settings_rects,
                    1 => &self.settings_audio_rects,
                    2 => &self.settings_controls_rects,
                    _ => &self.settings_debug_rects,
                };
                find_hovered_rect(mouse_pos, rects)
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
            2 => 11,
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
            crate::log_debug!(
                "SETTINGS",
                "Switched to tab {} ({})",
                self.settings_tab,
                match self.settings_tab {
                    0 => "Graphics",
                    1 => "Audio",
                    2 => "Controls",
                    _ => "Debug",
                }
            );
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

        // Unified option action: (opt_idx, forward, is_click)
        let opt_action = if let Some(opt_idx) = click_opt {
            Some((opt_idx, click_direction_forward, true))
        } else if (input.gp_dpad_left_just_pressed || left_key_just) && hovered_opt.is_some() {
            Some((hovered_opt.unwrap(), false, false))
        } else if (input.gp_dpad_right_just_pressed || right_key_just) && hovered_opt.is_some() {
            Some((hovered_opt.unwrap(), true, false))
        } else {
            None
        };

        if let Some((opt_idx, forward, is_click)) = opt_action {
            match self.settings_tab {
                0 => {
                    // GRAPHICS TAB
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    match opt_idx {
                        0 => {
                            self.settings.cycle_language(forward);
                            crate::core::i18n::set_language(self.settings.language);
                            crate::log_info!("SETTINGS", "Language changed to {:?}", self.settings.language);
                        }
                        1 => {
                            self.settings.cycle_render_distance(forward);
                            self.renderer.render_distance = self.settings.render_distance;
                            self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                            self.renderer.clear_mesh_cache();
                            crate::log_info!("SETTINGS", "Render distance changed to {} chunks", self.settings.render_distance);
                        }
                        2 => {
                            self.settings.cycle_leaf_quality(forward);
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
                            self.settings.cycle_fov(forward);
                            crate::log_info!("SETTINGS", "FOV changed to {} deg", self.settings.fov_deg);
                        }
                        8 => {
                            self.settings.cycle_brightness(forward);
                            self.renderer.brightness = self.settings.brightness;
                            crate::log_info!("SETTINGS", "Brightness changed to {:.2}", self.settings.brightness);
                        }
                        9 => {
                            self.settings.cycle_msaa(forward);
                            crate::log_info!("SETTINGS", "MSAA changed to {}", self.settings.msaa);
                        }
                        10 => {
                            self.settings.cycle_fps_limit(forward);
                            crate::log_info!("SETTINGS", "FPS limit changed to {}", self.settings.fps_limit);
                        }
                        11 => {
                            self.settings.underwater_fx = !self.settings.underwater_fx;
                            crate::log_info!("SETTINGS", "Underwater FX toggled to {}", self.settings.underwater_fx);
                        }
                        12 => {
                            if is_click {
                                let _ = self.settings.save();
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                        }
                        _ => {}
                    }
                }
                1 => {
                    // AUDIO TAB
                    match opt_idx {
                        0 => {
                            self.settings.cycle_master_volume(forward);
                            self.audio.set_master_volume(self.settings.master_volume);
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            crate::log_info!("SETTINGS", "Master volume changed to {:.0}%", self.settings.master_volume * 100.0);
                        }
                        1 => {
                            self.settings.cycle_sfx_volume(forward);
                            self.audio.set_sfx_volume(self.settings.sfx_volume);
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            crate::log_info!("SETTINGS", "SFX volume changed to {:.0}%", self.settings.sfx_volume * 100.0);
                        }
                        2 => {
                            self.settings.cycle_music_volume(forward);
                            self.audio.set_music_volume(self.settings.music_volume);
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            crate::log_info!("SETTINGS", "Music volume changed to {:.0}%", self.settings.music_volume * 100.0);
                        }
                        3 => {
                            self.settings.cycle_ambient_volume(forward);
                            self.audio.set_ambient_volume(self.settings.ambient_volume);
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            crate::log_info!("SETTINGS", "Ambient volume changed to {:.0}%", self.settings.ambient_volume * 100.0);
                        }
                        4 => {
                            self.settings.cycle_ui_volume(forward);
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
                            if is_click {
                                let _ = self.settings.save();
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                        }
                        _ => {}
                    }
                }
                2 => {
                    // CONTROLS TAB
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    match opt_idx {
                        0 => {
                            self.settings.cycle_sprint_mode(forward);
                            crate::log_info!("SETTINGS", "Sprint mode changed to {:?}", self.settings.sprint_mode);
                        }
                        1 => {
                            self.settings.cycle_sneak_mode(forward);
                            crate::log_info!("SETTINGS", "Sneak mode changed to {:?}", self.settings.sneak_mode);
                        }
                        2 => {
                            self.settings.cycle_jump_mode(forward);
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
                            self.settings.cycle_mouse_sensitivity(forward);
                            crate::log_info!("SETTINGS", "Mouse sensitivity changed to {:.1}x", self.settings.mouse_sensitivity);
                        }
                        6 => {
                            self.settings.cycle_gamepad_sensitivity(forward);
                            crate::log_info!("SETTINGS", "Gamepad sensitivity changed to {:.1}x", self.settings.gamepad_sensitivity);
                        }
                        7 => {
                            self.settings.toggle_vibration();
                            if self.settings.vibration {
                                self.request_rumble(0.5, 0.5, 120);
                            }
                            crate::log_info!("SETTINGS", "Gamepad vibration changed to {}", self.settings.vibration);
                        }
                        8 => {
                            self.settings.toggle_damage_tilt();
                            crate::log_info!("SETTINGS", "Damage tilt changed to {}", self.settings.damage_tilt);
                        }
                        9 => {
                            if is_click {
                                #[cfg(not(any(target_os = "vita", target_os = "xbox360", target_os = "xenon", target_os = "android")))]
                                {
                                    self.rebind_search.clear();
                                    self.rebind_scroll = 0;
                                    self.rebinding_target = None;
                                    self.rebind_cursor_timer = 0.0;
                                    self.state = GameState::ControlsRebind(prev);
                                    crate::log_info!("SETTINGS", "Opened Controls Rebinding menu");
                                }
                            }
                        }
                        10 => {
                            if is_click {
                                let _ = self.settings.save();
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    // DEBUG TAB
                    match opt_idx {
                        0 => {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            self.settings.toggle_profiler_server();
                            crate::log_info!("SETTINGS", "Profiler server toggled: {}", self.settings.profiler_server);
                            if self.settings.profiler_server {
                                self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                            } else if let Some(mut s) = self.profiler_server.take() {
                                s.stop();
                            }
                        }
                        1 => {
                            if is_click {
                                let server_ip = self.profiler_server.as_ref().map(|s| s.ip_address.as_str()).unwrap_or("127.0.0.1");
                                let url = format!("http://{}:{}", server_ip, self.settings.profiler_server_port);
                                crate::ui::clipboard::set_clipboard(&url);
                                self.profiler_copied_timer = 2.5;
                                crate::log_info!("SETTINGS", "Copied profiler server URL: {}", url);
                            }
                        }
                        2 => {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            self.settings.cycle_profiler_server_port(forward);
                            crate::log_info!("SETTINGS", "Profiler server port changed to {}", self.settings.profiler_server_port);
                            if self.settings.profiler_server {
                                if let Some(mut s) = self.profiler_server.take() {
                                    s.stop();
                                }
                                self.profiler_server = crate::core::profiler_server::ProfilerServer::start(self.settings.profiler_server_port).ok();
                            }
                        }
                        3 => {
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            self.settings.debug_overlay = !self.settings.debug_overlay;
                            crate::log_info!("SETTINGS", "Debug overlay toggled to {}", self.settings.debug_overlay);
                        }
                        4 => {
                            if is_click {
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                                let _ = self.settings.save();
                                crate::log_info!("SETTINGS", "Saved settings and returned to {:?}", prev);
                                self.state = match prev {
                                    PreviousMenu::MainMenu => GameState::MainMenu,
                                    PreviousMenu::Paused => GameState::Paused,
                                    _ => GameState::MainMenu,
                                };
                            }
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
}
