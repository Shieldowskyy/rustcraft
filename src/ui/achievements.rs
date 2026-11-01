use crate::gameplay::achievements::{AchievementId, AchievementProgress};
use crate::gameplay::mob::MobKind;
use crate::graphics::font;
use crate::ui::draw::{
    draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color, set_px,
};
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
    fill_rect(
        buffer,
        width,
        height,
        0,
        0,
        width as i32,
        height as i32,
        pack_color(0.035, 0.045, 0.065),
    );
    let heading = crate::core::i18n::t("achievement.tree_title");
    let heading_width = font::measure_text(heading, 2, 2);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        heading,
        (width as i32 - heading_width) / 2,
        18,
        2,
        2,
        pack_color(0.98, 0.82, 0.36),
    );

    for (from, to) in LINKS {
        let parent = node_position(from, width);
        let child = node_position(to, width);
        draw_line(
            buffer,
            width,
            height,
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
        let unlocked = progress.is_unlocked(id);
        let accent = if unlocked {
            pack_color(0.92, 0.72, 0.25)
        } else {
            pack_color(0.30, 0.36, 0.46)
        };
        fill_rect_alpha(
            buffer,
            width,
            height,
            x,
            y,
            card_width,
            card_height,
            if unlocked {
                pack_color(0.18, 0.15, 0.08)
            } else {
                pack_color(0.08, 0.10, 0.14)
            },
            0.96,
        );
        fill_rect(buffer, width, height, x, y, card_width, 2, accent);
        fill_rect(buffer, width, height, x, y, 2, card_height, accent);
        fill_rect(
            buffer,
            width,
            height,
            x + card_width - 2,
            y,
            2,
            card_height,
            accent,
        );
        fill_rect(
            buffer,
            width,
            height,
            x,
            y + card_height - 2,
            card_width,
            2,
            accent,
        );

        let title = crate::core::i18n::t(id.name_key());
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
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
        draw_text_shadowed(buffer, width, height, &description, x + 10, y + 21, 1, 1);
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
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
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
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        &summary,
        18,
        height as i32 - 20,
        1,
        1,
        pack_color(0.72, 0.78, 0.88),
    );
    let back = crate::core::i18n::t("achievement.back_hint");
    let back_width = font::measure_text(back, 1, 1);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        back,
        width as i32 - back_width - 18,
        height as i32 - 20,
        1,
        1,
        pack_color(0.72, 0.78, 0.88),
    );
}

pub fn draw_achievement_toast(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    title: &str,
    description: &str,
) {
    let box_width = (width as i32 - 32).clamp(180, 360);
    let x = width as i32 - box_width - 16;
    let y = 18;
    fill_rect_alpha(
        buffer,
        width,
        height,
        x,
        y,
        box_width,
        62,
        pack_color(0.035, 0.045, 0.065),
        0.96,
    );
    fill_rect(
        buffer,
        width,
        height,
        x,
        y,
        box_width,
        3,
        pack_color(0.96, 0.72, 0.22),
    );
    fill_rect(
        buffer,
        width,
        height,
        x,
        y,
        3,
        62,
        pack_color(0.96, 0.72, 0.22),
    );
    let heading = crate::core::i18n::t("achievement.toast_heading");
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        heading,
        x + 12,
        y + 8,
        1,
        1,
        pack_color(0.98, 0.80, 0.35),
    );
    let max_chars = ((box_width - 24) / 6).max(1) as usize;
    let title: String = title.chars().take(max_chars).collect();
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        &title,
        x + 12,
        y + 23,
        1,
        1,
        pack_color(1.0, 1.0, 1.0),
    );
    let description: String = description.chars().take(max_chars).collect();
    draw_text_shadowed(buffer, width, height, &description, x + 12, y + 40, 1, 1);
}

const FIELD_GUIDE_TABS: [&str; 3] = [
    "achievement.tab_goals",
    "achievement.tab_creatures",
    "achievement.tab_blocks",
];

pub fn field_guide_tab_rects(width: usize) -> [(i32, i32, i32, i32); 3] {
    let gap = 8;
    let tab_width = ((width as i32 - 40 - gap * 2) / 3).min(210);
    let total_width = tab_width * 3 + gap * 2;
    let start_x = (width as i32 - total_width) / 2;
    [
        (start_x, 48, tab_width, 24),
        (start_x + tab_width + gap, 48, tab_width, 24),
        (start_x + (tab_width + gap) * 2, 48, tab_width, 24),
    ]
}

