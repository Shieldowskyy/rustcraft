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
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::MobManager;
use crate::gameplay::particles::ParticleSystem;
use crate::gameplay::physics::{
    block_intersects_player, move_player_with_collision, BASE_FOV_DEG, CROUCH_EYE_OFFSET,
    CROUCH_FOV_DEG, CROUCH_HEIGHT, FLY_SPEED, GRAVITY, JUMP_SPEED, SNEAK_SPEED, SPRINT_FOV_DEG,
    SPRINT_JUMP_BOOST, SPRINT_SPEED, STANDING_EYE_OFFSET, STANDING_HEIGHT, SWIM_SPEED,
    TERMINAL_VELOCITY, WALK_SPEED,
};
use crate::gameplay::player::{GameMode, Player};
use crate::graphics::camera::Camera;
use crate::graphics::software::Renderer;
use crate::platform::mouse_capture::{self, MouseCapture};
use crate::world::block::{BlockId, Face};
use crate::world::storage::{save_current_world, save_path};
use crate::world::World;

const WIDTH: usize = 960;
const HEIGHT: usize = 540;
const TARGET_FPS: usize = 300;
const MAX_SAVE_SLOTS: u8 = 32;

enum GameState {
    MainMenu,
    Loading(mpsc::Receiver<WorldLoadResult>),
    Playing,
    Paused,
    Inventory,
}

enum WorldLoadResult {
    Success(World, Option<Vec3>),
    Error(String),
}

