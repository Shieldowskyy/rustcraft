//! Android Native Platform Module for RustCraft
//! Provides ANativeWindow hardware-accelerated surface rendering, JNI bridge,
//! multi-touch virtual controls (responsive virtual analog joystick, tap-to-interact,
//! hold-to-mine, action buttons, camera look), physical gamepad/keyboard support,
//! and Logcat diagnostics.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Mutex;
use std::time::Instant;

use jni_sys::{jboolean, jclass, jfloat, jint, jobject, jstring, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::app::{GameState, InputState, RustCraftApp, HEIGHT, WIDTH};
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect_alpha, pack_color};

// ============================================================================
// 1. Android NDK Low-Level FFI Definitions
// ============================================================================

pub enum ANativeWindow {}

#[repr(C)]
pub struct ANativeWindow_Buffer {
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub format: i32,
    pub bits: *mut std::ffi::c_void,
    pub reserved: [u32; 6],
}

pub const WINDOW_FORMAT_RGBA_8888: i32 = 1;
pub const WINDOW_FORMAT_RGBX_8888: i32 = 2;
pub const WINDOW_FORMAT_RGB_565: i32 = 4;

pub const ANDROID_LOG_DEBUG: i32 = 3;
pub const ANDROID_LOG_INFO: i32 = 4;
pub const ANDROID_LOG_WARN: i32 = 5;
pub const ANDROID_LOG_ERROR: i32 = 6;

#[link(name = "android")]
extern "C" {
    pub fn ANativeWindow_fromSurface(env: *mut JNIEnv, surface: jobject) -> *mut ANativeWindow;
    pub fn ANativeWindow_acquire(window: *mut ANativeWindow);
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
    pub fn ANativeWindow_setBuffersGeometry(
        window: *mut ANativeWindow,
        width: i32,
        height: i32,
        format: i32,
    ) -> i32;
    pub fn ANativeWindow_lock(
        window: *mut ANativeWindow,
        outBuffer: *mut ANativeWindow_Buffer,
        inOutDirtyBounds: *mut std::ffi::c_void,
    ) -> i32;
    pub fn ANativeWindow_unlockAndPost(window: *mut ANativeWindow) -> i32;
}

#[link(name = "log")]
extern "C" {
    pub fn __android_log_write(
        prio: i32,
        tag: *const std::os::raw::c_char,
        text: *const std::os::raw::c_char,
    ) -> i32;
}

pub fn android_log(level: i32, tag: &str, msg: &str) {
    if let (Ok(c_tag), Ok(c_msg)) = (CString::new(tag), CString::new(msg)) {
        unsafe {
            __android_log_write(level, c_tag.as_ptr(), c_msg.as_ptr());
        }
    }
}

// ============================================================================
// 2. Android Multi-Touch & Virtual Controls System
// ============================================================================

pub const AMOTION_EVENT_ACTION_DOWN: i32 = 0;
pub const AMOTION_EVENT_ACTION_UP: i32 = 1;
pub const AMOTION_EVENT_ACTION_MOVE: i32 = 2;
pub const AMOTION_EVENT_ACTION_CANCEL: i32 = 3;
pub const AMOTION_EVENT_ACTION_POINTER_DOWN: i32 = 5;
pub const AMOTION_EVENT_ACTION_POINTER_UP: i32 = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
enum TouchRole {
    MovementJoystick { origin_x: f32, origin_y: f32 },
    CameraLook {
        start_x: f32,
        start_y: f32,
        last_x: f32,
        last_y: f32,
        start_time: Instant,
        is_dragging: bool,
    },
    JumpButton,
    SneakButton,
    SprintButton,
    MineButton,
    PlaceButton,
    MenuButton,
    InventoryButton,
    ChatButton,
    FlyButton,
    CamButton,
    HotbarSlot(u8),
    UiTouch,
    CloseMenuButton,
}

pub struct TouchTracker {
    touches: HashMap<i32, (f32, f32)>,
    roles: HashMap<i32, TouchRole>,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub primary_touch_pos: Option<(f32, f32)>,
    pub primary_touch_down: bool,

    // Button states
    pub btn_jump: bool,
    pub btn_sneak: bool,
    pub btn_sprint: bool,
    pub btn_mine: bool,
    pub btn_place: bool,
    pub btn_menu_just: bool,
    pub btn_inv_just: bool,
    pub btn_chat_just: bool,
    pub btn_fly_just: bool,
    pub btn_cam_just: bool,
    pub selected_hotbar: Option<u8>,

