use crate::gameplay::player::{GameMode, Player};
use crate::graphics::font;
use crate::ui::draw::{
    draw_text_colored_shadowed, draw_text_light_colored_shadowed, draw_text_light_shadowed,
    draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color, set_px,
};

pub fn draw_crosshair(buffer: &mut [u32], width: usize, height: usize) {
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let size = 6i32;
    let color = pack_color(1.0, 1.0, 1.0);
    for i in -size..=size {
        set_px(buffer, width, height, cx + i, cy, color);
        set_px(buffer, width, height, cx, cy + i, color);
    }
}

pub fn draw_break_progress(buffer: &mut [u32], width: usize, height: usize, progress: f32) {
    if progress <= 0.0 {
        return;
    }
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let bar_w = 40i32;
    let bar_h = 3i32;
    let bar_x = cx - bar_w / 2;
    let bar_y = cy + 14;

    fill_rect_alpha(buffer, width, height, bar_x - 1, bar_y - 1, bar_w + 2, bar_h + 2, pack_color(0.0, 0.0, 0.0), 0.25);
    let t = progress.clamp(0.0, 1.0);
    let fill_w = (bar_w as f32 * t) as i32;
    if fill_w > 0 {
        fill_rect_alpha(buffer, width, height, bar_x, bar_y, fill_w, bar_h, pack_color(0.85, 0.85, 0.85), 0.5);
    }
}

pub fn draw_underwater_overlay(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.04, 0.28, 0.48), 0.34);
    let wave = ((time * 1.8).sin() * 3.0) as i32;
    for y in (18 + wave)..height as i32 {
        if y % 31 == 0 {
            fill_rect_alpha(buffer, width, height, 0, y, width as i32, 1, pack_color(0.25, 0.65, 0.80), 0.08);
        }
    }
}

