//! Kadencja kroków gracza, generatory atmosfery otoczenia (jaskinie, wiatr, woda, liście, deszcz) i tick synchroniczny.

use crate::core::math::Vec3;
use crate::features::audio::ring_buffer::AudioCommand;
use crate::features::audio::sound::SoundId;
use crate::features::blocks::BlockId;

use super::state::AudioEngine;
use super::types::AmbientContext;

impl AudioEngine {
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

        let target_wet = ambient_ctx.reverb_wet.clamp(0.0, 0.55);
        let target_decay = ambient_ctx.reverb_decay_seconds.clamp(0.35, 3.5);
        if (target_wet - self.reverb_wet).abs() >= 0.015
            || (target_decay - self.reverb_decay_seconds).abs() >= 0.1
        {
            self.reverb_wet = target_wet;
            self.reverb_decay_seconds = target_decay;
            self.dispatch_cmd(AudioCommand::SetReverb {
                wet: target_wet,
                decay_seconds: target_decay,
            });
        }

        if is_grounded && horiz_speed > 0.4 {
            let step_stride = if is_sprinting { 1.7 } else { 2.2 };
            self.footstep_dist += horiz_speed * dt;
            if self.footstep_dist >= step_stride {
                self.footstep_dist = 0.0;
                let step_sound = SoundId::from_block_step(block_below);
                let feet_pos = player_pos - Vec3::new(0.0, 1.4, 0.0);
                let vol = if is_sprinting { 0.85 } else { 0.65 };
                self.play_sound_3d_varied(step_sound, feet_pos, vol);

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

        if ambient_ctx.is_underground {
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

            self.ambient_cave_timer -= dt;
            if self.ambient_cave_timer <= 0.0 {
                self.ambient_cave_timer = self.rand_f32(30.0, 65.0);
                self.play_sound_3d(SoundId::AmbientCave, player_pos, 0.55, 1.0);
            }
        } else {
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

        if ambient_ctx.near_leaves && !ambient_ctx.is_underground {
            self.ambient_leaves_timer -= dt;
            if self.ambient_leaves_timer <= 0.0 {
                self.ambient_leaves_timer = self.rand_f32(7.0, 16.0);
                let target_pos = ambient_ctx
                    .leaves_pos
                    .unwrap_or(player_pos + Vec3::new(0.0, 2.5, 0.0));
                let leaves_vol = self.rand_f32(0.40, 0.60);
                let leaves_pitch = self.rand_f32(0.94, 1.06);
                self.play_sound_3d(SoundId::AmbientLeaves, target_pos, leaves_vol, leaves_pitch);
            }
        } else {
            self.ambient_leaves_timer = (self.ambient_leaves_timer - dt * 0.5).max(2.0);
        }

        if (ambient_ctx.is_desert || ambient_ctx.sand_pos.is_some()) && !ambient_ctx.is_underground {
            self.ambient_sand_timer -= dt;
            if self.ambient_sand_timer <= 0.0 {
                self.ambient_sand_timer = self.rand_f32(9.0, 20.0);
                let target_pos = ambient_ctx
                    .sand_pos
                    .unwrap_or(player_pos - Vec3::new(0.0, 1.0, 0.0));
                let sand_vol = self.rand_f32(0.42, 0.62);
                let sand_pitch = self.rand_f32(0.95, 1.05);
                self.play_sound_3d(SoundId::AmbientDesertSand, target_pos, sand_vol, sand_pitch);
            }
        } else {
            self.ambient_sand_timer = (self.ambient_sand_timer - dt * 0.5).max(2.5);
        }

        if ambient_ctx.rain_intensity > 0.02 && !ambient_ctx.is_underground {
            self.ambient_rain_timer -= dt;
            if self.ambient_rain_timer <= 0.0 {
                self.ambient_rain_timer = 1.95;
                let vol = (ambient_ctx.rain_intensity * 0.75).clamp(0.15, 0.85);
                let pitch = self.rand_f32(0.97, 1.03);
                self.play_sound(SoundId::Rain, vol, pitch);
            }
        } else {
            self.ambient_rain_timer = 0.5;
        }

        self.tick_sync_audio();
    }

    fn tick_sync_audio(&mut self) {
        let (Some(mixer), Some(backend)) = (&mut self.sync_mixer, &mut self.sync_backend) else {
            return;
        };

        let mut scratch = [0.0f32; 2048];
        if mixer.has_pending_audio() {
            mixer.mix_interleaved_stereo(&mut scratch);
            backend.write_samples(&scratch);
        }
    }
}
