// RustCraft - a tiny Minecraft-like voxel game with a 100% software renderer.
// No GPU APIs (no OpenGL/Vulkan/DirectX/wgpu) are used for rendering.
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

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions, CursorStyle};
use std::time::Instant;

use camera::Camera;
use math::Vec3;
use mouse_capture::MouseCapture;
use renderer::Renderer;
use world::World;

const WIDTH: usize = 960;
const HEIGHT: usize = 540;

// --- Player physics constants ---
const GRAVITY: f32 = 24.0; // blocks/sec^2
const TERMINAL_VELOCITY: f32 = 40.0; // blocks/sec
const JUMP_SPEED: f32 = 8.5; // blocks/sec, initial upward velocity
const PLAYER_HALF_WIDTH: f32 = 0.3; // player collision box half-extent on X/Z
const STANDING_HEIGHT: f32 = 1.8; // standing collision box height
const STANDING_EYE_OFFSET: f32 = 1.6; // eye height above player's feet when standing
const CROUCH_HEIGHT: f32 = 1.4; // crouched collision box height
const CROUCH_EYE_OFFSET: f32 = 1.25; // eye height above player's feet when crouching

const BASE_FOV_DEG: f32 = 70.0;
const SPRINT_FOV_DEG: f32 = 83.0;
const CROUCH_FOV_DEG: f32 = 66.0;

#[derive(Copy, Clone, PartialEq, Eq)]
enum GameState {
    Playing,
    Paused,
}

