use crate::core::math::Vec3;
use super::{RumbleRequest, RustCraftApp};

impl RustCraftApp {
    pub fn request_rumble(&mut self, strong: f32, weak: f32, duration_ms: u32) {
        if self.settings.vibration {
            self.pending_rumble = Some(RumbleRequest { strong, weak, duration_ms });
        }
    }

    /// Plays a 3D spatialized sound effect with realistic acoustic voxel occlusion through terrain/walls.
    pub fn play_world_sound(&mut self, sound_id: crate::audio::SoundId, pos: Vec3, volume: f32, pitch: f32) {
        let occlusion = self.world.calculate_acoustic_occlusion(self.camera.position, pos);
        self.audio.play_sound_3d_occluded(sound_id, pos, volume, pitch, occlusion);
    }

    /// Plays a 3D sound with realistic Minecraft-style pitch variation (±12%) and acoustic occlusion.
    pub fn play_world_sound_varied(&mut self, sound_id: crate::audio::SoundId, pos: Vec3, volume: f32) {
        let occlusion = self.world.calculate_acoustic_occlusion(self.camera.position, pos);
        self.audio.play_sound_3d_varied_occluded(sound_id, pos, volume, occlusion);
    }

    /// Flushes and plays any pending mob sound effects (ambient vocalizations, hurt cries, death sounds)
    /// spatialized in 3D with acoustic voxel occlusion.
    pub fn process_mob_sounds(&mut self) {
        for i in 0..self.mob_manager.mobs.len() {
            if let Some(snd) = self.mob_manager.mobs[i].pending_sound.take() {
                let pos = self.mob_manager.mobs[i].center_pos();
                self.play_world_sound_varied(snd, pos, 0.90);
            }
        }
    }
}
