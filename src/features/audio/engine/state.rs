//! Główny silnik audio (`AudioEngine`): inicjalizacja banku dźwięków, wątek miksera / tryb synchroniczny oraz sterowanie głośnością.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::JoinHandle;

use crate::core::math::Vec3;
use crate::features::audio::mixer::AudioMixer;
use crate::features::audio::ring_buffer::{AudioCommand, SpscCommandQueue};
use crate::features::audio::sound::{SoundCategory, SoundId, SoundSample};
use crate::features::audio::spatial::AudioListener;
use crate::features::audio::synth::synthesize_sound;
use crate::platform::audio::{AudioBackend, create_default_backend};
use crate::platform::capabilities;

const ALL_SYNTH_SOUNDS: [SoundId; 79] = [
    SoundId::StepGrass, SoundId::StepDirt, SoundId::StepStone, SoundId::StepWood,
    SoundId::StepSand, SoundId::StepGravel, SoundId::StepWater,
    SoundId::DigGrass, SoundId::DigDirt, SoundId::DigStone, SoundId::DigWood, SoundId::DigSand,
    SoundId::BreakGrass, SoundId::BreakDirt, SoundId::BreakStone, SoundId::BreakWood,
    SoundId::BreakGlass, SoundId::BreakSand,
    SoundId::PlaceGrass, SoundId::PlaceDirt, SoundId::PlaceStone, SoundId::PlaceWood,
    SoundId::PlaceSand, SoundId::PlaceGeneric,
    SoundId::PlayerJump, SoundId::PlayerLand, SoundId::PlayerHurt, SoundId::PlayerSplash,
    SoundId::PlayerEat,
    SoundId::ItemPop, SoundId::UiClick, SoundId::UiHover, SoundId::ChestOpen, SoundId::ChestClose,
    SoundId::DoorOpen, SoundId::DoorClose, SoundId::Explosion,
    SoundId::PigIdle, SoundId::PigHurt, SoundId::PigDeath,
    SoundId::CowIdle, SoundId::CowHurt, SoundId::CowDeath,
    SoundId::ChickenIdle, SoundId::ChickenHurt, SoundId::ChickenDeath,
    SoundId::ZombieIdle, SoundId::ZombieHurt, SoundId::ZombieDeath,
    SoundId::SkeletonIdle, SoundId::SkeletonHurt, SoundId::SkeletonDeath,
    SoundId::ReaperHiss, SoundId::ReaperHurt, SoundId::ReaperDeath,
    SoundId::SheepIdle, SoundId::SheepHurt, SoundId::SheepDeath, SoundId::SheepShear,
    SoundId::VillagerIdle, SoundId::VillagerHurt, SoundId::VillagerDeath,
    SoundId::VillagerTrade, SoundId::VillagerNo,
    SoundId::IronGolemIdle, SoundId::IronGolemHurt, SoundId::IronGolemDeath,
    SoundId::IronGolemAttack,
    SoundId::AmbientWind, SoundId::AmbientCave, SoundId::AmbientWater, SoundId::AmbientLeaves,
    SoundId::AmbientCaveRocks, SoundId::StepWoodCreak, SoundId::AmbientDesertSand,
    SoundId::Rain, SoundId::ThunderClose, SoundId::ThunderDistant,
    SoundId::ToolBreak,
];

pub struct AudioEngine {
    pub(super) soundbank: HashMap<SoundId, SoundSample>,
    pub(super) listener: AudioListener,
    pub(super) sample_rate: u32,
    pub(super) backend_name: &'static str,
    pub(super) is_active: bool,
    pub(super) queue: Option<Arc<std::sync::Mutex<SpscCommandQueue>>>,
    pub(super) worker_handle: Option<JoinHandle<()>>,
    pub(super) worker_running: Arc<AtomicBool>,
    pub(super) sync_mixer: Option<AudioMixer>,
    pub(super) sync_backend: Option<Box<dyn AudioBackend>>,
    pub(super) footstep_dist: f32,
    pub(super) ambient_wind_timer: f32,
    pub(super) ambient_cave_timer: f32,
    pub(super) ambient_water_timer: f32,
    pub(super) ambient_leaves_timer: f32,
    pub(super) ambient_rocks_timer: f32,
    pub(super) ambient_sand_timer: f32,
    pub(super) ambient_rain_timer: f32,
    pub(super) rng_seed: u32,
    pub(super) reverb_wet: f32,
    pub(super) reverb_decay_seconds: f32,
    pub(super) master_vol: f32,
    pub(super) sfx_vol: f32,
    pub(super) music_vol: f32,
    pub(super) ambient_vol: f32,
    pub(super) ui_vol: f32,
    pub(super) is_muted: bool,
    pub(super) music_playing: Arc<AtomicBool>,
    pub(super) music_progress: Arc<AtomicU32>,
}

