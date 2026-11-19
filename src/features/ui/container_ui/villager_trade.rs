//! Układ slotów oraz renderowanie interfejsu handlu z osadnikami (`VillagerProfession`, `TradeOffer`).

use crate::features::blocks::{BlockId, Face};
use crate::features::player::{
    Player, TradeOffer, VillagerProfession, count_item_in_player,
};
use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Grid, Rect, UiCanvas};

use super::chest_furnace::{
    SLOT_SIZE, centered_slot_grid, draw_container_cursor_and_tooltip,
    draw_player_inventory_grids, draw_slot, pack_rgb,
};

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
        centered_slot_grid(self.start_x, self.panel_w, self.start_y + 190, 9, 3)
    }

    #[inline]
    pub const fn hotbar_grid(&self) -> Grid {
        centered_slot_grid(self.start_x, self.panel_w, self.start_y + 335, 10, 1)
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
    profession: VillagerProfession,
    trades: &[TradeOffer],
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

        let em_count = count_item_in_player(player, BlockId::Emerald);
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

    let hovered_main = match hovered {
        Some(VillagerTradeSlotTarget::Main(i)) => Some(i),
        _ => None,
    };
    let hovered_hotbar = match hovered {
        Some(VillagerTradeSlotTarget::Hotbar(i)) => Some(i),
        _ => None,
    };
    draw_player_inventory_grids(&mut ui, layout.main_grid(), layout.hotbar_grid(), player, hovered_main, hovered_hotbar);

    let hovered_item = hovered.and_then(|target| match target {
        VillagerTradeSlotTarget::Main(i) => player.inventory.main[i],
        VillagerTradeSlotTarget::Hotbar(i) => player.inventory.hotbar[i],
        _ => None,
    });
    draw_container_cursor_and_tooltip(&mut ui, None, hovered_item, cursor_pos);
}
