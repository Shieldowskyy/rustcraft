//! Native desktop window host loop (`minifb`) shared across Linux, Windows, macOS, and FreeBSD.
//! Parses CLI modes (`--relay-server`, `--server`, `--crash-test`), initializes crash reporting, and drives `RustCraftApp`.

use std::time::Instant;

use minifb::{CursorStyle, Window, WindowOptions};

use crate::app::{GameState, RustCraftApp, HEIGHT, WIDTH};
use crate::cli::{parse_log_enabled, parse_profiler_enabled, parse_renderer_choice, parse_seed};
use crate::core::{init_crash_reporter, init_logger, set_crash_context, CrashReporterConfig};

use super::gamepad::GamepadManager;
use super::input_map::build_input_state;
use super::mouse_capture::{self, MouseCapture};

pub fn run_desktop_app() {
    let log_enabled = parse_log_enabled();
    init_logger(log_enabled);
    let max_ram = crate::cli::parse_max_ram_mb();
    crate::core::settings::set_max_ram_mb(max_ram);
    if crate::cli::parse_hard_ram_limit() {
        match max_ram {
            None => {
                eprintln!("--hard-ram-limit requires --max-ram SIZE");
                std::process::exit(2);
            }
            Some(mb) => match crate::core::settings::apply_hard_ram_limit(mb) {
                Ok(()) => {
                    crate::log_info!("APP", "Hard RAM limit enforced: {} MB", mb);
                }
                Err(e) => eprintln!("Warning: could not enforce hard RAM limit: {e}"),
            },
        }
    }

    let sentry_dsn = crate::cli::parse_sentry_dsn();
    let sentry_enabled = !crate::cli::parse_sentry_disabled();
    let _crash_guard = init_crash_reporter(CrashReporterConfig {
        dsn: sentry_dsn,
        sentry_enabled,
        ..Default::default()
    });

    if crate::cli::parse_crash_test() {
        crate::log_warn!("TEST", "Manual crash reporter test triggered via --crash-test!");
        panic!("RustCraft intentional crash test via --crash-test CLI flag");
    }

    if crate::cli::parse_relay_server_mode() {
        let port = crate::cli::parse_relay_server_port();
        let bind_addr = format!("0.0.0.0:{port}");
        if let Err(error) = crate::net::run_relay_server(&bind_addr) {
            eprintln!("Failed to start RustCraft relay: {error}");
            std::process::exit(1);
        }
        return;
    }

    if crate::cli::parse_server_mode() {
        let port = crate::cli::parse_server_port();
        let seed = parse_seed();
        set_crash_context("mode", "dedicated_server");
        set_crash_context("port", &port.to_string());
        set_crash_context("seed", &seed.to_string());
        crate::net::run_dedicated_server(port, seed);
        return;
    }

    let renderer_choice = parse_renderer_choice();
    let seed = parse_seed();
    let profiler_enabled = parse_profiler_enabled();

    set_crash_context("mode", "client");
    set_crash_context("renderer", renderer_choice);
    set_crash_context("seed", &seed.to_string());
    set_crash_context("profiler", &profiler_enabled.to_string());

    crate::log_info!(
        "MAIN",
        "Starting RustCraft (renderer: {}, seed: {}, profiler: {})",
        renderer_choice,
        seed,
        profiler_enabled
    );

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if std::env::var_os("DISPLAY").is_some() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    crate::audio::preload_intro_music();

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

    crate::log_info!("WINDOW", "Desktop window created ({}x{})", WIDTH, HEIGHT);

    let mut app = RustCraftApp::new(WIDTH, HEIGHT, renderer_choice, seed);
    app.profiler_enabled = profiler_enabled;

    if let Some(connect_addr) = crate::cli::parse_connect_addr() {
        crate::log_info!(
            "MAIN",
            "Auto-connecting to multiplayer server: {}",
            crate::net::redact_invite_address(&connect_addr)
        );
        app.connect_to_server(&connect_addr);
    }
    let mut gamepad_mgr = GamepadManager::new();

    mouse_capture::maximize_window(&window);
    let mut current_target_fps = app.settings.fps_limit;
    window.set_target_fps(if current_target_fps == 0 {
        0
    } else {
        current_target_fps as usize
    });
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

    print_controls_banner();

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
                crate::log_info!("STATE", "Game paused after window lost focus");
            }
        }
        was_window_active = window_active;

        let gp_events = gamepad_mgr.update(dt);
        let gp = gamepad_mgr.state;

        let cur_size = window.get_size();
        if cur_size != win_size {
            crate::log_debug!(
                "WINDOW",
                "Window resized from {}x{} to {}x{}",
                win_size.0,
                win_size.1,
                cur_size.0,
                cur_size.1
            );
            win_size = cur_size;
            mouse_capture.resize(win_size.0 as i32, win_size.1 as i32);
        }

        let input = build_input_state(
            &window,
            win_size,
            &mouse_capture,
            &mut last_mouse,
            gp,
            gp_events,
        );
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
            window.set_target_fps(if current_target_fps == 0 {
                0
            } else {
                current_target_fps as usize
            });
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

    crate::log_info!("MAIN", "RustCraft window closed, exiting cleanly");
}

fn print_controls_banner() {
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
}
