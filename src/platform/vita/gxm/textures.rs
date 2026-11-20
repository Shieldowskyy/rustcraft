//! Linear ABGR8888 512x512 padded 256-tile GPU texture atlas and triple-buffered HUD textures.
//! Packs all built-in and mod `TextureAtlas` layers with an 8px wrapped border and uploads aligned ARGB->ABGR HUD layers.

#![allow(dead_code)]

use crate::features::rendering::atlas::{
    texel_to_rgba8, BUILTIN_TEXTURE_COUNT, CUSTOM_TEXTURE_LAYERS, TEX_SIZE,
};
use crate::graphics::atlas::{TextureAtlas, TextureId};

use super::ffi::*;
use super::mem::GpuMem;

// ============================================================
// 5. GPU TEXTURE ATLAS (512x512 Padded Linear ABGR8888, 256 Tiles)
// ============================================================

const ATLAS_TILES_PER_ROW: usize = 16;
const TILE_BORDER: usize = 8;
const CELL_SIZE: usize = TEX_SIZE + TILE_BORDER * 2; // 32x32 cell (8px wrapped padding + 16x16 core)
const ATLAS_SIZE: usize = ATLAS_TILES_PER_ROW * CELL_SIZE; // 512x512 px (1 MB CDRAM)
const TOTAL_ATLAS_LAYERS: u32 = BUILTIN_TEXTURE_COUNT as u32 + CUSTOM_TEXTURE_LAYERS;

pub(super) unsafe fn populate_atlas(pixels: *mut u32, atlas: &TextureAtlas) {
    for layer in 0..TOTAL_ATLAS_LAYERS {
        let tex_id = TextureId::from_layer(layer);
        let tex = atlas.get(tex_id);
        let tile_col = (layer as usize) % ATLAS_TILES_PER_ROW;
        let tile_row = (layer as usize) / ATLAS_TILES_PER_ROW;
        let cell_x = tile_col * CELL_SIZE;
        let cell_y = tile_row * CELL_SIZE;

        let is_semi_trans = matches!(
            tex_id,
            TextureId::Water | TextureId::Glass | TextureId::EntityShadow
        );
        let is_crack = tex_id.is_destroy_stage();

        for cy in 0..CELL_SIZE {
            // DLACZEGO: 8px wrapped padding isolates adjacent tiles under sub-texel interpolation
            // without requiring mipmaps (sceGxmTextureInitLinear rejects mipCount > 1 on SGX543).
            let src_y = (cy + TILE_BORDER) & (TEX_SIZE - 1);
            for cx in 0..CELL_SIZE {
                let src_x = (cx + TILE_BORDER) & (TEX_SIZE - 1);
                let [ri, gi, bi, base_a] = texel_to_rgba8(tex.texels[src_y * TEX_SIZE + src_x]);
                let a = if base_a == 0 {
                    0u32
                } else if is_semi_trans {
                    122u32 // ~48% opacity matching WGSL 0.48
                } else if is_crack {
                    217u32 // ~85% opacity matching software rasterizer 0.85
                } else {
                    255u32
                };
                let abgr = (a << 24) | ((bi as u32) << 16) | ((gi as u32) << 8) | (ri as u32);
                *pixels.add((cell_y + cy) * ATLAS_SIZE + (cell_x + cx)) = abgr;
            }
        }
    }
}

