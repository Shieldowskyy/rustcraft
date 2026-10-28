//! Main RustCraft Audio Engine.
//!
//! Coordinates the polyphonic mixer, procedural soundbank, 3D spatial listener,
//! footstep cadence manager, and platform audio hardware stream.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
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

/// Contextual information about the player's immediate surrounding environment,
/// used to trigger realistic ambient soundscapes (wind, water, leaves, cave debris, desert sand).
#[derive(Clone, Copy, Debug)]
pub struct AmbientContext {
    /// True if the player is underground in a cave or deep mine.
    pub is_underground: bool,
    /// True if the sky is directly visible above the player.
    pub sky_exposed: bool,
    /// True if there is water nearby (river, ocean, lake, or submerged).
    pub near_water: bool,
    /// Approximate 3D position of the nearest water body, if known.
    pub water_pos: Option<Vec3>,
    /// True if tree leaves / foliage are nearby.
    pub near_leaves: bool,
    /// Approximate 3D position of nearby foliage.
    pub leaves_pos: Option<Vec3>,
    /// True if the player is in an arid / desert biome or on sand dunes.
    pub is_desert: bool,
    /// Approximate position of nearby sand dunes.
    pub sand_pos: Option<Vec3>,
}

impl Default for AmbientContext {
    fn default() -> Self {
        Self {
            is_underground: false,
            sky_exposed: true,
            near_water: false,
            water_pos: None,
            near_leaves: false,
            leaves_pos: None,
            is_desert: false,
            sand_pos: None,
        }
    }
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
    ambient_wind_timer: f32,
    ambient_cave_timer: f32,
    ambient_water_timer: f32,
    ambient_leaves_timer: f32,
    ambient_rocks_timer: f32,
    ambient_sand_timer: f32,
    rng_seed: u32,

    // Volumes
    master_vol: f32,
    sfx_vol: f32,
    music_vol: f32,
    ambient_vol: f32,
    ui_vol: f32,
    is_muted: bool,
    music_playing: Arc<AtomicBool>,
    music_progress: Arc<AtomicU32>,
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
            SoundId::SheepIdle,
            SoundId::SheepHurt,
            SoundId::SheepDeath,
            SoundId::SheepShear,
            SoundId::VillagerIdle,
            SoundId::VillagerHurt,
            SoundId::VillagerDeath,
            SoundId::VillagerTrade,
            SoundId::VillagerNo,
            SoundId::IronGolemIdle,
            SoundId::IronGolemHurt,
            SoundId::IronGolemDeath,
            SoundId::IronGolemAttack,
            SoundId::AmbientWind,
            SoundId::AmbientCave,
            SoundId::AmbientWater,
            SoundId::AmbientLeaves,
            SoundId::AmbientCaveRocks,
            SoundId::StepWoodCreak,
            SoundId::AmbientDesertSand,
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
        let music_playing = Arc::new(AtomicBool::new(false));
        let music_progress = Arc::new(AtomicU32::new(0));

