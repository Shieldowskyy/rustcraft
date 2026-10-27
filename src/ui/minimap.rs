//! Full implementation of Minimap + Worldmap.
//!
//! Features:
//! - Glassmorphic circular/square minimap with hillshading relief elevation.
//! - Live entity radar blips (hostile in bright red, passive in bright yellow/green).
//! - Cardinal direction markers (N, S, E, W).
//! - Centered player chevron indicator oriented to player yaw.
//! - Waypoints with customizable colors, in-world indicators, distance clamping, and disk persistence.
//! - Fullscreen interactive WorldMap with panning, zooming, coordinate inspection, biome sampling, and waypoint creation.

use std::collections::HashSet;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita"), not(target_arch = "powerpc"), not(feature = "xbox360")))]
use rayon::prelude::*;

use crate::core::math::Vec3;
use crate::gameplay::mob::Mob;
use crate::graphics::camera::Camera;
use crate::ui::draw::{
    draw_text_colored_shadowed, draw_text_light_colored_shadowed, draw_text_light_shadowed,
    fill_rect, fill_rect_alpha, pack_color, set_px,
};
use crate::world::block::{BlockId, Face};
use crate::world::{get_biome_at, World, CHUNK_SIZE_Y};

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

pub struct MinimapState {
    pub mode: MinimapMode,
    pub zoom: f32, // Blocks per radar pixel (1.0 = standard)
    pub waypoints: Vec<Waypoint>,
}

impl MinimapState {
    pub fn new(slot: u8) -> Self {
        Self {
            mode: MinimapMode::Circle,
            zoom: 1.0,
            waypoints: load_waypoints(slot),
        }
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
    pub zoom: f32, // Pixels per block (1.0 to 6.0)
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
            waypoint_coords_input: (player_pos.x.round() as i32, player_pos.y.round() as i32, player_pos.z.round() as i32),
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

fn waypoints_file_path(slot: u8) -> std::path::PathBuf {
    let save_file = crate::world::storage::save_path(slot);
    let p = std::path::Path::new(&save_file);
    if let Some(parent) = p.parent() {
        parent.join(format!("slot_{}_waypoints.txt", slot))
    } else {
        std::path::PathBuf::from(format!("slot_{}_waypoints.txt", slot))
    }
}

pub fn serialize_waypoints(waypoints: &[Waypoint]) -> String {
    let mut out = String::new();
    for wp in waypoints {
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|{:08X}\n",
            if wp.enabled { "1" } else { "0" },
            wp.name,
            wp.x,
            wp.y,
            wp.z,
            wp.color
        ));
    }
    out
}

pub fn parse_waypoints_str(content: &str) -> Vec<Waypoint> {
    let mut waypoints = Vec::new();
    for line in content.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 6 {
            let enabled = parts[0] == "1";
            let name = parts[1].to_string();
            if let (Ok(x), Ok(y), Ok(z), Ok(color)) = (
                parts[2].parse::<i32>(),
                parts[3].parse::<i32>(),
                parts[4].parse::<i32>(),
                u32::from_str_radix(parts[5].trim_start_matches("0x"), 16),
            ) {
                waypoints.push(Waypoint {
                    name,
                    x,
                    y,
                    z,
                    color: if color <= 0x00FFFFFF { 0xFF000000 | color } else { color },
                    enabled,
                });
            }
        }
    }
    waypoints
}

pub fn load_waypoints(slot: u8) -> Vec<Waypoint> {
    let path = waypoints_file_path(slot);
    if let Ok(content) = std::fs::read_to_string(path) {
        parse_waypoints_str(&content)
    } else {
        Vec::new()
    }
}

pub fn save_waypoints(slot: u8, waypoints: &[Waypoint]) {
    let path = waypoints_file_path(slot);
    let out = serialize_waypoints(waypoints);
    let _ = std::fs::write(path, out);
}

/// Computes surface height Y and top block at (wx, wz).
#[inline(always)]
pub fn sample_surface(world: &World, wx: i32, wz: i32) -> (i32, BlockId) {
    let cx = wx >> 4;
    let cz = wz >> 4;
    let lx = wx.rem_euclid(16);
    let lz = wz.rem_euclid(16);

    if let Some(chunk) = world.chunks.get(&(cx, cz)) {
        for y in (0..CHUNK_SIZE_Y).rev() {
            let blk = chunk.get_local(lx, y, lz);
            if blk != BlockId::Air {
                return (y, blk);
            }
        }
    }
    (0, BlockId::Air)
}

#[inline(always)]
pub fn sample_chunk_column(chunk: &crate::world::Chunk, lx: i32, lz: i32) -> (i32, BlockId) {
    for y in (0..CHUNK_SIZE_Y).rev() {
        let blk = chunk.get_local(lx, y, lz);
        if blk != BlockId::Air {
            return (y, blk);
        }
    }
    (0, BlockId::Air)
}

#[inline(always)]
pub fn sample_chunk_column_y(chunk: &crate::world::Chunk, lx: i32, lz: i32) -> i32 {
    for y in (0..CHUNK_SIZE_Y).rev() {
        let blk = chunk.get_local(lx, y, lz);
        if blk != BlockId::Air {
            return y;
        }
    }
    0
}

#[inline(always)]
pub fn fast_calculate_map_color(y: i32, blk: BlockId, y_north: i32) -> u32 {
    if blk == BlockId::Air {
        return pack_color(0.08, 0.08, 0.12);
    }

    let diff = (y - y_north).clamp(-4, 4);
    let shade = (1.0 + diff as f32 * 0.10).clamp(0.45, 1.45);

    if blk == BlockId::Water {
        let depth = ((40 - y).max(1) as f32 * 0.04).clamp(0.0, 0.45);
        let r = (0.10 - depth * 0.15).max(0.02) * shade;
        let g = (0.35 - depth * 0.25).max(0.08) * shade;
        let b = (0.75 - depth * 0.20).max(0.20) * shade;
        pack_color(r, g, b)
    } else {
        let base = blk.face_color(Face::Top);
        pack_color(
            (base[0] * shade).clamp(0.0, 1.0),
            (base[1] * shade).clamp(0.0, 1.0),
            (base[2] * shade).clamp(0.0, 1.0),
        )
    }
}

