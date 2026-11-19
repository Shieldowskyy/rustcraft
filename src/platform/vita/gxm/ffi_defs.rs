//! C ABI opaque handles, enum constants, descriptor structs, and `SceGxmBlendInfo` for `libgxm`.
//! Pure declarations re-exported through `ffi.rs` inside `platform::vita::gxm`.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

pub type SceUID = i32;
pub type SceSize = u32;
pub type SceBool = i32;
pub const SCE_OK: i32 = 0;

#[repr(C)]
pub struct SceGxmContext(u8);
#[repr(C)]
pub struct SceGxmRenderTarget(u8);
#[repr(C)]
pub struct SceGxmShaderPatcher(u8);
#[repr(C)]
pub struct SceGxmVertexProgram(u8);
#[repr(C)]
pub struct SceGxmFragmentProgram(u8);
#[repr(C)]
pub struct SceGxmSyncObject(u8);
#[repr(C)]
pub struct SceGxmProgram(u8);
#[repr(C)]
pub struct SceGxmProgramParameter(u8);

pub type SceGxmShaderPatcherId = *mut core::ffi::c_void;
pub type SceGxmDisplayQueueCallback = unsafe extern "C" fn(callbackData: *const core::ffi::c_void);

// GXM ring buffer size constants (tuned for PS Vita 256MB RAM / 112MB CDRAM limits)
pub const SCE_GXM_MINIMUM_CONTEXT_HOST_MEM_SIZE: usize = 2 * 1024;
pub const SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE: usize = 16 * 1024 * 1024;
pub const SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE: usize = 128 * 1024;
pub const SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE: usize = 2 * 1024 * 1024;
pub const SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE: usize = 512 * 1024;
pub const SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE: usize = 16 * 1024;
pub const SCE_GXM_DEFAULT_PATCHER_BUFFER_SIZE: usize = 64 * 1024;
pub const SCE_GXM_DEFAULT_PATCHER_VERTEX_USSE_SIZE: usize = 64 * 1024;
pub const SCE_GXM_DEFAULT_PATCHER_FRAGMENT_USSE_SIZE: usize = 64 * 1024;

// Surface, primitive & raster state constants
pub const SCE_GXM_COLOR_FORMAT_A8B8G8R8: u32 = 0x00000000;
pub const SCE_GXM_COLOR_SURFACE_LINEAR: u32 = 0x00000000;
pub const SCE_GXM_COLOR_SURFACE_SCALE_NONE: u32 = 0x00000000;
pub const SCE_GXM_OUTPUT_REGISTER_SIZE_32BIT: u32 = 0x00000000;
pub const SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4: u32 = 1;
pub const SCE_GXM_DEPTH_STENCIL_FORMAT_DF32: u32 = 0x00044000;
pub const SCE_GXM_DEPTH_STENCIL_SURFACE_LINEAR: u32 = 0x00000000;
pub const SCE_GXM_DEPTH_STENCIL_SURFACE_TILED: u32 = 0x00011000;
pub const SCE_GXM_MULTISAMPLE_NONE: u32 = 0;
pub const SCE_GXM_COLOR_SURFACE_DITHER_ENABLED: u32 = 0x00000008;
pub const SCE_GXM_PRIMITIVE_TRIANGLES: u32 = 0x00000000;

pub const SCE_GXM_INDEX_FORMAT_U16: u32 = 0x00000000;
pub const SCE_GXM_INDEX_SOURCE_INDEX_16BIT: u16 = 0;

pub const SCE_GXM_MEMORY_ATTRIB_READ: u32 = 1;
pub const SCE_GXM_MEMORY_ATTRIB_RW: u32 = 3;

pub const SCE_GXM_CULL_NONE: u32 = 0;
pub const SCE_GXM_CULL_CW: u32 = 1;
pub const SCE_GXM_CULL_CCW: u32 = 2;
pub const SCE_GXM_ATTRIBUTE_FORMAT_U8N: u32 = 4;
pub const SCE_GXM_ATTRIBUTE_FORMAT_F32: u32 = 9;

pub const SCE_GXM_DEPTH_FUNC_LESS: u32 = 0x00400000;
pub const SCE_GXM_DEPTH_FUNC_EQUAL: u32 = 0x00800000;
pub const SCE_GXM_DEPTH_FUNC_LESS_EQUAL: u32 = 0x00C00000;
pub const SCE_GXM_DEPTH_FUNC_ALWAYS_PASS: u32 = 0x01C00000;

