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

/// Background loader for asynchronously decoding audio files without frame stalls.
pub struct AsyncTrackLoader {
    pub track_idx: usize,
    pub receiver: std::sync::mpsc::Receiver<Result<Arc<SoundSample>, String>>,
}

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

    /// Renders the now-playing toast notification in the bottom-right corner with smooth fade and slide,
    /// along with a real-time progress bar.
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
            if elapsed < 0.4 {
                (elapsed / 0.4).clamp(0.0, 1.0)
            } else if self.timer < 0.6 {
                (self.timer / 0.6).clamp(0.0, 1.0)
            } else {
                1.0
            }
        };

        let slide_offset = if force_show {
            0
        } else {
            ((1.0 - alpha) * 25.0) as i32
        };
        let bx = base_x + slide_offset;
        let by = base_y;

        // Dark glassmorphic background
        let bg_color = pack_color(0.08, 0.10, 0.14);
        fill_rect_alpha(buffer, width, height, bx, by, toast_w, toast_h, bg_color, alpha * 0.90);

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
            by + 27,
            1,
            1,
            sub_color,
        );

        // Duration text on right side of subtext
        if let (Some(p), Some(dur)) = (progress, duration_sec) {
            if dur > 0.0 {
                let cur_s = (p * dur) as u32;
                let total_s = dur as u32;
                let time_str = format!("{}:{:02}/{}:{:02}", cur_s / 60, cur_s % 60, total_s / 60, total_s % 60);
                let tw = crate::graphics::font::measure_text(&time_str, 1, 1);
                let tx = bx + toast_w - 12 - tw;
                draw_text_colored_shadowed(buffer, width, height, &time_str, tx, by + 27, 1, 1, pack_color(0.60, 0.70, 0.82));
            }
        }

        // Real-time playback progress bar
        let bar_x = content_x;
        let bar_y = by + 41;
        let bar_w = toast_w - 24;
        let bar_h = 3;
        let track_bg = pack_color(0.18, 0.22, 0.28);
        fill_rect(buffer, width, height, bar_x, bar_y, bar_w, bar_h, track_bg);

        let prog_val = progress.unwrap_or(0.0).clamp(0.0, 1.0);
        let fill_w = ((bar_w as f32) * prog_val).round() as i32;
        if fill_w > 0 {
            fill_rect(buffer, width, height, bar_x, bar_y, fill_w, bar_h, self.accent_color);
            if fill_w < bar_w {
                fill_rect(buffer, width, height, bar_x + fill_w - 1, bar_y, 2, bar_h, pack_color(1.0, 1.0, 1.0));
            }
        }
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
    pub uncategorized_queue: Vec<usize>,
    pub loading_track: Option<AsyncTrackLoader>,
    pub current_duration_sec: f32,
}

impl SoundtrackManager {
    /// Creates a soundtrack manager and scans the target folder for manifest and audio tracks.
    pub fn new<P: AsRef<Path>>(folder: P) -> Self {
        #[allow(unused_mut)]
        let mut base_dir = folder.as_ref().to_path_buf();
        #[cfg(target_os = "android")]
        {
            if !base_dir.is_absolute() {
                let android_dir = crate::world::storage::app_files_dir().join(&base_dir);
                if android_dir.exists() {
                    base_dir = android_dir;
                }
            }
        }
        #[cfg(target_os = "vita")]
        {
            if !base_dir.to_string_lossy().contains(':') {
                let vita_dir = crate::world::storage::soundtracks_dir();
                if vita_dir.exists() {
                    base_dir = vita_dir;
                }
            }
        }
        let initial_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| {
                let nanos = d.as_nanos();
                ((nanos ^ (nanos >> 32)) as u32) ^ 0x517CC1B7
            })
            .unwrap_or(0x517CC1B7);
        let rng_seed = if initial_seed == 0 { 0x517CC1B7 } else { initial_seed };

        let mut manager = Self {
            base_dir,
            tracks: Vec::new(),
            current_track_idx: None,
            current_category: SoundtrackCategory::Menu,
            delay_timer: 1.5, // Start first track shortly after launch
            min_delay_seconds: 15.0,
            max_delay_seconds: 40.0,
            toast: MusicToast::default(),
            rng_seed,
            last_played_idx: None,
            uncategorized_queue: Vec::new(),
            loading_track: None,
            current_duration_sec: 0.0,
        };

