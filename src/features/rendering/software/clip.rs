//! Near-plane Sutherland-Hodgman polygon clipping, screen projection, and color packing helpers.
//! Zero-allocation stack clipping inside hot rasterizer paths; zero platform `#[cfg]`.

use crate::core::math::{Mat4, Vec3};
use crate::features::meshing::Vertex;
use crate::features::rendering::atlas::TextureId;

use super::types::{ClipVertAO, ScreenVertAO};

#[inline(always)]
pub(super) fn to_clip_ao(v: Vertex, view_proj: &Mat4) -> ClipVertAO {
    let c = view_proj.transform_point(v.pos);
    ClipVertAO {
        clip: c,
        uv: v.uv,
        ao: v.ao,
        sky_light: v.sky_light,
        block_light: v.block_light,
    }
}

/// Clips a triangle against the near plane (`w > 0.05`) on the stack without heap allocations.
///
/// // DLACZEGO: Returning a stack array `([ClipVertAO; 4], usize)` instead of `Vec<ClipVertAO>`
/// eliminates per-triangle heap allocations inside the hot chunk & cloud rasterization loops.
#[inline]
pub(super) fn clip_near_ao(verts: &[ClipVertAO; 3]) -> ([ClipVertAO; 4], usize) {
    const EPS: f32 = 0.05;
    let inside = |v: &ClipVertAO| v.clip[3] > EPS;

    let mut output = [verts[0]; 4];
    let mut len = 0usize;
    for i in 0..3 {
        let cur = verts[i];
        let next = verts[(i + 1) % 3];
        let cur_in = inside(&cur);
        let next_in = inside(&next);

        if cur_in {
            output[len] = cur;
            len += 1;
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
            let sky_light = lerp(cur.sky_light, next.sky_light, t);
            let block_light = lerp(cur.block_light, next.block_light, t);
            output[len] = ClipVertAO {
                clip,
                uv,
                ao,
                sky_light,
                block_light,
            };
            len += 1;
        }
    }
    (output, len)
}

#[inline(always)]
pub(super) fn project_to_screen_ao(v: &ClipVertAO, width: usize, height: usize) -> ScreenVertAO {
    let inv_w = 1.0 / v.clip[3];
    let ndc_x = v.clip[0] * inv_w;
    let ndc_y = v.clip[1] * inv_w;
    let screen_x = (ndc_x * 0.5 + 0.5) * width as f32;
    let screen_y = (1.0 - (ndc_y * 0.5 + 0.5)) * height as f32;
    ScreenVertAO {
        x: screen_x,
        y: screen_y,
        inv_w,
        uv_over_w: (v.uv.0 * inv_w, v.uv.1 * inv_w),
        ao_over_w: v.ao * inv_w,
        sky_light_over_w: v.sky_light * inv_w,
        block_light_over_w: v.block_light * inv_w,
    }
}

#[inline(always)]
pub(super) fn orient_screen_triangle(
    a: ScreenVertAO,
    b: ScreenVertAO,
    c: ScreenVertAO,
    texture: TextureId,
) -> Option<(ScreenVertAO, ScreenVertAO, ScreenVertAO, f32)> {
    let area = edge(a.x, a.y, b.x, b.y, c.x, c.y);
    if area > 0.0 {
        Some((a, b, c, area))
    } else if area < 0.0 && texture == TextureId::Water {
        Some((a, c, b, -area))
    } else {
        None
    }
}

#[inline]
pub(super) fn cloud_column_visible(
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

    let mut out_x_pos = 0;
    let mut out_x_neg = 0;
    let mut out_y_pos = 0;
    let mut out_y_neg = 0;
    let mut out_z_pos = 0;
    let mut out_z_neg = 0;

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

#[inline(always)]
pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline(always)]
pub(super) fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

#[inline(always)]
pub(super) fn pack_color(r: f32, g: f32, b: f32) -> u32 {
    let r = (r.clamp(0.0, 1.0) * 255.0) as u32;
    let g = (g.clamp(0.0, 1.0) * 255.0) as u32;
    let b = (b.clamp(0.0, 1.0) * 255.0) as u32;
    (r << 16) | (g << 8) | b
}

#[inline(always)]
pub(super) fn unpack_color(c: u32) -> (f32, f32, f32) {
    let r = ((c >> 16) & 0xFF) as f32 / 255.0;
    let g = ((c >> 8) & 0xFF) as f32 / 255.0;
    let b = (c & 0xFF) as f32 / 255.0;
    (r, g, b)
}
