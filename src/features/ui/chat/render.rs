//! Renderowanie historii wiadomości czatu, aktywnego paska wpisywania tekstu oraz menu kontekstowego.

use crate::features::rendering::font;
use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Rect, UiCanvas};

use super::state::ChatState;

impl ChatState {
    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize) {
        let base_x = 8i32;
        let line_height = 11i32;
        let max_visible_lines = 12;
        let input_y = height as i32 - 18;
        let mut ui = UiCanvas::new(buffer, width, height);

        let (rows, row_count) = self.visible_message_rows(height);
        for (index, y) in rows.into_iter().take(row_count.min(max_visible_lines)) {
            let msg = &self.messages[index];
            let alpha = if self.is_open {
                0.70f32
            } else {
                (1.0 - (msg.age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0) * 0.70
            };
            let text_w = font::measure_text(&msg.text, 1, 1);
            let bar_w = (text_w + 6).min(width as i32 - base_x * 2);

            ui.fill_alpha(
                Rect::new(base_x - 3, y - 2, bar_w, line_height),
                pack_color(0.0, 0.0, 0.0),
                alpha,
            );

            if let (Some(anchor), Some(focus)) = (self.selection_anchor, self.selection_focus) {
                let (start, end) = if anchor <= focus {
                    (anchor, focus)
                } else {
                    (focus, anchor)
                };
                if index >= start.0 && index <= end.0 {
                    let chars: Vec<char> = msg.text.chars().collect();
                    let from = if index == start.0 { start.1 } else { 0 };
                    let to = if index == end.0 { end.1 } else { chars.len() };
                    if from < to && to <= chars.len() {
                        let prefix: String = chars[..from].iter().collect();
                        let selected: String = chars[from..to].iter().collect();
                        let selection_x = base_x + font::measure_text(&prefix, 1, 1);
                        let selection_w = font::measure_text(&selected, 1, 1);
                        ui.fill_alpha(
                            Rect::new(selection_x, y - 1, selection_w.max(1), 9),
                            pack_color(0.18, 0.40, 0.78),
                            0.90,
                        );
                    }
                }
            }

            let text_alpha = if self.is_open {
                1.0
            } else {
                (1.0 - (msg.age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0)
            };

            if text_alpha > 0.05 {
                ui.text(&msg.text, base_x, y, 1, 1, msg.color);
            }
        }

        if self.is_open {
            let bar_w = (width as i32 - base_x * 2).max(100);
            let bar_h = 14i32;

            ui.fill_alpha(
                Rect::new(base_x - 3, input_y - 2, bar_w + 6, bar_h),
                pack_color(0.0, 0.0, 0.0),
                0.80,
            );
            ui.fill_alpha(
                Rect::new(base_x - 3, input_y + bar_h - 2, bar_w + 6, 1),
                pack_color(0.45, 0.45, 0.45),
                0.90,
            );

            let prefix = "> ";
            let display_str = format!("{}{}", prefix, self.input_text);
            let white = pack_color(1.0, 1.0, 1.0);
            if let Some((start, end)) = self.input_selection {
                if start < end && end <= self.input_text.len() {
                    if let (Some(selected), Some(before)) = (
                        self.input_text.get(start..end),
                        self.input_text.get(..start),
                    ) {
                        let sx = base_x + font::measure_text(&format!("{prefix}{before}"), 1, 1);
                        let sw = font::measure_text(selected, 1, 1);
                        ui.fill_alpha(
                            Rect::new(sx, input_y, sw, 9),
                            pack_color(0.18, 0.40, 0.78),
                            0.90,
                        );
                    }
                }
            }
            ui.text(&display_str, base_x, input_y + 1, 1, 1, white);

            let show_cursor = (self.cursor_blink % 0.8) < 0.4;
            if show_cursor {
                let cursor_sub = format!("{}{}", prefix, &self.input_text[..self.cursor_pos]);
                let cursor_x = base_x + font::measure_text(&cursor_sub, 1, 1);
                ui.fill(Rect::new(cursor_x, input_y + 1, 2, 7), white);
            }
        }

        if let Some(menu) = self.context_menu {
            let menu_rect = Rect::new(menu.x, menu.y, 116, 57);
            ui.fill_alpha(menu_rect, pack_color(0.08, 0.09, 0.12), 0.98);
            ui.bevel_border(
                menu_rect,
                1,
                pack_color(0.55, 0.58, 0.64),
                pack_color(0.22, 0.24, 0.28),
            );
            for (index, label) in ["Copy", "Paste", "Select all"].iter().enumerate() {
                ui.text(
                    label,
                    menu.x + 8,
                    menu.y + 5 + index as i32 * 19,
                    1,
                    1,
                    pack_color(0.95, 0.95, 0.97),
                );
            }
        }
    }
}