fn apply_command_to_mixer(
    cmd: AudioCommand,
    mixer: &mut AudioMixer,
    soundbank: &HashMap<SoundId, SoundSample>,
) {
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
            if let Some(sample) = soundbank.get(&sound_id) {
                if spatial {
                    mixer.play_spatial_sample(
                        sample,
                        sound_id.category(),
                        volume,
                        pitch,
                        position,
                        min_dist,
                        max_dist,
                        rolloff,
                        occlusion,
                        looping,
                    );
                } else {
                    let angle = (pan.clamp(-1.0, 1.0) + 1.0) * (std::f32::consts::PI * 0.25);
                    mixer.play_sample(
                        sample,
                        sound_id.category(),
                        volume,
                        pitch,
                        angle.cos(),
                        angle.sin(),
                        1.0,
                        looping,
                    );
                }
            }
        }
        AudioCommand::PlayMusicSample {
            sample,
            volume,
            looping,
        } => {
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
            mixer.update_spatial_listener(AudioListener::new(position, forward, right, up));
        }
        AudioCommand::SetCategoryVolume { category, volume } => {
            mixer.set_category_volume(category, volume);
        }
        AudioCommand::SetReverb { wet, decay_seconds } => {
            mixer.set_reverb(wet, decay_seconds);
        }
        AudioCommand::StopAll { fade_out_seconds } => {
            mixer.stop_all(fade_out_seconds);
        }
    }
}

impl AudioEngine {
    pub fn new() -> Self {
        let sample_rate = 48000;
        let mut soundbank = HashMap::with_capacity(ALL_SYNTH_SOUNDS.len());
        for &id in &ALL_SYNTH_SOUNDS {
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

        let caps = capabilities();
        // DLACZEGO: Na PS Vita (gdzie VitaAudioBackend posiada już własny wątek SceAudioOut) oraz na WASM
        // mikser jest wywoływany synchronicznie z głównej pętli gry (`tick_sync_audio`) bez dodatkowego wątku miksującego.
        let use_sync_mixer = caps.is_wasm || caps.is_vita;
        let (queue, worker_handle, sync_mixer, sync_backend) = if use_sync_mixer {
            (None, None, Some(AudioMixer::new(sample_rate)), Some(backend))
        } else {
            let running = Arc::clone(&worker_running);
            let music_playing_worker = Arc::clone(&music_playing);
            let music_progress_worker = Arc::clone(&music_progress);
            let bank_clone = soundbank.clone();
            let queue = Arc::new(std::sync::Mutex::new(SpscCommandQueue::new()));
            let queue_worker = Arc::clone(&queue);

            let handle = std::thread::Builder::new()
                .name("rustcraft-audio".to_string())
                .spawn(move || {
                    let mut mixer = AudioMixer::new(sample_rate);
                    let mut audio_backend = backend;
                    let mut scratch_buffer = [0.0f32; 2048];
                    let is_null_sink = !audio_backend.is_active()
                        || audio_backend.name() == "Null Audio Sink";

                    while running.load(Ordering::Relaxed) {
                        if let Ok(mut q) = queue_worker.try_lock() {
                            while let Some(cmd) = q.pop() {
                                apply_command_to_mixer(cmd, &mut mixer, &bank_clone);
                            }
                        }

                        if mixer.has_pending_audio() {
                            mixer.mix_interleaved_stereo(&mut scratch_buffer);
                            audio_backend.write_samples(&scratch_buffer);
                            let is_music = mixer.is_category_playing(SoundCategory::Music);
                            music_playing_worker.store(is_music, Ordering::Relaxed);
                            if is_music {
                                let p = (mixer
                                    .category_playback_progress(SoundCategory::Music)
                                    .unwrap_or(0.0)
                                    * 10000.0) as u32;
                                music_progress_worker.store(p, Ordering::Relaxed);
                            } else {
                                music_progress_worker.store(0, Ordering::Relaxed);
                            }

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
                    let sm = AudioMixer::new(sample_rate);
                    let mut sb = create_default_backend();
                    let _ = sb.init(sample_rate, 2);
                    (None, None, Some(sm), Some(sb))
                }
            }
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
            ambient_rain_timer: 0.5,
            rng_seed: 0x9E3779B9,
            reverb_wet: 0.0,
            reverb_decay_seconds: 0.35,
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

    #[inline]
    pub(super) fn rand_f32(&mut self, min: f32, max: f32) -> f32 {
        let mut x = self.rng_seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_seed = x;
        let frac = (x as f32) / 4294967295.0;
        min + frac * (max - min)
    }

    pub fn update_listener(&mut self, position: Vec3, forward: Vec3, right: Vec3, up: Vec3) {
        self.listener = AudioListener::new(position, forward, right, up);
        self.dispatch_cmd(AudioCommand::UpdateListener {
            position,
            forward,
            right,
            up,
        });
    }

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

    pub fn play_sound_3d(&mut self, sound_id: SoundId, position: Vec3, volume: f32, pitch: f32) {
        self.play_sound_3d_occluded(sound_id, position, volume, pitch, 0.0);
    }

    pub fn play_sound_3d_varied(&mut self, sound_id: SoundId, position: Vec3, volume: f32) {
        self.play_sound_3d_varied_occluded(sound_id, position, volume, 0.0);
    }

    pub fn play_sound_3d_varied_occluded(
        &mut self,
        sound_id: SoundId,
        position: Vec3,
        volume: f32,
        occlusion: f32,
    ) {
        let pitch = self.rand_f32(0.92, 1.08);
        self.play_sound_3d_occluded(sound_id, position, volume, pitch, occlusion);
    }

    pub(super) fn dispatch_cmd(&mut self, cmd: AudioCommand) {
        if let Some(queue) = &self.queue {
            if let Ok(mut q) = queue.lock() {
                q.push(cmd);
            }
        } else if let Some(mixer) = &mut self.sync_mixer {
            apply_command_to_mixer(cmd, mixer, &self.soundbank);
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
