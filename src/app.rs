//! Main RustCraft application state machine, game loop coordinator, and UI controller.
//!
//! Orchestrates player movement, voxel interaction, mob AI, particle simulation,
//! audio cues, inventory management, rendering backend dispatches, and menus.

use std::fmt::Write;
#[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
use std::sync::mpsc;
#[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
use std::thread;

use crate::cli::generate_random_seed;
use crate::core::math::{IVec3, Mat4, Vec3};
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
use crate::ui::{InventoryLayout, InventorySlotTarget, VISIBLE_SAVE_ROWS};
use crate::world::block::{BlockId, Face};
use crate::world::storage::{delete_save, get_save_metadata, save_current_world, save_path, SaveMetadata};
use crate::world::World;

/// Default framebuffer render width.
pub const WIDTH: usize = 960;

/// Default framebuffer render height.
pub const HEIGHT: usize = 540;

/// Maximum number of selectable world save slots.
pub const MAX_SAVE_SLOTS: u8 = 32;

/// Originating menu state for submenus (e.g. Settings).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PreviousMenu {
    /// Settings opened from the main menu.
    MainMenu,
    /// Settings opened from the in-game pause menu.
    Paused,
}

/// Result of asynchronous background world loading.
pub enum WorldLoadResult {
    /// Successfully loaded world instance and optional player position.
    Success(World, Option<Vec3>),
    /// Error description if world loading failed.
    Error(String),
}

/// Active top-level game state.
pub enum GameState {
    /// Main splash / start screen.
    MainMenu,
    /// World save slot carousel / selector.
    SaveSelect,
    /// New world creation dialogue.
    CreateWorld,
    /// Video / graphics settings menu.
    Settings(PreviousMenu),
    /// Asynchronous world generation / loading screen.
    #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
    Loading(mpsc::Receiver<WorldLoadResult>),
    /// Synchronous world loading state for embedded platforms.
    #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
    Loading,
    /// Active 3D gameplay.
    Playing,
    /// In-game pause menu overlay.
    Paused,
    /// Player inventory & crafting screen.
    Inventory,
    /// Player death screen with respawn and quit options.
    DeathScreen,
}

/// Unified input snapshot passed into `RustCraftApp::update_and_render`.
#[derive(Clone, Debug, Default)]
pub struct InputState {
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub mouse_pos: Option<(f32, f32)>,
    pub mouse_left: bool,
    pub mouse_right: bool,
    pub scroll_delta: f32,

    pub key_w: bool,
    pub key_s: bool,
    pub key_a: bool,
    pub key_d: bool,
    pub key_space: bool,
    pub key_shift: bool,
    pub key_ctrl: bool,
    pub key_c: bool,
    pub key_v: bool,
    pub key_x: bool,
    pub key_escape: bool,
    pub key_p: bool,
    pub key_f: bool,
    pub key_e: bool,
    pub key_q: bool,
    pub key_g: bool,
    pub key_m: bool,
    pub key_f5: bool,
    pub key_enter: bool,
    pub key_up: bool,
    pub key_down: bool,
    pub key_left: bool,
    pub key_right: bool,
    pub key_n: bool,
    pub key_del: bool,
    pub key_backspace: bool,
    pub key_tab: bool,
    pub key_digits: [bool; 10],
    pub typed_chars: Vec<char>,

    // Gamepad (analog axes & buttons)
    pub gp_left_stick_x: f32,
    pub gp_left_stick_y: f32,
    pub gp_right_stick_x: f32,
    pub gp_right_stick_y: f32,
    pub gp_left_trigger: f32,
    pub gp_right_trigger: f32,
    pub gp_btn_south: bool,
    pub gp_btn_east: bool,
    pub gp_btn_west: bool,
    pub gp_btn_north: bool,
    pub gp_btn_lb: bool,
    pub gp_btn_rb: bool,
    pub gp_btn_l3: bool,
    pub gp_btn_r3: bool,
    pub gp_btn_start: bool,
    pub gp_btn_select: bool,
    pub gp_dpad_up: bool,
    pub gp_dpad_down: bool,
    pub gp_dpad_left: bool,
    pub gp_dpad_right: bool,

    // One-shot edge events
    pub gp_south_just_pressed: bool,
    pub gp_east_just_pressed: bool,
    pub gp_west_just_pressed: bool,
    pub gp_north_just_pressed: bool,
    pub gp_lb_just_pressed: bool,
    pub gp_rb_just_pressed: bool,
    pub gp_r3_just_pressed: bool,
    pub gp_start_just_pressed: bool,
    pub gp_select_just_pressed: bool,
    pub gp_dpad_up_just_pressed: bool,
    pub gp_dpad_down_just_pressed: bool,
    pub gp_dpad_left_just_pressed: bool,
    pub gp_dpad_right_just_pressed: bool,
    pub gp_right_trigger_just_pressed: bool,
    pub gp_left_trigger_just_pressed: bool,

    pub is_mouse_captured: bool,
}

pub struct AppOutput {
    pub request_capture_mouse: bool,
    pub request_release_mouse: bool,
    pub quit_requested: bool,
    pub is_playing: bool,
    pub title: String,
}

pub struct RustCraftApp {
    pub width: usize,
    pub height: usize,
    pub renderer: Renderer,
    pub world: World,
    pub camera: Camera,
    pub particles: ParticleSystem,
    pub mob_manager: MobManager,
    pub player: Player,
    pub items: Vec<ItemEntity>,
    pub recipe_book: RecipeBook,
    pub crafting_table_open: bool,
    pub settings: GameSettings,
    pub state: GameState,
    pub save_slot: u8,
    pub save_select_slot: u8,
    pub save_select_scroll: usize,
    pub save_metadata_cache: [SaveMetadata; MAX_SAVE_SLOTS as usize],
    pub settings_hovered_opt: Option<usize>,
    pub menu_hovered_btn: Option<usize>,
    pub pause_hovered_btn: Option<usize>,
    pub death_hovered_btn: Option<usize>,
    pub vertical_velocity: f32,
    pub horiz_velocity: Vec3,
    pub fly_velocity: Vec3,
    pub on_ground: bool,
    pub fly_mode: bool,
    pub current_eye_offset: f32,
    pub current_player_height: f32,
    pub break_progress: f32,
    pub break_target: Option<IVec3>,
    pub total_time: f32,
    pub material_timer: f32,
    pub hand_animation: f32,
    pub walk_phase: f32,
    pub damage_tilt_timer: f32,
    pub damage_tilt_dir: f32,
    pub lava_damage_timer: f32,
    pub drown_damage_timer: f32,
    pub debug_lines: Vec<String>,
    pub debug_timer: f32,
    pub cpu_time_ms: f32,
    pub gpu_triangles: u32,
    pub cdram_mb: f32,
    pub ram_mb: f32,
    pub chunk_slots_info: (usize, usize),
    pub last_fov: f32,
    pub last_aspect: f32,
    pub proj_matrix: Mat4,
    pub main_menu_rects: [(i32, i32, i32, i32); 3],
    pub save_card_rects: [(i32, i32, i32, i32); VISIBLE_SAVE_ROWS],
    pub save_btn_rects: [(i32, i32, i32, i32); 4],
    pub create_world_rects: [(i32, i32, i32, i32); 6],
    pub settings_rects: [(i32, i32, i32, i32); 13],
    pub pause_menu_rects: [(i32, i32, i32, i32); 5],
    pub death_screen_rects: [(i32, i32, i32, i32); 2],
    pub create_world_name: String,
    pub create_world_seed: String,
    pub create_world_gamemode: GameMode,
    pub create_world_focus: usize,
    pub create_world_slot: u8,
    pub create_world_selected: bool,
    pub last_clean_thumbnail: Option<Vec<u32>>,
    pub thumbnail_timer: f32,
    pub seed: u32,
    pub fps_display: u32,
    pub frozen_frame: Option<Vec<u32>>,
    pub right_drag_visited: Vec<crate::ui::inventory_ui::InventorySlotTarget>,

    // Previous input state for edge detection
    prev_left_down: bool,
    prev_right_down: bool,
    prev_escape_down: bool,
    prev_p_down: bool,
    prev_e_down: bool,
    prev_f_down: bool,
    prev_q_down: bool,
    prev_m_down: bool,
    prev_g_down: bool,
    prev_f5_down: bool,
    prev_enter_down: bool,
    prev_up_arrow_down: bool,
    prev_down_arrow_down: bool,
    prev_left_arrow_down: bool,
    prev_right_arrow_down: bool,
    prev_n_down: bool,
    prev_del_down: bool,
    prev_backspace_down: bool,
    prev_tab_down: bool,
    prev_a_down: bool,
    prev_c_down: bool,
    prev_v_down: bool,
    prev_x_down: bool,
    prev_key_digits: [bool; 10],
}

