use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::Player;
use crate::ui::draw::pack_color;
use crate::ui::widgets::{Grid, Rect, UiCanvas};
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

    #[inline]
    pub const fn chest_grid(&self) -> Grid {
        let x = self.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 35, 9, 3, SLOT_SIZE, SLOT_GAP)
    }

    #[inline]
    pub const fn main_grid(&self) -> Grid {
        let x = self.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 185, 9, 3, SLOT_SIZE, SLOT_GAP)
    }

    #[inline]
    pub const fn hotbar_grid(&self) -> Grid {
        let x = self.start_x + (420 - 10 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 325, 10, 1, SLOT_SIZE, SLOT_GAP)
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
        let x = self.start_x + (420 - 9 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 185, 9, 3, SLOT_SIZE, SLOT_GAP)
    }

    #[inline]
    pub const fn hotbar_grid(&self) -> Grid {
        let x = self.start_x + (420 - 10 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 325, 10, 1, SLOT_SIZE, SLOT_GAP)
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

        if st.block.is_damageable() && st.durability < st.block.max_durability() {
            let ratio = st.durability_ratio();
            let color = if ratio > 0.6 {
                pack_color(0.2, 0.9, 0.2)
            } else if ratio > 0.25 {
                pack_color(0.95, 0.75, 0.1)
            } else {
                pack_color(0.9, 0.2, 0.2)
            };
            ui.progress_bar(Rect::new(x + 4, y + size - 5, size - 8, 2), ratio, pack_color(0.0, 0.0, 0.0), color);
        }
    }
}

fn draw_container_window(ui: &mut UiCanvas<'_>, rect: Rect) {
    ui.backdrop(None, 0x000000, 0.55);
    ui.fill(rect, pack_rgb(48, 48, 54));
    ui.bevel_border(rect, 3, pack_rgb(90, 90, 100), pack_rgb(24, 24, 28));
}

