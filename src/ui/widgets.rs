//! Modular UI framework: geometry primitives (`Rect`), layout builders (`VStack`, `HStack`),
//! stateless widget builders (`Button`, `Panel`, `TextInput`, `ProgressBar`, `Scrollbar`, `Toast`),
//! and `UiCanvas` rendering context to eliminate repetitive pixel-buffer boilerplate.
//!
//! Designed with **zero heap allocations** in the render loop.

use crate::gameplay::inventory::ItemStack;
use crate::graphics::font;
use crate::ui::draw::{
    draw_text_colored_shadowed, draw_text_light_colored_shadowed, draw_text_light_shadowed,
    draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color,
};
use crate::ui::inventory_ui::draw_item_icon;
use crate::world::block::BlockId;

/// Formats a `u8` (`0..=255`) into a stack-allocated ASCII buffer without heap allocation.
#[inline]
fn format_u8(mut n: u8, buf: &mut [u8; 4]) -> &str {
    if n == 0 {
        buf[0] = b'0';
        return std::str::from_utf8(&buf[..1]).unwrap_or("0");
    }
    let mut i = 4;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10);
        n /= 10;
    }
    std::str::from_utf8(&buf[i..]).unwrap_or("")
}

/// Axis-aligned 2D rectangle in screen pixel coordinates `(x, y, w, h)`.
/// Seamlessly converts to and from legacy `(i32, i32, i32, i32)` tuples.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    #[inline]
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub const fn from_tuple(t: (i32, i32, i32, i32)) -> Self {
        Self {
            x: t.0,
            y: t.1,
            w: t.2,
            h: t.3,
        }
    }

    #[inline]
    pub const fn to_tuple(self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.w, self.h)
    }

    /// Creates a rectangle centered inside `bounds` (or screen dimensions).
    #[inline]
    pub const fn centered_in(screen_w: usize, screen_h: usize, w: i32, h: i32) -> Self {
        Self {
            x: (screen_w as i32 - w) / 2,
            y: (screen_h as i32 - h) / 2,
            w,
            h,
        }
    }

    /// Creates a rectangle horizontally centered at `y`.
    #[inline]
    pub const fn centered_x(screen_w: usize, y: i32, w: i32, h: i32) -> Self {
        Self {
            x: (screen_w as i32 - w) / 2,
            y,
            w,
            h,
        }
    }

    #[inline]
    pub const fn right(self) -> i32 {
        self.x + self.w
    }

    #[inline]
    pub const fn bottom(self) -> i32 {
        self.y + self.h
    }

    #[inline]
    pub const fn center_x(self) -> i32 {
        self.x + self.w / 2
    }

    #[inline]
    pub const fn center_y(self) -> i32 {
        self.y + self.h / 2
    }

    /// Shrinks the rectangle uniformly on all sides by `pad` pixels (negative `pad` expands it).
    #[inline]
    pub const fn inset(self, pad: i32) -> Self {
        Self {
            x: self.x + pad,
            y: self.y + pad,
            w: self.w - pad * 2,
            h: self.h - pad * 2,
        }
    }

    /// Shrinks the rectangle horizontally by `pad_x` and vertically by `pad_y`.
    #[inline]
    pub const fn inset_xy(self, pad_x: i32, pad_y: i32) -> Self {
        Self {
            x: self.x + pad_x,
            y: self.y + pad_y,
            w: self.w - pad_x * 2,
            h: self.h - pad_y * 2,
        }
    }

    /// Translates the rectangle by `(dx, dy)`.
    #[inline]
    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            w: self.w,
            h: self.h,
        }
    }

    /// Checks whether an integer pixel point `(px, py)` lies inside this rectangle.
    #[inline]
    pub const fn contains_point(self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }

    /// Checks whether an optional floating-point mouse position lies inside this rectangle.
    #[inline]
    pub fn contains_mouse(self, mouse_pos: Option<(f32, f32)>) -> bool {
        if let Some((mx, my)) = mouse_pos {
            self.contains_point(mx as i32, my as i32)
        } else {
            false
        }
    }

    /// Splits `N` equal-width columns horizontally with `gap` pixels between them.
    pub fn split_cols<const N: usize>(self, gap: i32) -> [Rect; N] {
        let count = N as i32;
        if count <= 0 {
            return [Self::default(); N];
        }
        let col_w = (self.w - gap * (count - 1)) / count;
        std::array::from_fn(|i| {
            let x = self.x + (i as i32) * (col_w + gap);
            Rect::new(x, self.y, col_w, self.h)
        })
    }
}

