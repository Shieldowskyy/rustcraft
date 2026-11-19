//! Parsowanie pliku `manifest.json`, skanowanie katalogów `soundtracks/` na dysku oraz wbudowanych zasobów audio.

use std::fs;
use std::path::Path;

use crate::engine::modding::parse_json_value_root;
use crate::platform::{storage::secondary_soundtracks_dir, system::vita_debug_log};

use super::manager::SoundtrackManager;
use super::types::{SoundtrackCategory, SoundtrackTrack};

impl SoundtrackManager {
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

        self.scan_directory(&self.base_dir.clone());
        if let Some(secondary_dir) = secondary_soundtracks_dir(&self.base_dir) {
            self.scan_directory(&secondary_dir);
        }
        self.scan_embedded_soundtracks();

        vita_debug_log(&format!(
            "[SOUNDTRACK] Registered {} tracks (base_dir: '{}')",
            self.tracks.len(),
            self.base_dir.display()
        ));

        crate::log_info!(
            "SOUNDTRACK",
            "Soundtrack system initialized: {} registered tracks from '{}'",
            self.tracks.len(),
            self.base_dir.display()
        );
    }

    pub(super) fn parse_manifest(&mut self, json_str: &str) {
        let root = match parse_json_value_root(json_str) {
            Ok(node) => node,
            Err(e) => {
                crate::log_warn!("SOUNDTRACK", "Failed to parse soundtrack manifest: {}", e);
                return;
            }
        };

        let Some(obj) = root.as_object() else {
            return;
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
                    let Some(file_str) = t_obj.get("file").and_then(|f| f.as_str()) else {
                        continue;
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
                        embedded_data: None,
                    });
                }
            }
        }
    }

    pub(super) fn scan_directory(&mut self, dir: &Path) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
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
                        embedded_data: None,
                    });
                }
            }
        }
    }

    pub(super) fn scan_embedded_soundtracks(&mut self) {
        for &(asset_path, data) in crate::embedded_assets::SOUNDTRACKS {
            let Some(relative_path) = asset_path.strip_prefix("soundtracks/") else {
                continue;
            };
            let path = self.base_dir.join(relative_path);

            if let Some(track) = self.tracks.iter_mut().find(|track| track.file_path == path) {
                track.embedded_data = Some(data);
                continue;
            }

            let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("track");
            let parent_name = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");
            let category = match parent_name.to_ascii_lowercase().as_str() {
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
                categories: vec![category],
                sample: None,
                embedded_data: Some(data),
            });
        }
    }

    pub(super) fn prettify_title(raw: &str) -> String {
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

    pub(super) fn choose_next_index(&mut self, context: SoundtrackCategory) -> Option<usize> {
        let mut eligible_indices: Vec<usize> = self
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                (t.file_path.exists() || t.sample.is_some() || t.embedded_data.is_some())
                    && t.matches_category(context)
            })
            .map(|(i, _)| i)
            .collect();

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
            return None;
        }

        let uncategorized_indices: Vec<usize> = eligible_indices
            .iter()
            .copied()
            .filter(|&idx| {
                if let Some(t) = self.tracks.get(idx) {
                    t.categories.contains(&SoundtrackCategory::Uncategorized)
                        || t.file_path
                            .to_string_lossy()
                            .to_ascii_lowercase()
                            .contains("uncategorized")
                } else {
                    false
                }
            })
            .collect();

        let chosen_idx = if !uncategorized_indices.is_empty() {
            self.uncategorized_queue
                .retain(|idx| uncategorized_indices.contains(idx));

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
            let pool = if filtered.is_empty() {
                &eligible_indices
            } else {
                &filtered
            };
            let rand_val = self.rand_f32(0.0, pool.len() as f32);
            let idx = (rand_val as usize).min(pool.len() - 1);
            pool[idx]
        };

        Some(chosen_idx)
    }
}
