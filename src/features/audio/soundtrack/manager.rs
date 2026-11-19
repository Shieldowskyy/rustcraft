//! Zarządzanie kolejką odtwarzania ścieżek dźwiękowych (`SoundtrackManager`), losowaniem Fisher-Yates i asynchronicznym ładowaniem.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::features::audio::engine::AudioEngine;
use crate::features::audio::sound::SoundSample;
use crate::platform::{
    storage::{resolve_soundtrack_base_dir, soundtracks_dir},
    system::vita_debug_log,
    threading::configure_background_decoder_affinity,
};

use super::toast::MusicToast;
use super::types::{AsyncTrackLoader, SoundtrackCategory, SoundtrackTrack};

pub struct SoundtrackManager {
    pub base_dir: PathBuf,
    pub tracks: Vec<SoundtrackTrack>,
    pub current_track_idx: Option<usize>,
    pub current_category: SoundtrackCategory,
    pub delay_timer: f32,
    pub min_delay_seconds: f32,
    pub max_delay_seconds: f32,
    pub toast: MusicToast,
    pub(super) rng_seed: u32,
    pub last_played_idx: Option<usize>,
    pub uncategorized_queue: Vec<usize>,
    pub loading_track: Option<AsyncTrackLoader>,
    pub current_duration_sec: f32,
    pub pending_intro: Option<usize>,
}

static PRELOADED_MANAGER: std::sync::Mutex<Option<SoundtrackManager>> = std::sync::Mutex::new(None);

pub fn preload_intro_music() {
    let mut mgr = SoundtrackManager::new(soundtracks_dir());
    mgr.preload_intro(SoundtrackCategory::Menu);
    if let Ok(mut slot) = PRELOADED_MANAGER.lock() {
        *slot = Some(mgr);
    }
}

pub fn take_preloaded_manager() -> Option<SoundtrackManager> {
    PRELOADED_MANAGER.lock().ok().and_then(|mut s| s.take())
}

impl SoundtrackManager {
    pub fn new<P: AsRef<Path>>(folder: P) -> Self {
        let base_dir = resolve_soundtrack_base_dir(folder.as_ref());
        let initial_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| {
                let nanos = d.as_nanos();
                ((nanos ^ (nanos >> 32)) as u32) ^ 0x517CC1B7
            })
            .unwrap_or(0x517CC1B7);
        let rng_seed = if initial_seed == 0 {
            0x517CC1B7
        } else {
            initial_seed
        };

        let mut manager = Self {
            base_dir,
            tracks: Vec::new(),
            current_track_idx: None,
            current_category: SoundtrackCategory::Menu,
            delay_timer: 1.5,
            min_delay_seconds: 15.0,
            max_delay_seconds: 40.0,
            toast: MusicToast::default(),
            rng_seed,
            last_played_idx: None,
            uncategorized_queue: Vec::new(),
            loading_track: None,
            current_duration_sec: 0.0,
            pending_intro: None,
        };

