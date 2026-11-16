//! Save slot carousel and world selection menu rendering.

use crate::graphics::font;
use crate::ui::draw::pack_color;
use crate::ui::widgets::{Button, HStack, Rect, UiCanvas, VStack};
use crate::world::storage::SaveMetadata;

/// Number of visible save slot cards displayed on screen simultaneously.
pub const VISIBLE_SAVE_ROWS: usize = 4;
/// Maximum number of save slots supported by the storage system.
pub const MAX_SAVE_SLOTS: usize = 32;

/// Calculates bounding rectangles for visible save slot cards.
pub fn save_card_rects(width: usize) -> [(i32, i32, i32, i32); VISIBLE_SAVE_ROWS] {
    VStack::centered(width, 75, 680, 10).array(70)
}

/// Calculates bounding rectangles for the 4 bottom action buttons (Play, New, Delete, Back).
pub fn save_bottom_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    HStack::centered_row(width, height as i32 - 62, 160, 36, 4, 12).array(160)
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
    is_gamepad_active: bool,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();

    // Solid dark backdrop to prevent ghost frame artifacts
    ui.backdrop(
        Some(pack_color(0.06, 0.07, 0.09)),
        pack_color(0.02, 0.03, 0.05),
        0.65,
    );

    // Header title
    ui.text_centered_x(
        crate::core::i18n::t("save_select.title"),
        cx,
        20,
        3,
        2,
        pack_color(0.95, 0.85, 0.35),
    );
    ui.text_centered_x(
        "Choose a world to play or create a new one",
        cx,
        50,
        1,
        1,
        pack_color(0.7, 0.75, 0.85),
    );

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
        let card = Rect::from_tuple(card_rects[row]);

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

        ui.framed_box(card, 2, border_color, bg_color);

        // Screenshot / World icon on the left (56x56 px, Minecraft style)
        let thumb_rect = Rect::new(card.x + 7, card.y + 7, 56, 56);
        ui.fill(thumb_rect.inset(-1), pack_color(0.08, 0.09, 0.12));

        if let Some(ref thumb) = meta.thumbnail {
            for ty in 0..56 {
                for tx in 0..56 {
                    let pixel = thumb[ty * 56 + tx];
                    let px = thumb_rect.x + tx as i32;
                    let py = thumb_rect.y + ty as i32;
                    if px >= 0 && px < ui.width as i32 && py >= 0 && py < ui.height as i32 {
                        ui.buffer[py as usize * ui.width + px as usize] = pixel;
                    }
                }
            }
        } else if meta.exists {
            // Default terrain graphic when no snapshot exists
            ui.fill(thumb_rect, pack_color(0.2, 0.45, 0.8));
            ui.fill(
                Rect::new(thumb_rect.x, thumb_rect.y + 24, 56, 32),
                pack_color(0.35, 0.22, 0.12),
            );
            ui.fill(
                Rect::new(thumb_rect.x, thumb_rect.y + 24, 56, 6),
                pack_color(0.3, 0.7, 0.2),
            );
            // Cloudlet
            ui.fill(
                Rect::new(thumb_rect.x + 8, thumb_rect.y + 8, 22, 7),
                pack_color(0.95, 0.95, 1.0),
            );
            ui.fill(
                Rect::new(thumb_rect.x + 32, thumb_rect.y + 12, 16, 5),
                pack_color(0.95, 0.95, 1.0),
            );
        } else {
            // Empty slot icon
            ui.fill(thumb_rect, pack_color(0.14, 0.16, 0.20));
            let icon_cx = thumb_rect.center_x();
            let icon_cy = thumb_rect.center_y();
            ui.fill(
                Rect::new(icon_cx - 10, icon_cy - 2, 20, 4),
                pack_color(0.3, 0.35, 0.42),
            );
            ui.fill(
                Rect::new(icon_cx - 2, icon_cy - 10, 4, 20),
                pack_color(0.3, 0.35, 0.42),
            );
        }

        // Textual metadata to the right of the thumbnail
        let text_x = card.x + 72;

        if meta.exists {
            // Line 1: World Name + Gamemode & Slot Badges
            let name_color = if is_selected {
                pack_color(1.0, 0.95, 0.4)
            } else {
                pack_color(0.95, 0.95, 0.98)
            };
            ui.text(&meta.name, text_x, card.y + 8, 2, 1, name_color);

            let name_w = font::measure_text(&meta.name, 2, 1);
            let badge_x = (text_x + name_w + 14).min(card.right() - 210);

            let (mode_text, mode_color) = match meta.gamemode {
                crate::gameplay::player::GameMode::Survival => (
                    crate::core::i18n::t("create_world.mode_survival"),
                    pack_color(0.35, 0.95, 0.4),
                ),
                crate::gameplay::player::GameMode::Creative => (
                    crate::core::i18n::t("create_world.mode_creative"),
                    pack_color(0.35, 0.85, 1.0),
                ),
            };
            let badge_text = if meta.generator_type == crate::world::WorldGeneratorType::Superflat {
                format!("[ {} | FLAT ]", mode_text)
            } else {
                format!("[ {} ]", mode_text)
            };
            ui.text(
                &badge_text,
                badge_x,
                card.y + 10,
                1,
                1,
                mode_color,
            );

            // Slot badge on far right
            let slot_badge = format!("SLOT {:02}", slot_num);
            ui.text_right(
                &slot_badge,
                card.right() - 14,
                card.y + 10,
                1,
                1,
                pack_color(0.7, 0.75, 0.85),
            );

            // Line 2: Seed, Chunks, Size, Modified Date
            let size_kb = (meta.file_size_bytes as f32 / 1024.0).max(1.0) as u32;
            let line1 = format!(
                "Seed: {}   |   Chunks: {} ({} KB)   |   {}",
                meta.seed, meta.chunk_count, size_kb, meta.last_modified_str
            );
            ui.text(&line1, text_x, card.y + 32, 1, 1, pack_color(0.85, 0.90, 0.95));

            // Line 3: Player Coordinates
            let line2 = format!(
                "Player Pos: X: {:.0}  Y: {:.0}  Z: {:.0}",
                meta.player_pos.x, meta.player_pos.y, meta.player_pos.z
            );
            ui.text(&line2, text_x, card.y + 47, 1, 1, pack_color(0.65, 0.75, 0.85));
        } else {
            // Empty Slot Info
            let empty_title = format!(
                "[ {} {:02} ]",
                crate::core::i18n::t("save_select.empty_slot"),
                slot_num
            );
            ui.text(&empty_title, text_x, card.y + 14, 2, 1, pack_color(0.6, 0.65, 0.7));
            ui.text(
                "Press 'New' or 'Play' to create and configure a world.",
                text_x,
                card.y + 39,
                1,
                1,
                pack_color(0.55, 0.60, 0.70),
            );
        }
    }

    // Scrollbar on right edge
    let sb_h = (VISIBLE_SAVE_ROWS as i32) * 80 - 10;
    ui.scrollbar(
        Rect::new(cx + 340 + 8, 75, 6, sb_h),
        VISIBLE_SAVE_ROWS,
        MAX_SAVE_SLOTS,
        scroll_offset,
        pack_color(0.15, 0.18, 0.22),
        pack_color(0.5, 0.7, 1.0),
    );

    // Bottom action buttons
    let buttons = [
        crate::core::i18n::t("save_select.play"),
        crate::core::i18n::t("save_select.create"),
        crate::core::i18n::t("save_select.delete"),
        crate::core::i18n::t("save_select.back"),
    ];
    let btn_rects = save_bottom_button_rects(width, height);

    for (i, (&label, &rect)) in buttons.iter().zip(btn_rects.iter()).enumerate() {
        let mut btn = Button::new(label).scale(1).hovered(hovered_btn == Some(i));
        if i == 2 {
            btn = btn.danger();
        }
        ui.button(rect, btn);
    }

    let footer_tooltip = if is_gamepad_active {
        "[D-PAD / L-STICK] SELECT | [A] PLAY | [Y] NEW | [X] DELETE | [B] BACK"
    } else {
        "[ARROWS / MOUSE] SELECT | [ENTER] PLAY | [N] NEW | [DEL] DELETE | [ESC] BACK"
    };
    ui.footer(footer_tooltip, true);
}
