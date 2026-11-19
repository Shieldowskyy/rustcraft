//! Renderowanie minimapy (okrągłej/kwadratowej), kropek mobów, wskaźnika gracza oraz obrysów.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::entities::Mob;
use crate::features::ui::draw::{
    draw_text_colored_shadowed, draw_text_light_colored_shadowed, fill_rect, fill_rect_alpha,
    pack_color, set_px,
};
use crate::features::world::{Chunk, World};
use crate::platform::capabilities;

use super::storage::{fast_calculate_map_color, sample_chunk_column, sample_chunk_column_y};
use super::types::{MinimapMode, MinimapState};

const MINIMAP_VITA_IDLE_FRAMES: u32 = 12;
const MINIMAP_VITA_PERIODIC_REFRESH_FRAMES: u32 = 180;
const MINIMAP_DEFAULT_PERIODIC_REFRESH_FRAMES: u32 = 32;

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
    if x0 >= 0 && x0 < buf_w as i32 {
        let top = y0.max(min_clip_y).max(0);
        let bottom = y1.min(max_clip_y).min(buf_h as i32);
        for yy in top..bottom {
            buffer[yy as usize * buf_w + x0 as usize] = color;
        }
    }
    let rx = x1 - 1;
    if rx >= 0 && rx < buf_w as i32 {
        let top = y0.max(min_clip_y).max(0);
        let bottom = y1.min(max_clip_y).min(buf_h as i32);
        for yy in top..bottom {
            buffer[yy as usize * buf_w + rx as usize] = color;
        }
    }
}

