// 100% software renderer: no GPU draw calls of any kind.
// Pipeline: world-space triangles -> view/projection transform -> near-plane
// clipping -> perspective divide -> viewport transform -> scanline
// rasterization with a per-pixel depth buffer and texture sampling ->
// pixel buffer (u32 RGB) that main.rs blits into the OS window via minifb
// (minifb does NOT render anything itself; it just owns the window and
// copies our finished buffer to the screen).

use std::collections::HashMap;

use crate::camera::Camera;
use crate::font;
use crate::mesher::{build_chunk_mesh, Triangle, Vertex};
use crate::texture::TextureAtlas;
use crate::world::World;

pub struct Renderer {
    width: usize,
    height: usize,
    color_buffer: Vec<u32>,
    depth_buffer: Vec<f32>,
    mesh_cache: HashMap<(i32, i32), Vec<Triangle>>,
    selected_block: u8,
    textures: TextureAtlas,
}

/// A triangle vertex after the view-projection transform, still in
/// homogeneous clip space (x, y, z, w) plus its texture coordinate.
#[derive(Clone, Copy)]
struct ClipVert {
    clip: [f32; 4],
    uv: (f32, f32),
}

/// A vertex fully resolved for rasterization: screen-space (x, y), the
/// reciprocal of view-space w (used for perspective-correct interpolation
/// and depth testing), and UV pre-divided by w.
#[derive(Clone, Copy)]
struct ScreenVert {
    x: f32,
    y: f32,
    inv_w: f32,
    uv_over_w: (f32, f32),
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
        }
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
        // Vertical sky gradient, computed entirely on the CPU.
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
            for x in 0..self.width {
                self.color_buffer[row_start + x] = color;
            }
        }
        for d in self.depth_buffer.iter_mut() {
            *d = f32::INFINITY;
        }
    }

    pub fn render_frame(&mut self, world: &World, camera: &Camera) {
        self.clear(camera.pitch);

        let aspect = self.width as f32 / self.height as f32;
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect);
        let view_proj = proj.mul(&view);

        // Drop cached meshes for chunks that are no longer loaded.
        let loaded_keys: std::collections::HashSet<(i32, i32)> =
            world.chunks.keys().copied().collect();
        self.mesh_cache.retain(|k, _| loaded_keys.contains(k));

        let mut keys: Vec<(i32, i32)> = world.chunks.keys().copied().collect();
        keys.sort_unstable();

        // Rebuild meshes for chunks that are new or were edited since last
        // time we meshed them (tracked via each chunk's `dirty` flag, which
        // World::set_block flips to true on edits).
        for key in &keys {
            let chunk = world.chunks.get(key).unwrap();
            let already_built_clean = self.mesh_cache.contains_key(key) && !chunk.dirty;
            if !already_built_clean {
                let mesh = build_chunk_mesh(chunk, world);
                self.mesh_cache.insert(*key, mesh);
            }
        }

        for key in &keys {
            // Take the triangle list out temporarily so we can mutably
            // borrow `self` (for the depth/color buffers) while iterating.
            if let Some(tris) = self.mesh_cache.remove(key) {
                for tri in &tris {
                    self.raster_triangle(tri, &view_proj);
                }
                self.mesh_cache.insert(*key, tris);
            }
        }
    }

    fn raster_triangle(&mut self, tri: &Triangle, view_proj: &crate::math::Mat4) {
        let c0 = to_clip(tri.v0, view_proj);
        let c1 = to_clip(tri.v1, view_proj);
        let c2 = to_clip(tri.v2, view_proj);

        let clipped = clip_near(&[c0, c1, c2]);
        if clipped.len() < 3 {
            return;
        }

        // Fan-triangulate the (convex) clip polygon, then rasterize each
        // resulting triangle.
        for i in 1..clipped.len() - 1 {
            let sa = self.project_to_screen(&clipped[0]);
            let sb = self.project_to_screen(&clipped[i]);
            let sc = self.project_to_screen(&clipped[i + 1]);
            self.raster_screen_triangle(sa, sb, sc, tri.texture, tri.tint);
        }
    }

    fn project_to_screen(&self, v: &ClipVert) -> ScreenVert {
        let inv_w = 1.0 / v.clip[3];
        let ndc_x = v.clip[0] * inv_w;
        let ndc_y = v.clip[1] * inv_w;
        let screen_x = (ndc_x * 0.5 + 0.5) * self.width as f32;
        let screen_y = (1.0 - (ndc_y * 0.5 + 0.5)) * self.height as f32;
        ScreenVert {
            x: screen_x,
            y: screen_y,
            inv_w,
            uv_over_w: (v.uv.0 * inv_w, v.uv.1 * inv_w),
        }
    }

    fn raster_screen_triangle(
        &mut self,
        a: ScreenVert,
        b: ScreenVert,
        c: ScreenVert,
        texture: crate::texture::TextureId,
        tint: [f32; 3],
    ) {
        // Signed area (screen space, y-down, x-right). With this convention,
        // front-facing triangles (CCW winding in world space, as emitted by
        // the mesher, viewed from outside the cube) project to a POSITIVE
        // signed area here. Reject everything else (back-facing or
        // degenerate) so we only ever draw outward-facing surfaces.
        let area = edge(a.x, a.y, b.x, b.y, c.x, c.y);
        if area <= 0.0 {
            return; // back-facing or degenerate, cull it
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

        for py in min_y..=max_y {
            let fy = py as f32 + 0.5;
            for px in min_x..=max_x {
                let fx = px as f32 + 0.5;

                let w0 = edge(b.x, b.y, c.x, c.y, fx, fy);
                let w1 = edge(c.x, c.y, a.x, a.y, fx, fy);
                let w2 = edge(a.x, a.y, b.x, b.y, fx, fy);

                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    let l0 = w0 * inv_area;
                    let l1 = w1 * inv_area;
                    let l2 = w2 * inv_area;

                    let inv_w = l0 * a.inv_w + l1 * b.inv_w + l2 * c.inv_w;
                    if inv_w <= 0.0 {
                        continue;
                    }
                    let depth = 1.0 / inv_w;

                    let idx = (py as usize) * self.width + (px as usize);
                    if depth < self.depth_buffer[idx] {
                        self.depth_buffer[idx] = depth;

                        let u = (l0 * a.uv_over_w.0 + l1 * b.uv_over_w.0 + l2 * c.uv_over_w.0) * depth;
                        let v = (l0 * a.uv_over_w.1 + l1 * b.uv_over_w.1 + l2 * c.uv_over_w.1) * depth;

                        let texel = tex.sample(u, v);
                        let r = texel[0] * tint[0];
                        let g = texel[1] * tint[1];
                        let bl = texel[2] * tint[2];

                        // Distance fog blended toward the sky/horizon color.
                        let fog_t = (depth / 180.0).clamp(0.0, 1.0);
                        let fog = (0.75f32, 0.85f32, 1.0f32);
                        let fr = r * (1.0 - fog_t) + fog.0 * fog_t;
                        let fg = g * (1.0 - fog_t) + fog.1 * fog_t;
                        let fb = bl * (1.0 - fog_t) + fog.2 * fog_t;

                        self.color_buffer[idx] = pack_color(fr, fg, fb);
                    }
                }
            }
        }
    }

    // ---------------------------------------------------------------
    // Basic UI / HUD drawing. Everything below is 2D and drawn directly
    // into the color buffer after the 3D scene, still 100% on the CPU.
    // ---------------------------------------------------------------

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

    pub fn draw_hotbar(&mut self) {
        let slot_size = 28i32;
        let margin = 3i32;
        let total_slots = 9i32;
        let total_w = total_slots * (slot_size + margin) - margin;
        let start_x = (self.width as i32 - total_w) / 2;
        let y = self.height as i32 - slot_size - 12;

        // Backing bar behind all slots for readability.
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

            // Slot number label (1-9) in the corner of each swatch.
            let label = format!("{}", id);
            self.draw_text_shadowed(&label, x + 2, y + 2, 1, 1);
        }
    }

    /// Small debug/info overlay in the top-left: FPS, position, selected
    /// block name, and whether the mouse is currently captured.
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

    /// A centered overlay shown while the mouse is not captured, telling
    /// the player how to grab it. Dims the screen slightly behind the text.
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

    /// Computes the button bounding boxes (in renderer pixel coordinates) for the pause menu.
    pub fn pause_menu_button_rects(&self) -> [(i32, i32, i32, i32); 3] {
        let btn_w = 340;
        let btn_h = 36;
        let spacing = 12;
        let cx = self.width as i32 / 2;
        let start_y = self.height as i32 / 2 - 40;

        [
            (cx - btn_w / 2, start_y, btn_w, btn_h),
            (cx - btn_w / 2, start_y + btn_h + spacing, btn_w, btn_h),
            (cx - btn_w / 2, start_y + (btn_h + spacing) * 2, btn_w, btn_h),
        ]
    }

    /// Renders a full Minecraft-style pause menu overlay with interactive buttons.
    pub fn draw_pause_menu(&mut self, fly_mode: bool, hovered_idx: Option<usize>) {
        // Dim the world background
        self.fill_rect_alpha(0, 0, self.width as i32, self.height as i32, pack_color(0.0, 0.0, 0.0), 0.65);

        let cx = self.width as i32 / 2;
        let cy = self.height as i32 / 2;

        // Title
        let title = "MENU GRY";
        let t_scale = 4;
        let t_w = font::measure_text(title, t_scale, 2);
        self.draw_text_shadowed(title, cx - t_w / 2, cy - 110, t_scale, 2);

        let buttons = [
            "WZNOW GRE (ESC)",
            if fly_mode { "TRYB LOTU: WLACZONY (F)" } else { "TRYB LOTU: WYLACZONY (F)" },
            "WYJDZ Z GRY (Q)",
        ];

        let rects = self.pause_menu_button_rects();

        for (i, (&label, &(bx, by, bw, bh))) in buttons.iter().zip(rects.iter()).enumerate() {
            let is_hover = hovered_idx == Some(i);

            // Button body
            let body_color = if is_hover {
                pack_color(0.35, 0.45, 0.65)
            } else {
                pack_color(0.22, 0.22, 0.26)
            };
            self.fill_rect(bx, by, bw, bh, body_color);

            // 3D bevel borders
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

            // Top and left highlight border
            self.fill_rect(bx, by, bw, 2, top_left);
            self.fill_rect(bx, by, 2, bh, top_left);
            // Bottom and right shadow border
            self.fill_rect(bx, by + bh - 2, bw, 2, bottom_right);
            self.fill_rect(bx + bw - 2, by, 2, bh, bottom_right);

            // Text
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

        // Helpful controls reminder at the bottom of the pause menu
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

        // Shadow pass (offset by 1px) then foreground pass on top, for
        // legibility against any background.
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

    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u32) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set_px(xx, yy, color);
            }
        }
    }

    /// Alpha-blended rectangle fill, used for semi-transparent UI panels.
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

fn to_clip(v: Vertex, view_proj: &crate::math::Mat4) -> ClipVert {
    let c = view_proj.transform_point(v.pos);
    ClipVert { clip: c, uv: v.uv }
}

/// Clips a single triangle against the near plane (w > epsilon), using the
/// Sutherland-Hodgman algorithm against one plane. Since a triangle clipped
/// against a single plane produces at most a quad, the result has 0, 3, or
/// 4 vertices.
fn clip_near(verts: &[ClipVert; 3]) -> Vec<ClipVert> {
    const EPS: f32 = 1e-4;
    let inside = |v: &ClipVert| v.clip[3] > EPS;

    let mut output: Vec<ClipVert> = Vec::with_capacity(4);
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
            output.push(ClipVert { clip, uv });
        }
    }
    output
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

fn unpack_color(c: u32) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    (r, g, b)
}
