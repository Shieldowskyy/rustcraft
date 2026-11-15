use crate::gameplay::achievements::{AchievementId, AchievementProgress};
use crate::gameplay::mob::MobKind;
use crate::ui::draw::{pack_color, set_px};
use crate::ui::widgets::{Grid, HStack, Rect, Toast, UiCanvas};
use crate::world::block::{BlockId, Face};

const LINKS: [(usize, usize); 10] = [
    (0, 1),
    (0, 2),
    (0, 3),
    (1, 4),
    (2, 5),
    (3, 6),
    (5, 7),
    (4, 8),
    (6, 8),
    (7, 8),
];

fn node_position(index: usize, width: usize) -> (i32, i32, i32, i32) {
    let card_width = ((width as i32 - 56).max(240) / 3 - 8).min(300);
    let gap = 12;
    let left = (width as i32 - (card_width * 3 + gap * 2)) / 2;
    let col = match index {
        1 | 4 => 0,
        2 | 5 => 1,
        3 | 6 => 2,
        _ => 1,
    };
    let row = match index {
        0 => 0,
        1..=3 => 1,
        4..=6 => 2,
        7 => 3,
        _ => 4,
    };
    let x = if index == 0 || index == 7 || index == 8 {
        (width as i32 - card_width) / 2
    } else {
        left + col * (card_width + gap)
    };
    (x, 96 + row * 74, card_width, 62)
}

fn draw_line(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    start: (i32, i32),
    end: (i32, i32),
    color: u32,
) {
    let (mut x, mut y) = start;
    let dx = (end.0 - x).abs();
    let sx = if x < end.0 { 1 } else { -1 };
    let dy = -(end.1 - y).abs();
    let sy = if y < end.1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        set_px(buffer, width, height, x, y, color);
        if x == end.0 && y == end.1 {
            break;
        }
        let doubled = 2 * error;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }
}

pub fn draw_achievement_tree(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    progress: &AchievementProgress,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.fill(ui.screen_rect(), pack_color(0.035, 0.045, 0.065));

    let heading = crate::core::i18n::t("achievement.tree_title");
    ui.text_centered_x(heading, ui.cx(), 18, 2, 2, pack_color(0.98, 0.82, 0.36));

    for (from, to) in LINKS {
        let parent = node_position(from, width);
        let child = node_position(to, width);
        draw_line(
            ui.buffer,
            ui.width,
            ui.height,
            (parent.0 + parent.2 / 2, parent.1 + parent.3),
            (child.0 + child.2 / 2, child.1),
            if progress.is_unlocked(AchievementId::ALL[from]) {
                pack_color(0.64, 0.49, 0.20)
            } else {
                pack_color(0.18, 0.21, 0.28)
            },
        );
    }

    for (index, id) in AchievementId::ALL.iter().copied().enumerate() {
        let (x, y, card_width, card_height) = node_position(index, width);
        let card_rect = Rect::new(x, y, card_width, card_height);
        let unlocked = progress.is_unlocked(id);
        let accent = if unlocked {
            pack_color(0.92, 0.72, 0.25)
        } else {
            pack_color(0.30, 0.36, 0.46)
        };
        let bg = if unlocked {
            pack_color(0.18, 0.15, 0.08)
        } else {
            pack_color(0.08, 0.10, 0.14)
        };
        ui.fill_alpha(card_rect, bg, 0.96);
        ui.border(card_rect, 2, accent);

        let title = crate::core::i18n::t(id.name_key());
        ui.text(
            title,
            x + 10,
            y + 7,
            1,
            1,
            if unlocked {
                pack_color(1.0, 0.88, 0.42)
            } else {
                pack_color(0.76, 0.80, 0.87)
            },
        );
        let description = crate::core::i18n::t(id.description_key());
        let max_chars = ((card_width - 20) / 6).max(1) as usize;
        let description: String = description.chars().take(max_chars).collect();
        ui.text_white(&description, x + 10, y + 21, 1, 1);

        let (current, goal) = progress.counts_for(id);
        let status = if unlocked {
            crate::core::i18n::t("achievement.unlocked")
        } else {
            crate::core::i18n::t("achievement.progress")
        };
        let line = crate::core::i18n::t_fmt(
            "achievement.progress_line",
            &[status, &current.min(goal).to_string(), &goal.to_string()],
        );
        let line: String = line.chars().take(max_chars).collect();
        ui.text(
            &line,
            x + 10,
            y + 37,
            1,
            1,
            if unlocked {
                pack_color(0.53, 0.92, 0.58)
            } else {
                pack_color(0.61, 0.68, 0.78)
            },
        );
    }

    let summary = crate::core::i18n::t_fmt(
        "achievement.tree_summary",
        &[
            &progress.unlocked_count().to_string(),
            &AchievementId::ALL.len().to_string(),
        ],
    );
    ui.text(&summary, 18, height as i32 - 20, 1, 1, pack_color(0.72, 0.78, 0.88));
    let back = crate::core::i18n::t("achievement.back_hint");
    ui.text_right(back, width as i32 - 18, height as i32 - 20, 1, 1, pack_color(0.72, 0.78, 0.88));
}