pub fn draw_clipped_rect_outline(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: u32,
    min_clip_y: i32,
    max_clip_y: i32,
) {
    if w <= 0 || h <= 0 {
        return;
    }
    let x0 = x;
    let x1 = x + w;
    let y0 = y;
    let y1 = y + h;

    // Top horizontal edge
    if y0 >= min_clip_y && y0 < max_clip_y && y0 >= 0 && y0 < buf_h as i32 {
        let left = x0.max(0);
        let right = x1.min(buf_w as i32);
        if left < right {
            let row_start = y0 as usize * buf_w;
            for xx in left..right {
                buffer[row_start + xx as usize] = color;
            }
        }
    }
    // Bottom horizontal edge
    let by = y1 - 1;
    if by >= min_clip_y && by < max_clip_y && by >= 0 && by < buf_h as i32 {
        let left = x0.max(0);
        let right = x1.min(buf_w as i32);
        if left < right {
            let row_start = by as usize * buf_w;
            for xx in left..right {
                buffer[row_start + xx as usize] = color;
            }
        }
    }
    // Left vertical edge
    if x0 >= 0 && x0 < buf_w as i32 {
        let top = y0.max(min_clip_y).max(0);
        let bottom = y1.min(max_clip_y).min(buf_h as i32);
        for yy in top..bottom {
            buffer[yy as usize * buf_w + x0 as usize] = color;
        }
    }
    // Right vertical edge
    let rx = x1 - 1;
    if rx >= 0 && rx < buf_w as i32 {
        let top = y0.max(min_clip_y).max(0);
        let bottom = y1.min(max_clip_y).min(buf_h as i32);
        for yy in top..bottom {
            buffer[yy as usize * buf_w + rx as usize] = color;
        }
    }
}

/// Hillshading and water tinting based on surface height and neighbor heights.
#[inline(always)]
fn calculate_map_color(world: &World, wx: i32, wz: i32, y: i32, blk: BlockId) -> u32 {
    if blk == BlockId::Air {
        return pack_color(0.08, 0.08, 0.12);
    }

    let (y_north, _) = sample_surface(world, wx, wz - 1);
    let diff = (y - y_north).clamp(-4, 4);
    let shade = (1.0 + diff as f32 * 0.10).clamp(0.45, 1.45);

    if blk == BlockId::Water {
        // Depth-based ocean blue
        let depth = ((40 - y).max(1) as f32 * 0.04).clamp(0.0, 0.45);
        let r = (0.10 - depth * 0.15).max(0.02) * shade;
        let g = (0.35 - depth * 0.25).max(0.08) * shade;
        let b = (0.75 - depth * 0.20).max(0.20) * shade;
        pack_color(r, g, b)
    } else {
        let base = blk.face_color(Face::Top);
        pack_color(
            (base[0] * shade).clamp(0.0, 1.0),
            (base[1] * shade).clamp(0.0, 1.0),
            (base[2] * shade).clamp(0.0, 1.0),
        )
    }
}

/// Renders Minimap in the upper-right corner.
pub fn draw_minimap(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    world: &World,
    player_pos: Vec3,
    player_yaw: f32,
    mobs: &[Mob],
    state: &MinimapState,
) {
    if state.mode == MinimapMode::Hidden {
        return;
    }

    let radius = 54i32;
    let cx = buf_w as i32 - 72;
    let cy = 72i32;

    let is_circle = state.mode == MinimapMode::Circle;
    let rad_sq = radius * radius;
    let inner_rad_sq = (radius - 2) * (radius - 2);
    let shadow_rad_sq = (radius + 2) * (radius + 2);

    // Terrain rasterization
    for dy in -radius..=radius {
        let py = cy + dy;
        if py < 0 || py >= buf_h as i32 {
            continue;
        }

        for dx in -radius..=radius {
            let px = cx + dx;
            if px < 0 || px >= buf_w as i32 {
                continue;
            }

            let dist_sq = dx * dx + dy * dy;
            if is_circle && dist_sq > shadow_rad_sq {
                continue;
            }

            let idx = (py as usize) * buf_w + (px as usize);

            // Border & bezel
            if is_circle {
                if dist_sq > rad_sq {
                    // Soft drop shadow
                    let cur = buffer[idx];
                    let r = (((cur >> 16) & 0xFF) * 140) >> 8;
                    let g = (((cur >> 8) & 0xFF) * 140) >> 8;
                    let b = ((cur & 0xFF) * 140) >> 8;
                    buffer[idx] = (0xFF << 24) | (r << 16) | (g << 8) | b;
                    continue;
                } else if dist_sq >= inner_rad_sq {
                    // Glass border ring
                    buffer[idx] = 0xFF2A2A38;
                    continue;
                }
            } else {
                let border = dx == -radius || dx == radius || dy == -radius || dy == radius
                    || dx == -radius + 1 || dx == radius - 1 || dy == -radius + 1 || dy == radius - 1;
                if border {
                    buffer[idx] = 0xFF2A2A38;
                    continue;
                }
            }

            // World sample
            let wx = (player_pos.x + dx as f32 * state.zoom).floor() as i32;
            let wz = (player_pos.z + dy as f32 * state.zoom).floor() as i32;
            let (sy, sblk) = sample_surface(world, wx, wz);
            let color = calculate_map_color(world, wx, wz, sy, sblk);
            buffer[idx] = color;
        }
    }

    // Cardinal directions along the perimeter (North at top)
    let n_y = cy - radius + 6;
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "N", cx - 2, n_y, 1, 1, 0xFFFF4444);
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "S", cx - 2, cy + radius - 12, 1, 1, 0xFFE0E0E0);
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "E", cx + radius - 12, cy - 3, 1, 1, 0xFFE0E0E0);
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "W", cx - radius + 5, cy - 3, 1, 1, 0xFFE0E0E0);

    // Entity radar dots
    for mob in mobs {
        if mob.is_dead {
            continue;
        }
        let m_dx = (mob.pos.x - player_pos.x) / state.zoom;
        let m_dz = (mob.pos.z - player_pos.z) / state.zoom;
        let m_dist_sq = (m_dx * m_dx + m_dz * m_dz) as i32;

        if is_circle && m_dist_sq >= inner_rad_sq {
            continue;
        } else if !is_circle && (m_dx.abs() > (radius - 4) as f32 || m_dz.abs() > (radius - 4) as f32) {
            continue;
        }

        let sx = cx + m_dx as i32;
        let sy = cy + m_dz as i32;

        let dot_color = if mob.kind.is_hostile() {
            0xFFFF2222 // Hostile mob: Red
        } else {
            0xFF44FF44 // Passive animal: Green/Yellow
        };

        // 3x3 dot with outline
        fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, 0xFF000000);
        set_px(buffer, buf_w, buf_h, sx, sy, dot_color);
    }

    // Waypoints
    for wp in &state.waypoints {
        if !wp.enabled {
            continue;
        }
        let w_dx = (wp.x as f32 - player_pos.x) / state.zoom;
        let w_dz = (wp.z as f32 - player_pos.z) / state.zoom;
        let dist = (w_dx * w_dx + w_dz * w_dz).sqrt();

        let max_r = (radius - 7) as f32;
        let (px, py) = if dist > max_r {
            // Clamp to radar rim
            let scale = max_r / dist;
            (cx + (w_dx * scale) as i32, cy + (w_dz * scale) as i32)
        } else {
            (cx + w_dx as i32, cy + w_dz as i32)
        };

        // Diamond pin
        fill_rect(buffer, buf_w, buf_h, px - 1, py - 1, 3, 3, 0xFF000000);
        set_px(buffer, buf_w, buf_h, px, py - 2, wp.color);
        set_px(buffer, buf_w, buf_h, px - 2, py, wp.color);
        set_px(buffer, buf_w, buf_h, px + 2, py, wp.color);
        set_px(buffer, buf_w, buf_h, px, py + 2, wp.color);
        set_px(buffer, buf_w, buf_h, px, py, 0xFFFFFFFF);
    }

    // Centered player chevron / pointer
    draw_player_chevron(buffer, buf_w, buf_h, cx, cy, player_yaw);

    // Coordinates and biome banner beneath minimap
    let info_y = cy + radius + 6;
    let px_i = player_pos.x.floor() as i32;
    let py_i = player_pos.y.floor() as i32;
    let pz_i = player_pos.z.floor() as i32;
    let coord_str = format!("X:{} Y:{} Z:{}", px_i, py_i, pz_i);
    let (biome, _) = get_biome_at(px_i, pz_i, world.seed);
    let biome_str = biome.name();

    let bg_w = 120i32;
    let bg_x = cx - bg_w / 2;
    fill_rect_alpha(buffer, buf_w, buf_h, bg_x, info_y - 2, bg_w, 22, 0xFF000000, 0.45);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &coord_str, bg_x + 4, info_y, 1, 1, 0xFFFFFFFF);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, biome_str, bg_x + 4, info_y + 10, 1, 1, 0xFF77DD77);
}