    // Movement vector from virtual joystick
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub stick_x: f32,
    pub stick_y: f32,

    // Physical gamepad analog axes
    pub gp_axis_lx: f32,
    pub gp_axis_ly: f32,
    pub gp_axis_rx: f32,
    pub gp_axis_ry: f32,
    pub gp_axis_lt: f32,
    pub gp_axis_rt: f32,
    pub gp_axis_hat_x: f32,
    pub gp_axis_hat_y: f32,

    // Text input & keys from Android IME / physical keyboard
    pub pending_typed_chars: Vec<char>,
    pub key_backspace_just: bool,
    pub key_enter_just: bool,
}

impl Default for TouchTracker {
    fn default() -> Self {
        Self {
            touches: HashMap::new(),
            roles: HashMap::new(),
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            primary_touch_pos: None,
            primary_touch_down: false,
            btn_jump: false,
            btn_sneak: false,
            btn_sprint: false,
            btn_mine: false,
            btn_place: false,
            btn_menu_just: false,
            btn_inv_just: false,
            btn_chat_just: false,
            btn_fly_just: false,
            btn_cam_just: false,
            selected_hotbar: None,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            stick_x: 0.0,
            stick_y: 0.0,
            gp_axis_lx: 0.0,
            gp_axis_ly: 0.0,
            gp_axis_rx: 0.0,
            gp_axis_ry: 0.0,
            gp_axis_lt: 0.0,
            gp_axis_rt: 0.0,
            gp_axis_hat_x: 0.0,
            gp_axis_hat_y: 0.0,
            pending_typed_chars: Vec::new(),
            key_backspace_just: false,
            key_enter_just: false,
        }
    }
}

