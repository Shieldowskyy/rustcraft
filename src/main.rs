// RustCraft - a tiny Minecraft-like voxel game with selectable software/Vulkan backends.
// All rasterization (triangles, depth-buffer, lighting) happens on the CPU
// into a plain pixel buffer, which is then blitted to a window via `minifb`.
// `minifb` is used ONLY for creating an OS window and reading keyboard/mouse
// input - it performs no rendering itself.

pub mod cli;
pub mod core;
pub mod gameplay;
pub mod graphics;
pub mod platform;
pub mod ui;
pub mod world;

use minifb::{CursorStyle, Key, MouseButton, MouseMode, Window, WindowOptions};
use std::fmt::Write;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use crate::cli::{generate_random_seed, parse_renderer_choice, parse_seed};
use crate::core::math::{IVec3, Vec3};
use crate::core::settings::GameSettings;
use crate::gameplay::inventory::{Inventory, ItemStack};
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::MobManager;
use crate::gameplay::particles::ParticleSystem;
use crate::gameplay::physics::{
    block_intersects_player, move_player_with_collision, CROUCH_EYE_OFFSET, CROUCH_HEIGHT,
    FLY_SPEED, GRAVITY, JUMP_SPEED, SNEAK_SPEED, SPRINT_JUMP_BOOST, SPRINT_SPEED,
    STANDING_EYE_OFFSET, STANDING_HEIGHT, SWIM_SPEED, TERMINAL_VELOCITY, WALK_SPEED,
};
use crate::gameplay::player::{GameMode, Player};
use crate::gameplay::recipe::RecipeBook;
use crate::graphics::camera::Camera;
use crate::graphics::software::Renderer;
use crate::platform::gamepad::GamepadManager;
use crate::platform::mouse_capture::{self, MouseCapture};
use crate::ui::{InventoryLayout, InventorySlotTarget, VISIBLE_SAVE_ROWS};
use crate::world::block::{BlockId, Face};
use crate::world::storage::{delete_save, get_save_metadata, save_current_world, save_path, SaveMetadata};
use crate::world::World;

const WIDTH: usize = 960;
const HEIGHT: usize = 540;
const TARGET_FPS: usize = 300;
const MAX_SAVE_SLOTS: u8 = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
enum PreviousMenu {
    MainMenu,
    Paused,
}

enum GameState {
    MainMenu,
    SaveSelect,
    Settings(PreviousMenu),
    Loading(mpsc::Receiver<WorldLoadResult>),
    Playing,
    Paused,
    Inventory,
    DeathScreen,
}

enum WorldLoadResult {
    Success(World, Option<Vec3>),
    Error(String),
}

