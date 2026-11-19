//! Okna dialogowe menedżera znaczników (waypoints), tworzenia waypointów oraz znaczników 3D w świecie.

use crate::core::math::Vec3;
use crate::features::player::Camera;
use crate::features::ui::draw::{
    draw_text_light_colored_shadowed, draw_text_light_shadowed, fill_rect, set_px,
};
use crate::features::ui::widgets::{Rect, UiCanvas};

use super::types::{WAYPOINT_COLORS, Waypoint, WorldMapState};

pub fn worldmap_top_bar_buttons() -> (
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
) {
    let waypoints_btn = (175, 4, 105, 20);
    let new_wp_btn = (288, 4, 105, 20);
    let clear_chunks_btn = (401, 4, 120, 20);
    (waypoints_btn, new_wp_btn, clear_chunks_btn)
}

pub fn waypoint_manager_buttons(
    dx: i32,
    dy: i32,
    card_w: i32,
    card_h: i32,
) -> (
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
) {
    let header_close = (dx + card_w - 30, dy + 10, 20, 20);
    let new_wp_btn = (dx + 20, dy + card_h - 36, 130, 24);
    let bottom_close = (dx + card_w - 120, dy + card_h - 36, 100, 24);
    let scroll_up = (dx + card_w - 28, dy + 48, 18, 18);
    let scroll_down = (dx + card_w - 28, dy + card_h - 70, 18, 18);
    (header_close, new_wp_btn, bottom_close, scroll_up, scroll_down)
}

pub fn waypoint_row_buttons(
    dx: i32,
    list_y: i32,
    card_w: i32,
    row_idx: usize,
) -> (
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
) {
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
    let card = Rect::centered_in(buf_w, buf_h, 540, 360);
    let (dx, dy, card_w, card_h) = (card.x, card.y, card.w, card.h);

    let mut ui = UiCanvas::new(buffer, buf_w, buf_h);
    ui.backdrop(None, 0xFF000000, 0.60);
    ui.framed_box(card, 1, 0xFF3D3D54, 0xFF14141E);

    ui.text("WAYPOINT MANAGER", dx + 20, dy + 14, 1, 1, 0xFFFFDD44);
    let count_str = format!("({} waypoints)", waypoints.len());
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &count_str, dx + 180, dy + 15, 1, 1, 0xFFAAAAAA);

    let (header_close, new_wp_btn, bottom_close, scroll_up, scroll_down) =
        waypoint_manager_buttons(dx, dy, card_w, card_h);

    ui.light_button(header_close, "X", 0xFF7A2020, None, 0xFFFFFFFF);
    ui.fill(Rect::new(dx + 12, dy + 38, card_w - 24, 1), 0xFF2A2A3C);

    let list_y = dy + 46;
    let max_rows = 6usize;
    let end_idx = (scroll + max_rows).min(waypoints.len());

    if waypoints.is_empty() {
        draw_text_light_colored_shadowed(
            ui.buffer,
            ui.width,
            ui.height,
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
            ui.bordered_box(Rect::new(rx, ry, rw, 40), 1, row_bg, 0xFF2C2C3D);
            ui.framed_box(Rect::new(rx + 10, ry + 12, 16, 16), 1, 0xFF000000, wp.color);

            let name_color = if wp.enabled { 0xFFFFFFFF } else { 0xFF777788 };
            draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &wp.name, rx + 36, ry + 7, 1, 1, name_color);

            let dist = ((wp.x as f32 - player_pos.x).powi(2)
                + (wp.y as f32 - player_pos.y).powi(2)
                + (wp.z as f32 - player_pos.z).powi(2))
            .sqrt();
            let coords_str = format!("X: {} Y: {} Z: {} | {:.0}m", wp.x, wp.y, wp.z, dist);
            draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &coords_str, rx + 36, ry + 22, 1, 1, 0xFF888899);

            let (vis_btn, tp_btn, del_btn) = waypoint_row_buttons(dx, list_y, card_w, row_idx);
            let vis_bg = if wp.enabled { 0xFF2A602A } else { 0xFF444455 };
            let vis_txt = if wp.enabled { "VIS" } else { "HID" };
            ui.light_button(vis_btn, vis_txt, vis_bg, None, 0xFFFFFFFF);
            ui.light_button(tp_btn, "TP", 0xFF204875, None, 0xFFFFFFFF);
            ui.light_button(del_btn, "DEL", 0xFF7A2222, None, 0xFFFFFFFF);
        }

        if waypoints.len() > max_rows {
            ui.light_button(scroll_up, "^", 0xFF282838, None, 0xFFFFFFFF);
            ui.light_button(scroll_down, "v", 0xFF282838, None, 0xFFFFFFFF);
        }
    }

    ui.fill(Rect::new(dx + 12, dy + card_h - 44, card_w - 24, 1), 0xFF2A2A3C);
    ui.light_button(new_wp_btn, "[+] New Waypoint", 0xFF2A602A, None, 0xFFFFFFFF);
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, "[U/Esc] Close List", dx + 200, dy + card_h - 30, 1, 1, 0xFF888899);
    ui.light_button(bottom_close, "Close", 0xFF4A4A5A, None, 0xFFFFFFFF);
}

