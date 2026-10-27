use crate::app::{AppOutput, GameState, InputState, PreviousMenu, RustCraftApp};

impl RustCraftApp {
    pub(crate) fn update_and_render_controls_rebind(
        &mut self,
        prev: PreviousMenu,
        dt: f32,
        input: &InputState,
        _mouse_moved: bool,
        _out: &mut AppOutput,
    ) {
        self.rebind_cursor_timer += dt;
        let layout = crate::ui::controls_ui::ControlsRebindLayout::new(self.width, self.height);
        let lang = self.settings.language;
        let mouse_pos = input.mouse_pos;
        let left_just = input.mouse_left && !self.prev_left_down;
        let escape_just = input.key_escape && !self.prev_escape_down;
        let cx = self.width as i32 / 2;

        // If currently waiting for user to press a key/mouse/pad input to rebind
        if let Some((target_action, target_slot)) = self.rebinding_target {
            if escape_just {
                self.rebinding_target = None;
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            } else {
                match target_slot {
                    crate::ui::controls_ui::RebindSlot::KeyOrMouse => {
                        // Check for any keyboard key
                        let mut captured_key = None;
                        for &k in &input.keys_down {
                            if k != crate::core::controls::KeyCode::Escape {
                                captured_key = Some(k);
                                break;
                            }
                        }

                        // Check for mouse buttons / wheel
                        let captured_mouse = if input.mouse_left && !self.prev_left_down {
                            Some(crate::core::controls::MouseInput::Left)
                        } else if input.mouse_right && !self.prev_right_down {
                            Some(crate::core::controls::MouseInput::Right)
                        } else if input.mouse_middle {
                            Some(crate::core::controls::MouseInput::Middle)
                        } else if input.mouse_4 {
                            Some(crate::core::controls::MouseInput::Mouse4)
                        } else if input.mouse_5 {
                            Some(crate::core::controls::MouseInput::Mouse5)
                        } else if input.mouse_extra[0] {
                            Some(crate::core::controls::MouseInput::Mouse6)
                        } else if input.mouse_extra[1] {
                            Some(crate::core::controls::MouseInput::Mouse7)
                        } else if input.mouse_extra[2] {
                            Some(crate::core::controls::MouseInput::Mouse8)
                        } else if input.scroll_delta > 0.1 {
                            Some(crate::core::controls::MouseInput::WheelUp)
                        } else if input.scroll_delta < -0.1 {
                            Some(crate::core::controls::MouseInput::WheelDown)
                        } else {
                            None
                        };

                        if let Some(mouse_btn) = captured_mouse {
                            self.controls.config.rebind_mouse(target_action, mouse_btn);
                            let _ = self.controls.config.save();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.2);
                            self.rebinding_target = None;
                        } else if let Some(key) = captured_key {
                            self.controls.config.rebind_key(target_action, key);
                            let _ = self.controls.config.save();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.2);
                            self.rebinding_target = None;
                        }
                    }
                    crate::ui::controls_ui::RebindSlot::Gamepad => {
                        let mut captured_pad = None;
                        if input.gp_south_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::South);
                        } else if input.gp_east_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::East);
                        } else if input.gp_west_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::West);
                        } else if input.gp_north_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::North);
                        } else if input.gp_btn_lb {
                            captured_pad = Some(crate::core::controls::GamepadInput::LB);
                        } else if input.gp_btn_rb {
                            captured_pad = Some(crate::core::controls::GamepadInput::RB);
                        } else if input.gp_dpad_up_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::DpadUp);
                        } else if input.gp_dpad_down_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::DpadDown);
                        } else if input.gp_dpad_left_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::DpadLeft);
                        } else if input.gp_dpad_right_just_pressed {
                            captured_pad = Some(crate::core::controls::GamepadInput::DpadRight);
                        }

                        if let Some(pad_btn) = captured_pad {
                            self.controls.config.rebind_gamepad(target_action, pad_btn);
                            let _ = self.controls.config.save();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.8, 1.2);
                            self.rebinding_target = None;
                        } else if left_just {
                            // Clicking with mouse cancels gamepad rebind
                            self.rebinding_target = None;
                        }
                    }
                }
            }
        } else {
            // Normal navigation & typing
            if escape_just {
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                self.state = GameState::Settings(prev);
                return;
            }

            // Search typing: characters, backspace, delete
            let mut search_changed = false;
            for ch in &input.typed_chars {
                if !ch.is_control() && self.rebind_search.len() < 32 {
                    self.rebind_search.push(*ch);
                    search_changed = true;
                }
            }
            if input.key_backspace && !self.prev_backspace_down {
                self.rebind_search.pop();
                search_changed = true;
            }
            if input.key_del && !self.prev_del_down {
                self.rebind_search.clear();
                search_changed = true;
            }
            if search_changed {
                self.rebind_scroll = 0;
                self.rebind_search_focused = true;
            }
        }

        // Compute filtered actions with latest search query
        let filtered_actions = crate::ui::controls_ui::filter_actions(&self.rebind_search, &self.controls.config, lang);

        if self.rebinding_target.is_none() {
            // Mouse wheel scrolling
            if input.scroll_delta > 0.1 {
                self.rebind_scroll = self.rebind_scroll.saturating_sub(1);
            } else if input.scroll_delta < -0.1 {
                let max_scroll = filtered_actions.len().saturating_sub(crate::ui::controls_ui::VISIBLE_REBIND_ROWS);
                if self.rebind_scroll < max_scroll {
                    self.rebind_scroll += 1;
                }
            }

            // Click handling
            if left_just {
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;

                    // Search box focus
                    let (sx, sy, sw, sh) = layout.search_box;
                    if px >= sx && px < sx + sw && py >= sy && py < sy + sh {
                        self.rebind_search_focused = true;
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.2);
                    } else {
                        self.rebind_search_focused = false;
                    }

                    // Search clear button
                    let (scx, scy, scw, sch) = layout.search_clear_btn;
                    if px >= scx && px < scx + scw && py >= scy && py < scy + sch {
                        self.rebind_search.clear();
                        self.rebind_scroll = 0;
                        self.rebind_search_focused = true;
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    }

                    // Reset all button
                    let (rx, ry, rw, rh) = layout.reset_all_btn;
                    if px >= rx && px < rx + rw && py >= ry && py < ry + rh {
                        self.controls.config.reset_to_defaults();
                        let _ = self.controls.config.save();
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                    }

                    // Back button
                    let (bx, by, bw, bh) = layout.back_btn;
                    if px >= bx && px < bx + bw && py >= by && py < by + bh {
                        let _ = self.controls.config.save();
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                        self.state = GameState::Settings(prev);
                        return;
                    }

                    // Row items
                    for row_idx in 0..crate::ui::controls_ui::VISIBLE_REBIND_ROWS {
                        let action_idx = self.rebind_scroll + row_idx;
                        if action_idx >= filtered_actions.len() {
                            break;
                        }
                        let action = filtered_actions[action_idx];
                        let (key_btn, pad_btn, reset_btn) = layout.row_buttons(cx, row_idx);

                        if px >= key_btn.0 && px < key_btn.0 + key_btn.2 && py >= key_btn.1 && py < key_btn.1 + key_btn.3 {
                            self.rebinding_target = Some((action, crate::ui::controls_ui::RebindSlot::KeyOrMouse));
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                            break;
                        } else if px >= pad_btn.0 && px < pad_btn.0 + pad_btn.2 && py >= pad_btn.1 && py < pad_btn.1 + pad_btn.3 {
                            self.rebinding_target = Some((action, crate::ui::controls_ui::RebindSlot::Gamepad));
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.2);
                            break;
                        } else if px >= reset_btn.0 && px < reset_btn.0 + reset_btn.2 && py >= reset_btn.1 && py < reset_btn.1 + reset_btn.3 {
                            let def = crate::core::controls::ControlsConfig::default();
                            self.controls.config.bindings[action.index()] = def.bindings[action.index()].clone();
                            let _ = self.controls.config.save();
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                            break;
                        }
                    }
                }
            }
        }

        // Hover calculation
        let mut hovered_row = None;
        let mut hovered_col = None;
        let mut hovered_btn = None;

        if let Some((mx, my)) = mouse_pos {
            let px = mx as i32;
            let py = my as i32;

            let (rx, ry, rw, rh) = layout.reset_all_btn;
            let (bx, by, bw, bh) = layout.back_btn;
            let (scx, scy, scw, sch) = layout.search_clear_btn;

            if px >= rx && px < rx + rw && py >= ry && py < ry + rh {
                hovered_btn = Some(0);
            } else if px >= bx && px < bx + bw && py >= by && py < by + bh {
                hovered_btn = Some(1);
            } else if px >= scx && px < scx + scw && py >= scy && py < scy + sch {
                hovered_btn = Some(2);
            }

            for row_idx in 0..crate::ui::controls_ui::VISIBLE_REBIND_ROWS {
                let action_idx = self.rebind_scroll + row_idx;
                if action_idx >= filtered_actions.len() {
                    break;
                }
                let (k_btn, gp_btn, reset_btn) = layout.row_buttons(cx, row_idx);
                if px >= k_btn.0 && px < k_btn.0 + k_btn.2 && py >= k_btn.1 && py < k_btn.1 + k_btn.3 {
                    hovered_row = Some(row_idx);
                    hovered_col = Some(0);
                    break;
                } else if px >= gp_btn.0 && px < gp_btn.0 + gp_btn.2 && py >= gp_btn.1 && py < gp_btn.1 + gp_btn.3 {
                    hovered_row = Some(row_idx);
                    hovered_col = Some(1);
                    break;
                } else if px >= reset_btn.0 && px < reset_btn.0 + reset_btn.2 && py >= reset_btn.1 && py < reset_btn.1 + reset_btn.3 {
                    hovered_row = Some(row_idx);
                    hovered_col = Some(2);
                    break;
                }
            }
        }

        // Render controls rebinding menu
        crate::ui::controls_ui::draw_controls_rebind_menu(
            &mut self.renderer.color_buffer,
            self.width,
            self.height,
            &self.controls.config,
            &self.rebind_search,
            self.rebind_scroll,
            self.rebinding_target,
            hovered_row,
            hovered_col,
            hovered_btn,
            lang,
            self.rebind_cursor_timer,
            self.is_gamepad_active,
            self.rebind_search_focused,
        );
    }
}
