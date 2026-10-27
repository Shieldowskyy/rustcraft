//! Main RustCraft application state machine, game loop coordinator, and UI controller.
//!
//! Orchestrates player movement, voxel interaction, mob AI, particle simulation,
//! audio cues, inventory management, rendering backend dispatches, and menus.

pub mod types;
pub mod audio;
pub mod containers;
pub mod gameplay;
pub mod menus;
pub mod multiplayer;
pub mod profiler;

pub use types::*;

use crate::core::controls::ControlsManager;
use crate::core::math::{IVec3, Mat4, Vec3};
use crate::core::profiler::FrameProfiler;
use crate::core::settings::GameSettings;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::MobManager;
use crate::gameplay::particles::ParticleSystem;
use crate::gameplay::physics::{STANDING_EYE_OFFSET, STANDING_HEIGHT};
use crate::gameplay::player::{GameMode, Player};
use crate::gameplay::recipe::RecipeBook;
use crate::graphics::camera::Camera;
use crate::graphics::software::Renderer;
use crate::ui::VISIBLE_SAVE_ROWS;
use crate::world::block::BlockId;
use crate::world::storage::{get_save_metadata, SaveMetadata};
use crate::world::World;

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
    pub save_card_double_click_timer: f32,
    pub save_card_last_clicked_slot: Option<u8>,
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
    pub dig_sound_timer: f32,
    pub total_time: f32,
    pub material_timer: f32,
    pub hand_animation: f32,
    pub walk_phase: f32,
    pub damage_tilt_timer: f32,
    pub damage_tilt_dir: f32,
    pub lava_damage_timer: f32,
    pub drown_damage_timer: f32,
    pub debug_lines: Vec<String>,
    pub debug_right_lines: Vec<String>,
    pub profiler: FrameProfiler,
    pub profiler_enabled: bool,
    pub debug_timer: f32,
    pub cpu_time_ms: f32,
    pub gpu_triangles: u32,
    pub cdram_mb: f32,
    pub ram_mb: f32,
    pub chunk_slots_info: (usize, usize),
    pub last_fov: f32,
    pub last_aspect: f32,
    pub proj_matrix: Mat4,
    pub main_menu_rects: [(i32, i32, i32, i32); 4],
    pub main_menu_nick_rect: (i32, i32, i32, i32),
    pub nickname_input: String,
    pub nickname_selected: bool,
    pub nickname_dialog_hovered_btn: Option<usize>,
    pub nickname_dialog_rects: [(i32, i32, i32, i32); 4],
    pub ime_requested: Option<(String, String)>,
    pub is_ime_active: bool,
    pub multiplayer_menu_rects: [(i32, i32, i32, i32); crate::ui::MULTIPLAYER_RECT_COUNT],
    pub multiplayer_server_addr: String,
    pub multiplayer_focus: usize,
    pub multiplayer_status: Option<String>,
    pub multiplayer_is_connecting: bool,
    pub lan_discovery: crate::net::LanDiscoveryClient,
    pub selected_lan_server: Option<usize>,
    pub main_menu_nick_hovered: bool,
    pub net_client: Option<crate::net::NetworkClient>,
    pub is_multiplayer: bool,
    pub save_card_rects: [(i32, i32, i32, i32); VISIBLE_SAVE_ROWS],
    pub save_btn_rects: [(i32, i32, i32, i32); 4],
    pub create_world_rects: [(i32, i32, i32, i32); 6],
    pub settings_tab: usize,
    pub settings_tab_rects: [(i32, i32, i32, i32); 4],
    pub settings_rects: [(i32, i32, i32, i32); 13],
    pub settings_audio_rects: [(i32, i32, i32, i32); 7],
    pub settings_controls_rects: [(i32, i32, i32, i32); 11],
    pub settings_debug_rects: [(i32, i32, i32, i32); 5],
    pub rebind_search: String,
    pub rebind_scroll: usize,
    pub rebinding_target: Option<(crate::core::controls::Action, crate::ui::controls_ui::RebindSlot)>,
    pub rebind_cursor_timer: f32,
    pub rebind_search_focused: bool,
    pub is_gamepad_active: bool,
    pub last_mouse_pos: Option<(f32, f32)>,
    pub pending_rumble: Option<RumbleRequest>,
    pub profiler_server: Option<crate::core::profiler_server::ProfilerServer>,
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
    pub profiler_frame_count: u64,
    pub profiler_copied_timer: f32,
    pub telemetry_timer: f32,
    pub camera_perspective: CameraPerspective,
    pub autotest: Option<crate::gameplay::autotest::AutoTestRunner>,
    pub audio: crate::audio::AudioEngine,
    pub soundtrack: crate::audio::SoundtrackManager,
    pub was_on_ground: bool,
    pub ambient_probe_timer: f32,
    pub cached_ambient_ctx: crate::audio::AmbientContext,

    // Controls toggle states
    pub sprint_toggled: bool,
    pub crouch_toggled: bool,

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
    prev_sprint_btn: bool,
    prev_sneak_btn: bool,
    prev_l1_btn: bool,
    prev_r1_btn: bool,
    prev_key_digits: [bool; 10],
    pub chat: crate::gameplay::chat::ChatState,
    prev_t_down: bool,
    prev_slash_down: bool,
    prev_b_down: bool,
    prev_u_down: bool,
    prev_space_down: bool,
    pub controls: ControlsManager,
    pub minimap: crate::ui::minimap::MinimapState,
    pub worldmap: crate::ui::minimap::WorldMapState,
}