fn main() {
    let renderer_choice = parse_renderer_choice();
    let mut seed = parse_seed();
    let mut settings = GameSettings::load();
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
    .expect("Nie udało się utworzyć okna");

    mouse_capture::maximize_window(&window);
    window.set_target_fps(TARGET_FPS);
    window.set_cursor_visibility(false);
    let _ = window.set_cursor_style(CursorStyle::Crosshair);

    let mut win_size = window.get_size();

    let mut renderer = match renderer_choice {
        "software" => Renderer::new(WIDTH, HEIGHT),
        "vulkan" => Renderer::new_vulkan(WIDTH, HEIGHT).unwrap_or_else(|error| {
            eprintln!("Nie można uruchomić renderera Vulkan: {error}");
            std::process::exit(1);
        }),
        _ => unreachable!(),
    };
    let mut world = World::new_with_seed(seed);
    let mut camera = Camera::new(Vec3::new(8.0, 90.0, 8.0));

    let mut mouse_capture = MouseCapture::new(&window, win_size.0 as i32, win_size.1 as i32);
    mouse_capture.set_captured(false);

    let mut last_time = Instant::now();
    let mut last_mouse: Option<(f32, f32)> = None;

    let mut fps_timer = Instant::now();
    let mut frame_count = 0u32;
    let mut fps_display = 0u32;
    let mut last_title = String::new();

    let mut left_was_down = false;
    let mut right_was_down = false;
    let mut escape_was_down = false;
    let mut fly_key_was_down = false;
    let mut e_was_down = false;
    let mut q_was_down = false;
    let mut m_was_down = false;
    let mut enter_was_down = false;
    let mut up_arrow_was_down = false;
    let mut down_arrow_was_down = false;
    let mut n_was_down = false;
    let mut del_was_down = false;
    let mut left_arrow_was_down = false;
    let mut right_arrow_was_down = false;
    let mut save_was_down = false;
    let mut save_slot: u8 = 1;

    let mut vertical_velocity: f32 = 0.0;
    let mut horiz_velocity = Vec3::zero();
    let mut fly_velocity = Vec3::zero();
    let mut on_ground = false;
    let mut fly_mode = false;
    let mut current_eye_offset = STANDING_EYE_OFFSET;
    let mut current_player_height = STANDING_HEIGHT;

    // Block breaking state (like Minecraft - hold LMB to break)
    let mut break_progress: f32 = 0.0; // 0.0 to 1.0
    let mut break_target: Option<IVec3> = None;

    let mut state = GameState::MainMenu;
    let mut save_metadata_cache: [SaveMetadata; MAX_SAVE_SLOTS as usize] = std::array::from_fn(|index| {
        get_save_metadata(index as u8 + 1)
    });

    let mut save_select_slot: u8 = 1;
    let mut save_select_scroll: usize = 0;
    let mut settings_hovered_opt: Option<usize> = Some(0);

    let mut particles = ParticleSystem::new();
    let mut mob_manager = MobManager::new();
    let mut player = Player::new();
    player.gamemode = GameMode::Survival; // Start in survival as requested
    let mut items: Vec<ItemEntity> = Vec::new();
    let recipe_book = RecipeBook::new();
    let mut crafting_table_open = false;
    
    let mut g_was_down = false;
    let mut total_time = 0.0f32;
    let mut material_timer = 0.0f32;
    let mut hand_animation = 0.0f32;
    let mut walk_phase = 0.0f32;

    // OPTYMALIZACJA 1: Deklaracja bufora Strings dla interfejsu raz przed pętlą[cite: 2]
    let mut debug_lines = vec![
        String::with_capacity(32),
        String::with_capacity(64),
        String::with_capacity(32),
        String::with_capacity(32),
        String::with_capacity(32),
    ];

    // OPTYMALIZACJA 2: Pamięć podręczna dla macierzy projekcji[cite: 2]
    let mut last_fov = camera.fov_y;
    let mut last_aspect = WIDTH as f32 / HEIGHT as f32;
    let mut proj_matrix = camera.projection_matrix(last_aspect);

    // OPTYMALIZACJA 3: Zbieranie geometrii interfejsu (UI rects) raz, ominięcie alokacji podczas każdej klatki[cite: 2]
    let main_menu_rects = renderer.main_menu_button_rects();
    let save_card_rects = renderer.save_card_rects();
    let save_btn_rects = renderer.save_bottom_button_rects();
    let settings_rects = renderer.settings_option_rects();
    let pause_menu_rects = renderer.pause_menu_button_rects();
    let death_screen_rects = renderer.death_screen_button_rects();

    let mut gamepad_mgr = GamepadManager::new();
    let mut menu_hovered_btn: Option<usize> = Some(0);
    let mut pause_hovered_btn: Option<usize> = Some(0);
    let mut death_hovered_btn: Option<usize> = Some(0);

    let mut damage_tilt_timer = 0.0f32;
    let mut damage_tilt_dir = 1.0f32;

    println!("Sterowanie (Klawiatura / Mysz):");
    println!("  Kliknij w okno - zablokuj/przechwyc mysz");
    println!("  Esc            - Menu pauzy / zwolnij mysz");
    println!("  WASD           - ruch");
    println!("  Spacja         - skok (lot: w gore)");
    println!("  Shift (lewy)   - kucanie (lot: w dol)");
    println!("  Ctrl (lewy)    - bieg");
    println!("  F              - przelacz tryb lotu");
    println!("  Mysz           - rozgladanie sie");
    println!("  LPM            - niszcz blok / bij moby / kliknij UI");
    println!("  PPM            - stawiaj blok");
    println!("  Scroll / 1-9   - wybor typu bloku");
    println!("  Q              - wyjscie w menu / wyrzuc przedmiot");
    println!("Sterowanie (Kontroler / Gamepad):");
    println!("  Lewy analog   - ruch");
    println!("  Prawy analog  - rozgladanie sie");
    println!("  RT (R2)       - niszcz blok / atak");
    println!("  LT (L2)       - stawiaj blok");
    println!("  A (Krzyzyk)   - skok / potwierdz w UI");
    println!("  B (Kolko)     - kucanie / wstecz w UI");
    println!("  Y (Trojkat)   - ekwipunek");
    println!("  X (Kwadrat)   - wyrzuc przedmiot");
    println!("  LB / RB       - poprzedni / nastepny slot");
    println!("  L3 (Wcisnij L)- bieg");
    println!("  R3 (Wcisnij R)- tryb lotu");
    println!("  Start         - pauza");
    println!("  Select / Back - zmiana trybu gry (Survival/Creative)");
    println!("  D-Pad         - nawigacja w menu i ekwipunku");

    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32().min(0.1);
        last_time = now;
        total_time += dt;
        hand_animation = (hand_animation - dt * 3.5).max(0.0);

        let gp_events = gamepad_mgr.update(dt);
        let gp = gamepad_mgr.state;

        let cur_size = window.get_size();
        if cur_size != win_size {
            win_size = cur_size;
            mouse_capture.resize(win_size.0 as i32, win_size.1 as i32);
        }

        let escape_down = window.is_key_down(Key::Escape);
        let f_down = window.is_key_down(Key::F);
        let e_down = window.is_key_down(Key::E);
        let q_down = window.is_key_down(Key::Q);
        let f5_down = window.is_key_down(Key::F5);
        let m_down = window.is_key_down(Key::M);
        let (win_w, win_h) = window.get_size();

        let raw_mouse_pos = window.get_mouse_pos(MouseMode::Pass);
        let mouse_pos = raw_mouse_pos.map(|(mx, my)| {
            let fx = if win_w > 0 { mx * (WIDTH as f32) / (win_w as f32) } else { mx };
            let fy = if win_h > 0 { my * (HEIGHT as f32) / (win_h as f32) } else { my };
            (fx, fy)
        });
        let left_down = window.get_mouse_down(MouseButton::Left);
        let right_down = window.get_mouse_down(MouseButton::Right);

        let enter_down = window.is_key_down(Key::Enter);
        let n_down = window.is_key_down(Key::N);
        let del_down = window.is_key_down(Key::Delete) || window.is_key_down(Key::Backspace);
        let left_arrow_down = window.is_key_down(Key::Left);
        let right_arrow_down = window.is_key_down(Key::Right);
        let up_arrow_down = window.is_key_down(Key::Up);
        let down_arrow_down = window.is_key_down(Key::Down);

        let scroll_wheel = window.get_scroll_wheel();

        match state {
            GameState::MainMenu => {
                let mut hovered_idx = menu_hovered_btn;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in main_menu_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            menu_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                // Nawigacja D-Pad / Analog na padzie w Menu Głównym
                if gp_events.dpad_up_just_pressed || (up_arrow_down && !up_arrow_was_down) {
                    let next = match menu_hovered_btn {
                        Some(0) | None => 2,
                        Some(i) => i - 1,
                    };
                    menu_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if gp_events.dpad_down_just_pressed || (down_arrow_down && !down_arrow_was_down) {
                    let next = match menu_hovered_btn {
                        Some(2) | None => 0,
                        Some(i) => i + 1,
                    };
                    menu_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_button = if left_down && !left_was_down || gp_events.south_just_pressed {
                    hovered_idx
                } else {
                    None
                };

                if (enter_down && !enter_was_down) || click_button == Some(0) {
                    state = GameState::SaveSelect;
                    save_select_slot = save_slot;
                    save_select_scroll = ((save_slot as usize - 1).saturating_sub(1))
                        .min(MAX_SAVE_SLOTS as usize - VISIBLE_SAVE_ROWS);
                } else if click_button == Some(1) {
                    state = GameState::Settings(PreviousMenu::MainMenu);
                    settings_hovered_opt = Some(0);
                } else if (q_down && !q_was_down) || click_button == Some(2) || gp_events.east_just_pressed {
                    break;
                }

                renderer.draw_main_menu(hovered_idx);
            }
            GameState::SaveSelect => {
                // Przewijanie listy kart kółkiem myszy
                if let Some((_sx, sy)) = scroll_wheel {
                    if sy > 0.0 {
                        save_select_scroll = save_select_scroll.saturating_sub(1);
                    } else if sy < 0.0 {
                        save_select_scroll = (save_select_scroll + 1).min(MAX_SAVE_SLOTS as usize - VISIBLE_SAVE_ROWS);
                    }
                }

                // Nawigacja po kartach zapisu (D-Pad / Strzałki)
                if gp_events.dpad_up_just_pressed || (up_arrow_down && !up_arrow_was_down) {
                    save_select_slot = if save_select_slot <= 1 { MAX_SAVE_SLOTS } else { save_select_slot - 1 };
                    let slot_idx = save_select_slot as usize - 1;
                    if slot_idx < save_select_scroll {
                        save_select_scroll = slot_idx;
                    } else if slot_idx >= save_select_scroll + VISIBLE_SAVE_ROWS {
                        save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
                    }
                }
                if gp_events.dpad_down_just_pressed || (down_arrow_down && !down_arrow_was_down) {
                    save_select_slot = if save_select_slot >= MAX_SAVE_SLOTS { 1 } else { save_select_slot + 1 };
                    let slot_idx = save_select_slot as usize - 1;
                    if slot_idx >= save_select_scroll + VISIBLE_SAVE_ROWS {
                        save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
                    } else if slot_idx < save_select_scroll {
                        save_select_scroll = slot_idx;
                    }
                }

                let mut save_select_hovered_card = None;
                let mut save_select_hovered_btn = None;

                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (row, &(bx, by, bw, bh)) in save_card_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            save_select_hovered_card = Some(row);
                            if left_down && !left_was_down {
                                save_select_slot = (save_select_scroll + row + 1) as u8;
                            }
                            break;
                        }
                    }
                    for (i, &(bx, by, bw, bh)) in save_btn_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            save_select_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                let play_action = (enter_down && !enter_was_down)
                    || (left_down && !left_was_down && save_select_hovered_btn == Some(0))
                    || gp_events.south_just_pressed;
                let new_action = (n_down && !n_was_down)
                    || (left_down && !left_was_down && save_select_hovered_btn == Some(1))
                    || gp_events.north_just_pressed;
                let delete_action = (del_down && !del_was_down)
                    || (left_down && !left_was_down && save_select_hovered_btn == Some(2))
                    || gp_events.west_just_pressed;
                let back_action = (escape_down && !escape_was_down)
                    || (left_down && !left_was_down && save_select_hovered_btn == Some(3))
                    || gp_events.east_just_pressed;

                if play_action {
                    save_slot = save_select_slot;
                    let meta = &save_metadata_cache[save_slot as usize - 1];
                    if meta.exists {
                        let path = save_path(save_slot);
                        let (tx, rx) = mpsc::channel();
                        thread::spawn(move || {
                            match World::load(&path) {
                                Ok(loaded_world) => {
                                    let pos = loaded_world.player_position;
                                    let _ = tx.send(WorldLoadResult::Success(loaded_world, Some(pos)));
                                }
                                Err(error) => {
                                    let _ = tx.send(WorldLoadResult::Error(error.to_string()));
                                }
                            }
                        });
                        state = GameState::Loading(rx);
                    } else {
                        seed = generate_random_seed();
                        let (tx, rx) = mpsc::channel();
                        let thread_seed = seed;
                        thread::spawn(move || {
                            let new_world = World::new_with_seed(thread_seed);
                            let _ = tx.send(WorldLoadResult::Success(new_world, None));
                        });
                        state = GameState::Loading(rx);
                    }
                } else if new_action {
                    save_slot = save_select_slot;
                    seed = generate_random_seed();
                    let (tx, rx) = mpsc::channel();
                    let thread_seed = seed;
                    thread::spawn(move || {
                        let new_world = World::new_with_seed(thread_seed);
                        let _ = tx.send(WorldLoadResult::Success(new_world, None));
                    });
                    state = GameState::Loading(rx);
                } else if delete_action {
                    let _ = delete_save(save_select_slot);
                    save_metadata_cache[save_select_slot as usize - 1] = get_save_metadata(save_select_slot);
                } else if back_action {
                    state = GameState::MainMenu;
                }

                renderer.draw_save_select_menu(
                    save_select_slot,
                    save_select_scroll,
                    &save_metadata_cache,
                    save_select_hovered_card,
                    save_select_hovered_btn,
                );
            }
            GameState::Settings(prev) => {
                let mut hovered_opt = settings_hovered_opt;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in settings_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_opt = Some(i);
                            settings_hovered_opt = Some(i);
                            break;
                        }
                    }
                }

                if gp_events.dpad_up_just_pressed || (up_arrow_down && !up_arrow_was_down) {
                    let next = match settings_hovered_opt {
                        Some(0) | None => 5,
                        Some(i) => i - 1,
                    };
                    settings_hovered_opt = Some(next);
                    hovered_opt = Some(next);
                }
                if gp_events.dpad_down_just_pressed || (down_arrow_down && !down_arrow_was_down) {
                    let next = match settings_hovered_opt {
                        Some(5) | None => 0,
                        Some(i) => i + 1,
                    };
                    settings_hovered_opt = Some(next);
                    hovered_opt = Some(next);
                }

                let left_toggle = gp_events.dpad_left_just_pressed || gp_events.lb_just_pressed || (left_arrow_down && !left_arrow_was_down);
                let right_toggle = gp_events.dpad_right_just_pressed || gp_events.rb_just_pressed || (right_arrow_down && !right_arrow_was_down);
                let click_opt = if left_down && !left_was_down || gp_events.south_just_pressed || (enter_down && !enter_was_down) {
                    hovered_opt
                } else {
                    None
                };

                if left_toggle {
                    match settings_hovered_opt {
                        Some(0) => settings.cycle_render_distance(false),
                        Some(1) => settings.fog = !settings.fog,
                        Some(2) => settings.clouds = !settings.clouds,
                        Some(3) => settings.cycle_fov(false),
                        Some(4) => settings.underwater_fx = !settings.underwater_fx,
                        _ => {}
                    }
                }
                if right_toggle {
                    match settings_hovered_opt {
                        Some(0) => settings.cycle_render_distance(true),
                        Some(1) => settings.fog = !settings.fog,
                        Some(2) => settings.clouds = !settings.clouds,
                        Some(3) => settings.cycle_fov(true),
                        Some(4) => settings.underwater_fx = !settings.underwater_fx,
                        _ => {}
                    }
                }
                if let Some(opt) = click_opt {
                    match opt {
                        0 => settings.cycle_render_distance(true),
                        1 => settings.fog = !settings.fog,
                        2 => settings.clouds = !settings.clouds,
                        3 => settings.cycle_fov(true),
                        4 => settings.underwater_fx = !settings.underwater_fx,
                        5 => {
                            let _ = settings.save();
                            state = match prev {
                                PreviousMenu::MainMenu => GameState::MainMenu,
                                PreviousMenu::Paused => GameState::Paused,
                            };
                        }
                        _ => {}
                    }
                }

                if (escape_down && !escape_was_down) || gp_events.east_just_pressed {
                    let _ = settings.save();
                    state = match prev {
                        PreviousMenu::MainMenu => GameState::MainMenu,
                        PreviousMenu::Paused => GameState::Paused,
                    };
                }

                renderer.fog_enabled = settings.fog;
                if prev == PreviousMenu::Paused {
                    renderer.render_frame(&mut world, &camera);
                }
                renderer.draw_settings_menu(&settings, hovered_opt);
            }
            GameState::Loading(ref rx) => {
                renderer.draw_main_menu(None);

                if let Ok(result) = rx.try_recv() {
                    match result {
                        WorldLoadResult::Success(loaded_world, loaded_pos) => {
                            world = loaded_world;

                            let start_pos = loaded_pos.unwrap_or_else(|| Vec3::new(8.0, 90.0, 8.0));
                            camera = Camera::new(start_pos);
                            mob_manager = MobManager::new();
                            state = GameState::Playing;
                        }
                        WorldLoadResult::Error(e) => {
                            eprintln!("Błąd ładowania świata: {e}");
                            state = GameState::MainMenu;
                        }
                    }
                }
            }
            GameState::Playing => {
                if f5_down && !save_was_down {
                    let path = save_path(save_slot);
                    match save_current_world(&mut world, &camera, save_slot) {
                        Ok(()) => println!("Zapisano świat do {path}"),
                        Err(error) => eprintln!("{error}"),
                    }
                }
                if (escape_down && !escape_was_down) || gp_events.start_just_pressed {
                    state = GameState::Paused;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                if (e_down && !e_was_down) || gp_events.north_just_pressed {
                    crafting_table_open = false;
                    state = GameState::Inventory;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                let any_click = left_down || right_down;
                if any_click && !mouse_capture.is_captured() {
                    mouse_capture.set_captured(true);
                    window.set_cursor_visibility(false);
                    last_mouse = None;
                    left_was_down = left_down;
                    right_was_down = right_down;
                }

                if mouse_capture.is_captured() {
                    if mouse_capture.has_native_capture() {
                        let (dx, dy) = mouse_capture.poll_delta();
                        camera.rotate(dx * 0.0025, -dy * 0.0025);
                    } else if let Some((mx, my)) = raw_mouse_pos {
                        if let Some((lx, ly)) = last_mouse {
                            let dx = mx - lx;
                            let dy = my - ly;
                            camera.rotate(dx * 0.0025, -dy * 0.0025);
                        }
                        last_mouse = Some((mx, my));
                    }
                }

                // Rozglądanie się prawym analogiem kontrolera
                let gp_look_speed_x = 3.2;
                let gp_look_speed_y = 2.8;
                if gp.right_stick_x.abs() > 0.0 || gp.right_stick_y.abs() > 0.0 {
                    camera.rotate(gp.right_stick_x * gp_look_speed_x * dt, gp.right_stick_y * gp_look_speed_y * dt);
                }

                let forward = camera.forward_flat();
                let right = camera.right_flat();
                let mut wish_dir = Vec3::zero();

                if mouse_capture.is_captured() {
                    if window.is_key_down(Key::W) {
                        wish_dir = wish_dir + forward;
                    }
                    if window.is_key_down(Key::S) {
                        wish_dir = wish_dir - forward;
                    }
                    if window.is_key_down(Key::A) {
                        wish_dir = wish_dir - right;
                    }
                    if window.is_key_down(Key::D) {
                        wish_dir = wish_dir + right;
                    }
                }
                if gp.left_stick_y.abs() > 0.0 || gp.left_stick_x.abs() > 0.0 {
                    wish_dir = wish_dir + forward * gp.left_stick_y + right * gp.left_stick_x;
                }
                if wish_dir.length_sq() > 0.0001 {
                    wish_dir = wish_dir.normalized();
                }

                let f_down = window.is_key_down(Key::F);
                let g_down = window.is_key_down(Key::G);
                if (f_down && !fly_key_was_down) || gp_events.r3_just_pressed {
                    fly_mode = !fly_mode;
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                if (g_down && !g_was_down) || gp_events.select_just_pressed {
                    player.gamemode = if player.gamemode == GameMode::Survival {
                        GameMode::Creative
                    } else {
                        GameMode::Survival
                    };
                }
                g_was_down = g_down;

                let is_crouching = !fly_mode
                && ((mouse_capture.is_captured() && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::C))) || gp.btn_east);

                let target_eye = if is_crouching { CROUCH_EYE_OFFSET } else { STANDING_EYE_OFFSET };
                let target_h = if is_crouching { CROUCH_HEIGHT } else { STANDING_HEIGHT };
                let t_eye = 1.0 - (-14.0 * dt).exp();
                let old_eye = current_eye_offset;
                current_eye_offset += (target_eye - current_eye_offset) * t_eye;
                current_player_height += (target_h - current_player_height) * t_eye;

                if !fly_mode {
                    camera.position.y += current_eye_offset - old_eye;
                }

                let is_sprinting = !is_crouching
                && ((mouse_capture.is_captured() && window.is_key_down(Key::LeftCtrl)) || gp.btn_l3)
                && wish_dir.length_sq() > 0.01;

                let target_fov_deg = if is_sprinting {
                    settings.fov_deg + 15.0
                } else if is_crouching {
                    settings.fov_deg - 5.0
                } else {
                    settings.fov_deg
                };
                let target_fov = target_fov_deg.to_radians();
                let t_fov = 1.0 - (-8.0 * dt).exp();
                camera.fov_y += (target_fov - camera.fov_y) * t_fov;

                let move_speed = if is_crouching {
                    SNEAK_SPEED
                } else if is_sprinting {
                    SPRINT_SPEED
                } else {
                    WALK_SPEED
                };

                if fly_mode {
                    let mut move_dir = wish_dir;
                    if (mouse_capture.is_captured() && window.is_key_down(Key::Space)) || gp.btn_south {
                        move_dir.y += 1.0;
                    }
                    if (mouse_capture.is_captured() && window.is_key_down(Key::LeftShift)) || gp.btn_east {
                        move_dir.y -= 1.0;
                    }
                    if move_dir.length_sq() > 0.0001 {
                        move_dir = move_dir.normalized();
                    }

                    let fly_fast = (mouse_capture.is_captured() && window.is_key_down(Key::LeftCtrl)) || gp.btn_l3;
                    let fly_mult = if fly_fast { 2.0 } else { 1.0 };
                    let target_fly = move_dir * (FLY_SPEED * fly_mult);
                    let fly_damping = 14.0;
                    let t_fly = 1.0 - (-fly_damping * dt).exp();
                    fly_velocity = fly_velocity.lerp(target_fly, t_fly);

                    camera.position = camera.position + fly_velocity * dt;
                    vertical_velocity = 0.0;
                    horiz_velocity = Vec3::zero();
                } else {
                    let feet_pos = Vec3::new(
                        camera.position.x,
                        camera.position.y - current_eye_offset - 0.08,
                        camera.position.z,
                    );
                    let feet_block = world.get_block(feet_pos.floor_to_ivec());
                    let eye_block = world.get_block(camera.position.floor_to_ivec());
                    let in_water = feet_block == BlockId::Water || eye_block == BlockId::Water;

                    let target_horiz = wish_dir * (if in_water { SWIM_SPEED } else { move_speed });
                    let on_ice = feet_block == BlockId::Ice;
                    let damping = if in_water {
                        8.0
                    } else if on_ground {
                        if on_ice { 2.0 } else if is_crouching { 18.0 } else { 15.0 }
                    } else {
                        2.0 // In-air momentum retention (Minecraft-like sprint-jumping)
                    };
                    let t_horiz = 1.0 - (-damping * dt).exp();
                    horiz_velocity = horiz_velocity.lerp(target_horiz, t_horiz);

                    if in_water {
                        vertical_velocity -= 4.0 * dt;
                        if vertical_velocity < -2.5 {
                            vertical_velocity = -2.5;
                        }
                        if (window.is_key_down(Key::Space) && mouse_capture.is_captured()) || gp.btn_south {
                            vertical_velocity = 3.2;
                        }
                    } else {
                        vertical_velocity -= GRAVITY * dt;
                        if vertical_velocity < -TERMINAL_VELOCITY {
                            vertical_velocity = -TERMINAL_VELOCITY;
                        }

                        if ((window.is_key_down(Key::Space) && mouse_capture.is_captured()) || gp.btn_south) && on_ground {
                            vertical_velocity = JUMP_SPEED;
                            if is_sprinting && wish_dir.length_sq() > 0.01 {
                                horiz_velocity = horiz_velocity + wish_dir * SPRINT_JUMP_BOOST;
                            }
                        }
                    }

                    let old_y = camera.position.y;
                    let motion = Vec3::new(horiz_velocity.x, vertical_velocity, horiz_velocity.z) * dt;
                    on_ground = move_player_with_collision(
                        &world,
                        &mut camera.position,
                        motion,
                        &mut vertical_velocity,
                        current_eye_offset,
                        current_player_height,
                        is_crouching,
                    );
                    if vertical_velocity < 0.0 && !fly_mode && !in_water {
                        player.fall_distance += old_y - camera.position.y;
                    } else if vertical_velocity > 0.0 || fly_mode || in_water {
                        player.fall_distance = 0.0;
                    }

                    if on_ground {
                        if !player.has_landed {
                            player.has_landed = true;
                            player.fall_distance = 0.0;
                        } else if player.fall_distance > 3.0 {
                            let damage = player.fall_distance - 3.0;
                            if player.take_damage(damage) {
                                damage_tilt_timer = 0.4;
                                damage_tilt_dir = 1.0;
                            }
                        }
                        player.fall_distance = 0.0;
                    }
                    if player.is_dead() {
                        state = GameState::DeathScreen;
                        mouse_capture.set_captured(false);
                        window.set_cursor_visibility(true);
                        death_hovered_btn = Some(0);
                        horiz_velocity = Vec3::zero();
                        fly_velocity = Vec3::zero();
                    }
                }
                
                if player.invulnerability_timer > 0.0 {
                    player.invulnerability_timer -= dt;
                }
                if damage_tilt_timer > 0.0 {
                    damage_tilt_timer = (damage_tilt_timer - dt).max(0.0);
                }

                if camera.position.y < -32.0 {
                    if player.take_damage(20.0) {
                        damage_tilt_timer = 0.4;
                    }
                    if !player.is_dead() {
                        camera.position.y = 80.0;
                        vertical_velocity = 0.0;
                    }
                }
                if camera.position.y > 250.0 {
                    camera.position.y = 250.0;
                    vertical_velocity = vertical_velocity.min(0.0);
                }

                if let Some((_sx, sy)) = window.get_scroll_wheel() {
                    if sy > 0.0 {
                        player.selected_hotbar_slot = if player.selected_hotbar_slot >= 9 { 0 } else { player.selected_hotbar_slot + 1 };
                    } else if sy < 0.0 {
                        player.selected_hotbar_slot = if player.selected_hotbar_slot <= 0 { 9 } else { player.selected_hotbar_slot - 1 };
                    }
                }
                if gp_events.lb_just_pressed || gp_events.dpad_left_just_pressed {
                    player.selected_hotbar_slot = if player.selected_hotbar_slot <= 0 { 9 } else { player.selected_hotbar_slot - 1 };
                }
                if gp_events.rb_just_pressed || gp_events.dpad_right_just_pressed {
                    player.selected_hotbar_slot = if player.selected_hotbar_slot >= 9 { 0 } else { player.selected_hotbar_slot + 1 };
                }
                for (key, id) in [
                    (Key::Key1, 0),
                    (Key::Key2, 1),
                    (Key::Key3, 2),
                    (Key::Key4, 3),
                    (Key::Key5, 4),
                    (Key::Key6, 5),
                    (Key::Key7, 6),
                    (Key::Key8, 7),
                    (Key::Key9, 8),
                    (Key::Key0, 9),
                ] {
                    if window.is_key_down(key) {
                        player.selected_hotbar_slot = id;
                    }
                }

                let mut selected_block_id = 0u8; // 0 = Air
                if let Some(stack) = player.inventory.hotbar[player.selected_hotbar_slot as usize] {
                    selected_block_id = stack.block.to_u8();
                } else if player.gamemode == GameMode::Creative {
                    let mut creative_override = 0u8;
                    for (key, id) in [
                        (Key::Key0, 15u8),
                        (Key::F1, 16),
                        (Key::F2, 17),
                        (Key::F3, 18),
                        (Key::F4, 19),
                    ] {
                        if window.is_key_down(key) {
                            creative_override = id;
                        }
                    }
                    if creative_override > 0 {
                        player.inventory.hotbar[player.selected_hotbar_slot as usize] = Some(ItemStack {
                            block: BlockId::from_u8(creative_override),
                            count: 1
                        });
                        selected_block_id = creative_override;
                    }
                }
                renderer.set_selected_block(selected_block_id);

                // Progressive block breaking (hold LMB or RT like Minecraft)
                let interact_left = (mouse_capture.is_captured() && left_down) || gp.btn_rt || gp.right_trigger > 0.15;
                let interact_left_just = (!left_was_down && left_down && mouse_capture.is_captured()) || gp_events.right_trigger_just_pressed;

                if interact_left {
                    if hand_animation < 0.1 {
                        hand_animation = 1.0;
                    }
                    let forward = camera.forward();
                    let held_tool = player.inventory.hotbar[player.selected_hotbar_slot as usize].map(|s| s.block);

                    // First frame: check for mob hit
                    if interact_left_just {
                        if let Some((mob_idx, _)) = mob_manager.raycast_mob(camera.position, forward, 4.2) {
                            let mob = &mut mob_manager.mobs[mob_idx];
                            let hit_pos = mob.center_pos();
                            let dmg = held_tool.map(|b| b.attack_damage()).unwrap_or(1.5);
                            mob.take_damage(dmg, forward);
                            particles.spawn_mob_hit(hit_pos);
                            break_target = None;
                            break_progress = 0.0;
                        }
                    }

                    if let Some(hit) = world.raycast(camera.position, forward, 8.0) {
                        let target = hit.block_pos;
                        let broken_block = world.get_block(target);

                        // Handle instant-break blocks (TNT, water, lava)
                        if broken_block == BlockId::Tnt && interact_left_just {
                            world.prime_tnt(target);
                            break_target = None;
                            break_progress = 0.0;
                        } else {
                            // Reset progress if we changed target
                            if break_target != Some(target) {
                                break_target = Some(target);
                                break_progress = 0.0;
                            }

                            let break_time = if player.gamemode == GameMode::Creative {
                                0.0 // Instant break in creative
                            } else {
                                broken_block.break_time_with_tool(held_tool)
                            };

                            if break_time == 0.0 {
                                // Instant break
                                break_progress = 1.0;
                            } else {
                                break_progress += dt / break_time;
                            }

                            if break_progress >= 1.0 {
                                break_progress = 0.0;
                                break_target = None;
                                hand_animation = 1.0;

                                let bx = target.x as f32 + 0.5;
                                let by = target.y as f32 + 0.5;
                                let bz = target.z as f32 + 0.5;
                                let color = broken_block.face_color(Face::Top);
                                particles.spawn_block_break(Vec3::new(bx - 0.5, by - 0.5, bz - 0.5), color);
                                world.set_block(target, BlockId::Air);

                                if player.gamemode == GameMode::Survival {
                                    let seed = ((target.x as u32).wrapping_mul(73856093) ^ (target.y as u32).wrapping_mul(19349663) ^ (target.z as u32).wrapping_mul(83492791)) % 1000;
                                    let rx = ((seed % 100) as f32 / 100.0 - 0.5) * 1.8;
                                    let rz = (((seed / 10) % 100) as f32 / 100.0 - 0.5) * 1.8;
                                    let drop_vel = Vec3::new(rx, 3.2, rz);
                                    items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, broken_block, 1));
                                }

                                if broken_block == BlockId::Door {
                                    let above = target + IVec3::new(0, 1, 0);
                                    let below = target - IVec3::new(0, 1, 0);
                                    if world.get_block(above) == BlockId::Door {
                                        world.set_block(above, BlockId::Air);
                                        if player.gamemode == GameMode::Survival {
                                            items.push(ItemEntity::new(Vec3::new(bx, by + 1.0, bz), Vec3::new(0.0, 3.2, 0.0), BlockId::Door, 1));
                                        }
                                    }
                                    if world.get_block(below) == BlockId::Door {
                                        world.set_block(below, BlockId::Air);
                                        if player.gamemode == GameMode::Survival {
                                            items.push(ItemEntity::new(Vec3::new(bx, by - 1.0, bz), Vec3::new(0.0, 3.2, 0.0), BlockId::Door, 1));
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // No block targeted -> reset
                        break_target = None;
                        break_progress = 0.0;
                    }
                } else {
                    // LMB/RT released -> reset
                    break_target = None;
                    break_progress = 0.0;
                }
                let interact_right = (mouse_capture.is_captured() && right_down && !right_was_down) || gp_events.left_trigger_just_pressed;
                if interact_right {
                    if let Some(hit) = world.raycast(camera.position, camera.forward(), 8.0) {
                        let targeted_block = world.get_block(hit.block_pos);
                        if targeted_block == BlockId::CraftingTable && !is_crouching {
                            crafting_table_open = true;
                            state = GameState::Inventory;
                            mouse_capture.set_captured(false);
                            window.set_cursor_visibility(true);
                            horiz_velocity = Vec3::zero();
                            fly_velocity = Vec3::zero();
                        } else {
                            let place_pos = hit.block_pos + hit.normal;
                            let place_id = BlockId::from_u8(selected_block_id);
                            // Only place (and animate) if we actually have a solid/placeable block
                            if place_id != BlockId::Air
                                && place_id.is_placeable()
                                && world.get_block(place_pos) == BlockId::Air
                            {
                                hand_animation = 1.0;
                                if place_id == BlockId::Door {
                                    let top_pos = place_pos + IVec3::new(0, 1, 0);
                                    if !block_intersects_player(place_pos, camera.position, current_eye_offset, current_player_height)
                                        && !block_intersects_player(top_pos, camera.position, current_eye_offset, current_player_height)
                                        && world.get_block(top_pos) == BlockId::Air
                                    {
                                        world.set_block(place_pos, BlockId::Door);
                                        world.set_block(top_pos, BlockId::Door);
                                        // Consume in survival
                                        if player.gamemode == GameMode::Survival {
                                            if let Some(stack) = &mut player.inventory.hotbar[player.selected_hotbar_slot as usize] {
                                                stack.count -= 1;
                                                if stack.count == 0 { player.inventory.hotbar[player.selected_hotbar_slot as usize] = None; }
                                            }
                                        }
                                    }
                                } else if !block_intersects_player(place_pos, camera.position, current_eye_offset, current_player_height) {
                                    world.set_block(place_pos, place_id);
                                    // Consume in survival
                                    if player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut player.inventory.hotbar[player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 { player.inventory.hotbar[player.selected_hotbar_slot as usize] = None; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                world.update(camera.position, settings.render_distance);
                let hand_speed = if fly_mode {
                    fly_velocity.length()
                } else {
                    horiz_velocity.length()
                };
                if hand_speed > 0.05 {
                    walk_phase += dt * (5.0 + hand_speed * 0.9);
                }
                material_timer += dt;
                if material_timer >= 0.25 {
                    material_timer = 0.0;
                    world.simulate_materials(camera.position);
                }

                world.update_primed_tnts(dt);

                mob_manager.update(dt, &world, camera.position);
                for mob in &mob_manager.mobs {
                    if !mob.is_dead {
                        let dist_sq = (mob.pos.x - camera.position.x).powi(2) + (mob.pos.z - camera.position.z).powi(2);
                        if dist_sq < 1.4 * 1.4 && (mob.pos.y - camera.position.y).abs() < 1.8 {
                            if player.take_damage(2.0) {
                                damage_tilt_timer = 0.4;
                                damage_tilt_dir = if mob.pos.x > camera.position.x { -1.0 } else { 1.0 };
                            }
                        }
                    }
                }
                for poof_pos in mob_manager.dying_poofs.drain(..) {
                    particles.spawn_mob_death_poof(poof_pos);
                }

                if player.is_dead() {
                    state = GameState::DeathScreen;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                    death_hovered_btn = Some(0);
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                if (q_down && !q_was_down && mouse_capture.is_captured()) || gp_events.west_just_pressed {
                    if let Some(stack) = &mut player.inventory.hotbar[player.selected_hotbar_slot as usize] {
                        items.push(ItemEntity::new(
                            camera.position + camera.forward() * 0.5,
                            camera.forward() * 5.0,
                            stack.block,
                            1
                        ));
                        stack.count -= 1;
                        if stack.count == 0 {
                            player.inventory.hotbar[player.selected_hotbar_slot as usize] = None;
                        }
                    }
                }

                let player_center = camera.position - Vec3::new(0.0, 0.8, 0.0);
                let mut i = 0;
                while i < items.len() {
                    items[i].update(dt, &world, Some(player_center));
                    let dist_sq = (items[i].position - player_center).length_sq();
                    if items[i].pickup_delay <= 0.0 && dist_sq < 3.24 {
                        let remaining = player.inventory.add_item(items[i].block, items[i].count);
                        if remaining == 0 {
                            items.remove(i);
                            continue;
                        } else {
                            items[i].count = remaining;
                        }
                    }
                    i += 1;
                }

                for exp_pos in world.pending_explosions.drain(..) {
                    particles.spawn_explosion(exp_pos);
                    mob_manager.apply_explosion(exp_pos, 5.0, 30.0);
                    let dist = (camera.position - exp_pos).length();
                    if dist < 6.0 {
                        let dmg = (1.0 - dist / 6.0) * 18.0;
                        if player.take_damage(dmg) {
                            damage_tilt_timer = 0.4;
                            damage_tilt_dir = if exp_pos.x > camera.position.x { -1.0 } else { 1.0 };
                        }
                    }
                }

                particles.update(dt);

                let aspect = WIDTH as f32 / HEIGHT as f32;

                if (camera.fov_y - last_fov).abs() > 0.0001 || (aspect - last_aspect).abs() > 0.0001 {
                    last_fov = camera.fov_y;
                    last_aspect = aspect;
                    proj_matrix = camera.projection_matrix(aspect);
                }
                let (shake_offset, roll) = if damage_tilt_timer > 0.0 {
                    let progress = damage_tilt_timer / 0.4;
                    let roll = (progress * std::f32::consts::PI).sin() * 0.14 * damage_tilt_dir;
                    let sx = (total_time * 50.0).sin() * 0.08 * progress;
                    let sy = (total_time * 60.0).cos() * 0.05 * progress;
                    (camera.right_flat() * sx + Vec3::new(0.0, sy, 0.0), roll)
                } else {
                    (Vec3::zero(), 0.0)
                };
                let view_proj = proj_matrix.mul(&camera.view_matrix_with_tilt(shake_offset, roll));
                renderer.fog_enabled = settings.fog;
                renderer.render_frame(&mut world, &camera);

                for mob in &mob_manager.mobs {
                    renderer.draw_mob(mob, &camera, &view_proj);
                }

                for tnt in &world.primed_tnts {
                    renderer.draw_tnt_cube(tnt.pos, tnt.fuse, &camera, &view_proj);
                }

                if settings.clouds {
                    renderer.draw_clouds(camera.position, &view_proj, total_time);
                }
                
                for item in &items {
                    let y_offset = (item.age * 3.0).sin() * 0.08;
                    let mut pos = item.position;
                    pos.y += y_offset;
                    let yaw = item.age * 2.5;
                    renderer.draw_item_3d(item.block, pos, yaw, &view_proj);
                }

                if let Some(target) = break_target {
                    if break_progress > 0.0 {
                        renderer.draw_block_break_cracks(target, break_progress, &view_proj);
                    }
                }

                let block_at_camera = world.get_block(camera.position.floor_to_ivec());
                if settings.underwater_fx && block_at_camera == BlockId::Water {
                    renderer.draw_underwater_overlay(total_time);
                }

                for p in &particles.particles {
                    renderer.draw_billboard(p.pos, p.size, p.color, &camera, &view_proj);
                }

                if damage_tilt_timer > 0.0 {
                    renderer.draw_damage_flash(damage_tilt_timer / 0.4);
                }
                if player.gamemode == GameMode::Survival && player.health <= 2.0 && player.health > 0.0 {
                    renderer.draw_low_health_vignette(total_time);
                }

                renderer.draw_crosshair();
                renderer.draw_break_progress(break_progress);
                let hand_block: Option<u8> = player.inventory.hotbar[player.selected_hotbar_slot as usize]
                    .map(|s| s.block.to_u8());
                renderer.draw_hand_3d(&camera, &view_proj, hand_block, walk_phase, hand_speed, hand_animation);
                renderer.draw_hotbar(&player);

                let mode_str = if fly_mode {
                    "LOT"
                } else if is_crouching {
                    "KUCANIE"
                } else if is_sprinting {
                    "BIEG"
                } else {
                    "NORMALNY"
                };

                // OPTYMALIZACJA 1: Wykorzystywanie wcześniej zaalokowanych buforów zamiast `vec![]` do generacji logów Debug w String[cite: 2]
                debug_lines[0].clear();
                let _ = write!(&mut debug_lines[0], "FPS: {}", fps_display);
                debug_lines[1].clear();
                let _ = write!(&mut debug_lines[1], "POZYCJA X:{} Y:{} Z:{}", camera.position.x as i32, camera.position.y as i32, camera.position.z as i32);
                debug_lines[2].clear();
                let _ = write!(&mut debug_lines[2], "BLOK: {}", BlockId::from_u8(renderer.selected_block()).name().to_uppercase());
                debug_lines[3].clear();
                let _ = write!(&mut debug_lines[3], "MOBY: {}", mob_manager.mobs.len());
                debug_lines[4].clear();
                let _ = write!(&mut debug_lines[4], "TRYB: {}", mode_str);

                renderer.draw_debug_panel(&debug_lines);

                if !mouse_capture.is_captured() {
                    renderer.draw_capture_prompt(mouse_capture.has_native_capture());
                }
            }
            GameState::Paused => {
                window.set_cursor_visibility(true);

                let t_fov = 1.0 - (-8.0 * dt).exp();
                camera.fov_y += (settings.fov_deg.to_radians() - camera.fov_y) * t_fov;

                let mut hovered_idx = pause_hovered_btn;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in pause_menu_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            pause_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                // Nawigacja D-Pad / Analog na padzie w Menu Pauzy
                if gp_events.dpad_up_just_pressed || (up_arrow_down && !up_arrow_was_down) {
                    let next = match pause_hovered_btn {
                        Some(0) | None => 4,
                        Some(i) => i - 1,
                    };
                    pause_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if gp_events.dpad_down_just_pressed || (down_arrow_down && !down_arrow_was_down) {
                    let next = match pause_hovered_btn {
                        Some(4) | None => 0,
                        Some(i) => i + 1,
                    };
                    pause_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_pause_btn = if left_down && !left_was_down || gp_events.south_just_pressed || (enter_down && !enter_was_down) {
                    hovered_idx
                } else {
                    None
                };

                if let Some(btn) = click_pause_btn {
                    match btn {
                        0 => {
                            state = GameState::Playing;
                            mouse_capture.set_captured(true);
                            window.set_cursor_visibility(false);
                            last_mouse = None;
                        }
                        1 => {
                            state = GameState::Settings(PreviousMenu::Paused);
                            settings_hovered_opt = Some(0);
                        }
                        2 => {
                            fly_mode = !fly_mode;
                        }
                        3 => {
                            let _ = save_current_world(&mut world, &camera, save_slot);
                            save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                            state = GameState::MainMenu;
                            mouse_capture.set_captured(false);
                            window.set_cursor_visibility(true);
                        }
                        4 => {
                            let _ = save_current_world(&mut world, &camera, save_slot);
                            break;
                        }
                        _ => {}
                    }
                }

                if (escape_down && !escape_was_down) || gp_events.east_just_pressed || gp_events.start_just_pressed {
                    state = GameState::Playing;
                    mouse_capture.set_captured(true);
                    window.set_cursor_visibility(false);
                    last_mouse = None;
                }
                if f_down && !fly_key_was_down {
                    fly_mode = !fly_mode;
                }
                if q_down && !q_was_down {
                    let _ = save_current_world(&mut world, &camera, save_slot);
                    break;
                }
                if m_down && !m_was_down {
                    let _ = save_current_world(&mut world, &camera, save_slot);
                    save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    state = GameState::MainMenu;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                }

                renderer.fog_enabled = settings.fog;
                renderer.render_frame(&mut world, &camera);
                renderer.draw_pause_menu(fly_mode, hovered_idx);
            }
            GameState::Inventory => {
                window.set_cursor_visibility(true);

                // Zmiana aktywnego slotu hotbara na padzie za pomocą LB / RB
                if gp_events.lb_just_pressed {
                    player.selected_hotbar_slot = if player.selected_hotbar_slot <= 0 { 9 } else { player.selected_hotbar_slot - 1 };
                }
                if gp_events.rb_just_pressed {
                    player.selected_hotbar_slot = if player.selected_hotbar_slot >= 9 { 0 } else { player.selected_hotbar_slot + 1 };
                }

                if (e_down && !e_was_down)
                    || (escape_down && !escape_was_down)
                    || gp_events.east_just_pressed
                    || gp_events.north_just_pressed
                    || gp_events.start_just_pressed
                {
                    player.inventory.return_crafting_items();
                    crafting_table_open = false;
                    state = GameState::Playing;
                    mouse_capture.set_captured(true);
                    window.set_cursor_visibility(false);
                    last_mouse = None;
                }

                let layout = InventoryLayout::new(renderer.width, renderer.height, crafting_table_open);
                let inv_hovered_target = mouse_pos.and_then(|(mx, my)| {
                    layout.find_slot_at(mx as i32, my as i32, player.gamemode)
                });

                let click_left = (left_down && !left_was_down) || gp_events.south_just_pressed;
                let click_right = (right_down && !right_was_down) || gp_events.west_just_pressed;

                if click_left || click_right {
                    if let Some(target) = inv_hovered_target {
                        if player.gamemode == GameMode::Creative {
                            if let InventorySlotTarget::Creative(id) = target {
                                player.inventory.hotbar[player.selected_hotbar_slot as usize] = Some(ItemStack {
                                    block: BlockId::from_u8(id),
                                    count: 64,
                                });
                            }
                        } else {
                            match target {
                                InventorySlotTarget::Hotbar(idx) => {
                                    let slot = &mut player.inventory.hotbar[idx];
                                    if click_left {
                                        Inventory::handle_slot_left_click(slot, &mut player.inventory.held_item);
                                    } else {
                                        Inventory::handle_slot_right_click(slot, &mut player.inventory.held_item);
                                    }
                                }
                                InventorySlotTarget::Main(idx) => {
                                    let slot = &mut player.inventory.main[idx];
                                    if click_left {
                                        Inventory::handle_slot_left_click(slot, &mut player.inventory.held_item);
                                    } else {
                                        Inventory::handle_slot_right_click(slot, &mut player.inventory.held_item);
                                    }
                                }
                                InventorySlotTarget::Crafting2x2(idx) => {
                                    let slot = &mut player.inventory.crafting_2x2[idx];
                                    if click_left {
                                        Inventory::handle_slot_left_click(slot, &mut player.inventory.held_item);
                                    } else {
                                        Inventory::handle_slot_right_click(slot, &mut player.inventory.held_item);
                                    }
                                }
                                InventorySlotTarget::Crafting3x3(idx) => {
                                    let slot = &mut player.inventory.crafting_3x3[idx];
                                    if click_left {
                                        Inventory::handle_slot_left_click(slot, &mut player.inventory.held_item);
                                    } else {
                                        Inventory::handle_slot_right_click(slot, &mut player.inventory.held_item);
                                    }
                                }
                                InventorySlotTarget::CraftingOutput => {
                                    let result = if crafting_table_open {
                                        recipe_book.match_grid(&player.inventory.crafting_3x3, 3, 3)
                                    } else {
                                        recipe_book.match_grid(&player.inventory.crafting_2x2, 2, 2)
                                    };
                                    if let Some(res) = result {
                                        let can_take = match player.inventory.held_item {
                                            None => true,
                                            Some(h) => h.block == res.block && h.count + res.count <= 64,
                                        };
                                        if can_take {
                                            if let Some(h) = &mut player.inventory.held_item {
                                                h.count += res.count;
                                            } else {
                                                player.inventory.held_item = Some(res);
                                            }
                                            if crafting_table_open {
                                                RecipeBook::consume_grid(&mut player.inventory.crafting_3x3);
                                            } else {
                                                RecipeBook::consume_grid(&mut player.inventory.crafting_2x2);
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                renderer.render_frame(&mut world, &camera);
                renderer.draw_inventory(
                    &player,
                    inv_hovered_target,
                    crafting_table_open,
                    &recipe_book,
                    mouse_pos,
                );
            }
            GameState::DeathScreen => {
                window.set_cursor_visibility(true);

                let mut hovered_idx = death_hovered_btn;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in death_screen_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            death_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                if gp_events.dpad_up_just_pressed || (up_arrow_down && !up_arrow_was_down) {
                    let next = match death_hovered_btn {
                        Some(0) | None => 1,
                        Some(i) => i - 1,
                    };
                    death_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if gp_events.dpad_down_just_pressed || (down_arrow_down && !down_arrow_was_down) {
                    let next = match death_hovered_btn {
                        Some(1) | None => 0,
                        Some(i) => i + 1,
                    };
                    death_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_death_btn = if left_down && !left_was_down || gp_events.south_just_pressed || (enter_down && !enter_was_down) {
                    hovered_idx
                } else {
                    None
                };

                if let Some(btn) = click_death_btn {
                    match btn {
                        0 => {
                            // Odrodzenie
                            player.health = player.max_health;
                            camera.position = Vec3::new(8.0, 90.0, 8.0);
                            camera.pitch = 0.0;
                            vertical_velocity = 0.0;
                            horiz_velocity = Vec3::zero();
                            player.fall_distance = 0.0;
                            player.invulnerability_timer = 3.0;
                            player.has_landed = false;
                            damage_tilt_timer = 0.0;
                            state = GameState::Playing;
                            mouse_capture.set_captured(true);
                            window.set_cursor_visibility(false);
                            last_mouse = None;
                        }
                        1 => {
                            // Menu główne
                            let _ = save_current_world(&mut world, &camera, save_slot);
                            save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                            state = GameState::MainMenu;
                            mouse_capture.set_captured(false);
                            window.set_cursor_visibility(true);
                        }
                        _ => {}
                    }
                }

                if (escape_down && !escape_was_down) || gp_events.east_just_pressed {
                    let _ = save_current_world(&mut world, &camera, save_slot);
                    save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    state = GameState::MainMenu;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                }

                renderer.fog_enabled = settings.fog;
                renderer.render_frame(&mut world, &camera);
                renderer.draw_death_screen(hovered_idx);
            }
        }

        left_was_down = left_down;
        right_was_down = right_down;
        escape_was_down = escape_down;
        e_was_down = e_down;
        fly_key_was_down = f_down;
        q_was_down = q_down;
        m_was_down = m_down;
        save_was_down = f5_down;
        enter_was_down = enter_down;
        up_arrow_was_down = up_arrow_down;
        down_arrow_was_down = down_arrow_down;
        n_was_down = n_down;
        del_was_down = del_down;
        left_arrow_was_down = left_arrow_down;
        right_arrow_was_down = right_arrow_down;

        // OPTYMALIZACJA 3: Eliminacja przesyłu klatek za pomocą PCIe dla urządzeń, które wykorzystują architekturę i okna bezpośrednio w swapchain na Vulkan.[cite: 2]
        window
        .update_with_buffer(renderer.pixel_buffer(), WIDTH, HEIGHT)
        .expect("Blad podczas aktualizacji bufora okna");

        frame_count += 1;
        if fps_timer.elapsed().as_secs_f32() >= 0.5 {
            fps_display = (frame_count as f32 / fps_timer.elapsed().as_secs_f32()) as u32;
            frame_count = 0;
            fps_timer = Instant::now();
        }

        let new_title = format!(
            "RustCraft [{} renderer] - {} FPS - blok: {}{}{}",
            renderer.backend_name(),
                                fps_display,
                                BlockId::from_u8(renderer.selected_block()).name(),
                                if fly_mode { " - [LOT]" } else { "" },
                                    if matches!(state, GameState::Paused) { " - [PAUZA]" } else { "" }
        );
        if new_title != last_title {
            window.set_title(&new_title);
            last_title = new_title;
        }
    }

    if matches!(state, GameState::Playing | GameState::Paused) {
        let _ = save_current_world(&mut world, &camera, save_slot);
    }
}
