//! World chunk mesh scheduling, frustum culling, GPU/CPU dispatch, and tile-parallel terrain rasterization.
//! Zero platform `#[cfg]`; delegates parallel dispatch to `platform::threading` and GPU to `GpuBridge`.

use crate::core::math::{Mat4, Vec3};
use crate::features::meshing::{NUM_SUBCHUNKS, SUBCHUNK_SIZE};
use crate::features::player::Camera;
use crate::features::rendering::atlas::TextureId;
use crate::features::rendering::chunk_spawn_y_offset;
use crate::features::world::{CHUNK_SIZE_X, CHUNK_SIZE_Z, World};
use crate::platform::threading;

use super::clip::{clip_near_ao, orient_screen_triangle, project_to_screen_ao, to_clip_ao};
use super::raster::raster_screen_triangle_ao_tile;
use super::types::{Renderer, ScreenTriangle, TILE_HEIGHT};

impl Renderer {
    pub fn render_frame(&mut self, world: &mut World, camera: &Camera) {
        let (sky_top, sky_bottom, fog_col) = world.sky_colors();
        let daylight = world.sunlight_factor();
        self.current_daylight = daylight;
        self.current_fog_color = fog_col;

        let aspect = self.width as f32 / self.height as f32;
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect);
        let view_proj = proj.mul(&view);

        if !self.gpu.is_active() {
            self.clear_sky(camera.pitch, sky_top, sky_bottom);
            self.draw_celestials(camera, &view_proj, world.day_fraction());
        }

        self.loaded_keys.clear();
        self.loaded_keys.extend(world.chunks.keys().copied());
        self.mesh_cache.retain(|k, _| world.chunks.contains_key(k));
        self.chunk_spawn_times.retain(|k, _| world.chunks.contains_key(k));
        self.dirty_keys.clear();

        for key in &self.loaded_keys {
            self.chunk_spawn_times.entry(*key).or_insert(self.total_time);
        }

        // 1. Drain completed meshes from background threads
        for (key, mesh) in self.async_mesher.drain_completed() {
            self.pending_mesh_keys.remove(&key);
            if world.chunks.contains_key(&key) {
                self.mesh_cache.insert(key, mesh);
                self.dirty_keys.insert(key);
            }
        }

        self.pending_mesh_keys.retain(|k| world.chunks.contains_key(k));

        // 2. Submit background meshing jobs for dirty or unmeshed chunks
        let dirty_keys_to_build: Vec<(i32, i32)> = self
            .loaded_keys
            .iter()
            .copied()
            .filter(|key| {
                if self.pending_mesh_keys.contains(key) {
                    return false;
                }
                world
                    .chunks
                    .get(key)
                    .map_or(false, |chunk| chunk.dirty || !self.mesh_cache.contains_key(key))
            })
            .collect();

        for key in dirty_keys_to_build {
            if let Some(chunk) = world.chunks.get_mut(&key) {
                chunk.dirty = false;
            }
            if let Some(job) = world.create_mesh_job(key) {
                self.pending_mesh_keys.insert(key);
                self.async_mesher.submit_job(job);
            }
        }

        if self.gpu.is_active() {
            let is_occluded =
                world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);
            self.visible_keys.clear();
            if is_occluded {
                let (cam_cx, cam_cz, _, _) = World::world_to_chunk(
                    camera.position.x.floor() as i32,
                    camera.position.z.floor() as i32,
                );
                if self.loaded_keys.contains(&(cam_cx, cam_cz)) {
                    self.visible_keys.push((cam_cx, cam_cz));
                }
            } else {
                for key in &self.loaded_keys {
                    if Self::chunk_visible(*key, &view_proj) {
                        self.visible_keys.push(*key);
                    }
                }
            }