pub const SCE_GXM_DEPTH_WRITE_ENABLED: u32 = 0x00000000;
pub const SCE_GXM_DEPTH_WRITE_DISABLED: u32 = 0x00100000;
pub const SCE_GXM_TWO_SIDED_DISABLED: u32 = 0x00000000;
pub const SCE_GXM_TWO_SIDED_ENABLED: u32 = 0x00000800;
pub const SCE_GXM_POLYGON_MODE_TRIANGLE_FILL: u32 = 0x00000000;
pub const SCE_GXM_VIEWPORT_ENABLED: u32 = 0x00000000;

pub const SCE_GXM_TEXTURE_BASE_FORMAT_U8U8U8U8: u32 = 0x0C000000;
pub const SCE_GXM_TEXTURE_SWIZZLE4_ABGR: u32 = 0x00000000;
pub const SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR: u32 = SCE_GXM_TEXTURE_BASE_FORMAT_U8U8U8U8 | SCE_GXM_TEXTURE_SWIZZLE4_ABGR;
pub const SCE_GXM_TEXTURE_FILTER_POINT: u32 = 0;
pub const SCE_GXM_TEXTURE_ADDR_REPEAT: u32 = 0;
pub const SCE_GXM_TEXTURE_ADDR_CLAMP: u32 = 2;

pub const SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW: u32 = 0x09408060;
pub const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE: u32 = 0x0C208060;

pub const VITA_WIDTH: u32 = 960;
pub const VITA_HEIGHT: u32 = 544;
pub const VITA_STRIDE: u32 = 960;
pub const DISPLAY_BUFFER_COUNT: usize = 3;
pub const MAX_PENDING_SWAPS: u32 = DISPLAY_BUFFER_COUNT as u32 - 1;

#[repr(C)]
pub struct SceGxmInitializeParams {
    pub flags: u32,
    pub displayQueueMaxPendingCount: u32,
    pub displayQueueCallback: Option<SceGxmDisplayQueueCallback>,
    pub displayQueueCallbackDataSize: u32,
    pub parameterBufferSize: u32,
}

#[repr(C)]
pub struct SceGxmContextParams {
    pub hostMem: *mut core::ffi::c_void,
    pub hostMemSize: u32,
    pub vdmRingBufferMem: *mut core::ffi::c_void,
    pub vdmRingBufferMemSize: u32,
    pub vertexRingBufferMem: *mut core::ffi::c_void,
    pub vertexRingBufferMemSize: u32,
    pub fragmentRingBufferMem: *mut core::ffi::c_void,
    pub fragmentRingBufferMemSize: u32,
    pub fragmentUsseRingBufferMem: *mut core::ffi::c_void,
    pub fragmentUsseRingBufferMemSize: u32,
    pub fragmentUsseRingBufferOffset: u32,
}

