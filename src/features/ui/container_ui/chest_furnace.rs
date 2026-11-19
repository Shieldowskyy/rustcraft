//! Układ slotów oraz renderowanie okien skrzyni (`ChestData`) i pieca (`FurnaceData`).

use crate::features::blocks::Face;
use crate::features::player::{ItemStack, Player};
use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Grid, Rect, UiCanvas};
use crate::features::world::{ChestData, FurnaceData, SMELT_DURATION};

pub const SLOT_SIZE: i32 = 36;
pub const SLOT_GAP: i32 = 4;
pub const SLOT_STEP: i32 = SLOT_SIZE + SLOT_GAP;

#[inline(always)]
pub(super) fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

#[inline]
pub(super) const fn centered_slot_grid(
    start_x: i32,
    panel_w: i32,
    y: i32,
    cols: usize,
    rows: usize,
) -> Grid {
    let x = start_x + (panel_w - (cols as i32) * SLOT_STEP + SLOT_GAP) / 2;
    Grid::square(x, y, cols, rows, SLOT_SIZE, SLOT_GAP)
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

    #[inline]
    pub const fn chest_grid(&self) -> Grid {
        centered_slot_grid(self.start_x, 420, self.start_y + 35, 9, 3)
    }

    #[inline]
    pub const fn main_grid(&self) -> Grid {
        centered_slot_grid(self.start_x, 420, self.start_y + 185, 9, 3)
    }

    #[inline]
    pub const fn hotbar_grid(&self) -> Grid {
        centered_slot_grid(self.start_x, 420, self.start_y + 325, 10, 1)
    }

    pub fn find_slot_at(&self, mx: i32, my: i32) -> Option<ChestSlotTarget> {
        if let Some(idx) = self.chest_grid().hit_test(mx, my) {
            return Some(ChestSlotTarget::Chest(idx));
        }
        if let Some(idx) = self.main_grid().hit_test(mx, my) {
            return Some(ChestSlotTarget::Main(idx));
        }
        if let Some(idx) = self.hotbar_grid().hit_test(mx, my) {
            return Some(ChestSlotTarget::Hotbar(idx));
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

    #[inline]
    pub const fn input_rect(&self) -> Rect {
        Rect::new(self.start_x + 140, self.start_y + 38, SLOT_SIZE, SLOT_SIZE)
    }

    #[inline]
    pub const fn fuel_rect(&self) -> Rect {
        Rect::new(self.start_x + 140, self.start_y + 112, SLOT_SIZE, SLOT_SIZE)
    }

    #[inline]
    pub const fn output_rect(&self) -> Rect {
        Rect::new(self.start_x + 245, self.start_y + 70, 44, 44)
    }

    #[inline]
    pub const fn main_grid(&self) -> Grid {
        centered_slot_grid(self.start_x, 420, self.start_y + 185, 9, 3)
    }

    #[inline]
    pub const fn hotbar_grid(&self) -> Grid {
        centered_slot_grid(self.start_x, 420, self.start_y + 325, 10, 1)
    }

    pub fn find_slot_at(&self, mx: i32, my: i32) -> Option<FurnaceSlotTarget> {
        if self.input_rect().contains_point(mx, my) {
            return Some(FurnaceSlotTarget::Input);
        }
        if self.fuel_rect().contains_point(mx, my) {
            return Some(FurnaceSlotTarget::Fuel);
        }
        if self.output_rect().contains_point(mx, my) {
            return Some(FurnaceSlotTarget::Output);
        }
        if let Some(idx) = self.main_grid().hit_test(mx, my) {
            return Some(FurnaceSlotTarget::Main(idx));
        }
        if let Some(idx) = self.hotbar_grid().hit_test(mx, my) {
            return Some(FurnaceSlotTarget::Hotbar(idx));
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
    let mut ui = UiCanvas::new(framebuffer, fb_w, fb_h);
    let r = Rect::new(x, y, size, size);
    let slot_bg = if is_hovered { pack_rgb(70, 70, 80) } else { pack_rgb(40, 40, 44) };
    ui.fill(r, slot_bg);
    ui.bevel_border(r, 2, pack_rgb(20, 20, 22), pack_rgb(90, 90, 100));

    if let Some(st) = stack {
        let col = st.block.face_color(Face::Top);
        let icon_col = pack_color(col[0], col[1], col[2]);
        let pad = 6;
        ui.fill(r.inset(pad), icon_col);

        let side_col = st.block.face_color(Face::North);
        let icon_side = pack_color(side_col[0], side_col[1], side_col[2]);
        ui.fill(Rect::new(x + pad, y + size - pad - 4, size - pad * 2, 4), icon_side);

        if st.count > 1 {
            let count_str = format!("{}", st.count);
            let tx = x + size - 6 - (count_str.len() as i32 * 8);
            let ty = y + size - 14;
            ui.text_white(&count_str, tx, ty, 1, 1);
        }

        ui.item_durability_bar(Rect::new(x + 4, y + size - 5, size - 8, 2), &st);
    }
}

pub(super) fn draw_player_inventory_grids(
    ui: &mut UiCanvas<'_>,
    main_grid: Grid,
    hotbar_grid: Grid,
    player: &Player,
    hovered_main: Option<usize>,
    hovered_hotbar: Option<usize>,
) {
    for idx in 0..27 {
        let cell = main_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.main[idx], hovered_main == Some(idx));
    }
    for idx in 0..10 {
        let cell = hotbar_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.hotbar[idx], hovered_hotbar == Some(idx));
    }
}

fn draw_container_window(ui: &mut UiCanvas<'_>, rect: Rect) {
    ui.backdrop(None, 0x000000, 0.55);
    ui.fill(rect, pack_rgb(48, 48, 54));
    ui.bevel_border(rect, 3, pack_rgb(90, 90, 100), pack_rgb(24, 24, 28));
}

pub(super) fn draw_container_cursor_and_tooltip(
    ui: &mut UiCanvas<'_>,
    held_item: Option<ItemStack>,
    hovered_item: Option<ItemStack>,
    cursor_pos: Option<(f32, f32)>,
) {
    let Some((mx, my)) = cursor_pos else {
        return;
    };
    if let Some(held) = held_item {
        let hx = mx as i32 - SLOT_SIZE / 2;
        let hy = my as i32 - SLOT_SIZE / 2;
        let col = held.block.face_color(Face::Top);
        let icon_col = pack_color(col[0], col[1], col[2]);
        ui.fill(Rect::new(hx + 4, hy + 4, SLOT_SIZE - 8, SLOT_SIZE - 8), icon_col);
        if held.count > 1 {
            let count_str = format!("{}", held.count);
            let tx = hx + SLOT_SIZE - 6 - (count_str.len() as i32 * 8);
            let ty = hy + SLOT_SIZE - 14;
            ui.text_white(&count_str, tx, ty, 1, 1);
        }
    } else if let Some(item) = hovered_item {
        let text = item.block.display_name();
        let tw = text.len() as i32 * 8 + 12;
        let tx = (mx as i32 + 12).min(ui.width as i32 - tw - 4);
        let ty = (my as i32 - 24).max(4);
        ui.fill(Rect::new(tx, ty, tw, 20), pack_rgb(20, 20, 26));
        ui.fill(Rect::new(tx, ty, tw, 1), pack_rgb(80, 80, 100));
        ui.text(text, tx + 6, ty + 3, 1, 1, 0xFFFFFF);
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

    let mut ui = UiCanvas::new(framebuffer, fb_w, fb_h);
    draw_container_window(&mut ui, Rect::new(layout.start_x, layout.start_y, 420, 380));

    ui.text("Skrzynia", layout.start_x + 18, layout.start_y + 12, 1, 1, 0xFFE080);
    ui.text("Ekwipunek", layout.start_x + 18, layout.start_y + 165, 1, 1, 0xDDDDDD);

    let chest_grid = layout.chest_grid();
    for idx in 0..27 {
        let cell = chest_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, chest.items[idx], hovered == Some(ChestSlotTarget::Chest(idx)));
    }

    let hovered_main = match hovered {
        Some(ChestSlotTarget::Main(i)) => Some(i),
        _ => None,
    };
    let hovered_hotbar = match hovered {
        Some(ChestSlotTarget::Hotbar(i)) => Some(i),
        _ => None,
    };
    draw_player_inventory_grids(&mut ui, layout.main_grid(), layout.hotbar_grid(), player, hovered_main, hovered_hotbar);

    let hovered_item = hovered.and_then(|target| match target {
        ChestSlotTarget::Chest(i) => chest.items[i],
        ChestSlotTarget::Main(i) => player.inventory.main[i],
        ChestSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
    });
    draw_container_cursor_and_tooltip(&mut ui, player.inventory.held_item, hovered_item, cursor_pos);
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

    let mut ui = UiCanvas::new(framebuffer, fb_w, fb_h);
    draw_container_window(&mut ui, Rect::new(layout.start_x, layout.start_y, 420, 380));

    ui.text("Piec", layout.start_x + 18, layout.start_y + 12, 1, 1, 0xFFB040);
    ui.text("Ekwipunek", layout.start_x + 18, layout.start_y + 165, 1, 1, 0xDDDDDD);

    let center_x = layout.start_x + 210;

    let in_r = layout.input_rect();
    draw_slot(ui.buffer, ui.width, ui.height, in_r.x, in_r.y, SLOT_SIZE, furnace.input, hovered == Some(FurnaceSlotTarget::Input));

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

    ui.fill(Rect::new(flame_x, flame_y, flame_w, flame_h), pack_rgb(32, 32, 36));
    if is_lit {
        let lit_h = (flame_h as f32 * flame_ratio) as i32;
        let lit_y = flame_y + flame_h - lit_h;
        ui.fill(Rect::new(flame_x + 2, lit_y, flame_w - 4, lit_h), pack_rgb(255, 140, 20));
        ui.fill(Rect::new(flame_x + 5, lit_y + (lit_h / 3), flame_w - 10, lit_h * 2 / 3), pack_rgb(255, 235, 80));
    }

    let fuel_r = layout.fuel_rect();
    draw_slot(ui.buffer, ui.width, ui.height, fuel_r.x, fuel_r.y, SLOT_SIZE, furnace.fuel, hovered == Some(FurnaceSlotTarget::Fuel));

    let arrow_x = center_x - 18;
    let arrow_y = layout.start_y + 80;
    let arrow_w = 36;
    let arrow_h = 16;
    let cook_ratio = (furnace.cook_progress / SMELT_DURATION).clamp(0.0, 1.0);

    ui.fill(Rect::new(arrow_x, arrow_y + 4, arrow_w - 10, 8), pack_rgb(32, 32, 36));
    ui.fill(Rect::new(arrow_x + arrow_w - 10, arrow_y, 10, arrow_h), pack_rgb(32, 32, 36));

    let fill_w = (arrow_w as f32 * cook_ratio) as i32;
    if fill_w > 0 {
        let bar_fill = fill_w.min(arrow_w - 10);
        ui.fill(Rect::new(arrow_x, arrow_y + 4, bar_fill, 8), pack_rgb(255, 200, 50));
        if fill_w > arrow_w - 10 {
            let tip_fill = fill_w - (arrow_w - 10);
            ui.fill(Rect::new(arrow_x + arrow_w - 10, arrow_y, tip_fill, arrow_h), pack_rgb(255, 200, 50));
        }
    }

    let out_r = layout.output_rect();
    draw_slot(ui.buffer, ui.width, ui.height, out_r.x, out_r.y, out_r.w, furnace.output, hovered == Some(FurnaceSlotTarget::Output));

    let hovered_main = match hovered {
        Some(FurnaceSlotTarget::Main(i)) => Some(i),
        _ => None,
    };
    let hovered_hotbar = match hovered {
        Some(FurnaceSlotTarget::Hotbar(i)) => Some(i),
        _ => None,
    };
    draw_player_inventory_grids(&mut ui, layout.main_grid(), layout.hotbar_grid(), player, hovered_main, hovered_hotbar);

    let hovered_item = hovered.and_then(|target| match target {
        FurnaceSlotTarget::Input => furnace.input,
        FurnaceSlotTarget::Fuel => furnace.fuel,
        FurnaceSlotTarget::Output => furnace.output,
        FurnaceSlotTarget::Main(i) => player.inventory.main[i],
        FurnaceSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
    });
    draw_container_cursor_and_tooltip(&mut ui, player.inventory.held_item, hovered_item, cursor_pos);
}