pub(super) fn build_gpu_texture(atlas: &TextureAtlas) -> (GpuMem, SceGxmTexture) {
    let tex_bytes = ATLAS_SIZE * ATLAS_SIZE * 4;
    let mem = GpuMem::alloc_gpu(
        b"RC_GXM_Atlas\0",
        tex_bytes,
        SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
        SCE_GXM_MEMORY_ATTRIB_READ,
    )
    .expect("Failed to allocate atlas texture in CDRAM");

    let pixels = mem.ptr as *mut u32;

    unsafe {
        core::ptr::write_bytes(pixels as *mut u8, 0, tex_bytes);
        populate_atlas(pixels, atlas);
    }

    let mut texture = SceGxmTexture { _private: [0u32; 4] };
    unsafe {
        sceGxmTextureInitLinear(
            &mut texture,
            mem.ptr,
            SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR,
            ATLAS_SIZE as u32,
            ATLAS_SIZE as u32,
            1,
        );
        sceGxmTextureSetMinFilter(&mut texture, SCE_GXM_TEXTURE_FILTER_POINT);
        sceGxmTextureSetMagFilter(&mut texture, SCE_GXM_TEXTURE_FILTER_POINT);
        sceGxmTextureSetUAddrMode(&mut texture, SCE_GXM_TEXTURE_ADDR_REPEAT);
        sceGxmTextureSetVAddrMode(&mut texture, SCE_GXM_TEXTURE_ADDR_REPEAT);
    }

    (mem, texture)
}

#[inline(always)]
pub(super) fn atlas_uv(tex_id: TextureId) -> (f32, f32, f32, f32) {
    let layer = (tex_id.layer() as usize).min((TOTAL_ATLAS_LAYERS as usize).saturating_sub(1));
    let col = layer % ATLAS_TILES_PER_ROW;
    let row = layer / ATLAS_TILES_PER_ROW;
    // Sub-texel inset keeps hardware point sampling inside the 16x16 inner tile
    let inset = 0.25 / ATLAS_SIZE as f32;
    let u0 = (col * CELL_SIZE + TILE_BORDER) as f32 / ATLAS_SIZE as f32 + inset;
    let v0 = (row * CELL_SIZE + TILE_BORDER) as f32 / ATLAS_SIZE as f32 + inset;
    let u1 = (col * CELL_SIZE + TILE_BORDER + TEX_SIZE) as f32 / ATLAS_SIZE as f32 - inset;
    let v1 = (row * CELL_SIZE + TILE_BORDER + TEX_SIZE) as f32 / ATLAS_SIZE as f32 - inset;
    (u0, v0, u1, v1)
}

/// Allocates triple-buffered 960x544 ABGR8888 HUD overlay textures in CDRAM.
pub(super) fn build_hud_textures() -> ([GpuMem; DISPLAY_BUFFER_COUNT], [SceGxmTexture; DISPLAY_BUFFER_COUNT]) {
    let tex_bytes = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
    let names: [&[u8]; DISPLAY_BUFFER_COUNT] = [b"RC_GXM_HUD0\0", b"RC_GXM_HUD1\0", b"RC_GXM_HUD2\0"];

    let bufs: [GpuMem; DISPLAY_BUFFER_COUNT] = [
        GpuMem::alloc_gpu(names[0], tex_bytes, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_GXM_MEMORY_ATTRIB_READ).unwrap(),
        GpuMem::alloc_gpu(names[1], tex_bytes, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_GXM_MEMORY_ATTRIB_READ).unwrap(),
        GpuMem::alloc_gpu(names[2], tex_bytes, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_GXM_MEMORY_ATTRIB_READ).unwrap(),
    ];

    let mut texs = [
        SceGxmTexture { _private: [0u32; 4] },
        SceGxmTexture { _private: [0u32; 4] },
        SceGxmTexture { _private: [0u32; 4] },
    ];

    unsafe {
        for i in 0..DISPLAY_BUFFER_COUNT {
            core::ptr::write_bytes(bufs[i].ptr as *mut u8, 0, tex_bytes);
            let mut rc = sceGxmTextureInitLinearStrided(
                &mut texs[i],
                bufs[i].ptr,
                SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR,
                VITA_WIDTH,
                VITA_HEIGHT,
                VITA_WIDTH * 4,
            );
            if rc != SCE_OK {
                rc = sceGxmTextureInitLinear(
                    &mut texs[i],
                    bufs[i].ptr,
                    SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR,
                    VITA_WIDTH,
                    VITA_HEIGHT,
                    1,
                );
            }
            assert_eq!(rc, SCE_OK, "Failed to init HUD texture {}: 0x{:08X}", i, rc);
            sceGxmTextureSetMinFilter(&mut texs[i], SCE_GXM_TEXTURE_FILTER_POINT);
            sceGxmTextureSetMagFilter(&mut texs[i], SCE_GXM_TEXTURE_FILTER_POINT);
            sceGxmTextureSetUAddrMode(&mut texs[i], SCE_GXM_TEXTURE_ADDR_CLAMP);
            sceGxmTextureSetVAddrMode(&mut texs[i], SCE_GXM_TEXTURE_ADDR_CLAMP);
        }
    }

    (bufs, texs)
}

