//! Procedural HUD status icons (hearts, air bubbles, hunger shanks, armor chestplates).
//! Zero platform `#[cfg]`.

use super::super::draw::{fill_rect, pack_color, set_px};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeartIconState {
    Full,
    Half,
    Empty,
}

pub fn draw_heart_icon(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    state: HeartIconState,
    flashing: bool,
) {
    let outline = if flashing {
        pack_color(1.0, 1.0, 1.0)
    } else {
        pack_color(0.20, 0.04, 0.04)
    };
    let dark_bg = pack_color(0.12, 0.06, 0.06);
    let red_hi = if flashing { pack_color(1.0, 1.0, 1.0) } else { pack_color(1.0, 0.40, 0.40) };
    let red_base = if flashing { pack_color(1.0, 0.85, 0.85) } else { pack_color(0.88, 0.10, 0.10) };
    let red_dark = if flashing { pack_color(0.90, 0.65, 0.65) } else { pack_color(0.55, 0.05, 0.05) };

    set_px(buffer, width, height, x + 1, y, outline);
    set_px(buffer, width, height, x + 2, y, outline);
    set_px(buffer, width, height, x + 5, y, outline);
    set_px(buffer, width, height, x + 6, y, outline);
    set_px(buffer, width, height, x, y + 1, outline);
    set_px(buffer, width, height, x + 3, y + 1, outline);
    set_px(buffer, width, height, x + 4, y + 1, outline);
    set_px(buffer, width, height, x + 7, y + 1, outline);
    for r in 2..=4 {
        set_px(buffer, width, height, x, y + r, outline);
        set_px(buffer, width, height, x + 7, y + r, outline);
    }
    set_px(buffer, width, height, x + 1, y + 5, outline);
    set_px(buffer, width, height, x + 6, y + 5, outline);
    set_px(buffer, width, height, x + 2, y + 6, outline);
    set_px(buffer, width, height, x + 5, y + 6, outline);
    set_px(buffer, width, height, x + 3, y + 7, outline);
    set_px(buffer, width, height, x + 4, y + 7, outline);

    match state {
        HeartIconState::Full => {
            set_px(buffer, width, height, x + 1, y + 1, red_hi);
            set_px(buffer, width, height, x + 2, y + 1, red_base);
            set_px(buffer, width, height, x + 5, y + 1, red_base);
            set_px(buffer, width, height, x + 6, y + 1, red_base);

            set_px(buffer, width, height, x + 1, y + 2, red_hi);
            fill_rect(buffer, width, height, x + 2, y + 2, 4, 1, red_base);
            set_px(buffer, width, height, x + 6, y + 2, red_dark);

            fill_rect(buffer, width, height, x + 1, y + 3, 5, 1, red_base);
            set_px(buffer, width, height, x + 6, y + 3, red_dark);

            fill_rect(buffer, width, height, x + 1, y + 4, 4, 1, red_base);
            set_px(buffer, width, height, x + 5, y + 4, red_dark);
            set_px(buffer, width, height, x + 6, y + 4, red_dark);

            fill_rect(buffer, width, height, x + 2, y + 5, 3, 1, red_base);
            set_px(buffer, width, height, x + 5, y + 5, red_dark);

            set_px(buffer, width, height, x + 3, y + 6, red_base);
            set_px(buffer, width, height, x + 4, y + 6, red_dark);
        }
        HeartIconState::Half => {
            set_px(buffer, width, height, x + 1, y + 1, red_hi);
            set_px(buffer, width, height, x + 2, y + 1, red_base);
            set_px(buffer, width, height, x + 5, y + 1, dark_bg);
            set_px(buffer, width, height, x + 6, y + 1, dark_bg);

            set_px(buffer, width, height, x + 1, y + 2, red_hi);
            fill_rect(buffer, width, height, x + 2, y + 2, 2, 1, red_base);
            fill_rect(buffer, width, height, x + 4, y + 2, 3, 1, dark_bg);

            fill_rect(buffer, width, height, x + 1, y + 3, 3, 1, red_base);
            fill_rect(buffer, width, height, x + 4, y + 3, 3, 1, dark_bg);

            fill_rect(buffer, width, height, x + 1, y + 4, 3, 1, red_base);
            fill_rect(buffer, width, height, x + 4, y + 4, 3, 1, dark_bg);

            fill_rect(buffer, width, height, x + 2, y + 5, 2, 1, red_base);
            fill_rect(buffer, width, height, x + 4, y + 5, 2, 1, dark_bg);

            set_px(buffer, width, height, x + 3, y + 6, red_base);
            set_px(buffer, width, height, x + 4, y + 6, dark_bg);
        }
        HeartIconState::Empty => {
            set_px(buffer, width, height, x + 1, y + 1, dark_bg);
            set_px(buffer, width, height, x + 2, y + 1, dark_bg);
            set_px(buffer, width, height, x + 5, y + 1, dark_bg);
            set_px(buffer, width, height, x + 6, y + 1, dark_bg);

            for r in 2..=4 {
                fill_rect(buffer, width, height, x + 1, y + r, 6, 1, dark_bg);
            }
            fill_rect(buffer, width, height, x + 2, y + 5, 4, 1, dark_bg);
            fill_rect(buffer, width, height, x + 3, y + 6, 2, 1, dark_bg);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BubbleState {
    Full,
    Pop,
    Empty,
}

pub fn draw_bubble(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    state: BubbleState,
) {
    match state {
        BubbleState::Full => {
            let outline = pack_color(0.05, 0.22, 0.42);
            let highlight = pack_color(0.95, 0.98, 1.0);
            let cyan = pack_color(0.32, 0.78, 0.98);
            let shadow = pack_color(0.12, 0.48, 0.75);

            fill_rect(buffer, width, height, x + 2, y, 4, 1, outline);
            fill_rect(buffer, width, height, x + 1, y + 1, 1, 1, outline);
            fill_rect(buffer, width, height, x + 6, y + 1, 1, 1, outline);
            fill_rect(buffer, width, height, x, y + 2, 1, 4, outline);
            fill_rect(buffer, width, height, x + 7, y + 2, 1, 4, outline);
            fill_rect(buffer, width, height, x + 1, y + 6, 1, 1, outline);
            fill_rect(buffer, width, height, x + 6, y + 6, 1, 1, outline);
            fill_rect(buffer, width, height, x + 2, y + 7, 4, 1, outline);

            fill_rect(buffer, width, height, x + 2, y + 1, 4, 1, cyan);
            fill_rect(buffer, width, height, x + 1, y + 2, 6, 4, cyan);
            fill_rect(buffer, width, height, x + 2, y + 6, 4, 1, cyan);

            set_px(buffer, width, height, x + 3, y + 1, highlight);
            set_px(buffer, width, height, x + 4, y + 1, highlight);
            set_px(buffer, width, height, x + 2, y + 2, highlight);
            set_px(buffer, width, height, x + 3, y + 2, highlight);
            set_px(buffer, width, height, x + 2, y + 3, highlight);

            set_px(buffer, width, height, x + 6, y + 5, shadow);
            set_px(buffer, width, height, x + 5, y + 6, shadow);
        }
        BubbleState::Pop => {
            let drop = pack_color(0.40, 0.85, 1.0);
            let glint = pack_color(0.95, 0.98, 1.0);
            set_px(buffer, width, height, x + 1, y + 1, glint);
            set_px(buffer, width, height, x + 6, y + 1, drop);
            set_px(buffer, width, height, x, y + 4, drop);
            set_px(buffer, width, height, x + 7, y + 4, glint);
            set_px(buffer, width, height, x + 2, y + 6, drop);
            set_px(buffer, width, height, x + 5, y + 6, glint);
            set_px(buffer, width, height, x + 3, y + 3, glint);
            set_px(buffer, width, height, x + 4, y + 4, drop);
        }
        BubbleState::Empty => {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HungerIconState {
    Full,
    Half,
    Empty,
}

pub fn draw_hunger_icon(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    state: HungerIconState,
) {
    let outline = pack_color(0.24, 0.10, 0.04);
    let bone_white = pack_color(0.95, 0.92, 0.85);
    let bone_shadow = pack_color(0.68, 0.62, 0.55);
    let meat_hi = pack_color(0.96, 0.68, 0.32);
    let meat_base = pack_color(0.78, 0.42, 0.14);
    let meat_sh = pack_color(0.48, 0.22, 0.06);
    let empty_bg = pack_color(0.12, 0.10, 0.10);
    let empty_border = pack_color(0.22, 0.18, 0.18);

    match state {
        HungerIconState::Full => {
            set_px(buffer, width, height, x + 6, y, bone_white);
            set_px(buffer, width, height, x + 7, y, bone_white);
            set_px(buffer, width, height, x + 5, y + 1, outline);
            set_px(buffer, width, height, x + 6, y + 1, bone_white);
            set_px(buffer, width, height, x + 7, y + 1, bone_shadow);

            fill_rect(buffer, width, height, x + 2, y + 2, 3, 1, outline);
            set_px(buffer, width, height, x + 5, y + 2, outline);
            set_px(buffer, width, height, x + 1, y + 3, outline);
            set_px(buffer, width, height, x, y + 4, outline);
            set_px(buffer, width, height, x, y + 5, outline);
            set_px(buffer, width, height, x, y + 6, outline);
            set_px(buffer, width, height, x + 1, y + 7, outline);
            fill_rect(buffer, width, height, x + 2, y + 7, 4, 1, outline);
            set_px(buffer, width, height, x + 6, y + 6, outline);
            set_px(buffer, width, height, x + 7, y + 5, outline);
            set_px(buffer, width, height, x + 6, y + 4, outline);
            set_px(buffer, width, height, x + 6, y + 3, outline);

            fill_rect(buffer, width, height, x + 2, y + 3, 4, 1, meat_base);
            fill_rect(buffer, width, height, x + 1, y + 4, 5, 1, meat_base);
            fill_rect(buffer, width, height, x + 1, y + 5, 6, 1, meat_base);
            fill_rect(buffer, width, height, x + 1, y + 6, 5, 1, meat_sh);

            set_px(buffer, width, height, x + 2, y + 3, meat_hi);
            set_px(buffer, width, height, x + 3, y + 3, meat_hi);
            set_px(buffer, width, height, x + 2, y + 4, meat_hi);
            set_px(buffer, width, height, x + 3, y + 4, meat_hi);

            set_px(buffer, width, height, x + 5, y + 5, meat_sh);
            set_px(buffer, width, height, x + 6, y + 5, meat_sh);
            set_px(buffer, width, height, x + 4, y + 6, meat_sh);
            set_px(buffer, width, height, x + 5, y + 6, meat_sh);
        }
        HungerIconState::Half => {
            set_px(buffer, width, height, x + 6, y, bone_white);
            set_px(buffer, width, height, x + 7, y, bone_white);
            set_px(buffer, width, height, x + 5, y + 1, outline);
            set_px(buffer, width, height, x + 6, y + 1, bone_white);
            set_px(buffer, width, height, x + 7, y + 1, bone_shadow);

            fill_rect(buffer, width, height, x + 1, y + 3, 3, 4, empty_bg);
            fill_rect(buffer, width, height, x, y + 4, 1, 3, empty_border);
            fill_rect(buffer, width, height, x + 1, y + 7, 3, 1, empty_border);
            fill_rect(buffer, width, height, x + 1, y + 2, 3, 1, empty_border);

            set_px(buffer, width, height, x + 4, y + 2, outline);
            set_px(buffer, width, height, x + 5, y + 2, outline);
            set_px(buffer, width, height, x + 4, y + 3, meat_hi);
            set_px(buffer, width, height, x + 5, y + 3, meat_base);
            set_px(buffer, width, height, x + 6, y + 3, outline);
            set_px(buffer, width, height, x + 4, y + 4, meat_hi);
            set_px(buffer, width, height, x + 5, y + 4, meat_base);
            set_px(buffer, width, height, x + 6, y + 4, outline);
            set_px(buffer, width, height, x + 4, y + 5, meat_base);
            set_px(buffer, width, height, x + 5, y + 5, meat_sh);
            set_px(buffer, width, height, x + 6, y + 5, outline);
            set_px(buffer, width, height, x + 4, y + 6, meat_sh);
            set_px(buffer, width, height, x + 5, y + 6, outline);
            fill_rect(buffer, width, height, x + 4, y + 7, 2, 1, outline);
        }
        HungerIconState::Empty => {
            set_px(buffer, width, height, x + 6, y, empty_border);
            set_px(buffer, width, height, x + 7, y, empty_border);
            set_px(buffer, width, height, x + 6, y + 1, empty_bg);
            set_px(buffer, width, height, x + 7, y + 1, empty_border);

            fill_rect(buffer, width, height, x + 2, y + 2, 4, 1, empty_border);
            set_px(buffer, width, height, x + 1, y + 3, empty_border);
            set_px(buffer, width, height, x, y + 4, empty_border);
            set_px(buffer, width, height, x, y + 5, empty_border);
            set_px(buffer, width, height, x, y + 6, empty_border);
            set_px(buffer, width, height, x + 1, y + 7, empty_border);
            fill_rect(buffer, width, height, x + 2, y + 7, 4, 1, empty_border);
            set_px(buffer, width, height, x + 6, y + 6, empty_border);
            set_px(buffer, width, height, x + 7, y + 5, empty_border);
            set_px(buffer, width, height, x + 6, y + 4, empty_border);
            set_px(buffer, width, height, x + 6, y + 3, empty_border);

            fill_rect(buffer, width, height, x + 2, y + 3, 4, 4, empty_bg);
            fill_rect(buffer, width, height, x + 1, y + 4, 1, 3, empty_bg);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ArmorIconState {
    Full,
    Half,
    Empty,
}

fn draw_armor_outline(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, outline: u32) {
    fill_rect(buffer, width, height, x + 2, y, 5, 1, outline);
    set_px(buffer, width, height, x + 1, y + 1, outline);
    set_px(buffer, width, height, x + 7, y + 1, outline);
    set_px(buffer, width, height, x, y + 2, outline);
    set_px(buffer, width, height, x + 8, y + 2, outline);
    fill_rect(buffer, width, height, x, y + 3, 1, 3, outline);
    fill_rect(buffer, width, height, x + 8, y + 3, 1, 3, outline);
    set_px(buffer, width, height, x + 1, y + 6, outline);
    set_px(buffer, width, height, x + 7, y + 6, outline);
    fill_rect(buffer, width, height, x + 2, y + 7, 5, 1, outline);
    set_px(buffer, width, height, x + 4, y + 8, outline);
    set_px(buffer, width, height, x + 4, y + 1, outline);
    set_px(buffer, width, height, x + 4, y + 2, outline);
}

pub fn draw_armor_icon(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    state: ArmorIconState,
) {
    let outline = pack_color(0.12, 0.12, 0.12);
    let empty_bg = pack_color(0.18, 0.18, 0.18);
    let iron_hi = pack_color(0.95, 0.95, 0.98);
    let iron_base = pack_color(0.80, 0.82, 0.85);
    let iron_sh = pack_color(0.55, 0.58, 0.62);

    draw_armor_outline(buffer, width, height, x, y, outline);

    match state {
        ArmorIconState::Full => {
            fill_rect(buffer, width, height, x + 2, y + 1, 2, 1, iron_hi);
            fill_rect(buffer, width, height, x + 5, y + 1, 2, 1, iron_base);

            fill_rect(buffer, width, height, x + 1, y + 2, 3, 1, iron_hi);
            fill_rect(buffer, width, height, x + 5, y + 2, 3, 1, iron_sh);

            set_px(buffer, width, height, x + 1, y + 3, iron_hi);
            fill_rect(buffer, width, height, x + 2, y + 3, 4, 1, iron_base);
            set_px(buffer, width, height, x + 6, y + 3, iron_sh);
            set_px(buffer, width, height, x + 7, y + 3, iron_sh);

            set_px(buffer, width, height, x + 1, y + 4, iron_hi);
            fill_rect(buffer, width, height, x + 2, y + 4, 4, 1, iron_base);
            set_px(buffer, width, height, x + 6, y + 4, iron_sh);
            set_px(buffer, width, height, x + 7, y + 4, iron_sh);

            set_px(buffer, width, height, x + 1, y + 5, iron_base);
            fill_rect(buffer, width, height, x + 2, y + 5, 4, 1, iron_base);
            set_px(buffer, width, height, x + 6, y + 5, iron_sh);
            set_px(buffer, width, height, x + 7, y + 5, iron_sh);

            set_px(buffer, width, height, x + 2, y + 6, iron_base);
            fill_rect(buffer, width, height, x + 3, y + 6, 2, 1, iron_base);
            set_px(buffer, width, height, x + 5, y + 6, iron_sh);
            set_px(buffer, width, height, x + 6, y + 6, iron_sh);

            set_px(buffer, width, height, x + 3, y + 7, iron_sh);
            set_px(buffer, width, height, x + 4, y + 7, iron_sh);
            set_px(buffer, width, height, x + 5, y + 7, iron_sh);
        }
        ArmorIconState::Half => {
            fill_rect(buffer, width, height, x + 2, y + 1, 2, 1, iron_hi);
            fill_rect(buffer, width, height, x + 5, y + 1, 2, 1, empty_bg);

            fill_rect(buffer, width, height, x + 1, y + 2, 3, 1, iron_hi);
            fill_rect(buffer, width, height, x + 5, y + 2, 3, 1, empty_bg);

            fill_rect(buffer, width, height, x + 1, y + 3, 3, 1, iron_base);
            fill_rect(buffer, width, height, x + 5, y + 3, 3, 1, empty_bg);

            fill_rect(buffer, width, height, x + 1, y + 4, 3, 1, iron_base);
            fill_rect(buffer, width, height, x + 5, y + 4, 3, 1, empty_bg);

            fill_rect(buffer, width, height, x + 1, y + 5, 3, 1, iron_base);
            fill_rect(buffer, width, height, x + 5, y + 5, 3, 1, empty_bg);

            fill_rect(buffer, width, height, x + 2, y + 6, 2, 1, iron_base);
            fill_rect(buffer, width, height, x + 5, y + 6, 2, 1, empty_bg);

            set_px(buffer, width, height, x + 3, y + 7, iron_sh);
            fill_rect(buffer, width, height, x + 4, y + 7, 2, 1, empty_bg);
        }
        ArmorIconState::Empty => {
            fill_rect(buffer, width, height, x + 2, y + 1, 2, 1, empty_bg);
            fill_rect(buffer, width, height, x + 5, y + 1, 2, 1, empty_bg);
            fill_rect(buffer, width, height, x + 1, y + 2, 3, 1, empty_bg);
            fill_rect(buffer, width, height, x + 5, y + 2, 3, 1, empty_bg);
            fill_rect(buffer, width, height, x + 1, y + 3, 7, 3, empty_bg);
            fill_rect(buffer, width, height, x + 2, y + 6, 5, 1, empty_bg);
            fill_rect(buffer, width, height, x + 3, y + 7, 3, 1, empty_bg);
        }
    }
}
