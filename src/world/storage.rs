//! Transitional compatibility facade preserving legacy `crate::world::storage::*` paths.

pub use crate::features::world::save_meta::{get_save_metadata, SaveMetadata};
pub use crate::features::world::storage::{
    achievement_progress_path, app_files_dir, capture_thumbnail, delete_save,
    load_achievement_progress, load_quest_progress, load_thumbnail, quest_progress_path,
    save_achievement_progress, save_current_world, save_path, save_quest_progress, save_thumbnail,
    set_custom_save_dir, settings_file_path, soundtracks_dir, thumbnail_path, THUMB_SIZE,
};
