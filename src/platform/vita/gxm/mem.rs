//! PS Vita CDRAM, uncached kernel RAM, and USSE GPU memory allocator (`GpuMem`).
//! Manages mapping/unmapping via `sceGxmMapMemory` and vsync display queue callback.

#![allow(dead_code)]

use super::ffi::*;

// ============================================================
// 2. SAFE GPU MEMORY ALLOCATION FOR PS VITA
// ============================================================

#[derive(Debug, PartialEq)]
pub(super) enum MemKind {
    GxmMap,
    VertexUsse,
    FragmentUsse,
}

pub(super) struct GpuMem {
    pub uid: SceUID,
    pub ptr: *mut core::ffi::c_void,
    pub size: usize,
    pub is_cdram: bool,
    kind: MemKind,
}

impl GpuMem {
    fn align_size(mem_type: u32, size: usize) -> usize {
        match mem_type {
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW => (size + 256 * 1024 - 1) & !(256 * 1024 - 1),
            _ => (size + 4 * 1024 - 1) & !(4 * 1024 - 1),
        }
    }

    /// Allocates a kernel memblock (with CDRAM -> uncached fallback), then maps it via `map`.
    fn alloc_mapped(
        name: &[u8],
        size: usize,
        mem_type: u32,
        kind: MemKind,
        map: impl FnOnce(*mut core::ffi::c_void, u32) -> i32,
    ) -> Option<Self> {
        let mut is_cdram = mem_type == SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW;
        let mut aligned = Self::align_size(mem_type, size);
        let mut uid = unsafe {
            sceKernelAllocMemBlock(name.as_ptr(), mem_type, aligned as u32, core::ptr::null())
        };

        if uid < 0 && is_cdram {
            is_cdram = false;
            aligned = Self::align_size(SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, size);
            uid = unsafe {
                sceKernelAllocMemBlock(name.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, aligned as u32, core::ptr::null())
            };
        }
        if uid < 0 {
            crate::vita_log!(
                "GpuMem::alloc_mapped failed for {:?} (size: {} bytes, uid: 0x{:08X})",
                core::str::from_utf8(name).unwrap_or("?"),
                aligned,
                uid
            );
            return None;
        }

        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK || map(ptr, aligned as u32) != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        Some(Self { uid, ptr, size: aligned, is_cdram, kind })
    }

    #[inline(always)]
    pub fn cdram_bytes(&self) -> usize {
        if self.is_cdram { self.size } else { 0 }
    }

    #[inline(always)]
    pub fn ram_bytes(&self) -> usize {
        if self.is_cdram { 0 } else { self.size }
    }

    pub fn alloc_gpu(name: &[u8], size: usize, mem_type: u32, attribs: u32) -> Option<Self> {
        Self::alloc_mapped(name, size, mem_type, MemKind::GxmMap, |p, s| unsafe { sceGxmMapMemory(p, s, attribs) })
    }

    pub fn alloc_vertex_usse(name: &[u8], size: usize, out_offset: &mut u32) -> Option<Self> {
        Self::alloc_mapped(name, size, SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, MemKind::VertexUsse, |p, s| unsafe {
            sceGxmMapVertexUsseMemory(p, s, out_offset)
        })
    }

    pub fn alloc_fragment_usse(name: &[u8], size: usize, out_offset: &mut u32) -> Option<Self> {
        Self::alloc_mapped(name, size, SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, MemKind::FragmentUsse, |p, s| unsafe {
            sceGxmMapFragmentUsseMemory(p, s, out_offset)
        })
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.ptr as *mut T
    }
}

impl Drop for GpuMem {
    fn drop(&mut self) {
        unsafe {
            match self.kind {
                MemKind::GxmMap => {
                    let _ = sceGxmUnmapMemory(self.ptr);
                }
                MemKind::VertexUsse => {
                    let _ = sceGxmUnmapVertexUsseMemory(self.ptr);
                }
                MemKind::FragmentUsse => {
                    let _ = sceGxmUnmapFragmentUsseMemory(self.ptr);
                }
            }
            let _ = sceKernelFreeMemBlock(self.uid);
        }
    }
}

// ============================================================
// 3. DISPLAY CALLBACK (Async Swap)
// ============================================================

#[repr(C)]
pub(super) struct DisplayData {
    pub(super) address: *mut core::ffi::c_void,
}

pub(super) unsafe extern "C" fn display_callback(data: *const core::ffi::c_void) {
    let d = &*(data as *const DisplayData);
    let fb = SceDisplayFrameBuf {
        size: core::mem::size_of::<SceDisplayFrameBuf>() as u32,
        base: d.address,
        pitch: VITA_STRIDE,
        pixelformat: 0, // SCE_DISPLAY_PIXELFORMAT_A8B8G8R8
        width: VITA_WIDTH,
        height: VITA_HEIGHT,
    };
    sceDisplaySetFrameBuf(&fb, 1 /* SCE_DISPLAY_SETBUF_NEXTFRAME */);
}

