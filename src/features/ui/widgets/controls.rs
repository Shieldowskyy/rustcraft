//! High-level composite widget rendering on `UiCanvas` (`button`, `text_input`, `progress_bar`, `scrollbar`, `item_slot`, `toast_card`).
//! Zero platform `#[cfg]`.

use crate::features::blocks::BlockId;
use crate::features::rendering::font;
use crate::features::ui::inventory::draw_item_icon;

use super::super::draw::pack_color;
use super::canvas::UiCanvas;
use super::layout::Rect;
use super::types::{Button, ButtonVariant, TextInput, Toast};

impl<'a> UiCanvas<'a> {
    pub fn button(&mut self, rect: impl Into<Rect>, btn: Button<'_>) {
        let r = rect.into();
        let active_or_hover = btn.hovered || btn.focused;

        let (body_color, top_left, bottom_right, default_text_col) = match btn.variant {
            ButtonVariant::Standard => (
                if active_or_hover {
                    pack_color(0.35, 0.45, 0.65)
                } else {
                    pack_color(0.22, 0.22, 0.26)
                },
                if btn.focused {
                    pack_color(0.98, 0.85, 0.25)
                } else if btn.hovered {
                    pack_color(0.75, 0.85, 1.0)
                } else {
                    pack_color(0.55, 0.55, 0.60)
                },
                if active_or_hover {
                    pack_color(0.15, 0.20, 0.35)
                } else {
                    pack_color(0.10, 0.10, 0.12)
                },
                if active_or_hover {
                    pack_color(1.0, 1.0, 0.3)
                } else {
                    pack_color(1.0, 1.0, 1.0)
                },
            ),
            ButtonVariant::Primary => (
                if active_or_hover {
                    pack_color(0.25, 0.60, 0.35)
                } else {
                    pack_color(0.18, 0.42, 0.25)
                },
                if btn.focused {
                    pack_color(1.0, 0.95, 0.3)
                } else if btn.hovered {
                    pack_color(0.60, 0.95, 0.65)
                } else {
                    pack_color(0.32, 0.65, 0.42)
                },
                if active_or_hover {
                    pack_color(0.10, 0.25, 0.15)
                } else {
                    pack_color(0.08, 0.18, 0.10)
                },
                pack_color(1.0, 1.0, 1.0),
            ),
            ButtonVariant::Danger => (
                if active_or_hover {
                    pack_color(0.48, 0.24, 0.24)
                } else {
                    pack_color(0.30, 0.18, 0.18)
                },
                if btn.focused {
                    pack_color(1.0, 0.95, 0.3)
                } else if btn.hovered {
                    pack_color(0.95, 0.55, 0.55)
                } else {
                    pack_color(0.55, 0.38, 0.38)
                },
                if active_or_hover {
                    pack_color(0.24, 0.08, 0.08)
                } else {
                    pack_color(0.12, 0.08, 0.08)
                },
                pack_color(1.0, 1.0, 1.0),
            ),
            ButtonVariant::Accent => (
                if active_or_hover {
                    pack_color(0.26, 0.44, 0.70)
                } else {
                    pack_color(0.18, 0.28, 0.44)
                },
                if btn.focused {
                    pack_color(1.0, 0.95, 0.3)
                } else if btn.hovered {
                    pack_color(0.55, 0.75, 1.0)
                } else {
                    pack_color(0.35, 0.50, 0.75)
                },
                if active_or_hover {
                    pack_color(0.12, 0.18, 0.30)
                } else {
                    pack_color(0.08, 0.12, 0.20)
                },
                pack_color(1.0, 1.0, 1.0),
            ),
            ButtonVariant::Tab { active } => (
                if active {
                    pack_color(0.30, 0.48, 0.65)
                } else if btn.hovered {
                    pack_color(0.24, 0.32, 0.45)
                } else {
                    pack_color(0.14, 0.16, 0.20)
                },
                if active {
                    pack_color(0.85, 0.92, 1.0)
                } else if btn.hovered {
                    pack_color(0.60, 0.70, 0.85)
                } else {
                    pack_color(0.35, 0.38, 0.45)
                },
                if active {
                    pack_color(0.18, 0.25, 0.38)
                } else {
                    pack_color(0.08, 0.09, 0.12)
                },
                if active {
                    pack_color(1.0, 0.95, 0.4)
                } else if btn.hovered {
                    pack_color(0.9, 0.9, 1.0)
                } else {
                    pack_color(1.0, 1.0, 1.0)
                },
            ),
            ButtonVariant::Flat {
                bg,
                hover_bg,
                border,
                hover_border,
            } => {
                let b_col = if btn.focused {
                    pack_color(0.98, 0.85, 0.25)
                } else if btn.hovered {
                    hover_border
                } else {
                    border
                };
                let body = if active_or_hover { hover_bg } else { bg };
                (body, b_col, b_col, pack_color(1.0, 1.0, 1.0))
            }
        };

        if btn.outer_border {
            self.framed_box(r, 2, top_left, body_color);
        } else if btn.beveled {
            self.fill(r, body_color);
            self.bevel_border(r, 2, top_left, bottom_right);
        } else {
            self.bordered_box(r, 1, body_color, top_left);
        }

        let text_col = btn.text_color.unwrap_or(default_text_col);
        self.text_in_rect(r, btn.label, btn.scale, 1, text_col);
    }

