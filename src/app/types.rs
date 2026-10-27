#[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
use std::sync::mpsc;

use crate::core::math::{IVec3, Vec3};
use crate::world::World;

/// Default framebuffer render width.
pub const WIDTH: usize = 960;

/// Default framebuffer render height.
pub const HEIGHT: usize = 540;

/// Maximum number of selectable world save slots.
pub const MAX_SAVE_SLOTS: u8 = 32;

/// Originating menu state for submenus (e.g. Settings).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviousMenu {
    /// Settings opened from the main menu.
    MainMenu,
    /// Settings opened from the in-game pause menu.
    Paused,
    /// Opened from settings menu.
    Settings,
    /// Opened from multiplayer menu.
    MultiplayerMenu,
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
    /// Change nickname modal dialogue.
    EditNickname(PreviousMenu),
    /// Asynchronous world generation / loading screen.
    #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360")))]
    Loading(mpsc::Receiver<WorldLoadResult>),
    /// Synchronous world loading state for embedded platforms.
    #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", target_os = "xbox360", target_os = "xenon", feature = "xbox360"))]
    Loading,
    /// Active 3D gameplay.
    Playing,
    /// In-game pause menu overlay.
    Paused,
    /// Player inventory & crafting screen.
    Inventory,
    /// Chest container UI.
    Chest { pos: IVec3 },
    /// Furnace container UI.
    Furnace { pos: IVec3 },
    /// Player death screen with respawn and quit options.
    DeathScreen,
    /// Multiplayer server address / connection screen.
    MultiplayerMenu,
    /// Actively connecting to a multiplayer server.
    MultiplayerConnecting {
        server_addr: String,
        start_time: std::time::Instant,
    },
    /// Interactive fullscreen WorldMap viewer.
    WorldMap,
}

/// Camera perspective modes matching Minecraft F5 cycling.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CameraPerspective {
    /// Standard first-person view.
    FirstPerson,
    /// Third-person camera positioned behind player looking forward.
    ThirdPersonBack,
    /// Third-person camera positioned in front of player looking at player's face.
    ThirdPersonFront,
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
    pub key_f3: bool,
    pub key_f5: bool,
    pub key_enter: bool,
    pub key_up: bool,
    pub key_down: bool,
    pub key_left: bool,
    pub key_right: bool,
    pub key_n: bool,
    pub key_b: bool,
    pub key_u: bool,
    pub key_del: bool,
    pub key_backspace: bool,
    pub key_tab: bool,
    pub key_t: bool,
    pub key_slash: bool,
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
    pub gp_menu_up: bool,
    pub gp_menu_down: bool,
    pub gp_menu_left: bool,
    pub gp_menu_right: bool,
    pub gp_right_trigger_just_pressed: bool,
    pub gp_left_trigger_just_pressed: bool,

    pub is_mouse_captured: bool,
    pub gp_connected: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct RumbleRequest {
    pub strong: f32,
    pub weak: f32,
    pub duration_ms: u32,
}

pub struct AppOutput {
    pub request_capture_mouse: bool,
    pub request_release_mouse: bool,
    pub quit_requested: bool,
    pub is_playing: bool,
    pub title: String,
    pub rumble_request: Option<RumbleRequest>,
    pub cursor_visible: Option<bool>,
}
