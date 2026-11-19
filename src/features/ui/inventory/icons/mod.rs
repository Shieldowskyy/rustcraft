//! Proceduralne rysowanie ikon 16x16 przedmiotów i bloków w slotach ekwipunku oraz na hotbarze.

pub(super) mod blocks_misc;
pub(super) mod food_mobs;
pub(super) mod gear;

use crate::features::blocks::BlockId;
use crate::features::ui::draw::fill_rect;

pub(super) struct IconGrid<'a> {
    pub buffer: &'a mut [u32],
    pub buf_w: usize,
    pub buf_h: usize,
    pub x: i32,
    pub y: i32,
    pub size: i32,
    pub scale: i32,
    pub ox: i32,
    pub oy: i32,
}

impl<'a> IconGrid<'a> {
    #[inline]
    pub fn px(&mut self, lx: i32, ly: i32, col: u32) {
        if (0..16).contains(&lx) && (0..16).contains(&ly) {
            fill_rect(
                self.buffer,
                self.buf_w,
                self.buf_h,
                self.ox + lx * self.scale,
                self.oy + ly * self.scale,
                self.scale,
                self.scale,
                col,
            );
        }
    }

    #[inline]
    pub fn pts(&mut self, col: u32, points: &[(i32, i32)]) {
        for &(lx, ly) in points {
            self.px(lx, ly, col);
        }
    }
}

pub fn draw_item_icon(buffer: &mut [u32], buf_w: usize, buf_h: usize, block: BlockId, x: i32, y: i32, size: i32) {
    let scale = (size / 16).max(1);
    let ox = x + (size - 16 * scale) / 2;
    let oy = y + (size - 16 * scale) / 2;
    let mut g = IconGrid {
        buffer,
        buf_w,
        buf_h,
        x,
        y,
        size,
        scale,
        ox,
        oy,
    };

    if gear::draw_gear_icon(&mut g, block) {
        return;
    }
    if food_mobs::draw_food_mob_icon(&mut g, block) {
        return;
    }
    blocks_misc::draw_block_or_misc_icon(&mut g, block);
}
