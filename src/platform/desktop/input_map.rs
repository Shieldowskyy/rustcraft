//! Translates `minifb` window keyboard, mouse, and `gilrs` gamepad state into `InputState`.

use std::collections::HashSet;

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window};

use crate::app::{InputState, HEIGHT, WIDTH};
use crate::core::controls::KeyCode;

use super::gamepad::{GamepadEvents, GamepadState};
use super::mouse_capture::MouseCapture;

pub(super) fn collect_typed_chars(window: &Window) -> Vec<char> {
    let is_shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
    let is_alt = window.is_key_down(Key::LeftAlt) || window.is_key_down(Key::RightAlt);
    let is_ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
    let is_altgr = window.is_key_down(Key::RightAlt) || (is_alt && is_ctrl);
    let is_pure_ctrl = is_ctrl && !is_altgr;

    let mut typed_chars = Vec::new();
    if is_pure_ctrl {
        return typed_chars;
    }

    for key in window.get_keys_pressed(KeyRepeat::Yes) {
        if is_altgr {
            match key {
                Key::A => typed_chars.push(if is_shift { 'Ą' } else { 'ą' }),
                Key::C => typed_chars.push(if is_shift { 'Ć' } else { 'ć' }),
                Key::E => typed_chars.push(if is_shift { 'Ę' } else { 'ę' }),
                Key::L => typed_chars.push(if is_shift { 'Ł' } else { 'ł' }),
                Key::N => typed_chars.push(if is_shift { 'Ń' } else { 'ń' }),
                Key::O => typed_chars.push(if is_shift { 'Ó' } else { 'ó' }),
                Key::S => typed_chars.push(if is_shift { 'Ś' } else { 'ś' }),
                Key::X => typed_chars.push(if is_shift { 'Ź' } else { 'ź' }),
                Key::Z => typed_chars.push(if is_shift { 'Ż' } else { 'ż' }),
                _ => {}
            }
            continue;
        }

        macro_rules! push_typed {
            ($k:ident, $($v:ident => ($lo:literal, $hi:literal)),* $(,)?) => {
                match $k {
                    $(Key::$v => typed_chars.push(if is_shift { $hi } else { $lo }),)*
                    Key::Space => typed_chars.push(' '),
                    Key::Minus | Key::NumPadMinus => typed_chars.push(if is_shift { '_' } else { '-' }),
                    Key::Period | Key::NumPadDot => typed_chars.push('.'),
                    Key::Slash | Key::NumPadSlash => typed_chars.push(if is_shift { '?' } else { '/' }),
                    Key::Equal | Key::NumPadPlus => typed_chars.push(if is_shift { '+' } else { '=' }),
                    _ => {}
                }
            };
        }
        push_typed!(key,
            A => ('a', 'A'), B => ('b', 'B'), C => ('c', 'C'), D => ('d', 'D'), E => ('e', 'E'), F => ('f', 'F'),
            G => ('g', 'G'), H => ('h', 'H'), I => ('i', 'I'), J => ('j', 'J'), K => ('k', 'K'), L => ('l', 'L'),
            M => ('m', 'M'), N => ('n', 'N'), O => ('o', 'O'), P => ('p', 'P'), Q => ('q', 'Q'), R => ('r', 'R'),
            S => ('s', 'S'), T => ('t', 'T'), U => ('u', 'U'), V => ('v', 'V'), W => ('w', 'W'), X => ('x', 'X'),
            Y => ('y', 'Y'), Z => ('z', 'Z'),
            Key0 => ('0', ')'), Key1 => ('1', '!'), Key2 => ('2', '@'), Key3 => ('3', '#'), Key4 => ('4', '$'),
            Key5 => ('5', '%'), Key6 => ('6', '^'), Key7 => ('7', '&'), Key8 => ('8', '*'), Key9 => ('9', '('),
            Semicolon => (';', ':'), Comma => (',', '<'), Apostrophe => ('\'', '"'), Backslash => ('\\', '|'),
            LeftBracket => ('[', '{'), RightBracket => (']', '}'), Backquote => ('`', '~'),
            NumPad0 => ('0', '0'), NumPad1 => ('1', '1'), NumPad2 => ('2', '2'), NumPad3 => ('3', '3'), NumPad4 => ('4', '4'),
            NumPad5 => ('5', '5'), NumPad6 => ('6', '6'), NumPad7 => ('7', '7'), NumPad8 => ('8', '8'), NumPad9 => ('9', '9'),
        );
    }

    typed_chars
}

