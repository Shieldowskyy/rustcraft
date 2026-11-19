//! Obliczanie geometrii układu okna księgi zadań (Questbook) oraz testowanie trafień kursora.

use crate::features::player::{QuestCategory, quests_in_category};
use crate::features::ui::widgets::Rect;

pub const QUESTBOOK_VISIBLE_CARDS: usize = 5;

#[derive(Clone, Copy)]
pub(super) struct QuestbookLayout {
    pub panel: Rect,
    pub tab_bar: Rect,
    pub list_x: i32,
    pub list_y: i32,
    pub list_w: i32,
    pub card_h: i32,
    pub card_gap: i32,
    pub detail: Rect,
    pub claim_rect: Rect,
    pub close_rect: Rect,
}

pub(super) fn compute_layout(width: usize, height: usize) -> QuestbookLayout {
    let w = width as i32;
    let h = height as i32;
    let panel_w = (w - 48).clamp(480, 960);
    let panel_h = (h - 48).clamp(360, 620);
    let panel = Rect::centered_in(width, height, panel_w, panel_h);

    let tab_y = panel.y + 44;
    let tab_h = 28;
    let tab_bar = Rect::new(panel.x + 16, tab_y, panel.w - 32, tab_h);

    let content_y = tab_y + tab_h + 12;
    let content_h = panel.h - (content_y - panel.y) - 18;

    let list_x = panel.x + 16;
    let list_w = (panel.w * 46 / 100).max(220);
    let card_gap = 8;
    let card_h = ((content_h - card_gap * 4) / 5).clamp(44, 68);

    let detail_x = list_x + list_w + 14;
    let detail_w = panel.right() - 16 - detail_x;
    let detail = Rect::new(detail_x, content_y, detail_w, content_h);

    let claim_w = (detail.w - 32).min(240);
    let claim_h = 34;
    let claim_rect = Rect::new(
        detail.x + (detail.w - claim_w) / 2,
        detail.bottom() - claim_h - 14,
        claim_w,
        claim_h,
    );

    let close_rect = Rect::new(panel.right() - 34, panel.y + 10, 24, 24);

    QuestbookLayout {
        panel,
        tab_bar,
        list_x,
        list_y: content_y,
        list_w,
        card_h,
        card_gap,
        detail,
        claim_rect,
        close_rect,
    }
}

pub(super) fn tab_rects(layout: &QuestbookLayout) -> [Rect; QuestCategory::ALL.len()] {
    layout.tab_bar.split_cols(6)
}

pub fn questbook_tab_at(
    width: usize,
    height: usize,
    mouse_pos: Option<(f32, f32)>,
) -> Option<usize> {
    let layout = compute_layout(width, height);
    tab_rects(&layout)
        .iter()
        .position(|r| r.contains_mouse(mouse_pos))
}

pub fn questbook_card_at(
    width: usize,
    height: usize,
    tab: usize,
    scroll: usize,
    mouse_pos: Option<(f32, f32)>,
) -> Option<usize> {
    let layout = compute_layout(width, height);
    let cat = QuestCategory::ALL
        .get(tab)
        .copied()
        .unwrap_or(QuestCategory::Basics);
    let quests = quests_in_category(cat);
    for (row, &q_idx) in quests
        .iter()
        .skip(scroll)
        .take(QUESTBOOK_VISIBLE_CARDS)
        .enumerate()
    {
        let cy = layout.list_y + (row as i32) * (layout.card_h + layout.card_gap);
        let rect = Rect::new(layout.list_x, cy, layout.list_w, layout.card_h);
        if rect.contains_mouse(mouse_pos) {
            return Some(q_idx);
        }
    }
    None
}

pub fn questbook_claim_hit(width: usize, height: usize, mouse_pos: Option<(f32, f32)>) -> bool {
    compute_layout(width, height).claim_rect.contains_mouse(mouse_pos)
}

pub fn questbook_close_hit(width: usize, height: usize, mouse_pos: Option<(f32, f32)>) -> bool {
    compute_layout(width, height).close_rect.contains_mouse(mouse_pos)
}