fn main() {
    // If DISPLAY is set on Linux, prefer X11 over Wayland for minifb so that
    // hardware/X11 pointer grabbing and warping (XGrabPointer, XWarpPointer) works.
    #[cfg(target_os = "linux")]
    if std::env::var_os("DISPLAY").is_some() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let mut window = Window::new(
        "RustCraft - Software Rendered Voxel Engine",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("Nie udało się utworzyć okna");

    window.set_target_fps(60);
    window.set_cursor_visibility(false);
    let _ = window.set_cursor_style(CursorStyle::Crosshair);

    let mut win_size = window.get_size();

    let mut renderer = Renderer::new(WIDTH, HEIGHT);
    let mut world = World::new();
    let mut camera = Camera::new(Vec3::new(8.0, 90.0, 8.0));

    let mut mouse_capture = MouseCapture::new(&window, win_size.0 as i32, win_size.1 as i32);
    // Start un-captured so the player sees the window and clicks to begin,
    // matching the expected FPS-game convention.
    mouse_capture.set_captured(false);

    let mut last_time = Instant::now();
    let mut last_mouse: Option<(f32, f32)> = None;

    // Frame counter shown in the title bar as a simple FPS indicator.
    let mut fps_timer = Instant::now();
    let mut frame_count = 0u32;
    let mut fps_display = 0u32;

    // Debounce flags so a single key/click doesn't trigger multiple times.
    let mut left_was_down = false;
    let mut right_was_down = false;
    let mut escape_was_down = false;
    let mut fly_key_was_down = false;
    let mut q_was_down = false;

    // --- Player physics state ---
    let mut vertical_velocity: f32 = 0.0;
    let mut horiz_velocity = Vec3::zero();
    let mut fly_velocity = Vec3::zero();
    let mut on_ground = false;
    let mut fly_mode = false;
    let mut current_eye_offset = STANDING_EYE_OFFSET;
    let mut current_player_height = STANDING_HEIGHT;

    let mut state = GameState::Playing;

    println!("Sterowanie:");
    println!("  Kliknij w okno - zablokuj/przechwyc mysz");
    println!("  Esc            - Menu pauzy / zwolnij mysz (w pauzie: powrot do gry)");
    println!("  WASD           - ruch (z plynna inercja/akceleracja)");
    println!("  Spacja         - skok (w trybie lotu: w gore)");
    println!("  Shift (lewy)   - kucanie (w trybie lotu: w dol)");
    println!("  Ctrl (lewy)    - bieg (z plynna zmiana FOV)");
    println!("  F              - przelacz tryb lotu (bez kolizji/grawitacji)");
    println!("  Mysz           - rozgladanie sie (gdy przechwycona)");
    println!("  LPM            - niszcz blok / kliknij przycisk w menu");
    println!("  PPM            - stawiaj blok");
    println!("  Scroll / 1-6   - wybor typu bloku do stawiania");
    println!("  Q              - wyjscie z gry w menu pauzy");

    while window.is_open() {
        let now = Instant::now();
        let dt = (now - last_time).as_secs_f32().min(0.1);
        last_time = now;

        // Dynamic window resizing / maximizing support
        let cur_size = window.get_size();
        if cur_size != win_size {
            win_size = cur_size;
            mouse_capture.resize(win_size.0 as i32, win_size.1 as i32);
        }

        let escape_down = window.is_key_down(Key::Escape);
        let f_down = window.is_key_down(Key::F);
        let q_down = window.is_key_down(Key::Q);
        let (win_w, win_h) = window.get_size();
        let mouse_pos = window.get_mouse_pos(MouseMode::Pass).map(|(mx, my)| {
            let fx = if win_w > 0 { mx * (WIDTH as f32) / (win_w as f32) } else { mx };
            let fy = if win_h > 0 { my * (HEIGHT as f32) / (win_h as f32) } else { my };
            (fx, fy)
        });
        let left_down = window.get_mouse_down(MouseButton::Left);
        let right_down = window.get_mouse_down(MouseButton::Right);

        match state {
            GameState::Playing => {
                // Escape opens the Pause Menu
                if escape_down && !escape_was_down {
                    state = GameState::Paused;
                    mouse_capture.set_captured(false);
                    window.set_cursor_visibility(true);
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                // Click to capture mouse when prompt is visible
                let any_click = left_down || right_down;
                if any_click && !mouse_capture.is_captured() {
                    mouse_capture.set_captured(true);
                    window.set_cursor_visibility(false);
                    last_mouse = None;
                    left_was_down = left_down;
                    right_was_down = right_down;
                }

                // Mouse look
                if mouse_capture.is_captured() {
                    if mouse_capture.has_native_capture() {
                        let (dx, dy) = mouse_capture.poll_delta();
                        camera.rotate(dx * 0.0025, -dy * 0.0025);
                    } else if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Pass) {
                        if let Some((lx, ly)) = last_mouse {
                            let dx = mx - lx;
                            let dy = my - ly;
                            camera.rotate(dx * 0.0025, -dy * 0.0025);
                        }
                        last_mouse = Some((mx, my));
                    }
                }

                // Direction vectors
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

                // Toggle fly mode with F
                if f_down && !fly_key_was_down {
                    fly_mode = !fly_mode;
                    horiz_velocity = Vec3::zero();
                    fly_velocity = Vec3::zero();
                }

                // Crouching state (in survival/normal mode)
                let is_crouching = !fly_mode
                    && mouse_capture.is_captured()
                    && (window.is_key_down(Key::LeftShift) || window.is_key_down(Key::C));

                // Smooth eye height & bounding box transition for crouching
                let target_eye = if is_crouching { CROUCH_EYE_OFFSET } else { STANDING_EYE_OFFSET };
                let target_h = if is_crouching { CROUCH_HEIGHT } else { STANDING_HEIGHT };
                let t_eye = 1.0 - (-14.0 * dt).exp();
                let old_eye = current_eye_offset;
                current_eye_offset += (target_eye - current_eye_offset) * t_eye;
                current_player_height += (target_h - current_player_height) * t_eye;

                if !fly_mode {
                    camera.position.y += current_eye_offset - old_eye;
                }

                // Sprinting state
                let is_sprinting = !is_crouching
                    && mouse_capture.is_captured()
                    && window.is_key_down(Key::LeftCtrl)
                    && wish_dir.length_sq() > 0.01;

                // Dynamic smooth FOV interpolation
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

                    // Smooth fly velocity interpolation
                    let target_fly = move_dir * (move_speed * 1.6);
                    let fly_damping = 12.0;
                    let t_fly = 1.0 - (-fly_damping * dt).exp();
                    fly_velocity = fly_velocity.lerp(target_fly, t_fly);

                    camera.position = camera.position + fly_velocity * dt;
                    vertical_velocity = 0.0;
                    horiz_velocity = Vec3::zero();
                } else {
                    // Smooth horizontal velocity interpolation (acceleration + friction)
                    let target_horiz = wish_dir * move_speed;
                    let damping = if on_ground {
                        if is_crouching { 18.0 } else { 14.0 }
                    } else {
                        4.5
                    };
                    let t_horiz = 1.0 - (-damping * dt).exp();
                    horiz_velocity = horiz_velocity.lerp(target_horiz, t_horiz);

                    // Gravity
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

                // Safety clamp
                if camera.position.y < -32.0 {
                    camera.position.y = 80.0;
                    vertical_velocity = 0.0;
                }
                if camera.position.y > 250.0 {
                    camera.position.y = 250.0;
                    vertical_velocity = vertical_velocity.min(0.0);
                }

                // Block selection via keys (1-9) and scroll wheel
                let mut selected_block = renderer.selected_block();
                if let Some((_sx, sy)) = window.get_scroll_wheel() {
                    if sy > 0.0 {
                        // Scroll up -> next slot (1..=9)
                        selected_block = if selected_block >= 9 { 1 } else { selected_block + 1 };
                    } else if sy < 0.0 {
                        // Scroll down -> previous slot
                        selected_block = if selected_block <= 1 { 9 } else { selected_block - 1 };
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
                ] {
                    if window.is_key_down(key) {
                        selected_block = id;
                    }
                }
                renderer.set_selected_block(selected_block);

                // Block interaction
                let interact_left = mouse_capture.is_captured() && left_down;
                let interact_right = mouse_capture.is_captured() && right_down;

                if interact_left && !left_was_down {
                    if let Some(hit) = world.raycast(camera.position, camera.forward(), 8.0) {
                        // Bedrock jest niezniszczalny
                        if !world.get_block(hit.block_pos).is_bedrock() {
                            world.set_block(hit.block_pos, block::BlockId::Air);
                        }
                    }
                }
                if interact_right && !right_was_down {
                    if let Some(hit) = world.raycast(camera.position, camera.forward(), 8.0) {
                        let place_pos = hit.block_pos + hit.normal;
                        // Prevent placing block inside player's bounding box
                        if !block_intersects_player(place_pos, camera.position, current_eye_offset, current_player_height) {
                            world.set_block(place_pos, block::BlockId::from_u8(selected_block));
                        }
                    }
                }

                // Update world
                world.update(camera.position);

                // Render world and in-game HUD
                renderer.render_frame(&world, &camera);
                renderer.draw_crosshair();
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

                let debug_lines = vec![
                    format!("FPS: {}", fps_display),
                    format!(
                        "POZYCJA X:{} Y:{} Z:{}",
                        camera.position.x as i32, camera.position.y as i32, camera.position.z as i32
                    ),
                    format!("BLOK: {}", block::BlockId::from_u8(selected_block).name().to_uppercase()),
                    format!("TRYB: {}", mode_str),
                ];
                renderer.draw_debug_panel(&debug_lines);

                if !mouse_capture.is_captured() {
                    renderer.draw_capture_prompt(mouse_capture.has_native_capture());
                }
            }
            GameState::Paused => {
                window.set_cursor_visibility(true);

                // Gently return FOV to standard when paused
                let t_fov = 1.0 - (-8.0 * dt).exp();
                camera.fov_y += (BASE_FOV_DEG.to_radians() - camera.fov_y) * t_fov;

                // Compute hovered button in pause menu using scaled mouse coordinates
                let rects = renderer.pause_menu_button_rects();
                let mut hovered_idx = None;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            break;
                        }
                    }
                }

                // Click interaction on buttons
                if left_down && !left_was_down {
                    match hovered_idx {
                        Some(0) => {
                            // Resume
                            state = GameState::Playing;
                            mouse_capture.set_captured(true);
                            window.set_cursor_visibility(false);
                            last_mouse = None;
                        }
                        Some(1) => {
                            // Toggle fly mode
                            fly_mode = !fly_mode;
                        }
                        Some(2) => {
                            // Quit game
                            break;
                        }
                        _ => {}
                    }
                }

                // Keyboard shortcuts in pause menu
                if escape_down && !escape_was_down {
                    // Resume
                    state = GameState::Playing;
                    mouse_capture.set_captured(true);
                    window.set_cursor_visibility(false);
                    last_mouse = None;
                }
                if f_down && !fly_key_was_down {
                    fly_mode = !fly_mode;
                }
                if q_down && !q_was_down {
                    break;
                }

                // Render current frame with pause menu overlay
                renderer.render_frame(&world, &camera);
                renderer.draw_pause_menu(fly_mode, hovered_idx);
            }
        }

        left_was_down = left_down;
        right_was_down = right_down;
        escape_was_down = escape_down;
        fly_key_was_down = f_down;
        q_was_down = q_down;

        window
            .update_with_buffer(renderer.pixel_buffer(), WIDTH, HEIGHT)
            .expect("Blad podczas aktualizacji bufora okna");

        // --- FPS counter in title ---
        frame_count += 1;
        if fps_timer.elapsed().as_secs_f32() >= 0.5 {
            fps_display = (frame_count as f32 / fps_timer.elapsed().as_secs_f32()) as u32;
            frame_count = 0;
            fps_timer = Instant::now();
        }
        window.set_title(&format!(
            "RustCraft [software renderer] - {} FPS - blok: {}{}{}",
            fps_display,
            block::BlockId::from_u8(renderer.selected_block()).name(),
            if fly_mode { " - [LOT]" } else { "" },
            if state == GameState::Paused { " - [PAUZA]" } else { "" }
        ));
    }
}