fn draw_player_chevron(buffer: &mut [u32], buf_w: usize, buf_h: usize, cx: i32, cy: i32, yaw: f32) {
    let sin = yaw.sin();
    let cos = yaw.cos();

    // Triangle vertices relative to center
    let tip = (sin * 5.0, cos * 5.0);
    let left = (-cos * 3.5 - sin * 3.0, sin * 3.5 - cos * 3.0);
    let right = (cos * 3.5 - sin * 3.0, -sin * 3.5 - cos * 3.0);

    // Draw dark outline
    fill_rect(buffer, buf_w, buf_h, cx - 1, cy - 1, 3, 3, 0xFF000000);
    set_px(buffer, buf_w, buf_h, cx + (tip.0 * 1.2) as i32, cy + (tip.1 * 1.2) as i32, 0xFF000000);

    // Draw chevron body
    set_px(buffer, buf_w, buf_h, cx + tip.0 as i32, cy + tip.1 as i32, 0xFFFF2222);
    set_px(buffer, buf_w, buf_h, cx + left.0 as i32, cy + left.1 as i32, 0xFFFFFFFF);
    set_px(buffer, buf_w, buf_h, cx + right.0 as i32, cy + right.1 as i32, 0xFFFFFFFF);
    set_px(buffer, buf_w, buf_h, cx, cy, 0xFFFFFFFF);
}

fn render_worldmap_row(
    world: &World,
    row_pixels: &mut [u32],
    py: i32,
    map_w: i32,
    center_wx: f32,
    center_wz: f32,
    zoom: f32,
    half_w: f32,
    half_h: f32,
) {
    let wz = (center_wz - half_h + (py as f32) / zoom).floor() as i32;
    let cz = wz >> 4;
    let lz = wz.rem_euclid(16);

    let wz_north = wz - 1;
    let cz_north = wz_north >> 4;
    let lz_north = wz_north.rem_euclid(16);

    let mut last_wx = i32::MIN;
    let mut last_col = 0u32;
    let mut last_cx = i32::MIN;
    let mut curr_chunk: Option<&crate::world::Chunk> = None;
    let mut north_chunk: Option<&crate::world::Chunk> = None;

    for px in 0..map_w {
        let wx = (center_wx - half_w + (px as f32) / zoom).floor() as i32;
        if wx == last_wx {
            row_pixels[px as usize] = last_col;
            continue;
        }

        let cx = wx >> 4;
        let lx = wx.rem_euclid(16);

        if cx != last_cx {
            curr_chunk = world.chunks.get(&(cx, cz));
            north_chunk = if cz_north == cz {
                curr_chunk
            } else {
                world.chunks.get(&(cx, cz_north))
            };
            last_cx = cx;
        }

        let (sy, sblk) = if let Some(chunk) = curr_chunk {
            sample_chunk_column(chunk, lx, lz)
        } else {
            (0, BlockId::Air)
        };

        let y_north = if let Some(n_chunk) = north_chunk {
            sample_chunk_column_y(n_chunk, lx, lz_north)
        } else {
            0
        };

        let mut col = fast_calculate_map_color(sy, sblk, y_north);

        // Subtle chunk boundary lines (every 16 blocks)
        if lx == 0 || lz == 0 {
            let r = (((col >> 16) & 0xFF) * 200) >> 8;
            let g = (((col >> 8) & 0xFF) * 200) >> 8;
            let b = ((col & 0xFF) * 200) >> 8;
            col = (0xFF << 24) | (r << 16) | (g << 8) | b;
        }

        row_pixels[px as usize] = col;
        last_wx = wx;
        last_col = col;
    }
}

