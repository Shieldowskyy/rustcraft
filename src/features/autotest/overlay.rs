//! Renderowanie nakładki HUD autotestu (baner postępu, karta wyników POSITIVE/NEGATIVE, zminimalizowana odznaka).

use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Rect, UiCanvas};

use super::runner::{AutoTestRunner, AutoTestStage};

impl AutoTestRunner {
    pub fn render_overlay(&self, buffer: &mut [u32], width: usize, height: usize) {
        if !self.is_completed() {
            self.render_progress_banner(buffer, width, height);
        } else if !self.dismissed {
            self.render_completion_card(buffer, width, height);
        } else {
            self.render_minimized_badge(buffer, width, height);
        }
    }

    fn render_progress_banner(&self, buffer: &mut [u32], width: usize, height: usize) {
        let banner_w = 540.min(width as i32 - 20);
        let banner_h = 60;
        let x = (width as i32 - banner_w) / 2;
        let y = 14;
        let banner_rect = Rect::new(x, y, banner_w, banner_h);
        let mut ui = UiCanvas::new(buffer, width, height);

        ui.fill_alpha(banner_rect, pack_color(0.04, 0.08, 0.14), 0.88);
        ui.fill(Rect::new(x, y, banner_w, 2), pack_color(0.20, 0.75, 0.95));
        ui.fill(
            Rect::new(x, y + banner_h - 2, banner_w, 2),
            pack_color(0.20, 0.75, 0.95),
        );

        ui.text(
            "AUTOTEST BOT ACTIVE",
            x + 12,
            y + 8,
            2,
            1,
            pack_color(0.98, 0.88, 0.35),
        );

        let current_stage_idx = match self.stage {
            AutoTestStage::Init => 0,
            AutoTestStage::WorldGen => 1,
            AutoTestStage::Locomotion => 2,
            AutoTestStage::CameraAndPerspective => 3,
            AutoTestStage::FlightMode => 4,
            AutoTestStage::InventoryAndHotbar => 5,
            AutoTestStage::BlockPlacement => 6,
            AutoTestStage::BlockBreaking => 7,
            AutoTestStage::ItemPickup => 8,
            AutoTestStage::Crafting => 9,
            AutoTestStage::Containers => 10,
            AutoTestStage::MobCombat => 11,
            AutoTestStage::HungerAndFood => 12,
            AutoTestStage::DayNightCycle => 13,
            AutoTestStage::WorldPersistence => 14,
            AutoTestStage::GraphicsAndHud | AutoTestStage::Completed => 15,
        };

        let status_line = format!(
            "Stage [{}/15]: {:?} | Passed: {}/{}",
            current_stage_idx,
            self.stage,
            self.passed_count(),
            15
        );
        ui.text(
            &status_line,
            x + 12,
            y + 28,
            1,
            1,
            pack_color(0.70, 0.90, 1.0),
        );

        let bar_pad = 12;
        let progress = (current_stage_idx as f32 / 15.0).clamp(0.0, 1.0);
        ui.progress_bar(
            Rect::new(x + bar_pad, y + banner_h - 8, banner_w - bar_pad * 2, 4),
            progress,
            pack_color(0.12, 0.18, 0.28),
            pack_color(0.25, 0.90, 0.45),
        );
    }

