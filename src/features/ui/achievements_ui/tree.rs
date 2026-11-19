//! Renderowanie drzewa osiągnięć gracza oraz powiadomień (toastów) o zdobytym osiągnięciu.

use crate::features::player::{AchievementId, AchievementProgress};
use crate::features::ui::draw::{pack_color, set_px};
use crate::features::ui::widgets::{Rect, Toast, UiCanvas};

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
