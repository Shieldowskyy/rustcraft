use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::storage::SaveMetadata;

pub const VISIBLE_SAVE_ROWS: usize = 4;
pub const MAX_SAVE_SLOTS: usize = 32;

pub fn save_card_rects(width: usize) -> [(i32, i32, i32, i32); VISIBLE_SAVE_ROWS] {
    let card_w = 680;
    let card_h = 66;
    let spacing = 10;
    let cx = width as i32 / 2;
    let start_y = 75;

    std::array::from_fn(|i| {
        let y = start_y + (i as i32) * (card_h + spacing);
        (cx - card_w / 2, y, card_w, card_h)
    })
}

pub fn save_bottom_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let btn_w = 160;
    let btn_h = 36;
    let spacing = 12;
    let total_w = 4 * btn_w + 3 * spacing;
    let start_x = (width as i32 - total_w) / 2;
    let y = height as i32 - 62;

    [
        (start_x, y, btn_w, btn_h),
        (start_x + btn_w + spacing, y, btn_w, btn_h),
        (start_x + (btn_w + spacing) * 2, y, btn_w, btn_h),
        (start_x + (btn_w + spacing) * 3, y, btn_w, btn_h),
    ]
}

pub fn draw_save_select_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    selected_slot: u8,
    scroll_offset: usize,
    metadata_cache: &[SaveMetadata; MAX_SAVE_SLOTS],
    hovered_card: Option<usize>,
    hovered_btn: Option<usize>,
) {
    let cx = width as i32 / 2;

    // Półprzezroczyste tło menu
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.02, 0.03, 0.05), 0.85);

    // Nagłówek
    let title = "WYBOR SWIATA (ZAPISY)";
    let title_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - title_w / 2, 22, 3, 2, pack_color(0.95, 0.85, 0.35));

    let subtitle = "Wybierz slot do gry lub wygeneruj nowy swiat";
    let subtitle_w = font::measure_text(subtitle, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, subtitle, cx - subtitle_w / 2, 52, 1, 1, pack_color(0.7, 0.75, 0.85));

    // Karty zapisów
    let card_rects = save_card_rects(width);
    for row in 0..VISIBLE_SAVE_ROWS {
        let slot_idx = scroll_offset + row;
        if slot_idx >= MAX_SAVE_SLOTS {
            break;
        }

        let slot_num = (slot_idx + 1) as u8;
        let is_selected = slot_num == selected_slot;
        let is_hovered = hovered_card == Some(row);
        let meta = &metadata_cache[slot_idx];
        let (bx, by, bw, bh) = card_rects[row];

        let border_color = if is_selected {
            pack_color(0.98, 0.85, 0.25)
        } else if is_hovered {
            pack_color(0.5, 0.7, 1.0)
        } else {
            pack_color(0.25, 0.28, 0.35)
        };

        let bg_color = if is_selected {
            pack_color(0.22, 0.28, 0.40)
        } else if is_hovered {
            pack_color(0.18, 0.22, 0.30)
        } else {
            pack_color(0.12, 0.14, 0.18)
        };

        // Ramka i tło karty
        fill_rect(buffer, width, height, bx - 2, by - 2, bw + 4, bh + 4, border_color);
        fill_rect(buffer, width, height, bx, by, bw, bh, bg_color);

        // Numer slotu
        let slot_badge = format!("SLOT {:02}", slot_num);
        let badge_color = if is_selected { pack_color(1.0, 0.95, 0.4) } else { pack_color(0.9, 0.9, 0.9) };
        draw_text_colored_shadowed(buffer, width, height, &slot_badge, bx + 16, by + 12, 2, 1, badge_color);

        if meta.exists {
            // Status ZAPISANY
            draw_text_colored_shadowed(buffer, width, height, "[ ZAPISANY ]", bx + 130, by + 13, 2, 1, pack_color(0.3, 0.95, 0.4));

            let size_kb = (meta.file_size_bytes as f32 / 1024.0).max(1.0) as u32;
            let line1 = format!("Seed: {}   |   Chunki: {} ({} KB)   |   {}", meta.seed, meta.chunk_count, size_kb, meta.last_modified_str);
            draw_text_colored_shadowed(buffer, width, height, &line1, bx + 16, by + 36, 1, 1, pack_color(0.85, 0.90, 0.95));

            let line2 = format!("Pozycja gracza: X: {:.0}  Y: {:.0}  Z: {:.0}", meta.player_pos.x, meta.player_pos.y, meta.player_pos.z);
            draw_text_colored_shadowed(buffer, width, height, &line2, bx + 16, by + 49, 1, 1, pack_color(0.65, 0.75, 0.85));
        } else {
            // Status PUSTY
            draw_text_colored_shadowed(buffer, width, height, "[ PUSTY SLOT ]", bx + 130, by + 13, 2, 1, pack_color(0.6, 0.65, 0.7));

            let line1 = "Nacisnij 'Graj' lub 'Nowy Swiat', aby wygenerowac swiat w tym slocie.";
            draw_text_colored_shadowed(buffer, width, height, line1, bx + 16, by + 40, 1, 1, pack_color(0.55, 0.60, 0.70));
        }
    }

    // Pasek przewijania (Scrollbar) po prawej stronie
    let sb_x = cx + 340 + 8;
    let sb_y = 75;
    let sb_h = (VISIBLE_SAVE_ROWS as i32) * 76 - 10;
    fill_rect(buffer, width, height, sb_x, sb_y, 6, sb_h, pack_color(0.15, 0.18, 0.22));

    let max_scroll = MAX_SAVE_SLOTS - VISIBLE_SAVE_ROWS;
    let thumb_h = ((VISIBLE_SAVE_ROWS as f32 / MAX_SAVE_SLOTS as f32) * sb_h as f32).max(18.0) as i32;
    let thumb_y = sb_y + ((scroll_offset as f32 / max_scroll as f32) * (sb_h - thumb_h) as f32) as i32;
    fill_rect(buffer, width, height, sb_x, thumb_y, 6, thumb_h, pack_color(0.5, 0.7, 1.0));

    // Dolny pasek przycisków
    let buttons = [
        "GRAJ (A)",
        "NOWY (Y)",
        "USUN (X)",
        "WSTECZ (B)",
    ];
    let btn_rects = save_bottom_button_rects(width, height);

    for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(btn_rects.iter()).enumerate() {
        let is_hover = hovered_btn == Some(i);

        let body_color = if is_hover {
            if i == 2 { pack_color(0.55, 0.2, 0.2) } else { pack_color(0.35, 0.45, 0.65) }
        } else {
            if i == 2 { pack_color(0.35, 0.15, 0.15) } else { pack_color(0.22, 0.22, 0.26) }
        };

        let top_left = if is_hover { pack_color(0.8, 0.9, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
        let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 1;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }
}
