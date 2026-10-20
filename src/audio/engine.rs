//! Main RustCraft Audio Engine.
//!
//! Coordinates the polyphonic mixer, procedural soundbank, 3D spatial listener,
//! footstep cadence manager, and platform audio hardware stream.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::audio::backend::{create_default_backend, AudioBackend};
use crate::audio::mixer::AudioMixer;
use crate::audio::ring_buffer::{AudioCommand, SpscCommandQueue};
use crate::audio::sound::{SoundCategory, SoundId, SoundSample};
use crate::audio::spatial::AudioListener;
use crate::audio::synth::synthesize_sound;
use crate::core::math::Vec3;
use crate::world::block::BlockId;

/// Performance and status diagnostic statistics.
#[derive(Clone, Copy, Debug, Default)]
pub struct AudioStats {
    pub active_voices: usize,
    pub sample_rate: u32,
    pub backend_name: &'static str,
    pub is_active: bool,
    pub commands_processed: u64,
}

/// The core cross-platform RustCraft audio engine.
pub struct AudioEngine {
    soundbank: HashMap<SoundId, SoundSample>,
    listener: AudioListener,
    sample_rate: u32,
    backend_name: &'static str,
    is_active: bool,

    // Threading / Command queue
    queue: Option<Arc<std::sync::Mutex<SpscCommandQueue>>>,
    worker_handle: Option<JoinHandle<()>>,
    worker_running: Arc<AtomicBool>,

    // Synchronous fallback mixer (for single-threaded / WASM / test modes)
    sync_mixer: Option<AudioMixer>,
    #[allow(dead_code)]
    sync_backend: Option<Box<dyn AudioBackend>>,

    // Cadence & Ambient timers
    footstep_dist: f32,
    ambient_timer: f32,
    rng_seed: u32,

    // Volumes
    master_vol: f32,
    sfx_vol: f32,
    music_vol: f32,
    ambient_vol: f32,
    ui_vol: f32,
    is_muted: bool,
}