    fn render_completion_card(&self, buffer: &mut [u32], width: usize, height: usize) {
        let mut ui = UiCanvas::new(buffer, width, height);
        ui.backdrop(None, pack_color(0.02, 0.04, 0.08), 0.88);

        let card_w = 860.min(width as i32 - 24);
        let card_h = 490.min(height as i32 - 20);
        let card_rect = Rect::centered_in(width, height, card_w, card_h);
        let cx = ui.cx();
        let x = card_rect.x;
        let y = card_rect.y;

        ui.fill_alpha(card_rect, pack_color(0.06, 0.09, 0.16), 0.98);
        ui.border(card_rect, 2, pack_color(0.30, 0.55, 0.85));

        ui.text(
            "RUSTCRAFT AUTOTEST SUITE",
            cx - 180,
            y + 15,
            3,
            2,
            pack_color(0.98, 0.88, 0.45),
        );
        ui.text(
            "Automated Diagnostic Engine & Compliance Verification",
            cx - 165,
            y + 42,
            1,
            1,
            pack_color(0.65, 0.78, 0.92),
        );

        let is_pos = self.is_positive();
        let passed = self.passed_count();
        let total = self.total_count();

        let banner_box_w = card_w - 60;
        let banner_box_h = 38;
        let banner_x = x + 30;
        let banner_y = y + 58;
        let banner_rect = Rect::new(banner_x, banner_y, banner_box_w, banner_box_h);

        let (banner_bg, banner_edge, banner_text, text_x) = if is_pos {
            (
                pack_color(0.08, 0.35, 0.16),
                pack_color(0.30, 0.95, 0.40),
                format!("RESULT: POSITIVE  -  ALL {}/{} TESTS PASSED", passed, total),
                cx - 195,
            )
        } else {
            (
                pack_color(0.45, 0.10, 0.10),
                pack_color(0.95, 0.30, 0.30),
                format!("RESULT: NEGATIVE  -  {}/{} TESTS PASSED", passed, total),
                cx - 180,
            )
        };
        ui.fill(banner_rect, banner_bg);
        ui.fill(Rect::new(banner_x, banner_y, banner_box_w, 2), banner_edge);
        ui.fill(
            Rect::new(banner_x, banner_y + banner_box_h - 2, banner_box_w, 2),
            banner_edge,
        );
        ui.text_white(&banner_text, text_x, banner_y + 11, 2, 2);

        let col1_x = x + 30;
        let col2_x = x + card_w / 2 + 10;
        let row_start_y = y + 108;
        let row_step_y = 23;

        for (idx, r) in self.records.iter().enumerate() {
            let (rx, ry) = if idx < 8 {
                (col1_x, row_start_y + idx as i32 * row_step_y)
            } else {
                (col2_x, row_start_y + (idx - 8) as i32 * row_step_y)
            };

            if r.passed {
                ui.text("[PASS]", rx, ry, 1, 1, pack_color(0.25, 0.95, 0.40));
            } else {
                ui.text("[FAIL]", rx, ry, 1, 1, pack_color(0.95, 0.25, 0.25));
            }

            let label = format!("{:02}. {}", r.id, r.name);
            ui.text(&label, rx + 42, ry, 1, 1, pack_color(0.95, 0.95, 0.95));

            let detail = format!("{}ms", r.duration_ms);
            ui.text(&detail, rx + 325, ry, 1, 1, pack_color(0.60, 0.75, 0.85));
        }

        let footer_y = y + card_h - 38;
        ui.fill(
            Rect::new(x + 20, footer_y - 8, card_w - 40, 1),
            pack_color(0.20, 0.35, 0.50),
        );

        let summary = format!(
            "Score: {}/{} | Status: {} | Total Time: {:.2}s",
            passed,
            total,
            if is_pos { "POSITIVE" } else { "NEGATIVE" },
            self.total_time
        );
        ui.text(
            &summary,
            x + 30,
            footer_y,
            1,
            1,
            pack_color(0.75, 0.85, 0.95),
        );

        let prompt = "Press [ESC], [ENTER] or Controller [A] / [START] to dismiss";
        ui.text(
            prompt,
            x + card_w - 380,
            footer_y,
            1,
            1,
            pack_color(0.95, 0.85, 0.45),
        );
    }

    fn render_minimized_badge(&self, buffer: &mut [u32], width: usize, height: usize) {
        let is_pos = self.is_positive();
        let passed = self.passed_count();
        let total = self.total_count();

        let badge_w = 340;
        let badge_h = 24;
        let x = width as i32 - badge_w - 12;
        let y = 10;
        let mut ui = UiCanvas::new(buffer, width, height);

        ui.fill_alpha(
            Rect::new(x, y, badge_w, badge_h),
            pack_color(0.04, 0.08, 0.12),
            0.85,
        );

        let border_col = if is_pos {
            pack_color(0.25, 0.95, 0.40)
        } else {
            pack_color(0.95, 0.25, 0.25)
        };
        ui.fill(Rect::new(x, y, badge_w, 1), border_col);
        ui.fill(Rect::new(x, y + badge_h - 1, badge_w, 1), border_col);

        let text = format!(
            "AUTOTEST: {} ({}/{}) [ESC: Report]",
            if is_pos { "POSITIVE" } else { "NEGATIVE" },
            passed,
            total
        );
        ui.text(&text, x + 8, y + 7, 1, 1, border_col);
    }
}
