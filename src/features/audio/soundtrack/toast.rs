//! Powiadomienie "Teraz odtwarzane" (`MusicToast`) z paskiem postępu odtwarzania w prawym dolnym rogu.

use crate::core::i18n::t;
use crate::features::ui::draw::pack_color;
use crate::features::ui::widgets::{Rect, UiCanvas};

use super::types::SoundtrackCategory;

#[derive(Clone, Debug)]
pub struct MusicToast {
    pub title: String,
    pub artist: String,
    pub category_name: String,
    pub accent_color: u32,
    pub timer: f32,
    pub duration: f32,
    pub visible: bool,
}

impl Default for MusicToast {
    fn default() -> Self {
        Self {
            title: String::new(),
            artist: String::new(),
            category_name: String::new(),
            accent_color: pack_color(0.4, 0.7, 1.0),
            timer: 0.0,
            duration: 5.0,
            visible: false,
        }
    }
}

impl MusicToast {
    pub fn show(&mut self, title: &str, artist: &str, category: SoundtrackCategory) {
        self.title = title.to_string();
        self.artist = artist.to_string();
        self.category_name = category.display_name().to_string();
        self.accent_color = category.accent_color();
        self.duration = 7.0;
        self.timer = 7.0;
        self.visible = true;
    }

    pub fn update(&mut self, dt: f32) {
        if self.visible {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.visible = false;
            }
        }
    }

    pub fn draw(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        force_show: bool,
        progress: Option<f32>,
        duration_sec: Option<f32>,
    ) {
        if (!force_show && (!self.visible || self.timer <= 0.0)) || width < 300 || height < 100 {
            return;
        }

        let toast_w = 280i32;
        let toast_h = 50i32;
        let margin = 14i32;

        let base_x = width as i32 - toast_w - margin;
        let base_y = height as i32 - toast_h - margin;

        let alpha = if force_show {
            1.0
        } else {
            let elapsed = self.duration - self.timer;
            let linear = if elapsed < 1.0 {
                (elapsed / 1.0).clamp(0.0, 1.0)
            } else if self.timer < 1.4 {
                (self.timer / 1.4).clamp(0.0, 1.0)
            } else {
                1.0
            };
            linear * linear * (3.0 - 2.0 * linear)
        };

        let slide_offset = if force_show {
            0
        } else {
            ((1.0 - alpha) * 25.0) as i32
        };
        let bx = base_x + slide_offset;
        let by = base_y;

        let rx0 = bx.max(0) as usize;
        let rx1 = ((bx + toast_w).max(0) as usize).min(width);
        let ry0 = by.max(0) as usize;
        let ry1 = ((by + toast_h).max(0) as usize).min(height);
        let mut saved: Vec<u32> = Vec::new();
        if alpha < 0.999 && rx1 > rx0 && ry1 > ry0 {
            for y in ry0..ry1 {
                saved.extend_from_slice(&buffer[y * width + rx0..y * width + rx1]);
            }
        }

        let mut ui = UiCanvas::new(buffer, width, height);
        let toast_rect = Rect::new(bx, by, toast_w, toast_h);
        ui.fill_alpha(toast_rect, pack_color(0.08, 0.10, 0.14), 0.90);
        ui.bevel_border(
            toast_rect,
            1,
            pack_color(0.30, 0.38, 0.50),
            pack_color(0.04, 0.05, 0.08),
        );

        ui.fill(Rect::new(bx + 1, by + 1, 3, toast_h - 2), self.accent_color);
        let content_x = bx + 12;

        let header_str = t("toast.now_playing");
        let header_text = if header_str.is_empty() || header_str.starts_with("toast.") {
            "NOW PLAYING"
        } else {
            header_str
        };
        let header_display = format!("> {}", header_text);
        ui.text(&header_display, content_x, by + 4, 1, 1, self.accent_color);
        ui.text_white(&self.title, content_x, by + 16, 1, 1);

        let sub_color = pack_color(0.68, 0.74, 0.82);
        let sub_text = if !self.artist.is_empty() {
            format!("{} * {}", self.artist, self.category_name)
        } else {
            self.category_name.clone()
        };
        ui.text(&sub_text, content_x, by + 27, 1, 1, sub_color);

        if let (Some(p), Some(dur)) = (progress, duration_sec) {
            if dur > 0.0 {
                let cur_s = (p * dur) as u32;
                let total_s = dur as u32;
                let time_str = format!(
                    "{}:{:02}/{}:{:02}",
                    cur_s / 60,
                    cur_s % 60,
                    total_s / 60,
                    total_s % 60
                );
                ui.text_right(
                    &time_str,
                    bx + toast_w - 12,
                    by + 27,
                    1,
                    1,
                    pack_color(0.60, 0.70, 0.82),
                );
            }
        }

        let bar_rect = Rect::new(content_x, by + 41, toast_w - 24, 3);
        let prog_val = progress.unwrap_or(0.0).clamp(0.0, 1.0);
        ui.progress_bar(bar_rect, prog_val, pack_color(0.18, 0.22, 0.28), self.accent_color);
        let fill_w = ((bar_rect.w as f32) * prog_val).round() as i32;
        if fill_w > 0 && fill_w < bar_rect.w {
            ui.fill(
                Rect::new(bar_rect.x + fill_w - 1, bar_rect.y, 2, bar_rect.h),
                pack_color(1.0, 1.0, 1.0),
            );
        }

        if !saved.is_empty() {
            let rw = rx1 - rx0;
            for (row, y) in (ry0..ry1).enumerate() {
                for i in 0..rw {
                    let old = saved[row * rw + i];
                    let idx = y * width + rx0 + i;
                    let new = buffer[idx];
                    let mix = |s: u32| -> u32 {
                        let o = ((old >> s) & 0xFF) as f32;
                        let n = ((new >> s) & 0xFF) as f32;
                        (o + (n - o) * alpha).round().clamp(0.0, 255.0) as u32
                    };
                    buffer[idx] = (mix(16) << 16) | (mix(8) << 8) | mix(0);
                }
            }
        }
    }
}
