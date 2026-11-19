//! Tile-based triangle scanline rasterizer (4x SIMD-style batching + single-pixel tail).
//! Zero-allocation inner loops with perspective-correct UV, AO, sky/torch lighting, and fog.

use crate::core::math::Mat4;
use crate::features::meshing::Triangle;
use crate::features::rendering::atlas::{Texture, TextureAtlas, TextureId};
use crate::features::rendering::compute_entity_light_rgb;

use super::clip::{
    clip_near_ao, edge, orient_screen_triangle, pack_color, project_to_screen_ao, to_clip_ao,
    unpack_color,
};
use super::types::{Renderer, ScreenVertAO};

impl Renderer {
    pub(super) fn raster_triangle(&mut self, tri: &Triangle, view_proj: &Mat4) {
        self.raster_triangle_tinted(tri, 0.0, view_proj);
    }

    pub fn raster_triangle_tinted(&mut self, tri: &Triangle, hurt_tint: f32, view_proj: &Mat4) {
        let width = self.width;
        let height = self.height;
        let textures = &self.textures;

        let fog_dist = if self.fog_enabled {
            (self.render_distance as f32 * 16.0).max(32.0)
        } else {
            0.0
        };
        raster_triangle_tile(
            tri,
            hurt_tint,
            view_proj,
            width,
            height,
            0,
            height,
            &mut self.color_buffer,
            &mut self.depth_buffer,
            textures,
            fog_dist,
            self.current_daylight,
            self.current_fog_color,
            self.brightness,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn raster_triangle_tile(
    tri: &Triangle,
    hurt_tint: f32,
    view_proj: &Mat4,
    width: usize,
    height: usize,
    tile_y_start: usize,
    tile_y_end: usize,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    textures: &TextureAtlas,
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
    brightness: f32,
) {
    let c0 = to_clip_ao(tri.v0, view_proj);
    let c1 = to_clip_ao(tri.v1, view_proj);
    let c2 = to_clip_ao(tri.v2, view_proj);

    let (clipped, clipped_len) = clip_near_ao(&[c0, c1, c2]);
    if clipped_len < 3 {
        return;
    }

    for i in 1..clipped_len - 1 {
        let sa = project_to_screen_ao(&clipped[0], width, height);
        let sb = project_to_screen_ao(&clipped[i], width, height);
        let sc = project_to_screen_ao(&clipped[i + 1], width, height);

        raster_screen_triangle_ao_tile(
            sa,
            sb,
            sc,
            tri.texture,
            hurt_tint,
            width,
            tile_y_start,
            tile_y_end,
            color_buffer,
            depth_buffer,
            textures,
            fog_dist,
            daylight,
            fog_color,
            brightness,
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn raster_screen_triangle_ao_tile(
    a: ScreenVertAO,
    b: ScreenVertAO,
    c: ScreenVertAO,
    texture: TextureId,
    hurt_tint: f32,
    width: usize,
    tile_y_start: usize,
    tile_y_end: usize,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    textures: &TextureAtlas,
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
    brightness: f32,
) {
    let Some((a, b, c, area)) = orient_screen_triangle(a, b, c, texture) else {
        return;
    };

    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min(width as f32 - 1.0) as i32;
    let min_y = (a.y.min(b.y).min(c.y).floor() as i32).max(tile_y_start as i32);
    let max_y = (a.y.max(b.y).max(c.y).ceil() as i32).min(tile_y_end as i32 - 1);

    if min_x > max_x || min_y > max_y {
        return;
    }

    let inv_area = 1.0 / area;
    let tex = textures.get(texture);
    let is_crack = texture.is_destroy_stage();
    let transparent =
        matches!(texture, TextureId::Water | TextureId::Glass | TextureId::EntityShadow) || is_crack;
    let is_water = texture == TextureId::Water;

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
        let row_offset = (py as usize - tile_y_start) * width;

        let mut px = min_x;
        while px + 3 <= max_x {
            raster_scanline_simd_4x(
                px,
                row_offset,
                w0,
                w1,
                w2,
                dy0,
                dy1,
                dy2,
                inv_area,
                &a,
                &b,
                &c,
                transparent,
                is_water,
                is_crack,
                hurt_tint,
                tex,
                color_buffer,
                depth_buffer,
                fog_dist,
                daylight,
                fog_color,
                brightness,
            );

            px += 4;
            w0 += dy0 * 4.0;
            w1 += dy1 * 4.0;
            w2 += dy2 * 4.0;
        }

        while px <= max_x {
            raster_scanline_pixel(
                px,
                row_offset,
                w0,
                w1,
                w2,
                inv_area,
                &a,
                &b,
                &c,
                transparent,
                is_water,
                is_crack,
                hurt_tint,
                tex,
                color_buffer,
                depth_buffer,
                fog_dist,
                daylight,
                fog_color,
                brightness,
            );

            px += 1;
            w0 += dy0;
            w1 += dy1;
            w2 += dy2;
        }

        w0_row -= dx0;
        w1_row -= dx1;
        w2_row -= dx2;
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn raster_scanline_simd_4x(
    px: i32,
    row_offset: usize,
    w0_start: f32,
    w1_start: f32,
    w2_start: f32,
    dy0: f32,
    dy1: f32,
    dy2: f32,
    inv_area: f32,
    a: &ScreenVertAO,
    b: &ScreenVertAO,
    c: &ScreenVertAO,
    transparent: bool,
    is_water: bool,
    is_crack: bool,
    hurt_tint: f32,
    tex: &Texture,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
    brightness: f32,
) {
    let w0 = [w0_start, w0_start + dy0, w0_start + dy0 * 2.0, w0_start + dy0 * 3.0];
    let w1 = [w1_start, w1_start + dy1, w1_start + dy1 * 2.0, w1_start + dy1 * 3.0];
    let w2 = [w2_start, w2_start + dy2, w2_start + dy2 * 2.0, w2_start + dy2 * 3.0];

    let mask = [
        w0[0] >= 0.0 && w1[0] >= 0.0 && w2[0] >= 0.0,
        w0[1] >= 0.0 && w1[1] >= 0.0 && w2[1] >= 0.0,
        w0[2] >= 0.0 && w1[2] >= 0.0 && w2[2] >= 0.0,
        w0[3] >= 0.0 && w1[3] >= 0.0 && w2[3] >= 0.0,
    ];

    if !mask[0] && !mask[1] && !mask[2] && !mask[3] {
        return;
    }

    for i in 0..4 {
        if mask[i] {
            shade_and_write_pixel(
                px + i as i32,
                row_offset,
                w0[i],
                w1[i],
                w2[i],
                inv_area,
                a,
                b,
                c,
                transparent,
                is_water,
                is_crack,
                hurt_tint,
                tex,
                color_buffer,
                depth_buffer,
                fog_dist,
                daylight,
                fog_color,
                brightness,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn raster_scanline_pixel(
    px: i32,
    row_offset: usize,
    w0: f32,
    w1: f32,
    w2: f32,
    inv_area: f32,
    a: &ScreenVertAO,
    b: &ScreenVertAO,
    c: &ScreenVertAO,
    transparent: bool,
    is_water: bool,
    is_crack: bool,
    hurt_tint: f32,
    tex: &Texture,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
    brightness: f32,
) {
    if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
        shade_and_write_pixel(
            px,
            row_offset,
            w0,
            w1,
            w2,
            inv_area,
            a,
            b,
            c,
            transparent,
            is_water,
            is_crack,
            hurt_tint,
            tex,
            color_buffer,
            depth_buffer,
            fog_dist,
            daylight,
            fog_color,
            brightness,
        );
    }
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn shade_and_write_pixel(
    px: i32,
    row_offset: usize,
    w0: f32,
    w1: f32,
    w2: f32,
    inv_area: f32,
    a: &ScreenVertAO,
    b: &ScreenVertAO,
    c: &ScreenVertAO,
    transparent: bool,
    is_water: bool,
    is_crack: bool,
    hurt_tint: f32,
    tex: &Texture,
    color_buffer: &mut [u32],
    depth_buffer: &mut [f32],
    fog_dist: f32,
    daylight: f32,
    fog_color: [f32; 3],
    brightness: f32,
) {
    let l0 = w0 * inv_area;
    let l1 = w1 * inv_area;
    let l2 = w2 * inv_area;

    let inv_w = l0 * a.inv_w + l1 * b.inv_w + l2 * c.inv_w;
    if inv_w <= 0.0 {
        return;
    }
    let depth = 1.0 / inv_w;
    let idx = row_offset + (px as usize);

    if depth >= depth_buffer[idx] {
        return;
    }

    let u = (l0 * a.uv_over_w.0 + l1 * b.uv_over_w.0 + l2 * c.uv_over_w.0) * depth;
    let v = (l0 * a.uv_over_w.1 + l1 * b.uv_over_w.1 + l2 * c.uv_over_w.1) * depth;

    let texel = tex.sample(u, v);
    if texel[0] < 0.0 {
        return;
    }

    if !transparent {
        depth_buffer[idx] = depth;
    }

    let ao = (l0 * a.ao_over_w + l1 * b.ao_over_w + l2 * c.ao_over_w) * depth;
    let sky = (l0 * a.sky_light_over_w + l1 * b.sky_light_over_w + l2 * c.sky_light_over_w) * depth;
    let block =
        (l0 * a.block_light_over_w + l1 * b.block_light_over_w + l2 * c.block_light_over_w) * depth;

    let [lr, lg, lb] = compute_entity_light_rgb(ao, sky, block, daylight, brightness);

    let mut r = texel[0] * lr;
    let mut g = texel[1] * lg;
    let mut bl = texel[2] * lb;

    if hurt_tint > 0.0 {
        r = r * (1.0 - hurt_tint) + 0.95 * hurt_tint;
        g = g * (1.0 - hurt_tint) + 0.12 * hurt_tint;
        bl = bl * (1.0 - hurt_tint) + 0.12 * hurt_tint;
    }

    let fog_t = if fog_dist > 0.0 {
        ((depth - fog_dist * 0.45) / (fog_dist * 0.55)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fr = r * (1.0 - fog_t) + fog_color[0] * fog_t;
    let fg = g * (1.0 - fog_t) + fog_color[1] * fog_t;
    let fb = bl * (1.0 - fog_t) + fog_color[2] * fog_t;

    if is_crack {
        let (br, bg, bb) = unpack_color(color_buffer[idx]);
        let alpha = 0.85;
        color_buffer[idx] = pack_color(
            fr * alpha + br * (1.0 - alpha),
            fg * alpha + bg * (1.0 - alpha),
            fb * alpha + bb * (1.0 - alpha),
        );
    } else if transparent {
        let (br, bg, bb) = unpack_color(color_buffer[idx]);
        let alpha = if is_water { 0.64 } else { 0.48 };
        color_buffer[idx] = pack_color(
            fr * alpha + br * (1.0 - alpha),
            fg * alpha + bg * (1.0 - alpha),
            fb * alpha + bb * (1.0 - alpha),
        );
    } else {
        color_buffer[idx] = pack_color(fr, fg, fb);
    }
}