pub(super) fn draw_player_chevron(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    cx: i32,
    cy: i32,
    yaw: f32,
) {
    let sin = yaw.sin();
    let cos = yaw.cos();

    let tip = (sin * 5.0, cos * 5.0);
    let left = (-cos * 3.5 - sin * 3.0, sin * 3.5 - cos * 3.0);
    let right = (cos * 3.5 - sin * 3.0, -sin * 3.5 - cos * 3.0);

    fill_rect(buffer, buf_w, buf_h, cx - 1, cy - 1, 3, 3, 0xFF000000);
    set_px(buffer, buf_w, buf_h, cx + (tip.0 * 1.2) as i32, cy + (tip.1 * 1.2) as i32, 0xFF000000);

    set_px(buffer, buf_w, buf_h, cx + tip.0 as i32, cy + tip.1 as i32, 0xFFFF2222);
    set_px(buffer, buf_w, buf_h, cx + left.0 as i32, cy + left.1 as i32, 0xFFFFFFFF);
    set_px(buffer, buf_w, buf_h, cx + right.0 as i32, cy + right.1 as i32, 0xFFFFFFFF);
    set_px(buffer, buf_w, buf_h, cx, cy, 0xFFFFFFFF);
}

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
    let diameter = (radius * 2 + 1) as usize;
    let cx = buf_w as i32 - 72;
    let cy = 72i32;

    let is_circle = state.mode == MinimapMode::Circle;
    let rad_sq = radius * radius;
    let inner_rad_sq = (radius - 2) * (radius - 2);
    let shadow_rad_sq = (radius + 2) * (radius + 2);

    let px_block = player_pos.x.floor() as i32;
    let pz_block = player_pos.z.floor() as i32;
    let zoom_bits = state.zoom.to_bits();

    let mut cache = state.terrain_cache.borrow_mut();
    let moved_this_frame = cache.last_player_pos.map_or(false, |(last_x, last_z)| {
        let dx = player_pos.x - last_x;
        let dz = player_pos.z - last_z;
        dx * dx + dz * dz > 0.0004
    });
    if moved_this_frame {
        cache.stationary_frames = 0;
    } else {
        cache.stationary_frames = cache.stationary_frames.saturating_add(1);
    }
    cache.last_player_pos = Some((player_pos.x, player_pos.z));

    let cache_shape_valid = cache.pixels.len() == diameter * diameter;
    let settings_changed =
        cache.mode != Some(state.mode) || cache.zoom_bits != zoom_bits || !cache_shape_valid;
    let zoom_for_offset = state.zoom.abs().max(0.001);

    // DLACZEGO: Na PS Vita pełny raster terenu minimapy nie powinien przebudowywać się w trakcie biegu;
    // podczas ruchu przesuwamy bufor pod kursorem i odświeżamy po zatrzymaniu lub dużym przemieszczeniu.
    let (movement_refresh_due, periodic_refresh_due) = if capabilities().is_vita {
        let offset_x = ((player_pos.x - cache.center_world_pos.0) / zoom_for_offset).round() as i32;
        let offset_z = ((player_pos.z - cache.center_world_pos.1) / zoom_for_offset).round() as i32;
        let has_moved_from_cached_center = (player_pos.x - cache.center_world_pos.0).abs() > 0.0001
            || (player_pos.z - cache.center_world_pos.1).abs() > 0.0001;
        let mov = has_moved_from_cached_center
            && (cache.stationary_frames >= MINIMAP_VITA_IDLE_FRAMES
                || offset_x.abs() >= radius - 8
                || offset_z.abs() >= radius - 8);
        let per = cache.frames_since_refresh >= MINIMAP_VITA_PERIODIC_REFRESH_FRAMES
            && cache.stationary_frames >= MINIMAP_VITA_IDLE_FRAMES;
        (mov, per)
    } else {
        let mov = cache.center_block != (px_block, pz_block);
        let per = cache.frames_since_refresh >= MINIMAP_DEFAULT_PERIODIC_REFRESH_FRAMES;
        (mov, per)
    };

    let needs_rebuild = settings_changed || movement_refresh_due || periodic_refresh_due;
    if needs_rebuild {
        cache.pixels.resize(diameter * diameter, 0);
        cache.center_block = (px_block, pz_block);
        cache.center_world_pos = (player_pos.x, player_pos.z);
        cache.zoom_bits = zoom_bits;
        cache.mode = Some(state.mode);
        cache.frames_since_refresh = 0;

        let mut prev_row_y = [0i32; 112];
        let mut last_cx = i32::MIN;
        let mut last_cz = i32::MIN;
        let mut cached_chunk: Option<&Chunk> = None;

        let wz_top_north = (player_pos.z + (-radius - 1) as f32 * state.zoom).floor() as i32;
        let cz_top_north = wz_top_north >> 4;
        let lz_top_north = wz_top_north.rem_euclid(16);
        for (col_i, dx) in (-radius..=radius).enumerate() {
            let wx = (player_pos.x + dx as f32 * state.zoom).floor() as i32;
            let c_x = wx >> 4;
            if c_x != last_cx || cz_top_north != last_cz {
                cached_chunk = world.chunks.get(&(c_x, cz_top_north));
                last_cx = c_x;
                last_cz = cz_top_north;
            }
            prev_row_y[col_i] = if let Some(ch) = cached_chunk {
                sample_chunk_column_y(ch, wx.rem_euclid(16), lz_top_north)
            } else {
                0
            };
        }

        for (row_i, dy) in (-radius..=radius).enumerate() {
            let wz = (player_pos.z + dy as f32 * state.zoom).floor() as i32;
            let c_z = wz >> 4;
            let lz = wz.rem_euclid(16);
            let row_offset = row_i * diameter;

            for (col_i, dx) in (-radius..=radius).enumerate() {
                let dist_sq = dx * dx + dy * dy;
                if is_circle && dist_sq > inner_rad_sq {
                    continue;
                }

                let wx = (player_pos.x + dx as f32 * state.zoom).floor() as i32;
                let c_x = wx >> 4;
                if c_x != last_cx || c_z != last_cz {
                    cached_chunk = world.chunks.get(&(c_x, c_z));
                    last_cx = c_x;
                    last_cz = c_z;
                }

                let (sy, sblk) = if let Some(ch) = cached_chunk {
                    sample_chunk_column(ch, wx.rem_euclid(16), lz)
                } else {
                    (0, BlockId::Air)
                };

                let y_north = prev_row_y[col_i];
                prev_row_y[col_i] = sy;
                cache.pixels[row_offset + col_i] = fast_calculate_map_color(sy, sblk, y_north);
            }
        }
    } else {
        cache.frames_since_refresh = cache.frames_since_refresh.saturating_add(1);
    }

    let cache_offset_x = ((player_pos.x - cache.center_world_pos.0) / zoom_for_offset).round() as i32;
    let cache_offset_z = ((player_pos.z - cache.center_world_pos.1) / zoom_for_offset).round() as i32;

    for (row_i, dy) in (-radius..=radius).enumerate() {
        let py = cy + dy;
        if py < 0 || py >= buf_h as i32 {
            continue;
        }
        let cache_y = row_i as i32 + cache_offset_z;

        for (col_i, dx) in (-radius..=radius).enumerate() {
            let px = cx + dx;
            if px < 0 || px >= buf_w as i32 {
                continue;
            }

            let dist_sq = dx * dx + dy * dy;
            if is_circle && dist_sq > shadow_rad_sq {
                continue;
            }

            let idx = (py as usize) * buf_w + (px as usize);

            if is_circle {
                if dist_sq > rad_sq {
                    let cur = buffer[idx];
                    let r = (((cur >> 16) & 0xFF) * 140) >> 8;
                    let g = (((cur >> 8) & 0xFF) * 140) >> 8;
                    let b = ((cur & 0xFF) * 140) >> 8;
                    buffer[idx] = (0xFF << 24) | (r << 16) | (g << 8) | b;
                    continue;
                } else if dist_sq >= inner_rad_sq {
                    buffer[idx] = 0xFF2A2A38;
                    continue;
                }
            } else {
                let border = dx == -radius
                    || dx == radius
                    || dy == -radius
                    || dy == radius
                    || dx == -radius + 1
                    || dx == radius - 1
                    || dy == -radius + 1
                    || dy == radius - 1;
                if border {
                    buffer[idx] = 0xFF2A2A38;
                    continue;
                }
            }

            let cache_x = col_i as i32 + cache_offset_x;
            if cache_x >= 0
                && cache_x < diameter as i32
                && cache_y >= 0
                && cache_y < diameter as i32
            {
                buffer[idx] = cache.pixels[cache_y as usize * diameter + cache_x as usize];
            } else {
                buffer[idx] = pack_color(0.08, 0.08, 0.12);
            }
        }
    }
    drop(cache);

    let n_y = cy - radius + 6;
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "N", cx - 2, n_y, 1, 1, 0xFFFF4444);
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "S", cx - 2, cy + radius - 12, 1, 1, 0xFFE0E0E0);
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "E", cx + radius - 12, cy - 3, 1, 1, 0xFFE0E0E0);
    draw_text_colored_shadowed(buffer, buf_w, buf_h, "W", cx - radius + 5, cy - 3, 1, 1, 0xFFE0E0E0);

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
        let dot_color = if mob.kind.is_hostile() { 0xFFFF2222 } else { 0xFF44FF44 };
        fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, 0xFF000000);
        set_px(buffer, buf_w, buf_h, sx, sy, dot_color);
    }

    for wp in &state.waypoints {
        if !wp.enabled {
            continue;
        }
        let w_dx = (wp.x as f32 - player_pos.x) / state.zoom;
        let w_dz = (wp.z as f32 - player_pos.z) / state.zoom;
        let dist = (w_dx * w_dx + w_dz * w_dz).sqrt();

        let max_r = (radius - 7) as f32;
        let (px, py) = if dist > max_r {
            let scale = max_r / dist;
            (cx + (w_dx * scale) as i32, cy + (w_dz * scale) as i32)
        } else {
            (cx + w_dx as i32, cy + w_dz as i32)
        };

        fill_rect(buffer, buf_w, buf_h, px - 1, py - 1, 3, 3, 0xFF000000);
        set_px(buffer, buf_w, buf_h, px, py - 2, wp.color);
        set_px(buffer, buf_w, buf_h, px - 2, py, wp.color);
        set_px(buffer, buf_w, buf_h, px + 2, py, wp.color);
        set_px(buffer, buf_w, buf_h, px, py + 2, wp.color);
        set_px(buffer, buf_w, buf_h, px, py, 0xFFFFFFFF);
    }

    draw_player_chevron(buffer, buf_w, buf_h, cx, cy, player_yaw);

    let info_y = cy + radius + 6;
    let px_i = player_pos.x.floor() as i32;
    let py_i = player_pos.y.floor() as i32;
    let pz_i = player_pos.z.floor() as i32;
    let coord_str = format!("X:{} Y:{} Z:{}", px_i, py_i, pz_i);
    let biome = world.get_biome_at(px_i, pz_i);
    let biome_str = biome.name();

    let bg_w = 120i32;
    let bg_x = cx - bg_w / 2;
    fill_rect_alpha(buffer, buf_w, buf_h, bg_x, info_y - 2, bg_w, 22, 0xFF000000, 0.45);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &coord_str, bg_x + 4, info_y, 1, 1, 0xFFFFFFFF);
    draw_text_light_colored_shadowed(buffer, buf_w, buf_h, biome_str, bg_x + 4, info_y + 10, 1, 1, 0xFF77DD77);
}
