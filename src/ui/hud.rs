use crate::gameplay::player::{GameMode, Player};
use crate::graphics::font;
use crate::ui::draw::{
    draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color, set_px,
};
use crate::world::block::Face;

pub fn draw_crosshair(buffer: &mut [u32], width: usize, height: usize) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let size = 6i32;
    let color = pack_color(1.0, 1.0, 1.0);
    for i in -size..=size {
        set_px(buffer, width, height, cx + i, cy, color);
        set_px(buffer, width, height, cx, cy + i, color);
    }
}

pub fn draw_break_progress(buffer: &mut [u32], width: usize, height: usize, progress: f32) {
    if progress <= 0.0 {
        return;
    }
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let bar_w = 40i32;
    let bar_h = 3i32;
    let bar_x = cx - bar_w / 2;
    let bar_y = cy + 14;

    fill_rect_alpha(buffer, width, height, bar_x - 1, bar_y - 1, bar_w + 2, bar_h + 2, pack_color(0.0, 0.0, 0.0), 0.25);
    let t = progress.clamp(0.0, 1.0);
    let fill_w = (bar_w as f32 * t) as i32;
    if fill_w > 0 {
        fill_rect_alpha(buffer, width, height, bar_x, bar_y, fill_w, bar_h, pack_color(0.85, 0.85, 0.85), 0.5);
    }
}

pub fn draw_underwater_overlay(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.04, 0.28, 0.48), 0.34);
    let wave = ((time * 1.8).sin() * 3.0) as i32;
    for y in (18 + wave)..height as i32 {
        if y % 31 == 0 {
            fill_rect_alpha(buffer, width, height, 0, y, width as i32, 1, pack_color(0.25, 0.65, 0.80), 0.08);
        }
    }
}

pub fn draw_hotbar(buffer: &mut [u32], width: usize, height: usize, player: &Player) {
    let slot_size = 28i32;
    let margin = 3i32;
    let total_slots = 10i32;
    let total_w = total_slots * (slot_size + margin) - margin;
    let start_x = (width as i32 - total_w) / 2;
    let y = height as i32 - slot_size - 12;

    fill_rect_alpha(buffer, width, height, start_x - 6, y - 6, total_w + 12, slot_size + 12, pack_color(0.0, 0.0, 0.0), 0.40);

    for i in 0..total_slots {
        let is_selected = i == player.selected_hotbar_slot as i32;
        let border_color = if is_selected {
            pack_color(1.0, 1.0, 0.2)
        } else {
            pack_color(0.20, 0.20, 0.20)
        };

        let x = start_x + i * (slot_size + margin);
        fill_rect(buffer, width, height, x - 2, y - 2, slot_size + 4, slot_size + 4, border_color);
        fill_rect(buffer, width, height, x, y, slot_size, slot_size, pack_color(0.3, 0.3, 0.3));

        if let Some(stack) = &player.inventory.hotbar[i as usize] {
            let color = stack.block.face_color(Face::Top);
            fill_rect(buffer, width, height, x + 4, y + 4, slot_size - 8, slot_size - 8, pack_color(color[0], color[1], color[2]));

            if stack.count > 1 {
                let label = format!("{}", stack.count);
                draw_text_shadowed(buffer, width, height, &label, x + slot_size - 10, y + slot_size - 12, 1, 1);
            }
        }
    }

    if player.gamemode == GameMode::Survival {
        let health_start_x = start_x;
        let health_y = y - 16;
        let hp = player.health.ceil() as i32;
        let max_hp = player.max_health as i32;
        for i in 0..(max_hp / 2) {
            let heart_x = health_start_x + i * 10;
            let color = if (i * 2 + 1) <= hp {
                pack_color(1.0, 0.1, 0.1)
            } else if (i * 2) < hp {
                pack_color(1.0, 0.5, 0.5)
            } else {
                pack_color(0.0, 0.0, 0.0)
            };
            fill_rect(buffer, width, height, heart_x, health_y, 8, 8, pack_color(0.1, 0.1, 0.1));
            fill_rect(buffer, width, height, heart_x + 1, health_y + 1, 6, 6, color);
        }
    }
}