            if self.gpu.render_world(
                &self.mesh_cache,
                &self.loaded_keys,
                &self.dirty_keys,
                &self.visible_keys,
                &view_proj,
                camera.position,
                camera.pitch,
                self.render_distance,
                self.fog_enabled,
                self.chunk_anim,
                self.total_time,
                &self.chunk_spawn_times,
                daylight,
                world.day_fraction(),
                world.sun_direction(),
                (sky_top, sky_bottom, fog_col),
                &mut self.color_buffer,
                &mut self.depth_buffer,
            ) {
                self.last_draw_calls = self.visible_keys.len() as u32;
                let total_mesh_tris: usize = self
                    .loaded_keys
                    .iter()
                    .filter_map(|k| self.mesh_cache.get(k))
                    .map(|v| v.len())
                    .sum();
                let visible_mesh_tris: usize = self
                    .visible_keys
                    .iter()
                    .filter_map(|k| self.mesh_cache.get(k))
                    .map(|v| v.len())
                    .sum();
                self.last_triangles_rendered = visible_mesh_tris as u32;
                self.last_triangles_culled = total_mesh_tris.saturating_sub(visible_mesh_tris) as u32;
                self.last_subchunks_rendered =
                    (self.visible_keys.len() * NUM_SUBCHUNKS) as u32;
                self.last_subchunks_culled = (self
                    .loaded_keys
                    .len()
                    .saturating_sub(self.visible_keys.len())
                    * NUM_SUBCHUNKS) as u32;
                return;
            }
        }

        let mut visible: Vec<(i32, i32)> =
            threading::parallel_filter_map(&self.loaded_keys, |&key| {
                Self::chunk_visible(key, &view_proj).then_some(key)
            });

        let cam_x = camera.position.x - CHUNK_SIZE_X as f32 * 0.5;
        let cam_z = camera.position.z - CHUNK_SIZE_Z as f32 * 0.5;

        let is_occluded = world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);
        if is_occluded {
            let (cam_cx, cam_cz, _, _) = World::world_to_chunk(
                camera.position.x.floor() as i32,
                camera.position.z.floor() as i32,
            );
            visible.retain(|&k| k == (cam_cx, cam_cz));
        }

        visible.sort_unstable_by_key(|a| {
            let dx_a = a.0 as f32 * CHUNK_SIZE_X as f32 - cam_x;
            let dz_a = a.1 as f32 * CHUNK_SIZE_Z as f32 - cam_z;
            (dx_a * dx_a + dz_a * dz_a).to_bits()
        });

        let width = self.width;
        let height = self.height;
        let mesh_cache = &self.mesh_cache;
        let chunk_spawn_times = &self.chunk_spawn_times;
        let current_time = self.total_time;
        let chunk_anim = self.chunk_anim;

        let process_chunk = |key: &(i32, i32)| {
            let mut local_opaque = Vec::with_capacity(256);
            let mut local_transparent = Vec::with_capacity(64);

            let spawn_time = chunk_spawn_times.get(key).copied().unwrap_or(current_time);
            let y_offset = chunk_spawn_y_offset(chunk_anim, current_time, spawn_time);

            let chunk_vp = if y_offset != 0.0 {
                view_proj.mul(&Mat4::translation(0.0, y_offset, 0.0))
            } else {
                view_proj
            };

            if let Some(tris) = mesh_cache.get(key) {
                for tri in tris {
                    let c0 = to_clip_ao(tri.v0, &chunk_vp);
                    let c1 = to_clip_ao(tri.v1, &chunk_vp);
                    let c2 = to_clip_ao(tri.v2, &chunk_vp);

                    let (clipped, clipped_len) = clip_near_ao(&[c0, c1, c2]);
                    if clipped_len < 3 {
                        continue;
                    }

                    let is_transparent =
                        matches!(tri.texture, TextureId::Water | TextureId::Glass);

                    for i in 1..clipped_len - 1 {
                        let sa = project_to_screen_ao(&clipped[0], width, height);
                        let sb = project_to_screen_ao(&clipped[i], width, height);
                        let sc = project_to_screen_ao(&clipped[i + 1], width, height);

                        let Some((sa, sb, sc, _)) =
                            orient_screen_triangle(sa, sb, sc, tri.texture)
                        else {
                            continue;
                        };

                        let min_y = sa.y.min(sb.y).min(sc.y).floor() as i32;
                        let max_y = sa.y.max(sb.y).max(sc.y).ceil() as i32;

                        if min_y >= height as i32 || max_y < 0 {
                            continue;
                        }

                        let stri = ScreenTriangle {
                            sa,
                            sb,
                            sc,
                            texture: tri.texture,
                            min_y,
                            max_y,
                        };

                        if is_transparent {
                            local_transparent.push(stri);
                        } else {
                            local_opaque.push(stri);
                        }
                    }
                }
            }
            (local_opaque, local_transparent)
        };

        threading::parallel_map_unzip_flatten(
            &visible,
            &mut self.opaque_tris_buf,
            &mut self.transparent_tris_buf,
            process_chunk,
        );

        self.last_draw_calls = visible.len() as u32;
        self.last_triangles_rendered =
            (self.opaque_tris_buf.len() + self.transparent_tris_buf.len()) as u32;
        let total_mesh_tris: usize = self
            .loaded_keys
            .iter()
            .filter_map(|k| self.mesh_cache.get(k))
            .map(|v| v.len())
            .sum();
        self.last_triangles_culled =
            total_mesh_tris.saturating_sub(self.last_triangles_rendered as usize) as u32;
        self.last_subchunks_rendered = (visible.len() * NUM_SUBCHUNKS) as u32;
        self.last_subchunks_culled =
            (self.loaded_keys.len().saturating_sub(visible.len()) * NUM_SUBCHUNKS) as u32;

        let textures = &self.textures;
        let opaque_slice = self.opaque_tris_buf.as_slice();
        let transparent_slice = self.transparent_tris_buf.as_slice();

        let fog_dist = if self.fog_enabled {
            (self.render_distance as f32 * 16.0).max(32.0)
        } else {
            0.0
        };
        let daylight = self.current_daylight;
        let fog_color = self.current_fog_color;
        let brightness = self.brightness;

        threading::parallel_for_two_chunks_mut(
            &mut self.color_buffer,
            &mut self.depth_buffer,
            width * TILE_HEIGHT,
            |tile_idx, c_slice, d_slice| {
                let tile_y_start = tile_idx * TILE_HEIGHT;
                let tile_y_end = (tile_y_start + TILE_HEIGHT).min(height);

                for stri in opaque_slice {
                    if stri.max_y < tile_y_start as i32 || stri.min_y >= tile_y_end as i32 {
                        continue;
                    }
                    raster_screen_triangle_ao_tile(
                        stri.sa,
                        stri.sb,
                        stri.sc,
                        stri.texture,
                        0.0,
                        width,
                        tile_y_start,
                        tile_y_end,
                        c_slice,
                        d_slice,
                        textures,
                        fog_dist,
                        daylight,
                        fog_color,
                        brightness,
                    );
                }

                for stri in transparent_slice.iter().rev() {
                    if stri.max_y < tile_y_start as i32 || stri.min_y >= tile_y_end as i32 {
                        continue;
                    }
                    raster_screen_triangle_ao_tile(
                        stri.sa,
                        stri.sb,
                        stri.sc,
                        stri.texture,
                        0.0,
                        width,
                        tile_y_start,
                        tile_y_end,
                        c_slice,
                        d_slice,
                        textures,
                        fog_dist,
                        daylight,
                        fog_color,
                        brightness,
                    );
                }
            },
        );
    }

    pub(super) fn chunk_visible(key: (i32, i32), view_proj: &Mat4) -> bool {
        let min_x = key.0 as f32 * CHUNK_SIZE_X as f32;
        let min_y = -16.0;
        let min_z = key.1 as f32 * CHUNK_SIZE_Z as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let max_y = 66.0;
        let max_z = min_z + CHUNK_SIZE_Z as f32;
        aabb_visible(min_x, max_x, min_y, max_y, min_z, max_z, view_proj)
    }

    pub fn subchunk_visible(cx: i32, sub_y: usize, cz: i32, view_proj: &Mat4) -> bool {
        let min_x = cx as f32 * CHUNK_SIZE_X as f32;
        let min_y = (sub_y as i32 * SUBCHUNK_SIZE) as f32 - 14.0;
        let min_z = cz as f32 * CHUNK_SIZE_Z as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let max_y = (sub_y as i32 * SUBCHUNK_SIZE + SUBCHUNK_SIZE) as f32;
        let max_z = min_z + CHUNK_SIZE_Z as f32;
        aabb_visible(min_x, max_x, min_y, max_y, min_z, max_z, view_proj)
    }
}