    pub fn text_input(&mut self, rect: impl Into<Rect>, input: TextInput<'_>) {
        let r = rect.into();

        if let Some(lbl) = input.label {
            self.text(lbl, r.x, r.y - 16, 1, 1, pack_color(0.85, 0.90, 0.95));
        }

        let border_col = if input.focused {
            if input.outer_border {
                pack_color(0.98, 0.85, 0.25)
            } else {
                pack_color(0.45, 0.70, 1.0)
            }
        } else if input.hovered {
            pack_color(0.5, 0.7, 1.0)
        } else {
            pack_color(0.25, 0.28, 0.36)
        };

        let bg_col = if input.focused && !input.outer_border {
            pack_color(0.09, 0.12, 0.18)
        } else {
            pack_color(0.06, 0.07, 0.09)
        };

        if input.outer_border {
            self.framed_box(r, 2, border_col, bg_col);
        } else {
            self.bordered_box(r, 2, bg_col, border_col);
        }

        let is_placeholder = input.text.is_empty() && !input.focused;
        let display_text = if is_placeholder {
            input.placeholder
        } else {
            input.text
        };

        let pad_x = if input.outer_border { 12 } else { 10 };
        let text_x = r.x + pad_x;
        let glyph_h = font::GLYPH_H as i32 * input.scale;
        let text_y = r.y + (r.h - glyph_h) / 2;

        let is_sel = input.focused && input.selected && !input.text.is_empty();
        if is_sel {
            let text_w = font::measure_text(display_text, input.scale, 1);
            self.fill(
                Rect::new(text_x - 2, text_y - 2, text_w + 4, glyph_h + 4),
                pack_color(0.2, 0.45, 0.85),
            );
        }

        let text_col = if is_placeholder {
            pack_color(0.48, 0.53, 0.60)
        } else {
            pack_color(1.0, 1.0, 1.0)
        };

        if !display_text.is_empty() {
            self.text(display_text, text_x, text_y, input.scale, 1, text_col);
        }

        if input.focused && !is_sel && input.cursor_visible {
            let text_w = font::measure_text(input.text, input.scale, 1);
            let cur_x = (text_x + text_w + 2).min(r.right() - 14);
            self.text("_", cur_x, text_y, input.scale, 1, pack_color(1.0, 0.9, 0.3));
        }
    }

    pub fn progress_bar(
        &mut self,
        rect: impl Into<Rect>,
        progress_ratio: f32,
        track_color: u32,
        fill_color: u32,
    ) {
        let r = rect.into();
        self.fill(r, track_color);
        let fill_w = (progress_ratio.clamp(0.0, 1.0) * r.w as f32).round() as i32;
        if fill_w > 0 {
            self.fill(Rect::new(r.x, r.y, fill_w, r.h), fill_color);
        }
    }

    pub fn scrollbar(
        &mut self,
        track_rect: impl Into<Rect>,
        visible_items: usize,
        total_items: usize,
        scroll_offset: usize,
        track_color: u32,
        thumb_color: u32,
    ) {
        if total_items <= visible_items || total_items == 0 {
            return;
        }
        let r = track_rect.into();
        self.fill(r, track_color);

        let max_scroll = total_items.saturating_sub(visible_items);
        let clamped_scroll = scroll_offset.min(max_scroll);
        let thumb_h = ((r.h * visible_items as i32) / total_items as i32).max(18);
        let thumb_y = if max_scroll > 0 {
            r.y + ((r.h - thumb_h) * clamped_scroll as i32) / max_scroll as i32
        } else {
            r.y
        };
        self.fill(Rect::new(r.x, thumb_y, r.w, thumb_h), thumb_color);
    }

    pub fn item_slot(
        &mut self,
        rect: impl Into<Rect>,
        block: BlockId,
        count: Option<u8>,
        bg_color: u32,
        border_color: u32,
    ) {
        let r = rect.into();
        self.bordered_box(r, 1, bg_color, border_color);
        let pad = 5.min(r.w / 6);
        let icon_size = (r.w - pad * 2).max(8);
        draw_item_icon(
            self.buffer,
            self.width,
            self.height,
            block,
            r.x + pad,
            r.y + pad,
            icon_size,
        );
        if let Some(c) = count {
            let cnt_str = format!("x{c}");
            self.text_white(&cnt_str, r.right() - 20, r.bottom() - 14, 1, 1);
        }
    }

    pub fn toast_card(&mut self, toast: Toast<'_>) {
        let box_w = (self.width as i32 - 32).clamp(180, 360);
        let box_h = 62;
        let x = self.width as i32 - box_w - 16;
        let y = toast.y;
        let r = Rect::new(x, y, box_w, box_h);

        self.fill_alpha(r, pack_color(0.035, 0.045, 0.065), 0.96);
        self.fill(Rect::new(x, y, box_w, 3), toast.accent_color);
        self.fill(Rect::new(x, y, 3, box_h), toast.accent_color);

        let text_x = if let Some(icon_block) = toast.icon {
            draw_item_icon(self.buffer, self.width, self.height, icon_block, x + 10, y + 15, 32);
            x + 50
        } else {
            x + 12
        };

        self.text(toast.heading, text_x, y + 8, 1, 1, toast.accent_color);

        let max_chars = ((box_w - (text_x - x) - 10) / 6).max(1) as usize;
        let title_trunc: String = toast.title.chars().take(max_chars).collect();
        self.text(&title_trunc, text_x, y + 24, 1, 1, 0xFFFFFFFF);

        let sub_trunc: String = toast.subtitle.chars().take(max_chars).collect();
        self.text(&sub_trunc, text_x, y + 40, 1, 1, pack_color(0.78, 0.84, 0.92));
    }
}
