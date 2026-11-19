//! PS Vita gamepad (`SceCtrl`), analog sticks, front/back touch, and `InputState` mapper.
//! Translates hardware pad & touch reports into engine-neutral `InputState`.

#![allow(dead_code)]

use crate::app::InputState;

use super::ffi::*;

// ============================================================================
// 3. Vita Input & Controls Tracker
// ============================================================================

pub struct VitaInputTracker {
    prev_buttons: u32,
    prev_touch_count: u32,
    touch_buf: Box<SceTouchData>,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub cursor_active: bool,
}

impl VitaInputTracker {
    pub fn new() -> Self {
        // Enable analog stick sampling and touchscreen reporting
        unsafe {
            sceCtrlSetSamplingMode(SCE_CTRL_MODE_ANALOG_WIDE);
            sceTouchSetSamplingState(SCE_TOUCH_PORT_FRONT, 1);
        }
        Self {
            prev_buttons: 0,
            prev_touch_count: 0,
            touch_buf: Box::new(SceTouchData::default()),
            cursor_x: 480.0,
            cursor_y: 272.0,
            cursor_active: false,
        }
    }

    pub fn poll(&mut self, is_mouse_captured: bool, dt: f32) -> InputState {
        let mut pad = SceCtrlData::default();
        unsafe {
            sceCtrlPeekBufferPositive(0, &mut pad, 1);
        }

        let buttons = pad.buttons;
        let prev_buttons = self.prev_buttons;
        let just_pressed = move |mask: u32| -> bool { (buttons & mask != 0) && (prev_buttons & mask == 0) };

        // Analog sticks (0..255 range, 128 is center)
        let apply_deadzone = |val: u8| -> f32 {
            let f = (val as f32 - 128.0) / 128.0;
            if f.abs() < 0.18 {
                0.0
            } else {
                f
            }
        };

        let left_x = apply_deadzone(pad.lx);
        let left_y = apply_deadzone(pad.ly);
        let right_x = apply_deadzone(pad.rx);
        let right_y = apply_deadzone(pad.ry);

        // Map PS Vita buttons to action flags
        let btn_cross = buttons & SCE_CTRL_CROSS != 0;
        let btn_circle = buttons & SCE_CTRL_CIRCLE != 0;
        let btn_square = buttons & SCE_CTRL_SQUARE != 0;
        let btn_triangle = buttons & SCE_CTRL_TRIANGLE != 0;
        let btn_l = buttons & SCE_CTRL_LTRIGGER != 0;
        let btn_r = buttons & SCE_CTRL_RTRIGGER != 0;
        let btn_start = buttons & SCE_CTRL_START != 0;
        let btn_select = buttons & SCE_CTRL_SELECT != 0;

        let dpad_up = buttons & SCE_CTRL_UP != 0;
        let dpad_down = buttons & SCE_CTRL_DOWN != 0;
        let dpad_left = buttons & SCE_CTRL_LEFT != 0;
        let dpad_right = buttons & SCE_CTRL_RIGHT != 0;
        let dpad_any_just = just_pressed(SCE_CTRL_UP | SCE_CTRL_DOWN | SCE_CTRL_LEFT | SCE_CTRL_RIGHT);

        // Poll front capacitive touchscreen using heap-allocated buffer of full 64 reports
        self.touch_buf.report_num = 0;
        let touch_res = unsafe {
            sceTouchPeek(SCE_TOUCH_PORT_FRONT, &mut *self.touch_buf, 1)
        };

        let report_num = if touch_res >= 0 {
            self.touch_buf.report_num
        } else {
            0
        };

        let touch_active = report_num > 0;
        let touch_just_pressed = report_num > 0 && self.prev_touch_count == 0;

        if touch_active {
            // Read packed coordinates safely without unaligned reference UB
            let raw_x = self.touch_buf.report[0].x;
            let raw_y = self.touch_buf.report[0].y;
            let tx = (raw_x as f32 / 1919.0) * 960.0;
            let ty = (raw_y as f32 / 1087.0) * 544.0;
            self.cursor_x = tx.clamp(5.0, 955.0);
            self.cursor_y = ty.clamp(5.0, 539.0);
            self.cursor_active = true;
        } else if !is_mouse_captured {
            // Smooth cursor movement with analog sticks in menus / inventory
            let cursor_speed = 650.0;
            let move_x = left_x + right_x;
            let move_y = left_y + right_y;
            if move_x.abs() > 0.08 || move_y.abs() > 0.08 {
                self.cursor_x = (self.cursor_x + move_x * cursor_speed * dt).clamp(5.0, 955.0);
                self.cursor_y = (self.cursor_y + move_y * cursor_speed * dt).clamp(5.0, 539.0);
                self.cursor_active = true;
            }

            // D-Pad navigation deactivates free mouse cursor to avoid duplicate selection highlight
            if dpad_any_just {
                self.cursor_active = false;
            }
        }

        let mouse_pos = if is_mouse_captured || !self.cursor_active {
            None
        } else {
            Some((self.cursor_x, self.cursor_y))
        };

        let mouse_left = if is_mouse_captured {
            btn_r || touch_active
        } else {
            touch_just_pressed || (self.cursor_active && btn_r)
        };

        let mouse_right = if is_mouse_captured {
            btn_l
        } else {
            self.cursor_active && btn_l
        };

        let mouse_dx = 0.0;
        let mouse_dy = 0.0;
        let scroll_delta = 0.0;

        self.prev_buttons = buttons;
        self.prev_touch_count = report_num;

        InputState {
            mouse_dx,
            mouse_dy,
            mouse_pos,
            mouse_left,
            mouse_right,
            scroll_delta,

            key_w: dpad_up,
            key_s: dpad_down,
            key_a: dpad_left,
            key_d: dpad_right,
            key_space: btn_cross,
            key_shift: btn_circle,
            key_ctrl: false,
            key_c: false,
            key_v: false,
            key_x: false,
            key_escape: btn_start,
            key_p: btn_start,
            key_f: false,
            key_e: btn_triangle,
            key_q: btn_square,
            key_g: false,
            key_m: false,
            key_f3: false,
            key_f5: just_pressed(SCE_CTRL_SQUARE),
            key_enter: just_pressed(SCE_CTRL_CROSS),
            key_up: dpad_up,
            key_down: dpad_down,
            key_left: dpad_left,
            key_right: dpad_right,
            key_n: false,
            key_b: false,
            key_u: false,
            key_del: btn_circle,
            key_backspace: btn_circle,
            key_tab: false,
            key_t: false,
            key_slash: false,
            key_digits: [
                false, false, false, false, false, false, false, false, false, false,
            ],
            typed_chars: Vec::new(),

            gp_left_stick_x: left_x,
            gp_left_stick_y: -left_y,
            gp_right_stick_x: right_x,
            gp_right_stick_y: -right_y,
            gp_left_trigger: if btn_l { 1.0 } else { 0.0 },
            gp_right_trigger: if btn_r { 1.0 } else { 0.0 },
            gp_btn_south: btn_cross,
            gp_btn_east: btn_circle,
            gp_btn_west: btn_square,
            gp_btn_north: btn_triangle,
            gp_btn_lb: false,
            gp_btn_rb: false,
            gp_btn_l3: false,
            gp_btn_r3: false,
            gp_btn_start: btn_start,
            gp_btn_select: btn_select,
            gp_dpad_up: dpad_up,
            gp_dpad_down: dpad_down,
            gp_dpad_left: dpad_left,
            gp_dpad_right: dpad_right,

            gp_south_just_pressed: just_pressed(SCE_CTRL_CROSS),
            gp_east_just_pressed: just_pressed(SCE_CTRL_CIRCLE),
            gp_west_just_pressed: just_pressed(SCE_CTRL_SQUARE),
            gp_north_just_pressed: just_pressed(SCE_CTRL_TRIANGLE),
            gp_lb_just_pressed: just_pressed(SCE_CTRL_LEFT),
            gp_rb_just_pressed: just_pressed(SCE_CTRL_RIGHT),
            gp_r3_just_pressed: false,
            gp_start_just_pressed: just_pressed(SCE_CTRL_START),
            gp_select_just_pressed: just_pressed(SCE_CTRL_SELECT),
            gp_dpad_up_just_pressed: just_pressed(SCE_CTRL_UP),
            gp_dpad_down_just_pressed: just_pressed(SCE_CTRL_DOWN),
            gp_dpad_left_just_pressed: just_pressed(SCE_CTRL_LEFT),
            gp_dpad_right_just_pressed: just_pressed(SCE_CTRL_RIGHT),
            gp_menu_up: just_pressed(SCE_CTRL_UP),
            gp_menu_down: just_pressed(SCE_CTRL_DOWN),
            gp_menu_left: just_pressed(SCE_CTRL_LEFT),
            gp_menu_right: just_pressed(SCE_CTRL_RIGHT),
            gp_right_trigger_just_pressed: just_pressed(SCE_CTRL_RTRIGGER),
            gp_left_trigger_just_pressed: just_pressed(SCE_CTRL_LTRIGGER),

            is_mouse_captured,
            gp_connected: true,
            ..Default::default()
        }
    }
}

