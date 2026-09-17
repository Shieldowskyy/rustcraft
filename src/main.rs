// RustCraft - a tiny Minecraft-like voxel game with selectable software/Vulkan backends.
// All rasterization (triangles, depth-buffer, lighting) happens on the CPU
// into a plain pixel buffer, which is then blitted to a window via `minifb`.
// `minifb` is used ONLY for creating an OS window and reading keyboard/mouse
// input - it performs no rendering itself.

mod math;
mod camera;
mod world;
mod renderer;
mod mesher;
mod block;
mod noise;
mod texture;
mod font;
mod mouse_capture;
mod vulkan;
mod mob;
mod particles;
mod physics;
mod cli;
mod saves;

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions, CursorStyle};
use std::time::Instant;
use std::fmt::Write; // Wymagane dla optymalizacji formatowania tekstu

use camera::Camera;
use math::Vec3;
use mob::MobManager;
use mouse_capture::MouseCapture;
use particles::ParticleSystem;
use renderer::Renderer;
use world::World;

use cli::{parse_renderer_choice, parse_seed, generate_random_seed};
use physics::{
    BASE_FOV_DEG, CROUCH_EYE_OFFSET, CROUCH_FOV_DEG, CROUCH_HEIGHT, GRAVITY, JUMP_SPEED,
    SPRINT_FOV_DEG, STANDING_EYE_OFFSET, STANDING_HEIGHT, TERMINAL_VELOCITY,
    block_intersects_player, move_player_with_collision,
};
use saves::{save_current_world, save_path};

const WIDTH: usize = 960;
const HEIGHT: usize = 540;
const TARGET_FPS: usize = 300;
const MAX_SAVE_SLOTS: u8 = 32;

#[derive(Copy, Clone, PartialEq, Eq)]
enum GameState {
    MainMenu,
    Playing,
    Paused,
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

    let mut state = GameState::MainMenu;
    let mut save_slots: [bool; MAX_SAVE_SLOTS as usize] = std::array::from_fn(|index| {
        let path = save_path(index as u8 + 1);
        std::path::Path::new(&path).exists()
    });

