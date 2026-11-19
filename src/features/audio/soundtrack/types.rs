//! Typy kategorii muzyki (`SoundtrackCategory`), metadane utworu (`SoundtrackTrack`) i asynchroniczny loader (`AsyncTrackLoader`).

use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::features::audio::sound::SoundSample;
use crate::features::ui::draw::pack_color;

pub struct AsyncTrackLoader {
    pub track_idx: usize,
    pub receiver: std::sync::mpsc::Receiver<Result<Arc<SoundSample>, String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundtrackCategory {
    Creative,
    Survival,
    Menu,
    Uncategorized,
}

impl SoundtrackCategory {
    pub fn parse(s: &str) -> Self {
        match s.to_ascii_lowercase().trim() {
            "creative" => SoundtrackCategory::Creative,
            "survival" => SoundtrackCategory::Survival,
            "menu" | "main_menu" | "ui" => SoundtrackCategory::Menu,
            _ => SoundtrackCategory::Uncategorized,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SoundtrackCategory::Creative => "creative",
            SoundtrackCategory::Survival => "survival",
            SoundtrackCategory::Menu => "menu",
            SoundtrackCategory::Uncategorized => "uncategorized",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SoundtrackCategory::Creative => "Creative",
            SoundtrackCategory::Survival => "Survival",
            SoundtrackCategory::Menu => "Menu",
            SoundtrackCategory::Uncategorized => "Atmosphere",
        }
    }

    pub fn accent_color(&self) -> u32 {
        match self {
            SoundtrackCategory::Creative => pack_color(0.20, 0.85, 0.65),
            SoundtrackCategory::Survival => pack_color(0.95, 0.70, 0.20),
            SoundtrackCategory::Menu => pack_color(0.40, 0.65, 0.95),
            SoundtrackCategory::Uncategorized => pack_color(0.75, 0.50, 0.95),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SoundtrackTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub file_path: PathBuf,
    pub categories: Vec<SoundtrackCategory>,
    pub sample: Option<Arc<SoundSample>>,
    pub embedded_data: Option<&'static [u8]>,
}

impl SoundtrackTrack {
    pub fn matches_category(&self, current: SoundtrackCategory) -> bool {
        self.categories.contains(&current)
            || self.categories.contains(&SoundtrackCategory::Uncategorized)
    }

    pub fn load_sample(&mut self) -> Result<Arc<SoundSample>, String> {
        if let Some(s) = &self.sample {
            return Ok(Arc::clone(s));
        }

        let bytes: Cow<'static, [u8]> = if self.file_path.exists() {
            let data = fs::read(&self.file_path).map_err(|e| {
                format!(
                    "Could not read audio file '{}': {}",
                    self.file_path.display(),
                    e
                )
            })?;
            Cow::Owned(data)
        } else {
            let embedded = self.embedded_data.ok_or_else(|| {
                format!(
                    "Could not read audio file '{}': file not found",
                    self.file_path.display()
                )
            })?;
            Cow::Borrowed(embedded)
        };

        let sample = SoundSample::from_file_bytes(&bytes).map_err(|e| {
            format!(
                "Failed to decode audio file '{}': {}",
                self.file_path.display(),
                e
            )
        })?;

        let arc_sample = Arc::new(sample);
        self.sample = Some(Arc::clone(&arc_sample));
        Ok(arc_sample)
    }
}