        manager.reload();
        manager
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.rng_seed = if seed == 0 { 0x517CC1B7 } else { seed };
    }

    pub(super) fn rand_f32(&mut self, min: f32, max: f32) -> f32 {
        let mut x = self.rng_seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_seed = x;
        let frac = (x as f32) / 4294967295.0;
        min + frac * (max - min)
    }

    pub fn rand_usize(&mut self, min: usize, max_exclusive: usize) -> usize {
        if min >= max_exclusive {
            return min;
        }
        let range = (max_exclusive - min) as f32;
        let val = self.rand_f32(0.0, range) as usize;
        (min + val).min(max_exclusive - 1)
    }

    pub fn shuffle_indices(&mut self, list: &mut [usize]) {
        if list.len() <= 1 {
            return;
        }
        for i in (1..list.len()).rev() {
            let j = self.rand_usize(0, i + 1);
            list.swap(i, j);
        }
    }

    pub fn refill_uncategorized_queue(&mut self, eligible_uncategorized: &[usize]) {
        if eligible_uncategorized.is_empty() {
            self.uncategorized_queue.clear();
            return;
        }
        let mut new_queue = eligible_uncategorized.to_vec();
        self.shuffle_indices(&mut new_queue);

        if new_queue.len() > 1 && Some(new_queue[0]) == self.last_played_idx {
            let last_idx = new_queue.len() - 1;
            new_queue.swap(0, last_idx);
        }
        self.uncategorized_queue = new_queue;
    }

    pub fn update(
        &mut self,
        dt: f32,
        current_context: SoundtrackCategory,
        audio: &mut AudioEngine,
        music_vol: f32,
        is_muted: bool,
    ) {
        self.toast.update(dt);

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
                    vita_debug_log(&format!("[SOUNDTRACK] Async decode error: {}", err));
                    crate::log_warn!("SOUNDTRACK", "Async track decode error: {}", err);
                    self.loading_track = None;
                    self.delay_timer = 2.0;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => return,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    vita_debug_log(
                        "[SOUNDTRACK] Async loader thread crashed/disconnected (likely OOM or stack overflow)",
                    );
                    self.loading_track = None;
                }
            }
        }

        if is_muted || music_vol <= 0.001 {
            return;
        }

        if self.current_category != current_context {
            if let Some(cur_idx) = self.current_track_idx {
                if let Some(track) = self.tracks.get(cur_idx) {
                    if !track.matches_category(current_context) {
                        audio.stop_music(1.5);
                        self.current_track_idx = None;
                        self.delay_timer = 2.0;
                    }
                }
            }
            self.current_category = current_context;
        }

        if audio.is_music_playing() || self.loading_track.is_some() {
            return;
        }

        self.delay_timer -= dt;
        if self.delay_timer <= 0.0 {
            self.play_next_matching(current_context, audio);
            self.delay_timer = self.rand_f32(self.min_delay_seconds, self.max_delay_seconds);
        }
    }

    pub fn play_next_matching(&mut self, context: SoundtrackCategory, audio: &mut AudioEngine) {
        if let Some(idx) = self.choose_next_index(context) {
            self.play_track_at_index(idx, audio);
        }
    }

    pub fn preload_intro(&mut self, context: SoundtrackCategory) {
        let Some(idx) = self.choose_next_index(context) else {
            return;
        };
        let path = self.tracks[idx].file_path.clone();
        match self.tracks[idx].load_sample() {
            Ok(_) => self.pending_intro = Some(idx),
            Err(e) => {
                crate::log_warn!(
                    "SOUNDTRACK",
                    "Intro preload failed for '{}': {}",
                    path.display(),
                    e
                );
            }
        }
    }

    pub fn start_intro(&mut self, audio: &mut AudioEngine) {
        if let Some(idx) = self.pending_intro.take() {
            if let Some(sample) = self.tracks[idx].sample.clone() {
                self.start_playing_sample(idx, sample, audio);
                self.toast.visible = false;
                self.delay_timer = 2.0;
            }
        }
    }

    pub fn show_current_toast(&mut self) {
        if let Some(t) = self.current_track_idx.and_then(|i| self.tracks.get(i)) {
            let cat = t
                .categories
                .first()
                .copied()
                .unwrap_or(SoundtrackCategory::Uncategorized);
            let (title, artist) = (t.title.clone(), t.artist.clone());
            self.toast.show(&title, &artist, cat);
        }
    }

    pub fn play_track_at_index(&mut self, idx: usize, audio: &mut AudioEngine) {
        if idx >= self.tracks.len() {
            return;
        }

        if let Some(sample) = &self.tracks[idx].sample {
            let sample_clone = Arc::clone(sample);
            self.start_playing_sample(idx, sample_clone, audio);
            return;
        }

        if let Some(loader) = &self.loading_track {
            if loader.track_idx == idx {
                return;
            }
        }

        let (tx, rx) = std::sync::mpsc::channel();
        let track = self.tracks[idx].clone();
        // DLACZEGO: PS Vita przydziela nowym wątkom domyślnie tylko 64 KB stosu, co przepełnia stos
        // w dekoderach Opus/Vorbis; wymuszamy 512 KB stosu i przypinamy dekoder do rdzeni 1 i 2.
        let _ = std::thread::Builder::new()
            .name("soundtrack-async-loader".to_string())
            .stack_size(512 * 1024)
            .spawn(move || {
                configure_background_decoder_affinity();
                let mut track = track;
                let res = track.load_sample();
                let _ = tx.send(res);
            });

        self.loading_track = Some(AsyncTrackLoader {
            track_idx: idx,
            receiver: rx,
        });
    }

    fn start_playing_sample(
        &mut self,
        idx: usize,
        sample: Arc<SoundSample>,
        audio: &mut AudioEngine,
    ) {
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

        vita_debug_log(&format!(
            "[SOUNDTRACK] Playing '{}' ({:.1}s)",
            title, self.current_duration_sec
        ));

        crate::log_info!(
            "SOUNDTRACK",
            "Now playing soundtrack: '{}' by {} [{:?}]",
            title,
            artist,
            primary_cat
        );
    }

    pub fn skip_next(&mut self, context: SoundtrackCategory, audio: &mut AudioEngine) {
        audio.stop_music(0.5);
        self.play_next_matching(context, audio);
        self.delay_timer = self.rand_f32(self.min_delay_seconds, self.max_delay_seconds);
    }

    pub fn draw_toast(
        &mut self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        is_paused: bool,
        audio: &AudioEngine,
    ) {
        let progress = audio.music_playback_progress();
        let music_active = audio.is_music_playing();

        if is_paused && music_active {
            if let Some(track) = self.current_track_idx.and_then(|i| self.tracks.get(i)) {
                let cat = track
                    .categories
                    .first()
                    .copied()
                    .unwrap_or(SoundtrackCategory::Uncategorized);
                self.toast.title = track.title.clone();
                self.toast.artist = track.artist.clone();
                self.toast.category_name = cat.display_name().to_string();
                self.toast.accent_color = cat.accent_color();
                self.toast.visible = true;
            }
            if self.toast.visible {
                self.toast.draw(buffer, width, height, true, progress, Some(self.current_duration_sec));
            }
        } else {
            self.toast.draw(buffer, width, height, false, progress, Some(self.current_duration_sec));
        }
    }

    pub fn current_track(&self) -> Option<&SoundtrackTrack> {
        self.current_track_idx.and_then(|i| self.tracks.get(i))
    }
}
