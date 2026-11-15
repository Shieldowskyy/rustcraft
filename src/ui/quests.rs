use crate::gameplay::quests::{quests_in_category, QuestCategory, QuestProgress, QUESTS, QUEST_COUNT};
use crate::graphics::font;
use crate::ui::draw::{
    draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color,
};
use crate::ui::inventory_ui::draw_item_icon;

#[derive(Clone, Copy)]
struct QuestbookLayout {
    panel_x: i32,
    panel_y: i32,
    panel_w: i32,
    panel_h: i32,
    tab_y: i32,
    tab_h: i32,
    list_x: i32,
    list_y: i32,
    list_w: i32,
    card_h: i32,
    card_gap: i32,
    detail_x: i32,
    detail_y: i32,
    detail_w: i32,
    detail_h: i32,
    claim_rect: (i32, i32, i32, i32),
    close_rect: (i32, i32, i32, i32),
}

fn compute_layout(width: usize, height: usize) -> QuestbookLayout {
    let w = width as i32;
    let h = height as i32;
    let panel_w = (w - 48).clamp(480, 960);
    let panel_h = (h - 48).clamp(360, 620);
    let panel_x = (w - panel_w) / 2;
    let panel_y = (h - panel_h) / 2;

    let tab_y = panel_y + 44;
    let tab_h = 28;
    let content_y = tab_y + tab_h + 12;
    let content_h = panel_h - (content_y - panel_y) - 18;

    let list_x = panel_x + 16;
    let list_w = (panel_w * 46 / 100).max(220);
    let card_gap = 8;
    let card_h = ((content_h - card_gap * 4) / 5).clamp(44, 68);

    let detail_x = list_x + list_w + 14;
    let detail_w = panel_x + panel_w - 16 - detail_x;
    let detail_y = content_y;
    let detail_h = content_h;

    let claim_w = (detail_w - 32).min(240);
    let claim_h = 34;
    let claim_x = detail_x + (detail_w - claim_w) / 2;
    let claim_y = detail_y + detail_h - claim_h - 14;

    let close_rect = (panel_x + panel_w - 34, panel_y + 10, 24, 24);

    QuestbookLayout {
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        tab_y,
        tab_h,
        list_x,
        list_y: content_y,
        list_w,
        card_h,
        card_gap,
        detail_x,
        detail_y,
        detail_w,
        detail_h,
        claim_rect: (claim_x, claim_y, claim_w, claim_h),
        close_rect,
    }
}

fn tab_rect(layout: &QuestbookLayout, tab_idx: usize) -> (i32, i32, i32, i32) {
    let count = QuestCategory::ALL.len() as i32;
    let gap = 6;
    let total_w = layout.panel_w - 32;
    let tab_w = (total_w - gap * (count - 1)) / count;
    let x = layout.panel_x + 16 + (tab_idx as i32) * (tab_w + gap);
    (x, layout.tab_y, tab_w, layout.tab_h)
}

fn point_in_rect(mouse_pos: Option<(f32, f32)>, r: (i32, i32, i32, i32)) -> bool {
    let Some((mx, my)) = mouse_pos else {
        return false;
    };
    let x = mx as i32;
    let y = my as i32;
    x >= r.0 && x < r.0 + r.2 && y >= r.1 && y < r.1 + r.3
}

pub fn questbook_tab_at(width: usize, height: usize, mouse_pos: Option<(f32, f32)>) -> Option<usize> {
    let layout = compute_layout(width, height);
    for idx in 0..QuestCategory::ALL.len() {
        if point_in_rect(mouse_pos, tab_rect(&layout, idx)) {
            return Some(idx);
        }
    }
    None
}

pub const QUESTBOOK_VISIBLE_CARDS: usize = 5;

pub fn questbook_card_at(
    width: usize,
    height: usize,
    tab: usize,
    scroll: usize,
    mouse_pos: Option<(f32, f32)>,
) -> Option<usize> {
    let layout = compute_layout(width, height);
    let cat = QuestCategory::ALL.get(tab).copied().unwrap_or(QuestCategory::Basics);
    let quests = quests_in_category(cat);
    for (row, &q_idx) in quests.iter().skip(scroll).take(QUESTBOOK_VISIBLE_CARDS).enumerate() {
        let cy = layout.list_y + (row as i32) * (layout.card_h + layout.card_gap);
        let rect = (layout.list_x, cy, layout.list_w, layout.card_h);
        if point_in_rect(mouse_pos, rect) {
            return Some(q_idx);
        }
    }
    None
}