fn draw_container_cursor_and_tooltip(
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

    let main_grid = layout.main_grid();
    for idx in 0..27 {
        let cell = main_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.main[idx], hovered == Some(ChestSlotTarget::Main(idx)));
    }

    let hotbar_grid = layout.hotbar_grid();
    for idx in 0..10 {
        let cell = hotbar_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.hotbar[idx], hovered == Some(ChestSlotTarget::Hotbar(idx)));
    }

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

    // 1. Input slot
    let in_r = layout.input_rect();
    draw_slot(ui.buffer, ui.width, ui.height, in_r.x, in_r.y, SLOT_SIZE, furnace.input, hovered == Some(FurnaceSlotTarget::Input));

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

    ui.fill(Rect::new(flame_x, flame_y, flame_w, flame_h), pack_rgb(32, 32, 36));
    if is_lit {
        let lit_h = (flame_h as f32 * flame_ratio) as i32;
        let lit_y = flame_y + flame_h - lit_h;
        ui.fill(Rect::new(flame_x + 2, lit_y, flame_w - 4, lit_h), pack_rgb(255, 140, 20));
        ui.fill(Rect::new(flame_x + 5, lit_y + (lit_h / 3), flame_w - 10, lit_h * 2 / 3), pack_rgb(255, 235, 80));
    }

    // 3. Fuel slot
    let fuel_r = layout.fuel_rect();
    draw_slot(ui.buffer, ui.width, ui.height, fuel_r.x, fuel_r.y, SLOT_SIZE, furnace.fuel, hovered == Some(FurnaceSlotTarget::Fuel));

    // 4. Cooking Progress Arrow
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

    // 5. Output slot (44x44)
    let out_r = layout.output_rect();
    draw_slot(ui.buffer, ui.width, ui.height, out_r.x, out_r.y, out_r.w, furnace.output, hovered == Some(FurnaceSlotTarget::Output));

    // 6. Main Inventory Slots (9x3)
    let main_grid = layout.main_grid();
    for idx in 0..27 {
        let cell = main_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.main[idx], hovered == Some(FurnaceSlotTarget::Main(idx)));
    }

    // 7. Hotbar Slots (10)
    let hotbar_grid = layout.hotbar_grid();
    for idx in 0..10 {
        let cell = hotbar_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.hotbar[idx], hovered == Some(FurnaceSlotTarget::Hotbar(idx)));
    }

    let hovered_item = hovered.and_then(|target| match target {
        FurnaceSlotTarget::Input => furnace.input,
        FurnaceSlotTarget::Fuel => furnace.fuel,
        FurnaceSlotTarget::Output => furnace.output,
        FurnaceSlotTarget::Main(i) => player.inventory.main[i],
        FurnaceSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
    });
    draw_container_cursor_and_tooltip(&mut ui, player.inventory.held_item, hovered_item, cursor_pos);
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

    #[inline]
    pub const fn main_grid(&self) -> Grid {
        let x = self.start_x + (self.panel_w - 9 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 190, 9, 3, SLOT_SIZE, SLOT_GAP)
    }

    #[inline]
    pub const fn hotbar_grid(&self) -> Grid {
        let x = self.start_x + (self.panel_w - 10 * SLOT_STEP + SLOT_GAP) / 2;
        Grid::square(x, self.start_y + 335, 10, 1, SLOT_SIZE, SLOT_GAP)
    }

    pub fn find_target_at(&self, mx: i32, my: i32, trade_count: usize) -> Option<VillagerTradeSlotTarget> {
        let offers_grid = Grid::new(self.start_x + 16, self.start_y + 40, 1, trade_count.min(4), 180, 32, 180, 36);
        if let Some(i) = offers_grid.hit_test(mx, my) {
            return Some(VillagerTradeSlotTarget::TradeOption(i));
        }

        if Rect::new(self.start_x + 230, self.start_y + 130, 210, 32).contains_point(mx, my) {
            return Some(VillagerTradeSlotTarget::TradeButton);
        }

        if let Some(idx) = self.main_grid().hit_test(mx, my) {
            return Some(VillagerTradeSlotTarget::Main(idx));
        }
        if let Some(idx) = self.hotbar_grid().hit_test(mx, my) {
            return Some(VillagerTradeSlotTarget::Hotbar(idx));
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

    let mut ui = UiCanvas::new(framebuffer, fb_w, fb_h);
    ui.backdrop(None, 0x000000, 0.55);

    let panel_rect = Rect::new(layout.start_x, layout.start_y, layout.panel_w, layout.panel_h);
    ui.fill(panel_rect, pack_rgb(46, 46, 52));
    ui.bevel_border(panel_rect, 3, pack_rgb(90, 90, 100), pack_rgb(24, 24, 28));

    let title = format!("Osadnik: {}", profession.display_name());
    ui.text(&title, layout.start_x + 18, layout.start_y + 12, 1, 1, 0x2ECC71);
    ui.text("Oferty", layout.start_x + 18, layout.start_y + 28, 1, 1, 0xAAAAAA);

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
        let border_col = if is_selected {
            pack_rgb(255, 215, 0)
        } else if can_afford {
            pack_rgb(46, 204, 113)
        } else {
            pack_rgb(24, 24, 28)
        };
        ui.bordered_box(Rect::new(list_x, row_y, 185, 32), 1, bg_col, border_col);

        let dot_col = if can_afford { pack_rgb(46, 204, 113) } else { pack_rgb(200, 60, 60) };
        ui.fill(Rect::new(list_x + 4, row_y + 12, 6, 8), dot_col);

        let col_a = trade.cost_a.block.face_color(Face::Top);
        ui.fill(Rect::new(list_x + 14, row_y + 6, 20, 20), pack_color(col_a[0], col_a[1], col_a[2]));
        let count_a_str = format!("{}", trade.cost_a.count);
        ui.text_white(&count_a_str, list_x + 36, row_y + 10, 1, 1);

        let arrow_x = if trade.cost_b.is_some() { list_x + 95 } else { list_x + 80 };
        ui.text("->", arrow_x, row_y + 10, 1, 1, 0xFFFFFF);

        let col_r = trade.result.block.face_color(Face::Top);
        let res_x = arrow_x + 22;
        ui.fill(Rect::new(res_x, row_y + 6, 20, 20), pack_color(col_r[0], col_r[1], col_r[2]));
        let count_r_str = format!("{}", trade.result.count);
        ui.text_white(&count_r_str, res_x + 24, row_y + 10, 1, 1);
    }

    // Right Column: Detail Box for selected trade
    let detail_x = layout.start_x + 215;
    let detail_y = layout.start_y + 42;
    let detail_w = 245;
    let detail_h = 138;
    ui.fill(Rect::new(detail_x, detail_y, detail_w, detail_h), pack_rgb(34, 34, 40));
    ui.fill(Rect::new(detail_x, detail_y, detail_w, 2), pack_rgb(60, 60, 70));
    ui.fill(Rect::new(detail_x, detail_y, 2, detail_h), pack_rgb(60, 60, 70));

    if let Some(trade) = trades.get(selected_trade) {
        let can_afford = trade.player_can_afford(player);

        draw_slot(ui.buffer, ui.width, ui.height, detail_x + 16, detail_y + 16, SLOT_SIZE, Some(trade.cost_a), false);

        if let Some(cost_b) = trade.cost_b {
            ui.text("+", detail_x + 58, detail_y + 26, 1, 1, 0xFFFFFF);
            draw_slot(ui.buffer, ui.width, ui.height, detail_x + 72, detail_y + 16, SLOT_SIZE, Some(cost_b), false);
        }

        ui.text("===>", detail_x + 120, detail_y + 26, 1, 1, 0x2ECC71);
        draw_slot(ui.buffer, ui.width, ui.height, detail_x + 180, detail_y + 16, SLOT_SIZE, Some(trade.result), false);

        let em_count = crate::gameplay::trading::count_item_in_player(player, BlockId::Emerald);
        let balance_str = format!("Twoje szmaragdy: {}", em_count);
        ui.text(&balance_str, detail_x + 16, detail_y + 60, 1, 1, 0x2ECC71);

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

        ui.fill(Rect::new(btn_x, btn_y, btn_w, btn_h), btn_bg);
        ui.fill(Rect::new(btn_x, btn_y, btn_w, 2), if can_afford { pack_rgb(80, 220, 140) } else { pack_rgb(30, 30, 34) });
        ui.fill(Rect::new(btn_x, btn_y + btn_h - 2, btn_w, 2), pack_rgb(20, 20, 24));
        let tw = btn_text.len() as i32 * 8;
        let tx = btn_x + (btn_w - tw) / 2;
        ui.text(btn_text, tx, btn_y + 10, 1, 1, btn_text_col);
    }

    ui.text("Ekwipunek gracza", layout.start_x + 18, layout.start_y + 175, 1, 1, 0xDDDDDD);

    let main_grid = layout.main_grid();
    for idx in 0..27 {
        let cell = main_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.main[idx], hovered == Some(VillagerTradeSlotTarget::Main(idx)));
    }

    let hotbar_grid = layout.hotbar_grid();
    for idx in 0..10 {
        let cell = hotbar_grid.slot(idx);
        draw_slot(ui.buffer, ui.width, ui.height, cell.x, cell.y, SLOT_SIZE, player.inventory.hotbar[idx], hovered == Some(VillagerTradeSlotTarget::Hotbar(idx)));
    }

    let hovered_item = hovered.and_then(|target| match target {
        VillagerTradeSlotTarget::Main(i) => player.inventory.main[i],
        VillagerTradeSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
        _ => None,
    });
    draw_container_cursor_and_tooltip(&mut ui, None, hovered_item, cursor_pos);
}