pub fn draw_lava_overlay(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.92, 0.28, 0.02), 0.85);
    let wave = ((time * 2.5).sin() * 4.0) as i32;
    for y in (10 + wave)..height as i32 {
        if y % 17 == 0 {
            fill_rect_alpha(buffer, width, height, 0, y, width as i32, 2, pack_color(1.0, 0.55, 0.05), 0.18);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BubbleState {
    Full,
    Pop,
    Empty,
}

pub fn draw_bubble(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, state: BubbleState) {
    match state {
        BubbleState::Full => {
            let outline = pack_color(0.05, 0.22, 0.42);
            let highlight = pack_color(0.95, 0.98, 1.0);
            let cyan = pack_color(0.32, 0.78, 0.98);
            let shadow = pack_color(0.12, 0.48, 0.75);

            // 8x8 circular bubble outline
            fill_rect(buffer, width, height, x + 2, y + 0, 4, 1, outline);
            fill_rect(buffer, width, height, x + 1, y + 1, 1, 1, outline);
            fill_rect(buffer, width, height, x + 6, y + 1, 1, 1, outline);
            fill_rect(buffer, width, height, x + 0, y + 2, 1, 4, outline);
            fill_rect(buffer, width, height, x + 7, y + 2, 1, 4, outline);
            fill_rect(buffer, width, height, x + 1, y + 6, 1, 1, outline);
            fill_rect(buffer, width, height, x + 6, y + 6, 1, 1, outline);
            fill_rect(buffer, width, height, x + 2, y + 7, 4, 1, outline);

            // Bubble interior
            fill_rect(buffer, width, height, x + 2, y + 1, 4, 1, cyan);
            fill_rect(buffer, width, height, x + 1, y + 2, 6, 4, cyan);
            fill_rect(buffer, width, height, x + 2, y + 6, 4, 1, cyan);

            // Specular highlight
            set_px(buffer, width, height, x + 3, y + 1, highlight);
            set_px(buffer, width, height, x + 4, y + 1, highlight);
            set_px(buffer, width, height, x + 2, y + 2, highlight);
            set_px(buffer, width, height, x + 3, y + 2, highlight);
            set_px(buffer, width, height, x + 2, y + 3, highlight);

            // Spherical bottom-right shadow
            set_px(buffer, width, height, x + 6, y + 5, shadow);
            set_px(buffer, width, height, x + 5, y + 6, shadow);
        }
        BubbleState::Pop => {
            let drop = pack_color(0.40, 0.85, 1.0);
            let glint = pack_color(0.95, 0.98, 1.0);
            set_px(buffer, width, height, x + 1, y + 1, glint);
            set_px(buffer, width, height, x + 6, y + 1, drop);
            set_px(buffer, width, height, x + 0, y + 4, drop);
            set_px(buffer, width, height, x + 7, y + 4, glint);
            set_px(buffer, width, height, x + 2, y + 6, drop);
            set_px(buffer, width, height, x + 5, y + 6, glint);
            set_px(buffer, width, height, x + 3, y + 3, glint);
            set_px(buffer, width, height, x + 4, y + 4, drop);
        }
        BubbleState::Empty => {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HungerIconState {
    Full,
    Half,
    Empty,
}

pub fn draw_hunger_icon(buffer: &mut [u32], width: usize, height: usize, x: i32, y: i32, state: HungerIconState) {
    let outline = pack_color(0.24, 0.10, 0.04);
    let bone_white = pack_color(0.95, 0.92, 0.85);
    let bone_shadow = pack_color(0.68, 0.62, 0.55);
    let meat_hi = pack_color(0.96, 0.68, 0.32);
    let meat_base = pack_color(0.78, 0.42, 0.14);
    let meat_sh = pack_color(0.48, 0.22, 0.06);
    let empty_bg = pack_color(0.12, 0.10, 0.10);
    let empty_border = pack_color(0.22, 0.18, 0.18);

    match state {
        HungerIconState::Full => {
            // Bone handle at top-right
            set_px(buffer, width, height, x + 6, y + 0, bone_white);
            set_px(buffer, width, height, x + 7, y + 0, bone_white);
            set_px(buffer, width, height, x + 5, y + 1, outline);
            set_px(buffer, width, height, x + 6, y + 1, bone_white);
            set_px(buffer, width, height, x + 7, y + 1, bone_shadow);

            // Meat outline
            fill_rect(buffer, width, height, x + 2, y + 2, 3, 1, outline);
            set_px(buffer, width, height, x + 5, y + 2, outline);
            set_px(buffer, width, height, x + 1, y + 3, outline);
            set_px(buffer, width, height, x + 0, y + 4, outline);
            set_px(buffer, width, height, x + 0, y + 5, outline);
            set_px(buffer, width, height, x + 0, y + 6, outline);
            set_px(buffer, width, height, x + 1, y + 7, outline);
            fill_rect(buffer, width, height, x + 2, y + 7, 4, 1, outline);
            set_px(buffer, width, height, x + 6, y + 6, outline);
            set_px(buffer, width, height, x + 7, y + 5, outline);
            set_px(buffer, width, height, x + 6, y + 4, outline);
            set_px(buffer, width, height, x + 6, y + 3, outline);

            // Meat inner base
            fill_rect(buffer, width, height, x + 2, y + 3, 4, 1, meat_base);
            fill_rect(buffer, width, height, x + 1, y + 4, 5, 1, meat_base);
            fill_rect(buffer, width, height, x + 1, y + 5, 6, 1, meat_base);
            fill_rect(buffer, width, height, x + 1, y + 6, 5, 1, meat_sh);

            // Meat highlights
            set_px(buffer, width, height, x + 2, y + 3, meat_hi);
            set_px(buffer, width, height, x + 3, y + 3, meat_hi);
            set_px(buffer, width, height, x + 2, y + 4, meat_hi);
            set_px(buffer, width, height, x + 3, y + 4, meat_hi);

            // Meat bottom-right shadow
            set_px(buffer, width, height, x + 5, y + 5, meat_sh);
            set_px(buffer, width, height, x + 6, y + 5, meat_sh);
            set_px(buffer, width, height, x + 4, y + 6, meat_sh);
            set_px(buffer, width, height, x + 5, y + 6, meat_sh);
        }
        HungerIconState::Half => {
            // Bone handle at top-right
            set_px(buffer, width, height, x + 6, y + 0, bone_white);
            set_px(buffer, width, height, x + 7, y + 0, bone_white);
            set_px(buffer, width, height, x + 5, y + 1, outline);
            set_px(buffer, width, height, x + 6, y + 1, bone_white);
            set_px(buffer, width, height, x + 7, y + 1, bone_shadow);

            // Empty left side + filled right side
            fill_rect(buffer, width, height, x + 1, y + 3, 3, 4, empty_bg);
            fill_rect(buffer, width, height, x + 0, y + 4, 1, 3, empty_border);
            fill_rect(buffer, width, height, x + 1, y + 7, 3, 1, empty_border);
            fill_rect(buffer, width, height, x + 1, y + 2, 3, 1, empty_border);

            // Right half meat
            set_px(buffer, width, height, x + 4, y + 2, outline);
            set_px(buffer, width, height, x + 5, y + 2, outline);
            set_px(buffer, width, height, x + 4, y + 3, meat_hi);
            set_px(buffer, width, height, x + 5, y + 3, meat_base);
            set_px(buffer, width, height, x + 6, y + 3, outline);
            set_px(buffer, width, height, x + 4, y + 4, meat_hi);
            set_px(buffer, width, height, x + 5, y + 4, meat_base);
            set_px(buffer, width, height, x + 6, y + 4, outline);
            set_px(buffer, width, height, x + 4, y + 5, meat_base);
            set_px(buffer, width, height, x + 5, y + 5, meat_sh);
            set_px(buffer, width, height, x + 6, y + 5, outline);
            set_px(buffer, width, height, x + 4, y + 6, meat_sh);
            set_px(buffer, width, height, x + 5, y + 6, outline);
            fill_rect(buffer, width, height, x + 4, y + 7, 2, 1, outline);
        }
        HungerIconState::Empty => {
            // Empty dark shank outline
            set_px(buffer, width, height, x + 6, y + 0, empty_border);
            set_px(buffer, width, height, x + 7, y + 0, empty_border);
            set_px(buffer, width, height, x + 6, y + 1, empty_bg);
            set_px(buffer, width, height, x + 7, y + 1, empty_border);

            fill_rect(buffer, width, height, x + 2, y + 2, 4, 1, empty_border);
            set_px(buffer, width, height, x + 1, y + 3, empty_border);
            set_px(buffer, width, height, x + 0, y + 4, empty_border);
            set_px(buffer, width, height, x + 0, y + 5, empty_border);
            set_px(buffer, width, height, x + 0, y + 6, empty_border);
            set_px(buffer, width, height, x + 1, y + 7, empty_border);
            fill_rect(buffer, width, height, x + 2, y + 7, 4, 1, empty_border);
            set_px(buffer, width, height, x + 6, y + 6, empty_border);
            set_px(buffer, width, height, x + 7, y + 5, empty_border);
            set_px(buffer, width, height, x + 6, y + 4, empty_border);
            set_px(buffer, width, height, x + 6, y + 3, empty_border);

            fill_rect(buffer, width, height, x + 2, y + 3, 4, 4, empty_bg);
            fill_rect(buffer, width, height, x + 1, y + 4, 1, 3, empty_bg);
        }
    }
}

pub fn draw_hotbar(buffer: &mut [u32], width: usize, height: usize, player: &Player) {
    let slot_size = 28i32;
    let margin = 3i32;
    let total_slots = 10i32;
    let total_w = total_slots * (slot_size + margin) - margin;
    let start_x = (width as i32 - total_w) / 2;
    let y = height as i32 - slot_size - 12;

    fill_rect_alpha(buffer, width, height, start_x - 6, y - 6, total_w + 12, slot_size + 12, pack_color(0.0, 0.0, 0.0), 0.40);

    for i in 0..total_slots {
        let is_selected = i == player.selected_hotbar_slot as i32;
        let border_color = if is_selected {
            pack_color(1.0, 1.0, 0.2)
        } else {
            pack_color(0.20, 0.20, 0.20)
        };

        let x = start_x + i * (slot_size + margin);
        fill_rect(buffer, width, height, x - 2, y - 2, slot_size + 4, slot_size + 4, border_color);
        fill_rect(buffer, width, height, x, y, slot_size, slot_size, pack_color(0.3, 0.3, 0.3));

        if let Some(stack) = &player.inventory.hotbar[i as usize] {
            crate::ui::draw_item_icon(buffer, width, height, stack.block, x + 3, y + 3, slot_size - 6);

            if stack.count > 1 {
                let label = format!("{}", stack.count);
                draw_text_shadowed(buffer, width, height, &label, x + slot_size - 10, y + slot_size - 12, 1, 1);
            }
        }
    }

    if player.gamemode == GameMode::Survival {
        let health_start_x = start_x;
        let health_y = y - 16;
        let hp = player.health.ceil() as i32;
        let max_hp = player.max_health as i32;
        for i in 0..(max_hp / 2) {
            let heart_x = health_start_x + i * 10;
            let color = if (i * 2 + 1) <= hp {
                pack_color(1.0, 0.1, 0.1)
            } else if (i * 2) < hp {
                pack_color(1.0, 0.5, 0.5)
            } else {
                pack_color(0.0, 0.0, 0.0)
            };
            fill_rect(buffer, width, height, heart_x, health_y, 8, 8, pack_color(0.1, 0.1, 0.1));
            fill_rect(buffer, width, height, heart_x + 1, health_y + 1, 6, 6, color);
        }

        // Hunger bar - on right side above hotbar
        let hunger_start_x = start_x + total_w - 98;
        let hunger_pts = player.hunger.ceil() as i32;
        let is_starving = player.hunger <= 6.0;

        for i in 0..10 {
            // Hunger points deplete from right to left (Minecraft style)
            let icon_idx = i;
            let threshold = (icon_idx + 1) * 2;
            let state = if hunger_pts >= threshold {
                HungerIconState::Full
            } else if hunger_pts == threshold - 1 {
                HungerIconState::Half
            } else {
                HungerIconState::Empty
            };

            let wobble = if is_starving && player.hunger > 0.0 {
                if (i as i32 + (player.starvation_timer * 15.0) as i32) % 3 == 1 { -1 } else { 0 }
            } else {
                0
            };

            let hunger_x = hunger_start_x + i as i32 * 10;
            draw_hunger_icon(buffer, width, height, hunger_x, health_y + wobble, state);
        }

        // Air bubbles displayed above hunger meter while submerged underwater
        if player.head_underwater || player.air < player.max_air {
            let bubble_y = health_y - 10;
            let air_pts = player.air.ceil() as i32;
            for i in 0..10 {
                let bubble_x = hunger_start_x + i * 10;
                let state = if air_pts >= (i + 1) * 2 {
                    BubbleState::Full
                } else if air_pts == (i + 1) * 2 - 1 {
                    BubbleState::Pop
                } else {
                    BubbleState::Empty
                };
                draw_bubble(buffer, width, height, bubble_x, bubble_y, state);
            }
        }
    }
}

/// Draws a rasterized circular debug pie chart with colored slices.
pub fn draw_pie_chart(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    cx: i32,
    cy: i32,
    radius: i32,
    slices: &[crate::core::profiler::ProfilerSlice],
) {
    if slices.is_empty() || radius <= 0 {
        return;
    }

    let r_sq = radius * radius;
    let r_inner_sq = (radius - 1) * (radius - 1);
    let border_color = pack_color(0.9, 0.9, 0.95);

    let mut angles = Vec::with_capacity(slices.len() + 1);
    let mut current_angle = -std::f32::consts::FRAC_PI_2;
    angles.push(current_angle);

    for s in slices {
        current_angle += (s.percentage / 100.0) * std::f32::consts::TAU;
        angles.push(current_angle);
    }

    let min_x = (cx - radius).max(0);
    let max_x = (cx + radius).min(width as i32 - 1);
    let min_y = (cy - radius).max(0);
    let max_y = (cy + radius).min(height as i32 - 1);

    for y in min_y..=max_y {
        let dy = (y - cy) as f32;
        let dy_sq = (y - cy) * (y - cy);
        for x in min_x..=max_x {
            let dx = (x - cx) as f32;
            let dist_sq = (x - cx) * (x - cx) + dy_sq;
            if dist_sq <= r_sq {
                if dist_sq >= r_inner_sq {
                    set_px(buffer, width, height, x, y, border_color);
                } else {
                    let mut angle = dy.atan2(dx);
                    if angle < -std::f32::consts::FRAC_PI_2 {
                        angle += std::f32::consts::TAU;
                    }

                    let mut color = slices[0].color;
                    for (i, s) in slices.iter().enumerate() {
                        let a_start = angles[i];
                        let a_end = angles[i + 1];
                        if angle >= a_start && (angle <= a_end || i == slices.len() - 1) {
                            color = s.color;
                            break;
                        }
                    }
                    set_px(buffer, width, height, x, y, color);
                }
            }
        }
    }
}

/// Draws the comprehensive F3 debug overlay including left stats, right memory/renderer telemetry, and bottom-right pie chart.
pub fn draw_advanced_debug_overlay(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    left_lines: &[String],
    right_lines: &[String],
    profiler: &crate::core::profiler::FrameProfiler,
) {
    let scale = 1;
    let line_h = (font::GLYPH_H as i32) * scale + 3;
    let padding = 5;

    // 1. Left System & Coordinates Panel
    if !left_lines.is_empty() {
        let max_w = left_lines
            .iter()
            .map(|l| font::measure_text_light(l, scale, 1))
            .max()
            .unwrap_or(0);
        let panel_w = max_w + padding * 2;
        let panel_h = left_lines.len() as i32 * line_h + padding * 2 - 2;

        fill_rect_alpha(buffer, width, height, 4, 4, panel_w, panel_h, pack_color(0.0, 0.0, 0.0), 0.60);

        for (i, line) in left_lines.iter().enumerate() {
            let ty = 4 + padding + i as i32 * line_h;
            draw_text_light_shadowed(buffer, width, height, line, 4 + padding, ty, scale, 1);
        }
    }

    // 2. Right Memory Allocation & Detailed Renderer Pipeline Panel
    if !right_lines.is_empty() {
        let max_w = right_lines
            .iter()
            .map(|l| font::measure_text_light(l, scale, 1))
            .max()
            .unwrap_or(0);
        let panel_w = max_w + padding * 2;
        let panel_h = right_lines.len() as i32 * line_h + padding * 2 - 2;
        let rx = width as i32 - panel_w - 4;

        fill_rect_alpha(buffer, width, height, rx, 4, panel_w, panel_h, pack_color(0.0, 0.0, 0.0), 0.60);

        for (i, line) in right_lines.iter().enumerate() {
            let ty = 4 + padding + i as i32 * line_h;
            draw_text_light_shadowed(buffer, width, height, line, rx + padding, ty, scale, 1);
        }
    }

    // 3. Bottom-Right Minecraft-style Debug Pie Chart & Profiler Legend
    let slices = profiler.pie_slices();
    let pie_r = 34;
    let pie_cx = width as i32 - pie_r - 10;
    let pie_cy = height as i32 - pie_r - 12;

    let pie_panel_w = 310;
    let pie_panel_h = (slices.len() as i32 * 12) + 26;
    let pie_panel_x = width as i32 - pie_panel_w - 4;
    let pie_panel_y = height as i32 - pie_panel_h - 4;

    fill_rect_alpha(buffer, width, height, pie_panel_x, pie_panel_y, pie_panel_w, pie_panel_h, pack_color(0.0, 0.0, 0.0), 0.70);

    let prof_title = "DEBUG PROFILER (root.gameRenderer.level)";
    draw_text_light_colored_shadowed(buffer, width, height, prof_title, pie_panel_x + 6, pie_panel_y + 4, 1, 1, pack_color(0.95, 0.85, 0.35));

    draw_pie_chart(buffer, width, height, pie_cx, pie_cy, pie_r, &slices);

    let mut leg_y = pie_panel_y + 18;
    for s in &slices {
        fill_rect(buffer, width, height, pie_panel_x + 6, leg_y + 2, 6, 6, s.color);
        let leg_txt = format!("{:<11} {:>4.1}% {:>4.2}ms", s.name, s.percentage, s.time_ms);
        draw_text_light_shadowed(buffer, width, height, &leg_txt, pie_panel_x + 16, leg_y, 1, 1);
        leg_y += 12;
    }
}

pub fn draw_debug_panel(buffer: &mut [u32], width: usize, height: usize, lines: &[String]) {
    let scale = 1;
    let line_h = (font::GLYPH_H as i32) * scale + 3;
    let padding = 6;

    let max_w = lines
        .iter()
        .map(|l| font::measure_text_light(l, scale, 1))
        .max()
        .unwrap_or(0);

    let panel_w = max_w + padding * 2;
    let panel_h = lines.len() as i32 * line_h + padding * 2 - 4;

    fill_rect_alpha(buffer, width, height, 6, 6, panel_w, panel_h, pack_color(0.0, 0.0, 0.0), 0.5);

    for (i, line) in lines.iter().enumerate() {
        let ty = 6 + padding + i as i32 * line_h;
        draw_text_light_shadowed(buffer, width, height, line, 6 + padding, ty, scale, 1);
    }
}

pub fn draw_capture_prompt(buffer: &mut [u32], width: usize, height: usize, native_capture: bool) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.35);

    let line1 = crate::core::i18n::t("capture.prompt");
    let line2 = if native_capture {
        crate::core::i18n::t("capture.mouse_locked")
    } else {
        crate::core::i18n::t("capture.esc_pause")
    };

    let scale1 = 4;
    let scale2 = 2;
    let w1 = font::measure_text(line1, scale1, 2);
    let w2 = font::measure_text(line2, scale2, 1);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    draw_text_shadowed(buffer, width, height, line1, cx - w1 / 2, cy - 20, scale1, 2);
    draw_text_shadowed(buffer, width, height, line2, cx - w2 / 2, cy + 20, scale2, 1);
}

pub fn pause_menu_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 5] {
    let btn_w = 280;
    let btn_h = 34;
    let spacing = 8;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 - 95;

    [
        (cx - btn_w / 2, start_y, btn_w, btn_h),
        (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 3, btn_w, btn_h),
        (cx - btn_w / 2, start_y + (btn_h + spacing) * 4, btn_w, btn_h),
    ]
}

pub fn draw_pause_menu(buffer: &mut [u32], width: usize, height: usize, fly_mode: bool, hovered_idx: Option<usize>) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.72);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    // Title
    let title = crate::core::i18n::t("pause.title");
    let t_scale = 3;
    let t_w = font::measure_text(title, t_scale, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - t_w / 2, cy - 145, t_scale, 2, pack_color(0.95, 0.85, 0.35));

    #[cfg(target_os = "vita")]
    let b0 = format!("{} (START)", crate::core::i18n::t("pause.resume"));
    #[cfg(not(target_os = "vita"))]
    let b0 = format!("{} (ESC)", crate::core::i18n::t("pause.resume"));

    let b1 = crate::core::i18n::t("pause.settings").to_string();

    #[cfg(target_os = "vita")]
    let b2 = if fly_mode {
        format!("{} (R3)", crate::core::i18n::t("pause.fly_on"))
    } else {
        format!("{} (R3)", crate::core::i18n::t("pause.fly_off"))
    };
    #[cfg(not(target_os = "vita"))]
    let b2 = if fly_mode {
        format!("{} (F)", crate::core::i18n::t("pause.fly_on"))
    } else {
        format!("{} (F)", crate::core::i18n::t("pause.fly_off"))
    };

    #[cfg(target_os = "vita")]
    let b3 = format!("{} (SELECT)", crate::core::i18n::t("pause.main_menu"));
    #[cfg(not(target_os = "vita"))]
    let b3 = format!("{} (M)", crate::core::i18n::t("pause.main_menu"));

    #[cfg(target_os = "vita")]
    let b4 = crate::core::i18n::t("pause.quit").to_string();
    #[cfg(not(target_os = "vita"))]
    let b4 = format!("{} (Q)", crate::core::i18n::t("pause.quit"));

    let buttons = [b0, b1, b2, b3, b4];

    let rects = pause_menu_button_rects(width, height);

    for (i, (label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);

        let body_color = if is_hover {
            pack_color(0.32, 0.44, 0.68)
        } else {
            pack_color(0.20, 0.20, 0.24)
        };
        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);

        let top_left = if is_hover {
            pack_color(0.75, 0.85, 1.0)
        } else {
            pack_color(0.50, 0.50, 0.56)
        };
        let bottom_right = if is_hover {
            pack_color(0.15, 0.20, 0.35)
        } else {
            pack_color(0.10, 0.10, 0.12)
        };

        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    // Dynamic contextual tooltip for selected pause menu button
    let tooltip = match hovered_idx {
        Some(0) => "Resume gameplay and return to world (ESC / START / P)",
        Some(1) => "Adjust render distance, fog, 3D clouds, animations, and FOV",
        Some(2) => {
            if fly_mode {
                "Disable flight mode and restore standard gravity (F / R3)"
            } else {
                "Enable free flight mode without gravity (F / R3)"
            }
        }
        Some(3) => "Save current game world and return to save select menu (M)",
        Some(4) => "Save world progress and quit application (Q)",
        _ => "Navigate with mouse, arrow keys, or gamepad stick",
    };

    let tip_w = 460.min(width as i32 - 40);
    let tip_h = 24;
    let tip_x = cx - tip_w / 2;
    let tip_y = cy + 122;

    fill_rect_alpha(buffer, width, height, tip_x, tip_y, tip_w, tip_h, pack_color(0.08, 0.10, 0.16), 0.85);
    fill_rect(buffer, width, height, tip_x, tip_y, tip_w, 1, pack_color(0.35, 0.45, 0.65));
    fill_rect(buffer, width, height, tip_x, tip_y + tip_h - 1, tip_w, 1, pack_color(0.35, 0.45, 0.65));
    fill_rect(buffer, width, height, tip_x, tip_y, 1, tip_h, pack_color(0.35, 0.45, 0.65));
    fill_rect(buffer, width, height, tip_x + tip_w - 1, tip_y, 1, tip_h, pack_color(0.35, 0.45, 0.65));

    let tip_text_w = font::measure_text(tooltip, 1, 1);
    let tip_text_x = tip_x + (tip_w - tip_text_w) / 2;
    let tip_text_y = tip_y + (tip_h - font::GLYPH_H as i32) / 2;
    let tip_color = if hovered_idx.is_some() {
        pack_color(0.95, 0.90, 0.40)
    } else {
        pack_color(0.65, 0.70, 0.80)
    };
    draw_text_colored_shadowed(buffer, width, height, tooltip, tip_text_x, tip_text_y, 1, 1, tip_color);

    // Controls reference cards
    let card_w = 260;
    let card_h = 320;
    let card_y = cy - 150;

    if width >= 860 {
        // --- LEFT CARD: KEYBOARD & MOUSE ---
        let left_x = (cx - 140 - card_w - 20).max(12);
        fill_rect_alpha(buffer, width, height, left_x, card_y, card_w, card_h, pack_color(0.08, 0.09, 0.13), 0.82);
        fill_rect(buffer, width, height, left_x, card_y, card_w, 2, pack_color(0.40, 0.50, 0.70));
        fill_rect(buffer, width, height, left_x, card_y + card_h - 2, card_w, 2, pack_color(0.20, 0.25, 0.35));
        fill_rect(buffer, width, height, left_x, card_y, 2, card_h, pack_color(0.40, 0.50, 0.70));
        fill_rect(buffer, width, height, left_x + card_w - 2, card_y, 2, card_h, pack_color(0.20, 0.25, 0.35));

        let l_title = "[ KEYBOARD & MOUSE ]";
        let l_tw = font::measure_text(l_title, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, l_title, left_x + (card_w - l_tw) / 2, card_y + 10, 1, 1, pack_color(0.95, 0.85, 0.35));

        let kb_bindings = [
            ("W, A, S, D", "Movement"),
            ("SPACE", "Jump / Ascend"),
            ("SHIFT / C", "Sneak / Descend"),
            ("CTRL", "Sprint"),
            ("LMB", "Mine / Attack"),
            ("RMB", "Place / Consume"),
            ("E", "Inventory / Crafting"),
            ("Q", "Drop Item"),
            ("1-9 / SCROLL", "Hotbar Slot"),
            ("F", "Toggle Fly Mode"),
            ("G", "Creative / Survival"),
            ("F3", "Debug Diagnostics"),
            ("ESC / P", "Pause / Game Menu"),
        ];

        let mut row_y = card_y + 32;
        for (key, desc) in &kb_bindings {
            draw_text_colored_shadowed(buffer, width, height, key, left_x + 12, row_y, 1, 1, pack_color(0.45, 0.85, 1.0));
            let dw = font::measure_text(desc, 1, 1);
            draw_text_colored_shadowed(buffer, width, height, desc, left_x + card_w - 12 - dw, row_y, 1, 1, pack_color(0.85, 0.88, 0.92));
            row_y += 21;
        }

        // --- RIGHT CARD: CONTROLLER / GAMEPAD ---
        let right_x = (cx + 140 + 20).min(width as i32 - card_w - 12);
        fill_rect_alpha(buffer, width, height, right_x, card_y, card_w, card_h, pack_color(0.08, 0.09, 0.13), 0.82);
        fill_rect(buffer, width, height, right_x, card_y, card_w, 2, pack_color(0.40, 0.50, 0.70));
        fill_rect(buffer, width, height, right_x, card_y + card_h - 2, card_w, 2, pack_color(0.20, 0.25, 0.35));
        fill_rect(buffer, width, height, right_x, card_y, 2, card_h, pack_color(0.40, 0.50, 0.70));
        fill_rect(buffer, width, height, right_x + card_w - 2, card_y, 2, card_h, pack_color(0.20, 0.25, 0.35));

        let r_title = "[ GAMEPAD / PS VITA ]";
        let r_tw = font::measure_text(r_title, 1, 1);
        draw_text_colored_shadowed(buffer, width, height, r_title, right_x + (card_w - r_tw) / 2, card_y + 10, 1, 1, pack_color(0.95, 0.85, 0.35));

        let gp_bindings = [
            ("L-Stick", "Movement"),
            ("R-Stick", "Look / Aim"),
            ("[X] / A", "Jump / Ascend"),
            ("[O] / B", "Sneak / Descend"),
            ("L3 (Click)", "Sprint"),
            ("R2 / RT", "Mine / Attack"),
            ("L2 / LT", "Place / Consume"),
            ("[^] / Y", "Inventory / Crafting"),
            ("L1 / R1", "Hotbar Slot"),
            ("R3 (Click)", "Toggle Fly Mode"),
            ("SELECT", "Creative / Survival"),
            ("D-PAD UP", "Debug Overlay"),
            ("D-PAD LEFT", "Drop Item"),
        ];

        let mut row_y = card_y + 32;
        for (key, desc) in &gp_bindings {
            draw_text_colored_shadowed(buffer, width, height, key, right_x + 12, row_y, 1, 1, pack_color(0.45, 0.85, 1.0));
            let dw = font::measure_text(desc, 1, 1);
            draw_text_colored_shadowed(buffer, width, height, desc, right_x + card_w - 12 - dw, row_y, 1, 1, pack_color(0.85, 0.88, 0.92));
            row_y += 21;
        }
    } else {
        // Compact control hints bar for narrow screens
        let footers = [
            "KEYBOARD: WASD-Move | SPACE-Jump | SHIFT-Sneak | CTRL-Sprint | LMB-Mine | RMB-Place",
            "E-Inventory | Q-Drop | 1-9-Hotbar | F-Flight | G-Creative | F3-Debug | ESC-Pause",
            "GAMEPAD: L/R-Stick | [X]-Jump | [O]-Sneak | L3-Sprint | R2-Mine | L2-Place | [^]-Inventory",
        ];
        let mut fy = cy + 155;
        for &line in &footers {
            let lw = font::measure_text(line, 1, 1);
            draw_text_colored_shadowed(buffer, width, height, line, cx - lw / 2, fy, 1, 1, pack_color(0.75, 0.78, 0.85));
            fy += 15;
        }
    }
}