/// Checks if a placed block's 1x1x1 volume intersects the player's bounding box.
fn block_intersects_player(place_pos: crate::math::IVec3, eye_pos: Vec3, eye_offset: f32, height: f32) -> bool {
    let half = PLAYER_HALF_WIDTH;
    let feet_y = eye_pos.y - eye_offset;
    let p_min_x = eye_pos.x - half;
    let p_max_x = eye_pos.x + half;
    let p_min_y = feet_y;
    let p_max_y = feet_y + height;
    let p_min_z = eye_pos.z - half;
    let p_max_z = eye_pos.z + half;

    let b_min_x = place_pos.x as f32;
    let b_max_x = (place_pos.x + 1) as f32;
    let b_min_y = place_pos.y as f32;
    let b_max_y = (place_pos.y + 1) as f32;
    let b_min_z = place_pos.z as f32;
    let b_max_z = (place_pos.z + 1) as f32;

    p_min_x < b_max_x
        && p_max_x > b_min_x
        && p_min_y < b_max_y
        && p_max_y > b_min_y
        && p_min_z < b_max_z
        && p_max_z > b_min_z
}

/// Checks if there is solid ground beneath the player's feet box.
fn has_ground_below(world: &World, feet: Vec3, half: f32) -> bool {
    let min = Vec3::new(feet.x - half, feet.y - 0.4, feet.z - half);
    let max = Vec3::new(feet.x + half, feet.y + 0.05, feet.z + half);
    world.box_collides(min, max)
}

