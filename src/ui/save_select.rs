//! Save slot carousel and world selection menu rendering.

use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::storage::SaveMetadata;

/// Number of visible save slot cards displayed on screen simultaneously.
pub const VISIBLE_SAVE_ROWS: usize = 4;
/// Maximum number of save slots supported by the storage system.
pub const MAX_SAVE_SLOTS: usize = 32;

/// Calculates bounding rectangles for visible save slot cards.
pub fn save_card_rects(width: usize) -> [(i32, i32, i32, i32); VISIBLE_SAVE_ROWS] {
    let card_w = 680;
    let card_h = 70;
    let spacing = 10;
    let cx = width as i32 / 2;
    let start_y = 75;

    std::array::from_fn(|i| {
        let y = start_y + (i as i32) * (card_h + spacing);
        (cx - card_w / 2, y, card_w, card_h)
    })
}

/// Calculates bounding rectangles for the 4 bottom action buttons (Play, New, Delete, Back).
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

/// Draws the full Save Select screen with thumbnail previews and slot metadata.
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

    // Solid dark backdrop to prevent ghost frame artifacts
    fill_rect(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.06, 0.07, 0.09));
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.02, 0.03, 0.05), 0.65);

    // Header title
    let title = crate::core::i18n::t("save_select.title");
    let title_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - title_w / 2, 20, 3, 2, pack_color(0.95, 0.85, 0.35));

    let subtitle = "Choose a world to play or create a new one";
    let subtitle_w = font::measure_text(subtitle, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, subtitle, cx - subtitle_w / 2, 50, 1, 1, pack_color(0.7, 0.75, 0.85));

    // Save slot cards
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

        // Card frame and background
        fill_rect(buffer, width, height, bx - 2, by - 2, bw + 4, bh + 4, border_color);
        fill_rect(buffer, width, height, bx, by, bw, bh, bg_color);

        // Screenshot / World icon on the left (56x56 px, Minecraft style)
        let thumb_x = bx + 7;
        let thumb_y = by + 7;
        let thumb_size = 56i32;

        // Thumbnail frame
        fill_rect(buffer, width, height, thumb_x - 1, thumb_y - 1, thumb_size + 2, thumb_size + 2, pack_color(0.08, 0.09, 0.12));

        if let Some(ref thumb) = meta.thumbnail {
            for ty in 0..56 {
                for tx in 0..56 {
                    let pixel = thumb[ty * 56 + tx];
                    let px = thumb_x + tx as i32;
                    let py = thumb_y + ty as i32;
                    if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                        buffer[py as usize * width + px as usize] = pixel;
                    }
                }
            }
        } else if meta.exists {
            // Default terrain graphic when no snapshot exists
            fill_rect(buffer, width, height, thumb_x, thumb_y, thumb_size, thumb_size, pack_color(0.2, 0.45, 0.8));
            fill_rect(buffer, width, height, thumb_x, thumb_y + 24, thumb_size, thumb_size - 24, pack_color(0.35, 0.22, 0.12));
            fill_rect(buffer, width, height, thumb_x, thumb_y + 24, thumb_size, 6, pack_color(0.3, 0.7, 0.2));
            // Cloudlet
            fill_rect(buffer, width, height, thumb_x + 8, thumb_y + 8, 22, 7, pack_color(0.95, 0.95, 1.0));
            fill_rect(buffer, width, height, thumb_x + 32, thumb_y + 12, 16, 5, pack_color(0.95, 0.95, 1.0));
        } else {
            // Empty slot icon
            fill_rect(buffer, width, height, thumb_x, thumb_y, thumb_size, thumb_size, pack_color(0.14, 0.16, 0.20));
            let icon_cx = thumb_x + thumb_size / 2;
            let icon_cy = thumb_y + thumb_size / 2;
            fill_rect(buffer, width, height, icon_cx - 10, icon_cy - 2, 20, 4, pack_color(0.3, 0.35, 0.42));
            fill_rect(buffer, width, height, icon_cx - 2, icon_cy - 10, 4, 20, pack_color(0.3, 0.35, 0.42));
        }

        // Textual metadata to the right of the thumbnail
        let text_x = bx + 72;

        if meta.exists {
            // Line 1: World Name + Gamemode & Slot Badges
            let name_color = if is_selected { pack_color(1.0, 0.95, 0.4) } else { pack_color(0.95, 0.95, 0.98) };
            draw_text_colored_shadowed(buffer, width, height, &meta.name, text_x, by + 8, 2, 1, name_color);

            let name_w = font::measure_text(&meta.name, 2, 1);
            let badge_x = (text_x + name_w + 14).min(bx + bw - 210);

            // Game mode badge
            let (mode_text, mode_color) = match meta.gamemode {
                crate::gameplay::player::GameMode::Survival => (crate::core::i18n::t("create_world.mode_survival"), pack_color(0.35, 0.95, 0.4)),
                crate::gameplay::player::GameMode::Creative => (crate::core::i18n::t("create_world.mode_creative"), pack_color(0.35, 0.85, 1.0)),
            };
            draw_text_colored_shadowed(buffer, width, height, &format!("[ {} ]", mode_text), badge_x, by + 10, 1, 1, mode_color);

            // Slot badge on far right
            let slot_badge = format!("SLOT {:02}", slot_num);
            let slot_w = font::measure_text(&slot_badge, 1, 1);
            draw_text_colored_shadowed(buffer, width, height, &slot_badge, bx + bw - slot_w - 14, by + 10, 1, 1, pack_color(0.7, 0.75, 0.85));

            // Line 2: Seed, Chunks, Size, Modified Date
            let size_kb = (meta.file_size_bytes as f32 / 1024.0).max(1.0) as u32;
            let line1 = format!("Seed: {}   |   Chunks: {} ({} KB)   |   {}", meta.seed, meta.chunk_count, size_kb, meta.last_modified_str);
            draw_text_colored_shadowed(buffer, width, height, &line1, text_x, by + 32, 1, 1, pack_color(0.85, 0.90, 0.95));

            // Line 3: Player Coordinates
            let line2 = format!("Player Pos: X: {:.0}  Y: {:.0}  Z: {:.0}", meta.player_pos.x, meta.player_pos.y, meta.player_pos.z);
            draw_text_colored_shadowed(buffer, width, height, &line2, text_x, by + 47, 1, 1, pack_color(0.65, 0.75, 0.85));
        } else {
            // Empty Slot Info
            let empty_title = format!("[ {} {:02} ]", crate::core::i18n::t("save_select.empty_slot"), slot_num);
            draw_text_colored_shadowed(buffer, width, height, &empty_title, text_x, by + 14, 2, 1, pack_color(0.6, 0.65, 0.7));

            let line1 = "Press 'New' or 'Play' to create and configure a world.";
            draw_text_colored_shadowed(buffer, width, height, line1, text_x, by + 39, 1, 1, pack_color(0.55, 0.60, 0.70));
        }
    }

    // Scrollbar on right edge
    let sb_x = cx + 340 + 8;
    let sb_y = 75;
    let sb_h = (VISIBLE_SAVE_ROWS as i32) * 80 - 10;
    fill_rect(buffer, width, height, sb_x, sb_y, 6, sb_h, pack_color(0.15, 0.18, 0.22));

    let max_scroll = MAX_SAVE_SLOTS - VISIBLE_SAVE_ROWS;
    let thumb_h = ((VISIBLE_SAVE_ROWS as f32 / MAX_SAVE_SLOTS as f32) * sb_h as f32).max(18.0) as i32;
    let thumb_y = sb_y + ((scroll_offset as f32 / max_scroll as f32) * (sb_h - thumb_h) as f32) as i32;
    fill_rect(buffer, width, height, sb_x, thumb_y, 6, thumb_h, pack_color(0.5, 0.7, 1.0));

    // Bottom action buttons
    let buttons = [
        crate::core::i18n::t("save_select.play"),
        crate::core::i18n::t("save_select.create"),
        crate::core::i18n::t("save_select.delete"),
        crate::core::i18n::t("save_select.back"),
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

    #[cfg(target_os = "vita")]
    let footer_tooltip = "[D-PAD] SELECT | [L/R] SCROLL | [X] PLAY | [TRIANGLE] NEW | [SQUARE] DELETE | [O] BACK";
    #[cfg(not(target_os = "vita"))]
    let footer_tooltip = "[ARROWS / D-PAD] SELECT | [ENTER/A] PLAY | [N/Y] NEW | [DEL/X] DELETE | [ESC/B] BACK";

    let ft_w = font::measure_text(footer_tooltip, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, footer_tooltip, cx - ft_w / 2, height as i32 - 18, 1, 1, pack_color(0.65, 0.72, 0.85));
}
