//! PS Vita hardware CDRAM double-buffered display presenter (`VitaDisplay`).
//! Allocates `USER_CDRAM_RW` framebuffers and presents 32-bit color buffers in non-GXM paths.

#![allow(dead_code)]

use super::ffi::*;

// ============================================================================
// 2. Vita Hardware Display & Framebuffer Manager
// ============================================================================

pub struct VitaDisplay {
    memblock_uid: [SceUID; 2],
    framebuffers: [*mut u32; 2],
    current_buffer: usize,
}

impl VitaDisplay {
    pub fn new() -> Result<Self, &'static str> {
        let fb_size = ((VITA_SCREEN_STRIDE * VITA_SCREEN_HEIGHT * 4 + 0xFFFFF) & !0xFFFFF) as i32;
        let mut memblock_uid = [0; 2];
        let mut framebuffers = [core::ptr::null_mut(); 2];

        for i in 0..2 {
            let name = if i == 0 {
                b"RustCraft_FB0\0".as_ptr()
            } else {
                b"RustCraft_FB1\0".as_ptr()
            };
            // Try allocating in CDRAM first (fastest video memory on PS Vita)
            let mut uid = unsafe {
                sceKernelAllocMemBlock(
                    name,
                    SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
                    fb_size as u32,
                    core::ptr::null(),
                )
            };
            // Fallback to standard user RAM if CDRAM is unavailable
            if uid < 0 {
                uid = unsafe {
                    sceKernelAllocMemBlock(
                        name,
                        SCE_KERNEL_MEMBLOCK_TYPE_USER_RW,
                        fb_size as u32,
                        core::ptr::null(),
                    )
                };
            }
            if uid < 0 {
                return Err("Failed to allocate framebuffer memory block for PS Vita");
            }

            let mut base: *mut core::ffi::c_void = core::ptr::null_mut();
            unsafe {
                let res = sceKernelGetMemBlockBase(uid, &mut base);
                if res != SCE_OK || base.is_null() {
                    return Err("Failed to query base pointer for framebuffer memory block");
                }
                // Zero out framebuffer memory
                core::ptr::write_bytes(
                    base as *mut u8,
                    0,
                    (VITA_SCREEN_STRIDE * VITA_SCREEN_HEIGHT * 4) as usize,
                );
            }

            memblock_uid[i] = uid;
            framebuffers[i] = base as *mut u32;
        }

        Ok(Self {
            memblock_uid,
            framebuffers,
            current_buffer: 0,
        })
    }

    /// Blits rendered game buffer (960x540) to Vita framebuffer (960x544) and flips the display buffer.
    pub fn present(&mut self, src_pixel_buffer: &[u32], game_width: usize, game_height: usize) {
        let draw_buf = self.framebuffers[self.current_buffer];
        if draw_buf.is_null() {
            return;
        }

        // Copy scanlines with vertical centering (2 pixels top/bottom margin)
        let y_offset = if VITA_SCREEN_HEIGHT > game_height {
            (VITA_SCREEN_HEIGHT - game_height) / 2
        } else {
            0
        };

        let copy_width = game_width.min(VITA_SCREEN_WIDTH);
        let copy_height = game_height.min(VITA_SCREEN_HEIGHT);

        unsafe {
            for y in 0..copy_height {
                let src_offset = y * game_width;
                let dst_offset = (y + y_offset) * VITA_SCREEN_STRIDE;
                let src_ptr = src_pixel_buffer.as_ptr().add(src_offset);
                let dst_ptr = draw_buf.add(dst_offset);
                // Pixel format conversion: Game produces 0x00RRGGBB, Vita expects ABGR8888 (0xFFBBGGRR)
                for px in 0..copy_width {
                    let xrgb = *src_ptr.add(px);
                    let r = (xrgb >> 16) & 0xFF;
                    let g = (xrgb >> 8) & 0xFF;
                    let b = xrgb & 0xFF;
                    // ABGR: A=0xFF, B, G, R
                    *dst_ptr.add(px) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                }
            }

            // Set framebuffer display registers and wait for VBlank
            let fb_struct = SceDisplayFrameBuf {
                size: core::mem::size_of::<SceDisplayFrameBuf>() as SceSize,
                base: draw_buf as *mut core::ffi::c_void,
                pitch: VITA_SCREEN_STRIDE as u32,
                pixelformat: SCE_DISPLAY_PIXELFORMAT_A8B8G8R8 as u32,
                width: VITA_SCREEN_WIDTH as u32,
                height: VITA_SCREEN_HEIGHT as u32,
            };

            sceDisplaySetFrameBuf(&fb_struct, SCE_DISPLAY_SETBUF_NEXTFRAME);
            sceDisplayWaitVblankStart();
        }

        // Double buffering buffer flip
        self.current_buffer = 1 - self.current_buffer;
    }
}

impl Drop for VitaDisplay {
    fn drop(&mut self) {
        for uid in self.memblock_uid {
            if uid > 0 {
                unsafe {
                    sceKernelFreeMemBlock(uid);
                }
            }
        }
    }
}