fn main() {
    let renderer_choice = parse_renderer_choice();
    let mut seed = parse_seed();
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
    let mut load_was_down = false;
    let mut save_was_down = false;
    let mut slot_left_was_down = false;
    let mut slot_right_was_down = false;
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
    let mut save_slots: [bool; MAX_SAVE_SLOTS as usize] = std::array::from_fn(|index| {
        let path = save_path(index as u8 + 1);
        std::path::Path::new(&path).exists()
    });

    let mut particles = ParticleSystem::new();
    let mut mob_manager = MobManager::new();
    let mut player = Player::new();
    player.gamemode = GameMode::Survival; // Start in survival as requested
    let mut items: Vec<ItemEntity> = Vec::new();
    
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
    let world_rects = renderer.main_menu_world_rects();
    let pause_menu_rects = renderer.pause_menu_button_rects();

    println!("Sterowanie:");
    println!("  Kliknij w okno - zablokuj/przechwyc mysz");
    println!("  Esc            - Menu pauzy / zwolnij mysz (w pauzie: powrot do gry)");
    println!("  WASD           - ruch (z plynna inercja/akceleracja)");
    println!("  Spacja         - skok (w trybie lotu: w gore)");
    println!("  Shift (lewy)   - kucanie (w trybie lotu: w dol)");
    println!("  Ctrl (lewy)    - bieg (z plynna zmiana FOV)");
    println!("  F              - przelacz tryb lotu (bez kolizji/grawitacji)");
    println!("  Mysz           - rozgladanie sie (gdy przechwycona)");
    println!("  LPM            - niszcz blok / bij moby / uderz TNT aby podpalic / kliknij przycisk");
    println!("  PPM            - stawiaj blok");
    println!("  Scroll / 1-9, F1-F4 - wybor typu bloku do stawiania");
    println!("  Q              - wyjscie z gry w menu pauzy");

    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32().min(0.1);
        last_time = now;
        total_time += dt;
        hand_animation = (hand_animation - dt * 3.5).max(0.0);

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

        match state {
            GameState::MainMenu => {
                let enter_down = window.is_key_down(Key::Enter);
                let load_down = window.is_key_down(Key::L);
                let slot_left_down = window.is_key_down(Key::Left);
                let slot_right_down = window.is_key_down(Key::Right);

                let mut hovered_idx = None;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in main_menu_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            break;
                        }
                    }
                }
                let mut clicked_world = false;
                if left_down && !left_was_down {
                    if let Some((mx, my)) = mouse_pos {
                        let px = mx as i32;
                        let py = my as i32;
                        for (row, &(bx, by, bw, bh)) in world_rects.iter().enumerate() {
                            if px >= bx && px < bx + bw && py >= by && py < by + bh {
                                save_slot = world_slot_for_row(save_slot, row);
                                clicked_world = true;
                                break;
                            }
                        }
                    }
                }
                if slot_left_down && !slot_left_was_down {
                    save_slot = if save_slot == 1 { MAX_SAVE_SLOTS } else { save_slot - 1 };
                }
                if slot_right_down && !slot_right_was_down {
                    save_slot = if save_slot == MAX_SAVE_SLOTS { 1 } else { save_slot + 1 };
                }
                let click_button = if !clicked_world && left_down && !left_was_down { hovered_idx } else { None };

                if enter_down && !enter_was_down || click_button == Some(0) {
                    seed = generate_random_seed();
                    let (tx, rx) = mpsc::channel();
                    let thread_seed = seed;

                    thread::spawn(move || {
                        let new_world = World::new_with_seed(thread_seed);
                        let _ = tx.send(WorldLoadResult::Success(new_world, None));
                    });

                    state = GameState::Loading(rx);
                } else if load_down && !load_was_down || click_button == Some(1) {
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
                } else if q_down && !q_was_down || click_button == Some(2) {
                    break;
                }
                renderer.draw_main_menu(seed, save_slot, &save_slots, hovered_idx);
                enter_was_down = enter_down;
                load_was_down = load_down;
                slot_left_was_down = slot_left_down;
                slot_right_was_down = slot_right_down;
            }
            GameState::Loading(ref rx) => {
                // Gra nadal renderuje ekran menu, podczas gdy wątek oblicza świat (zapobiega blokowaniu okna)
                renderer.draw_main_menu(seed, save_slot, &save_slots, None);

                // Sprawdzamy czy świat został wygenerowany/załadowany
                if let Ok(result) = rx.try_recv() {
                    match result {
                        WorldLoadResult::Success(loaded_world, loaded_pos) => {
                            seed = loaded_world.seed;
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

                // Utrzymanie aktualnych stanów klawiszy w trakcie ładowania
                enter_was_down = window.is_key_down(Key::Enter);
                load_was_down = window.is_key_down(Key::L);
                slot_left_was_down = window.is_key_down(Key::Left);
                slot_right_was_down = window.is_key_down(Key::Right);
            }
            GameState::Playing => {
                if f5_down && !save_was_down {
                    let path = save_path(save_slot);
                    match save_current_world(&mut world, &camera, save_slot) {
                        Ok(()) => println!("Zapisano świat do {path}"),
                        Err(error) => eprintln!("{error}"),
                    }
                }
                if escape_down && !escape_was_down {
                    state = GameState::Paused;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                if e_down && !e_was_down {
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
                if wish_dir.length_sq() > 0.0001 {
                    wish_dir = wish_dir.normalized();
                }

                let f_down = window.is_key_down(Key::F);
                let g_down = window.is_key_down(Key::G);
                if f_down && !fly_key_was_down {
                    fly_mode = !fly_mode;
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                if g_down && !g_was_down {
                    player.gamemode = if player.gamemode == GameMode::Survival {
                        GameMode::Creative
                    } else {
                        GameMode::Survival
                    };
                }
                g_was_down = g_down;

                let is_crouching = !fly_mode
                && mouse_capture.is_captured()
                && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::C));

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
                && mouse_capture.is_captured()
                && window.is_key_down(Key::LeftCtrl)
                && wish_dir.length_sq() > 0.01;

                let target_fov_deg = if is_sprinting {
                    SPRINT_FOV_DEG
                } else if is_crouching {
                    CROUCH_FOV_DEG
                } else {
                    BASE_FOV_DEG
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
                    if mouse_capture.is_captured() {
                        if window.is_key_down(Key::Space) {
                            move_dir.y += 1.0;
                        }
                        if window.is_key_down(Key::LeftShift) {
                            move_dir.y -= 1.0;
                        }
                    }
                    if move_dir.length_sq() > 0.0001 {
                        move_dir = move_dir.normalized();
                    }

                    let fly_fast = window.is_key_down(Key::LeftCtrl);
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
                        if window.is_key_down(Key::Space) && mouse_capture.is_captured() {
                            vertical_velocity = 3.2;
                        }
                    } else {
                        vertical_velocity -= GRAVITY * dt;
                        if vertical_velocity < -TERMINAL_VELOCITY {
                            vertical_velocity = -TERMINAL_VELOCITY;
                        }

                        if window.is_key_down(Key::Space) && on_ground && mouse_capture.is_captured() && !is_crouching {
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
                            player.take_damage(damage);
                        }
                        player.fall_distance = 0.0;
                    }
                    if player.is_dead() {
                        player.health = player.max_health;
                        camera.position = Vec3::new(8.0, 90.0, 8.0);
                        vertical_velocity = 0.0;
                        horiz_velocity = Vec3::zero();
                        player.fall_distance = 0.0;
                        player.invulnerability_timer = 3.0;
                        player.has_landed = false;
                    }
                }
                
                if player.invulnerability_timer > 0.0 {
                    player.invulnerability_timer -= dt;
                }

                if camera.position.y < -32.0 {
                    camera.position.y = 80.0;
                    vertical_velocity = 0.0;
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

                // Progressive block breaking (hold LMB like Minecraft)
                let interact_left = mouse_capture.is_captured() && left_down;

                if interact_left {
                    let forward = camera.forward();

                    // First frame: check for mob hit
                    if !left_was_down {
                        hand_animation = 1.0;
                        if let Some((mob_idx, _)) = mob_manager.raycast_mob(camera.position, forward, 4.2) {
                            let mob = &mut mob_manager.mobs[mob_idx];
                            let hit_pos = mob.center_pos();
                            mob.take_damage(4.0, forward);
                            particles.spawn_mob_hit(hit_pos);
                            break_target = None;
                            break_progress = 0.0;
                        }
                    }

                    if let Some(hit) = world.raycast(camera.position, forward, 8.0) {
                        let target = hit.block_pos;
                        let broken_block = world.get_block(target);

                        // Handle instant-break blocks (TNT, water, lava)
                        if broken_block == BlockId::Tnt && !left_was_down {
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
                                broken_block.break_time()
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
                                    let drop_vel = Vec3::new(0.0, 3.0, 0.0);
                                    items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, broken_block, 1));
                                }

                                if broken_block == BlockId::Door {
                                    let above = target + IVec3::new(0, 1, 0);
                                    let below = target - IVec3::new(0, 1, 0);
                                    if world.get_block(above) == BlockId::Door {
                                        world.set_block(above, BlockId::Air);
                                        if player.gamemode == GameMode::Survival {
                                            items.push(ItemEntity::new(Vec3::new(bx, by + 1.0, bz), Vec3::new(0.0, 3.0, 0.0), BlockId::Door, 1));
                                        }
                                    }
                                    if world.get_block(below) == BlockId::Door {
                                        world.set_block(below, BlockId::Air);
                                        if player.gamemode == GameMode::Survival {
                                            items.push(ItemEntity::new(Vec3::new(bx, by - 1.0, bz), Vec3::new(0.0, 3.0, 0.0), BlockId::Door, 1));
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
                    // LMB released -> reset
                    break_target = None;
                    break_progress = 0.0;
                }
                let interact_right = mouse_capture.is_captured() && right_down;
                if interact_right && !right_was_down {
                    if let Some(hit) = world.raycast(camera.position, camera.forward(), 8.0) {
                        let place_pos = hit.block_pos + hit.normal;
                        let place_id = BlockId::from_u8(selected_block_id);
                        // Only place (and animate) if we actually have a block to place
                        if place_id != BlockId::Air
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

                world.update(camera.position);
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
                for poof_pos in mob_manager.dying_poofs.drain(..) {
                    particles.spawn_mob_death_poof(poof_pos);
                }

                if q_down && !q_was_down && mouse_capture.is_captured() {
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

                let mut i = 0;
                while i < items.len() {
                    items[i].update(dt, &world);
                    let dist_sq = (items[i].position - camera.position).length_sq();
                    if items[i].pickup_delay <= 0.0 && dist_sq < 2.25 {
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
                }

                particles.update(dt);

                let aspect = WIDTH as f32 / HEIGHT as f32;

                // OPTYMALIZACJA 2: Odświeżanie projekcji wyłącznie przy zmianie paramentów, unikanie niepotrzebnych instrukcji FPU[cite: 2]
                if (camera.fov_y - last_fov).abs() > 0.0001 || (aspect - last_aspect).abs() > 0.0001 {
                    last_fov = camera.fov_y;
                    last_aspect = aspect;
                    proj_matrix = camera.projection_matrix(aspect);
                }
                let view_proj = proj_matrix.mul(&camera.view_matrix());

                renderer.render_frame(&mut world, &camera);

                for mob in &mob_manager.mobs {
                    renderer.draw_mob(mob, &camera, &view_proj);
                }

                for tnt in &world.primed_tnts {
                    renderer.draw_tnt_cube(tnt.pos, tnt.fuse, &camera, &view_proj);
                }

                renderer.draw_clouds(camera.position, &view_proj, total_time);
                
                for item in &items {
                    let color = item.block.face_color(Face::Top);
                    let color_arr = [color[0], color[1], color[2]];
                    let y_offset = (item.age * 2.0).sin() * 0.1;
                    let mut pos = item.position;
                    pos.y += y_offset;
                    renderer.draw_billboard(pos, 0.3, color_arr, &camera, &view_proj);
                }

                let block_at_camera = world.get_block(camera.position.floor_to_ivec());
                if block_at_camera == BlockId::Water {
                    renderer.draw_underwater_overlay(total_time);
                }

                for p in &particles.particles {
                    renderer.draw_billboard(p.pos, p.size, p.color, &camera, &view_proj);
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
                camera.fov_y += (BASE_FOV_DEG.to_radians() - camera.fov_y) * t_fov;

                let mut hovered_idx = None;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in pause_menu_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            break;
                        }
                    }
                }

                if left_down && !left_was_down {
                    match hovered_idx {
                        Some(0) => {
                            state = GameState::Playing;
                            mouse_capture.set_captured(true);
                            window.set_cursor_visibility(false);
                            last_mouse = None;
                        }
                        Some(1) => {
                            fly_mode = !fly_mode;
                        }
                        Some(2) => {
                            let _ = save_current_world(&mut world, &camera, save_slot);
                            save_slots[save_slot as usize - 1] = true;
                            state = GameState::MainMenu;
                            mouse_capture.set_captured(false);
                            window.set_cursor_visibility(true);
                        }
                        Some(3) => {
                            let _ = save_current_world(&mut world, &camera, save_slot);
                            break;
                        }
                        _ => {}
                    }
                }

                if escape_down && !escape_was_down {
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
                    save_slots[save_slot as usize - 1] = true;
                    state = GameState::MainMenu;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                }

                renderer.render_frame(&mut world, &camera);
                renderer.draw_pause_menu(fly_mode, hovered_idx);
            }
            GameState::Inventory => {
                window.set_cursor_visibility(true);
                if e_down && !e_was_down || (escape_down && !escape_was_down) {
                    state = GameState::Playing;
                    mouse_capture.set_captured(true);
                    window.set_cursor_visibility(false);
                    last_mouse = None;
                }

                if left_down && !left_was_down {
                    if let Some((mx, my)) = mouse_pos {
                        let px = mx as i32;
                        let py = my as i32;
                        // Render inventory grid and check clicks
                        // We will draw it via renderer, but handle clicks here
                        let start_x = renderer.width as i32 / 2 - 200;
                        let start_y = renderer.height as i32 / 2 - 150;
                        let mut block_id = 1;
                        'outer: for row in 0..5 {
                            for col in 0..8 {
                                if block_id > 19 { break 'outer; }
                                let x = start_x + col * 40;
                                let y = start_y + row * 40;
                                if px >= x && px < x + 36 && py >= y && py < y + 36 {
                                    player.inventory.hotbar[player.selected_hotbar_slot as usize] = Some(ItemStack {
                                        block: BlockId::from_u8(block_id),
                                        count: 64,
                                    });
                                }
                                block_id += 1;
                            }
                        }
                    }
                }

                renderer.render_frame(&mut world, &camera);
                // Draw inventory
                renderer.draw_inventory(&player);
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

fn world_slot_for_row(selected_slot: u8, row: usize) -> u8 {
    ((selected_slot as i32 - 1 + row as i32 + 32 - 2).rem_euclid(32) + 1) as u8
}