        manager.reload();
        manager
    }

    /// Sets a specific PRNG seed (useful for deterministic tests).
    pub fn set_seed(&mut self, seed: u32) {
        self.rng_seed = if seed == 0 { 0x517CC1B7 } else { seed };
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

    /// Generates a random integer in `[min, max_exclusive)`.
    pub fn rand_usize(&mut self, min: usize, max_exclusive: usize) -> usize {
        if min >= max_exclusive {
            return min;
        }
        let range = (max_exclusive - min) as f32;
        let val = self.rand_f32(0.0, range) as usize;
        (min + val).min(max_exclusive - 1)
    }

    /// In-place Fisher-Yates shuffle for track index slices.
    pub fn shuffle_indices(&mut self, list: &mut [usize]) {
        if list.len() <= 1 {
            return;
        }
        for i in (1..list.len()).rev() {
            let j = self.rand_usize(0, i + 1);
            list.swap(i, j);
        }
    }

    /// Refills the uncategorized shuffle queue with a randomized permutation.
    pub fn refill_uncategorized_queue(&mut self, eligible_uncategorized: &[usize]) {
        if eligible_uncategorized.is_empty() {
            self.uncategorized_queue.clear();
            return;
        }
        let mut new_queue = eligible_uncategorized.to_vec();
        self.shuffle_indices(&mut new_queue);

        // Ensure the first track of the new cycle doesn't repeat the last played track
        if new_queue.len() > 1 && Some(new_queue[0]) == self.last_played_idx {
            let last_idx = new_queue.len() - 1;
            new_queue.swap(0, last_idx);
        }
        self.uncategorized_queue = new_queue;
    }

    /// Rescans `soundtracks/` directory, parses `manifest.json`, and registers any discovered audio files.
    pub fn reload(&mut self) {
        self.tracks.clear();
        self.uncategorized_queue.clear();

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
    /// Manages non-blocking async track decoding, track transitions, context switching,
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

        // Check if an asynchronously requested track has finished decoding in the background
        if let Some(loader) = &self.loading_track {
            match loader.receiver.try_recv() {
                Ok(Ok(sample)) => {
                    let idx = loader.track_idx;
                    self.loading_track = None;
                    if idx < self.tracks.len() {
                        self.tracks[idx].sample = Some(Arc::clone(&sample));
                        self.start_playing_sample(idx, sample, audio);
                    }
                }
                Ok(Err(err)) => {
                    crate::log_warn!("SOUNDTRACK", "Async track decode error: {}", err);
                    self.loading_track = None;
                    self.delay_timer = 2.0; // Retry after short pause
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still decoding in background thread: zero main thread lag!
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.loading_track = None;
                }
            }
        }

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

        if is_playing || self.loading_track.is_some() {
            // Track is playing normally or actively loading in background
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
    ///
    /// Tracks in the `uncategorized/` folder are managed via a randomized Fisher-Yates
    /// shuffle queue so they never play in the same predictable order and don't repeat
    /// before every other uncategorized track has had a chance to play.
    pub fn play_next_matching(&mut self, context: SoundtrackCategory, audio: &mut AudioEngine) {
        let mut eligible_indices: Vec<usize> = self
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| (t.file_path.exists() || t.sample.is_some()) && t.matches_category(context))
            .map(|(i, _)| i)
            .collect();

        // Fallback for tests or synthetic tracks where file_path may not exist on disk
        if eligible_indices.is_empty() {
            eligible_indices = self
                .tracks
                .iter()
                .enumerate()
                .filter(|(_, t)| t.matches_category(context))
                .map(|(i, _)| i)
                .collect();
        }

        if eligible_indices.is_empty() {
            return;
        }

        // Identify uncategorized tracks in the eligible pool
        let uncategorized_indices: Vec<usize> = eligible_indices
            .iter()
            .copied()
            .filter(|&idx| {
                if let Some(t) = self.tracks.get(idx) {
                    t.categories.contains(&SoundtrackCategory::Uncategorized)
                        || t.file_path.to_string_lossy().to_ascii_lowercase().contains("uncategorized")
                } else {
                    false
                }
            })
            .collect();

        let chosen_idx = if !uncategorized_indices.is_empty() {
            // Keep queue clean of any indices no longer eligible
            self.uncategorized_queue.retain(|idx| uncategorized_indices.contains(idx));

            if self.uncategorized_queue.is_empty() {
                self.refill_uncategorized_queue(&uncategorized_indices);
            }

            if !self.uncategorized_queue.is_empty() {
                self.uncategorized_queue.remove(0)
            } else {
                let rand_val = self.rand_f32(0.0, eligible_indices.len() as f32);
                let idx = (rand_val as usize).min(eligible_indices.len() - 1);
                eligible_indices[idx]
            }
        } else if eligible_indices.len() == 1 {
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

    /// Plays a specific track asynchronously without blocking the main game loop thread.
    pub fn play_track_at_index(&mut self, idx: usize, audio: &mut AudioEngine) {
        if idx >= self.tracks.len() {
            return;
        }

        // If sample is already in memory, start immediately!
        if let Some(sample) = &self.tracks[idx].sample {
            let sample_clone = Arc::clone(sample);
            self.start_playing_sample(idx, sample_clone, audio);
            return;
        }

        // If this track is already being loaded asynchronously in the background, don't re-spawn
        if let Some(loader) = &self.loading_track {
            if loader.track_idx == idx {
                return;
            }
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let file_path = self.tracks[idx].file_path.clone();
        let _ = std::thread::Builder::new()
            .name("soundtrack-async-loader".to_string())
            .spawn(move || {
                let res = (|| -> Result<Arc<SoundSample>, String> {
                    let bytes = fs::read(&file_path)
                        .map_err(|e| format!("Could not read audio file '{}': {}", file_path.display(), e))?;
                    let sample = SoundSample::from_file_bytes(&bytes)
                        .map_err(|e| format!("Failed to decode audio file '{}': {}", file_path.display(), e))?;
                    Ok(Arc::new(sample))
                })();
                let _ = tx.send(res);
            });

        self.loading_track = Some(AsyncTrackLoader {
            track_idx: idx,
            receiver: rx,
        });
    }

    fn start_playing_sample(&mut self, idx: usize, sample: Arc<SoundSample>, audio: &mut AudioEngine) {
        let title = self.tracks[idx].title.clone();
        let artist = self.tracks[idx].artist.clone();
        let primary_cat = self.tracks[idx]
            .categories
            .first()
            .copied()
            .unwrap_or(SoundtrackCategory::Uncategorized);

        self.current_track_idx = Some(idx);
        self.last_played_idx = Some(idx);
        self.current_duration_sec = sample.duration_seconds();

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

    /// Renders the now-playing toast notification, showing on pause menu only when music is active.
    pub fn draw_toast(
        &mut self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        is_paused: bool,
        audio: &AudioEngine,
    ) {
        let progress = audio.music_playback_progress();
        // Only force-show toast on pause menu when a track is actually playing.
        // During silence gaps between tracks current_track_idx may still be set,
        // but audio.is_music_playing() will return false — we respect that and hide
        // the toast so it doesn't linger over the pause menu during silent intervals.
        let music_active = audio.is_music_playing();

        if is_paused && music_active {
            // Pause menu: always display toast of current soundtrack with progress bar
            let track_info = self.current_track_idx
                .and_then(|i| self.tracks.get(i))
                .map(|track| {
                    let cat = track.categories.first().copied().unwrap_or(SoundtrackCategory::Uncategorized);
                    (track.title.clone(), track.artist.clone(), cat)
                });

            if let Some((title, artist, cat)) = track_info {
                self.toast.title = title;
                self.toast.artist = artist;
                self.toast.category_name = cat.display_name().to_string();
                self.toast.accent_color = cat.accent_color();
                self.toast.visible = true;
                self.toast.draw(buffer, width, height, true, progress, Some(self.current_duration_sec));
            } else if self.toast.visible {
                self.toast.draw(buffer, width, height, true, progress, Some(self.current_duration_sec));
            }
        } else {
            self.toast.draw(buffer, width, height, false, progress, Some(self.current_duration_sec));
        }
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
        toast.draw(&mut fb, 800, 600, false, Some(0.4), Some(180.0));

        // Verify pixels were written
        let painted_pixels = fb.iter().filter(|&&p| p != 0).count();
        assert!(painted_pixels > 100, "Toast must paint visible pixels to framebuffer");

        // Update timer
        toast.update(2.0);
        assert!(toast.visible);
        assert!(toast.timer <= 3.1);

        toast.update(4.0);
        assert!(!toast.visible);

        // Force show (e.g. pause menu) even when timer expired and not visible
        let mut fb_pause = vec![0u32; 800 * 600];
        toast.draw(&mut fb_pause, 800, 600, true, Some(0.75), Some(200.0));
        let painted_pause = fb_pause.iter().filter(|&&p| p != 0).count();
        assert!(painted_pause > 100, "Forced toast on pause menu must render pixels");
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

    #[test]
    fn test_uncategorized_random_shuffling_and_no_repeats() {
        let mut mgr = SoundtrackManager::new("soundtracks");
        // Filter uncategorized tracks
        let uncategorized_indices: Vec<usize> = mgr
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.categories.contains(&SoundtrackCategory::Uncategorized))
            .map(|(i, _)| i)
            .collect();

        assert!(uncategorized_indices.len() >= 3, "Expected at least 3 uncategorized tracks");

        // Seed 1
        mgr.set_seed(0x12345678);
        mgr.refill_uncategorized_queue(&uncategorized_indices);
        let order_1 = mgr.uncategorized_queue.clone();

        // Seed 2
        mgr.set_seed(0x87654321);
        mgr.refill_uncategorized_queue(&uncategorized_indices);
        let order_2 = mgr.uncategorized_queue.clone();

        // Different seeds must generate different random orders (not fixed identical order)
        assert_ne!(order_1, order_2, "Different PRNG seeds must yield different random playlist orders");

        // All elements in order_1 must be valid unique tracks from the pool
        assert_eq!(order_1.len(), uncategorized_indices.len());
        for idx in &uncategorized_indices {
            assert!(order_1.contains(idx), "Shuffle queue must contain all tracks without dropping any");
        }
    }
}

