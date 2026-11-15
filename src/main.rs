#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use minifb::{CursorStyle, Key, MouseButton, MouseMode, Window, WindowOptions};
#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use std::time::Instant;

#[cfg(not(any(target_os = "vita", target_os = "android", target_os = "xbox360", target_os = "xenon", target_arch = "powerpc", feature = "xbox360")))]
use rustcraft::app::{GameState, InputState, RustCraftApp, HEIGHT, WIDTH};
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
    let max_ram = rustcraft::cli::parse_max_ram_mb();
    rustcraft::core::settings::set_max_ram_mb(max_ram);
    if rustcraft::cli::parse_hard_ram_limit() {
        match max_ram {
            None => {
                eprintln!("--hard-ram-limit requires --max-ram SIZE");
                std::process::exit(2);
            }
            Some(mb) => match rustcraft::core::settings::apply_hard_ram_limit(mb) {
                Ok(()) => { rustcraft::log_info!("APP", "Hard RAM limit enforced: {} MB", mb); }
                Err(e) => eprintln!("Warning: could not enforce hard RAM limit: {e}"),
            },
        }
    }

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

    if rustcraft::cli::parse_relay_server_mode() {
        let port = rustcraft::cli::parse_relay_server_port();
        let bind_addr = format!("0.0.0.0:{port}");
        if let Err(error) = rustcraft::net::run_relay_server(&bind_addr) {
            eprintln!("Failed to start RustCraft relay: {error}");
            std::process::exit(1);
        }
        return;
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
        rustcraft::log_info!(
            "MAIN",
            "Auto-connecting to multiplayer server: {}",
            rustcraft::net::redact_invite_address(&connect_addr)
        );
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
    let mut was_window_active = true;

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

        let window_active = window.is_active();
        if was_window_active && !window_active && mouse_capture.is_captured() {
            mouse_capture.set_captured(false);
            window.set_cursor_visibility(true);
            last_mouse = None;
            if matches!(app.state, GameState::Playing) {
                app.state = GameState::Paused;
                rustcraft::log_info!("STATE", "Game paused after window lost focus");
            }
        }
        was_window_active = window_active;

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
            }
        }

        let mut keys_down = std::collections::HashSet::new();
        for key in window.get_keys() {
            use rustcraft::core::controls::KeyCode;
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
