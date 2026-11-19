//! Renderowanie pełnoekranowej mapy świata z równoległym próbkowaniem wierszy, zaznaczaniem chunków i menu kontekstowym.

use crate::core::Language;
use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::entities::Mob;
use crate::features::ui::draw::{
    draw_text_colored_shadowed, draw_text_light_colored_shadowed, fill_rect, fill_rect_alpha,
    set_px,
};
use crate::features::ui::widgets::{Rect, UiCanvas};
use crate::features::world::{Chunk, World};
use crate::platform::threading::parallel_for_chunks_mut;

use super::radar::{draw_clipped_rect_outline, draw_player_chevron};
use super::storage::{
    fast_calculate_map_color, sample_chunk_column, sample_chunk_column_y, sample_surface,
};
use super::types::{Waypoint, WorldMapState};
use super::waypoints_ui::{
    draw_waypoint_dialog, draw_waypoint_manager, worldmap_context_menu_rects,
    worldmap_top_bar_buttons,
};

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
    let mut curr_chunk: Option<&Chunk> = None;
    let mut north_chunk: Option<&Chunk> = None;

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
    lang: Language,
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

    let cache_matches = state.cache_valid
        && (state.cache_center.0 - center_wx).abs() < 0.001
        && (state.cache_center.1 - center_wz).abs() < 0.001
        && (state.cache_zoom - zoom).abs() < 0.001
        && state.cache_size == (map_w as usize, map_h as usize)
        && state.map_cache.len() == (map_w * map_h) as usize;

    if !cache_matches {
        state.map_cache.resize((map_w * map_h) as usize, 0);
        parallel_for_chunks_mut(&mut state.map_cache, map_w as usize, |py, row_pixels| {
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

        state.cache_valid = true;
        state.cache_center = (center_wx, center_wz);
        state.cache_zoom = zoom;
        state.cache_size = (map_w as usize, map_h as usize);
    }

    let dest_start = (top_bar_h as usize) * buf_w;
    let dest_len = (map_h as usize) * buf_w;
    buffer[dest_start..dest_start + dest_len].copy_from_slice(&state.map_cache[0..dest_len]);

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
            draw_clipped_rect_outline(
                buffer,
                buf_w,
                buf_h,
                csx,
                csy,
                c_size,
                c_size,
                0xFF55CCFF,
                top_bar_h,
                buf_h as i32 - bottom_bar_h,
            );
            if c_size >= 28 && csx >= 0 && csx < map_w - 50 && csy >= top_bar_h && csy < buf_h as i32 - bottom_bar_h - 14 {
                draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &format!("[{},{}]", scx, scz), csx + 4, csy + 4, 1, 1, 0xFFCCEEFF);
            }
        }
    }

    if let Some((hcx, hcz)) = state.hovered_chunk {
        let c_min_wx = hcx * 16;
        let c_min_wz = hcz * 16;
        let csx = (map_w as f32 * 0.5 + (c_min_wx as f32 - center_wx) * zoom).round() as i32;
        let csy = top_bar_h + (map_h as f32 * 0.5 + (c_min_wz as f32 - center_wz) * zoom).round() as i32;
        let c_size = (16.0 * zoom).round() as i32;

        if csx + c_size > 0 && csx < map_w && csy + c_size > top_bar_h && csy < buf_h as i32 - bottom_bar_h {
            draw_clipped_rect_outline(
                buffer,
                buf_w,
                buf_h,
                csx,
                csy,
                c_size,
                c_size,
                0xFFFFDD33,
                top_bar_h,
                buf_h as i32 - bottom_bar_h,
            );
        }
    }

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

    for wp in waypoints {
        if !wp.enabled {
            continue;
        }
        let sx = (map_w as f32 * 0.5 + (wp.x as f32 - center_wx) * zoom) as i32;
        let sy = top_bar_h + (map_h as f32 * 0.5 + (wp.z as f32 - center_wz) * zoom) as i32;

        if sx >= 0 && sx < map_w && sy >= top_bar_h && sy < buf_h as i32 - bottom_bar_h {
            fill_rect(buffer, buf_w, buf_h, sx - 2, sy - 2, 5, 5, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, wp.color);
            set_px(buffer, buf_w, buf_h, sx, sy, 0xFFFFFFFF);

            let label = format!("[{}]", wp.name);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &label, sx + 5, sy - 4, 1, 1, wp.color);
        }
    }

    let p_sx = (map_w as f32 * 0.5 + (player_pos.x - center_wx) * zoom) as i32;
    let p_sy = top_bar_h + (map_h as f32 * 0.5 + (player_pos.z - center_wz) * zoom) as i32;
    if p_sx >= 4 && p_sx < map_w - 4 && p_sy >= top_bar_h + 4 && p_sy < buf_h as i32 - bottom_bar_h - 4 {
        draw_player_chevron(buffer, buf_w, buf_h, p_sx, p_sy, player_yaw);
        draw_text_colored_shadowed(buffer, buf_w, buf_h, "You", p_sx + 8, p_sy - 4, 1, 1, 0xFFFFFFFF);
    }

    let mut ui = UiCanvas::new(buffer, buf_w, buf_h);
    ui.fill_alpha(Rect::new(0, 0, buf_w as i32, top_bar_h), 0xFF14141E, 0.95);
    ui.fill(Rect::new(0, top_bar_h - 1, buf_w as i32, 1), 0xFF353545);
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, "WORLD MAP", 12, 8, 1, 1, 0xFFFFDD44);

    let (wp_btn, new_wp_btn, clear_btn) = worldmap_top_bar_buttons();
    let wp_bg = if state.showing_waypoint_manager { 0xFF35588C } else { 0xFF232333 };
    ui.light_button(wp_btn, "[U] Waypoints", wp_bg, Some(0xFF4A4A62), 0xFFFFFFFF);
    ui.light_button(new_wp_btn, "[B] +Waypoint", 0xFF204820, Some(0xFF3D703D), 0xFFFFFFFF);

    let clear_bg = if !state.selected_chunks.is_empty() { 0xFF582424 } else { 0xFF20202C };
    let clear_txt = format!("[C] Clear ({})", state.selected_chunks.len());
    let clear_col = if !state.selected_chunks.is_empty() { 0xFFFFFFFF } else { 0xFF777788 };
    ui.light_button(clear_btn, &clear_txt, clear_bg, None, clear_col);

    let controls_hint =
        "[WASD/Drag] Pan  [Scroll] Zoom  [L-Click] Chunk  [R-Click] Menu  [Space] Center  [Esc/M] Close";
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, controls_hint, 535, 8, 1, 1, 0xFFB0B0C0);
    let zoom_hint = format!("Zoom: {:.1}x", zoom);
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &zoom_hint, buf_w as i32 - 95, 8, 1, 1, 0xFF88CCFF);

    ui.fill_alpha(Rect::new(0, buf_h as i32 - bottom_bar_h, buf_w as i32, bottom_bar_h), 0xFF14141E, 0.95);
    ui.fill(Rect::new(0, buf_h as i32 - bottom_bar_h, buf_w as i32, 1), 0xFF353545);

    state.hovered_world_pos = None;
    state.hovered_chunk = None;

    if let Some((mx, my)) = mouse_pos {
        if my >= top_bar_h as f32 && my < (buf_h as i32 - bottom_bar_h) as f32 {
            let h_wx = (center_wx - half_w + mx / zoom).floor() as i32;
            let h_wz = (center_wz - half_h + (my - top_bar_h as f32) / zoom).floor() as i32;
            let (h_sy, sblk) = sample_surface(world, h_wx, h_wz);
            state.hovered_world_pos = Some((h_wx, h_sy, h_wz));

            let (ch_cx, ch_cz) = (h_wx.div_euclid(16), h_wz.div_euclid(16));
            state.hovered_chunk = Some((ch_cx, ch_cz));

            let biome = world.get_biome_at(h_wx, h_wz);
            let dist_to_player = ((h_wx as f32 - player_pos.x).powi(2) + (h_wz as f32 - player_pos.z).powi(2)).sqrt();

            let is_sel = state.selected_chunks.contains(&(ch_cx, ch_cz));
            let sel_status = if is_sel { "[SELECTED] " } else { "" };

            let inspect_str = format!(
                "Block: X: {} Y: {} Z: {} ({}) | Chunk: [{}, {}] {}| Biome: {} | Dist: {:.0}m | Selected: {} chunks",
                h_wx, h_sy, h_wz, sblk.name(), ch_cx, ch_cz, sel_status, biome.name(), dist_to_player, state.selected_chunks.len()
            );
            draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &inspect_str, 12, buf_h as i32 - bottom_bar_h + 7, 1, 1, 0xFFFFFFFF);
        }
    }

    if state.hovered_world_pos.is_none() {
        let p_status = format!(
            "Player: X: {:.1}  Y: {:.1}  Z: {:.1} | Selected Chunks: {} | Total Waypoints: {}",
            player_pos.x, player_pos.y, player_pos.z, state.selected_chunks.len(), waypoints.len()
        );
        draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &p_status, 12, buf_h as i32 - bottom_bar_h + 7, 1, 1, 0xFFAAAAAA);
    }

    if let Some(ctx_menu) = &state.context_menu {
        let (card, item0, item1) =
            worldmap_context_menu_rects(ctx_menu.screen_pos.0, ctx_menu.screen_pos.1, buf_w, buf_h);
        let card_r = Rect::from_tuple(card);
        let item0_r = Rect::from_tuple(item0);
        let item1_r = Rect::from_tuple(item1);

        ui.fill_alpha(card_r.offset(3, 3), 0xFF000000, 0.55);
        ui.fill(card_r, 0xFF161822);
        ui.bevel_border(card_r, 1, 0xFF4A4E65, 0xFF2A2C38);

        let hover0 = item0_r.contains_mouse(mouse_pos);
        ui.fill(item0_r, if hover0 { 0xFF2B4D78 } else { 0xFF1F2332 });
        let txt0 = match lang {
            Language::Polish => "Zrób tutaj waypointa",
            Language::English => "Create waypoint here",
        };
        draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, txt0, item0_r.x + 8, item0_r.y + 6, 1, 1, if hover0 { 0xFFFFFF55 } else { 0xFFE0E0EE });

        let hover1 = item1_r.contains_mouse(mouse_pos);
        ui.fill(item1_r, if hover1 { 0xFF2B4D78 } else { 0xFF1F2332 });
        let txt1 = match lang {
            Language::Polish => "Udostępnij koordynaty na czacie",
            Language::English => "Share coordinates in chat",
        };
        draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, txt1, item1_r.x + 8, item1_r.y + 6, 1, 1, if hover1 { 0xFF55FFFF } else { 0xFFE0E0EE });
    }

    if state.showing_waypoint_manager {
        draw_waypoint_manager(
            ui.buffer,
            ui.width,
            ui.height,
            waypoints,
            player_pos,
            state.waypoint_manager_scroll,
        );
    }

    if state.creating_waypoint {
        draw_waypoint_dialog(ui.buffer, ui.width, ui.height, state);
    }
}