pub fn draw_achievement_toast(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    title: &str,
    description: &str,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.toast_card(Toast {
        heading: crate::core::i18n::t("achievement.toast_heading"),
        title,
        subtitle: description,
        accent_color: pack_color(0.96, 0.72, 0.22),
        icon: None,
        y: 18,
    });
}

const FIELD_GUIDE_TABS: [&str; 3] = [
    "achievement.tab_goals",
    "achievement.tab_creatures",
    "achievement.tab_blocks",
];

pub fn field_guide_tab_rects(width: usize) -> [(i32, i32, i32, i32); 3] {
    let gap = 8;
    let tab_width = ((width as i32 - 40 - gap * 2) / 3).min(210);
    HStack::centered_row(width, 48, tab_width, 24, 3, gap).array(tab_width)
}

pub fn field_guide_tab_at(width: usize, mouse_pos: Option<(f32, f32)>) -> Option<usize> {
    field_guide_tab_rects(width)
        .iter()
        .position(|&r| Rect::from_tuple(r).contains_mouse(mouse_pos))
}

fn draw_field_guide_tabs(buffer: &mut [u32], width: usize, tab: usize) {
    let height = height_from_buffer(buffer, width);
    let mut ui = UiCanvas::new(buffer, width, height);
    for (index, &rect_t) in field_guide_tab_rects(width).iter().enumerate() {
        let r = Rect::from(rect_t);
        let selected = index == tab;
        let background = if selected {
            pack_color(0.25, 0.30, 0.40)
        } else {
            pack_color(0.10, 0.12, 0.17)
        };
        let accent = if selected {
            pack_color(0.95, 0.75, 0.30)
        } else {
            pack_color(0.30, 0.36, 0.46)
        };
        ui.fill_alpha(r, background, 0.98);
        ui.fill(Rect::new(r.x, r.y, r.w, 2), accent);
        let label = crate::core::i18n::t(FIELD_GUIDE_TABS[index]);
        let max_chars = ((r.w - 12) / 6).max(1) as usize;
        let label: String = label.chars().take(max_chars).collect();
        let text_col = if selected {
            pack_color(1.0, 0.9, 0.56)
        } else {
            pack_color(0.78, 0.82, 0.9)
        };
        ui.text_in_rect(r, &label, 1, 1, text_col);
    }
    let hint = crate::core::i18n::t("achievement.tab_hint");
    ui.text_centered_x(hint, ui.cx(), 76, 1, 1, pack_color(0.58, 0.65, 0.76));
}

fn height_from_buffer(buffer: &[u32], width: usize) -> usize {
    if width == 0 {
        0
    } else {
        buffer.len() / width
    }
}

fn draw_catalog_heading(
    buffer: &mut [u32],
    width: usize,
    title_key: &str,
    count: usize,
    total: usize,
) {
    let height = height_from_buffer(buffer, width);
    let mut ui = UiCanvas::new(buffer, width, height);
    ui.fill(ui.screen_rect(), pack_color(0.035, 0.045, 0.065));

    let title = crate::core::i18n::t(title_key);
    ui.text_centered_x(title, ui.cx(), 18, 2, 2, pack_color(0.98, 0.82, 0.36));

    let summary = crate::core::i18n::t_fmt(
        "achievement.catalog_count",
        &[&count.to_string(), &total.to_string()],
    );
    ui.text(&summary, 14, height as i32 - 20, 1, 1, pack_color(0.72, 0.78, 0.88));
    let back = crate::core::i18n::t("achievement.back_hint");
    ui.text_right(back, width as i32 - 14, height as i32 - 20, 1, 1, pack_color(0.72, 0.78, 0.88));
}

