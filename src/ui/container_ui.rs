use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::Player;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::block::{BlockId, Face};
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

        if st.block.is_damageable() && st.durability < st.block.max_durability() {
            let ratio = st.durability_ratio();
            let bar_w = ((size - 8) as f32 * ratio).round() as i32;
            let bar_x = x + 4;
            let bar_y = y + size - 5;
            let color = if ratio > 0.6 {
                pack_color(0.2, 0.9, 0.2)
            } else if ratio > 0.25 {
                pack_color(0.95, 0.75, 0.1)
            } else {
                pack_color(0.9, 0.2, 0.2)
            };
            fill_rect(framebuffer, fb_w, fb_h, bar_x, bar_y, size - 8, 2, pack_color(0.0, 0.0, 0.0));
            if bar_w > 0 {
                fill_rect(framebuffer, fb_w, fb_h, bar_x, bar_y, bar_w, 2, color);
            }
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VillagerTradeSlotTarget {
    TradeOption(usize),
    TradeButton,
    Main(usize),
    Hotbar(usize),
}

pub struct VillagerTradeLayout {
    pub start_x: i32,
    pub start_y: i32,
    pub panel_w: i32,
    pub panel_h: i32,
}

impl VillagerTradeLayout {
    pub fn new(width: usize, height: usize) -> Self {
        let panel_w = 480;
        let panel_h = 390;
        let start_x = (width as i32 - panel_w) / 2;
        let start_y = (height as i32 - panel_h) / 2;
        Self {
            start_x,
            start_y,
            panel_w,
            panel_h,
        }
    }

    pub fn find_target_at(&self, mx: i32, my: i32, trade_count: usize) -> Option<VillagerTradeSlotTarget> {
        let list_x = self.start_x + 16;
        let list_y = self.start_y + 40;
        for i in 0..trade_count.min(4) {
            let row_y = list_y + (i as i32 * 36);
            if mx >= list_x && mx < list_x + 180 && my >= row_y && my < row_y + 32 {
                return Some(VillagerTradeSlotTarget::TradeOption(i));
            }
        }

        let btn_x = self.start_x + 230;
        let btn_y = self.start_y + 130;
        if mx >= btn_x && mx < btn_x + 210 && my >= btn_y && my < btn_y + 32 {
            return Some(VillagerTradeSlotTarget::TradeButton);
        }

        let chest_start_x = self.start_x + (self.panel_w - 9 * SLOT_STEP + SLOT_GAP) / 2;
        let main_top_y = self.start_y + 190;
        for r in 0..3 {
            for c in 0..9 {
                let sx = chest_start_x + c * SLOT_STEP;
                let sy = main_top_y + r * SLOT_STEP;
                if mx >= sx && mx < sx + SLOT_SIZE && my >= sy && my < sy + SLOT_SIZE {
                    return Some(VillagerTradeSlotTarget::Main((r * 9 + c) as usize));
                }
            }
        }

        let hotbar_y = self.start_y + 335;
        let hotbar_start_x = self.start_x + (self.panel_w - 10 * SLOT_STEP + SLOT_GAP) / 2;
        for i in 0..10 {
            let sx = hotbar_start_x + i * SLOT_STEP;
            if mx >= sx && mx < sx + SLOT_SIZE && my >= hotbar_y && my < hotbar_y + SLOT_SIZE {
                return Some(VillagerTradeSlotTarget::Hotbar(i as usize));
            }
        }

        None
    }
}

pub fn draw_villager_trade_ui(
    framebuffer: &mut [u32],
    fb_w: usize,
    fb_h: usize,
    profession: crate::gameplay::trading::VillagerProfession,
    trades: &[crate::gameplay::trading::TradeOffer],
    selected_trade: usize,
    player: &Player,
    cursor_pos: Option<(f32, f32)>,
) {
    let layout = VillagerTradeLayout::new(fb_w, fb_h);
    let hovered = cursor_pos.and_then(|(mx, my)| layout.find_target_at(mx as i32, my as i32, trades.len()));

    fill_rect_alpha(framebuffer, fb_w, fb_h, 0, 0, fb_w as i32, fb_h as i32, 0x000000, 0.55);

    // Main window panel
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, layout.panel_w, layout.panel_h, pack_rgb(46, 46, 52));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, layout.panel_w, 3, pack_rgb(90, 90, 100));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y, 3, layout.panel_h, pack_rgb(90, 90, 100));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x, layout.start_y + layout.panel_h - 3, layout.panel_w, 3, pack_rgb(24, 24, 28));
    fill_rect(framebuffer, fb_w, fb_h, layout.start_x + layout.panel_w - 3, layout.start_y, 3, layout.panel_h, pack_rgb(24, 24, 28));

    // Title Header
    let title = format!("Osadnik: {}", profession.display_name());
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, &title, layout.start_x + 18, layout.start_y + 12, 1, 1, 0x2ECC71);

    // Left Column: Trade offers list
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "Oferty", layout.start_x + 18, layout.start_y + 28, 1, 1, 0xAAAAAA);
    let list_x = layout.start_x + 16;
    let list_y = layout.start_y + 42;

    for (i, trade) in trades.iter().take(4).enumerate() {
        let row_y = list_y + (i as i32 * 36);
        let is_selected = i == selected_trade;
        let is_hovered = hovered == Some(VillagerTradeSlotTarget::TradeOption(i));
        let can_afford = trade.player_can_afford(player);

        let bg_col = if is_selected {
            pack_rgb(65, 70, 85)
        } else if is_hovered {
            pack_rgb(52, 54, 62)
        } else {
            pack_rgb(36, 36, 42)
        };
        fill_rect(framebuffer, fb_w, fb_h, list_x, row_y, 185, 32, bg_col);

        let border_col = if is_selected {
            pack_rgb(255, 215, 0)
        } else if can_afford {
            pack_rgb(46, 204, 113)
        } else {
            pack_rgb(24, 24, 28)
        };
        fill_rect(framebuffer, fb_w, fb_h, list_x, row_y, 185, 1, border_col);
        fill_rect(framebuffer, fb_w, fb_h, list_x, row_y + 31, 185, 1, border_col);
        fill_rect(framebuffer, fb_w, fb_h, list_x, row_y, 1, 32, border_col);
        fill_rect(framebuffer, fb_w, fb_h, list_x + 184, row_y, 1, 32, border_col);

        // Indicator dot
        let dot_col = if can_afford { pack_rgb(46, 204, 113) } else { pack_rgb(200, 60, 60) };
        fill_rect(framebuffer, fb_w, fb_h, list_x + 4, row_y + 12, 6, 8, dot_col);

        // Cost A mini icon & count
        let col_a = trade.cost_a.block.face_color(Face::Top);
        fill_rect(framebuffer, fb_w, fb_h, list_x + 14, row_y + 6, 20, 20, pack_color(col_a[0], col_a[1], col_a[2]));
        let count_a_str = format!("{}", trade.cost_a.count);
        draw_text_shadowed(framebuffer, fb_w, fb_h, &count_a_str, list_x + 36, row_y + 10, 1, 1);

        // Arrow
        let arrow_x = if trade.cost_b.is_some() { list_x + 95 } else { list_x + 80 };
        draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "->", arrow_x, row_y + 10, 1, 1, 0xFFFFFF);

        // Result mini icon & count
        let col_r = trade.result.block.face_color(Face::Top);
        let res_x = arrow_x + 22;
        fill_rect(framebuffer, fb_w, fb_h, res_x, row_y + 6, 20, 20, pack_color(col_r[0], col_r[1], col_r[2]));
        let count_r_str = format!("{}", trade.result.count);
        draw_text_shadowed(framebuffer, fb_w, fb_h, &count_r_str, res_x + 24, row_y + 10, 1, 1);
    }

    // Right Column: Detail Box for selected trade
    let detail_x = layout.start_x + 215;
    let detail_y = layout.start_y + 42;
    let detail_w = 245;
    let detail_h = 138;
    fill_rect(framebuffer, fb_w, fb_h, detail_x, detail_y, detail_w, detail_h, pack_rgb(34, 34, 40));
    fill_rect(framebuffer, fb_w, fb_h, detail_x, detail_y, detail_w, 2, pack_rgb(60, 60, 70));
    fill_rect(framebuffer, fb_w, fb_h, detail_x, detail_y, 2, detail_h, pack_rgb(60, 60, 70));

    if let Some(trade) = trades.get(selected_trade) {
        let can_afford = trade.player_can_afford(player);

        // Slot Cost A
        draw_slot(framebuffer, fb_w, fb_h, detail_x + 16, detail_y + 16, SLOT_SIZE, Some(trade.cost_a), false);

        // Slot Cost B if present
        if let Some(cost_b) = trade.cost_b {
            draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "+", detail_x + 58, detail_y + 26, 1, 1, 0xFFFFFF);
            draw_slot(framebuffer, fb_w, fb_h, detail_x + 72, detail_y + 16, SLOT_SIZE, Some(cost_b), false);
        }

        // Arrow
        draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "===>", detail_x + 120, detail_y + 26, 1, 1, 0x2ECC71);

        // Result slot
        draw_slot(framebuffer, fb_w, fb_h, detail_x + 180, detail_y + 16, SLOT_SIZE, Some(trade.result), false);

        // Player Emerald balance
        let em_count = crate::gameplay::trading::count_item_in_player(player, BlockId::Emerald);
        let balance_str = format!("Twoje szmaragdy: {}", em_count);
        draw_text_colored_shadowed(framebuffer, fb_w, fb_h, &balance_str, detail_x + 16, detail_y + 60, 1, 1, 0x2ECC71);

        // Trade Button
        let btn_x = detail_x + 16;
        let btn_y = detail_y + 85;
        let btn_w = detail_w - 32;
        let btn_h = 32;
        let is_btn_hovered = hovered == Some(VillagerTradeSlotTarget::TradeButton);

        let (btn_bg, btn_text_col, btn_text) = if can_afford {
            if is_btn_hovered {
                (pack_rgb(46, 180, 100), 0xFFFFFF, "WYMIEN (TRADE)")
            } else {
                (pack_rgb(38, 150, 80), 0xF0FFF0, "WYMIEN (TRADE)")
            }
        } else {
            (pack_rgb(48, 48, 54), 0x888888, "Brak surowcow")
        };

        fill_rect(framebuffer, fb_w, fb_h, btn_x, btn_y, btn_w, btn_h, btn_bg);
        fill_rect(framebuffer, fb_w, fb_h, btn_x, btn_y, btn_w, 2, if can_afford { pack_rgb(80, 220, 140) } else { pack_rgb(30, 30, 34) });
        fill_rect(framebuffer, fb_w, fb_h, btn_x, btn_y + btn_h - 2, btn_w, 2, pack_rgb(20, 20, 24));
        let tw = btn_text.len() as i32 * 8;
        let tx = btn_x + (btn_w - tw) / 2;
        draw_text_colored_shadowed(framebuffer, fb_w, fb_h, btn_text, tx, btn_y + 10, 1, 1, btn_text_col);
    }

    // Player inventory header
    draw_text_colored_shadowed(framebuffer, fb_w, fb_h, "Ekwipunek gracza", layout.start_x + 18, layout.start_y + 175, 1, 1, 0xDDDDDD);

    // Main Inventory Slots (9x3)
    let chest_start_x = layout.start_x + (layout.panel_w - 9 * SLOT_STEP + SLOT_GAP) / 2;
    let main_top_y = layout.start_y + 190;
    for r in 0..3 {
        for c in 0..9 {
            let idx = (r * 9 + c) as usize;
            let sx = chest_start_x + c * SLOT_STEP;
            let sy = main_top_y + r * SLOT_STEP;
            let is_h = hovered == Some(VillagerTradeSlotTarget::Main(idx));
            draw_slot(framebuffer, fb_w, fb_h, sx, sy, SLOT_SIZE, player.inventory.main[idx], is_h);
        }
    }

    // Hotbar Slots (10)
    let hotbar_y = layout.start_y + 335;
    let hotbar_start_x = layout.start_x + (layout.panel_w - 10 * SLOT_STEP + SLOT_GAP) / 2;
    for i in 0..10 {
        let sx = hotbar_start_x + i * SLOT_STEP;
        let is_h = hovered == Some(VillagerTradeSlotTarget::Hotbar(i as usize));
        draw_slot(framebuffer, fb_w, fb_h, sx, hotbar_y, SLOT_SIZE, player.inventory.hotbar[i as usize], is_h);
    }

    // Tooltip for hovered inventory slots
    if let (Some((mx, my)), Some(target)) = (cursor_pos, hovered) {
        let item_opt = match target {
            VillagerTradeSlotTarget::Main(i) => player.inventory.main[i],
            VillagerTradeSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
            _ => None,
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
