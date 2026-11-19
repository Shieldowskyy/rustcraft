//! Screen-space hit-testing for Android HUD buttons, hotbar slots, joystick, and camera drag zones.
//! Returns the `TouchRole` assigned to a newly pressed pointer coordinate.

#![allow(dead_code)]

use std::time::Instant;

use super::touch::{TouchRole, TouchTracker};

impl TouchTracker {
    pub(super) fn classify_touch_down(&self, x: f32, y: f32, w: f32, h: f32) -> TouchRole {
        // 1. Top bar buttons:
        // Menu / Pause button (center top)
        if (w * 0.5 - 45.0..=w * 0.5 + 45.0).contains(&x) && y <= 45.0 {
            return TouchRole::MenuButton;
        }
        // Inventory button (top right)
        if x >= w - 85.0 && y <= 45.0 {
            return TouchRole::InventoryButton;
        }
        // Chat button (top left)
        if (15.0..=70.0).contains(&x) && y <= 45.0 {
            return TouchRole::ChatButton;
        }
        // Fly button
        if (75.0..=130.0).contains(&x) && y <= 45.0 {
            return TouchRole::FlyButton;
        }
        // Camera perspective button
        if (135.0..=190.0).contains(&x) && y <= 45.0 {
            return TouchRole::CamButton;
        }

        // 2. Hotbar slots (bottom center)
        let slot_size = 28.0;
        let margin = 3.0;
        let total_slots = 10.0;
        let hotbar_w = total_slots * (slot_size + margin) - margin;
        let hotbar_left = (w - hotbar_w) * 0.5;
        let hotbar_top = h - slot_size - 18.0;
        if (hotbar_left - 4.0..=hotbar_left + hotbar_w + 4.0).contains(&x) && y >= hotbar_top - 6.0 {
            let slot = (((x - hotbar_left).max(0.0) / (slot_size + margin)).floor() as u8).min(9);
            return TouchRole::HotbarSlot(slot);
        }

        // Off-hand slot (to the left of hotbar)
        let offhand_x = hotbar_left - slot_size - 10.0;
        if (offhand_x - 6.0..=offhand_x + slot_size + 6.0).contains(&x) && y >= hotbar_top - 6.0 {
            return TouchRole::OffhandSlot;
        }

        // 3. Action buttons (right side, tactile circular layout)
        // Jump Button
        if (x - (w - 70.0)).powi(2) + (y - (h - 135.0)).powi(2) <= 38.0 * 38.0 {
            return TouchRole::JumpButton;
        }
        // Sprint Button
        if (x - (w - 145.0)).powi(2) + (y - (h - 135.0)).powi(2) <= 32.0 * 32.0 {
            return TouchRole::SprintButton;
        }
        // Sneak Button
        if (x - (w - 70.0)).powi(2) + (y - (h - 60.0)).powi(2) <= 32.0 * 32.0 {
            return TouchRole::SneakButton;
        }
        // Mine / Attack Button
        if (x - (w - 145.0)).powi(2) + (y - (h - 60.0)).powi(2) <= 35.0 * 35.0 {
            return TouchRole::MineButton;
        }
        // Place / Use Button
        if (x - (w - 220.0)).powi(2) + (y - (h - 60.0)).powi(2) <= 32.0 * 32.0 {
            return TouchRole::PlaceButton;
        }

        // 4. Movement Joystick (left half of screen, bottom area)
        let dpad_cx = 110.0;
        let dpad_cy = h - 110.0;
        if x < w * 0.40 && y > 120.0 {
            let dist_to_dpad = (x - dpad_cx).hypot(y - dpad_cy);
            let (ox, oy) = if dist_to_dpad <= 85.0 {
                (dpad_cx, dpad_cy)
            } else {
                (x, y)
            };
            return TouchRole::MovementJoystick { origin_x: ox, origin_y: oy };
        }

        // 5. Default right half: Camera rotation & touch-interaction
        TouchRole::CameraLook {
            start_x: x,
            start_y: y,
            last_x: x,
            last_y: y,
            start_time: Instant::now(),
            is_dragging: false,
        }
    }

    pub(super) fn apply_role_press(&mut self, role: TouchRole) {
        match role {
            TouchRole::JumpButton => self.btn_jump = true,
            TouchRole::SneakButton => self.btn_sneak = true,
            TouchRole::SprintButton => self.btn_sprint = !self.btn_sprint,
            TouchRole::MineButton => self.btn_mine = true,
            TouchRole::PlaceButton => self.btn_place_held = true,
            TouchRole::OffhandSlot => self.offhand_swap_just = true,
            TouchRole::MenuButton => self.btn_menu_just = true,
            TouchRole::InventoryButton => self.btn_inv_just = true,
            TouchRole::ChatButton => self.btn_chat_just = true,
            TouchRole::FlyButton => self.btn_fly_just = true,
            TouchRole::CamButton => self.btn_cam_just = true,
            TouchRole::HotbarSlot(slot) => self.selected_hotbar = Some(slot),
            TouchRole::CloseMenuButton => {
                self.btn_menu_just = true;
                self.btn_inv_just = true;
            }
            _ => {}
        }
    }

    pub(super) fn refresh_held_buttons(&mut self) {
        self.btn_jump = self.key_jump
            || self.roles.values().any(|role| *role == TouchRole::JumpButton);
        self.btn_sneak = self.key_sneak
            || self.roles.values().any(|role| *role == TouchRole::SneakButton);
        self.btn_place_held = self.key_place_held
            || self.roles.values().any(|role| *role == TouchRole::PlaceButton);
        self.btn_mine = self.key_mine
            || self.roles.values().any(|role| *role == TouchRole::MineButton)
            || self.roles.values().any(|role| {
                matches!(
                    role,
                    TouchRole::CameraLook {
                        start_time,
                        is_dragging: false,
                        ..
                    } if start_time.elapsed().as_secs_f32() >= 0.22
                )
            });
    }

    pub(super) fn refresh_joystick(&mut self) {
        let joystick = self.roles.iter().find_map(|(&pointer_id, role)| {
            if let TouchRole::MovementJoystick { origin_x, origin_y } = role {
                self.touches
                    .get(&pointer_id)
                    .map(|&(x, y)| (x - *origin_x, y - *origin_y))
            } else {
                None
            }
        });
        if let Some((dx, dy)) = joystick {
            self.update_joystick(dx, dy);
        } else {
            self.reset_movement();
        }
    }

    pub(super) fn reset_movement(&mut self) {
        self.move_forward = false;
        self.move_backward = false;
        self.move_left = false;
        self.move_right = false;
        self.stick_x = 0.0;
        self.stick_y = 0.0;
        self.stick_sprinting = false;
        self.refresh_held_buttons();
    }

    pub(super) fn cancel_touches(&mut self) {
        self.touches.clear();
        self.roles.clear();
        self.primary_touch_pos = None;
        self.primary_touch_down = false;
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
        self.btn_jump = false;
        self.btn_sneak = false;
        self.btn_mine = false;
        self.btn_place_held = false;
        self.tap_place_pulse = false;
        self.offhand_swap_just = false;
        self.menu_right_click_held = false;
        self.btn_menu_just = false;
        self.btn_inv_just = false;
        self.btn_chat_just = false;
        self.btn_fly_just = false;
        self.btn_cam_just = false;
        self.selected_hotbar = None;
        self.move_forward = false;
        self.move_backward = false;
        self.move_left = false;
        self.move_right = false;
        self.stick_x = 0.0;
        self.stick_y = 0.0;
        self.stick_sprinting = false;
        self.refresh_held_buttons();
    }
}
