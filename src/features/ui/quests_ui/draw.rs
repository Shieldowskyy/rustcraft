//! Renderowanie ekranu księgi zadań (Questbook), szczegółów zadania, nagród oraz powiadomienia toast.

use crate::features::player::{
    QUEST_COUNT, QUESTS, QuestCategory, QuestProgress, quests_in_category,
};
use crate::features::ui::draw::pack_color;
use crate::features::ui::inventory::draw_item_icon;
use crate::features::ui::widgets::{Button, ButtonVariant, Rect, Toast, UiCanvas};

use super::layout::{QUESTBOOK_VISIBLE_CARDS, compute_layout, tab_rects};

pub fn draw_questbook(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    progress: &QuestProgress,
    active_tab: usize,
    selected_quest: usize,
    scroll: usize,
    key_hint: &str,
    mouse_pos: Option<(f32, f32)>,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.backdrop(None, 0xFF000000, 0.62);

    let layout = compute_layout(width, height);
    let cat = QuestCategory::ALL
        .get(active_tab)
        .copied()
        .unwrap_or(QuestCategory::Basics);

    ui.fill_alpha(layout.panel, pack_color(0.06, 0.07, 0.10), 0.96);
    ui.border(layout.panel, 2, pack_color(0.28, 0.34, 0.45));

    ui.text(
        crate::core::i18n::t("questbook.title"),
        layout.panel.x + 18,
        layout.panel.y + 12,
        2,
        2,
        pack_color(0.98, 0.82, 0.32),
    );

    let summary = format!(
        "{}: {}/{}   [{}]",
        crate::core::i18n::t("questbook.completed_label"),
        progress.completed_count(),
        QUEST_COUNT,
        key_hint
    );
    ui.text_right(
        &summary,
        layout.close_rect.x - 14,
        layout.panel.y + 17,
        1,
        1,
        pack_color(0.75, 0.80, 0.90),
    );

    let close_hovered = layout.close_rect.contains_mouse(mouse_pos);
    ui.fill(
        layout.close_rect,
        if close_hovered {
            pack_color(0.72, 0.22, 0.22)
        } else {
            pack_color(0.18, 0.20, 0.26)
        },
    );
    ui.text_in_rect(layout.close_rect, "X", 1, 1, 0xFFFFFFFF);

    let tabs = tab_rects(&layout);
    for (idx, (&c, &r)) in QuestCategory::ALL.iter().zip(tabs.iter()).enumerate() {
        let is_active = idx == active_tab;
        let is_hover = r.contains_mouse(mouse_pos);
        let bg = if is_active {
            pack_color(0.16, 0.20, 0.28)
        } else if is_hover {
            pack_color(0.12, 0.15, 0.22)
        } else {
            pack_color(0.09, 0.11, 0.16)
        };
        ui.fill(r, bg);
        if is_active {
            ui.fill(Rect::new(r.x, r.bottom() - 3, r.w, 3), c.accent_color());
        }

        let cat_quests = quests_in_category(c);
        let claimable_in_cat = cat_quests
            .iter()
            .filter(|&&qi| progress.is_completed(qi) && !progress.is_claimed(qi))
            .count();

        let label = if claimable_in_cat > 0 {
            format!("{} ({})", crate::core::i18n::t(c.title_key()), claimable_in_cat)
        } else {
            crate::core::i18n::t(c.title_key()).to_string()
        };
        let text_col = if claimable_in_cat > 0 {
            pack_color(0.98, 0.85, 0.35)
        } else if is_active {
            0xFFFFFFFF
        } else {
            pack_color(0.70, 0.75, 0.82)
        };
        ui.text_in_rect(r, &label, 1, 1, text_col);
    }

    let cat_quests = quests_in_category(cat);
    let active_quest_idx = if cat_quests.contains(&selected_quest) {
        selected_quest
    } else {
        cat_quests.first().copied().unwrap_or(0)
    };
    let max_scroll = cat_quests.len().saturating_sub(QUESTBOOK_VISIBLE_CARDS);
    let clamped_scroll = scroll.min(max_scroll);

    for (row, &q_idx) in cat_quests
        .iter()
        .skip(clamped_scroll)
        .take(QUESTBOOK_VISIBLE_CARDS)
        .enumerate()
    {
        let q = &QUESTS[q_idx];
        let cy = layout.list_y + (row as i32) * (layout.card_h + layout.card_gap);
        let rect = Rect::new(layout.list_x, cy, layout.list_w, layout.card_h);
        let is_selected = q_idx == active_quest_idx;
        let is_hover = rect.contains_mouse(mouse_pos);
        let unlocked = progress.is_unlocked(q_idx);
        let completed = progress.is_completed(q_idx);
        let claimed = progress.is_claimed(q_idx);

        let card_bg = if is_selected {
            pack_color(0.15, 0.20, 0.30)
        } else if is_hover {
            pack_color(0.12, 0.15, 0.22)
        } else {
            pack_color(0.09, 0.11, 0.16)
        };
        let border_col = if completed && !claimed {
            pack_color(0.96, 0.76, 0.22)
        } else if is_selected {
            cat.accent_color()
        } else {
            pack_color(0.20, 0.24, 0.32)
        };
        ui.bordered_box(rect, 1, card_bg, border_col);

        let icon_size = (layout.card_h - 14).clamp(24, 34);
        let icon_y = rect.y + (rect.h - icon_size) / 2;
        draw_item_icon(ui.buffer, ui.width, ui.height, q.icon, rect.x + 10, icon_y, icon_size);

        let text_x = rect.x + 18 + icon_size;
        let title_col = if !unlocked {
            pack_color(0.48, 0.52, 0.60)
        } else if claimed {
            pack_color(0.55, 0.88, 0.58)
        } else if completed {
            pack_color(0.98, 0.82, 0.32)
        } else {
            0xFFFFFFFF
        };
        ui.text(
            crate::core::i18n::t(&q.title_key()),
            text_x,
            rect.y + 8,
            1,
            1,
            title_col,
        );

        let (cur, target) = progress.quest_progress(q_idx);
        let status_label = if !unlocked {
            crate::core::i18n::t("questbook.status.locked").to_string()
        } else if claimed {
            crate::core::i18n::t("questbook.status.claimed").to_string()
        } else if completed {
            crate::core::i18n::t("questbook.status.ready").to_string()
        } else {
            format!("{cur}/{target}")
        };
        ui.text(&status_label, text_x, rect.y + 23, 1, 1, pack_color(0.72, 0.78, 0.86));

        let bar_rect = Rect::new(
            text_x,
            rect.bottom() - 9,
            rect.w - (text_x - rect.x) - 12,
            4,
        );
        let ratio = if target > 0 {
            cur as f32 / target as f32
        } else {
            0.0
        };
        let bar_col = if claimed {
            pack_color(0.35, 0.78, 0.42)
        } else if completed {
            pack_color(0.96, 0.76, 0.22)
        } else {
            cat.accent_color()
        };
        ui.progress_bar(bar_rect, ratio, pack_color(0.05, 0.06, 0.09), bar_col);
    }

    let track_h = (layout.card_h * QUESTBOOK_VISIBLE_CARDS as i32)
        + (layout.card_gap * (QUESTBOOK_VISIBLE_CARDS as i32 - 1));
    ui.scrollbar(
        Rect::new(layout.list_x + layout.list_w + 3, layout.list_y, 4, track_h),
        QUESTBOOK_VISIBLE_CARDS,
        cat_quests.len(),
        clamped_scroll,
        pack_color(0.10, 0.12, 0.18),
        cat.accent_color(),
    );

    ui.bordered_box(
        layout.detail,
        1,
        pack_color(0.08, 0.10, 0.15),
        pack_color(0.24, 0.28, 0.38),
    );

    if active_quest_idx < QUEST_COUNT {
        let q = &QUESTS[active_quest_idx];
        let unlocked = progress.is_unlocked(active_quest_idx);
        let completed = progress.is_completed(active_quest_idx);
        let claimed = progress.is_claimed(active_quest_idx);
        let (cur, target) = progress.quest_progress(active_quest_idx);

        draw_item_icon(
            ui.buffer,
            ui.width,
            ui.height,
            q.icon,
            layout.detail.x + 16,
            layout.detail.y + 16,
            40,
        );

        ui.text(
            crate::core::i18n::t(&q.title_key()),
            layout.detail.x + 68,
            layout.detail.y + 20,
            2,
            2,
            pack_color(0.98, 0.84, 0.36),
        );

        let prog_text = format!(
            "{}: {}/{}",
            crate::core::i18n::t("questbook.progress_label"),
            cur,
            target
        );
        ui.text(
            &prog_text,
            layout.detail.x + 68,
            layout.detail.y + 44,
            1,
            1,
            pack_color(0.75, 0.82, 0.92),
        );

        let mut cursor_y = layout.detail.y + 74;
        cursor_y = ui.text_wrapped(
            crate::core::i18n::t(&q.desc_key()),
            layout.detail.x + 16,
            cursor_y,
            layout.detail.w - 32,
            1,
            0xFFE6ECF5,
        );

        if let Some(prereq_idx) = q.prerequisite {
            if !unlocked {
                cursor_y += 8;
                let req_text = format!(
                    "{}: {}",
                    crate::core::i18n::t("questbook.requires_label"),
                    crate::core::i18n::t(&QUESTS[prereq_idx].title_key())
                );
                cursor_y = ui.text_wrapped(
                    &req_text,
                    layout.detail.x + 16,
                    cursor_y,
                    layout.detail.w - 32,
                    1,
                    pack_color(0.95, 0.45, 0.45),
                );
            }
        }

        let reward_y = (cursor_y + 16).min(layout.claim_rect.y - 76);
        ui.text(
            crate::core::i18n::t("questbook.rewards_label"),
            layout.detail.x + 16,
            reward_y,
            1,
            1,
            pack_color(0.96, 0.78, 0.32),
        );

        let mut rx = layout.detail.x + 16;
        let ry = reward_y + 20;
        for &(block, count) in q.rewards {
            ui.item_slot(
                Rect::new(rx, ry, 42, 42),
                block,
                Some(count),
                pack_color(0.12, 0.14, 0.20),
                pack_color(0.28, 0.33, 0.44),
            );
            rx += 50;
        }

        let claim_hovered = layout.claim_rect.contains_mouse(mouse_pos);
        let (btn_bg, btn_hover_bg, btn_label, btn_col) = if claimed {
            let c = pack_color(0.16, 0.34, 0.20);
            (
                c,
                c,
                crate::core::i18n::t("questbook.btn.claimed"),
                pack_color(0.65, 0.95, 0.68),
            )
        } else if completed {
            (
                pack_color(0.80, 0.56, 0.10),
                pack_color(0.92, 0.68, 0.14),
                crate::core::i18n::t("questbook.btn.claim"),
                0xFFFFFFFF,
            )
        } else if !unlocked {
            let c = pack_color(0.14, 0.15, 0.19);
            (
                c,
                c,
                crate::core::i18n::t("questbook.btn.locked"),
                pack_color(0.55, 0.58, 0.65),
            )
        } else {
            let c = pack_color(0.15, 0.18, 0.25);
            (
                c,
                c,
                crate::core::i18n::t("questbook.btn.in_progress"),
                pack_color(0.70, 0.76, 0.86),
            )
        };

        let border_c = pack_color(0.35, 0.40, 0.52);
        ui.button(
            layout.claim_rect,
            Button::new(btn_label)
                .scale(1)
                .hovered(claim_hovered)
                .text_color(btn_col)
                .variant(ButtonVariant::Flat {
                    bg: btn_bg,
                    hover_bg: btn_hover_bg,
                    border: border_c,
                    hover_border: border_c,
                })
                .flat_border(),
        );
    }
}

pub fn draw_quest_toast(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    quest_idx: usize,
    key_hint: &str,
) {
    if quest_idx >= QUEST_COUNT {
        return;
    }
    let q = &QUESTS[quest_idx];
    let hint = format!("{} [{}]", crate::core::i18n::t("questbook.toast_hint"), key_hint);
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.toast_card(Toast {
        heading: crate::core::i18n::t("questbook.toast_heading"),
        title: crate::core::i18n::t(&q.title_key()),
        subtitle: &hint,
        accent_color: pack_color(0.38, 0.85, 0.46),
        icon: Some(q.icon),
        y: 90,
    });
}