pub fn draw_damage_flash(buffer: &mut [u32], width: usize, height: usize, intensity: f32) {
    if intensity <= 0.001 {
        return;
    }
    let alpha = (intensity * 0.35).clamp(0.0, 0.55);
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.95, 0.05, 0.05), alpha);
}

pub fn draw_low_health_vignette(buffer: &mut [u32], width: usize, height: usize, time: f32) {
    let pulse = (time * 6.5).sin() * 0.5 + 0.5; // Fast heartbeat pulsation
    let base_alpha = 0.22 + 0.32 * pulse;
    let color = pack_color(0.85, 0.02, 0.02);

    let w = width as i32;
    let h = height as i32;
    let border_depth = 55;

    for layer in 0..border_depth {
        let t = 1.0 - (layer as f32 / border_depth as f32);
        let alpha = base_alpha * t * t * 0.75;
        let l_w = w - layer * 2;
        let l_h = h - layer * 2;
        if l_w <= 0 || l_h <= 0 { break; }

        fill_rect_alpha(buffer, width, height, layer, layer, l_w, 1, color, alpha);
        fill_rect_alpha(buffer, width, height, layer, h - 1 - layer, l_w, 1, color, alpha);
        fill_rect_alpha(buffer, width, height, layer, layer + 1, 1, l_h - 2, color, alpha);
        fill_rect_alpha(buffer, width, height, w - 1 - layer, layer + 1, 1, l_h - 2, color, alpha);
    }
}

