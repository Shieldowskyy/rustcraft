use crate::app::{AppOutput, GameState, InputState, RustCraftApp};
use crate::core::math::Vec3;

impl RustCraftApp {
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
                    crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
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
                                crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
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
                                    crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
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
                                crate::ui::minimap::save_waypoints(self.save_slot, self.world.seed, &self.minimap.waypoints);
                                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 0.9);
                            }
                        }
                    }
                }
            }
        } else {
            if self.worldmap.context_menu.is_some() {
                if escape_down {
                    self.worldmap.context_menu = None;
                    self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.0);
                    return;
                }
            } else if escape_down || m_down {
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
            let left_released = !left_down && self.prev_left_down;

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

            // Context menu interaction
            let mut context_menu_handled = false;
            if let Some(ctx) = self.worldmap.context_menu.clone() {
                if left_click {
                    let (_card, item0, item1) = crate::ui::minimap::worldmap_context_menu_rects(
                        ctx.screen_pos.0,
                        ctx.screen_pos.1,
                        self.width,
                        self.height,
                    );
                    if let Some((mx, my)) = mouse_pos {
                        let px = mx as i32;
                        let py = my as i32;
                        if px >= item0.0 && px < item0.0 + item0.2 && py >= item0.1 && py < item0.1 + item0.3 {
                            // Option 0: Create waypoint here
                            self.worldmap.context_menu = None;
                            self.worldmap.creating_waypoint = true;
                            self.worldmap.waypoint_name_input = format!("Waypoint {}", self.minimap.waypoints.len() + 1);
                            self.worldmap.waypoint_coords_input = ctx.world_pos;
                            self.worldmap.waypoint_color_idx = (self.minimap.waypoints.len()) % crate::ui::minimap::WAYPOINT_COLORS.len();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                            context_menu_handled = true;
                        } else if px >= item1.0 && px < item1.0 + item1.2 && py >= item1.1 && py < item1.1 + item1.3 {
                            // Option 1: Share coordinates in chat
                            self.worldmap.context_menu = None;
                            let (hx, hy, hz) = ctx.world_pos;
                            let chat_msg = format!("Moje koordynaty: X: {}, Y: {}, Z: {}", hx, hy, hz);
                            if let Some(client) = &mut self.net_client {
                                client.send_chat(&chat_msg);
                            } else {
                                self.chat.add_player_chat(&self.settings.nickname, &chat_msg);
                            }
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                            context_menu_handled = true;
                        } else {
                            // Clicked outside context menu -> close it
                            self.worldmap.context_menu = None;
                            context_menu_handled = true;
                        }
                    }
                }
            }

            // Right-click: Open context menu with options (Zrób tutaj waypointa, Udostępnij koordynaty na czacie)
            if right_click && !top_bar_clicked {
                if let Some((mx, my)) = mouse_pos {
                    if my >= 28.0 && my < (self.height as f32 - 26.0) {
                        let coords = if let Some((hx, hy, hz)) = self.worldmap.hovered_world_pos {
                            (hx, hy, hz)
                        } else {
                            (self.camera.position.x.round() as i32, self.camera.position.y.round() as i32, self.camera.position.z.round() as i32)
                        };
                        let chunk = self.worldmap.hovered_chunk.unwrap_or((coords.0.div_euclid(16), coords.2.div_euclid(16)));
                        self.worldmap.context_menu = Some(crate::ui::minimap::WorldMapContextMenu {
                            screen_pos: (mx as i32, my as i32),
                            world_pos: coords,
                            chunk,
                        });
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.1);
                    }
                }
            }

            // Left-click chunk selection toggling & mouse dragging
            if let Some((mx, my)) = mouse_pos {
                if left_click && !top_bar_clicked && !context_menu_handled && my >= 28.0 && my < (self.height as f32 - 26.0) {
                    self.worldmap.drag_start_mouse = (mx, my);
                    self.worldmap.drag_start_center = (self.worldmap.center_x, self.worldmap.center_z);
                    self.worldmap.is_dragging = false;
                } else if left_down && !top_bar_clicked && !context_menu_handled && my >= 28.0 && my < (self.height as f32 - 26.0) {
                    let dx = mx - self.worldmap.drag_start_mouse.0;
                    let dy = my - self.worldmap.drag_start_mouse.1;
                    if dx.abs() + dy.abs() > 4.0 {
                        self.worldmap.is_dragging = true;
                        self.worldmap.center_x = self.worldmap.drag_start_center.0 - dx / self.worldmap.zoom;
                        self.worldmap.center_z = self.worldmap.drag_start_center.1 - dy / self.worldmap.zoom;
                    }
                } else if left_released {
                    if !self.worldmap.is_dragging && !top_bar_clicked && !context_menu_handled && my >= 28.0 && my < (self.height as f32 - 26.0) {
                        // Left-click selects chunk!
                        if let Some(ch) = self.worldmap.hovered_chunk {
                            if self.worldmap.selected_chunks.contains(&ch) {
                                self.worldmap.selected_chunks.remove(&ch);
                            } else {
                                self.worldmap.selected_chunks.insert(ch);
                            }
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                        }
                    }
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
            self.settings.language,
        );
    }
}
