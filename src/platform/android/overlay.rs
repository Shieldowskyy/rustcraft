//! Semi-transparent virtual joystick and touch action button overlay renderer for Android.
//! Draws HUD controls directly onto the ARGB framebuffer pixel buffer.

#![allow(dead_code)]

use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect_alpha, pack_color};

use super::touch::TouchTracker;

impl TouchTracker {
    /// Renders sleek, semi-transparent touch overlay controls directly onto the color buffer.
    pub fn draw_overlay(&self, buffer: &mut [u32], width: usize, height: usize, in_menu: bool) {
        let w_i32 = width as i32;
        let h_i32 = height as i32;

        if in_menu {
            // Draw a convenient "CLOSE [X]" button at top right in inventory/menus
            fill_rect_alpha(buffer, width, height, w_i32 - 75, 8, 65, 26, 0x000000, 0.65);
            draw_text_shadowed(buffer, width, height, "CLOSE", w_i32 - 62, 14, 1, 1);

            // Draw hint for 2-finger touch (right click / split stack / quick equip)
            fill_rect_alpha(buffer, width, height, 12, 8, 155, 22, 0x000000, 0.45);
            draw_text_shadowed(buffer, width, height, "2-FINGER: SPLIT/EQUIP", 16, 13, 1, 1);
            return;
        }

        let active_green = pack_color(0.35, 0.95, 0.35);
        let active_yellow = pack_color(1.0, 0.85, 0.3);
        let active_blue = pack_color(0.4, 0.8, 1.0);

        // --- Top Bar Buttons ---
        // Pause / Menu (Center)
        fill_rect_alpha(buffer, width, height, w_i32 / 2 - 35, 8, 70, 26, 0x000000, 0.50);
        draw_text_shadowed(buffer, width, height, "PAUSE", w_i32 / 2 - 20, 14, 1, 1);

        // Inventory (Right)
        fill_rect_alpha(buffer, width, height, w_i32 - 70, 8, 55, 26, 0x000000, 0.50);
        draw_text_shadowed(buffer, width, height, "INV", w_i32 - 48, 14, 1, 1);

        // Chat (Left)
        fill_rect_alpha(buffer, width, height, 15, 8, 55, 26, 0x000000, 0.50);
        draw_text_shadowed(buffer, width, height, "CHAT", 25, 14, 1, 1);

        // Fly
        fill_rect_alpha(buffer, width, height, 75, 8, 55, 26, 0x000000, 0.50);
        draw_text_shadowed(buffer, width, height, "FLY", 90, 14, 1, 1);

        // Camera perspective (F5)
        fill_rect_alpha(buffer, width, height, 135, 8, 55, 26, 0x000000, 0.50);
        draw_text_shadowed(buffer, width, height, "CAM", 150, 14, 1, 1);

        // --- Action Buttons (Right) ---
        // Jump Button
        let jump_cx = w_i32 - 70;
        let jump_cy = h_i32 - 135;
        fill_circle_alpha(buffer, width, height, jump_cx, jump_cy, 34, 0x000000, 0.55);
        if self.btn_jump {
            fill_circle_alpha(buffer, width, height, jump_cx, jump_cy, 30, 0x15803d, 0.70);
            draw_text_colored_shadowed(buffer, width, height, "JUMP", jump_cx - 16, jump_cy - 4, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "JUMP", jump_cx - 16, jump_cy - 4, 1, 1);
        }

        // Sprint Toggle Button
        let sprint_cx = w_i32 - 145;
        let sprint_cy = h_i32 - 135;
        fill_circle_alpha(buffer, width, height, sprint_cx, sprint_cy, 28, 0x000000, 0.55);
        if self.btn_sprint {
            fill_circle_alpha(buffer, width, height, sprint_cx, sprint_cy, 24, 0x0284c7, 0.70);
            draw_text_colored_shadowed(buffer, width, height, "RUN", sprint_cx - 12, sprint_cy - 4, 1, 1, active_blue);
        } else {
            draw_text_shadowed(buffer, width, height, "RUN", sprint_cx - 12, sprint_cy - 4, 1, 1);
        }

        // Sneak Button
        let sneak_cx = w_i32 - 70;
        let sneak_cy = h_i32 - 60;
        fill_circle_alpha(buffer, width, height, sneak_cx, sneak_cy, 28, 0x000000, 0.55);
        if self.btn_sneak {
            fill_circle_alpha(buffer, width, height, sneak_cx, sneak_cy, 24, 0x15803d, 0.70);
            draw_text_colored_shadowed(buffer, width, height, "SNEAK", sneak_cx - 18, sneak_cy - 4, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "SNEAK", sneak_cx - 18, sneak_cy - 4, 1, 1);
        }

        // Mine / Attack Button
        let mine_cx = w_i32 - 145;
        let mine_cy = h_i32 - 60;
        fill_circle_alpha(buffer, width, height, mine_cx, mine_cy, 34, 0x000000, 0.55);
        if self.btn_mine {
            fill_circle_alpha(buffer, width, height, mine_cx, mine_cy, 30, 0xb45309, 0.70);
            draw_text_colored_shadowed(buffer, width, height, "MINE", mine_cx - 16, mine_cy - 4, 1, 1, active_yellow);
        } else {
            draw_text_shadowed(buffer, width, height, "MINE", mine_cx - 16, mine_cy - 4, 1, 1);
        }

        // Place / Use Button
        let place_cx = w_i32 - 220;
        let place_cy = h_i32 - 60;
        fill_circle_alpha(buffer, width, height, place_cx, place_cy, 28, 0x000000, 0.55);
        if self.btn_place_held {
            fill_circle_alpha(buffer, width, height, place_cx, place_cy, 24, 0xb45309, 0.70);
            draw_text_colored_shadowed(buffer, width, height, "USE", place_cx - 12, place_cy - 4, 1, 1, active_yellow);
        } else {
            draw_text_shadowed(buffer, width, height, "USE", place_cx - 12, place_cy - 4, 1, 1);
        }

        // --- Movement Virtual Joystick (Left) ---
        let dpad_cx = 110;
        let dpad_cy = h_i32 - 110;

        // Base ring
        fill_circle_alpha(buffer, width, height, dpad_cx, dpad_cy, 52, 0x000000, 0.45);
        fill_circle_alpha(buffer, width, height, dpad_cx, dpad_cy, 50, 0x222222, 0.35);

        // Direction indicators
        draw_text_shadowed(buffer, width, height, "^", dpad_cx - 4, dpad_cy - 42, 1, 1);
        draw_text_shadowed(buffer, width, height, "v", dpad_cx - 4, dpad_cy + 30, 1, 1);
        draw_text_shadowed(buffer, width, height, "<", dpad_cx - 40, dpad_cy - 6, 1, 1);
        draw_text_shadowed(buffer, width, height, ">", dpad_cx + 32, dpad_cy - 6, 1, 1);

        // Dynamic analog thumb knob
        let knob_x = dpad_cx + (self.stick_x * 32.0) as i32;
        let knob_y = dpad_cy + (self.stick_y * 32.0) as i32;
        let is_moving = self.stick_x.abs() > 0.1 || self.stick_y.abs() > 0.1;
        if is_moving {
            fill_circle_alpha(buffer, width, height, knob_x, knob_y, 22, 0x16a34a, 0.75);
            fill_circle_alpha(buffer, width, height, knob_x, knob_y, 16, 0x4ade80, 0.90);
        } else {
            fill_circle_alpha(buffer, width, height, knob_x, knob_y, 20, 0x333333, 0.65);
            fill_circle_alpha(buffer, width, height, knob_x, knob_y, 14, 0x555555, 0.80);
        }
    }
}

/// Helper function to draw smooth anti-aliased circular overlays with alpha
fn fill_circle_alpha(buffer: &mut [u32], buf_w: usize, buf_h: usize, cx: i32, cy: i32, radius: i32, color: u32, alpha: f32) {
    let r2 = radius * radius;
    let min_y = (cy - radius).max(0);
    let max_y = (cy + radius).min(buf_h as i32 - 1);
    for y in min_y..=max_y {
        let dy = y - cy;
        let dx = ((r2 - dy * dy) as f32).sqrt() as i32;
        if dx > 0 {
            fill_rect_alpha(buffer, buf_w, buf_h, cx - dx, y, dx * 2, 1, color, alpha);
        }
    }
}

