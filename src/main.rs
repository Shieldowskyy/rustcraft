#[cfg(not(target_os = "vita"))]
use minifb::{CursorStyle, Key, MouseButton, MouseMode, Window, WindowOptions};
#[cfg(not(target_os = "vita"))]
use std::time::Instant;

#[cfg(not(target_os = "vita"))]
use rustcraft::app::{InputState, RustCraftApp, HEIGHT, WIDTH};
#[cfg(not(target_os = "vita"))]
use rustcraft::cli::{parse_renderer_choice, parse_seed};
#[cfg(not(target_os = "vita"))]
use rustcraft::platform::gamepad::GamepadManager;
#[cfg(not(target_os = "vita"))]
use rustcraft::platform::mouse_capture::{self, MouseCapture};


#[cfg(target_os = "vita")]
fn main() {
    rustcraft::platform::vita::run_vita_app();
}

#[cfg(not(target_os = "vita"))]
fn main() {
    let renderer_choice = parse_renderer_choice();
    let seed = parse_seed();

    #[cfg(target_os = "linux")]
    if std::env::var_os("DISPLAY").is_some() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let mut window = Window::new(
        &format!("RustCraft - {} renderer", renderer_choice),
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("Failed to create native desktop window");

    let mut app = RustCraftApp::new(WIDTH, HEIGHT, renderer_choice, seed);
    let mut gamepad_mgr = GamepadManager::new();

    mouse_capture::maximize_window(&window);
    let mut current_target_fps = app.settings.fps_limit;
    window.set_target_fps(if current_target_fps == 0 { 0 } else { current_target_fps as usize });
    window.set_cursor_visibility(false);
    let _ = window.set_cursor_style(CursorStyle::Crosshair);

    let mut win_size = window.get_size();
    let mut mouse_capture = MouseCapture::new(&window, win_size.0 as i32, win_size.1 as i32);
    mouse_capture.set_captured(false);

    let mut last_time = Instant::now();
    let mut fps_timer = Instant::now();
    let mut frame_count = 0u32;
    let mut last_title = String::new();
    let mut last_mouse: Option<(f32, f32)> = None;

    println!("Controls (Keyboard / Mouse):");
    println!("  Click in window - Capture mouse");
    println!("  Esc             - Pause menu / Release mouse");
    println!("  WASD            - Movement");
    println!("  Space           - Jump (Fly: ascend)");
    println!("  Left Shift      - Sneak (Fly: descend)");
    println!("  Left Ctrl       - Sprint");
    println!("  F               - Toggle flight mode");
    println!("  Mouse           - Look around");
    println!("  LMB             - Mine block / Attack / UI Click");
    println!("  RMB             - Place block / Interact");
    println!("  Scroll / 1-9    - Hotbar slot selection");
    println!("  Q               - Drop item / Back in menu");
    println!("Controls (Controller / Gamepad):");
    println!("  Left Stick      - Movement");
    println!("  Right Stick     - Camera look");
    println!("  RT (R2)         - Mine block / Attack");
    println!("  LT (L2)         - Place block");
    println!("  A (Cross)       - Jump / Confirm in UI");
    println!("  B (Circle)      - Sneak / Back in UI");
    println!("  Y (Triangle)    - Inventory");
    println!("  X (Square)      - Drop item");
    println!("  LB / RB         - Previous / Next hotbar slot");
    println!("  L3 (Left Stick) - Sprint");
    println!("  R3 (Right Stick)- Flight mode");
    println!("  Start           - Pause");
    println!("  Select / Back   - Toggle GameMode (Survival/Creative)");
    println!("  D-Pad           - Menu & inventory navigation");

    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32().min(0.1);
        last_time = now;

        let gp_events = gamepad_mgr.update(dt);
        let gp = gamepad_mgr.state;

        let cur_size = window.get_size();
        if cur_size != win_size {
            win_size = cur_size;
            mouse_capture.resize(win_size.0 as i32, win_size.1 as i32);
        }

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
                if let Some((lx, ly)) = last_mouse {
                    mouse_dx = mx - lx;
                    mouse_dy = my - ly;
                }
                last_mouse = Some((mx, my));
            }
        }

        let scroll_wheel = window.get_scroll_wheel();
        let scroll_delta = scroll_wheel.map(|(_sx, sy)| sy).unwrap_or(0.0);

        let is_shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
        let is_ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        let mut typed_chars = Vec::new();
        if !is_ctrl {
            for key in window.get_keys_pressed(minifb::KeyRepeat::Yes) {
                match key {
                    Key::A => typed_chars.push(if is_shift { 'A' } else { 'a' }),
                    Key::B => typed_chars.push(if is_shift { 'B' } else { 'b' }),
                    Key::C => typed_chars.push(if is_shift { 'C' } else { 'c' }),
                    Key::D => typed_chars.push(if is_shift { 'D' } else { 'd' }),
                    Key::E => typed_chars.push(if is_shift { 'E' } else { 'e' }),
                    Key::F => typed_chars.push(if is_shift { 'F' } else { 'f' }),
                    Key::G => typed_chars.push(if is_shift { 'G' } else { 'g' }),
                    Key::H => typed_chars.push(if is_shift { 'H' } else { 'h' }),
                    Key::I => typed_chars.push(if is_shift { 'I' } else { 'i' }),
                    Key::J => typed_chars.push(if is_shift { 'J' } else { 'j' }),
                    Key::K => typed_chars.push(if is_shift { 'K' } else { 'k' }),
                    Key::L => typed_chars.push(if is_shift { 'L' } else { 'l' }),
                    Key::M => typed_chars.push(if is_shift { 'M' } else { 'm' }),
                    Key::N => typed_chars.push(if is_shift { 'N' } else { 'n' }),
                    Key::O => typed_chars.push(if is_shift { 'O' } else { 'o' }),
                    Key::P => typed_chars.push(if is_shift { 'P' } else { 'p' }),
                    Key::Q => typed_chars.push(if is_shift { 'Q' } else { 'q' }),
                    Key::R => typed_chars.push(if is_shift { 'R' } else { 'r' }),
                    Key::S => typed_chars.push(if is_shift { 'S' } else { 's' }),
                    Key::T => typed_chars.push(if is_shift { 'T' } else { 't' }),
                    Key::U => typed_chars.push(if is_shift { 'U' } else { 'u' }),
                    Key::V => typed_chars.push(if is_shift { 'V' } else { 'v' }),
                    Key::W => typed_chars.push(if is_shift { 'W' } else { 'w' }),
                    Key::X => typed_chars.push(if is_shift { 'X' } else { 'x' }),
                    Key::Y => typed_chars.push(if is_shift { 'Y' } else { 'y' }),
                    Key::Z => typed_chars.push(if is_shift { 'Z' } else { 'z' }),
                    Key::Key0 => typed_chars.push(if is_shift { ')' } else { '0' }),
                    Key::Key1 => typed_chars.push(if is_shift { '!' } else { '1' }),
                    Key::Key2 => typed_chars.push(if is_shift { '@' } else { '2' }),
                    Key::Key3 => typed_chars.push(if is_shift { '#' } else { '3' }),
                    Key::Key4 => typed_chars.push(if is_shift { '$' } else { '4' }),
                    Key::Key5 => typed_chars.push(if is_shift { '%' } else { '5' }),
                    Key::Key6 => typed_chars.push(if is_shift { '^' } else { '6' }),
                    Key::Key7 => typed_chars.push(if is_shift { '&' } else { '7' }),
                    Key::Key8 => typed_chars.push(if is_shift { '*' } else { '8' }),
                    Key::Key9 => typed_chars.push(if is_shift { '(' } else { '9' }),
                    Key::Space => typed_chars.push(' '),
                    Key::Minus => typed_chars.push(if is_shift { '_' } else { '-' }),
                    Key::Period => typed_chars.push('.'),
                    _ => {}
                }
            }
        }

        let input = InputState {
            mouse_dx,
            mouse_dy,
            mouse_pos,
            mouse_left: window.get_mouse_down(MouseButton::Left),
            mouse_right: window.get_mouse_down(MouseButton::Right),
            scroll_delta,

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
            key_f5: window.is_key_down(Key::F5),
            key_enter: window.is_key_down(Key::Enter),
            key_up: window.is_key_down(Key::Up),
            key_down: window.is_key_down(Key::Down),
            key_left: window.is_key_down(Key::Left),
            key_right: window.is_key_down(Key::Right),
            key_n: window.is_key_down(Key::N),
            key_del: window.is_key_down(Key::Delete),
            key_backspace: window.is_key_down(Key::Backspace),
            key_tab: window.is_key_down(Key::Tab),
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
            gp_right_trigger_just_pressed: gp_events.right_trigger_just_pressed,
            gp_left_trigger_just_pressed: gp_events.left_trigger_just_pressed,

            is_mouse_captured: mouse_capture.is_captured(),
        };

        let out = app.update_and_render(dt, &input);

        if out.request_capture_mouse && !mouse_capture.is_captured() {
            mouse_capture.set_captured(true);
            window.set_cursor_visibility(false);
            last_mouse = None;
        }
        if out.request_release_mouse && mouse_capture.is_captured() {
            mouse_capture.set_captured(false);
            window.set_cursor_visibility(true);
        }
        if out.quit_requested {
            break;
        }

        if app.settings.fps_limit != current_target_fps {
            current_target_fps = app.settings.fps_limit;
            window.set_target_fps(if current_target_fps == 0 { 0 } else { current_target_fps as usize });
        }

        window
            .update_with_buffer(app.pixel_buffer(), WIDTH, HEIGHT)
            .expect("Failed to update window pixel buffer");

        frame_count += 1;
        if fps_timer.elapsed().as_secs_f32() >= 0.5 {
            app.fps_display = (frame_count as f32 / fps_timer.elapsed().as_secs_f32()) as u32;
            frame_count = 0;
            fps_timer = Instant::now();
        }

        if out.title != last_title {
            window.set_title(&out.title);
            last_title = out.title;
        }
    }
}