impl RustCraftApp {
    pub fn new(width: usize, height: usize, renderer_choice: &str, seed: u32) -> Self {
        let settings = GameSettings::load();
        crate::core::i18n::set_language(settings.language);
        let mut renderer = match renderer_choice {
            #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita")))]
            "vulkan" => Renderer::new_vulkan(width, height).unwrap_or_else(|error| {
                eprintln!("Failed to initialize Vulkan renderer: {error}, falling back to software renderer");
                Renderer::new(width, height)
            }),
            _ => Renderer::new(width, height),
        };
        renderer.fog_enabled = settings.fog;
        renderer.render_distance = settings.render_distance;
        if !settings.fancy_leaves {
            renderer.update_textures(false);
        }

        let world = World::new_with_seed(seed);
        let camera = Camera::new(world.player_position);

        let mut player = Player::new();
        player.gamemode = GameMode::Survival;

        let main_menu_rects = renderer.main_menu_button_rects();
        let save_card_rects = renderer.save_card_rects();
        let save_btn_rects = renderer.save_bottom_button_rects();
        let create_world_rects = renderer.create_world_rects();
        let settings_rects = renderer.settings_option_rects();
        let pause_menu_rects = renderer.pause_menu_button_rects();
        let death_screen_rects = renderer.death_screen_button_rects();

        let aspect = width as f32 / height as f32;
        let last_fov = camera.fov_y;
        let proj_matrix = camera.projection_matrix(aspect);

        let save_metadata_cache = std::array::from_fn(|index| {
            get_save_metadata(index as u8 + 1)
        });

        Self {
            width,
            height,
            renderer,
            world,
            camera,
            particles: ParticleSystem::new(),
            mob_manager: MobManager::new(),
            player,
            items: Vec::new(),
            recipe_book: RecipeBook::new(),
            crafting_table_open: false,
            settings,
            state: GameState::MainMenu,
            save_slot: 1,
            save_select_slot: 1,
            save_select_scroll: 0,
            save_metadata_cache,
            settings_hovered_opt: Some(0),
            menu_hovered_btn: Some(0),
            pause_hovered_btn: Some(0),
            death_hovered_btn: Some(0),
            vertical_velocity: 0.0,
            horiz_velocity: Vec3::zero(),
            fly_velocity: Vec3::zero(),
            on_ground: false,
            fly_mode: false,
            current_eye_offset: STANDING_EYE_OFFSET,
            current_player_height: STANDING_HEIGHT,
            break_progress: 0.0,
            break_target: None,
            total_time: 0.0,
            material_timer: 0.0,
            hand_animation: 0.0,
            walk_phase: 0.0,
            damage_tilt_timer: 0.0,
            damage_tilt_dir: 1.0,
            lava_damage_timer: 0.45,
            drown_damage_timer: 0.0,
            debug_lines: vec![
                String::with_capacity(64),
                String::with_capacity(64),
                String::with_capacity(64),
                String::with_capacity(64),
                String::with_capacity(64),
                String::with_capacity(64),
                String::with_capacity(64),
                String::with_capacity(64),
            ],
            debug_timer: 0.25,
            cpu_time_ms: 16.6,
            gpu_triangles: 0,
            cdram_mb: 0.0,
            ram_mb: 0.0,
            chunk_slots_info: (0, 144),
            last_fov,
            last_aspect: aspect,
            proj_matrix,
            main_menu_rects,
            save_card_rects,
            save_btn_rects,
            create_world_rects,
            settings_rects,
            pause_menu_rects,
            death_screen_rects,
            create_world_name: String::new(),
            create_world_seed: String::new(),
            create_world_gamemode: GameMode::Survival,
            create_world_focus: 0,
            create_world_slot: 1,
            create_world_selected: false,
            last_clean_thumbnail: None,
            thumbnail_timer: 0.0,
            seed,
            fps_display: 60,
            frozen_frame: None,
            right_drag_visited: Vec::new(),

            prev_left_down: false,
            prev_right_down: false,
            prev_escape_down: false,
            prev_p_down: false,
            prev_e_down: false,
            prev_f_down: false,
            prev_q_down: false,
            prev_m_down: false,
            prev_g_down: false,
            prev_f5_down: false,
            prev_enter_down: false,
            prev_up_arrow_down: false,
            prev_down_arrow_down: false,
            prev_left_arrow_down: false,
            prev_right_arrow_down: false,
            prev_n_down: false,
            prev_del_down: false,
            prev_backspace_down: false,
            prev_tab_down: false,
            prev_a_down: false,
            prev_c_down: false,
            prev_v_down: false,
            prev_x_down: false,
            prev_key_digits: [false; 10],
        }
    }

    pub fn pixel_buffer(&self) -> &[u32] {
        self.renderer.pixel_buffer()
    }

    /// Alias for PS Vita – full update of game logic and software state.
    /// On Vita the software framebuffer draw is bypassed in favor of native GXM GPU rendering.
    #[cfg(target_os = "vita")]
    pub fn update(&mut self, dt: f32, input: &crate::app::InputState) -> AppOutput {
        self.update_and_render(dt, input)
    }

