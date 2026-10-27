use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::Player;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::block::Face;
use crate::world::tile_entity::{ChestData, FurnaceData, SMELT_DURATION};

pub const SLOT_SIZE: i32 = 36;
pub const SLOT_GAP: i32 = 4;
pub const SLOT_STEP: i32 = SLOT_SIZE + SLOT_GAP;

#[inline(always)]
fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChestSlotTarget {
    Chest(usize),
    Main(usize),
    Hotbar(usize),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FurnaceSlotTarget {
    Input,
    Fuel,
    Output,
    Main(usize),
    Hotbar(usize),
}

pub struct ChestLayout {
    pub start_x: i32,
    pub start_y: i32,
}

impl ChestLayout {
    pub fn new(width: usize, height: usize) -> Self {
        let start_x = (width as i32 - 420) / 2;
        let start_y = (height as i32 - 380) / 2;
        Self { start_x, start_y }
    }

    pub fn find_slot_at(&self, mx: i32, my: i32) -> Option<ChestSlotTarget> {
        // 1. Chest 9x3 slots
        let chest_top_y = self.start_y + 35;
        let chest_start_x = self.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
        for r in 0..3 {
            for c in 0..9 {
                let sx = chest_start_x + c * SLOT_STEP;
                let sy = chest_top_y + r * SLOT_STEP;
                if mx >= sx && mx < sx + SLOT_SIZE && my >= sy && my < sy + SLOT_SIZE {
                    return Some(ChestSlotTarget::Chest((r * 9 + c) as usize));
                }
            }
        }

        // 2. Main inventory 9x3 slots
        let main_top_y = self.start_y + 185;
        for r in 0..3 {
            for c in 0..9 {
                let sx = chest_start_x + c * SLOT_STEP;
                let sy = main_top_y + r * SLOT_STEP;
                if mx >= sx && mx < sx + SLOT_SIZE && my >= sy && my < sy + SLOT_SIZE {
                    return Some(ChestSlotTarget::Main((r * 9 + c) as usize));
                }
            }
        }

        // 3. Hotbar 10 slots
        let hotbar_y = self.start_y + 325;
        let hotbar_start_x = self.start_x + (420 - 10 * SLOT_STEP + SLOT_GAP) / 2;
        for i in 0..10 {
            let sx = hotbar_start_x + i * SLOT_STEP;
            if mx >= sx && mx < sx + SLOT_SIZE && my >= hotbar_y && my < hotbar_y + SLOT_SIZE {
                return Some(ChestSlotTarget::Hotbar(i as usize));
            }
        }

        None
    }
}

pub struct FurnaceLayout {
    pub start_x: i32,
    pub start_y: i32,
}

impl FurnaceLayout {
    pub fn new(width: usize, height: usize) -> Self {
        let start_x = (width as i32 - 420) / 2;
        let start_y = (height as i32 - 380) / 2;
        Self { start_x, start_y }
    }

    pub fn find_slot_at(&self, mx: i32, my: i32) -> Option<FurnaceSlotTarget> {
        let center_x = self.start_x + 210;

        // Input slot
        let in_x = center_x - 70;
        let in_y = self.start_y + 38;
        if mx >= in_x && mx < in_x + SLOT_SIZE && my >= in_y && my < in_y + SLOT_SIZE {
            return Some(FurnaceSlotTarget::Input);
        }

        // Fuel slot
        let fuel_x = center_x - 70;
        let fuel_y = self.start_y + 112;
        if mx >= fuel_x && mx < fuel_x + SLOT_SIZE && my >= fuel_y && my < fuel_y + SLOT_SIZE {
            return Some(FurnaceSlotTarget::Fuel);
        }

        // Output slot (44x44)
        let out_x = center_x + 35;
        let out_y = self.start_y + 70;
        if mx >= out_x && mx < out_x + 44 && my >= out_y && my < out_y + 44 {
            return Some(FurnaceSlotTarget::Output);
        }

        // Main inventory 9x3 slots
        let chest_start_x = self.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
        let main_top_y = self.start_y + 185;
        for r in 0..3 {
            for c in 0..9 {
                let sx = chest_start_x + c * SLOT_STEP;
                let sy = main_top_y + r * SLOT_STEP;
                if mx >= sx && mx < sx + SLOT_SIZE && my >= sy && my < sy + SLOT_SIZE {
                    return Some(FurnaceSlotTarget::Main((r * 9 + c) as usize));
                }
            }
        }

        // Hotbar 10 slots
        let hotbar_y = self.start_y + 325;
        let hotbar_start_x = self.start_x + (420 - 10 * SLOT_STEP + SLOT_GAP) / 2;
        for i in 0..10 {
            let sx = hotbar_start_x + i * SLOT_STEP;
            if mx >= sx && mx < sx + SLOT_SIZE && my >= hotbar_y && my < hotbar_y + SLOT_SIZE {
                return Some(FurnaceSlotTarget::Hotbar(i as usize));
            }
        }

        None
    }
}

pub fn draw_slot(
    framebuffer: &mut [u32],
    fb_w: usize,
    fb_h: usize,
    x: i32,
    y: i32,
    size: i32,
    stack: Option<ItemStack>,
    is_hovered: bool,
) {
    let slot_bg = if is_hovered { pack_rgb(70, 70, 80) } else { pack_rgb(40, 40, 44) };
    let border_dark = pack_rgb(20, 20, 22);
    let border_light = pack_rgb(90, 90, 100);

    // Slot background & bevel
    fill_rect(framebuffer, fb_w, fb_h, x, y, size, size, slot_bg);
    fill_rect(framebuffer, fb_w, fb_h, x, y, size, 2, border_dark);
    fill_rect(framebuffer, fb_w, fb_h, x, y, 2, size, border_dark);
    fill_rect(framebuffer, fb_w, fb_h, x, y + size - 2, size, 2, border_light);
    fill_rect(framebuffer, fb_w, fb_h, x + size - 2, y, 2, size, border_light);

    if let Some(st) = stack {
        let col = st.block.face_color(Face::Top);
        let icon_col = pack_color(col[0], col[1], col[2]);
        let pad = 6;
        fill_rect(framebuffer, fb_w, fb_h, x + pad, y + pad, size - pad * 2, size - pad * 2, icon_col);

        // Highlight inner cube
        let side_col = st.block.face_color(Face::North);
        let icon_side = pack_color(side_col[0], side_col[1], side_col[2]);
        fill_rect(framebuffer, fb_w, fb_h, x + pad, y + size - pad - 4, size - pad * 2, 4, icon_side);

        if st.count > 1 {
            let count_str = format!("{}", st.count);
            let tx = x + size - 6 - (count_str.len() as i32 * 8);
            let ty = y + size - 14;
            draw_text_shadowed(framebuffer, fb_w, fb_h, &count_str, tx, ty, 1, 1);
        }
    }
}

pub fn draw_chest_ui(
    framebuffer: &mut [u32],
    fb_w: usize,
    fb_h: usize,
    chest: &ChestData,
    player: &Player,
    cursor_pos: Option<(f32, f32)>,
) {
    let layout = ChestLayout::new(fb_w, fb_h);
    let hovered = cursor_pos.and_then(|(mx, my)| layout.find_slot_at(mx as i32, my as i32));

    // Dim background
    fill_rect_alpha(framebuffer, fb_w, fb_h, 0, 0, fb_w as i32, fb_h as i32, 0x000000, 0.55);

    // Main window panel
    let panel_w = 420;
    let panel_h = 380;
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, panel_w, panel_h, pack_rgb(48, 48, 54));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, panel_w, 3, pack_rgb(90, 90, 100));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, 3, panel_h, pack_rgb(90, 90, 100));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y + panel_h - 3, panel_w, 3, pack_rgb(24, 24, 28));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x + panel_w - 3, layout.start_y, 3, panel_h, pack_rgb(24, 24, 28));

    // Headers
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "Skrzynia", layout.start_x + 18, layout.start_y + 12, 1, 1, 0xFFE080);
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "Ekwipunek", layout.start_x + 18, layout.start_y + 165, 1, 1, 0xDDDDDD);

    // 1. Draw Chest Slots (9x3)
    let chest_top_y = layout.start_y + 35;
    let chest_start_x = layout.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
    for r in 0..3 {
        for c in 0..9 {
            let idx = (r * 9 + c) as usize;
            let sx = chest_start_x + c * SLOT_STEP;
            let sy = chest_top_y + r * SLOT_STEP;
            let is_h = hovered == Some(ChestSlotTarget::Chest(idx));
            draw_slot(framebuffer, fb_w, fb_h, sx, sy, SLOT_SIZE, chest.items[idx], is_h);
        }
    }

    // 2. Draw Main Inventory Slots (9x3)
    let main_top_y = layout.start_y + 185;
    for r in 0..3 {
        for c in 0..9 {
            let idx = (r * 9 + c) as usize;
            let sx = chest_start_x + c * SLOT_STEP;
            let sy = main_top_y + r * SLOT_STEP;
            let is_h = hovered == Some(ChestSlotTarget::Main(idx));
            draw_slot(framebuffer, fb_w, fb_h, sx, sy, SLOT_SIZE, player.inventory.main[idx], is_h);
        }
    }

    // 3. Draw Hotbar Slots (10)
    let hotbar_y = layout.start_y + 325;
    let hotbar_start_x = layout.start_x + (420 - 10 * SLOT_STEP + SLOT_GAP) / 2;
    for i in 0..10 {
        let sx = hotbar_start_x + i * SLOT_STEP;
        let is_h = hovered == Some(ChestSlotTarget::Hotbar(i as usize));
        draw_slot(framebuffer, fb_w, fb_h, sx, hotbar_y, SLOT_SIZE, player.inventory.hotbar[i as usize], is_h);
    }

    // Draw Held Item on Cursor
    if let Some((mx, my)) = cursor_pos {
        if let Some(held) = player.inventory.held_item {
            let hx = mx as i32 - SLOT_SIZE / 2;
            let hy = my as i32 - SLOT_SIZE / 2;
            let col = held.block.face_color(Face::Top);
            let icon_col = pack_color(col[0], col[1], col[2]);
            fill_rect(framebuffer, fb_w, fb_h, hx + 4, hy + 4, SLOT_SIZE - 8, SLOT_SIZE - 8, icon_col);
            if held.count > 1 {
                let count_str = format!("{}", held.count);
                let tx = hx + SLOT_SIZE - 6 - (count_str.len() as i32 * 8);
                let ty = hy + SLOT_SIZE - 14;
                draw_text_shadowed(framebuffer, fb_w, fb_h, &count_str, tx, ty, 1, 1);
            }
        } else if let Some(target) = hovered {
            // Draw Item Tooltip
            let item_opt = match target {
                ChestSlotTarget::Chest(i) => chest.items[i],
                ChestSlotTarget::Main(i) => player.inventory.main[i],
                ChestSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
            };
            if let Some(item) = item_opt {
                let text = item.block.display_name();
                let tw = text.len() as i32 * 8 + 12;
                let tx = (mx as i32 + 12).min(fb_w as i32 - tw - 4);
                let ty = (my as i32 - 24).max(4);
                fill_rect(framebuffer, fb_w, fb_h, tx, ty, tw, 20, pack_rgb(20, 20, 26));
                fill_rect(framebuffer, fb_w, fb_h, tx, ty, tw, 1, pack_rgb(80, 80, 100));
                draw_text_colored_shadowed(framebuffer, fb_w, fb_h, text, tx + 6, ty + 3, 1, 1, 0xFFFFFF);
            }
        }
    }
}

