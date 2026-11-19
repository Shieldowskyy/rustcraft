//! Inicjalizacja instancji `RustCraftApp::new`: konfiguracja, mody, renderer, świat, audio, soundtrack i układy menu.

use crate::core::controls::ControlsManager;
use crate::core::math::Vec3;
use crate::core::profiler::FrameProfiler;
use crate::core::profiler_server::ProfilerServer;
use crate::core::settings::GameSettings;
use crate::features::audio::{
    take_preloaded_manager, AmbientContext, AudioEngine, SoundtrackCategory, SoundtrackManager,
};
use crate::features::autotest::AutoTestRunner;
use crate::features::entities::{MobManager, ParticleSystem};
use crate::features::modding::ModManager;
use crate::features::net::LanDiscoveryClient;
use crate::features::physics::{STANDING_EYE_OFFSET, STANDING_HEIGHT};
use crate::features::player::{
    AchievementProgress, Camera, GameMode, Player, QuestProgress, RecipeBook,
};
use crate::features::rendering::software::Renderer;
use crate::features::ui::chat::ChatState;
use crate::features::ui::dev_logo::DevLogo;
use crate::features::ui::minimap::{MinimapState, WorldMapState};
use crate::features::ui::splashes::SPLASH_TEXTS;
use crate::features::world::{get_biome_at, get_save_metadata, World, WorldGeneratorType};
use crate::platform::{capabilities, storage::soundtracks_dir, system::vita_debug_log};

use super::{CameraPerspective, GameState, RustCraftApp};

impl RustCraftApp {
    pub fn new(width: usize, height: usize, renderer_choice: &str, seed: u32) -> Self {
        let caps = capabilities();
        vita_debug_log("[App::new 1/6] Loading settings, mods and language...");
        crate::log_info!(
            "APP",
            "[Step 1/5] Loading game configuration, mods and language..."
        );
        let mut settings = GameSettings::load();
        settings.apply_ram_limit();
        crate::core::i18n::set_language(settings.language);

        let mut recipe_book = RecipeBook::new();
        ModManager::load_mods(&mut recipe_book);

        vita_debug_log(&format!(
            "[App::new 2/6] Initializing software/2D renderer ({}x{})...",
            width, height
        ));
        crate::log_info!(
            "APP",
            "[Step 2/5] Initializing renderer ({})...",
            renderer_choice
        );
        let mut renderer = match renderer_choice {
            "vulkan" if caps.is_desktop => {
                Renderer::new_vulkan(width, height).unwrap_or_else(|error| {
                    eprintln!(
                        "Failed to initialize Vulkan renderer: {error}, falling back to software renderer"
                    );
                    Renderer::new(width, height)
                })
            }
            "opengl" | "gl" if caps.supports_wgpu => {
                Renderer::new_opengl(width, height).unwrap_or_else(|error| {
                    eprintln!(
                        "Failed to initialize OpenGL renderer: {error}, falling back to software renderer"
                    );
                    Renderer::new(width, height)
                })
            }
            _ => Renderer::new(width, height),
        };
        renderer.fog_enabled = settings.fog;
        renderer.render_distance = settings.render_distance;
        if !settings.fancy_leaves {
            renderer.update_textures(false);
        }

        vita_debug_log(&format!(
            "[App::new 3/6] Initializing World::new_with_seed({})...",
            seed
        ));
        crate::log_info!(
            "APP",
            "[Step 3/5] Initializing world instance (seed: {})...",
            seed
        );
        let world = World::new_with_seed(seed);
        let start_player_pos = world.player_position;
        let (initial_biome, _) = get_biome_at(
            start_player_pos.x.floor() as i32,
            start_player_pos.z.floor() as i32,
            world.seed,
        );
        let initial_biome_tint = initial_biome.color_filter();

        vita_debug_log("[App::new 4/6] Initializing camera, player & UI layout...");
        crate::log_info!(
            "APP",
            "[Step 4/5] Initializing camera, player & UI layout..."
        );
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
            ProfilerServer::start(settings.profiler_server_port).ok()
        } else {
            None
        };

        let aspect = width as f32 / height as f32;
        let last_fov = camera.fov_y;
        let proj_matrix = camera.projection_matrix(aspect);

        crate::log_info!("APP", "[Step 5/5] Checking save slots cache...");
        let save_metadata_cache = std::array::from_fn(|index| get_save_metadata(index as u8 + 1));
        crate::log_info!("APP", "RustCraftApp state initialization complete.");

        let autotest_enabled = crate::cli::parse_autotest_enabled();
        let (initial_state, autotest) = if autotest_enabled {
            crate::log_info!(
                "AUTOTEST",
                "AutoTest Mode ENABLED: auto-starting automated bot test sequence in active world"
            );
            (GameState::Playing, Some(AutoTestRunner::new()))
        } else {
            (GameState::MainMenu, None)
        };

