//! GXM vertex formats, chunk buffer slot ranges, and frame telemetry.
//! Defines `GxmVertex`, `GxmEntityVertex`, `GxmPackedVertex`, `ChunkSlotRanges`, and `FrameRenderStats`.

#![allow(dead_code)]

use crate::world::mesher::Vertex;

/// 24-byte chunk terrain vertex (`SCE_GXM_ATTRIBUTE_FORMAT_U8N` light attribute).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GxmVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    /// Normalized `[ao, sky_light, block_light, 255]`.
    pub light: [u8; 4],
}

const _: () = assert!(core::mem::size_of::<GxmVertex>() == 24);

impl GxmVertex {
    #[inline(always)]
    pub fn from_vertex(v: &Vertex, u0: f32, v0: f32, u1: f32, v1: f32) -> Self {
        let u = u0 + (u1 - u0) * v.uv.0;
        let vv = v0 + (v1 - v0) * v.uv.1;
        Self {
            x: v.pos.x,
            y: v.pos.y,
            z: v.pos.z,
            u,
            v: vv,
            light: [
                (v.ao.clamp(0.0, 1.0) * 255.0).round() as u8,
                (v.sky_light.clamp(0.0, 1.0) * 255.0).round() as u8,
                (v.block_light.clamp(0.0, 1.0) * 255.0).round() as u8,
                255,
            ],
        }
    }
}

/// 40-byte dynamic geometry vertex for entities, players, clouds, weather, and first-person hand.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GxmEntityVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Atlas UV coordinates, or `u < 0.0` for untextured flat-color geometry.
    pub u: f32,
    pub v: f32,
    /// HDR RGB lighting multiplier (`0.04..1.45`) and alpha.
    pub color: [f32; 4],
    /// Normalized flash/hurt tint overlay `[r, g, b, mix_factor]` (`SCE_GXM_ATTRIBUTE_FORMAT_U8N`).
    pub tint: [u8; 4],
}

const _: () = assert!(core::mem::size_of::<GxmEntityVertex>() == 40);

/// Compact packed 8-byte vertex format reserved for streaming experiments.
#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct GxmPackedVertex {
    pub local_pos: u16,
    pub u: u16,
    pub v: u16,
    pub lighting_ao: u16,
}

impl GxmPackedVertex {
    #[inline(always)]
    pub fn new(lx: u8, ly: u8, lz: u8, u: u16, v: u16, ao: u8, sky_light: u8, block_light: u8, face_idx: u8) -> Self {
        let local_pos = ((lx as u16 & 0x1F) << 10) | ((ly as u16 & 0x1F) << 5) | (lz as u16 & 0x1F);
        let lighting_ao = ((ao as u16 & 0xF) << 12)
            | ((sky_light as u16 & 0xF) << 8)
            | ((block_light as u16 & 0xF) << 4)
            | (face_idx as u16 & 0xF);
        Self {
            local_pos,
            u,
            v,
            lighting_ao,
        }
    }
}

/// Render pass index ranges within a single GPU chunk slot (Opaque -> Cutout -> Transparent).
#[derive(Clone, Copy, Default)]
pub(super) struct ChunkSlotRanges {
    pub(super) opaque_start: u32,
    pub(super) opaque_count: u32,
    pub(super) cutout_start: u32,
    pub(super) cutout_count: u32,
    pub(super) trans_start: u32,
    pub(super) trans_count: u32,
}

/// Per-frame draw telemetry accumulated across clear, sky, terrain, entity, postfx, and HUD passes.
#[derive(Clone, Copy, Default)]
pub(super) struct FrameRenderStats {
    pub(super) tri_count: u32,
    pub(super) tri_culled: u32,
    pub(super) draw_calls: u32,
    pub(super) subchunks_rendered: u32,
    pub(super) subchunks_culled: u32,
}