    let mut particles = ParticleSystem::new();
    let mut mob_manager = MobManager::new();
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
                    world = World::new_with_seed(seed);
                    camera = Camera::new(Vec3::new(8.0, 90.0, 8.0));
                    mob_manager = MobManager::new();
                    state = GameState::Playing;
                } else if load_down && !load_was_down || click_button == Some(1) {
                    let path = save_path(save_slot);
                    match World::load(&path) {
                        Ok(loaded_world) => {
                            seed = loaded_world.seed;
                            let loaded_position = loaded_world.player_position;
                            world = loaded_world;
                            camera = Camera::new(loaded_position);
                            mob_manager = MobManager::new();
                            state = GameState::Playing;
                        }
                        Err(error) => eprintln!("{error}"),
                    }
                } else if q_down && !q_was_down || click_button == Some(2) {
                    break;
                }
                renderer.draw_main_menu(seed, save_slot, &save_slots, hovered_idx);
                enter_was_down = enter_down;
                load_was_down = load_down;
                slot_left_was_down = slot_left_down;
                slot_right_was_down = slot_right_down;
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

                if f_down && !fly_key_was_down {
                    fly_mode = !fly_mode;
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

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
                    2.0
                } else if is_sprinting {
                    8.5
                } else {
                    4.5
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

                    let target_fly = move_dir * (move_speed * 1.6);
                    let fly_damping = 12.0;
                    let t_fly = 1.0 - (-fly_damping * dt).exp();
                    fly_velocity = fly_velocity.lerp(target_fly, t_fly);

                    camera.position = camera.position + fly_velocity * dt;
                    vertical_velocity = 0.0;
                    horiz_velocity = Vec3::zero();
                } else {
                    let target_horiz = wish_dir * move_speed;
                    let feet_block = world.get_block(Vec3::new(
                        camera.position.x,
                        camera.position.y - current_eye_offset - 0.08,
                        camera.position.z,
                    ).floor_to_ivec());
                    let on_ice = feet_block == block::BlockId::Ice;
                    let damping = if on_ground {
                        if on_ice { 2.0 } else if is_crouching { 18.0 } else { 14.0 }
                    } else {
                        4.5
                    };
                    let t_horiz = 1.0 - (-damping * dt).exp();
                    horiz_velocity = horiz_velocity.lerp(target_horiz, t_horiz);

                    vertical_velocity -= GRAVITY * dt;
                    if vertical_velocity < -TERMINAL_VELOCITY {
                        vertical_velocity = -TERMINAL_VELOCITY;
                    }

                    if window.is_key_down(Key::Space) && on_ground && mouse_capture.is_captured() && !is_crouching {
                        vertical_velocity = JUMP_SPEED;
                    }

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
                }

                if camera.position.y < -32.0 {
                    camera.position.y = 80.0;
                    vertical_velocity = 0.0;
                }
                if camera.position.y > 250.0 {
                    camera.position.y = 250.0;
                    vertical_velocity = vertical_velocity.min(0.0);
                }

                let mut selected_block = renderer.selected_block();
                if let Some((_sx, sy)) = window.get_scroll_wheel() {
                    if sy > 0.0 {
                        selected_block = if selected_block >= 19 { 1 } else { selected_block + 1 };
                    } else if sy < 0.0 {
                        selected_block = if selected_block <= 1 { 19 } else { selected_block - 1 };
                    }
                }
                for (key, id) in [
                    (Key::Key1, 1u8),
                    (Key::Key2, 2),
                    (Key::Key3, 3),
                    (Key::Key4, 4),
                    (Key::Key5, 5),
                    (Key::Key6, 6),
                    (Key::Key7, 7),
                    (Key::Key8, 8),
                    (Key::Key9, 9),
                    (Key::Key0, 15),
                    (Key::F1, 16),
                    (Key::F2, 17),
                    (Key::F3, 18),
                    (Key::F4, 19),
                    (Key::I, 18),
                ] {
                    if window.is_key_down(key) {
                        selected_block = id;
                    }
                }
                renderer.set_selected_block(selected_block);

                let interact_left = mouse_capture.is_captured() && left_down;
                let interact_right = mouse_capture.is_captured() && right_down;

                if interact_left && !left_was_down {
                    hand_animation = 1.0;
                    let forward = camera.forward();
                    if let Some((mob_idx, _hit_dist)) = mob_manager.raycast_mob(camera.position, forward, 4.2) {
                        let mob = &mut mob_manager.mobs[mob_idx];
                        let hit_pos = mob.center_pos();
                        mob.take_damage(4.0, forward);
                        particles.spawn_mob_hit(hit_pos);
                    } else if let Some(hit) = world.raycast(camera.position, forward, 8.0) {
                        let broken_block = world.get_block(hit.block_pos);
                        if broken_block == block::BlockId::Tnt {
                            world.prime_tnt(hit.block_pos);
                        } else {
                            let bx = hit.block_pos.x as f32;
                            let by = hit.block_pos.y as f32;
                            let bz = hit.block_pos.z as f32;
                            let color = broken_block.face_color(block::Face::Top);
                            particles.spawn_block_break(Vec3::new(bx, by, bz), color);
                            world.set_block(hit.block_pos, block::BlockId::Air);

                            if broken_block == block::BlockId::Door {
                                let above = hit.block_pos + crate::math::IVec3::new(0, 1, 0);
                                let below = hit.block_pos - crate::math::IVec3::new(0, 1, 0);
                                if world.get_block(above) == block::BlockId::Door {
                                    world.set_block(above, block::BlockId::Air);
                                }
                                if world.get_block(below) == block::BlockId::Door {
                                    world.set_block(below, block::BlockId::Air);
                                }
                            }
                        }
                    }
                }
                if interact_right && !right_was_down {
                    hand_animation = 1.0;
                    if let Some(hit) = world.raycast(camera.position, camera.forward(), 8.0) {
                        let place_pos = hit.block_pos + hit.normal;
                        let place_id = block::BlockId::from_u8(selected_block);
                        if place_id == block::BlockId::Door {
                            let top_pos = place_pos + crate::math::IVec3::new(0, 1, 0);
                            if !block_intersects_player(place_pos, camera.position, current_eye_offset, current_player_height)
                                && !block_intersects_player(top_pos, camera.position, current_eye_offset, current_player_height)
                                && world.get_block(top_pos) == block::BlockId::Air
                                {
                                    world.set_block(place_pos, block::BlockId::Door);
                                    world.set_block(top_pos, block::BlockId::Door);
                                }
                        } else if !block_intersects_player(place_pos, camera.position, current_eye_offset, current_player_height) {
                            world.set_block(place_pos, place_id);
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
                let block_at_camera = world.get_block(camera.position.floor_to_ivec());
                if block_at_camera == block::BlockId::Water {
                    renderer.draw_underwater_overlay(total_time);
                }

                for p in &particles.particles {
                    renderer.draw_billboard(p.pos, p.size, p.color, &camera, &view_proj);
                }
                renderer.draw_crosshair();
                renderer.draw_hand_3d(&camera, &view_proj, renderer.selected_block(), walk_phase, hand_speed, hand_animation);
                renderer.draw_hotbar();

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
                let _ = write!(&mut debug_lines[2], "BLOK: {}", block::BlockId::from_u8(selected_block).name().to_uppercase());
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
        }

        left_was_down = left_down;
        right_was_down = right_down;
        escape_was_down = escape_down;
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
                                block::BlockId::from_u8(renderer.selected_block()).name(),
                                if fly_mode { " - [LOT]" } else { "" },
                                    if state == GameState::Paused { " - [PAUZA]" } else { "" }
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