pub fn worldmap_top_bar_buttons() -> ((i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let waypoints_btn = (175, 4, 105, 20);
    let new_wp_btn = (288, 4, 105, 20);
    let clear_chunks_btn = (401, 4, 120, 20);
    (waypoints_btn, new_wp_btn, clear_chunks_btn)
}

pub fn waypoint_manager_buttons(dx: i32, dy: i32, card_w: i32, card_h: i32) -> ((i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let header_close = (dx + card_w - 30, dy + 10, 20, 20);
    let new_wp_btn = (dx + 20, dy + card_h - 36, 130, 24);
    let bottom_close = (dx + card_w - 120, dy + card_h - 36, 100, 24);
    let scroll_up = (dx + card_w - 28, dy + 48, 18, 18);
    let scroll_down = (dx + card_w - 28, dy + card_h - 70, 18, 18);
    (header_close, new_wp_btn, bottom_close, scroll_up, scroll_down)
}

pub fn waypoint_row_buttons(dx: i32, list_y: i32, card_w: i32, row_idx: usize) -> ((i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let ry = list_y + (row_idx as i32) * 44;
    let rx = dx + 16;
    let rw = card_w - 48;
    let vis_btn = (rx + rw - 135, ry + 8, 38, 22);
    let tp_btn = (rx + rw - 92, ry + 8, 40, 22);
    let del_btn = (rx + rw - 47, ry + 8, 38, 22);
    (vis_btn, tp_btn, del_btn)
}

pub fn draw_waypoint_manager(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    waypoints: &[Waypoint],
    player_pos: Vec3,
    scroll: usize,
) {
    let card_w = 540i32;
    let card_h = 360i32;
    let dx = (buf_w as i32 - card_w) / 2;
    let dy = (buf_h as i32 - card_h) / 2;

    // Dark backdrop
    fill_rect_alpha(buffer, buf_w, buf_h, 0, 0, buf_w as i32, buf_h as i32, 0xFF000000, 0.60);

    // Card background & double outline
    fill_rect(buffer, buf_w, buf_h, dx, dy, card_w, card_h, 0xFF14141E);
    fill_rect(buffer, buf_w, buf_h, dx - 1, dy - 1, card_w + 2, 1, 0xFF3D3D54);
    fill_rect(buffer, buf_w, buf_h, dx - 1, dy + card_h, card_w + 2, 1, 0xFF3D3D54);
    fill_rect(buffer, buf_w, buf_h, dx - 1, dy - 1, 1, card_h + 2, 0xFF3D3D54);
    fill_rect(buffer, buf_w, buf_h, dx + card_w, dy - 1, 1, card_h + 2, 0xFF3D3D54);

    // Header
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "WAYPOINT MANAGER", dx + 20, dy + 14, 1, 1, 0xFFFFDD44);
    let count_str = format!("({} waypoints)", waypoints.len());
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &count_str, dx + 180, dy + 15, 1, 1, 0xFFAAAAAA);

    let (header_close, new_wp_btn, bottom_close, scroll_up, scroll_down) = waypoint_manager_buttons(dx, dy, card_w, card_h);

    // Close X
    fill_rect(buffer, buf_w, buf_h, header_close.0, header_close.1, header_close.2, header_close.3, 0xFF7A2020);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "X", header_close.0 + 6, header_close.1 + 4, 1, 1, 0xFFFFFFFF);

    // Separator line
    fill_rect(buffer, buf_w, buf_h, dx + 12, dy + 38, card_w - 24, 1, 0xFF2A2A3C);

    let list_y = dy + 46;
    let max_rows = 6usize;
    let end_idx = (scroll + max_rows).min(waypoints.len());

    if waypoints.is_empty() {
        draw_text_light_colored_shadowed(
            buffer,
            buf_w,
            buf_h,
            "No waypoints yet. Press [B] or click [+ New Waypoint] to create one.",
            dx + 40,
            dy + 150,
            1,
            1,
            0xFFAAAAAA,
        );
    } else {
        for i in scroll..end_idx {
            let row_idx = i - scroll;
            let ry = list_y + (row_idx as i32) * 44;
            let rx = dx + 16;
            let rw = card_w - 48;
            let wp = &waypoints[i];

            let row_bg = if wp.enabled { 0xFF1C1C29 } else { 0xFF161620 };
            fill_rect(buffer, buf_w, buf_h, rx, ry, rw, 40, row_bg);
            fill_rect(buffer, buf_w, buf_h, rx, ry, rw, 1, 0xFF2C2C3D);
            fill_rect(buffer, buf_w, buf_h, rx, ry + 39, rw, 1, 0xFF2C2C3D);
            fill_rect(buffer, buf_w, buf_h, rx, ry, 1, 40, 0xFF2C2C3D);
            fill_rect(buffer, buf_w, buf_h, rx + rw - 1, ry, 1, 40, 0xFF2C2C3D);

            // Color diamond/swatch
            fill_rect(buffer, buf_w, buf_h, rx + 10, ry + 12, 16, 16, wp.color);
            fill_rect(buffer, buf_w, buf_h, rx + 9, ry + 11, 18, 1, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, rx + 9, ry + 28, 18, 1, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, rx + 9, ry + 11, 1, 18, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, rx + 26, ry + 11, 1, 18, 0xFF000000);

            // Waypoint name
            let name_color = if wp.enabled { 0xFFFFFFFF } else { 0xFF777788 };
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &wp.name, rx + 36, ry + 7, 1, 1, name_color);

            // Position & distance
            let dist = ((wp.x as f32 - player_pos.x).powi(2) + (wp.y as f32 - player_pos.y).powi(2) + (wp.z as f32 - player_pos.z).powi(2)).sqrt();
            let coords_str = format!("X: {} Y: {} Z: {} | {:.0}m", wp.x, wp.y, wp.z, dist);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &coords_str, rx + 36, ry + 22, 1, 1, 0xFF888899);

            // Row buttons
            let (vis_btn, tp_btn, del_btn) = waypoint_row_buttons(dx, list_y, card_w, row_idx);

            // Visibility toggle [VIS] or [HID]
            let vis_bg = if wp.enabled { 0xFF2A602A } else { 0xFF444455 };
            let vis_txt = if wp.enabled { "VIS" } else { "HID" };
            fill_rect(buffer, buf_w, buf_h, vis_btn.0, vis_btn.1, vis_btn.2, vis_btn.3, vis_bg);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, vis_txt, vis_btn.0 + 8, vis_btn.1 + 5, 1, 1, 0xFFFFFFFF);

            // Teleport button [TP]
            fill_rect(buffer, buf_w, buf_h, tp_btn.0, tp_btn.1, tp_btn.2, tp_btn.3, 0xFF204875);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "TP", tp_btn.0 + 11, tp_btn.1 + 5, 1, 1, 0xFFFFFFFF);

            // Delete button [DEL]
            fill_rect(buffer, buf_w, buf_h, del_btn.0, del_btn.1, del_btn.2, del_btn.3, 0xFF7A2222);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "DEL", del_btn.0 + 7, del_btn.1 + 5, 1, 1, 0xFFFFFFFF);
        }

        // Scroll buttons if needed
        if waypoints.len() > max_rows {
            fill_rect(buffer, buf_w, buf_h, scroll_up.0, scroll_up.1, scroll_up.2, scroll_up.3, 0xFF282838);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "^", scroll_up.0 + 5, scroll_up.1 + 3, 1, 1, 0xFFFFFFFF);

            fill_rect(buffer, buf_w, buf_h, scroll_down.0, scroll_down.1, scroll_down.2, scroll_down.3, 0xFF282838);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "v", scroll_down.0 + 5, scroll_down.1 + 3, 1, 1, 0xFFFFFFFF);
        }
    }

    // Bottom action separator
    fill_rect(buffer, buf_w, buf_h, dx + 12, dy + card_h - 44, card_w - 24, 1, 0xFF2A2A3C);

    // [+ New Waypoint] button
    fill_rect(buffer, buf_w, buf_h, new_wp_btn.0, new_wp_btn.1, new_wp_btn.2, new_wp_btn.3, 0xFF2A602A);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "[+] New Waypoint", new_wp_btn.0 + 10, new_wp_btn.1 + 6, 1, 1, 0xFFFFFFFF);

    // Hint in bottom middle
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "[U/Esc] Close List", dx + 200, dy + card_h - 30, 1, 1, 0xFF888899);

    // [Close / ESC] button
    fill_rect(buffer, buf_w, buf_h, bottom_close.0, bottom_close.1, bottom_close.2, bottom_close.3, 0xFF4A4A5A);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "Close", bottom_close.0 + 32, bottom_close.1 + 6, 1, 1, 0xFFFFFFFF);
}