impl TouchTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates continuous hold state (e.g. hold-to-mine when touching the screen without dragging)
    pub fn update_step(&mut self) {
        let mut start_mining = false;
        for role in self.roles.values() {
            if let TouchRole::CameraLook { start_time, is_dragging, .. } = role {
                // If the player holds their finger on the world view for >= 220ms without dragging, start mining!
                if !*is_dragging && start_time.elapsed().as_secs_f32() >= 0.22 {
                    start_mining = true;
                    break;
                }
            }
        }
        if start_mining {
            self.btn_mine = true;
        }
    }

    pub fn handle_touch_event(
        &mut self,
        action: i32,
        pointer_id: i32,
        x: f32,
        y: f32,
        screen_w: f32,
        screen_h: f32,
        frame_w: f32,
        frame_h: f32,
        in_menu: bool,
    ) {
        // Map native screen coordinates to RustCraft virtual render frame
        let norm_x = if screen_w > 0.0 { (x / screen_w) * frame_w } else { x };
        let norm_y = if screen_h > 0.0 { (y / screen_h) * frame_h } else { y };

        let masked_action = action & 0xFF;

        match masked_action {
            AMOTION_EVENT_ACTION_DOWN | AMOTION_EVENT_ACTION_POINTER_DOWN => {
                self.touches.insert(pointer_id, (norm_x, norm_y));
                self.primary_touch_pos = Some((norm_x, norm_y));
                self.primary_touch_down = true;

                if in_menu {
                    // Check top-right close / back button in menus
                    if norm_x >= frame_w - 85.0 && norm_y <= 42.0 {
                        self.btn_menu_just = true;
                        self.btn_inv_just = true;
                        self.roles.insert(pointer_id, TouchRole::CloseMenuButton);
                    } else {
                        self.roles.insert(pointer_id, TouchRole::UiTouch);
                    }
                    return;
                }

                // Check UI / Virtual buttons
                let role = self.classify_touch_down(norm_x, norm_y, frame_w, frame_h);
                if let TouchRole::MovementJoystick { origin_x, origin_y } = role {
                    self.update_joystick(norm_x - origin_x, norm_y - origin_y);
                }
                self.apply_role_press(role);
                self.roles.insert(pointer_id, role);
            }
            AMOTION_EVENT_ACTION_MOVE => {
                self.touches.insert(pointer_id, (norm_x, norm_y));
                if in_menu {
                    self.primary_touch_pos = Some((norm_x, norm_y));
                    return;
                }

                if let Some(role) = self.roles.get_mut(&pointer_id) {
                    match role {
                        TouchRole::MovementJoystick { origin_x, origin_y } => {
                            let dx = norm_x - *origin_x;
                            let dy = norm_y - *origin_y;
                            self.update_joystick(dx, dy);
                        }
                        TouchRole::CameraLook {
                            start_x,
                            start_y,
                            last_x,
                            last_y,
                            is_dragging,
                            ..
                        } => {
                            let total_dist = (norm_x - *start_x).hypot(norm_y - *start_y);
                            if total_dist > 6.0 {
                                *is_dragging = true;
                            }
                            let dx = norm_x - *last_x;
                            let dy = norm_y - *last_y;
                            self.mouse_dx += dx * 1.20;
                            self.mouse_dy += dy * 1.20;
                            *last_x = norm_x;
                            *last_y = norm_y;
                        }
                        _ => {}
                    }
                }
            }
            AMOTION_EVENT_ACTION_UP | AMOTION_EVENT_ACTION_POINTER_UP | AMOTION_EVENT_ACTION_CANCEL => {
                self.touches.remove(&pointer_id);
                if let Some(role) = self.roles.remove(&pointer_id) {
                    // Tap vs Hold detection for CameraLook
                    if let TouchRole::CameraLook { start_time, is_dragging, .. } = role {
                        if !is_dragging && start_time.elapsed().as_secs_f32() < 0.22 {
                            // Quick tap on world view without drag = Place / Use block or interact!
                            self.btn_place = true;
                        }
                        if start_time.elapsed().as_secs_f32() >= 0.22 {
                            // Stopped holding mining
                            self.btn_mine = false;
                        }
                    }
                    self.apply_role_release(role);
                }

                if self.touches.is_empty() {
                    self.primary_touch_pos = None;
                    self.primary_touch_down = false;
                    self.reset_movement();
                } else if let Some((_, &(fx, fy))) = self.touches.iter().next() {
                    self.primary_touch_pos = Some((fx, fy));
                }
            }
            _ => {}
        }
    }

    fn update_joystick(&mut self, dx: f32, dy: f32) {
        let deadzone = 8.0;
        let max_dist = 52.0;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > deadzone {
            let norm_dx = (dx / max_dist).clamp(-1.0, 1.0);
            let norm_dy = (dy / max_dist).clamp(-1.0, 1.0);
            self.stick_x = norm_dx;
            self.stick_y = norm_dy;
            self.move_forward = norm_dy < -0.25;
            self.move_backward = norm_dy > 0.25;
            self.move_left = norm_dx < -0.25;
            self.move_right = norm_dx > 0.25;
            if norm_dy < -0.85 {
                self.btn_sprint = true;
            }
        } else {
            self.reset_movement();
        }
    }

    fn classify_touch_down(&self, x: f32, y: f32, w: f32, h: f32) -> TouchRole {
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
        if (hotbar_left - 6.0..=hotbar_left + hotbar_w + 6.0).contains(&x) && y >= hotbar_top - 6.0 {
            let slot = (((x - hotbar_left).max(0.0) / (slot_size + margin)).floor() as u8).min(9);
            return TouchRole::HotbarSlot(slot);
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

    fn apply_role_press(&mut self, role: TouchRole) {
        match role {
            TouchRole::JumpButton => self.btn_jump = true,
            TouchRole::SneakButton => self.btn_sneak = true,
            TouchRole::SprintButton => self.btn_sprint = !self.btn_sprint,
            TouchRole::MineButton => self.btn_mine = true,
            TouchRole::PlaceButton => self.btn_place = true,
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

    fn apply_role_release(&mut self, role: TouchRole) {
        match role {
            TouchRole::JumpButton => self.btn_jump = false,
            TouchRole::SneakButton => self.btn_sneak = false,
            TouchRole::MineButton => self.btn_mine = false,
            TouchRole::PlaceButton => self.btn_place = false,
            TouchRole::MovementJoystick { .. } => self.reset_movement(),
            _ => {}
        }
    }

    fn reset_movement(&mut self) {
        self.move_forward = false;
        self.move_backward = false;
        self.move_left = false;
        self.move_right = false;
        self.stick_x = 0.0;
        self.stick_y = 0.0;
    }

    /// Renders sleek, semi-transparent touch overlay controls directly onto the color buffer.
    pub fn draw_overlay(&self, buffer: &mut [u32], width: usize, height: usize, in_menu: bool) {
        let w_i32 = width as i32;
        let h_i32 = height as i32;

        if in_menu {
            // Draw a convenient "CLOSE [X]" button at top right in inventory/menus
            fill_rect_alpha(buffer, width, height, w_i32 - 75, 8, 65, 26, 0x000000, 0.65);
            draw_text_shadowed(buffer, width, height, "CLOSE", w_i32 - 62, 14, 1, 1);
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
        if self.btn_place {
            fill_circle_alpha(buffer, width, height, place_cx, place_cy, 24, 0xb45309, 0.70);
            draw_text_colored_shadowed(buffer, width, height, "PLACE", place_cx - 18, place_cy - 4, 1, 1, active_yellow);
        } else {
            draw_text_shadowed(buffer, width, height, "PLACE", place_cx - 18, place_cy - 4, 1, 1);
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

// ============================================================================
// 3. Android Game Context & Lifecycle Coordinator
// ============================================================================

pub struct AndroidGameContext {
    pub app: RustCraftApp,
    pub native_window: *mut ANativeWindow,
    pub touch_tracker: TouchTracker,
    pub last_frame_time: Instant,
    pub width: usize,
    pub height: usize,
    pub physical_width: i32,
    pub physical_height: i32,
}

unsafe impl Send for AndroidGameContext {}
unsafe impl Sync for AndroidGameContext {}

static GAME_INSTANCE: Mutex<Option<AndroidGameContext>> = Mutex::new(None);

// ============================================================================
// 4. JNI Exported Functions (Called by Java Activity)
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeInit(
    env: *mut JNIEnv,
    _class: jclass,
    width: jint,
    height: jint,
    seed: jint,
    internal_files_dir: jstring,
) -> jboolean {
    android_log(ANDROID_LOG_INFO, "RustCraftNative", "nativeInit starting...");

    // Setup save path from internal files directory
    if !internal_files_dir.is_null() {
        if let Some(get_utf_chars) = (**env).GetStringUTFChars {
            if let Some(release_utf_chars) = (**env).ReleaseStringUTFChars {
                let c_str_ptr = get_utf_chars(env, internal_files_dir, std::ptr::null_mut());
                if !c_str_ptr.is_null() {
                    if let Ok(path_str) = CStr::from_ptr(c_str_ptr).to_str() {
                        let saves_path = std::path::PathBuf::from(path_str).join("saves");
                        android_log(
                            ANDROID_LOG_INFO,
                            "RustCraftNative",
                            &format!("Configuring Android save directory: {:?}", saves_path),
                        );
                        crate::world::storage::set_custom_save_dir(saves_path);
                    }
                    release_utf_chars(env, internal_files_dir, c_str_ptr);
                }
            }
        }
    }

    let initial_seed = if seed == 0 { 1337 } else { seed as u32 };

    // Performance optimization for Android:
    // Scale internal rendering resolution to a high-performance target (e.g. ~540p),
    // preserving physical screen aspect ratio. ANativeWindow hardware compositor
    // automatically scales this up to the physical resolution (e.g. 2400x1080) with 0 GPU overhead!
    let (render_w, render_h) = if width > 0 && height > 0 {
        let aspect = width as f32 / height as f32;
        let target_h = 540.0f32.min(height as f32);
        let target_w = (target_h * aspect).round() as usize;
        let aligned_w = ((target_w + 3) & !3).max(640);
        let aligned_h = (((target_h as usize) + 3) & !3).max(360);
        (aligned_w, aligned_h)
    } else {
        (WIDTH, HEIGHT)
    };

    // Initialize with hardware-accelerated OpenGL (with automatic fallback to software if needed)
    let mut app = RustCraftApp::new(render_w, render_h, "opengl", initial_seed);
    // On touch mobile, enable AutoJump so moving across block elevations is effortless
    app.settings.jump_mode = crate::core::settings::JumpMode::AutoJump;

    let ctx = AndroidGameContext {
        app,
        native_window: std::ptr::null_mut(),
        touch_tracker: TouchTracker::new(),
        last_frame_time: Instant::now(),
        width: render_w,
        height: render_h,
        physical_width: width,
        physical_height: height,
    };

    let backend = ctx.app.renderer.backend_name();
    let is_hw = backend == "opengl" || backend == "vulkan" || backend.starts_with("gl");
    if is_hw {
        android_log(
            ANDROID_LOG_INFO,
            "RustCraftNative",
            &format!("GPU Hardware Acceleration initialized successfully ({})", backend),
        );
    } else {
        android_log(
            ANDROID_LOG_WARN,
            "RustCraftNative",
            "WARNING: Hardware acceleration failed! Running on Software Rasterizer fallback. Performance will be degraded.",
        );
    }

    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        *lock = Some(ctx);
        android_log(
            ANDROID_LOG_INFO,
            "RustCraftNative",
            &format!("nativeInit successful! Backend: {}, Render: {}x{}, Physical: {}x{}", backend, render_w, render_h, width, height),
        );
        JNI_TRUE
    } else {
        android_log(ANDROID_LOG_ERROR, "RustCraftNative", "Failed to acquire GAME_INSTANCE lock");
        JNI_FALSE
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeSetSurface(
    env: *mut JNIEnv,
    _class: jclass,
    surface: jobject,
) {
    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_mut() {
            if !ctx.native_window.is_null() {
                ANativeWindow_release(ctx.native_window);
                ctx.native_window = std::ptr::null_mut();
            }

            if !surface.is_null() {
                let window = ANativeWindow_fromSurface(env, surface);
                if !window.is_null() {
                    ANativeWindow_acquire(window);
                    // Set buffer geometry to internal render resolution (scales up to full screen automatically)
                    ANativeWindow_setBuffersGeometry(
                        window,
                        ctx.width as i32,
                        ctx.height as i32,
                        WINDOW_FORMAT_RGBA_8888,
                    );
                    ctx.native_window = window;
                    android_log(
                        ANDROID_LOG_INFO,
                        "RustCraftNative",
                        &format!("Native surface bound ({}x{})", ctx.width, ctx.height),
                    );
                } else {
                    android_log(ANDROID_LOG_ERROR, "RustCraftNative", "ANativeWindow_fromSurface returned NULL");
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeStep(
    _env: *mut JNIEnv,
    _class: jclass,
    dt: jfloat,
) -> jint {
    let mut lock = match GAME_INSTANCE.lock() {
        Ok(l) => l,
        Err(_) => return -1,
    };

    let ctx = match lock.as_mut() {
        Some(c) => c,
        None => return -2,
    };

    let delta_time = if dt > 0.0 && dt < 0.25 {
        dt
    } else {
        let now = Instant::now();
        let elapsed = now.duration_since(ctx.last_frame_time).as_secs_f32();
        ctx.last_frame_time = now;
        elapsed.clamp(0.001, 0.1)
    };

    let in_menu = !matches!(ctx.app.state, GameState::Playing);

    // Update continuous touch states (e.g. hold to mine block)
    ctx.touch_tracker.update_step();

    // Combine virtual movement joystick and physical gamepad analog stick
    let mut stick_x = ctx.touch_tracker.stick_x;
    let mut stick_y = ctx.touch_tracker.stick_y;
    if ctx.touch_tracker.gp_axis_lx.abs() > 0.15 || ctx.touch_tracker.gp_axis_ly.abs() > 0.15 {
        stick_x = ctx.touch_tracker.gp_axis_lx;
        stick_y = ctx.touch_tracker.gp_axis_ly;
    }

    let move_fwd = ctx.touch_tracker.move_forward || stick_y < -0.25;
    let move_back = ctx.touch_tracker.move_backward || stick_y > 0.25;
    let move_l = ctx.touch_tracker.move_left || stick_x < -0.25;
    let move_r = ctx.touch_tracker.move_right || stick_x > 0.25;

    // Physical gamepad right stick camera rotation
    if ctx.touch_tracker.gp_axis_rx.abs() > 0.15 {
        ctx.touch_tracker.mouse_dx += ctx.touch_tracker.gp_axis_rx * 14.0;
    }
    if ctx.touch_tracker.gp_axis_ry.abs() > 0.15 {
        ctx.touch_tracker.mouse_dy += ctx.touch_tracker.gp_axis_ry * 14.0;
    }

    let is_mine = ctx.touch_tracker.btn_mine || ctx.touch_tracker.gp_axis_rt > 0.4;
    let is_place = ctx.touch_tracker.btn_place || ctx.touch_tracker.gp_axis_lt > 0.4;
    // Reset place pulse after applying
    ctx.touch_tracker.btn_place = false;

    // Drain queued text input from software keyboard
    let typed_chars = std::mem::take(&mut ctx.touch_tracker.pending_typed_chars);
    let key_enter = ctx.touch_tracker.key_enter_just;
    let key_backspace = ctx.touch_tracker.key_backspace_just;
    ctx.touch_tracker.key_enter_just = false;
    ctx.touch_tracker.key_backspace_just = false;

    // Build InputState from touch controls & gamepads
    let mut input = InputState {
        mouse_dx: ctx.touch_tracker.mouse_dx,
        mouse_dy: ctx.touch_tracker.mouse_dy,
        mouse_pos: ctx.touch_tracker.primary_touch_pos,
        mouse_left: is_mine || (in_menu && ctx.touch_tracker.primary_touch_down),
        mouse_right: is_place,
        scroll_delta: 0.0,

        key_w: move_fwd,
        key_s: move_back,
        key_a: move_l,
        key_d: move_r,
        key_space: ctx.touch_tracker.btn_jump,
        key_shift: ctx.touch_tracker.btn_sneak,
        key_ctrl: ctx.touch_tracker.btn_sprint,
        key_escape: ctx.touch_tracker.btn_menu_just,
        key_f: ctx.touch_tracker.btn_fly_just,
        key_e: ctx.touch_tracker.btn_inv_just,
        key_f5: ctx.touch_tracker.btn_cam_just,
        key_enter,
        key_backspace,
        key_t: ctx.touch_tracker.btn_chat_just,
        key_digits: [false; 10],
        typed_chars,

        gp_left_stick_x: stick_x,
        // In RustCraft, forward wish_dir is positive (+Y)
        gp_left_stick_y: -stick_y,
        gp_right_stick_x: ctx.touch_tracker.gp_axis_rx,
        gp_right_stick_y: ctx.touch_tracker.gp_axis_ry,
        gp_left_trigger: if is_place { 1.0 } else { 0.0 },
        gp_right_trigger: if is_mine { 1.0 } else { 0.0 },
        gp_btn_south: ctx.touch_tracker.btn_jump,
        gp_btn_east: ctx.touch_tracker.btn_sneak,
        gp_btn_north: ctx.touch_tracker.btn_inv_just,
        gp_btn_start: ctx.touch_tracker.btn_menu_just,
        gp_dpad_up: ctx.touch_tracker.gp_axis_hat_y < -0.5,
        gp_dpad_down: ctx.touch_tracker.gp_axis_hat_y > 0.5,
        gp_dpad_left: ctx.touch_tracker.gp_axis_hat_x < -0.5,
        gp_dpad_right: ctx.touch_tracker.gp_axis_hat_x > 0.5,
        gp_start_just_pressed: ctx.touch_tracker.btn_menu_just,

        is_mouse_captured: !in_menu,
        gp_connected: false,
        ..Default::default()
    };

    // Hotbar selection from touch
    if let Some(slot) = ctx.touch_tracker.selected_hotbar.take() {
        ctx.app.player.selected_hotbar_slot = slot;
        if (slot as usize) < input.key_digits.len() {
            input.key_digits[slot as usize] = true;
        }
    }

    // Reset one-shot deltas and just-pressed button pulses
    ctx.touch_tracker.mouse_dx = 0.0;
    ctx.touch_tracker.mouse_dy = 0.0;
    ctx.touch_tracker.btn_menu_just = false;
    ctx.touch_tracker.btn_inv_just = false;
    ctx.touch_tracker.btn_chat_just = false;
    ctx.touch_tracker.btn_fly_just = false;
    ctx.touch_tracker.btn_cam_just = false;

    // Run game engine update & render
    let _output = ctx.app.update_and_render(delta_time, &input);

    // Blit frame buffer to ANativeWindow
    if !ctx.native_window.is_null() {
        let mut buffer = ANativeWindow_Buffer {
            width: 0,
            height: 0,
            stride: 0,
            format: 0,
            bits: std::ptr::null_mut(),
            reserved: [0; 6],
        };

        if ANativeWindow_lock(ctx.native_window, &mut buffer, std::ptr::null_mut()) == 0 {
            if !buffer.bits.is_null() {
                let pixel_data = ctx.app.pixel_buffer_mut();

                // Draw on-screen virtual touch controls overlay before final blit
                ctx.touch_tracker.draw_overlay(pixel_data, ctx.width, ctx.height, in_menu);

                let dst_stride = buffer.stride as usize;
                let src_w = ctx.width.min(buffer.width as usize);
                let src_h = ctx.height.min(buffer.height as usize);
                let dst_ptr = buffer.bits as *mut u32;

                // Software color space conversion: Pack 0x00RRGGBB (xRGB) -> 0xAABBGGRR (RGBA8888 for Android)
                // Optimized bit-swap: swaps R and B bytes while ensuring alpha is 0xFF
                for y in 0..src_h {
                    let src_row = &pixel_data[y * ctx.width..y * ctx.width + src_w];
                    let dst_row = std::slice::from_raw_parts_mut(dst_ptr.add(y * dst_stride), src_w);
                    for (x, &px) in src_row.iter().enumerate() {
                        let b_r_swap = ((px & 0x00FF0000) >> 16) | ((px & 0x000000FF) << 16);
                        dst_row[x] = 0xFF000000 | (px & 0x0000FF00) | b_r_swap;
                    }
                }
            }
            ANativeWindow_unlockAndPost(ctx.native_window);
        }
    }

    // Return status flags to Java:
    // Bit 0 (1): Text input active (chat, nickname, world creation - show soft keyboard)
    // Bit 1 (2): In menu state
    // Bit 2 (4): Software rasterizer fallback active (OpenGL / Vulkan failed)
    let mut flags: i32 = 0;
    if ctx.app.is_text_input_active() {
        flags |= 1;
    }
    if in_menu {
        flags |= 2;
    }
    let backend = ctx.app.renderer.backend_name();
    if backend == "android_software" || (!backend.contains("gl") && !backend.contains("vulkan")) {
        flags |= 4;
    }

    flags
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeTouchEvent(
    _env: *mut JNIEnv,
    _class: jclass,
    action: jint,
    pointer_id: jint,
    x: jfloat,
    y: jfloat,
    screen_w: jfloat,
    screen_h: jfloat,
) {
    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_mut() {
            let in_menu = !matches!(ctx.app.state, GameState::Playing);

            ctx.touch_tracker.handle_touch_event(
                action,
                pointer_id,
                x,
                y,
                screen_w,
                screen_h,
                ctx.width as f32,
                ctx.height as f32,
                in_menu,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeAxisEvent(
    _env: *mut JNIEnv,
    _class: jclass,
    axis_x: jfloat,
    axis_y: jfloat,
    axis_z: jfloat,
    axis_rz: jfloat,
    axis_lt: jfloat,
    axis_rt: jfloat,
    axis_hat_x: jfloat,
    axis_hat_y: jfloat,
) {
    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_mut() {
            ctx.touch_tracker.gp_axis_lx = axis_x;
            ctx.touch_tracker.gp_axis_ly = axis_y;
            ctx.touch_tracker.gp_axis_rx = axis_z;
            ctx.touch_tracker.gp_axis_ry = axis_rz;
            ctx.touch_tracker.gp_axis_lt = axis_lt;
            ctx.touch_tracker.gp_axis_rt = axis_rt;
            ctx.touch_tracker.gp_axis_hat_x = axis_hat_x;
            ctx.touch_tracker.gp_axis_hat_y = axis_hat_y;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeTextInput(
    env: *mut JNIEnv,
    _class: jclass,
    text: jstring,
) {
    if text.is_null() {
        return;
    }
    if let Some(get_utf_chars) = (**env).GetStringUTFChars {
        if let Some(release_utf_chars) = (**env).ReleaseStringUTFChars {
            let c_str_ptr = get_utf_chars(env, text, std::ptr::null_mut());
            if !c_str_ptr.is_null() {
                if let Ok(input_str) = CStr::from_ptr(c_str_ptr).to_str() {
                    if let Ok(mut lock) = GAME_INSTANCE.lock() {
                        if let Some(ctx) = lock.as_mut() {
                            for ch in input_str.chars() {
                                ctx.touch_tracker.pending_typed_chars.push(ch);
                            }
                        }
                    }
                }
                release_utf_chars(env, text, c_str_ptr);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeIsTextInputActive(
    _env: *mut JNIEnv,
    _class: jclass,
) -> jboolean {
    if let Ok(lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_ref() {
            if ctx.app.is_text_input_active() {
                return JNI_TRUE;
            }
        }
    }
    JNI_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeKeyEvent(
    _env: *mut JNIEnv,
    _class: jclass,
    key_code: jint,
    is_down: jboolean,
) {
    // Android key codes mapping
    const KEYCODE_DPAD_UP: i32 = 19;
    const KEYCODE_DPAD_DOWN: i32 = 20;
    const KEYCODE_DPAD_LEFT: i32 = 21;
    const KEYCODE_DPAD_RIGHT: i32 = 22;
    const KEYCODE_ENTER: i32 = 66;
    const KEYCODE_DEL: i32 = 67;
    const KEYCODE_FORWARD_DEL: i32 = 112;
    const KEYCODE_NUMPAD_ENTER: i32 = 160;

    const KEYCODE_BUTTON_A: i32 = 96;
    const KEYCODE_BUTTON_B: i32 = 97;
    const KEYCODE_BUTTON_X: i32 = 99;
    const KEYCODE_BUTTON_Y: i32 = 100;
    const KEYCODE_BUTTON_L1: i32 = 102;
    const KEYCODE_BUTTON_R1: i32 = 103;
    const KEYCODE_BUTTON_START: i32 = 108;
    const KEYCODE_BUTTON_SELECT: i32 = 109;
    const KEYCODE_ESCAPE: i32 = 111;

    const KEYCODE_W: i32 = 51;
    const KEYCODE_A: i32 = 29;
    const KEYCODE_S: i32 = 47;
    const KEYCODE_D: i32 = 32;
    const KEYCODE_SPACE: i32 = 62;
    const KEYCODE_E: i32 = 33;
    const KEYCODE_T: i32 = 48;
    const KEYCODE_SLASH: i32 = 76;

    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_mut() {
            let down = is_down != JNI_FALSE;
            match key_code {
                KEYCODE_W | KEYCODE_DPAD_UP => ctx.touch_tracker.move_forward = down,
                KEYCODE_S | KEYCODE_DPAD_DOWN => ctx.touch_tracker.move_backward = down,
                KEYCODE_A | KEYCODE_DPAD_LEFT => ctx.touch_tracker.move_left = down,
                KEYCODE_D | KEYCODE_DPAD_RIGHT => ctx.touch_tracker.move_right = down,
                KEYCODE_SPACE | KEYCODE_BUTTON_A => ctx.touch_tracker.btn_jump = down,
                KEYCODE_BUTTON_B => ctx.touch_tracker.btn_sneak = down,
                KEYCODE_BUTTON_X => {
                    if down {
                        ctx.touch_tracker.btn_sprint = !ctx.touch_tracker.btn_sprint;
                    }
                }
                KEYCODE_BUTTON_R1 => ctx.touch_tracker.btn_mine = down,
                KEYCODE_BUTTON_L1 => ctx.touch_tracker.btn_place = down,
                KEYCODE_ENTER | KEYCODE_NUMPAD_ENTER => {
                    if down {
                        ctx.touch_tracker.key_enter_just = true;
                    }
                }
                KEYCODE_DEL | KEYCODE_FORWARD_DEL => {
                    if down {
                        ctx.touch_tracker.key_backspace_just = true;
                    }
                }
                KEYCODE_ESCAPE | KEYCODE_BUTTON_START => {
                    if down {
                        ctx.touch_tracker.btn_menu_just = true;
                    }
                }
                KEYCODE_BUTTON_Y | KEYCODE_E => {
                    if down {
                        ctx.touch_tracker.btn_inv_just = true;
                    }
                }
                KEYCODE_T => {
                    if down {
                        ctx.touch_tracker.btn_chat_just = true;
                    }
                }
                _ => {}
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeSaveAndPause(
    _env: *mut JNIEnv,
    _class: jclass,
) {
    android_log(ANDROID_LOG_INFO, "RustCraftNative", "nativeSaveAndPause called");
    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_mut() {
            let _ = crate::world::storage::save_current_world(
                &mut ctx.app.world,
                &ctx.app.camera,
                &ctx.app.player,
                ctx.app.save_slot,
                None,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeDestroy(
    _env: *mut JNIEnv,
    _class: jclass,
) {
    android_log(ANDROID_LOG_INFO, "RustCraftNative", "nativeDestroy called");
    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        if let Some(mut ctx) = lock.take() {
            let _ = crate::world::storage::save_current_world(
                &mut ctx.app.world,
                &ctx.app.camera,
                &ctx.app.player,
                ctx.app.save_slot,
                None,
            );
            if !ctx.native_window.is_null() {
                ANativeWindow_release(ctx.native_window);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeGetRendererBackend(
    env: *mut JNIEnv,
    _class: jclass,
) -> jstring {
    let backend_name = if let Ok(lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_ref() {
            ctx.app.renderer.backend_name()
        } else {
            "unknown"
        }
    } else {
        "unknown"
    };

    if let Ok(c_str) = CString::new(backend_name) {
        if let Some(new_string) = (**env).NewStringUTF {
            return new_string(env, c_str.as_ptr());
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_rustcraft_game_RustCraftNative_nativeIsHardwareAccelerated(
    _env: *mut JNIEnv,
    _class: jclass,
) -> jboolean {
    if let Ok(lock) = GAME_INSTANCE.lock() {
        if let Some(ctx) = lock.as_ref() {
            let backend = ctx.app.renderer.backend_name();
            if backend == "opengl" || backend == "vulkan" || backend.starts_with("gl") {
                return JNI_TRUE;
            }
        }
    }
    JNI_FALSE
}