pub fn death_screen_button_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 2] {
    let btn_w = 340;
    let btn_h = 42;
    let spacing = 16;
    let cx = width as i32 / 2;
    let start_y = height as i32 / 2 + 10;

    [
        (cx - btn_w / 2, start_y, btn_w, btn_h),
        (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
    ]
}

pub fn draw_death_screen(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    hovered_idx: Option<usize>,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.50, 0.05, 0.05), 0.65);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    let title = crate::core::i18n::t("death.title");
    let t_scale = 5;
    let tw = font::measure_text(title, t_scale, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - tw / 2, cy - 110, t_scale, 2, pack_color(1.0, 0.15, 0.15));

    let subtitle = crate::core::i18n::t("death.subtitle");
    let s_scale = 2;
    let sw = font::measure_text(subtitle, s_scale, 1);
    draw_text_colored_shadowed(buffer, width, height, subtitle, cx - sw / 2, cy - 40, s_scale, 1, pack_color(0.9, 0.85, 0.85));

    #[cfg(target_os = "vita")]
    let b0 = format!("{} [X]", crate::core::i18n::t("death.respawn"));
    #[cfg(not(target_os = "vita"))]
    let b0 = format!("{} (ENTER)", crate::core::i18n::t("death.respawn"));

    #[cfg(target_os = "vita")]
    let b1 = format!("{} [O]", crate::core::i18n::t("death.main_menu"));
    #[cfg(not(target_os = "vita"))]
    let b1 = format!("{} (ESC)", crate::core::i18n::t("death.main_menu"));

    let buttons = [b0, b1];

    let rects = death_screen_button_rects(width, height);

    for (i, (label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);

        let body_color = if is_hover {
            if i == 0 { pack_color(0.30, 0.50, 0.30) } else { pack_color(0.45, 0.28, 0.28) }
        } else {
            pack_color(0.22, 0.18, 0.18)
        };
        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);

        let top_left = if is_hover { pack_color(0.85, 0.95, 0.85) } else { pack_color(0.55, 0.50, 0.50) };
        let bottom_right = if is_hover { pack_color(0.20, 0.30, 0.20) } else { pack_color(0.12, 0.08, 0.08) };

        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_bubble_renders_pixels() {
        let width = 32;
        let height = 32;
        let mut buffer = vec![0u32; width * height];
        draw_bubble(&mut buffer, width, height, 4, 4, BubbleState::Full);
        let filled_count = buffer.iter().filter(|&&p| p != 0).count();
        assert!(filled_count > 0, "Full bubble should write non-zero pixels");

        let mut buffer_pop = vec![0u32; width * height];
        draw_bubble(&mut buffer_pop, width, height, 4, 4, BubbleState::Pop);
        let pop_count = buffer_pop.iter().filter(|&&p| p != 0).count();
        assert!(pop_count > 0, "Pop bubble should write non-zero pixels");
    }

    #[test]
    fn test_draw_hotbar_with_underwater_bubbles() {
        let width = 320;
        let height = 240;
        let mut buffer_surface = vec![0u32; width * height];
        let mut buffer_underwater = vec![0u32; width * height];

        let mut player = Player::new();
        player.gamemode = GameMode::Survival;
        player.head_underwater = false;
        player.air = 20.0;
        draw_hotbar(&mut buffer_surface, width, height, &player);

        player.head_underwater = true;
        player.air = 15.0;
        draw_hotbar(&mut buffer_underwater, width, height, &player);

        let count_surf = buffer_surface.iter().filter(|&&p| p != 0).count();
        let count_under = buffer_underwater.iter().filter(|&&p| p != 0).count();
        assert!(count_under > count_surf, "Underwater HUD should render bubble pixels above hearts");
    }

    #[test]
    fn test_draw_hunger_icon_renders_pixels() {
        let width = 32;
        let height = 32;
        let mut buffer_full = vec![0u32; width * height];
        draw_hunger_icon(&mut buffer_full, width, height, 4, 4, HungerIconState::Full);
        assert!(buffer_full.iter().filter(|&&p| p != 0).count() > 0);

        let mut buffer_half = vec![0u32; width * height];
        draw_hunger_icon(&mut buffer_half, width, height, 4, 4, HungerIconState::Half);
        assert!(buffer_half.iter().filter(|&&p| p != 0).count() > 0);

        let mut buffer_empty = vec![0u32; width * height];
        draw_hunger_icon(&mut buffer_empty, width, height, 4, 4, HungerIconState::Empty);
        assert!(buffer_empty.iter().filter(|&&p| p != 0).count() > 0);
    }
}