/// Renders the fullscreen WorldMap viewer with controls and inspector.
pub fn draw_worldmap(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    world: &World,
    player_pos: Vec3,
    player_yaw: f32,
    mobs: &[Mob],
    state: &mut WorldMapState,
    waypoints: &[Waypoint],
    mouse_pos: Option<(f32, f32)>,
    lang: crate::core::Language,
) {
    let top_bar_h = 28i32;
    let bottom_bar_h = 26i32;
    let map_h = buf_h as i32 - top_bar_h - bottom_bar_h;
    let map_w = buf_w as i32;

    let center_wx = state.center_x;
    let center_wz = state.center_z;
    let zoom = state.zoom.clamp(0.5, 8.0);
    let half_w = (map_w as f32 * 0.5) / zoom;
    let half_h = (map_h as f32 * 0.5) / zoom;

    // Check persistent terrain cache validity
    let cache_matches = state.cache_valid
        && (state.cache_center.0 - center_wx).abs() < 0.001
        && (state.cache_center.1 - center_wz).abs() < 0.001
        && (state.cache_zoom - zoom).abs() < 0.001
        && state.cache_size == (map_w as usize, map_h as usize)
        && state.map_cache.len() == (map_w * map_h) as usize;

    if !cache_matches {
        state.map_cache.resize((map_w * map_h) as usize, 0);

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "vita"), not(target_arch = "powerpc"), not(feature = "xbox360")))]
        {
            state
                .map_cache
                .par_chunks_mut(map_w as usize)
                .enumerate()
                .for_each(|(py, row_pixels)| {
                    render_worldmap_row(
                        world,
                        row_pixels,
                        py as i32,
                        map_w,
                        center_wx,
                        center_wz,
                        zoom,
                        half_w,
                        half_h,
                    );
                });
        }

        #[cfg(any(target_arch = "wasm32", target_os = "vita", target_arch = "powerpc", feature = "xbox360"))]
        {
            for (py, row_pixels) in state.map_cache.chunks_mut(map_w as usize).enumerate() {
                render_worldmap_row(
                    world,
                    row_pixels,
                    py as i32,
                    map_w,
                    center_wx,
                    center_wz,
                    zoom,
                    half_w,
                    half_h,
                );
            }
        }

        state.cache_valid = true;
        state.cache_center = (center_wx, center_wz);
        state.cache_zoom = zoom;
        state.cache_size = (map_w as usize, map_h as usize);
    }

    // Blit cached terrain rows into framebuffer
    let dest_start = (top_bar_h as usize) * buf_w;
    let dest_len = (map_h as usize) * buf_w;
    buffer[dest_start..dest_start + dest_len].copy_from_slice(&state.map_cache[0..dest_len]);

    // Render selected chunks
    for &(scx, scz) in &state.selected_chunks {
        let c_min_wx = scx * 16;
        let c_min_wz = scz * 16;
        let csx = (map_w as f32 * 0.5 + (c_min_wx as f32 - center_wx) * zoom).round() as i32;
        let csy = top_bar_h + (map_h as f32 * 0.5 + (c_min_wz as f32 - center_wz) * zoom).round() as i32;
        let c_size = (16.0 * zoom).round() as i32;

        if csx + c_size > 0 && csx < map_w && csy + c_size > top_bar_h && csy < buf_h as i32 - bottom_bar_h {
            let clip_x0 = csx.max(0);
            let clip_y0 = csy.max(top_bar_h);
            let clip_x1 = (csx + c_size).min(map_w);
            let clip_y1 = (csy + c_size).min(buf_h as i32 - bottom_bar_h);
            if clip_x1 > clip_x0 && clip_y1 > clip_y0 {
                fill_rect_alpha(
                    buffer,
                    buf_w,
                    buf_h,
                    clip_x0,
                    clip_y0,
                    clip_x1 - clip_x0,
                    clip_y1 - clip_y0,
                    0xFF3388FF,
                    0.35,
                );
            }
            draw_clipped_rect_outline(buffer, buf_w, buf_h, csx, csy, c_size, c_size, 0xFF55CCFF, top_bar_h, buf_h as i32 - bottom_bar_h);
            if c_size >= 28 && csx >= 0 && csx < map_w - 50 && csy >= top_bar_h && csy < buf_h as i32 - bottom_bar_h - 14 {
                draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &format!("[{},{}]", scx, scz), csx + 4, csy + 4, 1, 1, 0xFFCCEEFF);
            }
        }
    }

    // Render hovered chunk
    if let Some((hcx, hcz)) = state.hovered_chunk {
        let c_min_wx = hcx * 16;
        let c_min_wz = hcz * 16;
        let csx = (map_w as f32 * 0.5 + (c_min_wx as f32 - center_wx) * zoom).round() as i32;
        let csy = top_bar_h + (map_h as f32 * 0.5 + (c_min_wz as f32 - center_wz) * zoom).round() as i32;
        let c_size = (16.0 * zoom).round() as i32;

        if csx + c_size > 0 && csx < map_w && csy + c_size > top_bar_h && csy < buf_h as i32 - bottom_bar_h {
            draw_clipped_rect_outline(buffer, buf_w, buf_h, csx, csy, c_size, c_size, 0xFFFFDD33, top_bar_h, buf_h as i32 - bottom_bar_h);
        }
    }

    // Mobs blips
    for mob in mobs {
        if mob.is_dead {
            continue;
        }
        let sx = (map_w as f32 * 0.5 + (mob.pos.x - center_wx) * zoom) as i32;
        let sy = top_bar_h + (map_h as f32 * 0.5 + (mob.pos.z - center_wz) * zoom) as i32;
        if sx >= 2 && sx < map_w - 2 && sy >= top_bar_h + 2 && sy < buf_h as i32 - bottom_bar_h - 2 {
            let col = if mob.kind.is_hostile() { 0xFFFF2222 } else { 0xFF44FF44 };
            fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, 0xFF000000);
            set_px(buffer, buf_w, buf_h, sx, sy, col);
        }
    }

    // Waypoints on map
    for wp in waypoints {
        if !wp.enabled {
            continue;
        }
        let sx = (map_w as f32 * 0.5 + (wp.x as f32 - center_wx) * zoom) as i32;
        let sy = top_bar_h + (map_h as f32 * 0.5 + (wp.z as f32 - center_wz) * zoom) as i32;

        if sx >= 0 && sx < map_w && sy >= top_bar_h && sy < buf_h as i32 - bottom_bar_h {
            // Pin diamond
            fill_rect(buffer, buf_w, buf_h, sx - 2, sy - 2, 5, 5, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, wp.color);
            set_px(buffer, buf_w, buf_h, sx, sy, 0xFFFFFFFF);

            // Label
            let label = format!("[{}]", wp.name);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &label, sx + 5, sy - 4, 1, 1, wp.color);
        }
    }

    // Player position chevron on map
    let p_sx = (map_w as f32 * 0.5 + (player_pos.x - center_wx) * zoom) as i32;
    let p_sy = top_bar_h + (map_h as f32 * 0.5 + (player_pos.z - center_wz) * zoom) as i32;
    if p_sx >= 4 && p_sx < map_w - 4 && p_sy >= top_bar_h + 4 && p_sy < buf_h as i32 - bottom_bar_h - 4 {
        draw_player_chevron(buffer, buf_w, buf_h, p_sx, p_sy, player_yaw);
        draw_text_colored_shadowed(buffer, buf_w, buf_h, "You", p_sx + 8, p_sy - 4, 1, 1, 0xFFFFFFFF);
    }

    // Top control banner
    fill_rect_alpha(buffer, buf_w, buf_h, 0, 0, buf_w as i32, top_bar_h, 0xFF14141E, 0.95);
    fill_rect(buffer, buf_w, buf_h, 0, top_bar_h - 1, buf_w as i32, 1, 0xFF353545);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "WORLD MAP", 12, 8, 1, 1, 0xFFFFDD44);

    let (wp_btn, new_wp_btn, clear_btn) = worldmap_top_bar_buttons();

    // [U] Waypoints button
    let wp_bg = if state.showing_waypoint_manager { 0xFF35588C } else { 0xFF232333 };
    fill_rect(buffer, buf_w, buf_h, wp_btn.0, wp_btn.1, wp_btn.2, wp_btn.3, wp_bg);
    fill_rect(buffer, buf_w, buf_h, wp_btn.0, wp_btn.1, wp_btn.2, 1, 0xFF4A4A62);
    fill_rect(buffer, buf_w, buf_h, wp_btn.0, wp_btn.1 + wp_btn.3 - 1, wp_btn.2, 1, 0xFF4A4A62);
    fill_rect(buffer, buf_w, buf_h, wp_btn.0, wp_btn.1, 1, wp_btn.3, 0xFF4A4A62);
    fill_rect(buffer, buf_w, buf_h, wp_btn.0 + wp_btn.2 - 1, wp_btn.1, 1, wp_btn.3, 0xFF4A4A62);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "[U] Waypoints", wp_btn.0 + 8, wp_btn.1 + 5, 1, 1, 0xFFFFFFFF);

    // [B] +Waypoint button
    fill_rect(buffer, buf_w, buf_h, new_wp_btn.0, new_wp_btn.1, new_wp_btn.2, new_wp_btn.3, 0xFF204820);
    fill_rect(buffer, buf_w, buf_h, new_wp_btn.0, new_wp_btn.1, new_wp_btn.2, 1, 0xFF3D703D);
    fill_rect(buffer, buf_w, buf_h, new_wp_btn.0, new_wp_btn.1 + new_wp_btn.3 - 1, new_wp_btn.2, 1, 0xFF3D703D);
    fill_rect(buffer, buf_w, buf_h, new_wp_btn.0, new_wp_btn.1, 1, new_wp_btn.3, 0xFF3D703D);
    fill_rect(buffer, buf_w, buf_h, new_wp_btn.0 + new_wp_btn.2 - 1, new_wp_btn.1, 1, new_wp_btn.3, 0xFF3D703D);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "[B] +Waypoint", new_wp_btn.0 + 10, new_wp_btn.1 + 5, 1, 1, 0xFFFFFFFF);

    // [C] Clear Chunks button
    let clear_bg = if !state.selected_chunks.is_empty() { 0xFF582424 } else { 0xFF20202C };
    fill_rect(buffer, buf_w, buf_h, clear_btn.0, clear_btn.1, clear_btn.2, clear_btn.3, clear_bg);
    let clear_txt = format!("[C] Clear ({})", state.selected_chunks.len());
    draw_text_light_colored_shadowed(
        buffer,
        buf_w,
        buf_h,
        &clear_txt,
        clear_btn.0 + 8,
        clear_btn.1 + 5,
        1,
        1,
        if !state.selected_chunks.is_empty() { 0xFFFFFFFF } else { 0xFF777788 },
    );

    let controls_hint = "[WASD/Drag] Pan  [Scroll] Zoom  [L-Click] Chunk  [R-Click] Menu  [Space] Center  [Esc/M] Close";
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, controls_hint, 535, 8, 1, 1, 0xFFB0B0C0);
    let zoom_hint = format!("Zoom: {:.1}x", zoom);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &zoom_hint, buf_w as i32 - 95, 8, 1, 1, 0xFF88CCFF);

    // Bottom inspection bar
    fill_rect_alpha(buffer, buf_w, buf_h, 0, buf_h as i32 - bottom_bar_h, buf_w as i32, bottom_bar_h, 0xFF14141E, 0.95);
    fill_rect(buffer, buf_w, buf_h, 0, buf_h as i32 - bottom_bar_h, buf_w as i32, 1, 0xFF353545);

    state.hovered_world_pos = None;
    state.hovered_chunk = None;

    if let Some((mx, my)) = mouse_pos {
        if my >= top_bar_h as f32 && my < (buf_h as i32 - bottom_bar_h) as f32 {
            let h_wx = (center_wx - half_w + mx / zoom).floor() as i32;
            let h_wz = (center_wz - half_h + (my - top_bar_h as f32) / zoom).floor() as i32;
            let (h_sy, sblk) = sample_surface(world, h_wx, h_wz);
            state.hovered_world_pos = Some((h_wx, h_sy, h_wz));

            let ch_cx = h_wx.div_euclid(16);
            let ch_cz = h_wz.div_euclid(16);
            state.hovered_chunk = Some((ch_cx, ch_cz));

            let (biome, _) = get_biome_at(h_wx, h_wz, world.seed);
            let dist_to_player = ((h_wx as f32 - player_pos.x).powi(2) + (h_wz as f32 - player_pos.z).powi(2)).sqrt();

            let is_sel = state.selected_chunks.contains(&(ch_cx, ch_cz));
            let sel_status = if is_sel { "[SELECTED] " } else { "" };

            let inspect_str = format!(
                "Block: X: {} Y: {} Z: {} ({}) | Chunk: [{}, {}] {}| Biome: {} | Dist: {:.0}m | Selected: {} chunks",
                h_wx, h_sy, h_wz, sblk.name(), ch_cx, ch_cz, sel_status, biome.name(), dist_to_player, state.selected_chunks.len()
            );
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &inspect_str, 12, buf_h as i32 - bottom_bar_h + 7, 1, 1, 0xFFFFFFFF);
        }
    }

    if state.hovered_world_pos.is_none() {
        let p_status = format!(
            "Player: X: {:.1}  Y: {:.1}  Z: {:.1} | Selected Chunks: {} | Total Waypoints: {}",
            player_pos.x, player_pos.y, player_pos.z, state.selected_chunks.len(), waypoints.len()
        );
        draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &p_status, 12, buf_h as i32 - bottom_bar_h + 7, 1, 1, 0xFFAAAAAA);
    }

    // Context Menu (Right-Click)
    if let Some(ctx_menu) = &state.context_menu {
        let (card, item0, item1) = worldmap_context_menu_rects(ctx_menu.screen_pos.0, ctx_menu.screen_pos.1, buf_w, buf_h);

        // Shadow & Background card
        fill_rect_alpha(buffer, buf_w, buf_h, card.0 + 3, card.1 + 3, card.2, card.3, 0xFF000000, 0.55);
        fill_rect(buffer, buf_w, buf_h, card.0, card.1, card.2, card.3, 0xFF161822);
        fill_rect(buffer, buf_w, buf_h, card.0, card.1, card.2, 1, 0xFF4A4E65);
        fill_rect(buffer, buf_w, buf_h, card.0, card.1 + card.3 - 1, card.2, 1, 0xFF2A2C38);
        fill_rect(buffer, buf_w, buf_h, card.0, card.1, 1, card.3, 0xFF4A4E65);
        fill_rect(buffer, buf_w, buf_h, card.0 + card.2 - 1, card.1, 1, card.3, 0xFF2A2C38);

        let (mouse_x, mouse_y) = mouse_pos.map(|(x, y)| (x as i32, y as i32)).unwrap_or((-1, -1));

        // Option 0: Create waypoint here
        let hover0 = mouse_x >= item0.0 && mouse_x < item0.0 + item0.2 && mouse_y >= item0.1 && mouse_y < item0.1 + item0.3;
        let bg0 = if hover0 { 0xFF2B4D78 } else { 0xFF1F2332 };
        fill_rect(buffer, buf_w, buf_h, item0.0, item0.1, item0.2, item0.3, bg0);
        let txt0 = match lang {
            crate::core::Language::Polish => "Zrób tutaj waypointa",
            crate::core::Language::English => "Create waypoint here",
        };
        draw_text_light_colored_shadowed(buffer, buf_w, buf_h, txt0, item0.0 + 8, item0.1 + 6, 1, 1, if hover0 { 0xFFFFFF55 } else { 0xFFE0E0EE });

        // Option 1: Share coordinates in chat
        let hover1 = mouse_x >= item1.0 && mouse_x < item1.0 + item1.2 && mouse_y >= item1.1 && mouse_y < item1.1 + item1.3;
        let bg1 = if hover1 { 0xFF2B4D78 } else { 0xFF1F2332 };
        fill_rect(buffer, buf_w, buf_h, item1.0, item1.1, item1.2, item1.3, bg1);
        let txt1 = match lang {
            crate::core::Language::Polish => "Udostępnij koordynaty na czacie",
            crate::core::Language::English => "Share coordinates in chat",
        };
        draw_text_light_colored_shadowed(buffer, buf_w, buf_h, txt1, item1.0 + 8, item1.1 + 6, 1, 1, if hover1 { 0xFF55FFFF } else { 0xFFE0E0EE });
    }

    // Modal Waypoint Manager Dialog
    if state.showing_waypoint_manager {
        draw_waypoint_manager(buffer, buf_w, buf_h, waypoints, player_pos, state.waypoint_manager_scroll);
    }

    // Modal Waypoint Creation Dialog
    if state.creating_waypoint {
        draw_waypoint_dialog(buffer, buf_w, buf_h, state);
    }
}

