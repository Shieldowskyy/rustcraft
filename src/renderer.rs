use std::collections::{HashMap, HashSet};

use crate::camera::Camera;
use crate::font;
use crate::math::{Mat4, Vec3};
use crate::mesher::{build_chunk_mesh, Triangle, Vertex};
use crate::texture::TextureAtlas;
use crate::vulkan::VulkanContext;
use crate::world::{World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec3,
    pub velocity: Vec3,
    pub life: f32,
    pub max_life: f32,
    pub color: [f32; 3],
    pub size: f32,
}

pub struct Renderer {
    width: usize,
    height: usize,
    color_buffer: Vec<u32>,
    depth_buffer: Vec<f32>,
    mesh_cache: HashMap<(i32, i32), Vec<Triangle>>,
    selected_block: u8,
    textures: TextureAtlas,
    vulkan: Option<VulkanContext>,
    loaded_keys: Vec<(i32, i32)>,
    dirty_keys: HashSet<(i32, i32)>,
    visible_keys: Vec<(i32, i32)>,
    #[allow(dead_code)]
    pub particles: Vec<Particle>,
}

#[derive(Clone, Copy)]
struct ClipVertAO {
    clip: [f32; 4],
    uv: (f32, f32),
    ao: f32,
}

#[derive(Clone, Copy)]
struct ScreenVertAO {
    x: f32,
    y: f32,
    inv_w: f32,
    uv_over_w: (f32, f32),
    ao_over_w: f32,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Renderer {
            width,
            height,
            color_buffer: vec![0; width * height],
            depth_buffer: vec![f32::INFINITY; width * height],
            mesh_cache: HashMap::new(),
            selected_block: 1,
            textures: TextureAtlas::generate(),
            vulkan: None,
            loaded_keys: Vec::with_capacity(128),
            dirty_keys: HashSet::with_capacity(64),
            visible_keys: Vec::with_capacity(128),
            particles: Vec::new(),
        }
    }

    pub fn draw_block_face(
        &mut self,
        pos: Vec3,
        face: crate::block::Face,
        _color: [f32; 3],
        _camera: &Camera,
        view_proj: &Mat4,
    ) {
        use crate::block::Face;

        let (v0, v1, v2, v3) = match face {
            Face::Top => (
                Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
                          Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
                          Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
                          Vec3::new(pos.x, pos.y + 1.0, pos.z),
            ),
            Face::Bottom => (
                Vec3::new(pos.x, pos.y, pos.z),
                             Vec3::new(pos.x + 1.0, pos.y, pos.z),
                             Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                             Vec3::new(pos.x, pos.y, pos.z + 1.0),
            ),
            Face::North => (
                Vec3::new(pos.x + 1.0, pos.y, pos.z),
                            Vec3::new(pos.x, pos.y, pos.z),
                            Vec3::new(pos.x, pos.y + 1.0, pos.z),
                            Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
            ),
            Face::South => (
                Vec3::new(pos.x, pos.y, pos.z + 1.0),
                            Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                            Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
                            Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
            ),
            Face::West => (
                Vec3::new(pos.x, pos.y, pos.z),
                           Vec3::new(pos.x, pos.y, pos.z + 1.0),
                           Vec3::new(pos.x, pos.y + 1.0, pos.z + 1.0),
                           Vec3::new(pos.x, pos.y + 1.0, pos.z),
            ),
            Face::East => (
                Vec3::new(pos.x + 1.0, pos.y, pos.z + 1.0),
                           Vec3::new(pos.x + 1.0, pos.y, pos.z),
                           Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z),
                           Vec3::new(pos.x + 1.0, pos.y + 1.0, pos.z + 1.0),
            ),
        };

        let tex_id = match face {
            Face::Top | Face::Bottom => crate::texture::TextureId::TntTop,
            _ => crate::texture::TextureId::TntSide,
        };

        let t1 = Triangle {
            v0: Vertex { pos: v0, uv: (0.0, 0.0), ao: 1.0 },
            v1: Vertex { pos: v1, uv: (1.0, 0.0), ao: 1.0 },
            v2: Vertex { pos: v2, uv: (1.0, 1.0), ao: 1.0 },
            texture: tex_id,
        };
        let t2 = Triangle {
            v0: Vertex { pos: v0, uv: (0.0, 0.0), ao: 1.0 },
            v1: Vertex { pos: v2, uv: (1.0, 1.0), ao: 1.0 },
            v2: Vertex { pos: v3, uv: (0.0, 1.0), ao: 1.0 },
            texture: tex_id,
        };

        self.raster_triangle(&t1, view_proj);
        self.raster_triangle(&t2, view_proj);
    }

    pub fn draw_tnt_cube(&mut self, pos: Vec3, fuse: f32, camera: &Camera, view_proj: &Mat4) {
        use crate::block::{BlockId, Face};

        let flash = (fuse * 12.0).sin().abs() * 0.6;
        let white_tint = Vec3::new(flash, flash, flash);

        for face in [Face::Top, Face::Bottom, Face::North, Face::South, Face::East, Face::West] {
            let color_base = BlockId::Tnt.face_color(face);
            let color = [
                (color_base[0] + white_tint.x).min(1.0),
                (color_base[1] + white_tint.y).min(1.0),
                (color_base[2] + white_tint.z).min(1.0),
            ];

            self.draw_block_face(pos, face, color, camera, view_proj);
        }
    }

    #[allow(dead_code)]
    pub fn spawn_explosion(&mut self, center: Vec3) {
        let mut rng_state = 12345u32;
        let mut rand_f32 = || -> f32 {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0 // -1.0 to 1.0
        };

        for _ in 0..40 {
            let vx = rand_f32() * 6.0;
            let vy = rand_f32() * 5.0 + 4.0;
            let vz = rand_f32() * 6.0;

            let life = (rand_f32() + 1.0) * 0.5 * 1.5 + 0.5; // 0.5 to 2.0 sekundy

            let color_val = (rand_f32() + 1.0) * 0.5 * 0.5 + 0.5;
            let color = if rand_f32() > 0.0 {
                [1.0, color_val * 0.6, 0.1] // Ogień
            } else {
                [color_val, color_val, color_val] // Dym
            };

            self.particles.push(Particle {
                pos: center,
                velocity: Vec3::new(vx, vy, vz),
                                life,
                                max_life: life,
                                color,
                                size: 0.6 + rand_f32() * 0.4,
            });
        }
    }

    #[allow(dead_code)]
    pub fn draw_particles(&mut self, camera: &Camera, view_proj: &Mat4, dt: f32) {
        let mut i = 0;
        while i < self.particles.len() {
            self.particles[i].life -= dt;
            if self.particles[i].life <= 0.0 {
                self.particles.swap_remove(i);
            } else {
                self.particles[i].velocity.y -= 9.81 * dt;
                self.particles[i].pos = self.particles[i].pos + self.particles[i].velocity * dt;

                let p = self.particles[i];
                let current_size = p.size * (p.life / p.max_life);

                self.draw_billboard(p.pos, current_size, p.color, camera, view_proj);
                i += 1;
            }
        }
    }

    pub fn new_vulkan(width: usize, height: usize) -> Result<Self, String> {
        let mut renderer = Self::new(width, height);
        renderer.vulkan = Some(VulkanContext::new(width, height, &renderer.textures)?);
        Ok(renderer)
    }

    pub fn backend_name(&self) -> &'static str {
        if self.vulkan.is_some() { "vulkan" } else { "software" }
    }

    pub fn pixel_buffer(&self) -> &[u32] {
        &self.color_buffer
    }

    pub fn selected_block(&self) -> u8 {
        self.selected_block
    }

    pub fn set_selected_block(&mut self, id: u8) {
        self.selected_block = id;
    }

    fn clear(&mut self, camera_pitch: f32) {
        for y in 0..self.height {
            let t = y as f32 / self.height.max(1) as f32;
            let t = (t + camera_pitch * 0.05).clamp(0.0, 1.0);
            let top = (0.35f32, 0.55f32, 0.95f32);
            let bottom = (0.75f32, 0.85f32, 1.0f32);
            let r = top.0 + (bottom.0 - top.0) * t;
            let g = top.1 + (bottom.1 - top.1) * t;
            let b = top.2 + (bottom.2 - top.2) * t;
            let color = pack_color(r, g, b);

            let row_start = y * self.width;
            // Optymalizacja: szybkie uzupełnianie wiersza (SIMD .fill)
            self.color_buffer[row_start..row_start + self.width].fill(color);
        }
        for d in self.depth_buffer.iter_mut() {
            *d = f32::INFINITY;
        }
    }

    pub fn render_frame(&mut self, world: &mut World, camera: &Camera) {
        if self.vulkan.is_none() {
            self.clear(camera.pitch);
        }

        let aspect = self.width as f32 / self.height as f32;
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect);
        let view_proj = proj.mul(&view);

        self.loaded_keys.clear();
        self.loaded_keys.extend(world.chunks.keys().copied());
        self.mesh_cache.retain(|k, _| world.chunks.contains_key(k));
        self.dirty_keys.clear();

        for key in &self.loaded_keys {
            let chunk = world.chunks.get(key).unwrap();
            let already_built_clean = self.mesh_cache.contains_key(key) && !chunk.dirty;
            if !already_built_clean {
                let mesh = build_chunk_mesh(chunk, world);
                self.mesh_cache.insert(*key, mesh);
                if let Some(chunk) = world.chunks.get_mut(key) {
                    chunk.dirty = false;
                }
                self.dirty_keys.insert(*key);
            }
        }

        if let Some(vulkan) = self.vulkan.as_mut() {
            self.visible_keys.clear();
            for key in &self.loaded_keys {
                if Self::chunk_visible(*key, &view_proj) {
                    self.visible_keys.push(*key);
                }
            }

            vulkan.render_world(
                &self.mesh_cache,
                &self.loaded_keys,
                &self.dirty_keys,
                &self.visible_keys,
                &view_proj,
                camera.position,
                camera.pitch,
                &mut self.color_buffer,
                &mut self.depth_buffer,
            );
            return;
        }

        let mut visible: Vec<(i32, i32)> = (0..self.loaded_keys.len())
        .filter(|&i| Self::chunk_visible(self.loaded_keys[i], &view_proj))
        .map(|i| self.loaded_keys[i])
        .collect();

        // Optymalizacja: szybkie przeliczanie distansu
        let cam_x = camera.position.x - CHUNK_SIZE_X as f32 * 0.5;
        let cam_z = camera.position.z - CHUNK_SIZE_Z as f32 * 0.5;

        visible.sort_unstable_by(|a, b| {
            let dx_a = a.0 as f32 * CHUNK_SIZE_X as f32 - cam_x;
            let dz_a = a.1 as f32 * CHUNK_SIZE_Z as f32 - cam_z;
            let dx_b = b.0 as f32 * CHUNK_SIZE_X as f32 - cam_x;
            let dz_b = b.1 as f32 * CHUNK_SIZE_Z as f32 - cam_z;

            (dx_a * dx_a + dz_a * dz_a).partial_cmp(&(dx_b * dx_b + dz_b * dz_b)).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Pass 1: Opaque geometry
        // Optymalizacja: Ominięcie alokacji Vec oraz problemu z Borrow Checkerem przez iterację po indeksach
        for key in &visible {
            let len = self.mesh_cache.get(key).map_or(0, |tris| tris.len());
            for i in 0..len {
                let tri = self.mesh_cache.get(key).unwrap()[i].clone();
                if !matches!(tri.texture, crate::texture::TextureId::Water | crate::texture::TextureId::Glass) {
                    self.raster_triangle(&tri, &view_proj);
                }
            }
        }

        // Pass 2: Transparent geometry
        for key in visible.iter().rev() {
            let len = self.mesh_cache.get(key).map_or(0, |tris| tris.len());
            for i in 0..len {
                let tri = self.mesh_cache.get(key).unwrap()[i].clone();
                if matches!(tri.texture, crate::texture::TextureId::Water | crate::texture::TextureId::Glass) {
                    self.raster_triangle(&tri, &view_proj);
                }
            }
        }
    }

    fn chunk_visible(key: (i32, i32), view_proj: &crate::math::Mat4) -> bool {
        let min_x = key.0 as f32 * CHUNK_SIZE_X as f32;
        let min_y = 0.0;
        let min_z = key.1 as f32 * CHUNK_SIZE_Z as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let max_y = CHUNK_SIZE_Y as f32;
        let max_z = min_z + CHUNK_SIZE_Z as f32;

        let corners = [
            crate::math::Vec3::new(min_x, min_y, min_z),
            crate::math::Vec3::new(max_x, min_y, min_z),
            crate::math::Vec3::new(min_x, max_y, min_z),
            crate::math::Vec3::new(max_x, max_y, min_z),
            crate::math::Vec3::new(min_x, min_y, max_z),
            crate::math::Vec3::new(max_x, min_y, max_z),
            crate::math::Vec3::new(min_x, max_y, max_z),
            crate::math::Vec3::new(max_x, max_y, max_z),
        ];

        let mut out_x_pos = 0; let mut out_x_neg = 0;
        let mut out_y_pos = 0; let mut out_y_neg = 0;
        let mut out_z_pos = 0; let mut out_z_neg = 0;

        for &corner in &corners {
            let clip = view_proj.transform_point(corner);
            if clip[0] > clip[3] { out_x_pos += 1; }
            if clip[0] < -clip[3] { out_x_neg += 1; }
            if clip[1] > clip[3] { out_y_pos += 1; }
            if clip[1] < -clip[3] { out_y_neg += 1; }
            if clip[2] > clip[3] { out_z_pos += 1; }
            if clip[2] < -clip[3] { out_z_neg += 1; }
        }

        !(out_x_pos == 8 || out_x_neg == 8 || out_y_pos == 8 || out_y_neg == 8 || out_z_pos == 8 || out_z_neg == 8)
    }

    fn raster_triangle(&mut self, tri: &Triangle, view_proj: &crate::math::Mat4) {
        self.raster_triangle_tinted(tri, 0.0, view_proj);
    }

    pub fn raster_triangle_tinted(&mut self, tri: &Triangle, hurt_tint: f32, view_proj: &crate::math::Mat4) {
        let c0 = to_clip_ao(tri.v0, view_proj);
        let c1 = to_clip_ao(tri.v1, view_proj);
        let c2 = to_clip_ao(tri.v2, view_proj);

        let clipped = clip_near_ao(&[c0, c1, c2]);
        if clipped.len() < 3 {
            return;
        }

        for i in 1..clipped.len() - 1 {
            let sa = self.project_to_screen_ao(&clipped[0]);
            let sb = self.project_to_screen_ao(&clipped[i]);
            let sc = self.project_to_screen_ao(&clipped[i + 1]);
            self.raster_screen_triangle_ao(sa, sb, sc, tri.texture, hurt_tint);
        }
    }

    fn project_to_screen_ao(&self, v: &ClipVertAO) -> ScreenVertAO {
        let inv_w = 1.0 / v.clip[3];
        let ndc_x = v.clip[0] * inv_w;
        let ndc_y = v.clip[1] * inv_w;
        let screen_x = (ndc_x * 0.5 + 0.5) * self.width as f32;
        let screen_y = (1.0 - (ndc_y * 0.5 + 0.5)) * self.height as f32;
        ScreenVertAO {
            x: screen_x,
            y: screen_y,
            inv_w,
            uv_over_w: (v.uv.0 * inv_w, v.uv.1 * inv_w),
            ao_over_w: v.ao * inv_w,
        }
    }

    fn raster_screen_triangle_ao(
        &mut self,
        a: ScreenVertAO,
        b: ScreenVertAO,
        c: ScreenVertAO,
        texture: crate::texture::TextureId,
        hurt_tint: f32,
    ) {
        let area = edge(a.x, a.y, b.x, b.y, c.x, c.y);
        if area <= 0.0 {
            return;
        }

        let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
        let max_x = a.x.max(b.x).max(c.x).ceil().min(self.width as f32 - 1.0) as i32;
        let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
        let max_y = a.y.max(b.y).max(c.y).ceil().min(self.height as f32 - 1.0) as i32;

        if min_x > max_x || min_y > max_y {
            return;
        }

        let inv_area = 1.0 / area;
        let tex = self.textures.get(texture);
        let transparent = matches!(texture, crate::texture::TextureId::Water | crate::texture::TextureId::Glass);

        let dx0 = c.x - b.x;
        let dy0 = c.y - b.y;
        let dx1 = a.x - c.x;
        let dy1 = a.y - c.y;
        let dx2 = b.x - a.x;
        let dy2 = b.y - a.y;

        let fx0 = min_x as f32 + 0.5;
        let fy0 = min_y as f32 + 0.5;

        let mut w0_row = edge(b.x, b.y, c.x, c.y, fx0, fy0);
        let mut w1_row = edge(c.x, c.y, a.x, a.y, fx0, fy0);
        let mut w2_row = edge(a.x, a.y, b.x, b.y, fx0, fy0);

        for py in min_y..=max_y {
            let mut w0 = w0_row;
            let mut w1 = w1_row;
            let mut w2 = w2_row;
            let row_idx = (py as usize) * self.width;

            for px in min_x..=max_x {
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let l0 = w0 * inv_area;
                    let l1 = w1 * inv_area;
                    let l2 = w2 * inv_area;

                    let inv_w = l0 * a.inv_w + l1 * b.inv_w + l2 * c.inv_w;
                    if inv_w > 0.0 {
                        let depth = 1.0 / inv_w;
                        let idx = row_idx + (px as usize);

                        if depth < self.depth_buffer[idx] {
                            if !transparent {
                                self.depth_buffer[idx] = depth;
                            }

                            let u = (l0 * a.uv_over_w.0 + l1 * b.uv_over_w.0 + l2 * c.uv_over_w.0) * depth;
                            let v = (l0 * a.uv_over_w.1 + l1 * b.uv_over_w.1 + l2 * c.uv_over_w.1) * depth;
                            let ao = (l0 * a.ao_over_w + l1 * b.ao_over_w + l2 * c.ao_over_w) * depth;

                            let texel = tex.sample(u, v);
                            let mut r = texel[0] * ao;
                            let mut g = texel[1] * ao;
                            let mut bl = texel[2] * ao;

                            if hurt_tint > 0.0 {
                                r = r * (1.0 - hurt_tint) + 0.95 * hurt_tint;
                                g = g * (1.0 - hurt_tint) + 0.12 * hurt_tint;
                                bl = bl * (1.0 - hurt_tint) + 0.12 * hurt_tint;
                            }

                            let fog_t = (depth / 180.0).clamp(0.0, 1.0);
                            let fog = (0.75f32, 0.85f32, 1.0f32);
                            let fr = r * (1.0 - fog_t) + fog.0 * fog_t;
                            let fg = g * (1.0 - fog_t) + fog.1 * fog_t;
                            let fb = bl * (1.0 - fog_t) + fog.2 * fog_t;

                            if transparent {
                                let (br, bg, bb) = unpack_color(self.color_buffer[idx]);
                                let alpha = 0.48;
                                self.color_buffer[idx] = pack_color(fr * alpha + br * (1.0 - alpha), fg * alpha + bg * (1.0 - alpha), fb * alpha + bb * (1.0 - alpha));
                            } else {
                                self.color_buffer[idx] = pack_color(fr, fg, fb);
                            }
                        }
                    }
                }
                w0 += dy0;
                w1 += dy1;
                w2 += dy2;
            }
            w0_row -= dx0;
            w1_row -= dx1;
            w2_row -= dx2;
        }
    }

    pub fn draw_billboard(
        &mut self,
        pos: crate::math::Vec3,
        size: f32,
        color: [f32; 3],
        camera: &crate::camera::Camera,
        view_proj: &crate::math::Mat4,
    ) {
        let right = camera.right_flat();
        let up = crate::math::Vec3::new(0.0, 1.0, 0.0);
        let h = size * 0.5;

        let corners = [
            pos - right * h - up * h,
            pos + right * h - up * h,
            pos + right * h + up * h,
            pos - right * h + up * h,
        ];

        let project = |p: crate::math::Vec3| -> Option<(f32, f32, f32)> {
            let c = view_proj.transform_point(p);
            if c[3] <= 1e-4 { return None; }
            let inv_w = 1.0 / c[3];
            let sx = (c[0] * inv_w * 0.5 + 0.5) * self.width as f32;
            let sy = (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * self.height as f32;
            Some((sx, sy, 1.0 / c[3]))
        };

        let pts: Vec<_> = corners.iter().filter_map(|&p| project(p)).collect();
        if pts.len() < 4 { return; }

        let depth_at_pos = {
            let c = view_proj.transform_point(pos);
            if c[3] <= 1e-4 { return; }
            c[3]
        };

        let packed = pack_color(color[0], color[1], color[2]);
        for &tri_idx in &[(0usize,1usize,2usize),(0usize,2usize,3usize)] {
            let (ai, bi, ci) = tri_idx;
            let (ax, ay, _) = pts[ai];
            let (bx, by, _) = pts[bi];
            let (cx, cy, _) = pts[ci];
            let area = edge(ax, ay, bx, by, cx, cy);
            if area <= 0.0 { continue; }
            let inv_area = 1.0 / area;
            let min_x = ax.min(bx).min(cx).floor().max(0.0) as i32;
            let max_x = ax.max(bx).max(cx).ceil().min(self.width as f32 - 1.0) as i32;
            let min_y = ay.min(by).min(cy).floor().max(0.0) as i32;
            let max_y = ay.max(by).max(cy).ceil().min(self.height as f32 - 1.0) as i32;
            for py in min_y..=max_y {
                for px in min_x..=max_x {
                    let fx = px as f32 + 0.5;
                    let fy = py as f32 + 0.5;
                    let w0 = edge(bx, by, cx, cy, fx, fy);
                    let w1 = edge(cx, cy, ax, ay, fx, fy);
                    let w2 = edge(ax, ay, bx, by, fx, fy);
                    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                        let _ = inv_area;
                        let idx = py as usize * self.width + px as usize;
                        if depth_at_pos < self.depth_buffer[idx] {
                            self.color_buffer[idx] = packed;
                        }
                    }
                }
            }
        }
    }

    pub fn draw_clouds(
        &mut self,
        player_pos: crate::math::Vec3,
        view_proj: &crate::math::Mat4,
        time: f32,
    ) {
        let cloud_y = 75.0f32;
        let cloud_color = [0.97f32, 0.97f32, 1.0f32];
        let cloud_size = 8.0f32;
        let scroll = time * 1.5;

        let cx = (player_pos.x / cloud_size).floor() as i32;
        let cz = (player_pos.z / cloud_size).floor() as i32;

        for dz in -6..=6 {
            for dx in -6..=6 {
                let wx = (cx + dx) as f32 * cloud_size + (scroll % cloud_size);
                let wz = (cz + dz) as f32 * cloud_size;

                let hx = (cx + dx).wrapping_mul(374761393i32);
                let hz = (cz + dz).wrapping_mul(668265263i32);
                let h = (hx ^ hz ^ hx.wrapping_mul(hz >> 3)) as u32;
                if h % 3 != 0 { continue; }

                let scale = cloud_size * (0.6 + (h % 100) as f32 / 250.0);

                if (cloud_y - player_pos.y).abs() > 80.0 { continue; }

                let half = scale * 0.5;
                let corners = [
                    crate::math::Vec3::new(wx - half, cloud_y, wz - half),
                    crate::math::Vec3::new(wx + half, cloud_y, wz - half),
                    crate::math::Vec3::new(wx + half, cloud_y, wz + half),
                    crate::math::Vec3::new(wx - half, cloud_y, wz + half),
                ];

                let mut pts = [(0.0f32, 0.0f32, 0.0f32); 4];
                let mut all_valid = true;
                for (i, &p) in corners.iter().enumerate() {
                    let c = view_proj.transform_point(p);
                    if c[3] <= 1e-4 { all_valid = false; break; }
                    let inv_w = 1.0 / c[3];
                    pts[i] = (
                        (c[0] * inv_w * 0.5 + 0.5) * self.width as f32,
                              (1.0 - (c[1] * inv_w * 0.5 + 0.5)) * self.height as f32,
                              inv_w,
                    );
                }
                if !all_valid { continue; }

                let depth_center = {
                    let c = view_proj.transform_point(crate::math::Vec3::new(wx, cloud_y, wz));
                    if c[3] <= 1e-4 { continue; }
                    c[3]
                };

                // Optymalizacja: Rozpakowanie stałych z góry pętli
                let packed = pack_color(cloud_color[0], cloud_color[1], cloud_color[2]);
                let (cr, cg, cb) = unpack_color(packed);
                let alpha = 0.72f32;

                for &(ai, bi, ci) in &[(0usize,1usize,2usize),(0usize,2usize,3usize)] {
                    let (ax, ay, _) = pts[ai];
                    let (bx, by, _) = pts[bi];
                    let (cx, cy, _) = pts[ci];
                    let area = edge(ax, ay, bx, by, cx, cy);
                    if area.abs() < 0.5 { continue; }
                    let min_x = ax.min(bx).min(cx).floor().max(0.0) as i32;
                    let max_x = ax.max(bx).max(cx).ceil().min(self.width as f32 - 1.0) as i32;
                    let min_y = ay.min(by).min(cy).floor().max(0.0) as i32;
                    let max_y = ay.max(by).max(cy).ceil().min(self.height as f32 - 1.0) as i32;
                    for py in min_y..=max_y {
                        for px in min_x..=max_x {
                            let fx = px as f32 + 0.5;
                            let fy = py as f32 + 0.5;
                            let w0 = edge(bx, by, cx, cy, fx, fy);
                            let w1 = edge(cx, cy, ax, ay, fx, fy);
                            let w2 = edge(ax, ay, bx, by, fx, fy);
                            let inside = if area > 0.0 {
                                w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0
                            } else {
                                w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0
                            };
                            if inside {
                                let idx = py as usize * self.width + px as usize;
                                let w0n = w0 / area;
                                let w1n = w1 / area;
                                let w2n = w2 / area;
                                let inv_w = w0n * pts[ai].2 + w1n * pts[bi].2 + w2n * pts[ci].2;
                                let cloud_depth = if inv_w > 1e-6 { 1.0 / inv_w } else { depth_center };
                                if cloud_depth < self.depth_buffer[idx] {
                                    let (br, bg, bb) = unpack_color(self.color_buffer[idx]);
                                    let r = cr * alpha + br * (1.0 - alpha);
                                    let g = cg * alpha + bg * (1.0 - alpha);
                                    let b = cb * alpha + bb * (1.0 - alpha);
                                    self.color_buffer[idx] = pack_color(r, g, b);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn draw_crosshair(&mut self) {
        let cx = self.width as i32 / 2;
        let cy = self.height as i32 / 2;
        let size = 6i32;
        let color = pack_color(1.0, 1.0, 1.0);
        for i in -size..=size {
            self.set_px(cx + i, cy, color);
            self.set_px(cx, cy + i, color);
        }
    }

    pub fn draw_hand_3d(
        &mut self,
        camera: &crate::camera::Camera,
        view_proj: &crate::math::Mat4,
        block_id: u8,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
    ) {
        let forward = camera.forward();
        let right = camera.forward().cross(crate::math::Vec3::new(0.0, 1.0, 0.0)).normalized();
        let up = right.cross(forward).normalized();
        let movement = (speed / 8.5).clamp(0.0, 1.0);
        let swing = walk_phase.sin() * 0.10 * movement;
        let attack = (attack_animation * std::f32::consts::PI).sin();
        let swing = swing + attack * 0.16;
        let bob = (walk_phase * 2.0).cos() * 0.035 * movement - attack.abs() * 0.08;
        let hand_pos = camera.position
        + forward * 0.68
        + right * (0.53 + swing)
        + up * (-0.40 + bob);
        let arm_pos = hand_pos - right * 0.10 - up * 0.13;
        let block = crate::block::BlockId::from_u8(block_id);
        self.draw_cube_3d(arm_pos, right, up, forward, [0.085, 0.085, 0.12], crate::texture::TextureId::Dirt, view_proj);
        self.draw_cube_3d(hand_pos, right, up, forward, [0.13, 0.13, 0.13], block.texture_id(crate::block::Face::Top), view_proj);
    }

    fn draw_cube_3d(
        &mut self,
        center: crate::math::Vec3,
        right: crate::math::Vec3,
        up: crate::math::Vec3,
        forward: crate::math::Vec3,
            half: [f32; 3],
            texture: crate::texture::TextureId,
            view_proj: &crate::math::Mat4,
    ) {
        let corner = |x: f32, y: f32, z: f32| {
            center + right * (x * half[0]) + up * (y * half[1]) + forward * (z * half[2])
        };
        let faces = [
            ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)]),
            ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)]),
            ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)]),
            ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)]),
            ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)]),
            ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)]),
        ];
        for corners in faces {
            let vertices = [
                Vertex { pos: corners[0], uv: (0.0, 1.0), ao: 1.0 },
                Vertex { pos: corners[1], uv: (1.0, 1.0), ao: 1.0 },
                Vertex { pos: corners[2], uv: (1.0, 0.0), ao: 1.0 },
                Vertex { pos: corners[3], uv: (0.0, 0.0), ao: 1.0 },
            ];
            let first = Triangle { v0: vertices[0], v1: vertices[1], v2: vertices[2], texture };
            let second = Triangle { v0: vertices[0], v1: vertices[2], v2: vertices[3], texture };
            self.raster_triangle(&first, view_proj);
            self.raster_triangle(&second, view_proj);
            self.raster_triangle(&Triangle { v0: first.v2, v1: first.v1, v2: first.v0, texture }, view_proj);
            self.raster_triangle(&Triangle { v0: second.v2, v1: second.v1, v2: second.v0, texture }, view_proj);
        }
    }

    pub fn draw_mob(&mut self, mob: &crate::mob::Mob, _camera: &crate::camera::Camera, view_proj: &crate::math::Mat4) {
        use std::f32::consts::PI;
        use crate::texture::TextureId;
        use crate::mob::MobKind;

        let hurt_tint = if mob.is_dead {
            0.65
        } else if mob.hurt_timer > 0.0 {
            (mob.hurt_timer / 0.35).min(1.0) * 0.75
        } else {
            0.0
        };

        let death_roll = if mob.is_dead {
            (mob.death_timer / 0.8).min(1.0) * (PI * 0.5)
        } else {
            0.0
        };

        match mob.kind {
            MobKind::Pig => {
                let center_y = 0.42;
                // Body: center at (0, 0.42, -0.05), size: half = [0.28, 0.20, 0.40]
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.42, -0.05),
                    [0.28, 0.20, 0.40],
                    mob.yaw,
                    0.0,
                    death_roll,
                    [TextureId::PigSkin; 6],
                    hurt_tint,
                    view_proj,
                );

                // Head: center at (0, 0.50, 0.40), size: half = [0.20, 0.20, 0.20]
                let head_tex = [
                    TextureId::PigFace, // Front (+Z)
                    TextureId::PigSkin, // Back (-Z)
                    TextureId::PigSkin, // Top (+Y)
                    TextureId::PigSkin, // Bottom (-Y)
                    TextureId::PigSkin, // Right (+X)
                    TextureId::PigSkin, // Left (-X)
                ];
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.50, 0.40),
                    [0.20, 0.20, 0.20],
                    mob.yaw + mob.head_yaw_offset,
                    0.0,
                    death_roll,
                    head_tex,
                    hurt_tint,
                    view_proj,
                );

                // Snout: center at (0, 0.44, 0.62), size: half = [0.10, 0.07, 0.04]
                let snout_tex = [
                    TextureId::PigSnout, // Front (+Z)
                    TextureId::PigSkin,  // Back (-Z)
                    TextureId::PigSkin,  // Top (+Y)
                    TextureId::PigSkin,  // Bottom (-Y)
                    TextureId::PigSkin,  // Right (+X)
                    TextureId::PigSkin,  // Left (-X)
                ];
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.44, 0.62),
                    [0.10, 0.07, 0.04],
                    mob.yaw + mob.head_yaw_offset,
                    0.0,
                    death_roll,
                    snout_tex,
                    hurt_tint,
                    view_proj,
                );

                // 4 Legs: height 0.32 (half_y = 0.16), width/depth 0.16 (half = 0.08)
                let leg_pitch = if mob.is_dead { 0.0 } else { mob.walk_phase.sin() * 0.65 };
                let leg_tex = [TextureId::PigSkin; 6];

                // Front Left Leg (x = -0.16, z = 0.22)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(-0.16, 0.16, 0.22),
                    [0.08, 0.16, 0.08],
                    mob.yaw,
                    leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );

                // Front Right Leg (x = 0.16, z = 0.22)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.16, 0.16, 0.22),
                    [0.08, 0.16, 0.08],
                    mob.yaw,
                    -leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );

                // Back Left Leg (x = -0.16, z = -0.26)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(-0.16, 0.16, -0.26),
                    [0.08, 0.16, 0.08],
                    mob.yaw,
                    -leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );

                // Back Right Leg (x = 0.16, z = -0.26)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.16, 0.16, -0.26),
                    [0.08, 0.16, 0.08],
                    mob.yaw,
                    leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );
            }

            MobKind::Cow => {
                let center_y = 0.68;
                // Body: center at (0, 0.74, -0.05), size: half = [0.32, 0.26, 0.48]
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.74, -0.05),
                    [0.32, 0.26, 0.48],
                    mob.yaw,
                    0.0,
                    death_roll,
                    [TextureId::CowSkin; 6],
                    hurt_tint,
                    view_proj,
                );

                // Udder: center at (0, 0.44, -0.22), size: half = [0.12, 0.05, 0.14]
                let udder_tex = [
                    TextureId::CowUdder, // Front
                    TextureId::CowUdder, // Back
                    TextureId::CowSkin,  // Top
                    TextureId::CowUdder, // Bottom
                    TextureId::CowUdder, // Right
                    TextureId::CowUdder, // Left
                ];
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.44, -0.22),
                    [0.12, 0.05, 0.14],
                    mob.yaw,
                    0.0,
                    death_roll,
                    udder_tex,
                    hurt_tint,
                    view_proj,
                );

                // Head: center at (0, 0.98, 0.46), size: half = [0.20, 0.20, 0.18]
                let head_tex = [
                    TextureId::CowFace, // Front (+Z)
                    TextureId::CowSkin, // Back (-Z)
                    TextureId::CowSkin, // Top (+Y)
                    TextureId::CowSkin, // Bottom (-Y)
                    TextureId::CowSkin, // Right (+X)
                    TextureId::CowSkin, // Left (-X)
                ];
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.98, 0.46),
                    [0.20, 0.20, 0.18],
                    mob.yaw + mob.head_yaw_offset,
                    0.0,
                    death_roll,
                    head_tex,
                    hurt_tint,
                    view_proj,
                );

                // Snout: center at (0, 0.90, 0.66), size: half = [0.15, 0.08, 0.05]
                let snout_tex = [
                    TextureId::CowSnout, // Front (+Z)
                    TextureId::CowSkin,  // Back (-Z)
                    TextureId::CowSkin,  // Top (+Y)
                    TextureId::CowSkin,  // Bottom (-Y)
                    TextureId::CowSkin,  // Right (+X)
                    TextureId::CowSkin,  // Left (-X)
                ];
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.0, 0.90, 0.66),
                    [0.15, 0.08, 0.05],
                    mob.yaw + mob.head_yaw_offset,
                    0.0,
                    death_roll,
                    snout_tex,
                    hurt_tint,
                    view_proj,
                );

                // Horns: left and right
                let horn_tex = [TextureId::CowHorns; 6];
                // Left Horn (x = -0.22, y = 1.20, z = 0.44)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(-0.22, 1.20, 0.44),
                    [0.04, 0.08, 0.04],
                    mob.yaw + mob.head_yaw_offset,
                    0.0,
                    death_roll,
                    horn_tex,
                    hurt_tint,
                    view_proj,
                );
                // Right Horn (x = 0.22, y = 1.20, z = 0.44)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.22, 1.20, 0.44),
                    [0.04, 0.08, 0.04],
                    mob.yaw + mob.head_yaw_offset,
                    0.0,
                    death_roll,
                    horn_tex,
                    hurt_tint,
                    view_proj,
                );

                // 4 Legs: height 0.50 (half_y = 0.25), width/depth 0.16 (half = 0.08)
                let leg_pitch = if mob.is_dead { 0.0 } else { mob.walk_phase.sin() * 0.65 };
                let leg_tex = [TextureId::CowSkin; 6];

                // Front Left Leg (x = -0.18, z = 0.26)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(-0.18, 0.25, 0.26),
                    [0.08, 0.25, 0.08],
                    mob.yaw,
                    leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );

                // Front Right Leg (x = 0.18, z = 0.26)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.18, 0.25, 0.26),
                    [0.08, 0.25, 0.08],
                    mob.yaw,
                    -leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );

                // Back Left Leg (x = -0.18, z = -0.32)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(-0.18, 0.25, -0.32),
                    [0.08, 0.25, 0.08],
                    mob.yaw,
                    -leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );

                // Back Right Leg (x = 0.18, z = -0.32)
                self.draw_mob_box(
                    mob.pos,
                    center_y,
                    Vec3::new(0.18, 0.25, -0.32),
                    [0.08, 0.25, 0.08],
                    mob.yaw,
                    leg_pitch,
                    death_roll,
                    leg_tex,
                    hurt_tint,
                    view_proj,
                );
            }
        }
    }

    fn draw_mob_box(
        &mut self,
        mob_pos: Vec3,
        mob_center_y: f32,
        box_offset: Vec3,
        half: [f32; 3],
        yaw: f32,
        part_pitch: f32,
        death_roll: f32,
        textures: [crate::texture::TextureId; 6],
        hurt_tint: f32,
        view_proj: &crate::math::Mat4,
    ) {
        let rotate_x = |p: Vec3, angle: f32| -> Vec3 {
            let (s, c) = angle.sin_cos();
            Vec3::new(p.x, p.y * c - p.z * s, p.y * s + p.z * c)
        };
        let rotate_y = |p: Vec3, angle: f32| -> Vec3 {
            let (s, c) = angle.sin_cos();
            Vec3::new(p.x * c + p.z * s, p.y, -p.x * s + p.z * c)
        };
        let rotate_z = |p: Vec3, angle: f32| -> Vec3 {
            let (s, c) = angle.sin_cos();
            Vec3::new(p.x * c - p.y * s, p.x * s + p.y * c, p.z)
        };

        let transform_corner = |x: f32, y: f32, z: f32| -> Vec3 {
            let mut p = Vec3::new(x * half[0], y * half[1], z * half[2]);
            if part_pitch.abs() > 1e-4 {
                p.y -= half[1];
                p = rotate_x(p, part_pitch);
                p.y += half[1];
            }
            p = p + box_offset;

            if death_roll.abs() > 1e-4 {
                p.y -= mob_center_y;
                p = rotate_z(p, death_roll);
                p.y += mob_center_y;
            }

            p = rotate_y(p, yaw);
            mob_pos + p
        };

        let c0 = transform_corner(-1.0, -1.0, -1.0);
        let c1 = transform_corner( 1.0, -1.0, -1.0);
        let c2 = transform_corner( 1.0,  1.0, -1.0);
        let c3 = transform_corner(-1.0,  1.0, -1.0);
        let c4 = transform_corner(-1.0, -1.0,  1.0);
        let c5 = transform_corner( 1.0, -1.0,  1.0);
        let c6 = transform_corner( 1.0,  1.0,  1.0);
        let c7 = transform_corner(-1.0,  1.0,  1.0);

        let faces = [
            ([c4, c5, c6, c7], textures[0]), // Front (+Z)
            ([c1, c0, c3, c2], textures[1]), // Back (-Z)
            ([c7, c6, c2, c3], textures[2]), // Top (+Y)
            ([c0, c1, c5, c4], textures[3]), // Bottom (-Y)
            ([c5, c1, c2, c6], textures[4]), // Right (+X)
            ([c0, c4, c7, c3], textures[5]), // Left (-X)
        ];

        for (corners, texture) in faces {
            let v0 = Vertex { pos: corners[0], uv: (0.0, 1.0), ao: 1.0 };
            let v1 = Vertex { pos: corners[1], uv: (1.0, 1.0), ao: 1.0 };
            let v2 = Vertex { pos: corners[2], uv: (1.0, 0.0), ao: 1.0 };
            let v3 = Vertex { pos: corners[3], uv: (0.0, 0.0), ao: 1.0 };

            let t1 = Triangle { v0, v1, v2, texture };
            let t2 = Triangle { v0, v1: v2, v2: v3, texture };

            self.raster_triangle_tinted(&t1, hurt_tint, view_proj);
            self.raster_triangle_tinted(&t2, hurt_tint, view_proj);
            self.raster_triangle_tinted(&Triangle { v0: t1.v2, v1: t1.v1, v2: t1.v0, texture }, hurt_tint, view_proj);
            self.raster_triangle_tinted(&Triangle { v0: t2.v2, v1: t2.v1, v2: t2.v0, texture }, hurt_tint, view_proj);
        }
    }

    pub fn draw_underwater_overlay(&mut self, time: f32) {
        self.fill_rect_alpha(0, 0, self.width as i32, self.height as i32, pack_color(0.04, 0.28, 0.48), 0.34);
        let wave = ((time * 1.8).sin() * 3.0) as i32;
        for y in (18 + wave)..self.height as i32 {
            if y % 31 == 0 {
                self.fill_rect_alpha(0, y, self.width as i32, 1, pack_color(0.25, 0.65, 0.80), 0.08);
            }
        }
    }

    pub fn draw_hotbar(&mut self) {
        let slot_size = 28i32;
        let margin = 3i32;
        let total_slots = 19i32;
        let total_w = total_slots * (slot_size + margin) - margin;
        let start_x = (self.width as i32 - total_w) / 2;
        let y = self.height as i32 - slot_size - 12;

        self.fill_rect_alpha(start_x - 6, y - 6, total_w + 12, slot_size + 12, pack_color(0.0, 0.0, 0.0), 0.40);

        for i in 0..total_slots {
            let id = (i + 1) as u8;
            let block = crate::block::BlockId::from_u8(id);
            let color = block.face_color(crate::block::Face::Top);
            let x = start_x + i * (slot_size + margin);

            let is_selected = id == self.selected_block;
            let border_color = if is_selected {
                pack_color(1.0, 1.0, 0.2)
            } else {
                pack_color(0.20, 0.20, 0.20)
            };

            self.fill_rect(x - 2, y - 2, slot_size + 4, slot_size + 4, border_color);
            self.fill_rect(x, y, slot_size, slot_size, pack_color(color[0], color[1], color[2]));

            let label = format!("{}", id);
            self.draw_text_shadowed(&label, x + 2, y + 2, 1, 1);
        }
    }

    pub fn draw_debug_panel(&mut self, lines: &[String]) {
        let scale = 2;
        let line_h = (font::GLYPH_H as i32) * scale + 4;
        let padding = 6;

        let max_w = lines
        .iter()
        .map(|l| font::measure_text(l, scale, 1))
        .max()
        .unwrap_or(0);

        let panel_w = max_w + padding * 2;
        let panel_h = lines.len() as i32 * line_h + padding * 2 - 4;

        self.fill_rect_alpha(6, 6, panel_w, panel_h, pack_color(0.0, 0.0, 0.0), 0.4);

        for (i, line) in lines.iter().enumerate() {
            let ty = 6 + padding + i as i32 * line_h;
            self.draw_text_shadowed(line, 6 + padding, ty, scale, 1);
        }
    }

    pub fn draw_capture_prompt(&mut self, native_capture: bool) {
        self.fill_rect_alpha(0, 0, self.width as i32, self.height as i32, pack_color(0.0, 0.0, 0.0), 0.35);

        let line1 = "KLIKNIJ ABY GRAC";
        let line2 = if native_capture {
            "MYSZ ZOSTANIE ZABLOKOWANA W OKNIE"
        } else {
            "ESC - MENU PAUZY / WYJSCIE"
        };

        let scale1 = 4;
        let scale2 = 2;
        let w1 = font::measure_text(line1, scale1, 2);
        let w2 = font::measure_text(line2, scale2, 1);

        let cx = self.width as i32 / 2;
        let cy = self.height as i32 / 2;

        self.draw_text_shadowed(line1, cx - w1 / 2, cy - 20, scale1, 2);
        self.draw_text_shadowed(line2, cx - w2 / 2, cy + 20, scale2, 1);
    }

    pub fn main_menu_button_rects(&self) -> [(i32, i32, i32, i32); 3] {
        let btn_w = 340;
        let btn_h = 36;
        let spacing = 12;
        let cx = self.width as i32 / 2;
        let start_y = self.height as i32 / 2 + 45;
        [
            (cx - btn_w / 2, start_y, btn_w, btn_h),
            (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
            (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
        ]
    }

    pub fn main_menu_world_rects(&self) -> [(i32, i32, i32, i32); 5] {
        let row_w = 340;
        let row_h = 26;
        let cx = self.width as i32 / 2;
        let start_y = 160;
        [
            (cx - row_w / 2, start_y, row_w, row_h),
            (cx - row_w / 2, start_y + 30, row_w, row_h),
            (cx - row_w / 2, start_y + 60, row_w, row_h),
            (cx - row_w / 2, start_y + 90, row_w, row_h),
            (cx - row_w / 2, start_y + 120, row_w, row_h),
        ]
    }

    pub fn draw_main_menu(&mut self, seed: u32, save_slot: u8, save_slots: &[bool; 32], hovered_idx: Option<usize>) {
        self.clear(0.0);
        let cx = self.width as i32 / 2;
        self.fill_rect_alpha(cx - 230, 25, 460, 475, pack_color(0.02, 0.03, 0.05), 0.72);
        self.draw_rustcraft_logo(cx, 48);
        let title = "RUSTCRAFT";
        let title_w = font::measure_text(title, 4, 3);
        self.draw_text_colored_shadowed(title, cx - title_w / 2, 115, 4, 3, pack_color(0.95, 0.85, 0.42));
        let subtitle = "WOKSELOWY SWIAT";
        let subtitle_w = font::measure_text(subtitle, 2, 2);
        self.draw_text_shadowed(subtitle, cx - subtitle_w / 2, 158, 2, 2);

        let world_rects = self.main_menu_world_rects();
        for (row, &(bx, by, bw, bh)) in world_rects.iter().enumerate() {
            let slot = ((save_slot as i32 - 1 + row as i32 + 32 - 2).rem_euclid(32) + 1) as u8;
            let selected = slot == save_slot;
            let body = if selected { pack_color(0.32, 0.38, 0.50) } else { pack_color(0.14, 0.15, 0.18) };
            self.fill_rect(bx, by, bw, bh, body);
            if selected {
                self.fill_rect(bx, by, 3, bh, pack_color(0.95, 0.78, 0.25));
            }
            let status = if save_slots[slot as usize - 1] { "ZAPIS" } else { "PUSTY" };
            let label = format!("SWIAT {:02}   {}", slot, status);
            self.draw_text_shadowed(&label, bx + 12, by + 7, 1, 1);
        }

        let buttons = [
            "NOWY SWIAT",
            if save_slots[save_slot as usize - 1] { "WCZYTAJ ZAPIS" } else { "WCZYTAJ ZAPIS (BRAK)" },
                "WYJDZ Z GRY",
        ];
        let rects = self.main_menu_button_rects();
        for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);
            let body_color = if is_hover { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.22, 0.22, 0.26) };
            let top_left = if is_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
            let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };
            self.fill_rect(bx, by, bw, bh, body_color);
            self.fill_rect(bx, by, bw, 2, top_left);
            self.fill_rect(bx, by, 2, bh, top_left);
            self.fill_rect(bx, by + bh - 2, bw, 2, bottom_right);
            self.fill_rect(bx + bw - 2, by, 2, bh, bottom_right);
            let scale = 2;
            let tw = font::measure_text(label, scale, 1);
            let tx = bx + (bw - tw) / 2;
            let ty = by + (bh - font::GLYPH_H as i32 * scale) / 2;
            if is_hover {
                self.draw_text_colored_shadowed(label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
            } else {
                self.draw_text_shadowed(label, tx, ty, scale, 1);
            }
        }

        let seed_label = format!("SEED: {}", seed);
        self.draw_text_shadowed(&seed_label, cx - (seed_label.len() as i32 * 4), 475, 1, 1);
        let slot_label = format!("SLOT ZAPISU: {:02}", save_slot);
        self.draw_text_shadowed(&slot_label, cx - (slot_label.len() as i32 * 4), 492, 1, 1);
    }

    fn draw_rustcraft_logo(&mut self, cx: i32, y: i32) {
        let gold = pack_color(0.95, 0.72, 0.20);
        let light = pack_color(1.0, 0.90, 0.48);
        let dark = pack_color(0.38, 0.22, 0.08);
        let blocks = [
            (-48, 0, 8, 8, gold), (-40, 0, 8, 8, light), (-32, 0, 8, 8, gold),
            (-24, 0, 8, 8, dark), (-16, 0, 8, 8, gold), (-8, 0, 8, 8, light),
            (-40, 8, 8, 8, gold), (-32, 16, 8, 8, light), (-24, 24, 8, 8, gold),
            (-16, 32, 8, 8, dark), (-8, 40, 8, 8, gold), (0, 32, 8, 8, light),
            (8, 24, 8, 8, gold), (16, 16, 8, 8, dark), (24, 8, 8, 8, gold),
            (32, 0, 8, 8, light), (40, -8, 8, 8, gold),
        ];
        for (x, offset_y, w, h, color) in blocks {
            self.fill_rect(cx + x, y + offset_y, w, h, color);
        }
    }

    pub fn pause_menu_button_rects(&self) -> [(i32, i32, i32, i32); 4] {
        let btn_w = 340;
        let btn_h = 36;
        let spacing = 12;
        let cx = self.width as i32 / 2;
        let start_y = self.height as i32 / 2 - 90;

        [
            (cx - btn_w / 2, start_y, btn_w, btn_h),
            (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
            (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
            (cx - btn_w / 2, start_y + (btn_h + spacing) * 3, btn_w, btn_h),
        ]
    }

    pub fn draw_pause_menu(&mut self, fly_mode: bool, hovered_idx: Option<usize>) {
        self.fill_rect_alpha(0, 0, self.width as i32, self.height as i32, pack_color(0.0, 0.0, 0.0), 0.65);

        let cx = self.width as i32 / 2;
        let cy = self.height as i32 / 2;

        let title = "MENU GRY";
        let t_scale = 4;
        let t_w = font::measure_text(title, t_scale, 2);
        self.draw_text_shadowed(title, cx - t_w / 2, cy - 110, t_scale, 2);

        let buttons = [
            "WZNOW GRE (ESC)",
            if fly_mode { "TRYB LOTU: WLACZONY (F)" } else { "TRYB LOTU: WYLACZONY (F)" },
                "MENU GLOWNE (M)",
                "WYJDZ Z GRY (Q)",
        ];

        let rects = self.pause_menu_button_rects();

        for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);

            let body_color = if is_hover {
                pack_color(0.35, 0.45, 0.65)
            } else {
                pack_color(0.22, 0.22, 0.26)
            };
            self.fill_rect(bx, by, bw, bh, body_color);

            let top_left = if is_hover {
                pack_color(0.75, 0.85, 1.0)
            } else {
                pack_color(0.55, 0.55, 0.60)
            };
            let bottom_right = if is_hover {
                pack_color(0.15, 0.20, 0.35)
            } else {
                pack_color(0.10, 0.10, 0.12)
            };

            self.fill_rect(bx, by, bw, 2, top_left);
            self.fill_rect(bx, by, 2, bh, top_left);
            self.fill_rect(bx, by + bh - 2, bw, 2, bottom_right);
            self.fill_rect(bx + bw - 2, by, 2, bh, bottom_right);

            let scale = 2;
            let tw = font::measure_text(label, scale, 1);
            let tx = bx + (bw - tw) / 2;
            let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

            if is_hover {
                self.draw_text_colored_shadowed(label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
            } else {
                self.draw_text_shadowed(label, tx, ty, scale, 1);
            }
        }

        let footer1 = "STEROWANIE: WASD - RUCH | SHIFT - KUCANIE | CTRL - BIEG | SPACJA - SKOK";
        let footer2 = "LPM - NISZCZ | PPM - STAWIAJ | SCROLL / 1-6 - WYBOR BLOKU";
        let f_scale = 1;
        let f1_w = font::measure_text(footer1, f_scale, 1);
        let f2_w = font::measure_text(footer2, f_scale, 1);
        self.draw_text_colored_shadowed(footer1, cx - f1_w / 2, cy + 120, f_scale, 1, pack_color(0.7, 0.7, 0.7));
        self.draw_text_colored_shadowed(footer2, cx - f2_w / 2, cy + 135, f_scale, 1, pack_color(0.7, 0.7, 0.7));
    }

    fn draw_text_colored_shadowed(&mut self, text: &str, x: i32, y: i32, scale: i32, spacing: i32, fg_color: u32) {
        let shadow = pack_color(0.0, 0.0, 0.0);
        let w = self.width as i32;
        let h = self.height as i32;
        let buf = &mut self.color_buffer;

        font::draw_text(text, x + 1, y + 1, scale, spacing, |px, py| {
            if px >= 0 && py >= 0 && px < w && py < h {
                buf[(py as usize) * (w as usize) + (px as usize)] = shadow;
            }
        });
        font::draw_text(text, x, y, scale, spacing, |px, py| {
            if px >= 0 && py >= 0 && px < w && py < h {
                buf[(py as usize) * (w as usize) + (px as usize)] = fg_color;
            }
        });
    }

    fn draw_text_shadowed(&mut self, text: &str, x: i32, y: i32, scale: i32, spacing: i32) {
        let shadow = pack_color(0.0, 0.0, 0.0);
        let fg = pack_color(1.0, 1.0, 1.0);

        let w = self.width as i32;
        let h = self.height as i32;
        let buf = &mut self.color_buffer;

        font::draw_text(text, x + 1, y + 1, scale, spacing, |px, py| {
            if px >= 0 && py >= 0 && px < w && py < h {
                buf[(py as usize) * (w as usize) + (px as usize)] = shadow;
            }
        });
        font::draw_text(text, x, y, scale, spacing, |px, py| {
            if px >= 0 && py >= 0 && px < w && py < h {
                buf[(py as usize) * (w as usize) + (px as usize)] = fg;
            }
        });
    }

    // Optymalizacja: Usprawnione iterowanie przez pamięć dla szybkich kwadratów UI
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        let min_x = x.max(0);
        let min_y = y.max(0);
        let max_x = (x + w).min(self.width as i32);
        let max_y = (y + h).min(self.height as i32);
        if min_x >= max_x || min_y >= max_y { return; }
        for yy in min_y..max_y {
            let start = yy as usize * self.width + min_x as usize;
            let end = yy as usize * self.width + max_x as usize;
            self.color_buffer[start..end].fill(color);
        }
    }

    fn fill_rect_alpha(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32, alpha: f32) {
        let (cr, cg, cb) = unpack_color(color);
        let min_x = x.max(0);
        let min_y = y.max(0);
        let max_x = (x + w).min(self.width as i32);
        let max_y = (y + h).min(self.height as i32);
        for yy in min_y..max_y {
            for xx in min_x..max_x {
                let idx = yy as usize * self.width + xx as usize;
                let (br, bg, bb) = unpack_color(self.color_buffer[idx]);
                let r = cr * alpha + br * (1.0 - alpha);
                let g = cg * alpha + bg * (1.0 - alpha);
                let b = cb * alpha + bb * (1.0 - alpha);
                self.color_buffer[idx] = pack_color(r, g, b);
            }
        }
    }

    fn set_px(&mut self, x: i32, y: i32, color: u32) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let idx = y as usize * self.width + x as usize;
        self.color_buffer[idx] = color;
    }
}

