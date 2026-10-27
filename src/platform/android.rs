//! Android Native Platform Module for RustCraft
//! Provides ANativeWindow hardware-accelerated surface rendering, JNI bridge,
//! multi-touch virtual controls (D-Pad/Joystick, action buttons, camera look),
//! physical gamepad/keyboard support, and Logcat diagnostics.

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
    CameraLook { last_x: f32, last_y: f32 },
    JumpButton,
    SneakButton,
    MineButton,
    PlaceButton,
    MenuButton,
    InventoryButton,
    ChatButton,
    FlyButton,
    CamButton,
    HotbarSlot(u8),
    UiTouch,
}

#[derive(Default)]
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

impl TouchTracker {
    pub fn new() -> Self {
        Self::default()
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
                    self.roles.insert(pointer_id, TouchRole::UiTouch);
                    return;
                }

                // Check UI / Virtual buttons
                let role = self.classify_touch_down(norm_x, norm_y, frame_w, frame_h);
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
                            let deadzone = 12.0;
                            let max_dist = 60.0;
                            let dist = (dx * dx + dy * dy).sqrt();

                            if dist > deadzone {
                                let norm_dx = (dx / max_dist).clamp(-1.0, 1.0);
                                let norm_dy = (dy / max_dist).clamp(-1.0, 1.0);
                                self.stick_x = norm_dx;
                                self.stick_y = norm_dy;
                                self.move_forward = norm_dy < -0.3;
                                self.move_backward = norm_dy > 0.3;
                                self.move_left = norm_dx < -0.3;
                                self.move_right = norm_dx > 0.3;
                            } else {
                                self.reset_movement();
                            }
                        }
                        TouchRole::CameraLook { last_x, last_y } => {
                            let dx = norm_x - *last_x;
                            let dy = norm_y - *last_y;
                            self.mouse_dx += dx * 1.25;
                            self.mouse_dy += dy * 1.25;
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

    fn classify_touch_down(&self, x: f32, y: f32, w: f32, h: f32) -> TouchRole {
        // 1. Top bar buttons:
        // Menu / Pause button (center top)
        if (w * 0.5 - 40.0..=w * 0.5 + 40.0).contains(&x) && y <= 48.0 {
            return TouchRole::MenuButton;
        }
        // Inventory button (top right)
        if x >= w - 85.0 && y <= 48.0 {
            return TouchRole::InventoryButton;
        }
        // Chat button (top left)
        if (20.0..=80.0).contains(&x) && y <= 48.0 {
            return TouchRole::ChatButton;
        }
        // Fly button
        if (90.0..=150.0).contains(&x) && y <= 48.0 {
            return TouchRole::FlyButton;
        }
        // Camera perspective button
        if (160.0..=220.0).contains(&x) && y <= 48.0 {
            return TouchRole::CamButton;
        }

        // 2. Hotbar slots (bottom center)
        let hotbar_w = 9.0 * 36.0;
        let hotbar_left = (w - hotbar_w) * 0.5;
        let hotbar_top = h - 42.0;
        if (hotbar_left..=hotbar_left + hotbar_w).contains(&x) && y >= hotbar_top {
            let slot = ((x - hotbar_left) / 36.0).floor() as u8;
            if slot < 9 {
                return TouchRole::HotbarSlot(slot);
            }
        }

        // 3. Action buttons (right side, anchored to right edge)
        // Jump Button
        if (x - (w - 80.0)).powi(2) + (y - (h - 150.0)).powi(2) <= 45.0 * 45.0 {
            return TouchRole::JumpButton;
        }
        // Sneak Button
        if (x - (w - 80.0)).powi(2) + (y - (h - 60.0)).powi(2) <= 38.0 * 38.0 {
            return TouchRole::SneakButton;
        }
        // Mine / Attack Button
        if (x - (w - 180.0)).powi(2) + (y - (h - 150.0)).powi(2) <= 45.0 * 45.0 {
            return TouchRole::MineButton;
        }
        // Place / Use Button
        if (x - (w - 180.0)).powi(2) + (y - (h - 60.0)).powi(2) <= 42.0 * 42.0 {
            return TouchRole::PlaceButton;
        }

        // 4. Movement Joystick (left half of screen, bottom area)
        if x < w * 0.42 && y > 150.0 {
            return TouchRole::MovementJoystick { origin_x: x, origin_y: y };
        }

        // 5. Default right half: Camera rotation
        TouchRole::CameraLook { last_x: x, last_y: y }
    }

    fn apply_role_press(&mut self, role: TouchRole) {
        match role {
            TouchRole::JumpButton => self.btn_jump = true,
            TouchRole::SneakButton => self.btn_sneak = true,
            TouchRole::MineButton => self.btn_mine = true,
            TouchRole::PlaceButton => self.btn_place = true,
            TouchRole::MenuButton => self.btn_menu_just = true,
            TouchRole::InventoryButton => self.btn_inv_just = true,
            TouchRole::ChatButton => self.btn_chat_just = true,
            TouchRole::FlyButton => self.btn_fly_just = true,
            TouchRole::CamButton => self.btn_cam_just = true,
            TouchRole::HotbarSlot(slot) => self.selected_hotbar = Some(slot),
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
        if in_menu {
            return;
        }

        let active_green = pack_color(0.4, 1.0, 0.4);
        let active_yellow = pack_color(1.0, 0.85, 0.3);

        let w_i32 = width as i32;
        let h_i32 = height as i32;

        // --- Top Bar Buttons ---
        // Pause / Menu (Center)
        fill_rect_alpha(buffer, width, height, w_i32 / 2 - 35, 10, 70, 26, 0x000000, 0.55);
        draw_text_shadowed(buffer, width, height, "PAUSE", w_i32 / 2 - 20, 16, 1, 1);

        // Inventory (Right)
        fill_rect_alpha(buffer, width, height, w_i32 - 75, 10, 65, 26, 0x000000, 0.55);
        draw_text_shadowed(buffer, width, height, "INV", w_i32 - 52, 16, 1, 1);

        // Chat (Left)
        fill_rect_alpha(buffer, width, height, 20, 10, 60, 26, 0x000000, 0.55);
        draw_text_shadowed(buffer, width, height, "CHAT", 35, 16, 1, 1);

        // Fly
        fill_rect_alpha(buffer, width, height, 90, 10, 60, 26, 0x000000, 0.55);
        draw_text_shadowed(buffer, width, height, "FLY", 110, 16, 1, 1);

        // Camera perspective (F5)
        fill_rect_alpha(buffer, width, height, 160, 10, 60, 26, 0x000000, 0.55);
        draw_text_shadowed(buffer, width, height, "CAM", 180, 16, 1, 1);

        // --- Action Buttons (Right) ---
        // Jump Button
        let jump_x = w_i32 - 115;
        let jump_y = h_i32 - 185;
        fill_rect_alpha(buffer, width, height, jump_x, jump_y, 70, 70, 0x000000, 0.60);
        if self.btn_jump {
            draw_text_colored_shadowed(buffer, width, height, "JUMP", jump_x + 15, jump_y + 30, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "JUMP", jump_x + 15, jump_y + 30, 1, 1);
        }

        // Sneak Button
        let sneak_x = w_i32 - 110;
        let sneak_y = h_i32 - 90;
        fill_rect_alpha(buffer, width, height, sneak_x, sneak_y, 60, 60, 0x000000, 0.60);
        if self.btn_sneak {
            draw_text_colored_shadowed(buffer, width, height, "SNEAK", sneak_x + 10, sneak_y + 25, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "SNEAK", sneak_x + 10, sneak_y + 25, 1, 1);
        }

        // Mine / Attack Button
        let mine_x = w_i32 - 215;
        let mine_y = h_i32 - 185;
        fill_rect_alpha(buffer, width, height, mine_x, mine_y, 70, 70, 0x000000, 0.60);
        if self.btn_mine {
            draw_text_colored_shadowed(buffer, width, height, "MINE", mine_x + 18, mine_y + 30, 1, 1, active_yellow);
        } else {
            draw_text_shadowed(buffer, width, height, "MINE", mine_x + 18, mine_y + 30, 1, 1);
        }

        // Place / Use Button
        let place_x = w_i32 - 210;
        let place_y = h_i32 - 90;
        fill_rect_alpha(buffer, width, height, place_x, place_y, 60, 60, 0x000000, 0.60);
        if self.btn_place {
            draw_text_colored_shadowed(buffer, width, height, "PLACE", place_x + 10, place_y + 25, 1, 1, active_yellow);
        } else {
            draw_text_shadowed(buffer, width, height, "PLACE", place_x + 10, place_y + 25, 1, 1);
        }

        // --- Movement D-Pad (Left) ---
        let dpad_cx = 120;
        let dpad_cy = h_i32 - 140;

        // Up (W)
        fill_rect_alpha(buffer, width, height, dpad_cx - 20, dpad_cy - 65, 40, 40, 0x000000, 0.55);
        if self.move_forward {
            draw_text_colored_shadowed(buffer, width, height, "^", dpad_cx - 4, dpad_cy - 50, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "^", dpad_cx - 4, dpad_cy - 50, 1, 1);
        }

        // Down (S)
        fill_rect_alpha(buffer, width, height, dpad_cx - 20, dpad_cy + 25, 40, 40, 0x000000, 0.55);
        if self.move_backward {
            draw_text_colored_shadowed(buffer, width, height, "v", dpad_cx - 4, dpad_cy + 38, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "v", dpad_cx - 4, dpad_cy + 38, 1, 1);
        }

        // Left (A)
        fill_rect_alpha(buffer, width, height, dpad_cx - 65, dpad_cy - 20, 40, 40, 0x000000, 0.55);
        if self.move_left {
            draw_text_colored_shadowed(buffer, width, height, "<", dpad_cx - 50, dpad_cy - 6, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, "<", dpad_cx - 50, dpad_cy - 6, 1, 1);
        }

        // Right (D)
        fill_rect_alpha(buffer, width, height, dpad_cx + 25, dpad_cy - 20, 40, 40, 0x000000, 0.55);
        if self.move_right {
            draw_text_colored_shadowed(buffer, width, height, ">", dpad_cx + 40, dpad_cy - 6, 1, 1, active_green);
        } else {
            draw_text_shadowed(buffer, width, height, ">", dpad_cx + 40, dpad_cy - 6, 1, 1);
        }

        // Center hub
        fill_rect_alpha(buffer, width, height, dpad_cx - 15, dpad_cy - 15, 30, 30, 0x222222, 0.65);
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
    let render_w = if width > 0 { width as usize } else { WIDTH };
    let render_h = if height > 0 { height as usize } else { HEIGHT };

    let app = RustCraftApp::new(render_w, render_h, "software", initial_seed);

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

    if let Ok(mut lock) = GAME_INSTANCE.lock() {
        *lock = Some(ctx);
        android_log(
            ANDROID_LOG_INFO,
            "RustCraftNative",
            &format!("nativeInit successful! Resolution: {}x{}", render_w, render_h),
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

    // Combine virtual movement joystick and physical gamepad analog stick
    let mut stick_x = ctx.touch_tracker.stick_x;
    let mut stick_y = ctx.touch_tracker.stick_y;
    if ctx.touch_tracker.gp_axis_lx.abs() > 0.15 || ctx.touch_tracker.gp_axis_ly.abs() > 0.15 {
        stick_x = ctx.touch_tracker.gp_axis_lx;
        stick_y = ctx.touch_tracker.gp_axis_ly;
    }

    let move_fwd = ctx.touch_tracker.move_forward || stick_y < -0.3;
    let move_back = ctx.touch_tracker.move_backward || stick_y > 0.3;
    let move_l = ctx.touch_tracker.move_left || stick_x < -0.3;
    let move_r = ctx.touch_tracker.move_right || stick_x > 0.3;

    // Physical gamepad right stick camera rotation
    if ctx.touch_tracker.gp_axis_rx.abs() > 0.15 {
        ctx.touch_tracker.mouse_dx += ctx.touch_tracker.gp_axis_rx * 14.0;
    }
    if ctx.touch_tracker.gp_axis_ry.abs() > 0.15 {
        ctx.touch_tracker.mouse_dy += ctx.touch_tracker.gp_axis_ry * 14.0;
    }

    let is_mine = ctx.touch_tracker.btn_mine || ctx.touch_tracker.gp_axis_rt > 0.4;
    let is_place = ctx.touch_tracker.btn_place || ctx.touch_tracker.gp_axis_lt > 0.4;

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
        key_ctrl: false,
        key_c: false,
        key_v: false,
        key_x: false,
        key_escape: ctx.touch_tracker.btn_menu_just,
        key_p: false,
        key_f: ctx.touch_tracker.btn_fly_just,
        key_e: ctx.touch_tracker.btn_inv_just,
        key_q: false,
        key_g: false,
        key_m: false,
        key_f3: false,
        key_f5: ctx.touch_tracker.btn_cam_just,
        key_enter,
        key_up: false,
        key_down: false,
        key_left: false,
        key_right: false,
        key_n: false,
        key_del: false,
        key_backspace,
        key_tab: false,
        key_t: ctx.touch_tracker.btn_chat_just,
        key_slash: false,
        key_digits: [false; 10],
        typed_chars,

        gp_left_stick_x: stick_x,
        gp_left_stick_y: stick_y,
        gp_right_stick_x: ctx.touch_tracker.gp_axis_rx,
        gp_right_stick_y: ctx.touch_tracker.gp_axis_ry,
        gp_left_trigger: if is_place { 1.0 } else { 0.0 },
        gp_right_trigger: if is_mine { 1.0 } else { 0.0 },
        gp_btn_south: ctx.touch_tracker.btn_jump,
        gp_btn_east: ctx.touch_tracker.btn_sneak,
        gp_btn_west: false,
        gp_btn_north: ctx.touch_tracker.btn_inv_just,
        gp_btn_lb: false,
        gp_btn_rb: false,
        gp_btn_l3: false,
        gp_btn_r3: false,
        gp_btn_start: ctx.touch_tracker.btn_menu_just,
        gp_btn_select: false,
        gp_dpad_up: ctx.touch_tracker.gp_axis_hat_y < -0.5,
        gp_dpad_down: ctx.touch_tracker.gp_axis_hat_y > 0.5,
        gp_dpad_left: ctx.touch_tracker.gp_axis_hat_x < -0.5,
        gp_dpad_right: ctx.touch_tracker.gp_axis_hat_x > 0.5,

        gp_south_just_pressed: false,
        gp_east_just_pressed: false,
        gp_west_just_pressed: false,
        gp_north_just_pressed: false,
        gp_lb_just_pressed: false,
        gp_rb_just_pressed: false,
        gp_r3_just_pressed: false,
        gp_start_just_pressed: ctx.touch_tracker.btn_menu_just,
        gp_select_just_pressed: false,
        gp_dpad_up_just_pressed: false,
        gp_dpad_down_just_pressed: false,
        gp_dpad_left_just_pressed: false,
        gp_dpad_right_just_pressed: false,
        gp_menu_up: false,
        gp_menu_down: false,
        gp_menu_left: false,
        gp_menu_right: false,
        gp_right_trigger_just_pressed: false,
        gp_left_trigger_just_pressed: false,

        is_mouse_captured: !in_menu,
        gp_connected: false,
    };

    // Hotbar selection from touch
    if let Some(slot) = ctx.touch_tracker.selected_hotbar.take() {
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
                for y in 0..src_h {
                    let src_row = &pixel_data[y * ctx.width..y * ctx.width + src_w];
                    let dst_row = std::slice::from_raw_parts_mut(dst_ptr.add(y * dst_stride), src_w);
                    for (x, &px) in src_row.iter().enumerate() {
                        let r = (px >> 16) & 0xFF;
                        let g = (px >> 8) & 0xFF;
                        let b = px & 0xFF;
                        dst_row[x] = 0xFF000000 | (b << 16) | (g << 8) | r;
                    }
                }
            }
            ANativeWindow_unlockAndPost(ctx.native_window);
        }
    }

    // Return status flags to Java:
    // Bit 0 (1): Text input active (chat, nickname, world creation - show soft keyboard)
    // Bit 1 (2): In menu state
    let mut flags: i32 = 0;
    if ctx.app.is_text_input_active() {
        flags |= 1;
    }
    if in_menu {
        flags |= 2;
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