#[repr(C)]
pub struct SceGxmRenderTargetParams {
    pub flags: u32,
    pub width: u16,
    pub height: u16,
    pub scenesPerFrame: u16,
    pub multisampleMode: u16,
    pub multisampleLocations: u32,
    pub driverMemBlock: SceUID,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SceGxmTexture {
    pub(super) _private: [u32; 4],
}

#[repr(C)]
pub struct SceGxmColorSurface {
    pub(super) _private: [u32; 12],
}

#[repr(C)]
pub struct SceGxmDepthStencilSurface {
    pub(super) _private: [u32; 5],
}

#[repr(C)]
pub struct SceGxmShaderPatcherParams {
    pub userData: *mut core::ffi::c_void,
    pub hostAllocCallback: Option<unsafe extern "C" fn(userData: *mut core::ffi::c_void, size: u32) -> *mut core::ffi::c_void>,
    pub hostFreeCallback: Option<unsafe extern "C" fn(userData: *mut core::ffi::c_void, mem: *mut core::ffi::c_void)>,
    pub bufferAllocCallback: *const core::ffi::c_void,
    pub bufferFreeCallback: *const core::ffi::c_void,
    pub bufferMem: *mut core::ffi::c_void,
    pub bufferMemSize: u32,
    pub vertexUsseAllocCallback: *const core::ffi::c_void,
    pub vertexUsseFreeCallback: *const core::ffi::c_void,
    pub vertexUsseMem: *mut core::ffi::c_void,
    pub vertexUsseMemSize: u32,
    pub vertexUsseOffset: u32,
    pub fragmentUsseAllocCallback: *const core::ffi::c_void,
    pub fragmentUsseFreeCallback: *const core::ffi::c_void,
    pub fragmentUsseMem: *mut core::ffi::c_void,
    pub fragmentUsseMemSize: u32,
    pub fragmentUsseOffset: u32,
}

#[repr(C)]
pub struct SceGxmVertexAttribute {
    pub streamIndex: u16,
    pub offset: u16,
    pub format: u8,
    pub componentCount: u8,
    pub regIndex: u16,
}

#[repr(C)]
pub struct SceGxmVertexStream {
    pub stride: u16,
    pub indexSource: u16,
}

pub const SCE_GXM_COLOR_MASK_NONE: u8 = 0;
pub const SCE_GXM_COLOR_MASK_A: u8 = 1 << 0;
pub const SCE_GXM_COLOR_MASK_R: u8 = 1 << 1;
pub const SCE_GXM_COLOR_MASK_G: u8 = 1 << 2;
pub const SCE_GXM_COLOR_MASK_B: u8 = 1 << 3;
pub const SCE_GXM_COLOR_MASK_ALL: u8 = SCE_GXM_COLOR_MASK_A | SCE_GXM_COLOR_MASK_B | SCE_GXM_COLOR_MASK_G | SCE_GXM_COLOR_MASK_R;

pub const SCE_GXM_BLEND_FUNC_ADD: u8 = 1;
pub const SCE_GXM_BLEND_FACTOR_ZERO: u8 = 0;
pub const SCE_GXM_BLEND_FACTOR_ONE: u8 = 1;
pub const SCE_GXM_BLEND_FACTOR_SRC_ALPHA: u8 = 4;
pub const SCE_GXM_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA: u8 = 5;
pub const SCE_GXM_BLEND_FACTOR_DST_COLOR: u8 = 6;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SceGxmBlendInfo {
    pub color_mask: u8,
    pub func_info: u8,
    pub color_factors: u8,
    pub alpha_factors: u8,
}

impl SceGxmBlendInfo {
    pub fn alpha_blend() -> Self {
        Self {
            color_mask: SCE_GXM_COLOR_MASK_ALL,
            func_info: SCE_GXM_BLEND_FUNC_ADD | (SCE_GXM_BLEND_FUNC_ADD << 4),
            color_factors: SCE_GXM_BLEND_FACTOR_SRC_ALPHA | (SCE_GXM_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA << 4),
            alpha_factors: SCE_GXM_BLEND_FACTOR_ONE | (SCE_GXM_BLEND_FACTOR_ZERO << 4),
        }
    }

    pub fn premultiplied_alpha() -> Self {
        Self {
            color_mask: SCE_GXM_COLOR_MASK_ALL,
            func_info: SCE_GXM_BLEND_FUNC_ADD | (SCE_GXM_BLEND_FUNC_ADD << 4),
            color_factors: SCE_GXM_BLEND_FACTOR_ONE | (SCE_GXM_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA << 4),
            alpha_factors: SCE_GXM_BLEND_FACTOR_ONE | (SCE_GXM_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA << 4),
        }
    }

    pub fn multiply() -> Self {
        Self {
            color_mask: SCE_GXM_COLOR_MASK_ALL,
            func_info: SCE_GXM_BLEND_FUNC_ADD | (SCE_GXM_BLEND_FUNC_ADD << 4),
            color_factors: SCE_GXM_BLEND_FACTOR_DST_COLOR | (SCE_GXM_BLEND_FACTOR_ZERO << 4),
            alpha_factors: SCE_GXM_BLEND_FACTOR_ZERO | (SCE_GXM_BLEND_FACTOR_ONE << 4),
        }
    }

    pub fn multiply_add() -> Self {
        Self {
            color_mask: SCE_GXM_COLOR_MASK_ALL,
            func_info: SCE_GXM_BLEND_FUNC_ADD | (SCE_GXM_BLEND_FUNC_ADD << 4),
            color_factors: SCE_GXM_BLEND_FACTOR_DST_COLOR | (SCE_GXM_BLEND_FACTOR_ONE << 4),
            alpha_factors: SCE_GXM_BLEND_FACTOR_ZERO | (SCE_GXM_BLEND_FACTOR_ONE << 4),
        }
    }

    pub fn depth_only_no_color() -> Self {
        Self {
            color_mask: SCE_GXM_COLOR_MASK_NONE,
            func_info: 0,
            color_factors: 0,
            alpha_factors: 0,
        }
    }
}

#[repr(C)]
pub struct SceDisplayFrameBuf {
    pub size: u32,
    pub base: *mut core::ffi::c_void,
    pub pitch: u32,
    pub pixelformat: u32,
    pub width: u32,
    pub height: u32,
}
