//! Cross-platform soundtrack management, manifest configuration, and now-playing toasts.
//!
//! Provides categorized soundtrack playback (.ogg Vorbis support on all platforms),
//! configurable delay intervals, context-aware category filtering (creative, survival,
//! menu, uncategorized), and real-time toast notifications in the bottom-right corner.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::audio::sound::SoundSample;
use crate::audio::AudioEngine;
use crate::core::i18n::t;
use crate::engine::modding::parse_json_value_root;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};

/// Soundtrack playback categorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundtrackCategory {
    /// Tracks intended for Creative gamemode.
    Creative,
    /// Tracks intended for Survival gamemode.
    Survival,
    /// Tracks intended for Main Menu and UI screens.
    Menu,
    /// Universal tracks that can play randomly at any time.
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
            SoundtrackCategory::Creative => pack_color(0.20, 0.85, 0.65), // Emerald
            SoundtrackCategory::Survival => pack_color(0.95, 0.70, 0.20), // Amber Gold
            SoundtrackCategory::Menu => pack_color(0.40, 0.65, 0.95),     // Azure Blue
            SoundtrackCategory::Uncategorized => pack_color(0.75, 0.50, 0.95), // Amethyst
        }
    }
}

/// Metadata and cached audio sample for a discrete soundtrack file.
#[derive(Clone, Debug)]
pub struct SoundtrackTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub file_path: PathBuf,
    pub categories: Vec<SoundtrackCategory>,
    pub sample: Option<Arc<SoundSample>>,
}

impl SoundtrackTrack {
    /// Determines whether this track is eligible to play under the given game context.
    pub fn matches_category(&self, current: SoundtrackCategory) -> bool {
        self.categories.contains(&current)
            || self.categories.contains(&SoundtrackCategory::Uncategorized)
    }

    /// Loads and decodes the audio file (.ogg Vorbis or .wav) into memory.
    pub fn load_sample(&mut self) -> Result<Arc<SoundSample>, String> {
        if let Some(s) = &self.sample {
            return Ok(Arc::clone(s));
        }

        let bytes = fs::read(&self.file_path)
            .map_err(|e| format!("Could not read audio file '{}': {}", self.file_path.display(), e))?;

        let sample = SoundSample::from_file_bytes(&bytes)
            .map_err(|e| format!("Failed to decode audio file '{}': {}", self.file_path.display(), e))?;

        let arc_sample = Arc::new(sample);
        self.sample = Some(Arc::clone(&arc_sample));
        Ok(arc_sample)
    }
}

/// A toast notification displayed in the bottom-right corner when a new track starts.
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
        self.duration = 5.0;
        self.timer = 5.0;
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

    /// Renders the now-playing toast notification in the bottom-right corner with smooth fade and slide.
    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize) {
        if !self.visible || self.timer <= 0.0 || width < 300 || height < 100 {
            return;
        }

        let toast_w = 270i32;
        let toast_h = 44i32;
        let margin = 14i32;

        let base_x = width as i32 - toast_w - margin;
        let base_y = height as i32 - toast_h - margin;

        let elapsed = self.duration - self.timer;
        let alpha = if elapsed < 0.4 {
            (elapsed / 0.4).clamp(0.0, 1.0)
        } else if self.timer < 0.6 {
            (self.timer / 0.6).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let slide_offset = ((1.0 - alpha) * 25.0) as i32;
        let bx = base_x + slide_offset;
        let by = base_y;

        // Dark glassmorphic background
        let bg_color = pack_color(0.08, 0.10, 0.14);
        fill_rect_alpha(buffer, width, height, bx, by, toast_w, toast_h, bg_color, alpha * 0.88);

        // Card borders
        let border_top = pack_color(0.30, 0.38, 0.50);
        let border_bottom = pack_color(0.04, 0.05, 0.08);
        fill_rect(buffer, width, height, bx, by, toast_w, 1, border_top);
        fill_rect(buffer, width, height, bx, by, 1, toast_h, border_top);
        fill_rect(buffer, width, height, bx, by + toast_h - 1, toast_w, 1, border_bottom);
        fill_rect(buffer, width, height, bx + toast_w - 1, by, 1, toast_h, border_bottom);

        // Accent strip on left edge
        fill_rect(buffer, width, height, bx + 1, by + 1, 3, toast_h - 2, self.accent_color);

        let content_x = bx + 12;

        // Header: "NOW PLAYING" / "TERAZ ODTWARZANE"
        let header_str = t("toast.now_playing");
        let header_text = if header_str.is_empty() || header_str.starts_with("toast.") {
            "NOW PLAYING"
        } else {
            header_str
        };
        let header_display = format!("> {}", header_text);
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &header_display,
            content_x,
            by + 4,
            1,
            1,
            self.accent_color,
        );

        // Track title (bold white)
        let title_color = pack_color(1.0, 1.0, 1.0);
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &self.title,
            content_x,
            by + 16,
            1,
            1,
            title_color,
        );

        // Subtext: "Artist • Category"
        let sub_color = pack_color(0.68, 0.74, 0.82);
        let sub_text = if !self.artist.is_empty() {
            format!("{} * {}", self.artist, self.category_name)
        } else {
            self.category_name.clone()
        };
        draw_text_colored_shadowed(
            buffer,
            width,
            height,
            &sub_text,
            content_x,
            by + 28,
            1,
            1,
            sub_color,
        );
    }
}

