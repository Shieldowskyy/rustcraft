//! Android JNI input, keyboard/IME, lifecycle pause/destroy, and renderer query entrypoints.
//! Bridges Java Activity motion/key/text callbacks into `GAME_INSTANCE`.

#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::{CStr, CString};

use jni_sys::{jboolean, jclass, jfloat, jint, jstring, JNIEnv, JNI_FALSE, JNI_TRUE};

use crate::app::GameState;

use super::ffi::*;
use super::lifecycle::GAME_INSTANCE;

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
                KEYCODE_W | KEYCODE_DPAD_UP => ctx.touch_tracker.key_move_forward = down,
                KEYCODE_S | KEYCODE_DPAD_DOWN => ctx.touch_tracker.key_move_backward = down,
                KEYCODE_A | KEYCODE_DPAD_LEFT => ctx.touch_tracker.key_move_left = down,
                KEYCODE_D | KEYCODE_DPAD_RIGHT => ctx.touch_tracker.key_move_right = down,
                KEYCODE_SPACE | KEYCODE_BUTTON_A => ctx.touch_tracker.key_jump = down,
                KEYCODE_BUTTON_B => ctx.touch_tracker.key_sneak = down,
                KEYCODE_BUTTON_X => {
                    if down {
                        ctx.touch_tracker.btn_sprint = !ctx.touch_tracker.btn_sprint;
                    }
                }
                KEYCODE_BUTTON_R1 => ctx.touch_tracker.key_mine = down,
                KEYCODE_BUTTON_L1 => ctx.touch_tracker.key_place_held = down,
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