/// Moves the player (whose `eye_pos` is the camera position, i.e. feet +
/// `player_eye_offset`) by `motion`, resolving collisions against solid
/// blocks one axis at a time (X, then Y, then Z) so the player slides along
/// walls/floors instead of just stopping dead on any contact. Returns
/// whether the player ended the move standing on solid ground.
fn move_player_with_collision(
    world: &World,
    eye_pos: &mut Vec3,
    motion: Vec3,
    vertical_velocity: &mut f32,
    player_eye_offset: f32,
    player_height: f32,
    is_crouching: bool,
) -> bool {
    let mut feet = Vec3::new(eye_pos.x, eye_pos.y - player_eye_offset, eye_pos.z);

    let half = PLAYER_HALF_WIDTH;
    let height = player_height;

    let aabb = |feet: Vec3| -> (Vec3, Vec3) {
        (
            Vec3::new(feet.x - half, feet.y, feet.z - half),
            Vec3::new(feet.x + half, feet.y + height, feet.z + half),
        )
    };

    // Move on X axis
    feet.x += motion.x;
    if world.box_collides(aabb(feet).0, aabb(feet).1)
        || (is_crouching && !has_ground_below(world, feet, half))
    {
        feet.x -= motion.x;
    }

    // Move on Z axis
    feet.z += motion.z;
    if world.box_collides(aabb(feet).0, aabb(feet).1)
        || (is_crouching && !has_ground_below(world, feet, half))
    {
        feet.z -= motion.z;
    }

    // Move on Y axis (gravity / jumping), tracked separately so we can
    // detect ground contact and zero out vertical velocity on landing.
    let mut on_ground = false;
    feet.y += motion.y;
    {
        let (min, max) = aabb(feet);
        if world.box_collides(min, max) {
            feet.y -= motion.y;
            if motion.y < 0.0 {
                on_ground = true;
            }
            *vertical_velocity = 0.0;
        }
    }

    eye_pos.x = feet.x;
    eye_pos.y = feet.y + player_eye_offset;
    eye_pos.z = feet.z;

    on_ground
}