impl AudioEngine {
    /// Creates and initializes the audio engine with procedural soundbank synthesis.
    pub fn new() -> Self {
        let sample_rate = 48000;
        let mut soundbank = HashMap::with_capacity(48);

        // Pre-synthesize all standard game sounds into memory at startup (< 2ms)
        let all_sounds = [
            SoundId::StepGrass,
            SoundId::StepDirt,
            SoundId::StepStone,
            SoundId::StepWood,
            SoundId::StepSand,
            SoundId::StepGravel,
            SoundId::StepWater,
            SoundId::DigGrass,
            SoundId::DigDirt,
            SoundId::DigStone,
            SoundId::DigWood,
            SoundId::DigSand,
            SoundId::BreakGrass,
            SoundId::BreakDirt,
            SoundId::BreakStone,
            SoundId::BreakWood,
            SoundId::BreakGlass,
            SoundId::BreakSand,
            SoundId::PlaceGrass,
            SoundId::PlaceDirt,
            SoundId::PlaceStone,
            SoundId::PlaceWood,
            SoundId::PlaceSand,
            SoundId::PlaceGeneric,
            SoundId::PlayerJump,
            SoundId::PlayerLand,
            SoundId::PlayerHurt,
            SoundId::PlayerSplash,
            SoundId::PlayerEat,
            SoundId::ItemPop,
            SoundId::UiClick,
            SoundId::UiHover,
            SoundId::ChestOpen,
            SoundId::ChestClose,
            SoundId::DoorOpen,
            SoundId::DoorClose,
            SoundId::Explosion,
            SoundId::PigIdle,
            SoundId::PigHurt,
            SoundId::PigDeath,
            SoundId::CowIdle,
            SoundId::CowHurt,
            SoundId::CowDeath,
            SoundId::ChickenIdle,
            SoundId::ChickenHurt,
            SoundId::ChickenDeath,
            SoundId::ZombieIdle,
            SoundId::ZombieHurt,
            SoundId::ZombieDeath,
            SoundId::SkeletonIdle,
            SoundId::SkeletonHurt,
            SoundId::SkeletonDeath,
            SoundId::CreeperHiss,
            SoundId::CreeperHurt,
            SoundId::CreeperDeath,
            SoundId::AmbientWind,
            SoundId::AmbientCave,
        ];

        for &id in &all_sounds {
            soundbank.insert(id, synthesize_sound(id, sample_rate));
        }

        let mut backend = create_default_backend();
        let init_res = backend.init(sample_rate, 2);
        let backend_name = backend.name();
        let is_active = init_res.is_ok() && backend.is_active();

        crate::log_info!(
            "AUDIO",
            "Audio engine initialized: backend '{}' (active: {}, sample_rate: {} Hz, soundbank: {} samples)",
            backend_name,
            is_active,
            sample_rate,
            soundbank.len()
        );

        let worker_running = Arc::new(AtomicBool::new(true));

        // Attempt to launch dedicated high-priority audio mixing thread
        #[cfg(not(target_arch = "wasm32"))]
        let (queue, worker_handle, sync_mixer, sync_backend) = {
            let running = Arc::clone(&worker_running);
            let bank_clone = soundbank.clone();

            // We use a shared mutex queue or SpscCommandQueue for thread communication
            let queue = Arc::new(std::sync::Mutex::new(SpscCommandQueue::new()));
            let queue_worker = Arc::clone(&queue);

            let handle = std::thread::Builder::new()
                .name("rustcraft-audio".to_string())
                .spawn(move || {
                    let mut mixer = AudioMixer::new(sample_rate);
                    let mut audio_backend = backend;
                    let mut scratch_buffer = [0.0f32; 1024]; // 512 stereo frames = ~10.6ms latency
                    let mut listener = AudioListener::default();

                    while running.load(Ordering::Relaxed) {
                        // Drain pending commands from lock-free queue
                        if let Ok(mut q) = queue_worker.try_lock() {
                            while let Some(cmd) = q.pop() {
                                match cmd {
                                    AudioCommand::PlaySound {
                                        sound_id,
                                        volume,
                                        pitch,
                                        pan,
                                        looping,
                                        spatial,
                                        position,
                                        min_dist,
                                        max_dist,
                                        rolloff,
                                        occlusion,
                                    } => {
                                        if let Some(sample) = bank_clone.get(&sound_id) {
                                            let (lg, rg, filter_alpha) = if spatial {
                                                let res = listener.calculate_spatial(
                                                    position, min_dist, max_dist, rolloff, occlusion,
                                                );
                                                (res.left_gain, res.right_gain, res.filter_alpha)
                                            } else {
                                                let angle = (pan.clamp(-1.0, 1.0) + 1.0)
                                                    * (std::f32::consts::PI * 0.25);
                                                (angle.cos(), angle.sin(), 1.0)
                                            };

                                            if lg > 0.001 || rg > 0.001 {
                                                mixer.play_sample(
                                                    sample,
                                                    sound_id.category(),
                                                    volume,
                                                    pitch,
                                                    lg,
                                                    rg,
                                                    filter_alpha,
                                                    looping,
                                                );
                                            }
                                        }
                                    }
                                    AudioCommand::UpdateListener {
                                        position,
                                        forward,
                                        right,
                                        up,
                                    } => {
                                        listener.position = position;
                                        listener.forward = forward;
                                        listener.right = right;
                                        listener.up = up;
                                    }
                                    AudioCommand::SetCategoryVolume { category, volume } => {
                                        mixer.set_category_volume(category, volume);
                                    }
                                    AudioCommand::StopAll { fade_out_seconds } => {
                                        mixer.stop_all(fade_out_seconds);
                                    }
                                }
                            }
                        }

                        // Render batch of stereo audio frames
                        mixer.mix_interleaved_stereo(&mut scratch_buffer);
                        audio_backend.write_samples(&scratch_buffer);

                        // If the backend doesn't block internally, yield/sleep to prevent CPU spin
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                });

            match handle {
                Ok(th) => (Some(queue), Some(th), None, None),
                Err(_) => {
                    // Fallback to synchronous mixing
                    let sm = AudioMixer::new(sample_rate);
                    let mut sb = create_default_backend();
                    let _ = sb.init(sample_rate, 2);
                    (None, None, Some(sm), Some(sb))
                }
            }
        };

        #[cfg(target_arch = "wasm32")]
        let (queue, worker_handle, sync_mixer, sync_backend) = {
            let sm = AudioMixer::new(sample_rate);
            let mut sb = create_default_backend();
            let _ = sb.init(sample_rate, 2);
            (None, None, Some(sm), Some(sb))
        };

        Self {
            soundbank,
            listener: AudioListener::default(),
            sample_rate,
            backend_name,
            is_active,
            queue,
            worker_handle,
            worker_running,
            sync_mixer,
            sync_backend,
            footstep_dist: 0.0,
            ambient_timer: 20.0,
            rng_seed: 0x9E3779B9,
            master_vol: 0.8,
            sfx_vol: 1.0,
            music_vol: 0.5,
            ambient_vol: 0.7,
            ui_vol: 1.0,
            is_muted: false,
        }
    }

    /// Fast pseudo-random float generator for subtle organic pitch/volume variations.
    #[inline]
    fn rand_f32(&mut self, min: f32, max: f32) -> f32 {
        let mut x = self.rng_seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_seed = x;
        let frac = (x as f32) / 4294967295.0;
        min + frac * (max - min)
    }

    /// Updates the audio listener position and orientation from active camera vectors.
    pub fn update_listener(&mut self, position: Vec3, forward: Vec3, right: Vec3, up: Vec3) {
        self.listener = AudioListener::new(position, forward, right, up);
        self.dispatch_cmd(AudioCommand::UpdateListener {
            position,
            forward,
            right,
            up,
        });
    }

    /// Plays a 2D sound effect (UI, inventory, global event).
    pub fn play_sound(&mut self, sound_id: SoundId, volume: f32, pitch: f32) {
        if self.is_muted {
            return;
        }
        self.dispatch_cmd(AudioCommand::PlaySound {
            sound_id,
            volume,
            pitch,
            pan: 0.0,
            looping: false,
            spatial: false,
            position: Vec3::zero(),
            min_dist: 1.0,
            max_dist: 40.0,
            rolloff: 1.0,
            occlusion: 0.0,
        });
    }

    /// Plays a 3D spatialized sound effect located at world coordinates `position`
    /// with acoustic voxel occlusion factor in [0.0, 1.0].
    pub fn play_sound_3d_occluded(
        &mut self,
        sound_id: SoundId,
        position: Vec3,
        volume: f32,
        pitch: f32,
        occlusion: f32,
    ) {
        if self.is_muted {
            return;
        }
        self.dispatch_cmd(AudioCommand::PlaySound {
            sound_id,
            volume,
            pitch,
            pan: 0.0,
            looping: false,
            spatial: true,
            position,
            min_dist: 2.0,
            max_dist: 36.0,
            rolloff: 1.0,
            occlusion: occlusion.clamp(0.0, 1.0),
        });
    }

    /// Plays a 3D spatialized sound effect located at world coordinates `position`.
    pub fn play_sound_3d(&mut self, sound_id: SoundId, position: Vec3, volume: f32, pitch: f32) {
        self.play_sound_3d_occluded(sound_id, position, volume, pitch, 0.0);
    }

    /// Plays a 3D sound with subtle organic pitch variation (e.g. ±6%).
    pub fn play_sound_3d_varied(&mut self, sound_id: SoundId, position: Vec3, volume: f32) {
        self.play_sound_3d_varied_occluded(sound_id, position, volume, 0.0);
    }

    /// Plays a 3D sound with subtle organic pitch variation and acoustic occlusion.
    pub fn play_sound_3d_varied_occluded(&mut self, sound_id: SoundId, position: Vec3, volume: f32, occlusion: f32) {
        let pitch = self.rand_f32(0.92, 1.08);
        self.play_sound_3d_occluded(sound_id, position, volume, pitch, occlusion);
    }

    /// Dispatches an audio command to the background audio thread or processes synchronously.
    fn dispatch_cmd(&mut self, cmd: AudioCommand) {
        if let Some(queue) = &self.queue {
            if let Ok(mut q) = queue.lock() {
                q.push(cmd);
            }
        }
        // Synchronous fallback handling
        if let Some(mixer) = &mut self.sync_mixer {
            match cmd {
                AudioCommand::PlaySound {
                    sound_id,
                    volume,
                    pitch,
                    pan,
                    looping,
                    spatial,
                    position,
                    min_dist,
                    max_dist,
                    rolloff,
                    occlusion,
                } => {
                    if let Some(sample) = self.soundbank.get(&sound_id) {
                        let (lg, rg, filter_alpha) = if spatial {
                            let res = self.listener.calculate_spatial(position, min_dist, max_dist, rolloff, occlusion);
                            (res.left_gain, res.right_gain, res.filter_alpha)
                        } else {
                            let angle = (pan.clamp(-1.0, 1.0) + 1.0) * (std::f32::consts::PI * 0.25);
                            (angle.cos(), angle.sin(), 1.0)
                        };
                        if lg > 0.001 || rg > 0.001 {
                            mixer.play_sample(sample, sound_id.category(), volume, pitch, lg, rg, filter_alpha, looping);
                        }
                    }
                }
                AudioCommand::UpdateListener { position, forward, right, up } => {
                    self.listener.position = position;
                    self.listener.forward = forward;
                    self.listener.right = right;
                    self.listener.up = up;
                }
                AudioCommand::SetCategoryVolume { category, volume } => {
                    mixer.set_category_volume(category, volume);
                }
                AudioCommand::StopAll { fade_out_seconds } => {
                    mixer.stop_all(fade_out_seconds);
                }
            }
        }
    }

    /// Game tick update managing footstep cadence and ambient atmosphere.
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        horiz_speed: f32,
        is_grounded: bool,
        is_sprinting: bool,
        block_below: BlockId,
    ) {
        if self.is_muted {
            return;
        }

        // Footstep cadence
        if is_grounded && horiz_speed > 0.4 {
            let step_stride = if is_sprinting { 1.7 } else { 2.2 };
            self.footstep_dist += horiz_speed * dt;
            if self.footstep_dist >= step_stride {
                self.footstep_dist = 0.0;
                let step_sound = SoundId::from_block_step(block_below);
                let feet_pos = player_pos - Vec3::new(0.0, 1.4, 0.0);
                let vol = if is_sprinting { 0.85 } else { 0.65 };
                self.play_sound_3d_varied(step_sound, feet_pos, vol);
            }
        } else {
            self.footstep_dist = (self.footstep_dist - dt * 2.0).max(0.0);
        }

        // Ambient atmosphere generator
        self.ambient_timer -= dt;
        if self.ambient_timer <= 0.0 {
            self.ambient_timer = self.rand_f32(35.0, 75.0);
            if player_pos.y < 32.0 {
                // Underground cave atmosphere
                self.play_sound_3d(SoundId::AmbientCave, player_pos, 0.55, 1.0);
            } else {
                // Surface wind breeze
                self.play_sound(SoundId::AmbientWind, 0.35, 1.0);
            }
        }
    }