impl RustCraftApp {
    pub fn new(width: usize, height: usize, renderer_choice: &str, seed: u32) -> Self {
        crate::log_info!("APP", "[Step 1/5] Loading game configuration, mods and language...");
        let settings = GameSettings::load();
        crate::core::i18n::set_language(settings.language);

        let mut recipe_book = RecipeBook::new();
        crate::engine::modding::ModManager::load_mods(&mut recipe_book);

        crate::log_info!("APP", "[Step 2/5] Initializing renderer ({})...", renderer_choice);
        let mut renderer = match renderer_choice {
            #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon"), not(feature = "xbox360")))]
            "vulkan" => Renderer::new_vulkan(width, height).unwrap_or_else(|error| {
                eprintln!("Failed to initialize Vulkan renderer: {error}, falling back to software renderer");
                Renderer::new(width, height)
            }),
            #[cfg(all(not(target_arch = "wasm32"), not(target_arch = "powerpc"), not(target_os = "vita"), not(target_os = "android"), not(target_os = "xbox360"), not(target_os = "xenon"), not(feature = "xbox360")))]
            "opengl" | "gl" => Renderer::new_opengl(width, height).unwrap_or_else(|error| {
                eprintln!("Failed to initialize OpenGL renderer: {error}, falling back to software renderer");
                Renderer::new(width, height)
            }),
            _ => Renderer::new(width, height),
        };
        renderer.fog_enabled = settings.fog;
        renderer.render_distance = settings.render_distance;
        if !settings.fancy_leaves {
            renderer.update_textures(false);
        }

        crate::log_info!("APP", "[Step 3/5] Initializing world instance (seed: {})...", seed);
        let world = World::new_with_seed(seed);
        let start_player_pos = world.player_position;

        crate::log_info!("APP", "[Step 4/5] Initializing camera, player & UI layout...");
        let camera = Camera::new(start_player_pos);

        let mut player = Player::new();
        player.gamemode = GameMode::Survival;

        let main_menu_rects = renderer.main_menu_button_rects();
        let main_menu_nick_rect = renderer.main_menu_nickname_rect();
        let nickname_dialog_rects = renderer.nickname_dialog_rects();
        let default_nickname = settings.nickname.clone();
        let multiplayer_menu_rects = renderer.multiplayer_menu_rects();
        let save_card_rects = renderer.save_card_rects();
        let save_btn_rects = renderer.save_bottom_button_rects();
        renderer.render_distance = settings.render_distance;
        renderer.brightness = settings.brightness;
        let create_world_rects = renderer.create_world_rects();
        let settings_tab_rects = renderer.settings_tab_rects();
        let settings_rects = renderer.settings_graphics_rects();
        let settings_audio_rects = renderer.settings_audio_rects();
        let settings_controls_rects = renderer.settings_controls_rects();
        let settings_debug_rects = renderer.settings_debug_rects();
        let pause_menu_rects = renderer.pause_menu_button_rects();
        let death_screen_rects = renderer.death_screen_button_rects();

        let profiler_server = if settings.profiler_server {
            crate::core::profiler_server::ProfilerServer::start(settings.profiler_server_port).ok()
        } else {
            None
        };

        let aspect = width as f32 / height as f32;
        let last_fov = camera.fov_y;
        let proj_matrix = camera.projection_matrix(aspect);

        crate::log_info!("APP", "[Step 5/5] Checking save slots cache...");
        let save_metadata_cache = std::array::from_fn(|index| {
            get_save_metadata(index as u8 + 1)
        });
        crate::log_info!("APP", "RustCraftApp state initialization complete.");

        let autotest_enabled = crate::cli::parse_autotest_enabled();
        let (initial_state, autotest) = if autotest_enabled {
            crate::log_info!("AUTOTEST", "AutoTest Mode ENABLED: auto-starting automated bot test sequence in active world");
            (GameState::Playing, Some(crate::gameplay::autotest::AutoTestRunner::new()))
        } else {
            (GameState::MainMenu, None)
        };

        let mut audio = crate::audio::AudioEngine::new();
        audio.set_master_volume(settings.master_volume);
        audio.set_sfx_volume(settings.sfx_volume);
        audio.set_music_volume(settings.music_volume);
        audio.set_ambient_volume(settings.ambient_volume);
        audio.set_ui_volume(settings.ui_volume);
        if settings.mute_audio {
            audio.toggle_mute();
        }

        let soundtrack = crate::audio::SoundtrackManager::new("soundtracks");

        let app = Self {
            width,
            height,
            renderer,
            world,
            camera,
            particles: ParticleSystem::new(),
            mob_manager: MobManager::new(),
            player,
            items: Vec::new(),
            recipe_book,
            crafting_table_open: false,
            settings,
            state: initial_state,
            autotest,
            save_slot: 1,
            save_select_slot: 1,
            save_select_scroll: 0,
            save_metadata_cache,
            save_card_double_click_timer: 0.0,
            save_card_last_clicked_slot: None,
            settings_tab: 0,
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
            dig_sound_timer: 0.0,
            total_time: 0.0,
            material_timer: 0.0,
            hand_animation: 0.0,
            walk_phase: 0.0,
            damage_tilt_timer: 0.0,
            damage_tilt_dir: 1.0,
            lava_damage_timer: 0.45,
            drown_damage_timer: 0.0,
            debug_lines: (0..10).map(|_| String::with_capacity(96)).collect(),
            debug_right_lines: (0..14).map(|_| String::with_capacity(96)).collect(),
            profiler: FrameProfiler::default(),
            profiler_enabled: cfg!(feature = "profiler"),
            debug_timer: 0.15,
            cpu_time_ms: 16.6,
            gpu_triangles: 0,
            cdram_mb: 0.0,
            ram_mb: 0.0,
            chunk_slots_info: (0, 144),
            last_fov,
            last_aspect: aspect,
            proj_matrix,
            main_menu_rects,
            main_menu_nick_rect,
            nickname_input: default_nickname,
            nickname_selected: false,
            nickname_dialog_hovered_btn: Some(0),
            nickname_dialog_rects,
            ime_requested: None,
            is_ime_active: false,
            multiplayer_menu_rects,
            multiplayer_server_addr: "127.0.0.1:26656".to_string(),
            multiplayer_focus: 0,
            multiplayer_status: None,
            multiplayer_is_connecting: false,
            lan_discovery: crate::net::LanDiscoveryClient::new(),
            selected_lan_server: None,
            main_menu_nick_hovered: false,
            net_client: None,
            is_multiplayer: false,
            save_card_rects,
            save_btn_rects,
            create_world_rects,
            settings_tab_rects,
            settings_rects,
            settings_audio_rects,
            settings_controls_rects,
            settings_debug_rects,
            rebind_search: String::new(),
            rebind_scroll: 0,
            rebinding_target: None,
            rebind_cursor_timer: 0.0,
            rebind_search_focused: true,
            is_gamepad_active: false,
            last_mouse_pos: None,
            pending_rumble: None,
            profiler_server,
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
            profiler_frame_count: 0,
            profiler_copied_timer: 0.0,
            telemetry_timer: 0.0,
            camera_perspective: CameraPerspective::FirstPerson,
            audio,
            soundtrack,
            was_on_ground: false,
            ambient_probe_timer: 0.0,
            cached_ambient_ctx: crate::audio::AmbientContext::default(),

            sprint_toggled: false,
            crouch_toggled: false,

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
            prev_sprint_btn: false,
            prev_sneak_btn: false,
            prev_l1_btn: false,
            prev_r1_btn: false,
            prev_key_digits: [false; 10],
            chat: crate::gameplay::chat::ChatState::new(),
            prev_t_down: false,
            prev_slash_down: false,
            prev_b_down: false,
            prev_u_down: false,
            prev_space_down: false,
            controls: ControlsManager::new(),
            minimap: crate::ui::minimap::MinimapState::new(1),
            worldmap: crate::ui::minimap::WorldMapState::new(start_player_pos),
        };
        crate::log_info!("INIT", "RustCraftApp initialized ({}x{}, renderer: {}, seed: {})", width, height, renderer_choice, seed);
        app
    }


