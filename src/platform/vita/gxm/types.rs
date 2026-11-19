//! GXM vertex formats, chunk buffer slot ranges, and render-pass texture classification.
//! Defines `GxmVertex`, `GxmPackedVertex`, `ChunkSlotRanges`, and `FrameRenderStats`.

#![allow(dead_code)]

use crate::graphics::atlas::TextureId;
use crate::world::mesher::Vertex;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GxmVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
    pub ao: f32,
}

impl GxmVertex {
    #[inline(always)]
    pub fn from_vertex(v: &Vertex, u0: f32, v0: f32, u1: f32, v1: f32, daylight: f32) -> Self {
        let u = u0 + (u1 - u0) * v.uv.0;
        let vv = v0 + (v1 - v0) * v.uv.1;
        // Feature parity with desktop WGSL lighting curve:
        // sky_factor = sky_light^1.25 * daylight, block_factor = block_light^1.15 * 1.30, cave ambient = 0.045
        let sky_factor = v.sky_light.max(0.0).powf(1.25) * daylight.clamp(0.05, 1.0);
        let block_factor = v.block_light.max(0.0).powf(1.15) * 1.30;
        let light = (sky_factor + block_factor + 0.045).clamp(0.04, 1.35) * v.ao;
        Self {
            x: v.pos.x,
            y: v.pos.y,
            z: v.pos.z,
            u,
            v: vv,
            ao: light.clamp(0.05, 1.25),
        }
    }
}

/// Compact packed 8-byte vertex format for ultra-fast GPU vertex streaming
#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
pub struct GxmPackedVertex {
    /// Local subchunk position (0..16 packed into 5 bits each)
    pub local_pos: u16,
    /// Texture coordinates (packed 16-bit fixed point)
    pub u: u16,
    pub v: u16,
    /// Ambient occlusion (4 bits), sky light (4 bits), block light (4 bits), face normal (4 bits)
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

/// Render pass index ranges within a single GPU chunk slot (Opaque -> Cutout -> Transparent)
#[derive(Clone, Copy, Default)]
pub(super) struct ChunkSlotRanges {
    pub(super) opaque_start: u32,
    pub(super) opaque_count: u32,
    pub(super) cutout_start: u32,
    pub(super) cutout_count: u32,
    pub(super) trans_start: u32,
    pub(super) trans_count: u32,
}

#[inline(always)]
pub(super) fn classify_texture_pass(tex: TextureId) -> u8 {
    match tex {
        TextureId::Water | TextureId::Glass => 2, // Transparent pass (alpha blend, depth write off)
        TextureId::Leaves
        | TextureId::SnowLeaves
        | TextureId::CherryLeaves
        | TextureId::AutumnLeaves
        | TextureId::LeavesSwamp
        | TextureId::LeavesSavanna
        | TextureId::Door
        | TextureId::DoorTop
        | TextureId::Torch
        | TextureId::Ladder
        | TextureId::RustStoneWire
        | TextureId::RustStoneWireLit
        | TextureId::Lever
        | TextureId::Button
        | TextureId::PressurePlate
        | TextureId::WheatCrop
        | TextureId::CarrotCrop
        | TextureId::PotatoCrop
        | TextureId::Sapling
        | TextureId::Kelp
        | TextureId::Seagrass
        | TextureId::DriedKelp => 1, // Cutout pass
        _ => 0, // Opaque pass (Early-Z / HSR active)
    }
}


/// Per-frame draw telemetry accumulated across clear, terrain, crack, and entity passes.
#[derive(Clone, Copy, Default)]
pub(super) struct FrameRenderStats {
    pub(super) tri_count: u32,
    pub(super) tri_culled: u32,
    pub(super) draw_calls: u32,
    pub(super) subchunks_rendered: u32,
    pub(super) subchunks_culled: u32,
}
