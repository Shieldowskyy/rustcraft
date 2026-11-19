//! 2D layout geometry primitives (`Rect`, `VStack`, `HStack`, `Grid`) and stack `u8` formatter.
//! Zero heap allocations and zero platform `#[cfg]`.

/// Formats a `u8` (`0..=255`) into a stack-allocated ASCII buffer without heap allocation.
#[inline]
pub(super) fn format_u8(mut n: u8, buf: &mut [u8; 4]) -> &str {
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

    #[inline]
    pub const fn centered_in(screen_w: usize, screen_h: usize, w: i32, h: i32) -> Self {
        Self {
            x: (screen_w as i32 - w) / 2,
            y: (screen_h as i32 - h) / 2,
            w,
            h,
        }
    }

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

    #[inline]
    pub const fn inset(self, pad: i32) -> Self {
        Self {
            x: self.x + pad,
            y: self.y + pad,
            w: self.w - pad * 2,
            h: self.h - pad * 2,
        }
    }

    #[inline]
    pub const fn inset_xy(self, pad_x: i32, pad_y: i32) -> Self {
        Self {
            x: self.x + pad_x,
            y: self.y + pad_y,
            w: self.w - pad_x * 2,
            h: self.h - pad_y * 2,
        }
    }

    #[inline]
    pub const fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            w: self.w,
            h: self.h,
        }
    }

    #[inline]
    pub const fn contains_point(self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }

    #[inline]
    pub fn contains_mouse(self, mouse_pos: Option<(f32, f32)>) -> bool {
        if let Some((mx, my)) = mouse_pos {
            self.contains_point(mx as i32, my as i32)
        } else {
            false
        }
    }

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

/// Stack-allocated vertical layout helper for placing rows top-to-bottom.
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

    #[inline]
    pub fn next(&mut self, h: i32) -> Rect {
        let r = Rect::new(self.x, self.y, self.w, h);
        self.y += h + self.gap;
        r
    }

    #[inline]
    pub fn skip(&mut self, extra_y: i32) {
        self.y += extra_y;
    }

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

    #[inline]
    pub const fn centered_row(
        screen_w: usize,
        y: i32,
        item_w: i32,
        h: i32,
        count: usize,
        gap: i32,
    ) -> Self {
        let total_w = (count as i32) * item_w + (count.saturating_sub(1) as i32) * gap;
        Self {
            x: (screen_w as i32 - total_w) / 2,
            y,
            h,
            gap,
        }
    }

    #[inline]
    pub fn next(&mut self, w: i32) -> Rect {
        let r = Rect::new(self.x, self.y, w, self.h);
        self.x += w + self.gap;
        r
    }

    pub fn array<const N: usize>(mut self, item_w: i32) -> [(i32, i32, i32, i32); N] {
        std::array::from_fn(|_| self.next(item_w).to_tuple())
    }
}

/// 2D uniform grid layout for inventory slots, crafting matrices, and card tables.
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
    #[allow(clippy::too_many_arguments)]
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

    #[inline]
    pub const fn square(
        x: i32,
        y: i32,
        cols: usize,
        rows: usize,
        cell_size: i32,
        gap: i32,
    ) -> Self {
        let step = cell_size + gap;
        Self::new(x, y, cols, rows, cell_size, cell_size, step, step)
    }

    #[inline]
    pub const fn total_width(&self) -> i32 {
        if self.cols == 0 {
            0
        } else {
            (self.cols as i32 - 1) * self.step_x + self.cell_w
        }
    }

    #[inline]
    pub const fn total_height(&self) -> i32 {
        if self.rows == 0 {
            0
        } else {
            (self.rows as i32 - 1) * self.step_y + self.cell_h
        }
    }

    #[inline]
    pub const fn cell(&self, row: usize, col: usize) -> Rect {
        Rect::new(
            self.x + (col as i32) * self.step_x,
            self.y + (row as i32) * self.step_y,
            self.cell_w,
            self.cell_h,
        )
    }

    #[inline]
    pub const fn slot(&self, idx: usize) -> Rect {
        if self.cols == 0 {
            return Rect::new(self.x, self.y, self.cell_w, self.cell_h);
        }
        self.cell(idx / self.cols, idx % self.cols)
    }

    pub fn hit_test(&self, px: i32, py: i32) -> Option<usize> {
        if px < self.x
            || py < self.y
            || self.cols == 0
            || self.rows == 0
            || self.step_x <= 0
            || self.step_y <= 0
        {
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
