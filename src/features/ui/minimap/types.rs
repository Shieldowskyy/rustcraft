//! Struktury stanu minimapy, pełnoekranowej mapy świata oraz znaczników (waypoints).

use std::cell::RefCell;
use std::collections::HashSet;

use crate::core::math::Vec3;

use super::storage::load_waypoints;

pub const WAYPOINT_COLORS: [(u32, &'static str); 8] = [
    (0xFFFF3333, "Red"),
    (0xFF33FF33, "Green"),
    (0xFF3388FF, "Blue"),
    (0xFFFFFF33, "Yellow"),
    (0xFFCC33FF, "Purple"),
    (0xFF33FFFF, "Cyan"),
    (0xFFFF9922, "Orange"),
    (0xFFFFFFFF, "White"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct Waypoint {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub color: u32,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinimapMode {
    Circle,
    Square,
    Hidden,
}

impl MinimapMode {
    pub fn next(&self) -> Self {
        match self {
            MinimapMode::Circle => MinimapMode::Square,
            MinimapMode::Square => MinimapMode::Hidden,
            MinimapMode::Hidden => MinimapMode::Circle,
        }
    }
}

#[derive(Default)]
pub(super) struct MinimapTerrainCache {
    pub pixels: Vec<u32>,
    pub center_block: (i32, i32),
    pub center_world_pos: (f32, f32),
    pub zoom_bits: u32,
    pub mode: Option<MinimapMode>,
    pub frames_since_refresh: u32,
    pub last_player_pos: Option<(f32, f32)>,
    pub stationary_frames: u32,
}

pub struct MinimapState {
    pub mode: MinimapMode,
    pub zoom: f32,
    pub waypoints: Vec<Waypoint>,
    pub(super) terrain_cache: RefCell<MinimapTerrainCache>,
}

impl MinimapState {
    pub fn new(slot: u8, seed: u32) -> Self {
        Self {
            mode: MinimapMode::Circle,
            zoom: 1.0,
            waypoints: load_waypoints(slot, seed),
            terrain_cache: RefCell::new(MinimapTerrainCache::default()),
        }
    }

    #[inline]
    pub fn invalidate_cache(&self) {
        self.terrain_cache.borrow_mut().mode = None;
    }
}

#[derive(Clone, Debug)]
pub struct WorldMapContextMenu {
    pub screen_pos: (i32, i32),
    pub world_pos: (i32, i32, i32),
    pub chunk: (i32, i32),
}

pub struct WorldMapState {
    pub center_x: f32,
    pub center_z: f32,
    pub zoom: f32,
    pub is_dragging: bool,
    pub drag_start_mouse: (f32, f32),
    pub drag_start_center: (f32, f32),
    pub creating_waypoint: bool,
    pub waypoint_name_input: String,
    pub waypoint_coords_input: (i32, i32, i32),
    pub waypoint_color_idx: usize,
    pub hovered_world_pos: Option<(i32, i32, i32)>,
    pub selected_chunks: HashSet<(i32, i32)>,
    pub hovered_chunk: Option<(i32, i32)>,
    pub showing_waypoint_manager: bool,
    pub waypoint_manager_scroll: usize,
    pub context_menu: Option<WorldMapContextMenu>,
    pub map_cache: Vec<u32>,
    pub cache_center: (f32, f32),
    pub cache_zoom: f32,
    pub cache_size: (usize, usize),
    pub cache_valid: bool,
}

impl WorldMapState {
    pub fn new(player_pos: Vec3) -> Self {
        Self {
            center_x: player_pos.x,
            center_z: player_pos.z,
            zoom: 2.0,
            is_dragging: false,
            drag_start_mouse: (0.0, 0.0),
            drag_start_center: (player_pos.x, player_pos.z),
            creating_waypoint: false,
            waypoint_name_input: String::from("Waypoint"),
            waypoint_coords_input: (
                player_pos.x.round() as i32,
                player_pos.y.round() as i32,
                player_pos.z.round() as i32,
            ),
            waypoint_color_idx: 0,
            hovered_world_pos: None,
            selected_chunks: HashSet::new(),
            hovered_chunk: None,
            showing_waypoint_manager: false,
            waypoint_manager_scroll: 0,
            context_menu: None,
            map_cache: Vec::new(),
            cache_center: (f32::NAN, f32::NAN),
            cache_zoom: 0.0,
            cache_size: (0, 0),
            cache_valid: false,
        }
    }

    #[inline]
    pub fn invalidate_cache(&mut self) {
        self.cache_valid = false;
    }
}