        vita_debug_log("[App::new 5/6] Initializing AudioEngine...");
        let mut audio = AudioEngine::new();
        audio.set_master_volume(settings.master_volume);
        audio.set_sfx_volume(settings.sfx_volume);
        audio.set_music_volume(settings.music_volume);
        audio.set_ambient_volume(settings.ambient_volume);
        audio.set_ui_volume(settings.ui_volume);
        if settings.mute_audio {
            audio.toggle_mute();
        }

        vita_debug_log("[App::new 6/6] Initializing SoundtrackManager...");
        let mut soundtrack = take_preloaded_manager().unwrap_or_else(|| {
            let mut m = SoundtrackManager::new(soundtracks_dir());
            // DLACZEGO: Na PS Vita pomijamy synchroniczne dekodowanie pełnego utworu PCM podczas startu,
            // aby nie zużywać ciągłego obszaru sterty Newlib przed załadowaniem tekstur i świata.
            if !caps.is_vita {
                m.preload_intro(SoundtrackCategory::Menu);
            }
            m
        });
        let dev_logo = if autotest_enabled || cfg!(test) {
            None
        } else {
            DevLogo::load()
        };
        let intro_allowed =
            !settings.mute_audio && settings.music_volume > 0.001 && !autotest_enabled;
        if !intro_allowed {
            soundtrack.pending_intro = None;
        } else if dev_logo.is_none() {
            soundtrack.start_intro(&mut audio);
            soundtrack.show_current_toast();
        }

        let app = Self {
            width,
            height,
            renderer,
            world,
            last_effective_camera: camera.clone(),
            camera,
            particles: ParticleSystem::new(),
            mob_manager: MobManager::new(),
            player,
            items: Vec::new(),
            arrows: Vec::new(),
            fishing_bobber: None,
            recipe_book,
            crafting_table_open: false,
            recipe_book_open: false,
            recipe_book_scroll: 0,
            recipe_book_status: None,
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
            jump_double_tap_timer: 0.0,
            current_eye_offset: STANDING_EYE_OFFSET,
            current_player_height: STANDING_HEIGHT,
            break_progress: 0.0,
            break_target: None,
            dig_sound_timer: 0.0,
            creative_break_cooldown: 0.0,
            total_time: 0.0,
            material_timer: 0.0,
            hand_animation: 0.0,
            walk_phase: 0.0,
            damage_tilt_timer: 0.0,
            damage_tilt_dir: 1.0,
            lava_damage_timer: 0.45,
            drown_damage_timer: 0.0,
            biome_filter_tint: initial_biome_tint,
            debug_lines: (0..10).map(|_| String::with_capacity(96)).collect(),
            debug_right_lines: (0..14).map(|_| String::with_capacity(96)).collect(),
            profiler: FrameProfiler::default(),
            profiler_enabled: cfg!(feature = "profiler"),
            debug_timer: 0.15,
            cpu_time_ms: 16.6,
            gpu_triangles: 0,
            cdram_mb: 0.0,
            ram_mb: 0.0,
            chunk_slots_info: (0, 96),
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
            multiplayer_toast: None,
            achievement_toast: None,
            pending_achievement_toasts: std::collections::VecDeque::new(),
            achievements: AchievementProgress::default(),
            field_guide_tab: 0,
            field_guide_scroll: 0,
            quests: QuestProgress::default(),
            questbook_tab: 0,
            questbook_selected: 0,
            questbook_scroll: 0,
            quest_toast: None,
            pending_quest_toasts: std::collections::VecDeque::new(),
            lan_discovery: LanDiscoveryClient::new(),
            selected_lan_server: None,
            main_menu_nick_hovered: false,
            net_client: None,
            pending_network_events: Vec::new(),
            is_multiplayer: false,
            is_hosting_multiplayer: false,
            death_message_sent: false,
            multiplayer_invite: None,
            embedded_server: None,
            relay_host: None,
            save_card_rects,
            save_btn_rects,
            create_world_rects,
            create_world_generator_type: WorldGeneratorType::Default,
            create_world_quests_enabled: true,
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
            cached_ambient_ctx: AmbientContext::default(),
            sprint_toggled: false,
            crouch_toggled: false,
            prev_left_down: false,
            prev_right_down: false,
            prev_middle_down: false,
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
            chat: ChatState::new(),
            prev_t_down: false,
            prev_slash_down: false,
            prev_b_down: false,
            prev_u_down: false,
            prev_space_down: false,
            controls: ControlsManager::new(),
            minimap: MinimapState::new(1, 0),
            worldmap: WorldMapState::new(start_player_pos),
            splash_index: (seed as usize) % SPLASH_TEXTS.len(),
            dev_logo,
            dev_logo_timer: 0.0,
            menu_fade_timer: 0.0,
            pending_gxm_profiler: None,
        };
        vita_debug_log("[App::new DONE] RustCraftApp constructed successfully.");
        crate::log_info!(
            "INIT",
            "RustCraftApp initialized ({}x{}, renderer: {}, seed: {})",
            width,
            height,
            renderer_choice,
            seed
        );
        app
    }
}