pub fn draw_furnace_ui(
    framebuffer: &mut [u32],
    fb_w: usize,
    fb_h: usize,
    furnace: &FurnaceData,
    player: &Player,
    cursor_pos: Option<(f32, f32)>,
) {
    let layout = FurnaceLayout::new(fb_w, fb_h);
    let hovered = cursor_pos.and_then(|(mx, my)| layout.find_slot_at(mx as i32, my as i32));

    // Dim background
    fill_rect_alpha(framebuffer, fb_w, fb_h, 0, 0, fb_w as i32, fb_h as i32, 0x000000, 0.55);

    // Main window panel
    let panel_w = 420;
    let panel_h = 380;
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, panel_w, panel_h, pack_rgb(48, 48, 54));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, panel_w, 3, pack_rgb(90, 90, 100));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, 3, panel_h, pack_rgb(90, 90, 100));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y + panel_h - 3, panel_w, 3, pack_rgb(24, 24, 28));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x + panel_w - 3, layout.start_y, 3, panel_h, pack_rgb(24, 24, 28));

    // Headers
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "Piec", layout.start_x + 18, layout.start_y + 12, 1, 1, 0xFFB040);
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "Ekwipunek", layout.start_x + 18, layout.start_y + 165, 1, 1, 0xDDDDDD);

    let center_x = layout.start_x + 210;

    // 1. Input slot
    let in_x = center_x - 70;
    let in_y = layout.start_y + 38;
    draw_slot(
        framebuffer,
        fb_w,
        fb_h,
        in_x,
        in_y,
        SLOT_SIZE,
        furnace.input,
        hovered == Some(FurnaceSlotTarget::Input),
    );

    // 2. Animated Flame (between Input and Fuel)
    let flame_x = center_x - 60;
    let flame_y = layout.start_y + 82;
    let flame_w = 16;
    let flame_h = 22;
    let is_lit = furnace.is_burning();
    let flame_ratio = if furnace.burn_time_total > 0.0 {
        (furnace.burn_time_remaining / furnace.burn_time_total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Dark flame outline background
    fill_rect(framebuffer, fb_w, fb_h, flame_x, flame_y, flame_w, flame_h, pack_rgb(32, 32, 36));
    if is_lit {
        let lit_h = (flame_h as f32 * flame_ratio) as i32;
        let lit_y = flame_y + flame_h - lit_h;
        fill_rect(framebuffer, fb_w, fb_h, flame_x + 2, lit_y, flame_w - 4, lit_h, pack_rgb(255, 140, 20));
        fill_rect(framebuffer, fb_w, fb_h, flame_x + 5, lit_y + (lit_h / 3), flame_w - 10, lit_h * 2 / 3, pack_rgb(255, 235, 80));
    }

    // 3. Fuel slot
    let fuel_x = center_x - 70;
    let fuel_y = layout.start_y + 112;
    draw_slot(
        framebuffer,
        fb_w,
        fb_h,
        fuel_x,
        fuel_y,
        SLOT_SIZE,
        furnace.fuel,
        hovered == Some(FurnaceSlotTarget::Fuel),
    );

    // 4. Cooking Progress Arrow (pointing to output slot)
    let arrow_x = center_x - 18;
    let arrow_y = layout.start_y + 80;
    let arrow_w = 36;
    let arrow_h = 16;
    let cook_ratio = (furnace.cook_progress / SMELT_DURATION).clamp(0.0, 1.0);

    // Arrow background
    fill_rect(framebuffer, fb_w, fb_h, arrow_x, arrow_y + 4, arrow_w - 10, 8, pack_rgb(32, 32, 36));
    fill_rect(framebuffer, fb_w, fb_h, arrow_x + arrow_w - 10, arrow_y, 10, arrow_h, pack_rgb(32, 32, 36));

    // Arrow filled progress
    let fill_w = (arrow_w as f32 * cook_ratio) as i32;
    if fill_w > 0 {
        let bar_fill = fill_w.min(arrow_w - 10);
        fill_rect(framebuffer, fb_w, fb_h, arrow_x, arrow_y + 4, bar_fill, 8, pack_rgb(255, 200, 50));
        if fill_w > arrow_w - 10 {
            let tip_fill = fill_w - (arrow_w - 10);
            fill_rect(framebuffer, fb_w, fb_h, arrow_x + arrow_w - 10, arrow_y, tip_fill, arrow_h, pack_rgb(255, 200, 50));
        }
    }

    // 5. Output slot (44x44)
    let out_x = center_x + 35;
    let out_y = layout.start_y + 70;
    draw_slot(
        framebuffer,
        fb_w,
        fb_h,
        out_x,
        out_y,
        44,
        furnace.output,
        hovered == Some(FurnaceSlotTarget::Output),
    );

    // 6. Draw Main Inventory Slots (9x3)
    let chest_start_x = layout.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
    let main_top_y = layout.start_y + 185;
    for r in 0..3 {
        for c in 0..9 {
            let idx = (r * 9 + c) as usize;
            let sx = chest_start_x + c * SLOT_STEP;
            let sy = main_top_y + r * SLOT_STEP;
            let is_h = hovered == Some(FurnaceSlotTarget::Main(idx));
            draw_slot(framebuffer, fb_w, fb_h, sx, sy, SLOT_SIZE, player.inventory.main[idx], is_h);
        }
    }

    // 7. Draw Hotbar Slots (10)
    let hotbar_y = layout.start_y + 325;
    let hotbar_start_x = layout.start_x + (420 - 10 * SLOT_STEP + SLOT_GAP) / 2;
    for i in 0..10 {
        let sx = hotbar_start_x + i * SLOT_STEP;
        let is_h = hovered == Some(FurnaceSlotTarget::Hotbar(i as usize));
        draw_slot(framebuffer, fb_w, fb_h, sx, hotbar_y, SLOT_SIZE, player.inventory.hotbar[i as usize], is_h);
    }

    // Draw Held Item on Cursor
    if let Some((mx, my)) = cursor_pos {
        if let Some(held) = player.inventory.held_item {
            let hx = mx as i32 - SLOT_SIZE / 2;
            let hy = my as i32 - SLOT_SIZE / 2;
            let col = held.block.face_color(Face::Top);
            let icon_col = pack_color(col[0], col[1], col[2]);
            fill_rect(framebuffer, fb_w, fb_h, hx + 4, hy + 4, SLOT_SIZE - 8, SLOT_SIZE - 8, icon_col);
            if held.count > 1 {
                let count_str = format!("{}", held.count);
                let tx = hx + SLOT_SIZE - 6 - (count_str.len() as i32 * 8);
                let ty = hy + SLOT_SIZE - 14;
                draw_text_shadowed(framebuffer, fb_w, fb_h, &count_str, tx, ty, 1, 1);
            }
        } else if let Some(target) = hovered {
            // Draw Item Tooltip
            let item_opt = match target {
                FurnaceSlotTarget::Input => furnace.input,
                FurnaceSlotTarget::Fuel => furnace.fuel,
                FurnaceSlotTarget::Output => furnace.output,
                FurnaceSlotTarget::Main(i) => player.inventory.main[i],
                FurnaceSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
            };
            if let Some(item) = item_opt {
                let text = item.block.display_name();
                let tw = text.len() as i32 * 8 + 12;
                let tx = (mx as i32 + 12).min(fb_w as i32 - tw - 4);
                let ty = (my as i32 - 24).max(4);
                fill_rect(framebuffer, fb_w, fb_h, tx, ty, tw, 20, pack_rgb(20, 20, 26));
                fill_rect(framebuffer, fb_w, fb_h, tx, ty, tw, 1, pack_rgb(80, 80, 100));
                draw_text_colored_shadowed(framebuffer, fb_w, fb_h, text, tx + 6, ty + 3, 1, 1, 0xFFFFFF);
            }
        }
    }
}