fn draw_waypoint_dialog(buffer: &mut [u32], buf_w: usize, buf_h: usize, state: &WorldMapState) {
    let dia_w = 320i32;
    let dia_h = 180i32;
    let dx = (buf_w as i32 - dia_w) / 2;
    let dy = (buf_h as i32 - dia_h) / 2;

    // Dark backdrop
    fill_rect_alpha(buffer, buf_w, buf_h, 0, 0, buf_w as i32, buf_h as i32, 0xFF000000, 0.55);

    // Dialog card
    fill_rect(buffer, buf_w, buf_h, dx, dy, dia_w, dia_h, 0xFF1C1C26);
    fill_rect(buffer, buf_w, buf_h, dx - 1, dy - 1, dia_w + 2, 1, 0xFF4A4A62);
    fill_rect(buffer, buf_w, buf_h, dx - 1, dy + dia_h, dia_w + 2, 1, 0xFF4A4A62);
    fill_rect(buffer, buf_w, buf_h, dx - 1, dy - 1, 1, dia_h + 2, 0xFF4A4A62);
    fill_rect(buffer, buf_w, buf_h, dx + dia_w, dy - 1, 1, dia_h + 2, 0xFF4A4A62);

    // Title
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "NEW WAYPOINT", dx + 20, dy + 16, 1, 1, 0xFFFFDD44);

    // Name prompt
    let name_str = format!("Name: {}_", state.waypoint_name_input);
    fill_rect(buffer, buf_w, buf_h, dx + 20, dy + 40, dia_w - 40, 24, 0xFF101018);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &name_str, dx + 26, dy + 46, 1, 1, 0xFFFFFFFF);

    // Coords
    let coords_str = format!(
        "Coords: X: {}, Y: {}, Z: {}",
        state.waypoint_coords_input.0, state.waypoint_coords_input.1, state.waypoint_coords_input.2
    );
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &coords_str, dx + 20, dy + 76, 1, 1, 0xFFCCCCCC);

    // Color Swatches
    draw_text_light_shadowed(buffer, buf_w, buf_h, "Color:", dx + 20, dy + 98, 1, 1);
    for (i, &(c, _)) in WAYPOINT_COLORS.iter().enumerate() {
        let sw_x = dx + 70 + (i as i32) * 28;
        let sw_y = dy + 95;
        fill_rect(buffer, buf_w, buf_h, sw_x, sw_y, 20, 18, c);
        if i == state.waypoint_color_idx {
            fill_rect(buffer, buf_w, buf_h, sw_x - 1, sw_y - 1, 22, 1, 0xFFFFFFFF);
            fill_rect(buffer, buf_w, buf_h, sw_x - 1, sw_y + 18, 22, 1, 0xFFFFFFFF);
            fill_rect(buffer, buf_w, buf_h, sw_x - 1, sw_y - 1, 1, 20, 0xFFFFFFFF);
            fill_rect(buffer, buf_w, buf_h, sw_x + 20, sw_y - 1, 1, 20, 0xFFFFFFFF);
        }
    }

    // Action buttons
    let (save_btn, cancel_btn) = waypoint_dialog_buttons(dx, dy, dia_w, dia_h);
    fill_rect(buffer, buf_w, buf_h, save_btn.0, save_btn.1, save_btn.2, save_btn.3, 0xFF2D6A2D);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "[ENTER] Save", save_btn.0 + 12, save_btn.1 + 6, 1, 1, 0xFFFFFFFF);

    fill_rect(buffer, buf_w, buf_h, cancel_btn.0, cancel_btn.1, cancel_btn.2, cancel_btn.3, 0xFF6A2D2D);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, "[ESC] Cancel", cancel_btn.0 + 10, cancel_btn.1 + 6, 1, 1, 0xFFFFFFFF);
}