fn to_clip_ao(v: Vertex, view_proj: &crate::math::Mat4) -> ClipVertAO {
    let c = view_proj.transform_point(v.pos);
    ClipVertAO { clip: c, uv: v.uv, ao: v.ao }
}

fn clip_near_ao(verts: &[ClipVertAO; 3]) -> Vec<ClipVertAO> {
    const EPS: f32 = 1e-4;
    let inside = |v: &ClipVertAO| v.clip[3] > EPS;

    let mut output: Vec<ClipVertAO> = Vec::with_capacity(4);
    for i in 0..3 {
        let cur = verts[i];
        let next = verts[(i + 1) % 3];
        let cur_in = inside(&cur);
        let next_in = inside(&next);

        if cur_in {
            output.push(cur);
        }
        if cur_in != next_in {
            let t = (EPS - cur.clip[3]) / (next.clip[3] - cur.clip[3]);
            let clip = [
                lerp(cur.clip[0], next.clip[0], t),
                lerp(cur.clip[1], next.clip[1], t),
                lerp(cur.clip[2], next.clip[2], t),
                lerp(cur.clip[3], next.clip[3], t),
            ];
            let uv = (
                lerp(cur.uv.0, next.uv.0, t),
                      lerp(cur.uv.1, next.uv.1, t),
            );
            let ao = lerp(cur.ao, next.ao, t);
            output.push(ClipVertAO { clip, uv, ao });
        }
    }
    output
}

#[inline(always)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline(always)]
fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

#[inline(always)]
fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

#[inline(always)]
fn unpack_color(c: u32) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    (r, g, b)
}