    pub fn pixel_buffer(&self) -> &[u32] {
        self.renderer.pixel_buffer()
    }

    pub fn pixel_buffer_mut(&mut self) -> &mut [u32] {
        self.renderer.pixel_buffer_mut()
    }

    /// Alias for PS Vita – full update of game logic and software state.
    /// On Vita the software framebuffer draw is bypassed in favor of native GXM GPU rendering.
    #[cfg(target_os = "vita")]
    pub fn update(&mut self, dt: f32, input: &crate::app::InputState) -> AppOutput {
        self.update_and_render(dt, input)
    }

    pub fn update_and_render(&mut self, dt: f32, input: &InputState) -> AppOutput {
        let mut bot_input = input.clone();
        if let Some(mut runner) = self.autotest.take() {
            runner.update(self, dt, &mut bot_input);
            self.autotest = Some(runner);
        }
        let input = &bot_input;

        self.total_time += dt;
        self.renderer.total_time = self.total_time;
        self.renderer.chunk_anim = self.settings.chunk_anim;
        self.hand_animation = (self.hand_animation - dt * 3.5).max(0.0);
        self.profiler_copied_timer = (self.profiler_copied_timer - dt).max(0.0);
        self.controls.update(input);

        let mut out = AppOutput {
            request_capture_mouse: false,
            request_release_mouse: false,
            quit_requested: false,
            is_playing: matches!(self.state, GameState::Playing),
            title: String::new(),
            rumble_request: None,
            cursor_visible: None,
        };

        // Active input device detection (Gamepad vs Keyboard/Mouse)
        let mouse_moved = match (input.mouse_pos, self.last_mouse_pos) {
            (Some((mx, my)), Some((lx, ly))) => (mx - lx).abs() > 1.5 || (my - ly).abs() > 1.5,
            (Some(_), None) => true,
            _ => false,
        };
        self.last_mouse_pos = input.mouse_pos;

        let kb_mouse_activity = mouse_moved
            || input.mouse_left
            || input.mouse_right
            || input.mouse_dx.abs() > 1.0
            || input.mouse_dy.abs() > 1.0
            || input.scroll_delta.abs() > 0.1
            || input.key_w || input.key_s || input.key_a || input.key_d
            || input.key_space || input.key_shift || input.key_ctrl
            || input.key_escape || input.key_e || input.key_q
            || input.key_enter || input.key_up || input.key_down || input.key_left || input.key_right
            || !input.typed_chars.is_empty();

        let gp_stick_moved = input.gp_left_stick_x.abs() > 0.25
            || input.gp_left_stick_y.abs() > 0.25
            || input.gp_right_stick_x.abs() > 0.25
            || input.gp_right_stick_y.abs() > 0.25
            || input.gp_left_trigger > 0.25
            || input.gp_right_trigger > 0.25;

        let gp_activity = input.gp_connected && (
            gp_stick_moved
            || input.gp_btn_south || input.gp_btn_east || input.gp_btn_west || input.gp_btn_north
            || input.gp_btn_lb || input.gp_btn_rb || input.gp_btn_l3 || input.gp_btn_r3
            || input.gp_btn_start || input.gp_btn_select
            || input.gp_dpad_up || input.gp_dpad_down || input.gp_dpad_left || input.gp_dpad_right
            || input.gp_south_just_pressed || input.gp_east_just_pressed || input.gp_west_just_pressed || input.gp_north_just_pressed
            || input.gp_lb_just_pressed || input.gp_rb_just_pressed || input.gp_r3_just_pressed
            || input.gp_start_just_pressed || input.gp_select_just_pressed
            || input.gp_dpad_up_just_pressed || input.gp_dpad_down_just_pressed || input.gp_dpad_left_just_pressed || input.gp_dpad_right_just_pressed
            || input.gp_menu_up || input.gp_menu_down || input.gp_menu_left || input.gp_menu_right
            || input.gp_right_trigger_just_pressed || input.gp_left_trigger_just_pressed
        );

        if gp_activity {
            self.is_gamepad_active = true;
        } else if kb_mouse_activity || !input.gp_connected {
            self.is_gamepad_active = false;
        }

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
        let t_down = input.key_t;
        let slash_down = input.key_slash;
        let b_down = input.key_b;
        let u_down = input.key_u;
        let space_down = input.key_space;

        match self.state {
            GameState::MainMenu => self.update_and_render_main_menu(input, mouse_moved, &mut out),
            GameState::MultiplayerMenu => self.update_and_render_multiplayer_menu(dt, input, &mut out),
            GameState::EditNickname(prev) => self.update_and_render_nickname_dialog(input, prev, &mut out),
            GameState::MultiplayerConnecting { .. } => self.update_and_render_multiplayer_connecting(dt, input, &mut out),
            GameState::SaveSelect => self.update_and_render_save_select(dt, input, mouse_moved, &mut out),
            GameState::CreateWorld => self.update_and_render_create_world(input, mouse_moved, &mut out),
            GameState::Settings(prev) => self.update_and_render_settings(input, prev, mouse_moved, &mut out),
            GameState::ControlsRebind(prev) => self.update_and_render_controls_rebind(prev, dt, input, mouse_moved, &mut out),
            #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
            GameState::Loading(ref rx) => {
                let res = rx.try_recv().ok();
                self.update_and_render_loading(res);
            }
            #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
            GameState::Loading => self.update_and_render_loading(),
            GameState::Playing => self.update_and_render_playing(dt, input, &mut out),
            GameState::Paused => self.update_and_render_paused(dt, input, mouse_moved, &mut out),
            GameState::Inventory => self.update_and_render_inventory(dt, input, mouse_moved, &mut out),
            GameState::Chest { pos } => self.update_and_render_chest(dt, input, pos, &mut out),
            GameState::Furnace { pos } => self.update_and_render_furnace(dt, input, pos, &mut out),
            GameState::DeathScreen => self.update_and_render_death_screen(input, mouse_moved, &mut out),
            GameState::WorldMap => self.update_and_render_worldmap(dt, input, mouse_moved, &mut out),
        }

        // Context-aware soundtrack update (Creative vs Survival vs Menu)
        let soundtrack_category = match self.state {
            GameState::Playing
            | GameState::Paused
            | GameState::Inventory
            | GameState::Chest { .. }
            | GameState::Furnace { .. }
            | GameState::DeathScreen
            | GameState::WorldMap => {
                match self.player.gamemode {
                    GameMode::Creative => crate::audio::SoundtrackCategory::Creative,
                    GameMode::Survival => crate::audio::SoundtrackCategory::Survival,
                }
            }
            _ => crate::audio::SoundtrackCategory::Menu,
        };

        self.soundtrack.update(
            dt,
            soundtrack_category,
            &mut self.audio,
            self.settings.music_volume,
            self.settings.mute_audio,
        );

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
        self.prev_t_down = t_down;
        self.prev_slash_down = slash_down;
        self.prev_b_down = b_down;
        self.prev_u_down = u_down;
        self.prev_space_down = space_down;

        out.title = format!(
            "RustCraft [{} renderer] - {} FPS - block: {}{}{}",
            self.renderer.backend_name(),
            self.fps_display,
            BlockId::from_u8(self.renderer.selected_block()).name(),
            if self.fly_mode { " - [FLYING]" } else { "" },
            if matches!(self.state, GameState::Paused) { " - [PAUSED]" } else { "" }
        );

        if let Some(ref server) = self.profiler_server {
            if server.is_screenshot_requested() {
                let bmp_bytes = crate::core::profiler_server::encode_bmp(
                    self.renderer.pixel_buffer(),
                    self.width,
                    self.height,
                );
                let bpos = self.camera.position.floor_to_ivec();
                let biome = self.world.biome_name_at(bpos.x, bpos.z).to_string();
                let game_state_str = match &self.state {
                    GameState::MainMenu => "Main Menu",
                    GameState::SaveSelect => "Save Select",
                    GameState::CreateWorld => "Create World",
                    GameState::Settings(_) => "Settings",
                    GameState::ControlsRebind(_) => "Controls Rebind",
                    GameState::EditNickname(_) => "Change Nickname",
                    #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
                    GameState::Loading(_) => "Loading",
                    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
                    GameState::Loading => "Loading",
                    GameState::Playing => "Playing",
                    GameState::Paused => "Paused",
                    GameState::Inventory => "Inventory",
                    GameState::Chest { .. } => "Chest",
                    GameState::Furnace { .. } => "Furnace",
                    GameState::DeathScreen => "Death Screen",
                    GameState::WorldMap => "World Map",
                    GameState::MultiplayerMenu => "Multiplayer Menu",
                    GameState::MultiplayerConnecting { .. } => "Multiplayer Connecting",
                };
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);

                server.set_screenshot(crate::core::profiler_server::ScreenshotData {
                    bmp_bytes,
                    width: self.width,
                    height: self.height,
                    timestamp_ms: now_ms,
                    fps: self.fps_display,
                    player_pos: [
                        self.camera.position.x,
                        self.camera.position.y,
                        self.camera.position.z,
                    ],
                    biome,
                    game_state: game_state_str.to_string(),
                    gamemode: match self.player.gamemode {
                        GameMode::Survival => "Survival".to_string(),
                        GameMode::Creative => "Creative".to_string(),
                    },
                });
            }
        }

