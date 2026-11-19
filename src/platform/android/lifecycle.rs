//! Android native application context, surface attachment, and frame step JNI entrypoints.
//! Owns global `GAME_INSTANCE` state and drives `RustCraftApp::update_and_render` per frame.

#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CStr;
use std::sync::Mutex;
use std::time::Instant;

use jni_sys::{jboolean, jclass, jfloat, jint, jobject, jstring, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::app::{GameState, InputState, RustCraftApp, HEIGHT, WIDTH};

use super::ffi::*;
use super::touch::TouchTracker;

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

pub(super) static GAME_INSTANCE: Mutex<Option<AndroidGameContext>> = Mutex::new(None);

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

    let move_fwd = ctx.touch_tracker.key_move_forward
        || ctx.touch_tracker.move_forward
        || stick_y < -0.25;
    let move_back = ctx.touch_tracker.key_move_backward
        || ctx.touch_tracker.move_backward
        || stick_y > 0.25;
    let move_l = ctx.touch_tracker.key_move_left || ctx.touch_tracker.move_left || stick_x < -0.25;
    let move_r = ctx.touch_tracker.key_move_right || ctx.touch_tracker.move_right || stick_x > 0.25;

    // Physical gamepad right stick camera rotation
    if ctx.touch_tracker.gp_axis_rx.abs() > 0.15 {
        ctx.touch_tracker.mouse_dx += ctx.touch_tracker.gp_axis_rx * 14.0;
    }
    if ctx.touch_tracker.gp_axis_ry.abs() > 0.15 {
        ctx.touch_tracker.mouse_dy += ctx.touch_tracker.gp_axis_ry * 14.0;
    }

    let is_mine = ctx.touch_tracker.btn_mine || ctx.touch_tracker.gp_axis_rt > 0.4;
    let is_place = ctx.touch_tracker.btn_place_held || ctx.touch_tracker.tap_place_pulse || ctx.touch_tracker.gp_axis_lt > 0.4;
    // Reset tap place pulse after applying
    ctx.touch_tracker.tap_place_pulse = false;

    // Swap offhand when tapping the offhand HUD slot
    if ctx.touch_tracker.offhand_swap_just {
        ctx.touch_tracker.offhand_swap_just = false;
        let sel = ctx.app.player.selected_hotbar_slot as usize;
        std::mem::swap(&mut ctx.app.player.inventory.hotbar[sel], &mut ctx.app.player.inventory.offhand);
        ctx.app.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
    }

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
        mouse_left: is_mine || (in_menu && ctx.touch_tracker.primary_touch_down && !ctx.touch_tracker.menu_right_click_held),
        mouse_right: is_place || (in_menu && ctx.touch_tracker.menu_right_click_held),
        scroll_delta: 0.0,

        key_w: move_fwd,
        key_s: move_back,
        key_a: move_l,
        key_d: move_r,
        key_space: ctx.touch_tracker.btn_jump,
        key_shift: ctx.touch_tracker.btn_sneak,
        key_ctrl: ctx.touch_tracker.btn_sprint || ctx.touch_tracker.stick_sprinting,
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
    let output = ctx.app.update_and_render(delta_time, &input);

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
    // Bit 3 (8): Quit game requested by user from menu (Activity should finish)
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
    if output.quit_requested {
        flags |= 8;
        let _ = crate::world::storage::save_current_world(
            &mut ctx.app.world,
            &ctx.app.camera,
            &ctx.app.player,
            ctx.app.save_slot,
            None,
        );
        android_log(ANDROID_LOG_INFO, "RustCraftNative", "Quit requested from menu. World saved.");
    }

    flags
}