pub fn draw_debug_panel(buffer: &mut [u32], width: usize, height: usize, lines: &[String]) {
    let scale = 2;
    let line_h = (font::GLYPH_H as i32) * scale + 4;
    let padding = 6;

    let max_w = lines
        .iter()
        .map(|l| font::measure_text(l, scale, 1))
        .max()
        .unwrap_or(0);

    let panel_w = max_w + padding * 2;
    let panel_h = lines.len() as i32 * line_h + padding * 2 - 4;

    fill_rect_alpha(buffer, width, height, 6, 6, panel_w, panel_h, pack_color(0.0, 0.0, 0.0), 0.4);

    for (i, line) in lines.iter().enumerate() {
        let ty = 6 + padding + i as i32 * line_h;
        draw_text_shadowed(buffer, width, height, line, 6 + padding, ty, scale, 1);
    }
}

pub fn draw_capture_prompt(buffer: &mut [u32], width: usize, height: usize, native_capture: bool) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.35);

    let line1 = "KLIKNIJ ABY GRAC";
    let line2 = if native_capture {
        "MYSZ ZOSTANIE ZABLOKOWANA W OKNIE"
    } else {
        "ESC - MENU PAUZY / WYJSCIE"
    };

    let scale1 = 4;
    let scale2 = 2;
    let w1 = font::measure_text(line1, scale1, 2);
    let w2 = font::measure_text(line2, scale2, 1);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    draw_text_shadowed(buffer, width, height, line1, cx - w1 / 2, cy - 20, scale1, 2);
    draw_text_shadowed(buffer, width, height, line2, cx - w2 / 2, cy + 20, scale2, 1);
}

pub fn pause_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 5] {
    let btn_w = 340;
    let btn_h = 36;
    let spacing = 10;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 - 110;

    [
        (cx - btn_w / 2, start_y, btn_w, btn_h),
        (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 3, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 4, btn_w, btn_h),
    ]
}

pub fn draw_pause_menu(buffer: &mut [u32], width: usize, height: usize, fly_mode: bool, hovered_idx: Option<usize>) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.65);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    let title = "MENU GRY";
    let t_scale = 4;
    let t_w = font::measure_text(title, t_scale, 2);
    draw_text_shadowed(buffer, width, height, title, cx - t_w / 2, cy - 150, t_scale, 2);

    let buttons = [
        "WZNOW GRE (ESC)",
        "USTAWIENIA GRAFIKI",
        if fly_mode { "TRYB LOTU: WLACZONY (F)" } else { "TRYB LOTU: WYLACZONY (F)" },
        "MENU GLOWNE (M)",
        "WYJDZ Z GRY (Q)",
    ];

    let rects = pause_menu_button_rects(width, height);

    for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);

        let body_color = if is_hover {
            pack_color(0.35, 0.45, 0.65)
        } else {
            pack_color(0.22, 0.22, 0.26)
        };
        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);

        let top_left = if is_hover {
            pack_color(0.75, 0.85, 1.0)
        } else {
            pack_color(0.55, 0.55, 0.60)
        };
        let bottom_right = if is_hover {
            pack_color(0.15, 0.20, 0.35)
        } else {
            pack_color(0.10, 0.10, 0.12)
        };

        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    let footer1 = "STEROWANIE: WASD - RUCH | SHIFT - KUCANIE | CTRL - BIEG | SPACJA - SKOK";
    let footer2 = "LPM - NISZCZ | PPM - STAWIAJ | SCROLL / 1-9 - WYBOR BLOKU";
    let f_scale = 1;
    let f1_w = font::measure_text(footer1, f_scale, 1);
    let f2_w = font::measure_text(footer2, f_scale, 1);
    draw_text_colored_shadowed(buffer, width, height, footer1, cx - f1_w / 2, cy + 135, f_scale, 1, pack_color(0.7, 0.7, 0.7));
    draw_text_colored_shadowed(buffer, width, height, footer2, cx - f2_w / 2, cy + 150, f_scale, 1, pack_color(0.7, 0.7, 0.7));
}