        if let Some(ref runner) = self.autotest {
            runner.render_overlay(self.renderer.pixel_buffer_mut(), self.width, self.height);
        }

        // Draw Now-Playing soundtrack toast in bottom-right corner (with progress bar on pause menu)
        let is_paused = matches!(self.state, GameState::Paused);
        self.soundtrack.draw_toast(self.renderer.pixel_buffer_mut(), self.width, self.height, is_paused, &self.audio);

        out.rumble_request = self.pending_rumble.take();
        if !input.is_mouse_captured && !matches!(self.state, GameState::Playing) {
            out.cursor_visible = Some(!self.is_gamepad_active);
        }

        out
    }



    pub fn resize(&mut self, width: usize, height: usize) {
        if width == 0 || height == 0 || (self.width == width && self.height == height) {
            return;
        }
        self.width = width;
        self.height = height;
        self.renderer.resize(width, height);

        self.main_menu_rects = self.renderer.main_menu_button_rects();
        self.main_menu_nick_rect = self.renderer.main_menu_nickname_rect();
        self.nickname_dialog_rects = self.renderer.nickname_dialog_rects();
        self.multiplayer_menu_rects = self.renderer.multiplayer_menu_rects();
        self.save_card_rects = self.renderer.save_card_rects();
        self.save_btn_rects = self.renderer.save_bottom_button_rects();
        self.create_world_rects = self.renderer.create_world_rects();
        self.settings_tab_rects = self.renderer.settings_tab_rects();
        self.settings_rects = self.renderer.settings_graphics_rects();
        self.settings_audio_rects = self.renderer.settings_audio_rects();
        self.settings_controls_rects = self.renderer.settings_controls_rects();
        self.settings_debug_rects = self.renderer.settings_debug_rects();
        self.pause_menu_rects = self.renderer.pause_menu_button_rects();
        self.death_screen_rects = self.renderer.death_screen_button_rects();

        let aspect = width as f32 / height as f32;
        self.last_aspect = aspect;
        self.proj_matrix = self.camera.projection_matrix(aspect);
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
        self.resize(width, height);
    }

    fn pixel_buffer(&self) -> &[u32] {
        self.pixel_buffer()
    }
}