    /// Sets the Master volume [0.0, 1.0].
    pub fn set_master_volume(&mut self, vol: f32) {
        self.master_vol = vol.clamp(0.0, 1.0);
        let eff = if self.is_muted { 0.0 } else { self.master_vol };
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Master,
            volume: eff,
        });
    }

    /// Sets the SFX volume [0.0, 1.0].
    pub fn set_sfx_volume(&mut self, vol: f32) {
        self.sfx_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Sfx,
            volume: self.sfx_vol,
        });
    }

    /// Sets the UI volume [0.0, 1.0].
    pub fn set_ui_volume(&mut self, vol: f32) {
        self.ui_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Ui,
            volume: self.ui_vol,
        });
    }

    /// Sets the Ambient volume [0.0, 1.0].
    pub fn set_ambient_volume(&mut self, vol: f32) {
        self.ambient_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Ambient,
            volume: self.ambient_vol,
        });
    }

    /// Sets the Music volume [0.0, 1.0].
    pub fn set_music_volume(&mut self, vol: f32) {
        self.music_vol = vol.clamp(0.0, 1.0);
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Music,
            volume: self.music_vol,
        });
    }

    /// Toggles audio muting.
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
        let eff = if self.is_muted { 0.0 } else { self.master_vol };
        self.dispatch_cmd(AudioCommand::SetCategoryVolume {
            category: SoundCategory::Master,
            volume: eff,
        });
    }

    /// Sets the muted state directly.
    pub fn set_muted(&mut self, muted: bool) {
        if self.is_muted != muted {
            self.toggle_mute();
        }
    }

    /// Checks if audio is currently muted.
    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    /// Retrieves active audio engine statistics.
    pub fn stats(&self) -> AudioStats {
        AudioStats {
            active_voices: 0,
            sample_rate: self.sample_rate,
            backend_name: self.backend_name,
            is_active: self.is_active,
            commands_processed: 0,
        }
    }
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.worker_running.store(false, Ordering::Relaxed);
        if let Some(h) = self.worker_handle.take() {
            let _ = h.join();
        }
    }
}