pub(super) fn draw_waypoint_dialog(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    state: &WorldMapState,
) {
    let dialog = Rect::centered_in(buf_w, buf_h, 320, 180);
    let (dx, dy, dia_w, dia_h) = (dialog.x, dialog.y, dialog.w, dialog.h);

    let mut ui = UiCanvas::new(buffer, buf_w, buf_h);
    ui.backdrop(None, 0xFF000000, 0.55);
    ui.framed_box(dialog, 1, 0xFF4A4A62, 0xFF1C1C26);

    ui.text("NEW WAYPOINT", dx + 20, dy + 16, 1, 1, 0xFFFFDD44);

    let name_str = format!("Name: {}_", state.waypoint_name_input);
    ui.fill(Rect::new(dx + 20, dy + 40, dia_w - 40, 24), 0xFF101018);
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &name_str, dx + 26, dy + 46, 1, 1, 0xFFFFFFFF);

    let coords_str = format!(
        "Coords: X: {}, Y: {}, Z: {}",
        state.waypoint_coords_input.0, state.waypoint_coords_input.1, state.waypoint_coords_input.2
    );
    draw_text_light_colored_shadowed(ui.buffer, ui.width, ui.height, &coords_str, dx + 20, dy + 76, 1, 1, 0xFFCCCCCC);

    draw_text_light_shadowed(ui.buffer, ui.width, ui.height, "Color:", dx + 20, dy + 98, 1, 1);
    for (i, &(c, _)) in WAYPOINT_COLORS.iter().enumerate() {
        let swatch = Rect::new(dx + 70 + (i as i32) * 28, dy + 95, 20, 18);
        if i == state.waypoint_color_idx {
            ui.framed_box(swatch, 1, 0xFFFFFFFF, c);
        } else {
            ui.fill(swatch, c);
        }
    }

    let (save_btn, cancel_btn) = waypoint_dialog_buttons(dx, dy, dia_w, dia_h);
    ui.light_button(save_btn, "[ENTER] Save", 0xFF2D6A2D, None, 0xFFFFFFFF);
    ui.light_button(cancel_btn, "[ESC] Cancel", 0xFF6A2D2D, None, 0xFFFFFFFF);
}

pub fn worldmap_context_menu_rects(
    screen_x: i32,
    screen_y: i32,
    buf_w: usize,
    buf_h: usize,
) -> (
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
    (i32, i32, i32, i32),
) {
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

pub fn waypoint_dialog_buttons(
    dx: i32,
    dy: i32,
    dia_w: i32,
    dia_h: i32,
) -> ((i32, i32, i32, i32), (i32, i32, i32, i32)) {
    let btn_w = 110i32;
    let btn_h = 24i32;
    let btn_y = dy + dia_h - 36;
    let save_rect = (dx + 25, btn_y, btn_w, btn_h);
    let cancel_rect = (dx + dia_w - btn_w - 25, btn_y, btn_w, btn_h);
    (save_rect, cancel_rect)
}

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

        let clip = view_proj.transform_point(world_pos);
        if clip[3] <= 0.01 {
            continue;
        }

        let ndc_x = clip[0] / clip[3];
        let ndc_y = clip[1] / clip[3];
        if ndc_x < -1.1 || ndc_x > 1.1 || ndc_y < -1.1 || ndc_y > 1.1 {
            continue;
        }

        let sx = ((ndc_x * 0.5 + 0.5) * buf_w as f32) as i32;
        let sy = (((-ndc_y) * 0.5 + 0.5) * buf_h as f32) as i32;

        if sx >= 10 && sx < buf_w as i32 - 10 && sy >= 10 && sy < buf_h as i32 - 10 {
            fill_rect(buffer, buf_w, buf_h, sx - 2, sy - 2, 5, 5, 0xFF000000);
            fill_rect(buffer, buf_w, buf_h, sx - 1, sy - 1, 3, 3, wp.color);
            set_px(buffer, buf_w, buf_h, sx, sy, 0xFFFFFFFF);

            let text = format!("{} ({:.0}m)", wp.name, dist);
            let text_w = text.len() as i32 * 6;
            let tx = (sx - text_w / 2).max(4);
            let ty = (sy - 14).max(4);
            draw_text_light_colored_shadowed(buffer, buf_w, buf_h, &text, tx, ty, 1, 1, wp.color);
        }
    }
}