#[inline]
fn aabb_visible(
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    min_z: f32,
    max_z: f32,
    view_proj: &Mat4,
) -> bool {
    let corners = [
        Vec3::new(min_x, min_y, min_z),
        Vec3::new(max_x, min_y, min_z),
        Vec3::new(min_x, max_y, min_z),
        Vec3::new(max_x, max_y, min_z),
        Vec3::new(min_x, min_y, max_z),
        Vec3::new(max_x, min_y, max_z),
        Vec3::new(min_x, max_y, max_z),
        Vec3::new(max_x, max_y, max_z),
    ];

    let mut out_x_pos = 0u8;
    let mut out_x_neg = 0u8;
    let mut out_y_pos = 0u8;
    let mut out_y_neg = 0u8;
    let mut out_z_pos = 0u8;
    let mut out_z_neg = 0u8;

    for (i, &corner) in corners.iter().enumerate() {
        let clip = view_proj.transform_point(corner);
        if clip[0] > clip[3] { out_x_pos += 1; }
        if clip[0] < -clip[3] { out_x_neg += 1; }
        if clip[1] > clip[3] { out_y_pos += 1; }
        if clip[1] < -clip[3] { out_y_neg += 1; }
        if clip[2] > clip[3] { out_z_pos += 1; }
        if clip[2] < -clip[3] { out_z_neg += 1; }

        let remaining = (corners.len() - 1 - i) as u8;
        let none_can_reach_8 = out_x_pos + remaining < 8
            && out_x_neg + remaining < 8
            && out_y_pos + remaining < 8
            && out_y_neg + remaining < 8
            && out_z_pos + remaining < 8
            && out_z_neg + remaining < 8;
        if none_can_reach_8 {
            return true;
        }
    }

    !(out_x_pos == 8
        || out_x_neg == 8
        || out_y_pos == 8
        || out_y_neg == 8
        || out_z_pos == 8
        || out_z_neg == 8)
}