pub fn questbook_claim_hit(width: usize, height: usize, mouse_pos: Option<(f32, f32)>) -> bool {
    let layout = compute_layout(width, height);
    point_in_rect(mouse_pos, layout.claim_rect)
}

pub fn questbook_close_hit(width: usize, height: usize, mouse_pos: Option<(f32, f32)>) -> bool {
    let layout = compute_layout(width, height);
    point_in_rect(mouse_pos, layout.close_rect)
}

fn draw_rect_border(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    thickness: i32,
    color: u32,
) {
    fill_rect(buffer, width, height, x, y, w, thickness, color);
    fill_rect(buffer, width, height, x, y + h - thickness, w, thickness, color);
    fill_rect(buffer, width, height, x, y, thickness, h, color);
    fill_rect(buffer, width, height, x + w - thickness, y, thickness, h, color);
}

fn draw_wrapped_text(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    text: &str,
    x: i32,
    mut y: i32,
    max_w: i32,
    scale: i32,
    color: u32,
) -> i32 {
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if font::measure_text(&candidate, scale, 1) > max_w && !line.is_empty() {
            draw_text_colored_shadowed(buffer, width, height, &line, x, y, scale, 1, color);
            y += 14 * scale.max(1);
            line = word.to_string();
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_text_colored_shadowed(buffer, width, height, &line, x, y, scale, 1, color);
        y += 14 * scale.max(1);
    }
    y
}

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
    // Dim background
    fill_rect_alpha(
        buffer,
        width,
        height,
        0,
        0,
        width as i32,
        height as i32,
        0xFF000000,
        0.62,
    );

    let layout = compute_layout(width, height);
    let cat = QuestCategory::ALL
        .get(active_tab)
        .copied()
        .unwrap_or(QuestCategory::Basics);

    // Main panel
    fill_rect_alpha(
        buffer,
        width,
        height,
        layout.panel_x,
        layout.panel_y,
        layout.panel_w,
        layout.panel_h,
        pack_color(0.06, 0.07, 0.10),
        0.96,
    );
    draw_rect_border(
        buffer,
        width,
        height,
        layout.panel_x,
        layout.panel_y,
        layout.panel_w,
        layout.panel_h,
        2,
        pack_color(0.28, 0.34, 0.45),
    );

    // Header
    let title = crate::core::i18n::t("questbook.title");
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        title,
        layout.panel_x + 18,
        layout.panel_y + 12,
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
    let sum_w = font::measure_text(&summary, 1, 1);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        &summary,
        layout.close_rect.0 - sum_w - 14,
        layout.panel_y + 17,
        1,
        1,
        pack_color(0.75, 0.80, 0.90),
    );

    // Close button
    let close_hovered = point_in_rect(mouse_pos, layout.close_rect);
    fill_rect(
        buffer,
        width,
        height,
        layout.close_rect.0,
        layout.close_rect.1,
        layout.close_rect.2,
        layout.close_rect.3,
        if close_hovered {
            pack_color(0.72, 0.22, 0.22)
        } else {
            pack_color(0.18, 0.20, 0.26)
        },
    );
    draw_text_shadowed(
        buffer,
        width,
        height,
        "X",
        layout.close_rect.0 + 8,
        layout.close_rect.1 + 6,
        1,
        1,
    );

    // Category tabs
    for (idx, &c) in QuestCategory::ALL.iter().enumerate() {
        let r = tab_rect(&layout, idx);
        let is_active = idx == active_tab;
        let is_hover = point_in_rect(mouse_pos, r);
        let bg = if is_active {
            pack_color(0.16, 0.20, 0.28)
        } else if is_hover {
            pack_color(0.12, 0.15, 0.22)
        } else {
            pack_color(0.09, 0.11, 0.16)
        };
        fill_rect(buffer, width, height, r.0, r.1, r.2, r.3, bg);
        if is_active {
            fill_rect(buffer, width, height, r.0, r.1 + r.3 - 3, r.2, 3, c.accent_color());
        }

        // Count claimable in this category
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
        let lw = font::measure_text(&label, 1, 1);
        let text_col = if claimable_in_cat > 0 {
            pack_color(0.98, 0.85, 0.35)
        } else if is_active {
            0xFFFFFFFF
        } else {
            pack_color(0.70, 0.75, 0.82)
        };
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &label,
            r.0 + (r.2 - lw) / 2,
            r.1 + 9,
            1,
            1,
            text_col,
        );
    }

    // Left list: quests in current category (scrollable window of 5)
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
        let rect = (layout.list_x, cy, layout.list_w, layout.card_h);
        let is_selected = q_idx == active_quest_idx;
        let is_hover = point_in_rect(mouse_pos, rect);
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
        fill_rect(buffer, width, height, rect.0, rect.1, rect.2, rect.3, card_bg);

        let border_col = if completed && !claimed {
            pack_color(0.96, 0.76, 0.22)
        } else if is_selected {
            cat.accent_color()
        } else {
            pack_color(0.20, 0.24, 0.32)
        };
        draw_rect_border(buffer, width, height, rect.0, rect.1, rect.2, rect.3, 1, border_col);

        // Icon
        let icon_size = (layout.card_h - 14).clamp(24, 34);
        let icon_y = rect.1 + (rect.3 - icon_size) / 2;
        draw_item_icon(buffer, width, height, q.icon, rect.0 + 10, icon_y, icon_size);

        // Title & progress bar
        let text_x = rect.0 + 18 + icon_size;
        let title_str = crate::core::i18n::t(&q.title_key());
        let title_col = if !unlocked {
            pack_color(0.48, 0.52, 0.60)
        } else if claimed {
            pack_color(0.55, 0.88, 0.58)
        } else if completed {
            pack_color(0.98, 0.82, 0.32)
        } else {
            0xFFFFFFFF
        };
        draw_text_colored_shadowed(buffer, width, height, title_str, text_x, rect.1 + 8, 1, 1, title_col);

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
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &status_label,
            text_x,
            rect.1 + 23,
            1,
            1,
            pack_color(0.72, 0.78, 0.86),
        );

        // Progress bar at bottom of card
        let bar_w = rect.2 - (text_x - rect.0) - 12;
        let bar_y = rect.1 + rect.3 - 9;
        fill_rect(buffer, width, height, text_x, bar_y, bar_w, 4, pack_color(0.05, 0.06, 0.09));
        let fill_w = if target > 0 {
            ((cur as f32 / target as f32).clamp(0.0, 1.0) * bar_w as f32) as i32
        } else {
            0
        };
        if fill_w > 0 {
            let bar_col = if claimed {
                pack_color(0.35, 0.78, 0.42)
            } else if completed {
                pack_color(0.96, 0.76, 0.22)
            } else {
                cat.accent_color()
            };
            fill_rect(buffer, width, height, text_x, bar_y, fill_w, 4, bar_col);
        }
    }

    // Subtle scrollbar when category has more than 5 quests
    if cat_quests.len() > QUESTBOOK_VISIBLE_CARDS {
        let track_x = layout.list_x + layout.list_w + 3;
        let track_y = layout.list_y;
        let track_h = (layout.card_h * QUESTBOOK_VISIBLE_CARDS as i32)
            + (layout.card_gap * (QUESTBOOK_VISIBLE_CARDS as i32 - 1));
        fill_rect(buffer, width, height, track_x, track_y, 4, track_h, pack_color(0.10, 0.12, 0.18));
        let thumb_h = ((track_h * QUESTBOOK_VISIBLE_CARDS as i32) / cat_quests.len() as i32).max(20);
        let thumb_y = if max_scroll > 0 {
            track_y + ((track_h - thumb_h) * clamped_scroll as i32) / max_scroll as i32
        } else {
            track_y
        };
        fill_rect(buffer, width, height, track_x, thumb_y, 4, thumb_h, cat.accent_color());
    }

    // Right panel: Quest Details
    fill_rect(
        buffer,
        width,
        height,
        layout.detail_x,
        layout.detail_y,
        layout.detail_w,
        layout.detail_h,
        pack_color(0.08, 0.10, 0.15),
    );
    draw_rect_border(
        buffer,
        width,
        height,
        layout.detail_x,
        layout.detail_y,
        layout.detail_w,
        layout.detail_h,
        1,
        pack_color(0.24, 0.28, 0.38),
    );

    if active_quest_idx < QUEST_COUNT {
        let q = &QUESTS[active_quest_idx];
        let unlocked = progress.is_unlocked(active_quest_idx);
        let completed = progress.is_completed(active_quest_idx);
        let claimed = progress.is_claimed(active_quest_idx);
        let (cur, target) = progress.quest_progress(active_quest_idx);

        draw_item_icon(
            buffer,
            width,
            height,
            q.icon,
            layout.detail_x + 16,
            layout.detail_y + 16,
            40,
        );

        let q_title = crate::core::i18n::t(&q.title_key());
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            q_title,
            layout.detail_x + 68,
            layout.detail_y + 20,
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
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &prog_text,
            layout.detail_x + 68,
            layout.detail_y + 44,
            1,
            1,
            pack_color(0.75, 0.82, 0.92),
        );

        let mut cursor_y = layout.detail_y + 74;
        let desc = crate::core::i18n::t(&q.desc_key());
        cursor_y = draw_wrapped_text(
            buffer,
            width,
            height,
            desc,
            layout.detail_x + 16,
            cursor_y,
            layout.detail_w - 32,
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
                cursor_y = draw_wrapped_text(
                    buffer,
                    width,
                    height,
                    &req_text,
                    layout.detail_x + 16,
                    cursor_y,
                    layout.detail_w - 32,
                    1,
                    pack_color(0.95, 0.45, 0.45),
                );
            }
        }

        // Rewards section
        let reward_y = (cursor_y + 16).min(layout.claim_rect.1 - 76);
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            crate::core::i18n::t("questbook.rewards_label"),
            layout.detail_x + 16,
            reward_y,
            1,
            1,
            pack_color(0.96, 0.78, 0.32),
        );

        let mut rx = layout.detail_x + 16;
        let ry = reward_y + 20;
        for &(block, count) in q.rewards {
            fill_rect(buffer, width, height, rx, ry, 42, 42, pack_color(0.12, 0.14, 0.20));
            draw_rect_border(buffer, width, height, rx, ry, 42, 42, 1, pack_color(0.28, 0.33, 0.44));
            draw_item_icon(buffer, width, height, block, rx + 5, ry + 5, 32);
            let cnt_str = format!("x{count}");
            draw_text_shadowed(buffer, width, height, &cnt_str, rx + 22, ry + 28, 1, 1);
            rx += 50;
        }

        // Claim / Status Button
        let cr = layout.claim_rect;
        let claim_hovered = point_in_rect(mouse_pos, cr);
        let (btn_bg, btn_label, btn_col) = if claimed {
            (
                pack_color(0.16, 0.34, 0.20),
                crate::core::i18n::t("questbook.btn.claimed"),
                pack_color(0.65, 0.95, 0.68),
            )
        } else if completed {
            (
                if claim_hovered {
                    pack_color(0.92, 0.68, 0.14)
                } else {
                    pack_color(0.80, 0.56, 0.10)
                },
                crate::core::i18n::t("questbook.btn.claim"),
                0xFFFFFFFF,
            )
        } else if !unlocked {
            (
                pack_color(0.14, 0.15, 0.19),
                crate::core::i18n::t("questbook.btn.locked"),
                pack_color(0.55, 0.58, 0.65),
            )
        } else {
            (
                pack_color(0.15, 0.18, 0.25),
                crate::core::i18n::t("questbook.btn.in_progress"),
                pack_color(0.70, 0.76, 0.86),
            )
        };
        fill_rect(buffer, width, height, cr.0, cr.1, cr.2, cr.3, btn_bg);
        draw_rect_border(
            buffer,
            width,
            height,
            cr.0,
            cr.1,
            cr.2,
            cr.3,
            1,
            pack_color(0.35, 0.40, 0.52),
        );
        let blw = font::measure_text(btn_label, 1, 1);
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            btn_label,
            cr.0 + (cr.2 - blw) / 2,
            cr.1 + 11,
            1,
            1,
            btn_col,
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
    let box_width = (width as i32 - 32).clamp(200, 360);
    let x = width as i32 - box_width - 16;
    let y = 90;
    fill_rect_alpha(
        buffer,
        width,
        height,
        x,
        y,
        box_width,
        62,
        pack_color(0.04, 0.06, 0.08),
        0.96,
    );
    fill_rect(buffer, width, height, x, y, box_width, 3, pack_color(0.38, 0.85, 0.46));
    fill_rect(buffer, width, height, x, y, 3, 62, pack_color(0.38, 0.85, 0.46));
    draw_item_icon(buffer, width, height, q.icon, x + 10, y + 15, 32);

    let heading = crate::core::i18n::t("questbook.toast_heading");
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        heading,
        x + 50,
        y + 9,
        1,
        1,
        pack_color(0.45, 0.92, 0.54),
    );
    let title = crate::core::i18n::t(&q.title_key());
    draw_text_shadowed(buffer, width, height, title, x + 50, y + 25, 1, 1);
    let hint = format!("{} [{}]", crate::core::i18n::t("questbook.toast_hint"), key_hint);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        &hint,
        x + 50,
        y + 41,
        1,
        1,
        pack_color(0.75, 0.82, 0.90),
    );
}
