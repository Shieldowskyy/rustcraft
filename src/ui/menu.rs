use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

pub fn main_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 3] {
    let btn_w = 340;
    let btn_h = 36;
    let spacing = 12;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 + 45;
    [
        (cx - btn_w / 2, start_y, btn_w, btn_h),
        (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
    ]
}

pub fn main_menu_world_rects(width: usize) -> [(i32, i32, i32, i32); 5] {
    let row_w = 340;
    let row_h = 26;
    let cx = width as i32 / 2;
    let start_y = 160;
    [
        (cx - row_w / 2, start_y, row_w, row_h),
        (cx - row_w / 2, start_y + 30, row_w, row_h),
        (cx - row_w / 2, start_y + 60, row_w, row_h),
        (cx - row_w / 2, start_y + 90, row_w, row_h),
        (cx - row_w / 2, start_y + 120, row_w, row_h),
    ]
}

pub fn draw_rustcraft_logo(buffer: &mut [u32], width: usize, height: usize, cx: i32, y: i32) {
    let gold = pack_color(0.95, 0.72, 0.20);
    let light = pack_color(1.0, 0.90, 0.48);
    let dark = pack_color(0.38, 0.22, 0.08);
    let blocks = [
        (-48, 0, 8, 8, gold), (-40, 0, 8, 8, light), (-32, 0, 8, 8, gold),
        (-24, 0, 8, 8, dark), (-16, 0, 8, 8, gold), (-8, 0, 8, 8, light),
        (-40, 8, 8, 8, gold), (-32, 16, 8, 8, light), (-24, 24, 8, 8, gold),
        (-16, 32, 8, 8, dark), (-8, 40, 8, 8, gold), (0, 32, 8, 8, light),
        (8, 24, 8, 8, gold), (16, 16, 8, 8, dark), (24, 8, 8, 8, gold),
        (32, 0, 8, 8, light), (40, -8, 8, 8, gold),
    ];
    for (x, offset_y, w, h, color) in blocks {
        fill_rect(buffer, width, height, cx + x, y + offset_y, w, h, color);
    }
}

pub fn draw_main_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    seed: u32,
    save_slot: u8,
    save_slots: &[bool; 32],
    hovered_idx: Option<usize>,
) {
    let cx = width as i32 / 2;
    fill_rect_alpha(buffer, width, height, cx - 230, 25, 460, 475, pack_color(0.02, 0.03, 0.05), 0.72);
    draw_rustcraft_logo(buffer, width, height, cx, 48);

    let title = "RUSTCRAFT";
    let title_w = font::measure_text(title, 4, 3);
    draw_text_colored_shadowed(buffer, width, height, title, cx - title_w / 2, 115, 4, 3, pack_color(0.95, 0.85, 0.42));

    let subtitle = "WOKSELOWY SWIAT";
    let subtitle_w = font::measure_text(subtitle, 2, 2);
    draw_text_shadowed(buffer, width, height, subtitle, cx - subtitle_w / 2, 158, 2, 2);

    let world_rects = main_menu_world_rects(width);
    for (row, &(bx, by, bw, bh)) in world_rects.iter().enumerate() {
        let slot = ((save_slot as i32 - 1 + row as i32 + 32 - 2).rem_euclid(32) + 1) as u8;
        let selected = slot == save_slot;
        let body = if selected { pack_color(0.32, 0.38, 0.50) } else { pack_color(0.14, 0.15, 0.18) };
        fill_rect(buffer, width, height, bx, by, bw, bh, body);
        if selected {
            fill_rect(buffer, width, height, bx, by, 3, bh, pack_color(0.95, 0.78, 0.25));
        }
        let status = if save_slots[slot as usize - 1] { "ZAPIS" } else { "PUSTY" };
        let label = format!("SWIAT {:02}   {}", slot, status);
        draw_text_shadowed(buffer, width, height, &label, bx + 12, by + 7, 1, 1);
    }

    let buttons = [
        "NOWY SWIAT",
        if save_slots[save_slot as usize - 1] { "WCZYTAJ ZAPIS" } else { "WCZYTAJ ZAPIS (BRAK)" },
        "WYJDZ Z GRY",
    ];
    let rects = main_menu_button_rects(width, height);
    for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);
        let body_color = if is_hover { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.22, 0.22, 0.26) };
        let top_left = if is_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
        let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - font::GLYPH_H as i32 * scale) / 2;
        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    let seed_label = format!("SEED: {}", seed);
    draw_text_shadowed(buffer, width, height, &seed_label, cx - (seed_label.len() as i32 * 4), 475, 1, 1);
    let slot_label = format!("SLOT ZAPISU: {:02}", save_slot);
    draw_text_shadowed(buffer, width, height, &slot_label, cx - (slot_label.len() as i32 * 4), 492, 1, 1);
}
