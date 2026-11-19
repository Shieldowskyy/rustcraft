//! Playing-state 3D scene rendering: projection, camera shake/perspective, terrain, entities, weather, and post-FX.
//! Mutates `renderer` framebuffer/projection cache and records terrain/clouds/entities timings in `FrameProfiler`.
//! Uses `capabilities().uses_hardware_gxm` to skip CPU 3D passes on PS Vita hardware GXM without platform `#[cfg]`.

use crate::core::math::{IVec3, Vec3};
use crate::core::profiler::FrameProfiler;
use crate::gameplay::player::GameMode;
use crate::graphics::camera::Camera;
use crate::platform::capabilities;
use crate::world::block::BlockId;

use super::super::{CameraPerspective, RustCraftApp};

impl RustCraftApp {
    pub(super) fn render_playing_scene(
        &mut self,
        collect_profiling: bool,
        cur_profiler: &mut FrameProfiler,
    ) -> Camera {
        let aspect = self.width as f32 / self.height as f32;

        if (self.camera.fov_y - self.last_fov).abs() > 0.0001
            || (aspect - self.last_aspect).abs() > 0.0001
        {
            self.last_fov = self.camera.fov_y;
            self.last_aspect = aspect;
            self.proj_matrix = self.camera.projection_matrix(aspect);
        }

        let (shake_offset, roll) = if self.settings.damage_tilt && self.damage_tilt_timer > 0.0 {
            let progress = self.damage_tilt_timer / 0.4;
            let roll = (progress * std::f32::consts::PI).sin() * 0.14 * self.damage_tilt_dir;
            let sx = (self.total_time * 50.0).sin() * 0.08 * progress;
            let sy = (self.total_time * 60.0).cos() * 0.05 * progress;
            (self.camera.right_flat() * sx + Vec3::new(0.0, sy, 0.0), roll)
        } else {
            (Vec3::zero(), 0.0)
        };

        let mut effective_camera = match self.camera_perspective {
            CameraPerspective::FirstPerson => self.camera.clone(),
            CameraPerspective::ThirdPersonBack => {
                let eye_pos = self.camera.position;
                let forward = self.camera.forward();
                let actual_dist = self.world.raycast_camera_collision(eye_pos, -forward, 3.8);
                let mut cam = self.camera.clone();
                cam.position = eye_pos - forward * actual_dist;
                cam
            }
            CameraPerspective::ThirdPersonFront => {
                let eye_pos = self.camera.position;
                let forward = self.camera.forward();
                let actual_dist = self.world.raycast_camera_collision(eye_pos, forward, 3.8);
                let mut cam = self.camera.clone();
                cam.position = eye_pos + forward * actual_dist;
                cam.yaw = self.camera.yaw + std::f32::consts::PI;
                cam.pitch = -self.camera.pitch;
                cam
            }
        };
        effective_camera.position += shake_offset;
        effective_camera.roll += roll;

        // DLACZEGO: Na PS Vita 3D renderuje bezposrednio backend libgxm; color_buffer sluzy wylacznie jako warstwa HUD (alpha=0).
        if capabilities().uses_hardware_gxm {
            self.renderer.color_buffer.fill(0);
            return effective_camera;
        }

        let view_proj = self.proj_matrix.mul(&effective_camera.view_matrix());
        let render_terrain_start = collect_profiling.then(std::time::Instant::now);
        self.renderer.fog_enabled = self.settings.fog;
        self.renderer
            .render_frame(&mut self.world, &effective_camera);
        if let Some(t_start) = render_terrain_start {
            cur_profiler.render_terrain_ms = t_start.elapsed().as_secs_f32() * 1000.0;
        }

        let render_entities_start = collect_profiling.then(std::time::Instant::now);
        let is_occluded = self.world.is_view_fully_occluded(
            effective_camera.position,
            effective_camera.yaw,
            effective_camera.pitch,
        );

        if !is_occluded {
            if self.camera_perspective != CameraPerspective::FirstPerson {
                let feet_pos = Vec3::new(
                    self.camera.position.x,
                    self.camera.position.y - self.current_eye_offset,
                    self.camera.position.z,
                );
                let held = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
                    .map(|s| s.block);
                let swing = self.break_progress.max(self.hand_animation);
                let hurt_tint = if self.damage_tilt_timer > 0.0 {
                    0.65
                } else {
                    0.0
                };
                self.renderer.draw_player_model(
                    feet_pos,
                    self.camera.yaw,
                    self.camera.pitch,
                    self.walk_phase,
                    swing,
                    self.crouch_toggled,
                    hurt_tint,
                    None,
                    held,
                    &self.player.inventory.armor,
                    &effective_camera,
                    &view_proj,
                );
            }

            for mob in &self.mob_manager.mobs {
                self.renderer.draw_mob(mob, &effective_camera, &view_proj);
            }

            for tnt in &self.world.primed_tnts {
                self.renderer
                    .draw_tnt_cube(tnt.pos, tnt.fuse, &effective_camera, &view_proj);
            }

            if self.settings.clouds {
                let clouds_start = collect_profiling.then(std::time::Instant::now);
                self.renderer.draw_clouds(
                    effective_camera.position,
                    &view_proj,
                    self.total_time,
                    self.world.weather.rain_strength,
                    self.world.weather.thunder_strength,
                );
                if let Some(c_start) = clouds_start {
                    cur_profiler.render_clouds_ms = c_start.elapsed().as_secs_f32() * 1000.0;
                }
            }

            for item in &self.items {
                let ipos_x = item.position.x.floor() as i32;
                let ipos_z = item.position.z.floor() as i32;
                let item_y = item.position.y;
                for dy in 0..=3 {
                    let test_y = (item_y.floor() as i32) - dy;
                    let b = self.world.get_block(IVec3::new(ipos_x, test_y, ipos_z));
                    if b.is_solid() {
                        let top_y = test_y as f32 + 1.0 + 0.015;
                        if item_y >= top_y - 0.2 {
                            self.renderer
                                .draw_item_shadow(item.position, top_y, &view_proj);
                            break;
                        }
                    }
                }

                let y_offset = (item.age * 3.0).sin() * 0.08;
                let mut pos = item.position;
                pos.y += y_offset;
                let yaw = item.age * 2.5;
                self.renderer.draw_item_3d(item.block, pos, yaw, &view_proj);
            }

            if let Some(target) = self.break_target {
                if self.break_progress > 0.0 {
                    self.renderer
                        .draw_block_break_cracks(target, self.break_progress, &view_proj);
                }
            }

            if self.settings.particles {
                for p in &self.particles.particles {
                    self.renderer.draw_billboard(
                        p.pos,
                        p.size,
                        p.color,
                        &effective_camera,
                        &view_proj,
                    );
                }
            }

            if let Some(bobber) = &self.fishing_bobber {
                let bob_color = if bobber.in_water && bobber.bite_timer <= 0.0 {
                    [1.0, 0.3, 0.1]
                } else {
                    [0.9, 0.15, 0.15]
                };
                self.renderer.draw_billboard(
                    bobber.pos,
                    0.16,
                    bob_color,
                    &effective_camera,
                    &view_proj,
                );
                self.renderer.draw_billboard(
                    bobber.pos + Vec3::new(0.0, 0.08, 0.0),
                    0.10,
                    [0.95, 0.95, 0.95],
                    &effective_camera,
                    &view_proj,
                );
            }

            for arrow in &self.arrows {
                let dir = if arrow.velocity.length() > 0.001 {
                    arrow.velocity.normalized()
                } else {
                    Vec3::new(0.0, -1.0, 0.0)
                };
                self.renderer.draw_billboard(
                    arrow.position,
                    0.12,
                    [0.35, 0.35, 0.38],
                    &effective_camera,
                    &view_proj,
                );
                self.renderer.draw_billboard(
                    arrow.position - dir * 0.18,
                    0.09,
                    [0.65, 0.45, 0.25],
                    &effective_camera,
                    &view_proj,
                );
                self.renderer.draw_billboard(
                    arrow.position - dir * 0.36,
                    0.11,
                    [0.95, 0.95, 0.95],
                    &effective_camera,
                    &view_proj,
                );
            }

            self.renderer.draw_precipitation(
                &self.world,
                &effective_camera,
                &view_proj,
                self.total_time,
            );
            self.renderer
                .draw_lightning_bolts(&self.world.weather.active_lightning, &view_proj);
        }

        if let Some(client) = &self.net_client {
            for rp in client.remote_players.values() {
                self.renderer
                    .draw_remote_player(rp, &effective_camera, &view_proj);
            }
        }

        let block_at_camera = self.world.get_block(self.camera.position.floor_to_ivec());
        if block_at_camera == BlockId::Lava {
            self.renderer.draw_lava_overlay(self.total_time);
        } else {
            if self.settings.underwater_fx
                && matches!(
                    block_at_camera,
                    BlockId::Water | BlockId::Kelp | BlockId::Seagrass
                )
            {
                self.renderer.draw_underwater_overlay(self.total_time);
            }
            if self.player.fire_timer > 0.0 {
                self.renderer.draw_burning_fire_overlay(self.total_time);
            }
        }

        self.renderer
            .apply_biome_color_filter(self.biome_filter_tint);

        if self.damage_tilt_timer > 0.0 {
            self.renderer
                .draw_damage_flash(self.damage_tilt_timer / 0.4);
        }
        if self.player.gamemode == GameMode::Survival
            && self.player.health <= 2.0
            && self.player.health > 0.0
        {
            self.renderer.draw_low_health_vignette(self.total_time);
        }
        if let Some(e_start) = render_entities_start {
            cur_profiler.render_entities_ms = e_start.elapsed().as_secs_f32() * 1000.0;
        }

        effective_camera
    }
}
