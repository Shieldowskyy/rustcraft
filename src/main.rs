#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use minifb::{CursorStyle, Key, MouseButton, MouseMode, Window, WindowOptions};
#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use std::time::Instant;

#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use rustcraft::app::{InputState, RustCraftApp, HEIGHT, WIDTH};
#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use rustcraft::cli::{parse_log_enabled, parse_profiler_enabled, parse_renderer_choice, parse_seed};
#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use rustcraft::core::{init_crash_reporter, init_logger, set_crash_context, CrashReporterConfig};
#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use rustcraft::platform::gamepad::GamepadManager;
#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use rustcraft::platform::mouse_capture::{self, MouseCapture};


#[cfg(target_os = "vita")]
fn main() {
    rustcraft::platform::vita::run_vita_app();
}

#[cfg(target_os = "android")]
fn main() {
    // Android entry points are in platform::android (JNI / cdylib)
}

#[cfg(any(target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360"))]
fn main() {
    rustcraft::platform::xbox360::run_xbox360_app();
}

#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
fn main() {
    let log_enabled = parse_log_enabled();
    init_logger(log_enabled);

    let sentry_dsn = rustcraft::cli::parse_sentry_dsn();
    let sentry_enabled = !rustcraft::cli::parse_sentry_disabled();
    let _crash_guard = init_crash_reporter(CrashReporterConfig {
        dsn: sentry_dsn,
        sentry_enabled,
        ..Default::default()
    });

    if rustcraft::cli::parse_crash_test() {
        rustcraft::log_warn!("TEST", "Manual crash reporter test triggered via --crash-test!");
        panic!("RustCraft intentional crash test via --crash-test CLI flag");
    }

    if rustcraft::cli::parse_server_mode() {
        let port = rustcraft::cli::parse_server_port();
        let seed = parse_seed();
        set_crash_context("mode", "dedicated_server");
        set_crash_context("port", &port.to_string());
        set_crash_context("seed", &seed.to_string());
        rustcraft::net::run_dedicated_server(port, seed);
        return;
    }

    let renderer_choice = parse_renderer_choice();
    let seed = parse_seed();
    let profiler_enabled = parse_profiler_enabled();

    set_crash_context("mode", "client");
    set_crash_context("renderer", renderer_choice);
    set_crash_context("seed", &seed.to_string());
    set_crash_context("profiler", &profiler_enabled.to_string());

    rustcraft::log_info!("MAIN", "Starting RustCraft (renderer: {}, seed: {}, profiler: {})", renderer_choice, seed, profiler_enabled);

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if std::env::var_os("DISPLAY").is_some() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    // Decode the first soundtrack before the window appears so it starts exactly with the intro.
    rustcraft::audio::preload_intro_music();

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

    rustcraft::log_info!("WINDOW", "Desktop window created ({}x{})", WIDTH, HEIGHT);

    let mut app = RustCraftApp::new(WIDTH, HEIGHT, renderer_choice, seed);
    app.profiler_enabled = profiler_enabled;

    if let Some(connect_addr) = rustcraft::cli::parse_connect_addr() {
        rustcraft::log_info!("MAIN", "Auto-connecting to multiplayer server: {}", connect_addr);
        app.connect_to_server(&connect_addr);
    }
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
    println!("  T               - Open chat");
    println!("  /               - Open command prompt");
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
            rustcraft::log_debug!("WINDOW", "Window resized from {}x{} to {}x{}", win_size.0, win_size.1, cur_size.0, cur_size.1);
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
        let is_alt = window.is_key_down(Key::LeftAlt) || window.is_key_down(Key::RightAlt);
        let is_ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        let is_altgr = window.is_key_down(Key::RightAlt) || (is_alt && is_ctrl);
        let is_pure_ctrl = is_ctrl && !is_altgr;
        let mut typed_chars = Vec::new();
        if !is_pure_ctrl {
            for key in window.get_keys_pressed(minifb::KeyRepeat::Yes) {
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
                } else {
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
                        Key::Minus | Key::NumPadMinus => typed_chars.push(if is_shift { '_' } else { '-' }),
                        Key::Period | Key::NumPadDot => typed_chars.push('.'),
                        Key::Slash | Key::NumPadSlash => typed_chars.push(if is_shift { '?' } else { '/' }),
                        Key::Semicolon => typed_chars.push(if is_shift { ':' } else { ';' }),
                        Key::Equal | Key::NumPadPlus => typed_chars.push(if is_shift { '+' } else { '=' }),
                        Key::Comma => typed_chars.push(if is_shift { '<' } else { ',' }),
                        Key::Apostrophe => typed_chars.push(if is_shift { '"' } else { '\'' }),
                        Key::Backslash => typed_chars.push(if is_shift { '|' } else { '\\' }),
                        Key::LeftBracket => typed_chars.push(if is_shift { '{' } else { '[' }),
                        Key::RightBracket => typed_chars.push(if is_shift { '}' } else { ']' }),
                        Key::Backquote => typed_chars.push(if is_shift { '~' } else { '`' }),
                        Key::NumPad0 => typed_chars.push('0'),
                        Key::NumPad1 => typed_chars.push('1'),
                        Key::NumPad2 => typed_chars.push('2'),
                        Key::NumPad3 => typed_chars.push('3'),
                        Key::NumPad4 => typed_chars.push('4'),
                        Key::NumPad5 => typed_chars.push('5'),
                        Key::NumPad6 => typed_chars.push('6'),
                        Key::NumPad7 => typed_chars.push('7'),
                        Key::NumPad8 => typed_chars.push('8'),
                        Key::NumPad9 => typed_chars.push('9'),
                        _ => {}
                    }
                }
            }
        }

        let mut keys_down = std::collections::HashSet::new();
        for key in window.get_keys() {
            match key {
                Key::A => { keys_down.insert(rustcraft::core::controls::KeyCode::A); }
                Key::B => { keys_down.insert(rustcraft::core::controls::KeyCode::B); }
                Key::C => { keys_down.insert(rustcraft::core::controls::KeyCode::C); }
                Key::D => { keys_down.insert(rustcraft::core::controls::KeyCode::D); }
                Key::E => { keys_down.insert(rustcraft::core::controls::KeyCode::E); }
                Key::F => { keys_down.insert(rustcraft::core::controls::KeyCode::F); }
                Key::G => { keys_down.insert(rustcraft::core::controls::KeyCode::G); }
                Key::H => { keys_down.insert(rustcraft::core::controls::KeyCode::H); }
                Key::I => { keys_down.insert(rustcraft::core::controls::KeyCode::I); }
                Key::J => { keys_down.insert(rustcraft::core::controls::KeyCode::J); }
                Key::K => { keys_down.insert(rustcraft::core::controls::KeyCode::K); }
                Key::L => { keys_down.insert(rustcraft::core::controls::KeyCode::L); }
                Key::M => { keys_down.insert(rustcraft::core::controls::KeyCode::M); }
                Key::N => { keys_down.insert(rustcraft::core::controls::KeyCode::N); }
                Key::O => { keys_down.insert(rustcraft::core::controls::KeyCode::O); }
                Key::P => { keys_down.insert(rustcraft::core::controls::KeyCode::P); }
                Key::Q => { keys_down.insert(rustcraft::core::controls::KeyCode::Q); }
                Key::R => { keys_down.insert(rustcraft::core::controls::KeyCode::R); }
                Key::S => { keys_down.insert(rustcraft::core::controls::KeyCode::S); }
                Key::T => { keys_down.insert(rustcraft::core::controls::KeyCode::T); }
                Key::U => { keys_down.insert(rustcraft::core::controls::KeyCode::U); }
                Key::V => { keys_down.insert(rustcraft::core::controls::KeyCode::V); }
                Key::W => { keys_down.insert(rustcraft::core::controls::KeyCode::W); }
                Key::X => { keys_down.insert(rustcraft::core::controls::KeyCode::X); }
                Key::Y => { keys_down.insert(rustcraft::core::controls::KeyCode::Y); }
                Key::Z => { keys_down.insert(rustcraft::core::controls::KeyCode::Z); }
                Key::Key0 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key0); }
                Key::Key1 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key1); }
                Key::Key2 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key2); }
                Key::Key3 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key3); }
                Key::Key4 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key4); }
                Key::Key5 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key5); }
                Key::Key6 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key6); }
                Key::Key7 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key7); }
                Key::Key8 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key8); }
                Key::Key9 => { keys_down.insert(rustcraft::core::controls::KeyCode::Key9); }
                Key::F1 => { keys_down.insert(rustcraft::core::controls::KeyCode::F1); }
                Key::F2 => { keys_down.insert(rustcraft::core::controls::KeyCode::F2); }
                Key::F3 => { keys_down.insert(rustcraft::core::controls::KeyCode::F3); }
                Key::F4 => { keys_down.insert(rustcraft::core::controls::KeyCode::F4); }
                Key::F5 => { keys_down.insert(rustcraft::core::controls::KeyCode::F5); }
                Key::F6 => { keys_down.insert(rustcraft::core::controls::KeyCode::F6); }
                Key::F7 => { keys_down.insert(rustcraft::core::controls::KeyCode::F7); }
                Key::F8 => { keys_down.insert(rustcraft::core::controls::KeyCode::F8); }
                Key::F9 => { keys_down.insert(rustcraft::core::controls::KeyCode::F9); }
                Key::F10 => { keys_down.insert(rustcraft::core::controls::KeyCode::F10); }
                Key::F11 => { keys_down.insert(rustcraft::core::controls::KeyCode::F11); }
                Key::F12 => { keys_down.insert(rustcraft::core::controls::KeyCode::F12); }
                Key::F13 => { keys_down.insert(rustcraft::core::controls::KeyCode::F13); }
                Key::F14 => { keys_down.insert(rustcraft::core::controls::KeyCode::F14); }
                Key::F15 => { keys_down.insert(rustcraft::core::controls::KeyCode::F15); }
                Key::Space => { keys_down.insert(rustcraft::core::controls::KeyCode::Space); }
                Key::LeftShift => {
                    keys_down.insert(rustcraft::core::controls::KeyCode::Shift);
                    keys_down.insert(rustcraft::core::controls::KeyCode::LeftShift);
                }
                Key::RightShift => {
                    keys_down.insert(rustcraft::core::controls::KeyCode::Shift);
                    keys_down.insert(rustcraft::core::controls::KeyCode::RightShift);
                }
                Key::LeftCtrl => {
                    keys_down.insert(rustcraft::core::controls::KeyCode::Ctrl);
                    keys_down.insert(rustcraft::core::controls::KeyCode::LeftCtrl);
                }
                Key::RightCtrl => {
                    keys_down.insert(rustcraft::core::controls::KeyCode::Ctrl);
                    keys_down.insert(rustcraft::core::controls::KeyCode::RightCtrl);
                }
                Key::LeftAlt => {
                    keys_down.insert(rustcraft::core::controls::KeyCode::Alt);
                    keys_down.insert(rustcraft::core::controls::KeyCode::LeftAlt);
                }
                Key::RightAlt => {
                    keys_down.insert(rustcraft::core::controls::KeyCode::Alt);
                    keys_down.insert(rustcraft::core::controls::KeyCode::RightAlt);
                }
                Key::Tab => { keys_down.insert(rustcraft::core::controls::KeyCode::Tab); }
                Key::Enter => { keys_down.insert(rustcraft::core::controls::KeyCode::Enter); }
                Key::Escape => { keys_down.insert(rustcraft::core::controls::KeyCode::Escape); }
                Key::Backspace => { keys_down.insert(rustcraft::core::controls::KeyCode::Backspace); }
                Key::Delete => { keys_down.insert(rustcraft::core::controls::KeyCode::Delete); }
                Key::Insert => { keys_down.insert(rustcraft::core::controls::KeyCode::Insert); }
                Key::Home => { keys_down.insert(rustcraft::core::controls::KeyCode::Home); }
                Key::End => { keys_down.insert(rustcraft::core::controls::KeyCode::End); }
                Key::PageUp => { keys_down.insert(rustcraft::core::controls::KeyCode::PageUp); }
                Key::PageDown => { keys_down.insert(rustcraft::core::controls::KeyCode::PageDown); }
                Key::Up => { keys_down.insert(rustcraft::core::controls::KeyCode::Up); }
                Key::Down => { keys_down.insert(rustcraft::core::controls::KeyCode::Down); }
                Key::Left => { keys_down.insert(rustcraft::core::controls::KeyCode::Left); }
                Key::Right => { keys_down.insert(rustcraft::core::controls::KeyCode::Right); }
                Key::Minus => { keys_down.insert(rustcraft::core::controls::KeyCode::Minus); }
                Key::Equal => { keys_down.insert(rustcraft::core::controls::KeyCode::Equal); }
                Key::LeftBracket => { keys_down.insert(rustcraft::core::controls::KeyCode::LeftBracket); }
                Key::RightBracket => { keys_down.insert(rustcraft::core::controls::KeyCode::RightBracket); }
                Key::Semicolon => { keys_down.insert(rustcraft::core::controls::KeyCode::Semicolon); }
                Key::Apostrophe => { keys_down.insert(rustcraft::core::controls::KeyCode::Apostrophe); }
                Key::Backquote => { keys_down.insert(rustcraft::core::controls::KeyCode::Grave); }
                Key::Backslash => { keys_down.insert(rustcraft::core::controls::KeyCode::Backslash); }
                Key::Slash => { keys_down.insert(rustcraft::core::controls::KeyCode::Slash); }
                Key::Comma => { keys_down.insert(rustcraft::core::controls::KeyCode::Comma); }
                Key::Period => { keys_down.insert(rustcraft::core::controls::KeyCode::Period); }
                Key::NumPad0 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad0); }
                Key::NumPad1 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad1); }
                Key::NumPad2 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad2); }
                Key::NumPad3 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad3); }
                Key::NumPad4 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad4); }
                Key::NumPad5 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad5); }
                Key::NumPad6 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad6); }
                Key::NumPad7 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad7); }
                Key::NumPad8 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad8); }
                Key::NumPad9 => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPad9); }
                Key::NumPadDot => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPadDot); }
                Key::NumPadSlash => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPadSlash); }
                Key::NumPadAsterisk => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPadAsterisk); }
                Key::NumPadMinus => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPadMinus); }
                Key::NumPadPlus => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPadPlus); }
                Key::NumPadEnter => { keys_down.insert(rustcraft::core::controls::KeyCode::NumPadEnter); }
                _ => {}
            }
        }
        mouse_capture.poll_extra_keys(&mut keys_down);
        let (extra_mouse4, extra_mouse5, extra_mouse_arr) = mouse_capture.poll_extra_buttons();

        let input = InputState {
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
        };

        let out = app.update_and_render(dt, &input);

        if let Some(r) = out.rumble_request {
            gamepad_mgr.rumble(r.strong, r.weak, r.duration_ms);
        }
        if let Some(visible) = out.cursor_visible {
            if !mouse_capture.is_captured() {
                window.set_cursor_visibility(visible);
            }
        }

        if out.request_capture_mouse {
            mouse_capture.set_captured(true);
            mouse_capture.focus();
            window.set_cursor_visibility(false);
            last_mouse = None;
        } else if out.request_release_mouse && mouse_capture.is_captured() {
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

    rustcraft::log_info!("MAIN", "RustCraft window closed, exiting cleanly");
}