fn draw_creature_catalog(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    progress: &AchievementProgress,
) {
    draw_catalog_heading(
        buffer,
        width,
        "creature.catalog_title",
        progress.discovered_mob_count(),
        MobKind::ALL.len(),
    );
    let gap = 10;
    let card_width = (width as i32 - 36 - gap) / 2;
    let grid = Grid::new(18, 88, 2, MobKind::ALL.len().div_ceil(2), card_width, 43, card_width + gap, 49);
    let mut ui = UiCanvas::new(buffer, width, height);

    for (index, mob) in MobKind::ALL.iter().copied().enumerate() {
        let discovered = progress.has_seen_mob(mob);
        let r = grid.slot(index);
        let hostile = mob.is_hostile();
        let accent = if !discovered {
            pack_color(0.27, 0.31, 0.38)
        } else if hostile {
            pack_color(0.88, 0.30, 0.25)
        } else if mob.is_aquatic() {
            pack_color(0.25, 0.63, 0.88)
        } else {
            pack_color(0.38, 0.78, 0.48)
        };
        ui.fill_alpha(r, pack_color(0.08, 0.10, 0.14), 0.96);
        ui.fill(Rect::new(r.x, r.y, 3, 43), accent);
        ui.fill(Rect::new(r.x + 10, r.y + 13, 14, 14), accent);

        let name = if discovered {
            crate::core::i18n::t(mob.catalog_name_key())
        } else {
            crate::core::i18n::t("creature.unknown")
        };
        ui.text(
            name,
            r.x + 33,
            r.y + 8,
            1,
            1,
            if discovered {
                pack_color(0.92, 0.94, 0.98)
            } else {
                pack_color(0.50, 0.54, 0.61)
            },
        );
        let detail = if !discovered {
            crate::core::i18n::t("creature.not_encountered")
        } else if hostile {
            crate::core::i18n::t("creature.hostile")
        } else if mob.is_aquatic() {
            crate::core::i18n::t("creature.aquatic")
        } else {
            crate::core::i18n::t("creature.peaceful")
        };
        ui.text(detail, r.x + 33, r.y + 23, 1, 1, accent);
    }
}

pub fn block_catalog_page_size(width: usize, height: usize) -> usize {
    let columns = if width >= 720 {
        4
    } else if width >= 500 {
        3
    } else {
        2
    };
    let rows = ((height as i32 - 132) / 32).max(1) as usize;
    columns * rows
}

pub fn block_catalog_entry_count() -> usize {
    (1..=u8::MAX)
        .map(BlockId::from_u8)
        .filter(|block| block.to_u8() != 0 && block.is_placeable())
        .count()
}

fn draw_block_catalog(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    progress: &AchievementProgress,
    scroll: usize,
) {
    let total = block_catalog_entry_count();
    draw_catalog_heading(
        buffer,
        width,
        "block.catalog_title",
        progress.discovered_block_count(),
        total,
    );
    let columns = if width >= 720 {
        4
    } else if width >= 500 {
        3
    } else {
        2
    };
    let page_size = block_catalog_page_size(width, height);
    let card_gap = 6;
    let card_width =
        ((width as i32 - 32 - card_gap * (columns as i32 - 1)) / columns as i32).max(48);
    let grid = Grid::new(16, 88, columns, page_size.div_ceil(columns), card_width, 27, card_width + card_gap, 32);
    let mut ui = UiCanvas::new(buffer, width, height);

    let mut visible_index = 0usize;
    let mut total_index = 0usize;
    for raw_id in 1..=u8::MAX {
        let block = BlockId::from_u8(raw_id);
        if block.to_u8() != raw_id || !block.is_placeable() {
            continue;
        }
        let item_index = total_index;
        total_index += 1;
        if item_index < scroll || item_index >= scroll + page_size {
            continue;
        }

        let discovered = progress.has_discovered_block(block);
        let r = grid.slot(visible_index);
        visible_index += 1;
        ui.fill_alpha(r, pack_color(0.08, 0.10, 0.14), 0.96);
        let color = if discovered {
            let [r_c, g_c, b_c] = block.face_color(Face::Top);
            pack_color(r_c, g_c, b_c)
        } else {
            pack_color(0.20, 0.22, 0.27)
        };
        ui.fill(Rect::new(r.x + 6, r.y + 6, 14, 14), color);
        let name = if discovered {
            block.display_name()
        } else {
            crate::core::i18n::t("block.unknown")
        };
        let max_chars = ((card_width - 30) / 6).max(1) as usize;
        let name: String = name.chars().take(max_chars).collect();
        ui.text(
            &name,
            r.x + 25,
            r.y + 10,
            1,
            1,
            if discovered {
                pack_color(0.88, 0.91, 0.96)
            } else {
                pack_color(0.48, 0.52, 0.60)
            },
        );
    }

    let page = scroll / page_size.max(1) + 1;
    let page_count = total.div_ceil(page_size.max(1));
    let page_text = crate::core::i18n::t_fmt(
        "block.catalog_page",
        &[&page.to_string(), &page_count.to_string()],
    );
    ui.text_centered_x(&page_text, ui.cx(), height as i32 - 20, 1, 1, pack_color(0.72, 0.78, 0.88));
}

pub fn draw_field_guide(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    progress: &AchievementProgress,
    tab: usize,
    scroll: usize,
) {
    match tab {
        1 => draw_creature_catalog(buffer, width, height, progress),
        2 => draw_block_catalog(buffer, width, height, progress, scroll),
        _ => draw_achievement_tree(buffer, width, height, progress),
    }
    draw_field_guide_tabs(buffer, width, tab.min(2));
}