        // Attempt to launch dedicated high-priority audio mixing thread
        #[cfg(not(target_arch = "wasm32"))]
        let (queue, worker_handle, sync_mixer, sync_backend) = {
            let running = Arc::clone(&worker_running);
            let music_playing_worker = Arc::clone(&music_playing);
            let music_progress_worker = Arc::clone(&music_progress);
            let bank_clone = soundbank.clone();

            // We use a shared mutex queue or SpscCommandQueue for thread communication
            let queue = Arc::new(std::sync::Mutex::new(SpscCommandQueue::new()));
            let queue_worker = Arc::clone(&queue);

            let handle = std::thread::Builder::new()
                .name("rustcraft-audio".to_string())
                .spawn(move || {
                    let mut mixer = AudioMixer::new(sample_rate);
                    let mut audio_backend = backend;
                    let mut scratch_buffer = [0.0f32; 2048]; // 1024 stereo frames = ~21.3ms latency
                    let mut listener = AudioListener::default();
                    let is_null_sink = !audio_backend.is_active() || audio_backend.name() == "Null Audio Sink";

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
                                    AudioCommand::PlayMusicSample { sample, volume, looping } => {
                                        mixer.stop_category(SoundCategory::Music, 1.0);
                                        mixer.play_sample(
                                            &sample,
                                            SoundCategory::Music,
                                            volume,
                                            1.0,
                                            1.0,
                                            1.0,
                                            1.0,
                                            looping,
                                        );
                                    }
                                    AudioCommand::StopMusic { fade_out_seconds } => {
                                        mixer.stop_category(SoundCategory::Music, fade_out_seconds);
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

                        let active_voices = mixer.active_voice_count();
                        if active_voices > 0 {
                            // Render batch of stereo audio frames
                            mixer.mix_interleaved_stereo(&mut scratch_buffer);
                            audio_backend.write_samples(&scratch_buffer);
                            let is_music = mixer.is_category_playing(SoundCategory::Music);
                            music_playing_worker.store(is_music, Ordering::Relaxed);
                            if is_music {
                                let p = (mixer.category_playback_progress(SoundCategory::Music).unwrap_or(0.0) * 10000.0) as u32;
                                music_progress_worker.store(p, Ordering::Relaxed);
                            } else {
                                music_progress_worker.store(0, Ordering::Relaxed);
                            }

                            // For non-blocking/null backend, yield to avoid pegging the CPU
                            if is_null_sink {
                                std::thread::sleep(std::time::Duration::from_millis(15));
                            }
                        } else {
                            music_playing_worker.store(false, Ordering::Relaxed);
                            music_progress_worker.store(0, Ordering::Relaxed);
                            std::thread::sleep(std::time::Duration::from_millis(10));
                        }
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
            ambient_wind_timer: 6.0,
            ambient_cave_timer: 16.0,
            ambient_water_timer: 3.5,
            ambient_leaves_timer: 5.0,
            ambient_rocks_timer: 9.0,
            ambient_sand_timer: 6.5,
            rng_seed: 0x9E3779B9,
            master_vol: 0.8,
            sfx_vol: 1.0,
            music_vol: 0.5,
            ambient_vol: 0.7,
            ui_vol: 1.0,
            is_muted: false,
            music_playing,
            music_progress,
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
        } else if let Some(mixer) = &mut self.sync_mixer {
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
                AudioCommand::PlayMusicSample { sample, volume, looping } => {
                    mixer.stop_category(SoundCategory::Music, 1.0);
                    mixer.play_sample(&sample, SoundCategory::Music, volume, 1.0, 1.0, 1.0, 1.0, looping);
                }
                AudioCommand::StopMusic { fade_out_seconds } => {
                    mixer.stop_category(SoundCategory::Music, fade_out_seconds);
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
        ambient_ctx: AmbientContext,
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

                // Wood creaking when walking on wooden surfaces
                if block_below.is_wood()
                    || matches!(
                        block_below,
                        BlockId::Wood
                            | BlockId::Planks
                            | BlockId::CraftingTable
                            | BlockId::Chest
                            | BlockId::Door
                            | BlockId::DoorOpen
                    )
                {
                    if self.rand_f32(0.0, 1.0) < 0.55 {
                        let creak_vol = self.rand_f32(0.40, 0.65);
                        let creak_pitch = self.rand_f32(0.88, 1.12);
                        self.play_sound_3d(SoundId::StepWoodCreak, feet_pos, creak_vol, creak_pitch);
                    }
                }
            }
        } else {
            self.footstep_dist = (self.footstep_dist - dt * 2.0).max(0.0);
        }

        // Ambient atmosphere system
        // 1. Underground Cave Ambience (Drones & Falling Rocks)
        if ambient_ctx.is_underground {
            // Echoing rock & pebble tumble in caverns
            self.ambient_rocks_timer -= dt;
            if self.ambient_rocks_timer <= 0.0 {
                self.ambient_rocks_timer = self.rand_f32(11.0, 24.0);
                let offset_x = self.rand_f32(-8.0, 8.0);
                let offset_y = self.rand_f32(1.0, 6.0);
                let offset_z = self.rand_f32(-8.0, 8.0);
                let rock_pos = player_pos + Vec3::new(offset_x, offset_y, offset_z);
                let rock_vol = self.rand_f32(0.50, 0.75);
                let rock_pitch = self.rand_f32(0.92, 1.08);
                self.play_sound_3d(SoundId::AmbientCaveRocks, rock_pos, rock_vol, rock_pitch);
            }

            // Deep cave drone
            self.ambient_cave_timer -= dt;
            if self.ambient_cave_timer <= 0.0 {
                self.ambient_cave_timer = self.rand_f32(30.0, 65.0);
                self.play_sound_3d(SoundId::AmbientCave, player_pos, 0.55, 1.0);
            }
        } else {
            // 2. Surface Wind Breeze (Open air / high elevations)
            self.ambient_wind_timer -= dt;
            if self.ambient_wind_timer <= 0.0 {
                let wind_interval = if player_pos.y > 80.0 {
                    self.rand_f32(10.0, 18.0)
                } else {
                    self.rand_f32(16.0, 32.0)
                };
                self.ambient_wind_timer = wind_interval;
                let wind_vol = if player_pos.y > 85.0 { 0.50 } else { 0.38 };
                let wind_pitch = self.rand_f32(0.95, 1.05);
                self.play_sound(SoundId::AmbientWind, wind_vol, wind_pitch);
            }
        }

        // 3. Water Ambience (Gentle bubbling brook & shore lapping)
        if ambient_ctx.near_water {
            self.ambient_water_timer -= dt;
            if self.ambient_water_timer <= 0.0 {
                self.ambient_water_timer = self.rand_f32(6.0, 13.0);
                let target_pos = ambient_ctx.water_pos.unwrap_or(player_pos);
                let water_vol = self.rand_f32(0.45, 0.65);
                let water_pitch = self.rand_f32(0.95, 1.06);
                self.play_sound_3d(SoundId::AmbientWater, target_pos, water_vol, water_pitch);
            }
        } else {
            self.ambient_water_timer = (self.ambient_water_timer - dt * 0.5).max(1.5);
        }

        // 4. Tree Leaves Rustling Ambience
        if ambient_ctx.near_leaves && !ambient_ctx.is_underground {
            self.ambient_leaves_timer -= dt;
            if self.ambient_leaves_timer <= 0.0 {
                self.ambient_leaves_timer = self.rand_f32(7.0, 16.0);
                let target_pos = ambient_ctx.leaves_pos.unwrap_or(player_pos + Vec3::new(0.0, 2.5, 0.0));
                let leaves_vol = self.rand_f32(0.40, 0.60);
                let leaves_pitch = self.rand_f32(0.94, 1.06);
                self.play_sound_3d(SoundId::AmbientLeaves, target_pos, leaves_vol, leaves_pitch);
            }
        } else {
            self.ambient_leaves_timer = (self.ambient_leaves_timer - dt * 0.5).max(2.0);
        }

        // 5. Desert Sand Cascades / Trickling
        if (ambient_ctx.is_desert || ambient_ctx.sand_pos.is_some()) && !ambient_ctx.is_underground {
            self.ambient_sand_timer -= dt;
            if self.ambient_sand_timer <= 0.0 {
                self.ambient_sand_timer = self.rand_f32(9.0, 20.0);
                let target_pos = ambient_ctx.sand_pos.unwrap_or(player_pos - Vec3::new(0.0, 1.0, 0.0));
                let sand_vol = self.rand_f32(0.42, 0.62);
                let sand_pitch = self.rand_f32(0.95, 1.05);
                self.play_sound_3d(SoundId::AmbientDesertSand, target_pos, sand_vol, sand_pitch);
            }
        } else {
            self.ambient_sand_timer = (self.ambient_sand_timer - dt * 0.5).max(2.5);
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

    /// Plays a music soundtrack sample.
    pub fn play_music(&mut self, sample: Arc<SoundSample>, volume: f32, looping: bool) {
        if self.is_muted {
            return;
        }
        self.dispatch_cmd(AudioCommand::PlayMusicSample {
            sample,
            volume,
            looping,
        });
    }

    /// Stops any playing music with a smooth fade-out.
    pub fn stop_music(&mut self, fade_out_seconds: f32) {
        self.dispatch_cmd(AudioCommand::StopMusic { fade_out_seconds });
    }

    /// Returns true if a soundtrack is actively playing in the background.
    pub fn is_music_playing(&self) -> bool {
        if let Some(sync_mixer) = &self.sync_mixer {
            sync_mixer.is_category_playing(SoundCategory::Music)
        } else {
            self.music_playing.load(Ordering::Relaxed)
        }
    }

    /// Returns the normalized playback progress [0.0, 1.0] of current background music if playing.
    pub fn music_playback_progress(&self) -> Option<f32> {
        if let Some(sync_mixer) = &self.sync_mixer {
            sync_mixer.category_playback_progress(SoundCategory::Music)
        } else if self.is_music_playing() {
            Some((self.music_progress.load(Ordering::Relaxed) as f32 / 10000.0).clamp(0.0, 1.0))
        } else {
            None
        }
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