pub fn worldmap_context_menu_rects(
    screen_x: i32,
    screen_y: i32,
    buf_w: usize,
    buf_h: usize,
) -> ((i32, i32, i32, i32), (i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let w = 240i32;
    let h = 58i32;
    let top_bar_h = 28i32;
    let bottom_bar_h = 26i32;

    let mx = screen_x.clamp(8, buf_w as i32 - w - 8);
    let my = screen_y.clamp(top_bar_h + 2, buf_h as i32 - bottom_bar_h - h - 2);

    let card = (mx, my, w, h);
    let item0 = (mx + 4, my + 4, w - 8, 23);
    let item1 = (mx + 4, my + 30, w - 8, 23);
    (card, item0, item1)
}

pub fn waypoint_dialog_buttons(dx: i32, dy: i32, dia_w: i32, dia_h: i32) -> ((i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let btn_w = 110i32;
    let btn_h = 24i32;
    let btn_y = dy + dia_h - 36;
    let save_rect = (dx + 25, btn_y, btn_w, btn_h);
    let cancel_rect = (dx + dia_w - btn_w - 25, btn_y, btn_w, btn_h);
    (save_rect, cancel_rect)
}

/// Renders in-world floating waypoint beacons and distance markers on screen.
pub fn draw_world_waypoint_markers(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    camera: &Camera,
    waypoints: &[Waypoint],
) {
    let aspect = buf_w as f32 / buf_h as f32;
    let view_proj = camera.projection_matrix(aspect).mul(&camera.view_matrix());

    for wp in waypoints {
        if !wp.enabled {
            continue;
        }

        let world_pos = Vec3::new(wp.x as f32 + 0.5, wp.y as f32 + 1.2, wp.z as f32 + 0.5);
        let dist = (world_pos - camera.position).length();
        if dist < 0.5 {
            continue;
        }

        // Project 3D point to 2D NDC
        let clip = view_proj.transform_point(world_pos);
        if clip[3] <= 0.01 {
            continue; // Behind camera
        }

        let ndc_x = clip[0] / clip[3];
        let ndc_y = clip[1] / clip[3];

        if ndc_x < -1.1 || ndc_x > 1.1 || ndc_y < -1.1 || ndc_y > 1.1 {
            continue;
        }

        let sx = ((ndc_x * 0.5 + 0.5) * buf_w as f32) as i32;
        let sy = (((-ndc_y) * 0.5 + 0.5) * buf_h as f32) as i32;

        if sx >= 10 && sx < buf_w as i32 - 10 && sy >= 10 && sy < buf_h as i32 - 10 {
            // Beacon diamond
            fill_rect(buffer, buf_w, buf_h, sx - 2, sy - 2, 5, 5, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, wp.color);
            set_px(buffer, buf_w, buf_h, sx, sy, 0xFFFFFFFF);

            // Text label: "Name (120m)"
            let text = format!("{} ({:.0}m)", wp.name, dist);
            let text_w = text.len() as i32 * 6;
            let tx = (sx - text_w / 2).max(4);
            let ty = (sy - 14).max(4);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &text, tx, ty, 1, 1, wp.color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimap_mode_cycling() {
        let mode = MinimapMode::Circle;
        assert_eq!(mode.next(), MinimapMode::Square);
        assert_eq!(mode.next().next(), MinimapMode::Hidden);
        assert_eq!(mode.next().next().next(), MinimapMode::Circle);
    }

    #[test]
    fn test_waypoint_dialog_button_bounds() {
        let (save, cancel) = waypoint_dialog_buttons(100, 100, 320, 180);
        assert!(save.0 >= 100);
        assert!(cancel.0 > save.0);
        assert_eq!(save.2, 110);
        assert_eq!(cancel.2, 110);
    }

    #[test]
    fn test_waypoint_serialization_roundtrip() {
        let original = vec![
            Waypoint {
                name: "Base".to_string(),
                x: 10,
                y: 64,
                z: -25,
                color: 0xFFFF3333,
                enabled: true,
            },
            Waypoint {
                name: "Iron Mine".to_string(),
                x: 120,
                y: 32,
                z: 400,
                color: 0xFF33FF33,
                enabled: false,
            },
        ];

        let serialized = serialize_waypoints(&original);
        let parsed = parse_waypoints_str(&serialized);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].name, "Base");
        assert_eq!(parsed[0].x, 10);
        assert_eq!(parsed[0].y, 64);
        assert_eq!(parsed[0].z, -25);
        assert_eq!(parsed[0].color, 0xFFFF3333);
        assert!(parsed[0].enabled);

        assert_eq!(parsed[1].name, "Iron Mine");
        assert_eq!(parsed[1].x, 120);
        assert_eq!(parsed[1].y, 32);
        assert_eq!(parsed[1].z, 400);
        assert_eq!(parsed[1].color, 0xFF33FF33);
        assert!(!parsed[1].enabled);
    }

    #[test]
    fn test_worldmap_top_bar_buttons() {
        let (wp, new_wp, clear) = worldmap_top_bar_buttons();
        assert_eq!(wp.1, 4);
        assert_eq!(wp.3, 20);
        assert!(new_wp.0 > wp.0);
        assert!(clear.0 > new_wp.0);
    }

    #[test]
    fn test_waypoint_manager_buttons() {
        let (header_close, new_wp, bottom_close, scroll_up, scroll_down) =
            waypoint_manager_buttons(100, 100, 540, 360);
        assert!(header_close.0 > 100);
        assert_eq!(new_wp.1, 100 + 360 - 36);
        assert!(bottom_close.0 > new_wp.0);
        assert!(scroll_down.1 > scroll_up.1);
    }

    #[test]
    fn test_waypoint_row_buttons() {
        let (vis, tp, del) = waypoint_row_buttons(100, 150, 540, 0);
        assert!(tp.0 > vis.0);
        assert!(del.0 > tp.0);
    }

    #[test]
    fn test_worldmap_state_selection_and_cache() {
        let mut state = WorldMapState::new(Vec3::new(0.0, 64.0, 0.0));
        assert!(state.selected_chunks.is_empty());
        assert!(!state.cache_valid);

        state.selected_chunks.insert((2, -3));
        assert!(state.selected_chunks.contains(&(2, -3)));
        assert_eq!(state.selected_chunks.len(), 1);

        state.selected_chunks.remove(&(2, -3));
        assert!(state.selected_chunks.is_empty());

        state.cache_valid = true;
        state.invalidate_cache();
        assert!(!state.cache_valid);
    }

    #[test]
    fn test_worldmap_context_menu_rects() {
        let (card, item0, item1) = worldmap_context_menu_rects(50, 50, 800, 600);
        assert_eq!(card.0, 50);
        assert_eq!(card.1, 50);
        assert!(card.2 >= 180);
        assert!(card.3 >= 50);

        // Options must be inside card
        assert!(item0.0 >= card.0 && item0.0 + item0.2 <= card.0 + card.2);
        assert!(item1.0 >= card.0 && item1.0 + item1.2 <= card.0 + card.2);
        assert!(item1.1 > item0.1);

        // Clamping at edge
        let (card_edge, _, _) = worldmap_context_menu_rects(795, 595, 800, 600);
        assert!(card_edge.0 + card_edge.2 <= 796);
        assert!(card_edge.1 + card_edge.3 <= 596);
    }
}