pub(super) fn collect_keys_down(window: &Window, mouse_capture: &MouseCapture) -> HashSet<KeyCode> {
    let mut keys_down = HashSet::new();
    for key in window.get_keys() {
        macro_rules! map_minifb_keys {
            ($k:ident, $($same:ident),* ; $($from:ident => [$($to:ident),+]),* $(,)?) => {
                match $k {
                    $(Key::$same => { keys_down.insert(KeyCode::$same); })*
                    $(Key::$from => { $(keys_down.insert(KeyCode::$to);)+ })*
                    _ => {}
                }
            };
        }
        map_minifb_keys!(key,
            A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
            Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
            F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12, F13, F14, F15,
            Space, Tab, Enter, Escape, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
            Up, Down, Left, Right, Minus, Equal, LeftBracket, RightBracket, Semicolon, Apostrophe,
            Backslash, Slash, Comma, Period,
            NumPad0, NumPad1, NumPad2, NumPad3, NumPad4, NumPad5, NumPad6, NumPad7, NumPad8, NumPad9,
            NumPadDot, NumPadSlash, NumPadAsterisk, NumPadMinus, NumPadPlus, NumPadEnter;
            Backquote => [Grave],
            LeftShift => [Shift, LeftShift],
            RightShift => [Shift, RightShift],
            LeftCtrl => [Ctrl, LeftCtrl],
            RightCtrl => [Ctrl, RightCtrl],
            LeftAlt => [Alt, LeftAlt],
            RightAlt => [Alt, RightAlt],
        );
    }
    mouse_capture.poll_extra_keys(&mut keys_down);
    keys_down
}