impl From<(i32, i32, i32, i32)> for Rect {
    #[inline]
    fn from(t: (i32, i32, i32, i32)) -> Self {
        Self::from_tuple(t)
    }
}

impl From<Rect> for (i32, i32, i32, i32) {
    #[inline]
    fn from(r: Rect) -> Self {
        r.to_tuple()
    }
}

/// Stack-allocated vertical layout helper for placing rows top-to-bottom without manual coordinate math.
#[derive(Clone, Copy, Debug)]
pub struct VStack {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub gap: i32,
}

impl VStack {
    #[inline]
    pub const fn new(x: i32, start_y: i32, w: i32, gap: i32) -> Self {
        Self {
            x,
            y: start_y,
            w,
            gap,
        }
    }

    #[inline]
    pub const fn centered(screen_w: usize, start_y: i32, w: i32, gap: i32) -> Self {
        Self {
            x: (screen_w as i32 - w) / 2,
            y: start_y,
            w,
            gap,
        }
    }

    /// Allocates the next item of height `h` and advances the cursor.
    #[inline]
    pub fn next(&mut self, h: i32) -> Rect {
        let r = Rect::new(self.x, self.y, self.w, h);
        self.y += h + self.gap;
        r
    }

    /// Adds extra vertical spacing before the next item.
    #[inline]
    pub fn skip(&mut self, extra_y: i32) {
        self.y += extra_y;
    }

    /// Generates a fixed-size array of `N` uniformly sized vertical slots.
    pub fn array<const N: usize>(mut self, item_h: i32) -> [(i32, i32, i32, i32); N] {
        std::array::from_fn(|_| self.next(item_h).to_tuple())
    }
}

/// Stack-allocated horizontal layout helper for placing items left-to-right.
#[derive(Clone, Copy, Debug)]
pub struct HStack {
    pub x: i32,
    pub y: i32,
    pub h: i32,
    pub gap: i32,
}

impl HStack {
    #[inline]
    pub const fn new(start_x: i32, y: i32, h: i32, gap: i32) -> Self {
        Self {
            x: start_x,
            y,
            h,
            gap,
        }
    }

    /// Creates a horizontal stack of `count` items of `item_w` centered horizontally on screen.
    #[inline]
    pub const fn centered_row(screen_w: usize, y: i32, item_w: i32, h: i32, count: usize, gap: i32) -> Self {
        let total_w = (count as i32) * item_w + (count.saturating_sub(1) as i32) * gap;
        Self {
            x: (screen_w as i32 - total_w) / 2,
            y,
            h,
            gap,
        }
    }

    /// Allocates the next item of width `w` and advances the cursor.
    #[inline]
    pub fn next(&mut self, w: i32) -> Rect {
        let r = Rect::new(self.x, self.y, w, self.h);
        self.x += w + self.gap;
        r
    }

    /// Generates a fixed-size array of `N` uniformly sized horizontal slots.
    pub fn array<const N: usize>(mut self, item_w: i32) -> [(i32, i32, i32, i32); N] {
        std::array::from_fn(|_| self.next(item_w).to_tuple())
    }
}

/// 2D uniform grid layout for inventory slots, crafting matrices, and card tables.
/// Eliminates duplicate nested `for row / for col` coordinate math across hit-testing and rendering.
#[derive(Clone, Copy, Debug)]
pub struct Grid {
    pub x: i32,
    pub y: i32,
    pub cols: usize,
    pub rows: usize,
    pub cell_w: i32,
    pub cell_h: i32,
    pub step_x: i32,
    pub step_y: i32,
}

impl Grid {
    #[inline]
    pub const fn new(
        x: i32,
        y: i32,
        cols: usize,
        rows: usize,
        cell_w: i32,
        cell_h: i32,
        step_x: i32,
        step_y: i32,
    ) -> Self {
        Self {
            x,
            y,
            cols,
            rows,
            cell_w,
            cell_h,
            step_x,
            step_y,
        }
    }

    /// Creates a square-cell grid where `step = cell_size + gap`.
    #[inline]
    pub const fn square(x: i32, y: i32, cols: usize, rows: usize, cell_size: i32, gap: i32) -> Self {
        let step = cell_size + gap;
        Self::new(x, y, cols, rows, cell_size, cell_size, step, step)
    }

    /// Total width in pixels from the left edge of the first column to the right edge of the last column.
    #[inline]
    pub const fn total_width(&self) -> i32 {
        if self.cols == 0 {
            0
        } else {
            (self.cols as i32 - 1) * self.step_x + self.cell_w
        }
    }