pub fn field_guide_tab_at(width: usize, mouse_pos: Option<(f32, f32)>) -> Option<usize> {
    let (mx, my) = mouse_pos?;
    field_guide_tab_rects(width)
        .iter()
        .position(|&(x, y, w, h)| {
            mx >= x as f32 && mx < (x + w) as f32 && my >= y as f32 && my < (y + h) as f32
        })
}

fn draw_field_guide_tabs(buffer: &mut [u32], width: usize, tab: usize) {
    let height = height_from_buffer(buffer, width);
    for (index, &(x, y, w, h)) in field_guide_tab_rects(width).iter().enumerate() {
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
        fill_rect_alpha(buffer, width, height, x, y, w, h, background, 0.98);
        fill_rect(buffer, width, height, x, y, w, 2, accent);
        let label = crate::core::i18n::t(FIELD_GUIDE_TABS[index]);
        let max_chars = ((w - 12) / 6).max(1) as usize;
        let label: String = label.chars().take(max_chars).collect();
        let text_width = font::measure_text(&label, 1, 1);
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &label,
            x + (w - text_width) / 2,
            y + 8,
            1,
            1,
            if selected {
                pack_color(1.0, 0.9, 0.56)
            } else {
                pack_color(0.78, 0.82, 0.9)
            },
        );
    }
    let hint = crate::core::i18n::t("achievement.tab_hint");
    let hint_width = font::measure_text(hint, 1, 1);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        hint,
        (width as i32 - hint_width) / 2,
        76,
        1,
        1,
        pack_color(0.58, 0.65, 0.76),
    );
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
    fill_rect(
        buffer,
        width,
        height,
        0,
        0,
        width as i32,
        height as i32,
        pack_color(0.035, 0.045, 0.065),
    );
    let title = crate::core::i18n::t(title_key);
    let title_width = font::measure_text(title, 2, 2);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        title,
        (width as i32 - title_width) / 2,
        18,
        2,
        2,
        pack_color(0.98, 0.82, 0.36),
    );
    let summary = crate::core::i18n::t_fmt(
        "achievement.catalog_count",
        &[&count.to_string(), &total.to_string()],
    );
    draw_text_colored_shadowed(
        buffer,
        width,
        height_from_buffer(buffer, width),
        &summary,
        14,
        height as i32 - 20,
        1,
        1,
        pack_color(0.72, 0.78, 0.88),
    );
    let back = crate::core::i18n::t("achievement.back_hint");
    let back_width = font::measure_text(back, 1, 1);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        back,
        width as i32 - back_width - 14,
        height as i32 - 20,
        1,
        1,
        pack_color(0.72, 0.78, 0.88),
    );
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
    for (index, mob) in MobKind::ALL.iter().copied().enumerate() {
        let discovered = progress.has_seen_mob(mob);
        let x = 18 + (index % 2) as i32 * (card_width + gap);
        let y = 88 + (index / 2) as i32 * 49;
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
        fill_rect_alpha(
            buffer,
            width,
            height,
            x,
            y,
            card_width,
            43,
            pack_color(0.08, 0.10, 0.14),
            0.96,
        );
        fill_rect(buffer, width, height, x, y, 3, 43, accent);
        fill_rect(buffer, width, height, x + 10, y + 13, 14, 14, accent);

        let name = if discovered {
            crate::core::i18n::t(mob.catalog_name_key())
        } else {
            crate::core::i18n::t("creature.unknown")
        };
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            name,
            x + 33,
            y + 8,
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
        draw_text_colored_shadowed(buffer, width, height, detail, x + 33, y + 23, 1, 1, accent);
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
        let x = 16 + (visible_index % columns) as i32 * (card_width + card_gap);
        let y = 88 + (visible_index / columns) as i32 * 32;
        visible_index += 1;
        fill_rect_alpha(
            buffer,
            width,
            height,
            x,
            y,
            card_width,
            27,
            pack_color(0.08, 0.10, 0.14),
            0.96,
        );
        let color = if discovered {
            let [r, g, b] = block.face_color(Face::Top);
            pack_color(r, g, b)
        } else {
            pack_color(0.20, 0.22, 0.27)
        };
        fill_rect(buffer, width, height, x + 6, y + 6, 14, 14, color);
        let name = if discovered {
            block.display_name()
        } else {
            crate::core::i18n::t("block.unknown")
        };
        let max_chars = ((card_width - 30) / 6).max(1) as usize;
        let name: String = name.chars().take(max_chars).collect();
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &name,
            x + 25,
            y + 10,
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
    let page_width = font::measure_text(&page_text, 1, 1);
    draw_text_colored_shadowed(
        buffer,
        width,
        height,
        &page_text,
        (width as i32 - page_width) / 2,
        height as i32 - 20,
        1,
        1,
        pack_color(0.72, 0.78, 0.88),
    );
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