    pub fn update_and_render(&mut self, dt: f32, input: &InputState) -> AppOutput {
        self.total_time += dt;
        self.renderer.total_time = self.total_time;
        self.renderer.chunk_anim = self.settings.chunk_anim;
        self.hand_animation = (self.hand_animation - dt * 3.5).max(0.0);

        let mut out = AppOutput {
            request_capture_mouse: false,
            request_release_mouse: false,
            quit_requested: false,
            is_playing: matches!(self.state, GameState::Playing),
            title: String::new(),
        };

        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let right_down = input.mouse_right;
        let escape_down = input.key_escape;
        let p_down = input.key_p;
        let f_down = input.key_f;
        let e_down = input.key_e;
        let q_down = input.key_q;
        let m_down = input.key_m;
        let g_down = input.key_g;
        let a_down = input.key_a;
        let c_down = input.key_c;
        let v_down = input.key_v;
        let x_down = input.key_x;
        let ctrl_down = input.key_ctrl;
        let f5_down = input.key_f5;
        let enter_down = input.key_enter;
        let n_down = input.key_n;
        let del_down = input.key_del;
        let backspace_down = input.key_backspace;
        let tab_down = input.key_tab;
        let left_arrow_down = input.key_left;
        let right_arrow_down = input.key_right;
        let up_arrow_down = input.key_up;
        let down_arrow_down = input.key_down;


        match self.state {
            GameState::MainMenu => {
                let mut hovered_idx = self.menu_hovered_btn;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in self.main_menu_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            self.menu_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.menu_hovered_btn {
                        Some(0) | None => 2,
                        Some(i) => i - 1,
                    };
                    self.menu_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.menu_hovered_btn {
                        Some(2) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.menu_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_button = if (left_down && !self.prev_left_down) || input.gp_south_just_pressed {
                    hovered_idx
                } else {
                    None
                };

                if (enter_down && !self.prev_enter_down) || click_button == Some(0) {
                    self.state = GameState::SaveSelect;
                    self.save_select_slot = self.save_slot;
                    self.save_select_scroll = ((self.save_slot as usize - 1).saturating_sub(1))
                        .min(MAX_SAVE_SLOTS as usize - VISIBLE_SAVE_ROWS);
                } else if click_button == Some(1) {
                    self.state = GameState::Settings(PreviousMenu::MainMenu);
                    self.settings_hovered_opt = Some(0);
                } else if (q_down && !self.prev_q_down) || click_button == Some(2) || input.gp_east_just_pressed {
                    out.quit_requested = true;
                }

                self.renderer.draw_main_menu(hovered_idx);
            }
            GameState::SaveSelect => {
                if input.scroll_delta > 0.0 || input.gp_lb_just_pressed {
                    self.save_select_scroll = self.save_select_scroll.saturating_sub(VISIBLE_SAVE_ROWS);
                    self.save_select_slot = (self.save_select_scroll + 1) as u8;
                } else if input.scroll_delta < 0.0 || input.gp_rb_just_pressed {
                    let max_scroll = (MAX_SAVE_SLOTS as usize).saturating_sub(VISIBLE_SAVE_ROWS);
                    self.save_select_scroll = (self.save_select_scroll + VISIBLE_SAVE_ROWS).min(max_scroll);
                    self.save_select_slot = (self.save_select_scroll + 1) as u8;
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    self.save_select_slot = if self.save_select_slot <= 1 { MAX_SAVE_SLOTS } else { self.save_select_slot - 1 };
                    let slot_idx = self.save_select_slot as usize - 1;
                    if slot_idx < self.save_select_scroll {
                        self.save_select_scroll = slot_idx;
                    } else if slot_idx >= self.save_select_scroll + VISIBLE_SAVE_ROWS {
                        self.save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
                    }
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    self.save_select_slot = if self.save_select_slot >= MAX_SAVE_SLOTS { 1 } else { self.save_select_slot + 1 };
                    let slot_idx = self.save_select_slot as usize - 1;
                    if slot_idx >= self.save_select_scroll + VISIBLE_SAVE_ROWS {
                        self.save_select_scroll = slot_idx - VISIBLE_SAVE_ROWS + 1;
                    } else if slot_idx < self.save_select_scroll {
                        self.save_select_scroll = slot_idx;
                    }
                }

                let mut save_select_hovered_card = None;
                let mut save_select_hovered_btn = None;

                let mut card_launch_action = false;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;

                    // Save cards touch interaction
                    for (row, &(bx, by, bw, bh)) in self.save_card_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            save_select_hovered_card = Some(row);
                            if left_down && !self.prev_left_down {
                                let target_slot = (self.save_select_scroll + row + 1) as u8;
                                if self.save_select_slot == target_slot {
                                    // Touching an already selected slot launches the world
                                    card_launch_action = true;
                                } else {
                                    self.save_select_slot = target_slot;
                                }
                            }
                            break;
                        }
                    }

                    // Scrollbar touch and drag
                    let cx = self.width as i32 / 2;
                    let sb_x = cx + 340;
                    let sb_w = 40; // enlarged touch target for scrollbar
                    let sb_y = 70;
                    let sb_h = (VISIBLE_SAVE_ROWS as i32) * 76;
                    if px >= sb_x && px <= sb_x + sb_w && py >= sb_y && py <= sb_y + sb_h && left_down {
                        let rel_y = ((py - sb_y) as f32 / sb_h as f32).clamp(0.0, 1.0);
                        let max_scroll = (MAX_SAVE_SLOTS as usize).saturating_sub(VISIBLE_SAVE_ROWS);
                        self.save_select_scroll = (rel_y * max_scroll as f32).round() as usize;
                    }

                    // Bottom buttons
                    for (i, &(bx, by, bw, bh)) in self.save_btn_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            save_select_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                let play_action = (enter_down && !self.prev_enter_down)
                    || (left_down && !self.prev_left_down && save_select_hovered_btn == Some(0))
                    || card_launch_action
                    || input.gp_south_just_pressed;
                let new_action = (n_down && !self.prev_n_down)
                    || (left_down && !self.prev_left_down && save_select_hovered_btn == Some(1))
                    || input.gp_north_just_pressed;
                let delete_action = (del_down && !self.prev_del_down)
                    || (left_down && !self.prev_left_down && save_select_hovered_btn == Some(2))
                    || input.gp_west_just_pressed;
                let back_action = (escape_down && !self.prev_escape_down)
                    || (left_down && !self.prev_left_down && save_select_hovered_btn == Some(3))
                    || input.gp_east_just_pressed;

                if play_action {
                    self.save_slot = self.save_select_slot;
                    let meta = &self.save_metadata_cache[self.save_slot as usize - 1];
                    if meta.exists {
                        let path = save_path(self.save_slot);
                        #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
                        {
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
                            self.state = GameState::Loading(rx);
                        }
                        #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
                        {
                            match World::load(&path) {
                                Ok(loaded_world) => {
                                    let pos = loaded_world.player_position;
                                    self.player.gamemode = loaded_world.gamemode;
                                    self.world = loaded_world;
                                    self.camera = Camera::new(pos);
                                    self.mob_manager = MobManager::new();
                                    self.state = GameState::Playing;
                                }
                                Err(_) => {
                                    self.world = World::new_with_seed(self.seed);
                                    self.camera = Camera::new(self.world.player_position);
                                    self.mob_manager = MobManager::new();
                                    self.state = GameState::Playing;
                                }
                            }
                        }
                    } else {
                        // Open world creation menu for empty slot
                        self.create_world_slot = self.save_select_slot;
                        self.create_world_name = format!("New World {:02}", self.save_select_slot);
                        self.create_world_seed = String::new();
                        self.create_world_gamemode = GameMode::Survival;
                        self.create_world_focus = 0;
                        self.create_world_selected = false;
                        self.state = GameState::CreateWorld;
                    }
                } else if new_action {
                    // Open world creation menu for selected slot
                    self.create_world_slot = self.save_select_slot;
                    self.create_world_name = format!("New World {:02}", self.save_select_slot);
                    self.create_world_seed = String::new();
                    self.create_world_gamemode = GameMode::Survival;
                    self.create_world_focus = 0;
                    self.create_world_selected = false;
                    self.state = GameState::CreateWorld;
                } else if delete_action {
                    let _ = delete_save(self.save_select_slot);
                    self.save_metadata_cache[self.save_select_slot as usize - 1] = get_save_metadata(self.save_select_slot);
                } else if back_action {
                    self.state = GameState::MainMenu;
                }

                if matches!(self.state, GameState::SaveSelect) {
                    self.renderer.draw_save_select_menu(
                        self.save_select_slot,
                        self.save_select_scroll,
                        &self.save_metadata_cache,
                        save_select_hovered_card,
                        save_select_hovered_btn,
                    );
                }
            }
            GameState::CreateWorld => {
                let mut hovered_elem = None;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in self.create_world_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_elem = Some(i);
                            if left_down && !self.prev_left_down {
                                if self.create_world_focus != i {
                                    self.create_world_selected = false;
                                }
                                self.create_world_focus = i;
                            }
                            break;
                        }
                    }
                }

                let tab_just = tab_down && !self.prev_tab_down;
                let up_just = (up_arrow_down && !self.prev_up_arrow_down) || input.gp_dpad_up_just_pressed;
                let down_just = (down_arrow_down && !self.prev_down_arrow_down) || input.gp_dpad_down_just_pressed;
                let left_just = (left_arrow_down && !self.prev_left_arrow_down) || input.gp_dpad_left_just_pressed;
                let right_just = (right_arrow_down && !self.prev_right_arrow_down) || input.gp_dpad_right_just_pressed;

                if tab_just || down_just {
                    self.create_world_focus = (self.create_world_focus + 1) % 6;
                    self.create_world_selected = false;
                } else if up_just {
                    self.create_world_focus = if self.create_world_focus == 0 { 5 } else { self.create_world_focus - 1 };
                    self.create_world_selected = false;
                } else if left_just && self.create_world_focus == 5 {
                    self.create_world_focus = 4;
                    self.create_world_selected = false;
                } else if right_just && self.create_world_focus == 4 {
                    self.create_world_focus = 5;
                    self.create_world_selected = false;
                }

                // Keyboard shortcuts: Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X
                let a_just = a_down && !self.prev_a_down;
                let c_just = c_down && !self.prev_c_down;
                let v_just = v_down && !self.prev_v_down;
                let x_just = x_down && !self.prev_x_down;

                if ctrl_down {
                    if a_just {
                        if self.create_world_focus == 0 && !self.create_world_name.is_empty() {
                            self.create_world_selected = true;
                        } else if self.create_world_focus == 1 && !self.create_world_seed.is_empty() {
                            self.create_world_selected = true;
                        }
                    } else if c_just {
                        if self.create_world_focus == 0 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_name);
                        } else if self.create_world_focus == 1 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_seed);
                        }
                    } else if x_just {
                        if self.create_world_focus == 0 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_name);
                            self.create_world_name.clear();
                            self.create_world_selected = false;
                        } else if self.create_world_focus == 1 {
                            crate::ui::clipboard::set_clipboard(&self.create_world_seed);
                            self.create_world_seed.clear();
                            self.create_world_selected = false;
                        }
                    } else if v_just {
                        let clip = crate::ui::clipboard::get_clipboard();
                        let filtered: String = clip.chars().filter(|c| !c.is_control()).collect();
                        if self.create_world_focus == 0 {
                            if self.create_world_selected {
                                self.create_world_name.clear();
                                self.create_world_selected = false;
                            }
                            for ch in filtered.chars() {
                                if self.create_world_name.len() < 24 {
                                    self.create_world_name.push(ch);
                                }
                            }
                        } else if self.create_world_focus == 1 {
                            if self.create_world_selected {
                                self.create_world_seed.clear();
                                self.create_world_selected = false;
                            }
                            for ch in filtered.chars() {
                                if self.create_world_seed.len() < 16 {
                                    self.create_world_seed.push(ch);
                                }
                            }
                        }
                    }
                }

                // Append typed characters to active text input field
                if !ctrl_down {
                    for &ch in &input.typed_chars {
                        if !ch.is_control() {
                            if self.create_world_focus == 0 {
                                if self.create_world_selected {
                                    self.create_world_name.clear();
                                    self.create_world_selected = false;
                                }
                                if self.create_world_name.len() < 24 {
                                    self.create_world_name.push(ch);
                                }
                            } else if self.create_world_focus == 1 {
                                if self.create_world_selected {
                                    self.create_world_seed.clear();
                                    self.create_world_selected = false;
                                }
                                if self.create_world_seed.len() < 16 {
                                    self.create_world_seed.push(ch);
                                }
                            }
                        }
                    }
                }

                let backspace_just = (backspace_down && !self.prev_backspace_down) || (del_down && !self.prev_del_down);
                if backspace_just {
                    if self.create_world_focus == 0 {
                        if self.create_world_selected {
                            self.create_world_name.clear();
                            self.create_world_selected = false;
                        } else {
                            self.create_world_name.pop();
                        }
                    } else if self.create_world_focus == 1 {
                        if self.create_world_selected {
                            self.create_world_seed.clear();
                            self.create_world_selected = false;
                        } else {
                            self.create_world_seed.pop();
                        }
                    }
                }

                if input.gp_north_just_pressed {
                    self.create_world_seed = format!("{}", generate_random_seed());
                    self.create_world_selected = false;
                }
                if input.gp_west_just_pressed {
                    self.create_world_gamemode = match self.create_world_gamemode {
                        GameMode::Survival => GameMode::Creative,
                        GameMode::Creative => GameMode::Survival,
                    };
                }

                let enter_just = enter_down && !self.prev_enter_down;
                let south_just = input.gp_south_just_pressed;
                let click_elem = if left_down && !self.prev_left_down { hovered_elem } else { None };

                let do_cancel = (escape_down && !self.prev_escape_down)
                    || input.gp_east_just_pressed
                    || click_elem == Some(5)
                    || (self.create_world_focus == 5 && (enter_just || south_just));

                let do_create = click_elem == Some(4)
                    || (enter_just && (self.create_world_focus == 0 || self.create_world_focus == 1 || self.create_world_focus == 4))
                    || (south_just && self.create_world_focus == 4);

                let do_rand_seed = (self.create_world_focus == 2 && (enter_just || south_just)) || click_elem == Some(2);
                let do_toggle_mode = (self.create_world_focus == 3 && (enter_just || south_just)) || click_elem == Some(3);

                if do_rand_seed {
                    self.create_world_seed = format!("{}", generate_random_seed());
                    self.create_world_selected = false;
                } else if do_toggle_mode {
                    self.create_world_gamemode = match self.create_world_gamemode {
                        GameMode::Survival => GameMode::Creative,
                        GameMode::Creative => GameMode::Survival,
                    };
                } else if do_cancel {
                    self.state = GameState::SaveSelect;
                } else if do_create {
                    let seed = if self.create_world_seed.trim().is_empty() {
                        generate_random_seed()
                    } else if let Ok(s) = self.create_world_seed.trim().parse::<u32>() {
                        s
                    } else {
                        let mut s = 0u32;
                        for b in self.create_world_seed.trim().bytes() {
                            s = s.wrapping_mul(31).wrapping_add(b as u32);
                        }
                        if s == 0 { generate_random_seed() } else { s }
                    };

                    let name = if self.create_world_name.trim().is_empty() {
                        format!("New World {:02}", self.create_world_slot)
                    } else {
                        self.create_world_name.trim().to_string()
                    };

                    let gamemode = self.create_world_gamemode;
                    self.save_slot = self.create_world_slot;
                    self.seed = seed;
                    self.player.gamemode = gamemode;

                    #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
                    {
                        let (tx, rx) = mpsc::channel();
                        let thread_seed = self.seed;
                        let thread_name = name;
                        let thread_gamemode = gamemode;
                        thread::spawn(move || {
                            let mut new_world = World::new_with_seed(thread_seed);
                            new_world.name = thread_name;
                            new_world.gamemode = thread_gamemode;
                            let _ = tx.send(WorldLoadResult::Success(new_world, None));
                        });
                        self.state = GameState::Loading(rx);
                    }
                    #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
                    {
                        self.world = World::new_with_seed(self.seed);
                        self.world.name = name;
                        self.world.gamemode = gamemode;
                        self.camera = Camera::new(self.world.player_position);
                        self.mob_manager = MobManager::new();
                        self.state = GameState::Playing;
                    }
                }

                if matches!(self.state, GameState::CreateWorld) {
                    self.renderer.draw_create_world_menu(
                        self.create_world_slot,
                        &self.create_world_name,
                        &self.create_world_seed,
                        self.create_world_gamemode,
                        self.create_world_focus,
                        hovered_elem,
                        self.total_time,
                        self.create_world_selected,
                    );
                }
            }
            GameState::Settings(prev) => {
                let mut hovered_opt = self.settings_hovered_opt;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in self.settings_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_opt = Some(i);
                            self.settings_hovered_opt = Some(i);
                            break;
                        }
                    }
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.settings_hovered_opt {
                        Some(0) | None => 12,
                        Some(i) => i - 1,
                    };
                    self.settings_hovered_opt = Some(next);
                    hovered_opt = Some(next);
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.settings_hovered_opt {
                        Some(12) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.settings_hovered_opt = Some(next);
                    hovered_opt = Some(next);
                }

                let click_opt = if (left_down && !self.prev_left_down) || input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
                    hovered_opt
                } else {
                    None
                };

                let left_key_just = left_arrow_down && !self.prev_left_arrow_down;
                let right_key_just = right_arrow_down && !self.prev_right_arrow_down;

                if let Some(opt_idx) = click_opt {
                    match opt_idx {
                        0 => {
                            self.settings.cycle_language(true);
                            crate::core::i18n::set_language(self.settings.language);
                        }
                        1 => {
                            self.settings.cycle_render_distance(true);
                            self.renderer.render_distance = self.settings.render_distance;
                            self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                            self.renderer.clear_mesh_cache();
                        }
                        2 => {
                            self.settings.fancy_leaves = !self.settings.fancy_leaves;
                            self.renderer.update_textures(self.settings.fancy_leaves);
                            self.world.mark_all_chunks_dirty();
                            self.renderer.clear_mesh_cache();
                        }
                        3 => self.settings.fog = !self.settings.fog,
                        4 => self.settings.clouds = !self.settings.clouds,
                        5 => self.settings.particles = !self.settings.particles,
                        6 => self.settings.chunk_anim = !self.settings.chunk_anim,
                        7 => self.settings.cycle_fov(true),
                        8 => self.settings.cycle_msaa(true),
                        9 => self.settings.cycle_fps_limit(true),
                        10 => self.settings.underwater_fx = !self.settings.underwater_fx,
                        11 => self.settings.debug_overlay = !self.settings.debug_overlay,
                        12 => {
                            let _ = self.settings.save();
                            self.state = match prev {
                                PreviousMenu::MainMenu => GameState::MainMenu,
                                PreviousMenu::Paused => GameState::Paused,
                            };
                        }
                        _ => {}
                    }
                } else if input.gp_dpad_left_just_pressed || left_key_just {
                    if let Some(opt_idx) = hovered_opt {
                        match opt_idx {
                            0 => {
                                self.settings.cycle_language(false);
                                crate::core::i18n::set_language(self.settings.language);
                            }
                            1 => {
                                self.settings.cycle_render_distance(false);
                                self.renderer.render_distance = self.settings.render_distance;
                                self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                                self.renderer.clear_mesh_cache();
                            }
                            2 => {
                                self.settings.fancy_leaves = !self.settings.fancy_leaves;
                                self.renderer.update_textures(self.settings.fancy_leaves);
                                self.world.mark_all_chunks_dirty();
                                self.renderer.clear_mesh_cache();
                            }
                            3 => self.settings.fog = !self.settings.fog,
                            4 => self.settings.clouds = !self.settings.clouds,
                            5 => self.settings.particles = !self.settings.particles,
                            6 => self.settings.chunk_anim = !self.settings.chunk_anim,
                            7 => self.settings.cycle_fov(false),
                            8 => self.settings.cycle_msaa(false),
                            9 => self.settings.cycle_fps_limit(false),
                            10 => self.settings.underwater_fx = !self.settings.underwater_fx,
                            11 => self.settings.debug_overlay = !self.settings.debug_overlay,
                            _ => {}
                        }
                    }
                } else if input.gp_dpad_right_just_pressed || right_key_just {
                    if let Some(opt_idx) = hovered_opt {
                        match opt_idx {
                            0 => {
                                self.settings.cycle_language(true);
                                crate::core::i18n::set_language(self.settings.language);
                            }
                            1 => {
                                self.settings.cycle_render_distance(true);
                                self.renderer.render_distance = self.settings.render_distance;
                                self.world.purge_distant_chunks(self.camera.position, self.settings.render_distance);
                                self.renderer.clear_mesh_cache();
                            }
                            2 => {
                                self.settings.fancy_leaves = !self.settings.fancy_leaves;
                                self.renderer.update_textures(self.settings.fancy_leaves);
                                self.world.mark_all_chunks_dirty();
                                self.renderer.clear_mesh_cache();
                            }
                            3 => self.settings.fog = !self.settings.fog,
                            4 => self.settings.clouds = !self.settings.clouds,
                            5 => self.settings.particles = !self.settings.particles,
                            6 => self.settings.chunk_anim = !self.settings.chunk_anim,
                            7 => self.settings.cycle_fov(true),
                            8 => self.settings.cycle_msaa(true),
                            9 => self.settings.cycle_fps_limit(true),
                            10 => self.settings.underwater_fx = !self.settings.underwater_fx,
                            11 => self.settings.debug_overlay = !self.settings.debug_overlay,
                            _ => {}
                        }
                    }
                }

                if (escape_down && !self.prev_escape_down) || (p_down && !self.prev_p_down) || input.gp_east_just_pressed {
                    let _ = self.settings.save();
                    self.state = match prev {
                        PreviousMenu::MainMenu => GameState::MainMenu,
                        PreviousMenu::Paused => GameState::Paused,
                    };
                }

                self.renderer.fog_enabled = self.settings.fog;
                if prev == PreviousMenu::Paused {
                    #[cfg(not(target_os = "vita"))]
                    self.renderer.render_frame(&mut self.world, &self.camera);
                    #[cfg(target_os = "vita")]
                    {
                        self.renderer.color_buffer.fill(0x0C0C10);
                    }
                }
                self.renderer.draw_settings_menu(&self.settings, hovered_opt);
            }
            #[cfg(not(any(target_arch = "wasm32", target_os = "vita")))]
            GameState::Loading(ref rx) => {
                self.renderer.draw_main_menu(None);

                if let Ok(result) = rx.try_recv() {
                    match result {
                        WorldLoadResult::Success(loaded_world, loaded_pos) => {
                            self.world = loaded_world;
                            let start_pos = loaded_pos.unwrap_or(self.world.player_position);
                            self.camera = Camera::new(start_pos);
                            self.mob_manager = MobManager::new();
                            self.state = GameState::Playing;
                        }
                        WorldLoadResult::Error(e) => {
                            eprintln!("Failed to load world: {e}");
                            self.state = GameState::MainMenu;
                        }
                    }
                }
            }
            #[cfg(any(target_arch = "wasm32", target_os = "vita"))]
            GameState::Loading => {
                self.renderer.draw_main_menu(None);
                self.state = GameState::Playing;
            }
            GameState::Playing => {
                if f5_down && !self.prev_f5_down {
                    let path = save_path(self.save_slot);
                    match save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref()) {
                        Ok(()) => println!("Saved world to {path}"),
                        Err(error) => eprintln!("{error}"),
                    }
                    self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                }
                if (escape_down && !self.prev_escape_down) || (p_down && !self.prev_p_down) || input.gp_start_just_pressed {
                    self.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(self.renderer.pixel_buffer(), self.width, self.height));
                    self.state = GameState::Paused;
                    out.request_release_mouse = true;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                }

                if (e_down && !self.prev_e_down) || input.gp_north_just_pressed {
                    self.crafting_table_open = false;
                    self.state = GameState::Inventory;
                    out.request_release_mouse = true;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                }

                let any_click = left_down || right_down;
                if any_click && !input.is_mouse_captured {
                    out.request_capture_mouse = true;
                }

                if input.is_mouse_captured {
                    self.camera.rotate(input.mouse_dx * 0.0025, -input.mouse_dy * 0.0025);
                }

                let gp_look_speed_x = 3.2;
                let gp_look_speed_y = 2.8;
                if input.gp_right_stick_x.abs() > 0.0 || input.gp_right_stick_y.abs() > 0.0 {
                    self.camera.rotate(
                        input.gp_right_stick_x * gp_look_speed_x * dt,
                        input.gp_right_stick_y * gp_look_speed_y * dt,
                    );
                }

                let forward = self.camera.forward_flat();
                let right = self.camera.right_flat();
                let mut wish_dir = Vec3::zero();

                if input.is_mouse_captured {
                    if input.key_w { wish_dir = wish_dir + forward; }
                    if input.key_s { wish_dir = wish_dir - forward; }
                    if input.key_a { wish_dir = wish_dir - right; }
                    if input.key_d { wish_dir = wish_dir + right; }
                }
                if input.gp_left_stick_y.abs() > 0.0 || input.gp_left_stick_x.abs() > 0.0 {
                    wish_dir = wish_dir + forward * input.gp_left_stick_y + right * input.gp_left_stick_x;
                }
                if wish_dir.length_sq() > 0.0001 {
                    wish_dir = wish_dir.normalized();
                }

                if (f_down && !self.prev_f_down) || input.gp_r3_just_pressed {
                    self.fly_mode = !self.fly_mode;
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                }

                if (g_down && !self.prev_g_down) || input.gp_select_just_pressed {
                    self.player.gamemode = if self.player.gamemode == GameMode::Survival {
                        GameMode::Creative
                    } else {
                        GameMode::Survival
                    };
                }

                let is_crouching = !self.fly_mode
                    && ((input.is_mouse_captured && (input.key_shift || input.key_c)) || input.gp_btn_east);

                let target_eye = if is_crouching { CROUCH_EYE_OFFSET } else { STANDING_EYE_OFFSET };
                let target_h = if is_crouching { CROUCH_HEIGHT } else { STANDING_HEIGHT };
                let t_eye = 1.0 - (-14.0 * dt).exp();
                let old_eye = self.current_eye_offset;
                self.current_eye_offset += (target_eye - self.current_eye_offset) * t_eye;
                self.current_player_height += (target_h - self.current_player_height) * t_eye;

                if !self.fly_mode {
                    self.camera.position.y += self.current_eye_offset - old_eye;
                }

                let can_sprint = self.player.gamemode == GameMode::Creative || self.player.hunger > 6.0;
                let is_sprinting = !is_crouching
                    && can_sprint
                    && ((input.is_mouse_captured && input.key_ctrl) || input.gp_btn_l3 || input.gp_left_stick_y > 0.85)
                    && wish_dir.length_sq() > 0.01;

                let target_fov_deg = if is_sprinting {
                    self.settings.fov_deg + 15.0
                } else if is_crouching {
                    self.settings.fov_deg - 5.0
                } else {
                    self.settings.fov_deg
                };
                let target_fov = target_fov_deg.to_radians();
                let t_fov = 1.0 - (-8.0 * dt).exp();
                self.camera.fov_y += (target_fov - self.camera.fov_y) * t_fov;

                let move_speed = if is_crouching {
                    SNEAK_SPEED
                } else if is_sprinting {
                    SPRINT_SPEED
                } else {
                    WALK_SPEED
                };

                if self.fly_mode {
                    let mut move_dir = wish_dir;
                    if (input.is_mouse_captured && input.key_space) || input.gp_btn_south {
                        move_dir.y += 1.0;
                    }
                    if (input.is_mouse_captured && input.key_shift) || input.gp_btn_east {
                        move_dir.y -= 1.0;
                    }
                    if move_dir.length_sq() > 0.0001 {
                        move_dir = move_dir.normalized();
                    }

                    let fly_fast = (input.is_mouse_captured && input.key_ctrl) || input.gp_btn_l3;
                    let fly_mult = if fly_fast { 2.0 } else { 1.0 };
                    let target_fly = move_dir * (FLY_SPEED * fly_mult);
                    let fly_damping = 14.0;
                    let t_fly = 1.0 - (-fly_damping * dt).exp();
                    self.fly_velocity = self.fly_velocity.lerp(target_fly, t_fly);

                    self.camera.position = self.camera.position + self.fly_velocity * dt;
                    self.vertical_velocity = 0.0;
                    self.horiz_velocity = Vec3::zero();
                    self.player.head_underwater = self.world.get_block(self.camera.position.floor_to_ivec()) == BlockId::Water;
                    self.player.air = self.player.max_air;
                    self.drown_damage_timer = 0.0;
                } else {
                    let feet_pos = Vec3::new(
                        self.camera.position.x,
                        self.camera.position.y - self.current_eye_offset - 0.08,
                        self.camera.position.z,
                    );
                    let feet_block = self.world.get_block(feet_pos.floor_to_ivec());
                    let eye_block = self.world.get_block(self.camera.position.floor_to_ivec());
                    let in_water = feet_block == BlockId::Water || eye_block == BlockId::Water;
                    let in_lava = feet_block == BlockId::Lava || eye_block == BlockId::Lava;

                    let target_horiz = wish_dir * (if in_lava { SWIM_SPEED * 0.55 } else if in_water { SWIM_SPEED } else { move_speed });
                    let on_ice = feet_block == BlockId::Ice;
                    let damping = if in_lava {
                        12.0
                    } else if in_water {
                        8.0
                    } else if self.on_ground {
                        if on_ice { 2.0 } else if is_crouching { 18.0 } else { 15.0 }
                    } else {
                        2.0
                    };
                    let t_horiz = 1.0 - (-damping * dt).exp();
                    self.horiz_velocity = self.horiz_velocity.lerp(target_horiz, t_horiz);

                    if in_lava {
                        self.vertical_velocity -= 2.5 * dt;
                        if self.vertical_velocity < -1.8 {
                            self.vertical_velocity = -1.8;
                        }
                        if (input.key_space && input.is_mouse_captured) || input.gp_btn_south {
                            self.vertical_velocity = 2.2;
                        }
                    } else if in_water {
                        self.vertical_velocity -= 4.0 * dt;
                        if self.vertical_velocity < -2.5 {
                            self.vertical_velocity = -2.5;
                        }
                        if (input.key_space && input.is_mouse_captured) || input.gp_btn_south {
                            self.vertical_velocity = 3.2;
                        }
                    } else {
                        self.vertical_velocity -= GRAVITY * dt;
                        if self.vertical_velocity < -TERMINAL_VELOCITY {
                            self.vertical_velocity = -TERMINAL_VELOCITY;
                        }

                        if ((input.key_space && input.is_mouse_captured) || input.gp_btn_south) && self.on_ground {
                            self.vertical_velocity = JUMP_SPEED;
                            if is_sprinting && wish_dir.length_sq() > 0.01 {
                                self.horiz_velocity = self.horiz_velocity + wish_dir * SPRINT_JUMP_BOOST;
                                self.player.exhaustion += 0.2;
                            } else {
                                self.player.exhaustion += 0.05;
                            }
                        }
                    }

                    let old_y = self.camera.position.y;
                    let motion = Vec3::new(self.horiz_velocity.x, self.vertical_velocity, self.horiz_velocity.z) * dt;
                    self.on_ground = move_player_with_collision(
                        &self.world,
                        &mut self.camera.position,
                        motion,
                        &mut self.vertical_velocity,
                        self.current_eye_offset,
                        self.current_player_height,
                        is_crouching,
                    );
                    if self.vertical_velocity < 0.0 && !self.fly_mode && !in_water && !in_lava {
                        self.player.fall_distance += old_y - self.camera.position.y;
                    } else if self.vertical_velocity > 0.0 || self.fly_mode || in_water || in_lava {
                        self.player.fall_distance = 0.0;
                    }

                    // Lava damage (4.0 dmg = 2 hearts every 0.5s)
                    if in_lava && self.player.gamemode == GameMode::Survival {
                        self.lava_damage_timer += dt;
                        if self.lava_damage_timer >= 0.5 {
                            self.lava_damage_timer = 0.0;
                            if self.player.take_damage(4.0) {
                                self.damage_tilt_timer = 0.4;
                                self.damage_tilt_dir = 1.0;
                                self.particles.spawn_lava_burn(feet_pos);
                            }
                        }
                    } else if !in_lava {
                        self.lava_damage_timer = 0.45;
                    }

                    // Oxygen and underwater breathing mechanics
                    let is_head_underwater = eye_block == BlockId::Water;
                    self.player.head_underwater = is_head_underwater;

                    if self.player.gamemode == GameMode::Survival {
                        if is_head_underwater {
                            // 15 seconds to fully exhaust 20 oxygen points (~1.33 pt/s)
                            self.player.air -= (self.player.max_air / 15.0) * dt;
                            if self.player.air <= 0.0 {
                                self.player.air = 0.0;
                                self.drown_damage_timer += dt;
                                if self.drown_damage_timer >= 1.0 {
                                    self.drown_damage_timer = 0.0;
                                    if self.player.take_damage(2.0) {
                                        self.damage_tilt_timer = 0.4;
                                        self.damage_tilt_dir = 1.0;
                                        self.particles.spawn_drown_bubbles(self.camera.position);
                                    }
                                }
                            } else {
                                self.drown_damage_timer = 0.0;
                            }
                        } else {
                            self.drown_damage_timer = 0.0;
                            self.player.air = (self.player.air + 20.0 * dt).min(self.player.max_air);
                        }

                        // Hunger / Saturation / Exhaustion mechanics
                        if is_sprinting {
                            self.player.exhaustion += 0.18 * dt;
                        } else if wish_dir.length_sq() > 0.01 {
                            self.player.exhaustion += 0.04 * dt;
                        } else {
                            self.player.exhaustion += 0.01 * dt;
                        }

                        while self.player.exhaustion >= 4.0 {
                            self.player.exhaustion -= 4.0;
                            if self.player.saturation > 0.0 {
                                self.player.saturation = (self.player.saturation - 1.0).max(0.0);
                            } else {
                                self.player.hunger = (self.player.hunger - 1.0).max(0.0);
                            }
                        }

                        // Natural health regeneration (when hunger >= 18)
                        if self.player.hunger >= 18.0 && self.player.health < self.player.max_health {
                            self.player.heal_timer += dt;
                            if self.player.heal_timer >= 4.0 {
                                self.player.heal_timer = 0.0;
                                self.player.heal(1.0);
                                self.player.exhaustion += 4.0;
                            }
                        } else {
                            self.player.heal_timer = 0.0;
                        }

                        // Starvation damage (when hunger == 0)
                        if self.player.hunger <= 0.0 {
                            self.player.starvation_timer += dt;
                            if self.player.starvation_timer >= 4.0 {
                                self.player.starvation_timer = 0.0;
                                if self.player.take_damage(1.0) {
                                    self.damage_tilt_timer = 0.4;
                                    self.damage_tilt_dir = 1.0;
                                }
                            }
                        } else {
                            self.player.starvation_timer = 0.0;
                        }
                    } else {
                        self.player.air = self.player.max_air;
                        self.drown_damage_timer = 0.0;
                        self.player.hunger = self.player.max_hunger;
                        self.player.saturation = 5.0;
                        self.player.exhaustion = 0.0;
                        self.player.starvation_timer = 0.0;
                        self.player.heal_timer = 0.0;
                    }

                    if self.on_ground {
                        if !self.player.has_landed {
                            self.player.has_landed = true;
                            self.player.fall_distance = 0.0;
                        } else if self.player.fall_distance > 3.0 {
                            let damage = self.player.fall_distance - 3.0;
                            if self.player.take_damage(damage) {
                                self.damage_tilt_timer = 0.4;
                                self.damage_tilt_dir = 1.0;
                            }
                        }
                        self.player.fall_distance = 0.0;
                    }
                    if self.player.is_dead() {
                        self.state = GameState::DeathScreen;
                        out.request_release_mouse = true;
                        self.death_hovered_btn = Some(0);
                        self.horiz_velocity = Vec3::zero();
                        self.fly_velocity = Vec3::zero();
                    }
                }

                if self.player.invulnerability_timer > 0.0 {
                    self.player.invulnerability_timer -= dt;
                }
                if self.damage_tilt_timer > 0.0 {
                    self.damage_tilt_timer = (self.damage_tilt_timer - dt).max(0.0);
                }

                if self.camera.position.y < -32.0 {
                    if self.player.take_damage(20.0) {
                        self.damage_tilt_timer = 0.4;
                    }
                    if !self.player.is_dead() {
                        self.camera.position.y = 80.0;
                        self.vertical_velocity = 0.0;
                    }
                }
                if self.camera.position.y > 250.0 {
                    self.camera.position.y = 250.0;
                    self.vertical_velocity = self.vertical_velocity.min(0.0);
                }

                if input.scroll_delta > 0.0 {
                    self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
                } else if input.scroll_delta < 0.0 {
                    self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
                }

                if input.gp_lb_just_pressed || input.gp_dpad_left_just_pressed {
                    self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
                }
                if input.gp_rb_just_pressed || input.gp_dpad_right_just_pressed {
                    self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
                }

                for i in 0..10 {
                    if input.key_digits[i] {
                        self.player.selected_hotbar_slot = i as u8;
                    }
                }

                let mut selected_block_id = 0u8;
                if let Some(stack) = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                    selected_block_id = stack.block.to_u8();
                }
                self.renderer.set_selected_block(selected_block_id);

                let interact_left = (input.is_mouse_captured && left_down) || input.gp_right_trigger > 0.15;
                let interact_left_just = (!self.prev_left_down && left_down && input.is_mouse_captured) || input.gp_right_trigger_just_pressed;

                if interact_left {
                    if self.hand_animation < 0.1 {
                        self.hand_animation = 1.0;
                    }
                    let forward = self.camera.forward();
                    let held_tool = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].map(|s| s.block);

                    if interact_left_just {
                        if let Some((mob_idx, _)) = self.mob_manager.raycast_mob(self.camera.position, forward, 4.2) {
                            let mob = &mut self.mob_manager.mobs[mob_idx];
                            let hit_pos = mob.center_pos();
                            let dmg = held_tool.map(|b| b.attack_damage()).unwrap_or(1.5);
                            mob.take_damage(dmg, forward);
                            self.particles.spawn_mob_hit(hit_pos);
                            self.break_target = None;
                            self.break_progress = 0.0;
                        }
                    }

                    if let Some(hit) = self.world.raycast(self.camera.position, forward, 8.0) {
                        let target = hit.block_pos;
                        let broken_block = self.world.get_block(target);

                        if broken_block == BlockId::Tnt && interact_left_just {
                            self.world.prime_tnt(target);
                            self.break_target = None;
                            self.break_progress = 0.0;
                        } else {
                            if self.break_target != Some(target) {
                                self.break_target = Some(target);
                                self.break_progress = 0.0;
                            }

                            let break_time = if self.player.gamemode == GameMode::Creative {
                                0.0
                            } else {
                                broken_block.break_time_with_tool(held_tool)
                            };

                            if break_time == 0.0 {
                                self.break_progress = 1.0;
                            } else {
                                self.break_progress += dt / break_time;
                            }

                            if self.break_progress >= 1.0 {
                                self.break_progress = 0.0;
                                self.break_target = None;
                                self.hand_animation = 1.0;

                                let bx = target.x as f32 + 0.5;
                                let by = target.y as f32 + 0.5;
                                let bz = target.z as f32 + 0.5;
                                let color = broken_block.face_color(Face::Top);
                                self.particles.spawn_block_break(Vec3::new(bx - 0.5, by - 0.5, bz - 0.5), color);
                                self.world.set_block(target, BlockId::Air);

                                if self.player.gamemode == GameMode::Survival {
                                    self.player.exhaustion += 0.025;
                                    let seed = ((target.x as u32).wrapping_mul(73856093) ^ (target.y as u32).wrapping_mul(19349663) ^ (target.z as u32).wrapping_mul(83492791)) % 1000;
                                    let rx = ((seed % 100) as f32 / 100.0 - 0.5) * 1.8;
                                    let rz = (((seed / 10) % 100) as f32 / 100.0 - 0.5) * 1.8;
                                    let drop_vel = Vec3::new(rx, 3.2, rz);
                                    if broken_block != BlockId::Glass {
                                        let drop_block = if matches!(broken_block, BlockId::Leaves | BlockId::CherryLeaves | BlockId::AutumnLeaves) {
                                            if seed % 25 == 0 { Some(BlockId::Apple) } else { None }
                                        } else if matches!(broken_block, BlockId::Door | BlockId::DoorOpen) {
                                            Some(BlockId::Door)
                                        } else {
                                            Some(broken_block)
                                        };
                                        if let Some(drop_id) = drop_block {
                                            self.items.push(ItemEntity::new(Vec3::new(bx, by, bz), drop_vel, drop_id, 1));
                                        }
                                    }
                                }

                                if matches!(broken_block, BlockId::Door | BlockId::DoorOpen) {
                                    let above = target + IVec3::new(0, 1, 0);
                                    let below = target - IVec3::new(0, 1, 0);
                                    if matches!(self.world.get_block(above), BlockId::Door | BlockId::DoorOpen) {
                                        self.world.set_block(above, BlockId::Air);
                                    }
                                    if matches!(self.world.get_block(below), BlockId::Door | BlockId::DoorOpen) {
                                        self.world.set_block(below, BlockId::Air);
                                    }
                                }
                            }
                        }
                    } else {
                        self.break_target = None;
                        self.break_progress = 0.0;
                    }
                } else {
                    self.break_target = None;
                    self.break_progress = 0.0;
                }

                let interact_right = (input.is_mouse_captured && right_down && !self.prev_right_down) || input.gp_left_trigger_just_pressed;
                if interact_right {
                    let held_food = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize].map(|s| s.block);
                    let mut ate_food = false;
                    if let Some(food_id) = held_food {
                        if self.player.can_eat(food_id) {
                            if self.player.eat(food_id) {
                                ate_food = true;
                                self.hand_animation = 1.0;
                                let color = food_id.face_color(Face::Top);
                                let mouth_pos = self.camera.position + self.camera.forward() * 0.4 - Vec3::new(0.0, 0.2, 0.0);
                                self.particles.spawn_block_break(mouth_pos - Vec3::new(0.5, 0.5, 0.5), color);
                                if self.player.gamemode == GameMode::Survival {
                                    if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                        stack.count -= 1;
                                        if stack.count == 0 {
                                            self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !ate_food {
                        if let Some(hit) = self.world.raycast(self.camera.position, self.camera.forward(), 8.0) {
                        let targeted_block = self.world.get_block(hit.block_pos);
                        if matches!(targeted_block, BlockId::Door | BlockId::DoorOpen) && !is_crouching {
                            let new_block = if targeted_block == BlockId::Door { BlockId::DoorOpen } else { BlockId::Door };
                            self.world.set_block(hit.block_pos, new_block);
                            let above = hit.block_pos + IVec3::new(0, 1, 0);
                            let below = hit.block_pos - IVec3::new(0, 1, 0);
                            if matches!(self.world.get_block(above), BlockId::Door | BlockId::DoorOpen) {
                                self.world.set_block(above, new_block);
                            } else if matches!(self.world.get_block(below), BlockId::Door | BlockId::DoorOpen) {
                                self.world.set_block(below, new_block);
                            }
                            self.hand_animation = 1.0;
                        } else if targeted_block == BlockId::CraftingTable && !is_crouching {
                            self.crafting_table_open = true;
                            self.state = GameState::Inventory;
                            out.request_release_mouse = true;
                            self.horiz_velocity = Vec3::zero();
                            self.fly_velocity = Vec3::zero();
                        } else {
                            let place_pos = hit.block_pos + hit.normal;
                            let place_id = BlockId::from_u8(selected_block_id);
                            if place_id != BlockId::Air
                                && place_id.is_placeable()
                                && self.world.get_block(place_pos) == BlockId::Air
                            {
                                self.hand_animation = 1.0;
                                if place_id == BlockId::Door {
                                    let top_pos = place_pos + IVec3::new(0, 1, 0);
                                    if !block_intersects_player(place_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                        && !block_intersects_player(top_pos, self.camera.position, self.current_eye_offset, self.current_player_height)
                                        && self.world.get_block(top_pos) == BlockId::Air
                                    {
                                        self.world.set_block(place_pos, BlockId::Door);
                                        self.world.set_block(top_pos, BlockId::Door);
                                        if self.player.gamemode == GameMode::Survival {
                                            if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                                stack.count -= 1;
                                                if stack.count == 0 { self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None; }
                                            }
                                        }
                                    }
                                } else if !block_intersects_player(place_pos, self.camera.position, self.current_eye_offset, self.current_player_height) {
                                    self.world.set_block(place_pos, place_id);
                                    if self.player.gamemode == GameMode::Survival {
                                        if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                                            stack.count -= 1;
                                            if stack.count == 0 { self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None; }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

                self.world.advance_time(dt);
                self.world.update(self.camera.position, self.settings.render_distance);
                let hand_speed = if self.fly_mode {
                    self.fly_velocity.length()
                } else {
                    self.horiz_velocity.length()
                };
                if hand_speed > 0.05 {
                    self.walk_phase += dt * (5.0 + hand_speed * 0.9);
                }
                self.material_timer += dt;
                if self.material_timer >= 0.25 {
                    self.material_timer = 0.0;
                    self.world.simulate_materials(self.camera.position);
                }

                self.world.update_primed_tnts(dt);

                self.mob_manager.update(dt, &self.world, self.camera.position);
                for mob in &mut self.mob_manager.mobs {
                    if !mob.is_dead {
                        let dx = self.camera.position.x - mob.pos.x;
                        let dz = self.camera.position.z - mob.pos.z;
                        let dist_sq = dx * dx + dz * dz;

                        // Soft push separation between player and mob
                        let min_dist = mob.kind.half_width() + 0.35;
                        if dist_sq < min_dist * min_dist && (mob.pos.y - self.camera.position.y).abs() < 1.6 {
                            let dist = dist_sq.sqrt().max(0.001);
                            let push = (min_dist - dist) * 0.5;
                            let push_x = (dx / dist) * push;
                            let push_z = (dz / dist) * push;
                            mob.velocity.x -= push_x * 3.0;
                            mob.velocity.z -= push_z * 3.0;
                            self.horiz_velocity.x += push_x * 3.0;
                            self.horiz_velocity.z += push_z * 3.0;
                        }

                        // Hostile mob attack
                        if mob.kind.is_hostile() && dist_sq < 1.4 * 1.4 && (mob.pos.y - self.camera.position.y).abs() < 1.8 {
                            if self.player.take_damage(2.0) {
                                self.damage_tilt_timer = 0.4;
                                self.damage_tilt_dir = if mob.pos.x > self.camera.position.x { -1.0 } else { 1.0 };
                            }
                        }
                    }
                }
                for poof_pos in self.mob_manager.dying_poofs.drain(..) {
                    self.particles.spawn_mob_death_poof(poof_pos);
                }
                for drop in self.mob_manager.dying_drops.drain(..) {
                    let rx = ((self.mob_manager.mobs.len() as f32 * 17.0).sin()) * 1.5;
                    let rz = ((self.mob_manager.mobs.len() as f32 * 23.0).cos()) * 1.5;
                    self.items.push(ItemEntity::new(
                        drop.pos,
                        Vec3::new(rx, 3.0, rz),
                        drop.block,
                        drop.count,
                    ));
                }

                if self.player.is_dead() {
                    self.state = GameState::DeathScreen;
                    out.request_release_mouse = true;
                    self.death_hovered_btn = Some(0);
                    self.horiz_velocity = Vec3::zero();
                    self.fly_velocity = Vec3::zero();
                }

                if (q_down && !self.prev_q_down && input.is_mouse_captured) || input.gp_west_just_pressed {
                    if let Some(stack) = &mut self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] {
                        self.items.push(ItemEntity::new(
                            self.camera.position + self.camera.forward() * 0.5,
                            self.camera.forward() * 5.0,
                            stack.block,
                            1,
                        ));
                        stack.count -= 1;
                        if stack.count == 0 {
                            self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = None;
                        }
                    }
                }

                let player_center = self.camera.position - Vec3::new(0.0, 0.8, 0.0);
                let mut i = 0;
                while i < self.items.len() {
                    self.items[i].update(dt, &self.world, Some(player_center));

                    // Lava burns dropped items
                    let item_pos = self.items[i].position;
                    let item_block = self.world.get_block(item_pos.floor_to_ivec());
                    if item_block == BlockId::Lava {
                        self.particles.spawn_lava_burn(item_pos);
                        self.items.remove(i);
                        continue;
                    }

                    let dist_sq = (self.items[i].position - player_center).length_sq();
                    if self.items[i].pickup_delay <= 0.0 && dist_sq < 3.24 {
                        let remaining = self.player.inventory.add_item(self.items[i].block, self.items[i].count);
                        if remaining == 0 {
                            self.items.remove(i);
                            continue;
                        } else {
                            self.items[i].count = remaining;
                        }
                    }
                    i += 1;
                }

                for exp_pos in self.world.pending_explosions.drain(..) {
                    self.particles.spawn_explosion(exp_pos);
                    self.mob_manager.apply_explosion(exp_pos, 5.0, 30.0);
                    let dist = (self.camera.position - exp_pos).length();
                    if dist < 6.0 {
                        let dmg = (1.0 - dist / 6.0) * 18.0;
                        if self.player.take_damage(dmg) {
                            self.damage_tilt_timer = 0.4;
                            self.damage_tilt_dir = if exp_pos.x > self.camera.position.x { -1.0 } else { 1.0 };
                        }
                    }
                }

                self.particles.update(dt);

                let aspect = self.width as f32 / self.height as f32;

                if (self.camera.fov_y - self.last_fov).abs() > 0.0001 || (aspect - self.last_aspect).abs() > 0.0001 {
                    self.last_fov = self.camera.fov_y;
                    self.last_aspect = aspect;
                    self.proj_matrix = self.camera.projection_matrix(aspect);
                }
                #[cfg(not(target_os = "vita"))]
                let (shake_offset, roll) = if self.damage_tilt_timer > 0.0 {
                    let progress = self.damage_tilt_timer / 0.4;
                    let roll = (progress * std::f32::consts::PI).sin() * 0.14 * self.damage_tilt_dir;
                    let sx = (self.total_time * 50.0).sin() * 0.08 * progress;
                    let sy = (self.total_time * 60.0).cos() * 0.05 * progress;
                    (self.camera.right_flat() * sx + Vec3::new(0.0, sy, 0.0), roll)
                } else {
                    (Vec3::zero(), 0.0)
                };
                #[cfg(not(target_os = "vita"))]
                let view_proj = self.proj_matrix.mul(&self.camera.view_matrix_with_tilt(shake_offset, roll));
                #[cfg(not(target_os = "vita"))]
                {
                    self.renderer.fog_enabled = self.settings.fog;
                    self.renderer.render_frame(&mut self.world, &self.camera);

                    let is_occluded = self.world.is_view_fully_occluded(self.camera.position, self.camera.yaw, self.camera.pitch);

                    if !is_occluded {
                        for mob in &self.mob_manager.mobs {
                            self.renderer.draw_mob(mob, &self.camera, &view_proj);
                        }

                        for tnt in &self.world.primed_tnts {
                            self.renderer.draw_tnt_cube(tnt.pos, tnt.fuse, &self.camera, &view_proj);
                        }

                        if self.settings.clouds {
                            self.renderer.draw_clouds(self.camera.position, &view_proj, self.total_time);
                        }

                        for item in &self.items {
                            // Ground drop shadow below floating block (Minecraft style)
                            let ipos_x = item.position.x.floor() as i32;
                            let ipos_z = item.position.z.floor() as i32;
                            let item_y = item.position.y;
                            for dy in 0..=3 {
                                let test_y = (item_y.floor() as i32) - dy;
                                let b = self.world.get_block(crate::core::math::IVec3::new(ipos_x, test_y, ipos_z));
                                if b.is_solid() {
                                    let top_y = test_y as f32 + 1.0 + 0.015;
                                    if item_y >= top_y - 0.2 {
                                        self.renderer.draw_item_shadow(item.position, top_y, &view_proj);
                                        break;
                                    }
                                }
                            }

                            let y_offset = (item.age * 3.0).sin() * 0.08;
                            let mut pos = item.position;
                            pos.y += y_offset;
                            let yaw = item.age * 2.5;
                            self.renderer.draw_item_3d(item.block, pos, yaw, &view_proj);
                        }

                        if let Some(target) = self.break_target {
                            if self.break_progress > 0.0 {
                                self.renderer.draw_block_break_cracks(target, self.break_progress, &view_proj);
                            }
                        }

                        if self.settings.particles {
                            for p in &self.particles.particles {
                                self.renderer.draw_billboard(p.pos, p.size, p.color, &self.camera, &view_proj);
                            }
                        }
                    }

                    let block_at_camera = self.world.get_block(self.camera.position.floor_to_ivec());
                    if block_at_camera == BlockId::Lava {
                        self.renderer.draw_lava_overlay(self.total_time);
                    } else if self.settings.underwater_fx && block_at_camera == BlockId::Water {
                        self.renderer.draw_underwater_overlay(self.total_time);
                    }

                    if self.damage_tilt_timer > 0.0 {
                        self.renderer.draw_damage_flash(self.damage_tilt_timer / 0.4);
                    }
                    if self.player.gamemode == GameMode::Survival && self.player.health <= 2.0 && self.player.health > 0.0 {
                        self.renderer.draw_low_health_vignette(self.total_time);
                    }
                }

                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0);
                }

                self.renderer.draw_crosshair();
                self.renderer.draw_break_progress(self.break_progress);
                #[cfg(not(target_os = "vita"))]
                {
                    let hand_block: Option<u8> = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
                        .map(|s| s.block.to_u8());
                    self.renderer.draw_hand_3d(&self.camera, &view_proj, hand_block, self.walk_phase, hand_speed, self.hand_animation);
                }
                self.renderer.draw_hotbar(&self.player);

                if self.settings.debug_overlay {
                    self.debug_timer += dt;
                    if self.debug_timer >= 0.25 {
                        self.debug_timer = 0.0;
                        self.cpu_time_ms = dt * 1000.0;
                        self.fps_display = (1.0 / dt.max(0.001)).round() as u32;

                        let mode_str = if self.fly_mode {
                            "FLY"
                        } else if is_crouching {
                            "SNEAK"
                        } else if is_sprinting {
                            "SPRINT"
                        } else {
                            "WALK"
                        };

                        // Line 0: FPS & CPU Time
                        self.debug_lines[0].clear();
                        let _ = write!(&mut self.debug_lines[0], "FPS: {} ({:.1}MS) | CPU: {:.1}MS", self.fps_display, dt * 1000.0, self.cpu_time_ms);

                        // Line 1: GPU Triangles & Chunks
                        self.debug_lines[1].clear();
                        #[cfg(target_os = "vita")]
                        let _ = write!(&mut self.debug_lines[1], "GPU: {:.1}K TRI | CHUNKS: {}/{}", self.gpu_triangles as f32 / 1000.0, self.chunk_slots_info.0, self.chunk_slots_info.1);
                        #[cfg(not(target_os = "vita"))]
                        let _ = write!(&mut self.debug_lines[1], "CHUNKS: {}", self.world.chunks.len());

                        // Line 2: RAM / VRAM Allocations
                        self.debug_lines[2].clear();
                        #[cfg(target_os = "vita")]
                        let _ = write!(&mut self.debug_lines[2], "RAM: {:.1}MB | CDRAM: {:.1}MB", self.ram_mb, self.cdram_mb);
                        #[cfg(not(target_os = "vita"))]
                        let _ = write!(&mut self.debug_lines[2], "RAM (EST): ~{:.1}MB", (self.world.chunks.len() * 32 * 32 * 128) as f32 / (1024.0 * 1024.0));

                        // Line 3: Position
                        self.debug_lines[3].clear();
                        let _ = write!(&mut self.debug_lines[3], "POS: X:{:.1} Y:{:.1} Z:{:.1}", self.camera.position.x, self.camera.position.y, self.camera.position.z);

                        // Line 4: Selected or Targeted Block
                        self.debug_lines[4].clear();
                        let sel_name = BlockId::from_u8(self.renderer.selected_block()).name().to_uppercase();
                        if let Some(target) = self.break_target {
                            let target_block = self.world.get_block(target);
                            let _ = write!(&mut self.debug_lines[4], "TARGET: {} X:{} Y:{} Z:{}", target_block.name().to_uppercase(), target.x, target.y, target.z);
                        } else {
                            let _ = write!(&mut self.debug_lines[4], "BLOCK: {}", sel_name);
                        }

                        // Line 5: Entities & Gamemode
                        self.debug_lines[5].clear();
                        let _ = write!(&mut self.debug_lines[5], "MOBS: {} | ENTITIES: {} | MODE: {}", self.mob_manager.mobs.len(), self.particles.particles.len(), mode_str);

                        // Line 6: Biome
                        self.debug_lines[6].clear();
                        let px = self.camera.position.x as i32;
                        let pz = self.camera.position.z as i32;
                        let biome = self.world.biome_name_at(px, pz);
                        let biome_key = match biome {
                            "MOUNTAINS" => "biome.mountains",
                            "SNOW" => "biome.snow",
                            "DESERT" => "biome.desert",
                            "OCEAN" => "biome.ocean",
                            "CHERRY GROVE" => "biome.cherry_grove",
                            "SAVANNA" => "biome.savanna",
                            "FOREST" => "biome.forest",
                            _ => "biome.plains",
                        };
                        let _ = write!(&mut self.debug_lines[6], "BIOME: {}", crate::core::i18n::t(biome_key));

                        // Line 7: Time of Day & Light Level
                        self.debug_lines[7].clear();
                        let time_hrs = (self.world.day_fraction() * 24.0 + 6.0) % 24.0;
                        let hours = time_hrs.floor() as u32;
                        let minutes = (time_hrs.fract() * 60.0) as u32;
                        let time_key = if time_hrs >= 5.0 && time_hrs < 7.0 {
                            "time.dawn"
                        } else if time_hrs >= 7.0 && time_hrs < 12.0 {
                            "time.morning"
                        } else if time_hrs >= 12.0 && time_hrs < 14.0 {
                            "time.noon"
                        } else if time_hrs >= 14.0 && time_hrs < 18.0 {
                            "time.afternoon"
                        } else if time_hrs >= 18.0 && time_hrs < 21.0 {
                            "time.dusk"
                        } else {
                            "time.night"
                        };
                        let _ = write!(
                            &mut self.debug_lines[7],
                            "TIME: {} ({:02}:{:02}) | LIGHT: {:.0}%",
                            crate::core::i18n::t(time_key),
                            hours,
                            minutes,
                            self.world.sunlight_factor() * 100.0
                        );
                    }

                    self.renderer.draw_debug_panel(&self.debug_lines);
                }

                if !input.is_mouse_captured {
                    self.renderer.draw_capture_prompt(true);
                }

                self.thumbnail_timer += dt;
                if self.thumbnail_timer >= 0.5 {
                    self.thumbnail_timer = 0.0;
                    self.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(self.renderer.pixel_buffer(), self.width, self.height));
                }
            }
            GameState::Paused => {
                let t_fov = 1.0 - (-8.0 * dt).exp();
                self.camera.fov_y += (self.settings.fov_deg.to_radians() - self.camera.fov_y) * t_fov;

                let mut hovered_idx = self.pause_hovered_btn;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in self.pause_menu_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            self.pause_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.pause_hovered_btn {
                        Some(0) | None => 4,
                        Some(i) => i - 1,
                    };
                    self.pause_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.pause_hovered_btn {
                        Some(4) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.pause_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_pause_btn = if (left_down && !self.prev_left_down) || input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
                    hovered_idx
                } else {
                    None
                };

                if let Some(btn) = click_pause_btn {
                    match btn {
                        0 => {
                            self.state = GameState::Playing;
                            out.request_capture_mouse = true;
                        }
                        1 => {
                            self.state = GameState::Settings(PreviousMenu::Paused);
                            self.settings_hovered_opt = Some(0);
                        }
                        2 => {
                            self.fly_mode = !self.fly_mode;
                        }
                        3 => {
                            let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                            self.state = GameState::MainMenu;
                            out.request_release_mouse = true;
                        }
                        4 => {
                            let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                            out.quit_requested = true;
                        }
                        _ => {}
                    }
                }

                if (escape_down && !self.prev_escape_down) || (p_down && !self.prev_p_down) || input.gp_east_just_pressed || input.gp_start_just_pressed {
                    self.state = GameState::Playing;
                    out.request_capture_mouse = true;
                }
                if f_down && !self.prev_f_down {
                    self.fly_mode = !self.fly_mode;
                }
                if q_down && !self.prev_q_down {
                    let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                    out.quit_requested = true;
                }
                if m_down && !self.prev_m_down {
                    let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                    self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    self.state = GameState::MainMenu;
                    out.request_release_mouse = true;
                }

                self.renderer.fog_enabled = self.settings.fog;
                #[cfg(not(target_os = "vita"))]
                self.renderer.render_frame(&mut self.world, &self.camera);
                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0x0C0C10);
                }
                self.renderer.draw_pause_menu(self.fly_mode, hovered_idx);
            }
            GameState::Inventory => {
                if input.gp_lb_just_pressed {
                    self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
                }
                if input.gp_rb_just_pressed {
                    self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
                }

                if (e_down && !self.prev_e_down)
                    || (escape_down && !self.prev_escape_down)
                    || (p_down && !self.prev_p_down)
                    || input.gp_east_just_pressed
                    || input.gp_north_just_pressed
                    || input.gp_start_just_pressed
                {
                    self.player.inventory.return_crafting_items();
                    if self.player.gamemode == GameMode::Creative {
                        self.player.inventory.held_item = None;
                    }
                    self.crafting_table_open = false;
                    self.state = GameState::Playing;
                    out.request_capture_mouse = true;
                }

                let layout = InventoryLayout::new(self.renderer.width, self.renderer.height, self.crafting_table_open);
                let inv_hovered_target = mouse_pos.and_then(|(mx, my)| {
                    layout.find_slot_at(mx as i32, my as i32, &self.player)
                });

                // Number keys 1-9 (indices 1..9) and 0 (index 0) to assign/swap into hotbar slots
                for i in 0..10 {
                    if input.key_digits[i] && !self.prev_key_digits[i] {
                        if let Some(target) = inv_hovered_target {
                            match target {
                                InventorySlotTarget::Creative(id) => {
                                    self.player.inventory.hotbar[i] = Some(ItemStack {
                                        block: BlockId::from_u8(id),
                                        count: 64,
                                    });
                                }
                                InventorySlotTarget::Hotbar(from_idx) => {
                                    self.player.inventory.hotbar.swap(i, from_idx);
                                }
                                InventorySlotTarget::Main(from_idx) => {
                                    std::mem::swap(&mut self.player.inventory.hotbar[i], &mut self.player.inventory.main[from_idx]);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
                let is_right_active = right_down || input.gp_btn_west;
                let click_right = (!self.prev_right_down && right_down) || input.gp_west_just_pressed;

                if !is_right_active {
                    self.right_drag_visited.clear();
                }

                // Handle right-click dragging across slots to distribute 1 item per slot (like Minecraft)
                if is_right_active {
                    if let Some(target) = inv_hovered_target {
                        if matches!(target, InventorySlotTarget::Hotbar(_) | InventorySlotTarget::Main(_) | InventorySlotTarget::Crafting2x2(_) | InventorySlotTarget::Crafting3x3(_)) {
                            if !self.right_drag_visited.contains(&target) {
                                self.right_drag_visited.push(target);
                                match target {
                                    InventorySlotTarget::Hotbar(idx) => {
                                        let slot = &mut self.player.inventory.hotbar[idx];
                                        if click_right && self.player.inventory.held_item.is_none() {
                                            Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                        } else {
                                            Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                        }
                                    }
                                    InventorySlotTarget::Main(idx) => {
                                        let slot = &mut self.player.inventory.main[idx];
                                        if click_right && self.player.inventory.held_item.is_none() {
                                            Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                        } else {
                                            Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                        }
                                    }
                                    InventorySlotTarget::Crafting2x2(idx) => {
                                        let slot = &mut self.player.inventory.crafting_2x2[idx];
                                        if click_right && self.player.inventory.held_item.is_none() {
                                            Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                        } else {
                                            Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                        }
                                    }
                                    InventorySlotTarget::Crafting3x3(idx) => {
                                        let slot = &mut self.player.inventory.crafting_3x3[idx];
                                        if click_right && self.player.inventory.held_item.is_none() {
                                            Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                        } else {
                                            Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                if click_left || (click_right && inv_hovered_target.map_or(false, |t| matches!(t, InventorySlotTarget::CreativeTab(_) | InventorySlotTarget::Creative(_) | InventorySlotTarget::Trash | InventorySlotTarget::CraftingOutput))) {
                    if let Some(target) = inv_hovered_target {
                        match target {
                            InventorySlotTarget::CreativeTab(tab_idx) => {
                                self.player.creative_tab = tab_idx;
                            }
                            InventorySlotTarget::Creative(id) => {
                                let block = BlockId::from_u8(id);
                                if input.key_shift {
                                    // Shift-click places 64-stack into first empty hotbar slot
                                    let placed = if let Some(slot) = self.player.inventory.hotbar.iter_mut().find(|s| s.is_none()) {
                                        *slot = Some(ItemStack { block, count: 64 });
                                        true
                                    } else {
                                        false
                                    };
                                    if !placed {
                                        self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = Some(ItemStack { block, count: 64 });
                                    }
                                } else {
                                    // Normal click: pick up full 64-stack into hand
                                    self.player.inventory.held_item = Some(ItemStack { block, count: 64 });
                                }
                            }
                            InventorySlotTarget::Trash => {
                                if input.key_shift {
                                    // Shift-click on trash clears all hotbar slots
                                    self.player.inventory.hotbar.fill(None);
                                    self.player.inventory.held_item = None;
                                } else {
                                    self.player.inventory.held_item = None;
                                }
                            }
                            InventorySlotTarget::Hotbar(idx) => {
                                let slot = &mut self.player.inventory.hotbar[idx];
                                if click_left {
                                    Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Main(idx) => {
                                let slot = &mut self.player.inventory.main[idx];
                                if click_left {
                                    Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Crafting2x2(idx) => {
                                let slot = &mut self.player.inventory.crafting_2x2[idx];
                                if click_left {
                                    Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Crafting3x3(idx) => {
                                let slot = &mut self.player.inventory.crafting_3x3[idx];
                                if click_left {
                                    Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::CraftingOutput => {
                                let result = if self.crafting_table_open {
                                    self.recipe_book.match_grid(&self.player.inventory.crafting_3x3, 3, 3)
                                } else {
                                    self.recipe_book.match_grid(&self.player.inventory.crafting_2x2, 2, 2)
                                };
                                if let Some(res) = result {
                                    let can_take = match self.player.inventory.held_item {
                                        None => true,
                                        Some(h) => h.block == res.block && h.count + res.count <= 64,
                                    };
                                    if can_take {
                                        if let Some(h) = &mut self.player.inventory.held_item {
                                            h.count += res.count;
                                        } else {
                                            self.player.inventory.held_item = Some(res);
                                        }
                                        if self.crafting_table_open {
                                            RecipeBook::consume_grid(&mut self.player.inventory.crafting_3x3);
                                        } else {
                                            RecipeBook::consume_grid(&mut self.player.inventory.crafting_2x2);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                #[cfg(not(target_os = "vita"))]
                self.renderer.render_frame(&mut self.world, &self.camera);
                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0x0C0C10);
                }
                self.renderer.draw_inventory(
                    &self.player,
                    inv_hovered_target,
                    self.crafting_table_open,
                    &self.recipe_book,
                    mouse_pos,
                );
            }
            GameState::DeathScreen => {
                let mut hovered_idx = self.death_hovered_btn;
                if let Some((mx, my)) = mouse_pos {
                    let px = mx as i32;
                    let py = my as i32;
                    for (i, &(bx, by, bw, bh)) in self.death_screen_rects.iter().enumerate() {
                        if px >= bx && px < bx + bw && py >= by && py < by + bh {
                            hovered_idx = Some(i);
                            self.death_hovered_btn = Some(i);
                            break;
                        }
                    }
                }

                if input.gp_dpad_up_just_pressed || (up_arrow_down && !self.prev_up_arrow_down) {
                    let next = match self.death_hovered_btn {
                        Some(0) | None => 1,
                        Some(i) => i - 1,
                    };
                    self.death_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }
                if input.gp_dpad_down_just_pressed || (down_arrow_down && !self.prev_down_arrow_down) {
                    let next = match self.death_hovered_btn {
                        Some(1) | None => 0,
                        Some(i) => i + 1,
                    };
                    self.death_hovered_btn = Some(next);
                    hovered_idx = Some(next);
                }

                let click_death_btn = if (left_down && !self.prev_left_down) || input.gp_south_just_pressed || (enter_down && !self.prev_enter_down) {
                    hovered_idx
                } else {
                    None
                };

                if let Some(btn) = click_death_btn {
                    match btn {
                        0 => {
                            self.player.health = self.player.max_health;
                            self.player.hunger = self.player.max_hunger;
                            self.player.saturation = 5.0;
                            self.player.exhaustion = 0.0;
                            self.player.starvation_timer = 0.0;
                            self.player.heal_timer = 0.0;
                            self.player.air = self.player.max_air;
                            self.player.head_underwater = false;
                            self.drown_damage_timer = 0.0;
                            self.camera.position = self.world.player_position;
                            self.camera.pitch = 0.0;
                            self.vertical_velocity = 0.0;
                            self.horiz_velocity = Vec3::zero();
                            self.player.fall_distance = 0.0;
                            self.player.invulnerability_timer = 3.0;
                            self.player.has_landed = false;
                            self.damage_tilt_timer = 0.0;
                            self.state = GameState::Playing;
                            out.request_capture_mouse = true;
                        }
                        1 => {
                            let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                            self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                            self.state = GameState::MainMenu;
                            out.request_release_mouse = true;
                        }
                        _ => {}
                    }
                }

                if (escape_down && !self.prev_escape_down) || (p_down && !self.prev_p_down) || input.gp_east_just_pressed {
                    let _ = save_current_world(&mut self.world, &self.camera, &self.player, self.save_slot, self.last_clean_thumbnail.as_deref());
                    self.save_metadata_cache = std::array::from_fn(|idx| get_save_metadata(idx as u8 + 1));
                    self.state = GameState::MainMenu;
                    out.request_release_mouse = true;
                }

                self.renderer.fog_enabled = self.settings.fog;
                #[cfg(not(target_os = "vita"))]
                self.renderer.render_frame(&mut self.world, &self.camera);
                #[cfg(target_os = "vita")]
                {
                    self.renderer.color_buffer.fill(0x0C0C10);
                }
                self.renderer.draw_death_screen(hovered_idx);
            }
        }

        self.prev_left_down = left_down;
        self.prev_right_down = right_down;
        self.prev_escape_down = escape_down;
        self.prev_p_down = p_down;
        self.prev_e_down = e_down;
        self.prev_f_down = f_down;
        self.prev_q_down = q_down;

        self.prev_m_down = m_down;
        self.prev_g_down = g_down;
        self.prev_f5_down = f5_down;
        self.prev_enter_down = enter_down;
        self.prev_up_arrow_down = up_arrow_down;
        self.prev_down_arrow_down = down_arrow_down;
        self.prev_left_arrow_down = left_arrow_down;
        self.prev_right_arrow_down = right_arrow_down;
        self.prev_n_down = n_down;
        self.prev_del_down = del_down;
        self.prev_backspace_down = backspace_down;
        self.prev_tab_down = tab_down;
        self.prev_a_down = a_down;
        self.prev_c_down = c_down;
        self.prev_v_down = v_down;
        self.prev_x_down = x_down;
        self.prev_key_digits = input.key_digits;

        out.title = format!(
            "RustCraft [{} renderer] - {} FPS - block: {}{}{}",
            self.renderer.backend_name(),
            self.fps_display,
            BlockId::from_u8(self.renderer.selected_block()).name(),
            if self.fly_mode { " - [FLYING]" } else { "" },
            if matches!(self.state, GameState::Paused) { " - [PAUSED]" } else { "" }
        );

        out
    }
}

impl crate::engine::EngineApp for RustCraftApp {
    fn update(&mut self, dt: f32, input: &InputState) -> AppOutput {
        self.update_and_render(dt, input)
    }

    fn render(&mut self) {
        // Frame rendering is integrated into the update_and_render pipeline
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.renderer.resize(width, height);
    }

    fn pixel_buffer(&self) -> &[u32] {
        self.pixel_buffer()
    }
}