    /// Total height in pixels from the top edge of the first row to the bottom edge of the last row.
    #[inline]
    pub const fn total_height(&self) -> i32 {
        if self.rows == 0 {
            0
        } else {
            (self.rows as i32 - 1) * self.step_y + self.cell_h
        }
    }

    /// Returns the bounding `Rect` for `(row, col)`.
    #[inline]
    pub const fn cell(&self, row: usize, col: usize) -> Rect {
        Rect::new(
            self.x + (col as i32) * self.step_x,
            self.y + (row as i32) * self.step_y,
            self.cell_w,
            self.cell_h,
        )
    }

    /// Returns the bounding `Rect` for linear slot index `idx = row * cols + col`.
    #[inline]
    pub const fn slot(&self, idx: usize) -> Rect {
        if self.cols == 0 {
            return Rect::new(self.x, self.y, self.cell_w, self.cell_h);
        }
        self.cell(idx / self.cols, idx % self.cols)
    }

    /// Hit-tests `(px, py)` against all cells in the grid and returns the linear index `0..(rows * cols)` if hit.
    pub fn hit_test(&self, px: i32, py: i32) -> Option<usize> {
        if px < self.x || py < self.y || self.cols == 0 || self.rows == 0 || self.step_x <= 0 || self.step_y <= 0 {
            return None;
        }
        let rel_x = px - self.x;
        let rel_y = py - self.y;
        let col = (rel_x / self.step_x) as usize;
        let row = (rel_y / self.step_y) as usize;
        if col < self.cols
            && row < self.rows
            && (rel_x % self.step_x) < self.cell_w
            && (rel_y % self.step_y) < self.cell_h
        {
            Some(row * self.cols + col)
        } else {
            None
        }
    }
}

/// Visual theme preset for buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Standard slate-blue button used across menus.
    Standard,
    /// Green confirm / play / save action button.
    Primary,
    /// Red cancel / delete / danger action button.
    Danger,
    /// Blue accent button (e.g. refresh / secondary action).
    Accent,
    /// Tab header button with active/inactive highlight.
    Tab { active: bool },
    /// Custom flat background and border colors.
    Flat {
        bg: u32,
        hover_bg: u32,
        border: u32,
        hover_border: u32,
    },
}

/// Stateless button widget configuration.
#[derive(Clone, Copy)]
pub struct Button<'a> {
    pub label: &'a str,
    pub variant: ButtonVariant,
    pub hovered: bool,
    pub focused: bool,
    pub scale: i32,
    pub text_color: Option<u32>,
    pub outer_border: bool,
    pub beveled: bool,
}

impl<'a> Button<'a> {
    #[inline]
    pub const fn new(label: &'a str) -> Self {
        Self {
            label,
            variant: ButtonVariant::Standard,
            hovered: false,
            focused: false,
            scale: 2,
            text_color: None,
            outer_border: false,
            beveled: true,
        }
    }

    #[inline]
    pub const fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    #[inline]
    pub const fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    #[inline]
    pub const fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    #[inline]
    pub const fn accent(mut self) -> Self {
        self.variant = ButtonVariant::Accent;
        self
    }

    #[inline]
    pub const fn tab(mut self, active: bool) -> Self {
        self.variant = ButtonVariant::Tab { active };
        self
    }

    #[inline]
    pub const fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[inline]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[inline]
    pub const fn scale(mut self, scale: i32) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub const fn text_color(mut self, color: u32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Uses a 2px outer border frame (`x-2, y-2, w+4, h+4`) instead of inner 3D bevels.
    #[inline]
    pub const fn outer_frame(mut self) -> Self {
        self.outer_border = true;
        self.beveled = false;
        self
    }

    /// Uses a flat inner border instead of 3D bevels.
    #[inline]
    pub const fn flat_border(mut self) -> Self {
        self.outer_border = false;
        self.beveled = false;
        self
    }
}

/// Stateless text input box widget configuration.
#[derive(Clone, Copy)]
pub struct TextInput<'a> {
    pub text: &'a str,
    pub placeholder: &'a str,
    pub label: Option<&'a str>,
    pub focused: bool,
    pub hovered: bool,
    pub selected: bool,
    pub cursor_visible: bool,
    pub scale: i32,
    pub outer_border: bool,
}

impl<'a> TextInput<'a> {
    #[inline]
    pub const fn new(text: &'a str) -> Self {
        Self {
            text,
            placeholder: "",
            label: None,
            focused: false,
            hovered: false,
            selected: false,
            cursor_visible: false,
            scale: 2,
            outer_border: false,
        }
    }