/// Orchestrates background soundtrack playback, playlist manifests, and notifications.
pub struct SoundtrackManager {
    pub base_dir: PathBuf,
    pub tracks: Vec<SoundtrackTrack>,
    pub current_track_idx: Option<usize>,
    pub current_category: SoundtrackCategory,
    pub delay_timer: f32,
    pub min_delay_seconds: f32,
    pub max_delay_seconds: f32,
    pub toast: MusicToast,
    rng_seed: u32,
    pub last_played_idx: Option<usize>,
}

impl SoundtrackManager {
    /// Creates a soundtrack manager and scans the target folder for manifest and .ogg tracks.
    pub fn new(folder: &str) -> Self {
        let base_dir = PathBuf::from(folder);
        let mut manager = Self {
            base_dir,
            tracks: Vec::new(),
            current_track_idx: None,
            current_category: SoundtrackCategory::Menu,
            delay_timer: 1.5, // Start first track shortly after launch
            min_delay_seconds: 15.0,
            max_delay_seconds: 40.0,
            toast: MusicToast::default(),
            rng_seed: 0x517CC1B7,
            last_played_idx: None,
        };

        manager.reload();
        manager
    }

    fn rand_f32(&mut self, min: f32, max: f32) -> f32 {
        let mut x = self.rng_seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_seed = x;
        let frac = (x as f32) / 4294967295.0;
        min + frac * (max - min)
    }

    /// Rescans `soundtracks/` directory, parses `manifest.json`, and registers any discovered audio files.
    pub fn reload(&mut self) {
        self.tracks.clear();

        if !self.base_dir.exists() {
            let _ = fs::create_dir_all(&self.base_dir);
            crate::log_info!(
                "SOUNDTRACK",
                "Created empty soundtracks directory at '{}'",
                self.base_dir.display()
            );
        }

        let manifest_path = self.base_dir.join("manifest.json");
        if manifest_path.exists() {
            if let Ok(content) = fs::read_to_string(&manifest_path) {
                self.parse_manifest(&content);
            }
        }

        // Auto-discover any .ogg or .wav tracks on disk not explicitly listed in manifest
        self.scan_directory(&self.base_dir.clone());

        crate::log_info!(
            "SOUNDTRACK",
            "Soundtrack system initialized: {} registered tracks from '{}'",
            self.tracks.len(),
            self.base_dir.display()
        );
    }

