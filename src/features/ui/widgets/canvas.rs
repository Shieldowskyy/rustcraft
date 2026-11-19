//! `UiCanvas` primitive framebuffer, panel, slot, and typography operations.
//! Zero platform `#[cfg]`.

use crate::features::player::ItemStack;
use crate::features::rendering::font;
use crate::features::ui::inventory::draw_item_icon;

use super::super::draw::{
    draw_text_colored_shadowed, draw_text_light_colored_shadowed, draw_text_light_shadowed,
    draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color,
};
use super::layout::{Rect, format_u8};

/// Zero-allocation rendering wrapper around the raw pixel framebuffer `(buffer, width, height)`.
pub struct UiCanvas<'a> {
    pub buffer: &'a mut [u32],
    pub width: usize,
    pub height: usize,
}

impl<'a> UiCanvas<'a> {
    #[inline]
    pub fn new(buffer: &'a mut [u32], width: usize, height: usize) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    #[inline]
    pub const fn cx(&self) -> i32 {
        self.width as i32 / 2
    }

    #[inline]
    pub const fn cy(&self) -> i32 {
        self.height as i32 / 2
    }

    #[inline]
    pub const fn screen_rect(&self) -> Rect {
        Rect::new(0, 0, self.width as i32, self.height as i32)
    }

    #[inline]
    pub fn fill(&mut self, rect: impl Into<Rect>, color: u32) {
        let r = rect.into();
        fill_rect(self.buffer, self.width, self.height, r.x, r.y, r.w, r.h, color);
    }

    #[inline]
    pub fn fill_alpha(&mut self, rect: impl Into<Rect>, color: u32, alpha: f32) {
        let r = rect.into();
        fill_rect_alpha(
            self.buffer,
            self.width,
            self.height,
            r.x,
            r.y,
            r.w,
            r.h,
            color,
            alpha,
        );
    }

    pub fn backdrop(&mut self, solid_color: Option<u32>, overlay_color: u32, alpha: f32) {
        let full = self.screen_rect();
        if let Some(solid) = solid_color {
            self.fill(full, solid);
        }
        if alpha > 0.0 {
            self.fill_alpha(full, overlay_color, alpha);
        }
    }

    pub fn border(&mut self, rect: impl Into<Rect>, thickness: i32, color: u32) {
        let r = rect.into();
        if thickness <= 0 || r.w <= 0 || r.h <= 0 {
            return;
        }
        fill_rect(self.buffer, self.width, self.height, r.x, r.y, r.w, thickness, color);
        fill_rect(
            self.buffer,
            self.width,
            self.height,
            r.x,
            r.y + r.h - thickness,
            r.w,
            thickness,
            color,
        );
        fill_rect(self.buffer, self.width, self.height, r.x, r.y, thickness, r.h, color);
        fill_rect(
            self.buffer,
            self.width,
            self.height,
            r.x + r.w - thickness,
            r.y,
            thickness,
            r.h,
            color,
        );
    }

    pub fn bevel_border(
        &mut self,
        rect: impl Into<Rect>,
        thickness: i32,
        top_left_color: u32,
        bottom_right_color: u32,
    ) {
        let r = rect.into();
        fill_rect(self.buffer, self.width, self.height, r.x, r.y, r.w, thickness, top_left_color);
        fill_rect(self.buffer, self.width, self.height, r.x, r.y, thickness, r.h, top_left_color);
        fill_rect(
            self.buffer,
            self.width,
            self.height,
            r.x,
            r.y + r.h - thickness,
            r.w,
            thickness,
            bottom_right_color,
        );
        fill_rect(
            self.buffer,
            self.width,
            self.height,
            r.x + r.w - thickness,
            r.y,
            thickness,
            r.h,
            bottom_right_color,
        );
    }

    pub fn framed_box(
        &mut self,
        rect: impl Into<Rect>,
        outer_thickness: i32,
        border_color: u32,
        bg_color: u32,
    ) {
        let r = rect.into();
        self.fill(r.inset(-outer_thickness), border_color);
        self.fill(r, bg_color);
    }

    pub fn bordered_box(
        &mut self,
        rect: impl Into<Rect>,
        thickness: i32,
        bg_color: u32,
        border_color: u32,
    ) {
        let r = rect.into();
        self.fill(r, bg_color);
        self.border(r, thickness, border_color);
    }

    pub fn modal_panel(
        &mut self,
        rect: impl Into<Rect>,
        bg_color: u32,
        alpha: f32,
        top_left_border: u32,
        bottom_right_border: u32,
    ) {
        let r = rect.into();
        self.fill_alpha(r, bg_color, alpha);
        self.bevel_border(r, 2, top_left_border, bottom_right_border);
    }

    pub fn inset_panel(&mut self, rect: impl Into<Rect>, bg_color: u32, alpha: f32) {
        let r = rect.into();
        self.fill_alpha(r, bg_color, alpha);
        self.bevel_border(
            r,
            2,
            pack_color(0.05, 0.05, 0.06),
            pack_color(0.26, 0.26, 0.30),
        );
    }

    pub fn light_button(
        &mut self,
        rect: impl Into<Rect>,
        label: &str,
        bg_color: u32,
        border_color: Option<u32>,
        text_color: u32,
    ) {
        let r = rect.into();
        if let Some(border) = border_color {
            self.bordered_box(r, 1, bg_color, border);
        } else {
            self.fill(r, bg_color);
        }
        let tw = font::measure_text(label, 1, 1);
        let tx = r.x + (r.w - tw) / 2;
        let ty = r.y + (r.h - font::GLYPH_H as i32) / 2;
        draw_text_light_colored_shadowed(
            self.buffer,
            self.width,
            self.height,
            label,
            tx,
            ty,
            1,
            1,
            text_color,
        );
    }

    pub fn item_durability_bar(&mut self, bar_rect: Rect, stack: &ItemStack) {
        if stack.block.is_damageable() && stack.durability < stack.block.max_durability() {
            let ratio = stack.durability_ratio();
            let color = if ratio > 0.6 {
                pack_color(0.2, 0.9, 0.2)
            } else if ratio > 0.25 {
                pack_color(0.95, 0.75, 0.1)
            } else {
                pack_color(0.9, 0.2, 0.2)
            };
            self.progress_bar(bar_rect, ratio, pack_color(0.0, 0.0, 0.0), color);
        }
    }

    pub fn inventory_slot(
        &mut self,
        rect: impl Into<Rect>,
        item: Option<ItemStack>,
        is_hovered: bool,
        border_override: Option<u32>,
    ) {
        let r = rect.into();
        let border_color = if is_hovered {
            pack_color(1.0, 0.9, 0.2)
        } else if let Some(b) = border_override {
            b
        } else {
            pack_color(0.24, 0.24, 0.28)
        };

        self.fill(r, border_color);
        self.fill(r.inset(2), pack_color(0.12, 0.12, 0.14));

        if let Some(stack) = item {
            let icon_size = (r.w - 12).max(16);
            let ix = r.x + (r.w - icon_size) / 2;
            let iy = r.y + (r.h - icon_size) / 2;
            draw_item_icon(self.buffer, self.width, self.height, stack.block, ix, iy, icon_size);

            if stack.count > 1 {
                let mut count_buf = [0u8; 4];
                let count_str = format_u8(stack.count, &mut count_buf);
                let tx = r.right() - (count_str.len() as i32 * 8) - 4;
                let ty = r.bottom() - 11;
                self.text(count_str, tx, ty, 1, 1, pack_color(1.0, 1.0, 1.0));
            }

            self.item_durability_bar(Rect::new(r.x + 4, r.bottom() - 5, r.w - 8, 2), &stack);
        }
    }

    pub fn hotbar_slot(
        &mut self,
        rect: impl Into<Rect>,
        item: Option<&ItemStack>,
        border_color: u32,
        bg_color: u32,
    ) {
        let r = rect.into();
        self.framed_box(r, 2, border_color, bg_color);

        if let Some(stack) = item {
            draw_item_icon(
                self.buffer,
                self.width,
                self.height,
                stack.block,
                r.x + 3,
                r.y + 3,
                r.w - 6,
            );

            if stack.count > 1 {
                let mut count_buf = [0u8; 4];
                let label = format_u8(stack.count, &mut count_buf);
                self.text_white(label, r.x + r.w - 10, r.y + r.h - 12, 1, 1);
            }

            self.item_durability_bar(Rect::new(r.x + 3, r.y + r.h - 4, r.w - 6, 2), stack);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn debug_text_panel(
        &mut self,
        margin_x: i32,
        y: i32,
        lines: &[String],
        padding: i32,
        height_trim: i32,
        alpha: f32,
        right_aligned: bool,
    ) {
        if lines.is_empty() {
            return;
        }
        let scale = 1;
        let line_h = (font::GLYPH_H as i32) * scale + 3;
        let max_w = lines
            .iter()
            .map(|l| font::measure_text_light(l, scale, 1))
            .max()
            .unwrap_or(0);
        let panel_w = max_w + padding * 2;
        let panel_h = lines.len() as i32 * line_h + padding * 2 - height_trim;
        let x = if right_aligned {
            self.width as i32 - panel_w - margin_x
        } else {
            margin_x
        };

        self.fill_alpha(
            Rect::new(x, y, panel_w, panel_h),
            pack_color(0.0, 0.0, 0.0),
            alpha,
        );

        for (i, line) in lines.iter().enumerate() {
            let ty = y + padding + i as i32 * line_h;
            self.text_light(line, x + padding, ty, scale, 1);
        }
    }

    pub fn tooltip(&mut self, text: &str, mouse_pos: Option<(f32, f32)>, text_color: u32) {
        let Some((mx, my)) = mouse_pos else {
            return;
        };
        let box_w = (text.len() as i32 * 8 + 12).max(24);
        let box_h = 20;
        let tx = (mx as i32 + 15).min(self.width as i32 - box_w - 4).max(4);
        let ty = (my as i32 - 20).max(10);
        let box_rect = Rect::new(tx - 4, ty - 4, box_w, box_h);
        self.fill_alpha(box_rect, pack_color(0.05, 0.05, 0.08), 0.92);
        self.border(box_rect, 1, pack_color(0.28, 0.30, 0.38));
        self.text(text, tx, ty, 1, 1, text_color);
    }

    #[inline]
    pub fn text(&mut self, text: &str, x: i32, y: i32, scale: i32, spacing: i32, color: u32) {
        draw_text_colored_shadowed(
            self.buffer,
            self.width,
            self.height,
            text,
            x,
            y,
            scale,
            spacing,
            color,
        );
    }

    #[inline]
    pub fn text_white(&mut self, text: &str, x: i32, y: i32, scale: i32, spacing: i32) {
        draw_text_shadowed(self.buffer, self.width, self.height, text, x, y, scale, spacing);
    }

    #[inline]
    pub fn text_light(&mut self, text: &str, x: i32, y: i32, scale: i32, spacing: i32) {
        draw_text_light_shadowed(self.buffer, self.width, self.height, text, x, y, scale, spacing);
    }

    #[inline]
    pub fn text_light_colored(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        scale: i32,
        spacing: i32,
        color: u32,
    ) {
        draw_text_light_colored_shadowed(
            self.buffer,
            self.width,
            self.height,
            text,
            x,
            y,
            scale,
            spacing,
            color,
        );
    }

    #[inline]
    pub fn text_centered_x(
        &mut self,
        text: &str,
        center_x: i32,
        y: i32,
        scale: i32,
        spacing: i32,
        color: u32,
    ) {
        let tw = font::measure_text(text, scale, spacing);
        self.text(text, center_x - tw / 2, y, scale, spacing, color);
    }

    #[inline]
    pub fn text_right(
        &mut self,
        text: &str,
        right_x: i32,
        y: i32,
        scale: i32,
        spacing: i32,
        color: u32,
    ) {
        let tw = font::measure_text(text, scale, spacing);
        self.text(text, right_x - tw, y, scale, spacing, color);
    }

    #[inline]
    pub fn text_in_rect(
        &mut self,
        rect: impl Into<Rect>,
        text: &str,
        scale: i32,
        spacing: i32,
        color: u32,
    ) {
        let r = rect.into();
        let tw = font::measure_text(text, scale, spacing);
        let th = font::GLYPH_H as i32 * scale;
        let tx = r.x + (r.w - tw) / 2;
        let ty = r.y + (r.h - th) / 2;
        self.text(text, tx, ty, scale, spacing, color);
    }

    pub fn text_wrapped(
        &mut self,
        text: &str,
        x: i32,
        mut y: i32,
        max_w: i32,
        scale: i32,
        color: u32,
    ) -> i32 {
        let mut line = String::new();
        for word in text.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_string()
            } else {
                format!("{line} {word}")
            };
            if font::measure_text(&candidate, scale, 1) > max_w && !line.is_empty() {
                self.text(&line, x, y, scale, 1, color);
                y += 14 * scale.max(1);
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            self.text(&line, x, y, scale, 1, color);
            y += 14 * scale.max(1);
        }
        y
    }

    pub fn footer(&mut self, hint: &str, show_build_info: bool) {
        let hint_y = if show_build_info {
            self.height as i32 - 24
        } else {
            self.height as i32 - 18
        };
        self.text_centered_x(hint, self.cx(), hint_y, 1, 1, pack_color(0.65, 0.72, 0.85));

        if show_build_info {
            let build_info = crate::core::build_info::get_build_info();
            self.text_centered_x(
                build_info,
                self.cx(),
                self.height as i32 - 12,
                1,
                1,
                pack_color(0.48, 0.54, 0.65),
            );
        }
    }
}