    #[inline]
    pub const fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    #[inline]
    pub const fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    #[inline]
    pub const fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    #[inline]
    pub const fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    #[inline]
    pub const fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    #[inline]
    pub const fn cursor(mut self, visible: bool) -> Self {
        self.cursor_visible = visible;
        self
    }

    #[inline]
    pub const fn scale(mut self, scale: i32) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub const fn outer_frame(mut self) -> Self {
        self.outer_border = true;
        self
    }
}

/// Toast notification configuration (used by Quests, Achievements, Multiplayer, Music).
#[derive(Clone, Copy)]
pub struct Toast<'a> {
    pub heading: &'a str,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub accent_color: u32,
    pub icon: Option<BlockId>,
    pub y: i32,
}

/// Zero-allocation rendering wrapper around the raw pixel framebuffer `(buffer, width, height)`.
/// Eliminates repetitive parameter passing and provides declarative widget drawing methods.
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

    // --- Primitive Rectangle Operations ---

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

    /// Fills the entire screen with a solid backdrop color and optional translucent overlay.
    pub fn backdrop(&mut self, solid_color: Option<u32>, overlay_color: u32, alpha: f32) {
        let full = self.screen_rect();
        if let Some(solid) = solid_color {
            self.fill(full, solid);
        }
        if alpha > 0.0 {
            self.fill_alpha(full, overlay_color, alpha);
        }
    }

    /// Draws an inner border of `thickness` pixels inside `rect`.
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

    /// Draws a 3D beveled border inside `rect` (top/left highlight, bottom/right shadow).
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

    /// Draws a rectangle with an outer border expanding `thickness` pixels outside `rect`,
    /// followed by filling `rect` with `bg_color`.
    pub fn framed_box(&mut self, rect: impl Into<Rect>, outer_thickness: i32, border_color: u32, bg_color: u32) {
        let r = rect.into();
        self.fill(r.inset(-outer_thickness), border_color);
        self.fill(r, bg_color);
    }

    /// Draws a rectangle with an inner border of `thickness` pixels.
    pub fn bordered_box(&mut self, rect: impl Into<Rect>, thickness: i32, bg_color: u32, border_color: u32) {
        let r = rect.into();
        self.fill(r, bg_color);
        self.border(r, thickness, border_color);
    }

    /// Draws a modal panel with semi-transparent background and 3D beveled 2px border.
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

    /// Draws an inset/sunken panel with dark top/left shadow and light bottom/right rim (used in Inventory & Containers).
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

    /// Draws a compact flat button with a 1px light text label (used in WorldMap, Waypoints, and toolbars).
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
        draw_text_light_colored_shadowed(self.buffer, self.width, self.height, label, tx, ty, 1, 1, text_color);
    }

    /// Renders an interactive inventory/container slot with item icon, stack count, and durability bar.
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

            if stack.block.is_damageable() && stack.durability < stack.block.max_durability() {
                let ratio = stack.durability_ratio();
                let color = if ratio > 0.6 {
                    pack_color(0.2, 0.9, 0.2)
                } else if ratio > 0.25 {
                    pack_color(0.95, 0.75, 0.1)
                } else {
                    pack_color(0.9, 0.2, 0.2)
                };
                self.progress_bar(
                    Rect::new(r.x + 4, r.bottom() - 5, r.w - 8, 2),
                    ratio,
                    pack_color(0.0, 0.0, 0.0),
                    color,
                );
            }
        }
    }

    /// Renders a HUD hotbar/offhand slot with outer frame, item icon, stack count, and durability bar.
    /// Zero heap allocations in the per-frame render loop.
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

            if stack.block.is_damageable() && stack.durability < stack.block.max_durability() {
                let ratio = stack.durability_ratio();
                let color = if ratio > 0.6 {
                    pack_color(0.2, 0.9, 0.2)
                } else if ratio > 0.25 {
                    pack_color(0.95, 0.75, 0.1)
                } else {
                    pack_color(0.9, 0.2, 0.2)
                };
                self.progress_bar(
                    Rect::new(r.x + 3, r.y + r.h - 4, r.w - 6, 2),
                    ratio,
                    pack_color(0.0, 0.0, 0.0),
                    color,
                );
            }
        }
    }

    /// Renders a translucent diagnostic text panel (used in F3 Debug Overlay and simple Debug Panel).
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

    /// Renders a floating hover tooltip near the mouse cursor, clamped to screen bounds.
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

    // --- Typography Helpers ---

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

    /// Draws text horizontally centered around `center_x` at `y`.
    #[inline]
    pub fn text_centered_x(&mut self, text: &str, center_x: i32, y: i32, scale: i32, spacing: i32, color: u32) {
        let tw = font::measure_text(text, scale, spacing);
        self.text(text, center_x - tw / 2, y, scale, spacing, color);
    }

    /// Draws text right-aligned so it ends at `right_x` at `y`.
    #[inline]
    pub fn text_right(&mut self, text: &str, right_x: i32, y: i32, scale: i32, spacing: i32, color: u32) {
        let tw = font::measure_text(text, scale, spacing);
        self.text(text, right_x - tw, y, scale, spacing, color);
    }

    /// Draws text centered both horizontally and vertically inside `rect`.
    #[inline]
    pub fn text_in_rect(&mut self, rect: impl Into<Rect>, text: &str, scale: i32, spacing: i32, color: u32) {
        let r = rect.into();
        let tw = font::measure_text(text, scale, spacing);
        let th = font::GLYPH_H as i32 * scale;
        let tx = r.x + (r.w - tw) / 2;
        let ty = r.y + (r.h - th) / 2;
        self.text(text, tx, ty, scale, spacing, color);
    }

    /// Draws word-wrapped text within `max_w` and returns the advancing `y` coordinate.
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

    /// Draws standard screen footer hint and optional build telemetry line at the bottom of the screen.
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

    // --- High-Level Composite Widgets ---

    /// Renders a configurable `Button` inside `rect`.
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

    /// Renders a single-line text input field with optional top label, placeholder, selection highlight, and blinking cursor.
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

    /// Renders a horizontal progress bar inside `rect`.
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

    /// Renders a vertical scrollbar track and proportional thumb inside `track_rect`.
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

    /// Renders an icon slot box with border, block/item icon, and optional stack count label.
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

    /// Renders a top-right notification toast card (used for Achievements and Quests).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_geometry_and_layout_stacks() {
        let r = Rect::centered_in(800, 600, 400, 300);
        assert_eq!(r.to_tuple(), (200, 150, 400, 300));
        assert!(r.contains_point(250, 200));
        assert!(!r.contains_point(100, 200));
        assert!(r.contains_mouse(Some((399.0, 299.0))));

        let inset = r.inset(10);
        assert_eq!(inset.to_tuple(), (210, 160, 380, 280));

        let cols: [Rect; 4] = Rect::new(10, 20, 430, 30).split_cols(10);
        assert_eq!(cols[0].to_tuple(), (10, 20, 100, 30));
        assert_eq!(cols[1].to_tuple(), (120, 20, 100, 30));
        assert_eq!(cols[3].to_tuple(), (340, 20, 100, 30));

        let v_arr: [(i32, i32, i32, i32); 3] = VStack::centered(800, 100, 200, 10).array(40);
        assert_eq!(v_arr[0], (300, 100, 200, 40));
        assert_eq!(v_arr[1], (300, 150, 200, 40));
        assert_eq!(v_arr[2], (300, 200, 200, 40));

        let grid = Grid::square(20, 30, 9, 3, 38, 4);
        assert_eq!(grid.slot(0).to_tuple(), (20, 30, 38, 38));
        assert_eq!(grid.slot(10).to_tuple(), (62, 72, 38, 38));
        assert_eq!(grid.hit_test(25, 35), Some(0));
        assert_eq!(grid.hit_test(59, 35), None); // inside 4px gap
        assert_eq!(grid.hit_test(65, 75), Some(10));
        assert_eq!(grid.hit_test(500, 500), None);
    }

    #[test]
    fn test_ui_canvas_widgets_render() {
        let (w, h) = (320, 240);
        let mut buf = vec![0u32; w * h];
        let mut ui = UiCanvas::new(&mut buf, w, h);

        ui.backdrop(Some(0xFF101010), 0xFF000000, 0.5);
        ui.button(
            Rect::new(20, 20, 120, 32),
            Button::new("Play").primary().hovered(true),
        );
        ui.text_input(
            Rect::new(20, 60, 160, 28),
            TextInput::new("World 1").focused(true).cursor(true),
        );
        ui.progress_bar(Rect::new(20, 100, 100, 6), 0.5, 0xFF222222, 0xFF00FF00);
        ui.scrollbar(Rect::new(200, 20, 6, 100), 4, 12, 2, 0xFF222222, 0xFF88AAFF);

        assert!(buf.iter().any(|&px| px != 0xFF101010));
    }
}