    fn parse_manifest(&mut self, json_str: &str) {
        let root = match parse_json_value_root(json_str) {
            Ok(node) => node,
            Err(e) => {
                crate::log_warn!("SOUNDTRACK", "Failed to parse soundtrack manifest: {}", e);
                return;
            }
        };

        let obj = match root.as_object() {
            Some(o) => o,
            None => return,
        };

        if let Some(settings_obj) = obj.get("settings").and_then(|s| s.as_object()) {
            if let Some(min_d) = settings_obj.get("min_delay_seconds").and_then(|v| v.as_f32()) {
                self.min_delay_seconds = min_d.max(1.0);
            }
            if let Some(max_d) = settings_obj.get("max_delay_seconds").and_then(|v| v.as_f32()) {
                self.max_delay_seconds = max_d.max(self.min_delay_seconds);
            }
        }

        if let Some(tracks_arr) = obj.get("tracks").and_then(|t| t.as_array()) {
            for track_node in tracks_arr {
                if let Some(t_obj) = track_node.as_object() {
                    let file_str = match t_obj.get("file").and_then(|f| f.as_str()) {
                        Some(f) => f,
                        None => continue,
                    };
                    let file_path = self.base_dir.join(file_str);
                    let id = t_obj
                        .get("id")
                        .and_then(|i| i.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            file_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("track")
                                .to_string()
                        });

                    let title = t_obj
                        .get("name")
                        .or_else(|| t_obj.get("title"))
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| Self::prettify_title(&id));

                    let artist = t_obj
                        .get("artist")
                        .and_then(|a| a.as_str())
                        .unwrap_or("RustCraft")
                        .to_string();

                    let mut categories = Vec::new();
                    if let Some(cats) = t_obj.get("categories").and_then(|c| c.as_array()) {
                        for c in cats {
                            if let Some(c_str) = c.as_str() {
                                categories.push(SoundtrackCategory::parse(c_str));
                            }
                        }
                    } else if let Some(cat_str) = t_obj.get("category").and_then(|c| c.as_str()) {
                        categories.push(SoundtrackCategory::parse(cat_str));
                    }

                    if categories.is_empty() {
                        categories.push(SoundtrackCategory::Uncategorized);
                    }

                    self.tracks.push(SoundtrackTrack {
                        id,
                        title,
                        artist,
                        file_path,
                        categories,
                        sample: None,
                    });
                }
            }
        }
    }

    fn scan_directory(&mut self, dir: &Path) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_directory(&path);
            } else if path.is_file() {
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();

                if ext == "ogg" || ext == "opus" || ext == "wav" {
                    // Check if already registered by manifest
                    if self.tracks.iter().any(|t| t.file_path == path) {
                        continue;
                    }

                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("track");

                    let parent_name = path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("");

                    let cat = match parent_name.to_ascii_lowercase().as_str() {
                        "creative" => SoundtrackCategory::Creative,
                        "survival" => SoundtrackCategory::Survival,
                        "menu" => SoundtrackCategory::Menu,
                        _ => SoundtrackCategory::Uncategorized,
                    };

                    self.tracks.push(SoundtrackTrack {
                        id: file_stem.to_string(),
                        title: Self::prettify_title(file_stem),
                        artist: "RustCraft".to_string(),
                        file_path: path,
                        categories: vec![cat],
                        sample: None,
                    });
                }
            }
        }
    }

    fn prettify_title(raw: &str) -> String {
        let replaced = raw.replace(['_', '-'], " ");
        let mut result = String::new();
        let mut capitalize_next = true;
        for c in replaced.chars() {
            if c.is_whitespace() {
                capitalize_next = true;
                result.push(c);
            } else if capitalize_next {
                result.extend(c.to_uppercase());
                capitalize_next = false;
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Primary game-loop update coordinator.
    ///
    /// Manages track transitions, context switching (creative vs survival vs menu),
    /// pause intervals between songs, and toast expiration.
    pub fn update(
        &mut self,
        dt: f32,
        current_context: SoundtrackCategory,
        audio: &mut AudioEngine,
        music_vol: f32,
        is_muted: bool,
    ) {
        self.toast.update(dt);

        if is_muted || music_vol <= 0.001 {
            return;
        }

        // Context transition: if game switched (e.g. from Menu to In-Game), smoothly fade out menu music
        if self.current_category != current_context {
            if let Some(cur_idx) = self.current_track_idx {
                if let Some(track) = self.tracks.get(cur_idx) {
                    if !track.matches_category(current_context) {
                        audio.stop_music(1.5);
                        self.current_track_idx = None;
                        self.delay_timer = 2.0; // Quick pause before starting in-game soundtrack
                    }
                }
            }
            self.current_category = current_context;
        }

        let is_playing = audio.is_music_playing();

        if is_playing {
            // Track is playing normally
            return;
        }

        // Delay between songs
        self.delay_timer -= dt;
        if self.delay_timer <= 0.0 {
            self.play_next_matching(current_context, audio);
            self.delay_timer = self.rand_f32(self.min_delay_seconds, self.max_delay_seconds);
        }
    }

    /// Selects and starts playing the next eligible track for the specified context.
    pub fn play_next_matching(&mut self, context: SoundtrackCategory, audio: &mut AudioEngine) {
        let eligible_indices: Vec<usize> = self
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.matches_category(context))
            .map(|(i, _)| i)
            .collect();

        if eligible_indices.is_empty() {
            return;
        }

        // Pick next track (avoiding immediately repeating previous track if multiple exist)
        let chosen_idx = if eligible_indices.len() == 1 {
            eligible_indices[0]
        } else {
            let filtered: Vec<usize> = eligible_indices
                .iter()
                .copied()
                .filter(|&i| Some(i) != self.last_played_idx)
                .collect();
            let pool = if filtered.is_empty() { &eligible_indices } else { &filtered };
            let rand_val = self.rand_f32(0.0, pool.len() as f32);
            let idx = (rand_val as usize).min(pool.len() - 1);
            pool[idx]
        };

        self.play_track_at_index(chosen_idx, audio);
    }

    /// Plays a specific track by its index in the soundtrack list.
    pub fn play_track_at_index(&mut self, idx: usize, audio: &mut AudioEngine) {
        if idx >= self.tracks.len() {
            return;
        }

        let sample = match self.tracks[idx].load_sample() {
            Ok(s) => s,
            Err(e) => {
                crate::log_warn!("SOUNDTRACK", "Failed to load soundtrack: {}", e);
                return;
            }
        };

        let title = self.tracks[idx].title.clone();
        let artist = self.tracks[idx].artist.clone();
        let primary_cat = self.tracks[idx]
            .categories
            .first()
            .copied()
            .unwrap_or(SoundtrackCategory::Uncategorized);

        self.current_track_idx = Some(idx);
        self.last_played_idx = Some(idx);

        audio.play_music(sample, 1.0, false);
        self.toast.show(&title, &artist, primary_cat);

        crate::log_info!(
            "SOUNDTRACK",
            "Now playing soundtrack: '{}' by {} [{:?}]",
            title,
            artist,
            primary_cat
        );
    }

    /// Manually skips to the next track.
    pub fn skip_next(&mut self, context: SoundtrackCategory, audio: &mut AudioEngine) {
        audio.stop_music(0.5);
        self.play_next_matching(context, audio);
        self.delay_timer = self.rand_f32(self.min_delay_seconds, self.max_delay_seconds);
    }

    /// Renders the now-playing toast notification.
    pub fn draw_toast(&self, buffer: &mut [u32], width: usize, height: usize) {
        self.toast.draw(buffer, width, height);
    }

    /// Returns the currently active track metadata if any.
    pub fn current_track(&self) -> Option<&SoundtrackTrack> {
        self.current_track_idx.and_then(|i| self.tracks.get(i))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundtrack_category_parsing_and_properties() {
        assert_eq!(SoundtrackCategory::parse("creative"), SoundtrackCategory::Creative);
        assert_eq!(SoundtrackCategory::parse("CREATIVE"), SoundtrackCategory::Creative);
        assert_eq!(SoundtrackCategory::parse("survival"), SoundtrackCategory::Survival);
        assert_eq!(SoundtrackCategory::parse("menu"), SoundtrackCategory::Menu);
        assert_eq!(SoundtrackCategory::parse("main_menu"), SoundtrackCategory::Menu);
        assert_eq!(SoundtrackCategory::parse("uncategorized"), SoundtrackCategory::Uncategorized);
        assert_eq!(SoundtrackCategory::parse("anything_else"), SoundtrackCategory::Uncategorized);

        assert_eq!(SoundtrackCategory::Creative.name(), "creative");
        assert_eq!(SoundtrackCategory::Survival.name(), "survival");
        assert_eq!(SoundtrackCategory::Menu.name(), "menu");
        assert_eq!(SoundtrackCategory::Uncategorized.name(), "uncategorized");
    }

    #[test]
    fn test_soundtrack_manifest_parsing() {
        let mgr = SoundtrackManager::new("soundtracks");
        assert!(!mgr.tracks.is_empty(), "Should have discovered soundtrack tracks");

        // Verify tracks from manifest
        let creative_track = mgr.tracks.iter().find(|t| t.id == "creative_calm");
        assert!(creative_track.is_some(), "creative_calm track must exist");
        let ct = creative_track.unwrap();
        assert!(ct.matches_category(SoundtrackCategory::Creative));
        assert!(!ct.matches_category(SoundtrackCategory::Survival));

        let wanderer_track = mgr.tracks.iter().find(|t| t.id == "wanderer");
        assert!(wanderer_track.is_some(), "wanderer track must exist");
        let wt = wanderer_track.unwrap();
        // Uncategorized tracks must match any context
        assert!(wt.matches_category(SoundtrackCategory::Creative));
        assert!(wt.matches_category(SoundtrackCategory::Survival));
        assert!(wt.matches_category(SoundtrackCategory::Menu));
    }

    #[test]
    fn test_ogg_opus_decoding_roundtrip() {
        // Create an in-memory Ogg Opus stream (Stereo 48kHz)
        let mut encoder = opus_pure::OpusEncoder::new(48000, 2, opus_pure::Application::Audio).unwrap();
        let head = opus_pure::OpusHead::for_encoder(&encoder, 48000);
        let mut writer = opus_pure::OggOpusWriter::new(Vec::new(), head).unwrap();

        // 20ms of stereo 440Hz sine wave (960 samples per channel)
        let frame_size = 960;
        let mut pcm_in = Vec::with_capacity(frame_size * 2);
        for i in 0..frame_size {
            let t = i as f32 / 48000.0;
            let val = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
            pcm_in.push(val);
            pcm_in.push(val * 0.8);
        }

        let mut packet_buf = vec![0u8; 1000];
        let pkt_len = encoder.encode(&pcm_in, frame_size, &mut packet_buf).unwrap();
        writer.write_packet(&packet_buf[..pkt_len]).unwrap();
        let ogg_bytes = writer.finish().unwrap();

        // Decode using SoundSample::from_file_bytes
        let sample = SoundSample::from_file_bytes(&ogg_bytes).expect("Failed to decode Ogg Opus stream");
        assert_eq!(sample.channels, 2, "Decoded sample must be stereo");
        assert_eq!(sample.sample_rate, 48000);
        assert!(!sample.data.is_empty(), "Must contain decoded audio frames");
    }

    #[test]
    fn test_music_toast_lifecycle_and_rendering() {
        let mut toast = MusicToast::default();
        assert!(!toast.visible);

        toast.show("Calm Horizon", "RustCraft", SoundtrackCategory::Creative);
        assert!(toast.visible);
        assert_eq!(toast.title, "Calm Horizon");
        assert_eq!(toast.category_name, "Creative");

        // Render to dummy framebuffer
        let mut fb = vec![0u32; 800 * 600];
        toast.draw(&mut fb, 800, 600);

        // Verify pixels were written
        let painted_pixels = fb.iter().filter(|&&p| p != 0).count();
        assert!(painted_pixels > 100, "Toast must paint visible pixels to framebuffer");

        // Update timer
        toast.update(2.0);
        assert!(toast.visible);
        assert!(toast.timer <= 3.1);

        toast.update(4.0);
        assert!(!toast.visible);
    }

    #[test]
    fn test_decode_user_opus_if_present() {
        let path = "soundtracks/uncategorized/Buant Groove.opus";
        if std::path::Path::new(path).exists() {
            let bytes = std::fs::read(path).unwrap();
            let sample = SoundSample::from_file_bytes(&bytes).unwrap();
            assert_eq!(sample.channels, 2);
            assert_eq!(sample.sample_rate, 48000);
            assert!(sample.data.len() > 0);
        }
    }
}