#[inline]
fn find_nonzero_span(row: &[u32]) -> (u16, u16) {
    let n = row.len();
    let mut i = 0usize;
    while i + 4 <= n {
        if (row[i] | row[i + 1] | row[i + 2] | row[i + 3]) != 0 {
            break;
        }
        i += 4;
    }
    while i < n && row[i] == 0 {
        i += 1;
    }
    if i >= n {
        return (0, 0);
    }
    let mut j = n;
    while j >= i + 4 {
        if (row[j - 4] | row[j - 3] | row[j - 2] | row[j - 1]) != 0 {
            break;
        }
        j -= 4;
    }
    while j > i && row[j - 1] == 0 {
        j -= 1;
    }
    (i as u16, j as u16)
}

#[inline]
unsafe fn zero_u32_range(dst_row: *mut u32, start: usize, end: usize) {
    core::ptr::write_bytes(dst_row.add(start), 0, end - start);
}

#[inline]
unsafe fn copy_u32_range(dst_row: *mut u32, src_row: *const u32, start: usize, end: usize) {
    core::ptr::copy_nonoverlapping(src_row.add(start), dst_row.add(start), end - start);
}

/// Copies dirty non-zero ARGB8888 HUD spans to a 960x544 ABGR8888 GXM texture using 32-bit aligned writes.
pub(super) unsafe fn upload_hud_buffer(
    dst: *mut u32,
    src: &[u32],
    width: usize,
    height: usize,
    prev_bounds: &mut [(u16, u16); VITA_HEIGHT as usize],
) {
    let vita_w = VITA_WIDTH as usize;
    let vita_h = VITA_HEIGHT as usize;
    let copy_w = width.min(vita_w);
    let copy_h = height.min(vita_h).min(src.len() / width.max(1));

    for y in 0..copy_h {
        let row_slice = &src[y * width..y * width + copy_w];
        let (new_min, new_max) = find_nonzero_span(row_slice);
        let (prev_min, prev_max) = prev_bounds[y];
        let dst_row = dst.add(y * vita_w);

        if new_min >= new_max {
            if prev_min < prev_max {
                zero_u32_range(dst_row, prev_min as usize, prev_max as usize);
            }
        } else {
            if prev_min < prev_max {
                if prev_max <= new_min || new_max <= prev_min {
                    zero_u32_range(dst_row, prev_min as usize, prev_max as usize);
                } else {
                    if prev_min < new_min {
                        zero_u32_range(dst_row, prev_min as usize, new_min as usize);
                    }
                    if new_max < prev_max {
                        zero_u32_range(dst_row, new_max as usize, prev_max as usize);
                    }
                }
            }
            copy_u32_range(dst_row, row_slice.as_ptr(), new_min as usize, new_max as usize);
        }
        prev_bounds[y] = (new_min, new_max);
    }

    for y in copy_h..vita_h {
        let (prev_min, prev_max) = prev_bounds[y];
        if prev_min < prev_max {
            zero_u32_range(dst.add(y * vita_w), prev_min as usize, prev_max as usize);
            prev_bounds[y] = (0, 0);
        }
    }
}