pub(super) fn build_input_state(
    window: &Window,
    win_size: (usize, usize),
    mouse_capture: &MouseCapture,
    last_mouse: &mut Option<(f32, f32)>,
    gp: GamepadState,
    gp_events: GamepadEvents,
) -> InputState {
    let raw_mouse_pos = window.get_mouse_pos(MouseMode::Pass);
    let mouse_pos = raw_mouse_pos.map(|(mx, my)| {
        let fx = if win_size.0 > 0 { mx * (WIDTH as f32) / (win_size.0 as f32) } else { mx };
        let fy = if win_size.1 > 0 { my * (HEIGHT as f32) / (win_size.1 as f32) } else { my };
        (fx, fy)
    });

    let mut mouse_dx = 0.0;
    let mut mouse_dy = 0.0;
    if mouse_capture.is_captured() {
        if mouse_capture.has_native_capture() {
            let (dx, dy) = mouse_capture.poll_delta();
            mouse_dx = dx;
            mouse_dy = dy;
        } else if let Some((mx, my)) = raw_mouse_pos {
            if let Some((lx, ly)) = *last_mouse {
                mouse_dx = mx - lx;
                mouse_dy = my - ly;
            }
            *last_mouse = Some((mx, my));
        }
    }

    let scroll_delta = window.get_scroll_wheel().map(|(_sx, sy)| sy).unwrap_or(0.0);
    let is_shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
    let is_ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
    let typed_chars = collect_typed_chars(window);
    let keys_down = collect_keys_down(window, mouse_capture);
    let (extra_mouse4, extra_mouse5, extra_mouse_arr) = mouse_capture.poll_extra_buttons();

    InputState {
        gp_connected: gp.is_connected,
        mouse_dx,
        mouse_dy,
        mouse_pos,
        mouse_left: window.get_mouse_down(MouseButton::Left),
        mouse_right: window.get_mouse_down(MouseButton::Right),
        mouse_middle: window.get_mouse_down(MouseButton::Middle),
        mouse_4: extra_mouse4,
        mouse_5: extra_mouse5,
        mouse_extra: extra_mouse_arr,
        scroll_delta,
        keys_down,

        key_w: window.is_key_down(Key::W),
        key_s: window.is_key_down(Key::S),
        key_a: window.is_key_down(Key::A),
        key_d: window.is_key_down(Key::D),
        key_space: window.is_key_down(Key::Space),
        key_shift: is_shift,
        key_ctrl: is_ctrl,
        key_c: window.is_key_down(Key::C),
        key_v: window.is_key_down(Key::V),
        key_x: window.is_key_down(Key::X),
        key_escape: window.is_key_down(Key::Escape),
        key_p: window.is_key_down(Key::P),
        key_f: window.is_key_down(Key::F),
        key_e: window.is_key_down(Key::E),
        key_q: window.is_key_down(Key::Q),
        key_g: window.is_key_down(Key::G),
        key_m: window.is_key_down(Key::M),
        key_f3: window.is_key_down(Key::F3),
        key_f5: window.is_key_down(Key::F5),
        key_enter: window.is_key_down(Key::Enter),
        key_up: window.is_key_down(Key::Up),
        key_down: window.is_key_down(Key::Down),
        key_left: window.is_key_down(Key::Left),
        key_right: window.is_key_down(Key::Right),
        key_n: window.is_key_down(Key::N),
        key_b: window.is_key_down(Key::B),
        key_u: window.is_key_down(Key::U),
        key_del: window.is_key_down(Key::Delete),
        key_backspace: window.is_key_down(Key::Backspace),
        key_tab: window.is_key_down(Key::Tab),
        key_t: window.is_key_down(Key::T),
        key_slash: window.is_key_down(Key::Slash) || window.is_key_down(Key::NumPadSlash),
        key_digits: [
            window.is_key_down(Key::Key1),
            window.is_key_down(Key::Key2),
            window.is_key_down(Key::Key3),
            window.is_key_down(Key::Key4),
            window.is_key_down(Key::Key5),
            window.is_key_down(Key::Key6),
            window.is_key_down(Key::Key7),
            window.is_key_down(Key::Key8),
            window.is_key_down(Key::Key9),
            window.is_key_down(Key::Key0),
        ],
        typed_chars,

        gp_left_stick_x: gp.left_stick_x,
        gp_left_stick_y: gp.left_stick_y,
        gp_right_stick_x: gp.right_stick_x,
        gp_right_stick_y: gp.right_stick_y,
        gp_left_trigger: gp.left_trigger,
        gp_right_trigger: gp.right_trigger,
        gp_btn_south: gp.btn_south,
        gp_btn_east: gp.btn_east,
        gp_btn_west: gp.btn_west,
        gp_btn_north: gp.btn_north,
        gp_btn_lb: gp.btn_lb,
        gp_btn_rb: gp.btn_rb,
        gp_btn_l3: gp.btn_l3,
        gp_btn_r3: gp.btn_r3,
        gp_btn_start: gp.btn_start,
        gp_btn_select: gp.btn_select,
        gp_dpad_up: gp.dpad_up,
        gp_dpad_down: gp.dpad_down,
        gp_dpad_left: gp.dpad_left,
        gp_dpad_right: gp.dpad_right,

        gp_south_just_pressed: gp_events.south_just_pressed,
        gp_east_just_pressed: gp_events.east_just_pressed,
        gp_west_just_pressed: gp_events.west_just_pressed,
        gp_north_just_pressed: gp_events.north_just_pressed,
        gp_lb_just_pressed: gp_events.lb_just_pressed,
        gp_rb_just_pressed: gp_events.rb_just_pressed,
        gp_r3_just_pressed: gp_events.r3_just_pressed,
        gp_start_just_pressed: gp_events.start_just_pressed,
        gp_select_just_pressed: gp_events.select_just_pressed,
        gp_dpad_up_just_pressed: gp_events.dpad_up_just_pressed,
        gp_dpad_down_just_pressed: gp_events.dpad_down_just_pressed,
        gp_dpad_left_just_pressed: gp_events.dpad_left_just_pressed,
        gp_dpad_right_just_pressed: gp_events.dpad_right_just_pressed,
        gp_menu_up: gp_events.menu_up,
        gp_menu_down: gp_events.menu_down,
        gp_menu_left: gp_events.menu_left,
        gp_menu_right: gp_events.menu_right,
        gp_right_trigger_just_pressed: gp_events.right_trigger_just_pressed,
        gp_left_trigger_just_pressed: gp_events.left_trigger_just_pressed,

        is_mouse_captured: mouse_capture.is_captured(),
    }
}

