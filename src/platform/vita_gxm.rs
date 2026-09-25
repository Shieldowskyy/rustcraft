// =============================================================================
// RustCraft – PS Vita GXM Hardware Renderer
// src/platform/vita_gxm.rs
//
// Implementuje pełny sprzętowy rendering 3D przez libgxm (SGX543MP4 GPU PS Vita).
// Pipeline:
//   1. sceGxmInitialize
//   2. Alokacja ring bufferów GPU (VDM, Vertex, Fragment, Fragment USSE)
//   3. sceGxmCreateContext z hostMem (malloc)
//   4. Alokacja potrójnego bufora ramki CDRAM + Color/Depth Surfaces + RenderTarget
//   5. ShaderPatcher + rejestracja skompilowanych GXP shaderów (voxel & clear)
//   6. Generowanie tekstury atlasu GPU w formacie ABGR8888
//   7. Renderowanie klatki: BeginScene -> Hardware Clear -> Draw Chunks -> EndScene -> Display Queue
// =============================================================================

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

use crate::core::math::Vec3;
use crate::graphics::atlas::{TextureAtlas, TEX_SIZE, TextureId};
use crate::graphics::camera::Camera;
use crate::world::mesher::{build_chunk_mesh, Vertex};
use crate::world::{BlockId, World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
use std::collections::HashMap;

// ============================================================
// 1. FFI TYPY I STAŁE VITA GXM
// ============================================================

type SceUID = i32;
type SceSize = u32;
type SceBool = i32;
const SCE_OK: i32 = 0;

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

// Rozmiary ring bufferów GXM
const SCE_GXM_MINIMUM_CONTEXT_HOST_MEM_SIZE: usize = 2 * 1024;
const SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE: usize = 32 * 1024 * 1024;
const SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE: usize = 512 * 1024;
const SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE: usize = 8 * 1024 * 1024;
const SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE: usize = 2 * 1024 * 1024;
const SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE: usize = 64 * 1024;
const SCE_GXM_DEFAULT_PATCHER_BUFFER_SIZE: usize = 256 * 1024;
const SCE_GXM_DEFAULT_PATCHER_VERTEX_USSE_SIZE: usize = 256 * 1024;
const SCE_GXM_DEFAULT_PATCHER_FRAGMENT_USSE_SIZE: usize = 256 * 1024;

// Formaty i flagi
const SCE_GXM_COLOR_FORMAT_A8B8G8R8: u32 = 0x00000000;
const SCE_GXM_COLOR_SURFACE_LINEAR: u32 = 0x00000000;
const SCE_GXM_COLOR_SURFACE_SCALE_NONE: u32 = 0x00000000;
const SCE_GXM_OUTPUT_REGISTER_SIZE_32BIT: u32 = 0x00000000;
const SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4: u32 = 1;
const SCE_GXM_DEPTH_STENCIL_FORMAT_DF32: u32 = 0x00044000;
const SCE_GXM_DEPTH_STENCIL_SURFACE_LINEAR: u32 = 0x00000000;
const SCE_GXM_MULTISAMPLE_NONE: u32 = 0;
const SCE_GXM_MULTISAMPLE_2X: u32 = 1;
const SCE_GXM_MULTISAMPLE_4X: u32 = 2;
const SCE_GXM_COLOR_SURFACE_DITHER_DISABLED: u32 = 0x00000000;
const SCE_GXM_COLOR_SURFACE_DITHER_ENABLED: u32 = 0x00000008;
const SCE_GXM_PRIMITIVE_TRIANGLES: u32 = 0x00000000;

const SCE_GXM_INDEX_FORMAT_U16: u32 = 0x00000000;
const SCE_GXM_INDEX_SOURCE_INDEX_16BIT: u16 = 0;

const SCE_GXM_MEMORY_ATTRIB_READ: u32 = 1;
const SCE_GXM_MEMORY_ATTRIB_WRITE: u32 = 2;
const SCE_GXM_MEMORY_ATTRIB_RW: u32 = 3;

const SCE_GXM_CULL_NONE: u32 = 0;
const SCE_GXM_CULL_CW: u32 = 1;
const SCE_GXM_ATTRIBUTE_FORMAT_F32: u32 = 9;

const SCE_GXM_DEPTH_FUNC_ALWAYS: u32 = 0x00000000 | 0x01000000; // Passes always (LESS_EQUAL with no test or ALWAYS)
const SCE_GXM_DEPTH_FUNC_NEVER: u32 = 0x00000000;
const SCE_GXM_DEPTH_FUNC_LESS: u32 = 0x00400000;
const SCE_GXM_DEPTH_FUNC_EQUAL: u32 = 0x00800000;
const SCE_GXM_DEPTH_FUNC_LESS_EQUAL: u32 = 0x00C00000;
const SCE_GXM_DEPTH_FUNC_GREATER: u32 = 0x01000000;
const SCE_GXM_DEPTH_FUNC_NOT_EQUAL: u32 = 0x01400000;
const SCE_GXM_DEPTH_FUNC_GREATER_EQUAL: u32 = 0x01800000;
const SCE_GXM_DEPTH_FUNC_ALWAYS_PASS: u32 = 0x01C00000;

const SCE_GXM_DEPTH_WRITE_ENABLED: u32 = 0x00000000;
const SCE_GXM_DEPTH_WRITE_DISABLED: u32 = 0x00100000;
const SCE_GXM_TWO_SIDED_DISABLED: u32 = 0x00000000;
const SCE_GXM_TWO_SIDED_ENABLED: u32 = 0x00000800;
const SCE_GXM_POLYGON_MODE_TRIANGLE_FILL: u32 = 0x00000000;
const SCE_GXM_VIEWPORT_ENABLED: u32 = 0x00000000;

const SCE_GXM_TEXTURE_BASE_FORMAT_U8U8U8U8: u32 = 0x0C000000;
const SCE_GXM_TEXTURE_SWIZZLE4_ABGR: u32 = 0x00000000;
const SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR: u32 = SCE_GXM_TEXTURE_BASE_FORMAT_U8U8U8U8 | SCE_GXM_TEXTURE_SWIZZLE4_ABGR;
const SCE_GXM_TEXTURE_FILTER_POINT: u32 = 0;
const SCE_GXM_TEXTURE_FILTER_LINEAR: u32 = 1;
const SCE_GXM_TEXTURE_MIP_FILTER_DISABLED: u32 = 0x00000000;
const SCE_GXM_TEXTURE_MIP_FILTER_ENABLED: u32 = 0x00000200;
const SCE_GXM_TEXTURE_ADDR_REPEAT: u32 = 0;

const SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW: u32 = 0x09408060;
const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE: u32 = 0x0C208060;
const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW: u32 = 0x0C20D060;

const VITA_WIDTH: u32 = 960;
const VITA_HEIGHT: u32 = 544;
const VITA_STRIDE: u32 = 960;
const DISPLAY_BUFFER_COUNT: usize = 3;
const MAX_PENDING_SWAPS: u32 = DISPLAY_BUFFER_COUNT as u32 - 1;

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
    _private: [u32; 4], // 16 bytes (0x10)
}

#[repr(C)]
pub struct SceGxmColorSurface {
    _private: [u32; 12], // 48 bytes (0x30)
}

#[repr(C)]
pub struct SceGxmDepthStencilSurface {
    _private: [u32; 5], // 20 bytes (0x14)
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
}

#[repr(C)]
struct SceDisplayFrameBuf {
    size: u32,
    base: *mut core::ffi::c_void,
    pitch: u32,
    pixelformat: u32,
    width: u32,
    height: u32,
}

// ---- FFI do libgxm i libkernel ----
#[link(name = "SceGxm_stub")]
extern "C" {
    fn sceGxmInitialize(params: *const SceGxmInitializeParams) -> i32;
    fn sceGxmTerminate() -> i32;
    fn sceGxmCreateContext(params: *const SceGxmContextParams, context: *mut *mut SceGxmContext) -> i32;
    fn sceGxmDestroyContext(context: *mut SceGxmContext) -> i32;
    fn sceGxmCreateRenderTarget(params: *const SceGxmRenderTargetParams, renderTarget: *mut *mut SceGxmRenderTarget) -> i32;
    fn sceGxmDestroyRenderTarget(renderTarget: *mut SceGxmRenderTarget) -> i32;
    fn sceGxmMapMemory(base: *mut core::ffi::c_void, size: u32, attr: u32) -> i32;
    fn sceGxmUnmapMemory(base: *mut core::ffi::c_void) -> i32;
    fn sceGxmMapVertexUsseMemory(base: *mut core::ffi::c_void, size: u32, offset: *mut u32) -> i32;
    fn sceGxmUnmapVertexUsseMemory(base: *mut core::ffi::c_void) -> i32;
    fn sceGxmMapFragmentUsseMemory(base: *mut core::ffi::c_void, size: u32, offset: *mut u32) -> i32;
    fn sceGxmUnmapFragmentUsseMemory(base: *mut core::ffi::c_void) -> i32;

    fn sceGxmColorSurfaceInit(
        surface: *mut SceGxmColorSurface,
        colorFormat: u32,
        surfaceType: u32,
        scaleMode: u32,
        outputRegisterSize: u32,
        width: u32,
        height: u32,
        strideInPixels: u32,
        data: *mut core::ffi::c_void,
    ) -> i32;

    fn sceGxmDepthStencilSurfaceInit(
        surface: *mut SceGxmDepthStencilSurface,
        depthStencilFormat: u32,
        surfaceType: u32,
        strideInSamples: u32,
        depthData: *mut core::ffi::c_void,
        stencilData: *mut core::ffi::c_void,
    ) -> i32;

    fn sceGxmBeginScene(
        context: *mut SceGxmContext,
        flags: u32,
        renderTarget: *const SceGxmRenderTarget,
        validRegion: *const core::ffi::c_void,
        vertexSyncObject: *mut SceGxmSyncObject,
        fragmentSyncObject: *mut SceGxmSyncObject,
        colorSurface: *const SceGxmColorSurface,
        depthStencil: *const SceGxmDepthStencilSurface,
    ) -> i32;

    fn sceGxmEndScene(
        context: *mut SceGxmContext,
        vertexNotification: *const core::ffi::c_void,
        fragmentNotification: *const core::ffi::c_void,
    ) -> i32;

    fn sceGxmPadHeartbeat(colorSurface: *const SceGxmColorSurface, syncObject: *mut SceGxmSyncObject) -> i32;
    fn sceGxmDisplayQueueAddEntry(
        oldBuffer: *mut SceGxmSyncObject,
        newBuffer: *mut SceGxmSyncObject,
        callbackData: *const core::ffi::c_void,
    ) -> i32;

    fn sceGxmSetVertexProgram(context: *mut SceGxmContext, vertexProgram: *const SceGxmVertexProgram);
    fn sceGxmSetFragmentProgram(context: *mut SceGxmContext, fragmentProgram: *const SceGxmFragmentProgram);
    fn sceGxmReserveVertexDefaultUniformBuffer(context: *mut SceGxmContext, uniformBuffer: *mut *mut core::ffi::c_void) -> i32;
    fn sceGxmReserveFragmentDefaultUniformBuffer(context: *mut SceGxmContext, uniformBuffer: *mut *mut core::ffi::c_void) -> i32;
    fn sceGxmSetUniformDataF(
        uniformBuffer: *mut core::ffi::c_void,
        parameter: *const SceGxmProgramParameter,
        componentOffset: u32,
        componentCount: u32,
        sourceData: *const f32,
    ) -> i32;
    fn sceGxmSetVertexStream(context: *mut SceGxmContext, streamIndex: u32, streamData: *const core::ffi::c_void);
    fn sceGxmDraw(
        context: *mut SceGxmContext,
        primType: u32,
        indexType: u32,
        indexData: *const core::ffi::c_void,
        indexCount: u32,
    ) -> i32;

    fn sceGxmSetFrontDepthFunc(context: *mut SceGxmContext, depthFunc: u32);
    fn sceGxmSetBackDepthFunc(context: *mut SceGxmContext, depthFunc: u32);
    fn sceGxmSetFrontDepthWriteEnable(context: *mut SceGxmContext, enable: u32);
    fn sceGxmSetBackDepthWriteEnable(context: *mut SceGxmContext, enable: u32);
    fn sceGxmSetFrontPolygonMode(context: *mut SceGxmContext, mode: u32);
    fn sceGxmSetBackPolygonMode(context: *mut SceGxmContext, mode: u32);
    fn sceGxmSetTwoSidedEnable(context: *mut SceGxmContext, enable: u32);
    fn sceGxmSetDefaultRegionClipAndViewport(context: *mut SceGxmContext, xMax: u32, yMax: u32);
    fn sceGxmSetViewportEnable(context: *mut SceGxmContext, enable: u32);
    fn sceGxmSetCullMode(context: *mut SceGxmContext, mode: u32);
    fn sceGxmFinish(context: *mut SceGxmContext);

    fn sceGxmColorSurfaceSetDitherMode(surface: *mut SceGxmColorSurface, ditherMode: u32) -> i32;

    fn sceGxmTextureInitLinear(
        texture: *mut SceGxmTexture,
        data: *const core::ffi::c_void,
        format: u32,
        width: u32,
        height: u32,
        mipCount: u32,
    ) -> i32;
    fn sceGxmTextureInitSwizzled(
        texture: *mut SceGxmTexture,
        data: *const core::ffi::c_void,
        format: u32,
        width: u32,
        height: u32,
        mipCount: u32,
    ) -> i32;
    fn sceGxmTextureSetMinFilter(texture: *mut SceGxmTexture, filter: u32) -> i32;
    fn sceGxmTextureSetMagFilter(texture: *mut SceGxmTexture, filter: u32) -> i32;
    fn sceGxmTextureSetMipFilter(texture: *mut SceGxmTexture, filter: u32) -> i32;
    fn sceGxmTextureSetUAddrMode(texture: *mut SceGxmTexture, mode: u32) -> i32;
    fn sceGxmTextureSetVAddrMode(texture: *mut SceGxmTexture, mode: u32) -> i32;
    fn sceGxmSetFragmentTexture(context: *mut SceGxmContext, textureIndex: u32, texture: *const SceGxmTexture) -> i32;

    fn sceGxmShaderPatcherCreate(params: *const SceGxmShaderPatcherParams, patcher: *mut *mut SceGxmShaderPatcher) -> i32;
    fn sceGxmShaderPatcherDestroy(patcher: *mut SceGxmShaderPatcher) -> i32;
    fn sceGxmShaderPatcherRegisterProgram(
        patcher: *mut SceGxmShaderPatcher,
        programHeader: *const SceGxmProgram,
        programId: *mut SceGxmShaderPatcherId,
    ) -> i32;
    fn sceGxmShaderPatcherReleaseVertexProgram(
        patcher: *mut SceGxmShaderPatcher,
        vertexProgram: *mut SceGxmVertexProgram,
    ) -> i32;
    fn sceGxmShaderPatcherReleaseFragmentProgram(
        patcher: *mut SceGxmShaderPatcher,
        fragmentProgram: *mut SceGxmFragmentProgram,
    ) -> i32;
    fn sceGxmShaderPatcherCreateVertexProgram(
        patcher: *mut SceGxmShaderPatcher,
        programId: SceGxmShaderPatcherId,
        attributes: *const SceGxmVertexAttribute,
        attributeCount: u32,
        streams: *const SceGxmVertexStream,
        streamCount: u32,
        vertexProgram: *mut *mut SceGxmVertexProgram,
    ) -> i32;
    fn sceGxmShaderPatcherCreateFragmentProgram(
        patcher: *mut SceGxmShaderPatcher,
        programId: SceGxmShaderPatcherId,
        outputFormat: u32,
        multisampleMode: u32,
        blendInfo: *const core::ffi::c_void,
        vertexProgramHeader: *const SceGxmProgram,
        fragmentProgram: *mut *mut SceGxmFragmentProgram,
    ) -> i32;
    fn sceGxmShaderPatcherGetProgramFromId(programId: SceGxmShaderPatcherId) -> *const SceGxmProgram;

    fn sceGxmProgramCheck(program: *const SceGxmProgram) -> i32;
    fn sceGxmProgramFindParameterByName(program: *const SceGxmProgram, name: *const u8) -> *const SceGxmProgramParameter;
    fn sceGxmProgramParameterGetResourceIndex(parameter: *const SceGxmProgramParameter) -> u32;

    fn sceGxmSyncObjectCreate(syncObject: *mut *mut SceGxmSyncObject) -> i32;
    fn sceGxmSyncObjectDestroy(syncObject: *mut SceGxmSyncObject) -> i32;
}

#[link(name = "SceSysmem_stub")]
extern "C" {
    fn sceKernelAllocMemBlock(name: *const u8, type_: u32, size: u32, opt: *const core::ffi::c_void) -> SceUID;
    fn sceKernelFreeMemBlock(uid: SceUID) -> i32;
    fn sceKernelGetMemBlockBase(uid: SceUID, base: *mut *mut core::ffi::c_void) -> i32;
}

#[link(name = "SceDisplay_stub")]
extern "C" {
    fn sceDisplaySetFrameBuf(pParam: *const SceDisplayFrameBuf, sync: i32) -> i32;
    fn sceDisplayWaitVblankStart() -> i32;
}

extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}

// ============================================================
// 2. BEZPIECZNA ALOKACJA PAMIĘCI GPU VITA
// ============================================================

#[derive(Debug, PartialEq)]
enum MemKind {
    GxmMap,
    VertexUsse,
    FragmentUsse,
}

pub struct GpuMem {
    pub uid: SceUID,
    pub ptr: *mut core::ffi::c_void,
    pub size: usize,
    kind: MemKind,
}

impl GpuMem {
    fn align_size(mem_type: u32, size: usize) -> usize {
        match mem_type {
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW => (size + 256 * 1024 - 1) & !(256 * 1024 - 1),
            _ => (size + 4 * 1024 - 1) & !(4 * 1024 - 1),
        }
    }

    pub fn alloc_gpu(name: &[u8], size: usize, mem_type: u32, attribs: u32) -> Option<Self> {
        let mut aligned = Self::align_size(mem_type, size);
        let mut uid = unsafe {
            sceKernelAllocMemBlock(name.as_ptr(), mem_type, aligned as u32, core::ptr::null())
        };

        if uid < 0 && mem_type == SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW {
            aligned = Self::align_size(SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, size);
            uid = unsafe {
                sceKernelAllocMemBlock(name.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, aligned as u32, core::ptr::null())
            };
        }

        if uid < 0 {
            return None;
        }

        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        if unsafe { sceGxmMapMemory(ptr, aligned as u32, attribs) } != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        Some(Self {
            uid,
            ptr,
            size: aligned,
            kind: MemKind::GxmMap,
        })
    }

    pub fn alloc_vertex_usse(name: &[u8], size: usize, out_offset: &mut u32) -> Option<Self> {
        let aligned = (size + 4095) & !4095;
        let uid = unsafe {
            sceKernelAllocMemBlock(name.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, aligned as u32, core::ptr::null())
        };
        if uid < 0 {
            return None;
        }

        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        if unsafe { sceGxmMapVertexUsseMemory(ptr, aligned as u32, out_offset) } != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        Some(Self {
            uid,
            ptr,
            size: aligned,
            kind: MemKind::VertexUsse,
        })
    }

    pub fn alloc_fragment_usse(name: &[u8], size: usize, out_offset: &mut u32) -> Option<Self> {
        let aligned = (size + 4095) & !4095;
        let uid = unsafe {
            sceKernelAllocMemBlock(name.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, aligned as u32, core::ptr::null())
        };
        if uid < 0 {
            return None;
        }

        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        if unsafe { sceGxmMapFragmentUsseMemory(ptr, aligned as u32, out_offset) } != SCE_OK {
            unsafe { sceKernelFreeMemBlock(uid); }
            return None;
        }

        Some(Self {
            uid,
            ptr,
            size: aligned,
            kind: MemKind::FragmentUsse,
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
struct DisplayData {
    address: *mut core::ffi::c_void,
}

unsafe extern "C" fn display_callback(data: *const core::ffi::c_void) {
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

// ============================================================
// 4. SHADERY GXP BYTECODE
// ============================================================

static VOXEL_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.vert.gxp"));
static VOXEL_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.frag.gxp"));

// Prekompilowane shadery sprzętowego czyszczenia (ze standardowego renderera SDL2 GXM)
static CLEAR_VERT_GXP: &[u8] = &[
    0x47, 0x58, 0x50, 0x00, 0x01, 0x05, 0x50, 0x03, 
    0xfa, 0x00, 0x00, 0x00, 0xdc, 0x25, 0x34, 0x74, 
    0x53, 0x4a, 0x7a, 0x5b, 0x04, 0x00, 0x19, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 
    0xb8, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 
    0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x74, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x78, 0x00, 0x00, 0x00, 0xc0, 0x3d, 0x03, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x6c, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x5c, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x54, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x4c, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x04, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x44, 0xfa, 
    0x01, 0x00, 0x04, 0x90, 0x85, 0x11, 0xa5, 0x08, 
    0x01, 0x80, 0x56, 0x90, 0x81, 0x11, 0x83, 0x08, 
    0x00, 0x00, 0x20, 0xa0, 0x00, 0x50, 0x27, 0xfb, 
    0x10, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x61, 0x50, 0x6f, 0x73, 0x69, 0x74, 0x69, 0x6f, 
    0x6e, 0x00, 0x00, 0x00, 
];

static CLEAR_FRAG_GXP: &[u8] = &[
    0x47, 0x58, 0x50, 0x00, 0x01, 0x05, 0x50, 0x03, 
    0xe8, 0x00, 0x00, 0x00, 0xa2, 0x55, 0x22, 0x3e, 
    0xc6, 0x7e, 0x77, 0xf1, 0x01, 0x00, 0x18, 0x00, 
    0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 
    0xa4, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 
    0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00, 
    0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x68, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 
    0x5c, 0x00, 0x00, 0x00, 0xc0, 0x3d, 0x03, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00, 
    0x01, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 
    0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
    0x00, 0x07, 0x44, 0xfa, 0x02, 0x80, 0x19, 0xf0, 
    0x7e, 0x0d, 0x80, 0x40, 0x0e, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x04, 0x00, 0x10, 0x00, 0x00, 0x00, 
    0x01, 0xe4, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 
    0x00, 0x00, 0x00, 0x00, 0x75, 0x43, 0x6c, 0x65, 
    0x61, 0x72, 0x43, 0x6f, 0x6c, 0x6f, 0x72, 0x00, 
];

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
    pub fn from_vertex(v: &Vertex, u0: f32, v0: f32, u1: f32, v1: f32) -> Self {
        let u = u0 + (u1 - u0) * v.uv.0;
        let vv = v0 + (v1 - v0) * v.uv.1;
        let light = (v.sky_light + v.block_light * 1.35).clamp(0.08, 1.0) * v.ao;
        Self {
            x: v.pos.x,
            y: v.pos.y,
            z: v.pos.z,
            u,
            v: vv,
            ao: light.clamp(0.10, 1.0),
        }
    }
}

// ============================================================
// 5. TEKSTURA ATLASU (GPU Linear Format)
// ============================================================

const ATLAS_TILES_PER_ROW: usize = 16;
const ATLAS_SIZE: usize = ATLAS_TILES_PER_ROW * TEX_SIZE; // 256x256 px

pub const ALL_TEXTURES: &[TextureId] = &[
    TextureId::GrassTop, TextureId::GrassSide, TextureId::Dirt, TextureId::Stone, TextureId::Sand,
    TextureId::WoodTop, TextureId::WoodSide, TextureId::Leaves, TextureId::Water, TextureId::Snow,
    TextureId::CactusTop, TextureId::CactusSide, TextureId::CoalOre, TextureId::IronOre, TextureId::Bedrock,
    TextureId::Lava, TextureId::SnowLeaves, TextureId::Planks, TextureId::Door, TextureId::DoorTop,
    TextureId::Glass, TextureId::Ice, TextureId::TntTop, TextureId::TntSide, TextureId::PigSkin,
    TextureId::PigFace, TextureId::PigSnout, TextureId::CowSkin, TextureId::CowFace, TextureId::CowSnout,
    TextureId::CowHorns, TextureId::CowUdder, TextureId::CraftingTableTop, TextureId::CraftingTableSide,
    TextureId::CraftingTableFront, TextureId::DestroyStage0, TextureId::DestroyStage1, TextureId::DestroyStage2,
    TextureId::DestroyStage3, TextureId::DestroyStage4, TextureId::DestroyStage5, TextureId::DestroyStage6,
    TextureId::DestroyStage7, TextureId::DestroyStage8, TextureId::DestroyStage9, TextureId::DamageFlash,
    TextureId::UnderwaterTint, TextureId::VignetteTint, TextureId::LavaTint, TextureId::CherryLeaves,
    TextureId::AutumnLeaves, TextureId::Mud, TextureId::Torch, TextureId::EntityShadow,
    TextureId::GrassTopSwamp, TextureId::GrassSideSwamp, TextureId::LeavesSwamp,
    TextureId::GrassTopSavanna, TextureId::GrassSideSavanna, TextureId::LeavesSavanna,
    TextureId::GrassTopCherry, TextureId::GrassSideCherry,
    TextureId::GrassTopAutumn, TextureId::GrassSideAutumn,
    TextureId::GrassTopTaiga, TextureId::GrassSideTaiga,
];

unsafe fn populate_atlas(pixels: *mut u32, atlas: &TextureAtlas) {
    for (tile_idx, &tex_id) in ALL_TEXTURES.iter().enumerate() {
        let tex = atlas.texture_for(tex_id);
        let tile_col = tile_idx % ATLAS_TILES_PER_ROW;
        let tile_row = tile_idx / ATLAS_TILES_PER_ROW;
        let base_x = tile_col * TEX_SIZE;
        let base_y = tile_row * TEX_SIZE;
        let is_water = tex_id == TextureId::Water;
        let is_glass = tex_id == TextureId::Glass;
        let is_damage = tex_id == TextureId::DamageFlash;
        let is_underwater = tex_id == TextureId::UnderwaterTint;
        let is_vignette = tex_id == TextureId::VignetteTint;
        let is_lava = tex_id == TextureId::LavaTint;
        let is_shadow = tex_id == TextureId::EntityShadow;
        for py in 0..TEX_SIZE {
            for px in 0..TEX_SIZE {
                let [r, g, b] = tex.texels[py * TEX_SIZE + px];
                let is_transparent = r < 0.0;
                let real_r = if is_transparent { (-r - 0.001).clamp(0.0, 1.0) } else { r.clamp(0.0, 1.0) };
                let a = if is_transparent {
                    0u32
                } else if is_shadow {
                    130u32 // Semi-transparent black entity shadow (~51% opacity)
                } else if is_water {
                    130u32 // Semi-transparent water (~51% opacity)
                } else if is_glass {
                    120u32 // Semi-transparent glass (~47% opacity)
                } else if is_damage {
                    85u32  // Semi-transparent damage flash (~33% opacity)
                } else if is_underwater {
                    95u32  // Semi-transparent underwater overlay (~37% opacity)
                } else if is_vignette {
                    70u32  // Semi-transparent heartbeat vignette
                } else if is_lava {
                    215u32 // Gęsta, gorąca mgła lawowa (~84% opacity)
                } else {
                    255u32
                };
                let ri = (real_r * 255.0) as u32;
                let gi = (g.clamp(0.0, 1.0) * 255.0) as u32;
                let bi = (b.clamp(0.0, 1.0) * 255.0) as u32;
                // ABGR8888
                let abgr = (a << 24) | (bi << 16) | (gi << 8) | ri;
                *pixels.add((base_y + py) * ATLAS_SIZE + (base_x + px)) = abgr;
            }
        }
    }
}

fn build_gpu_texture(atlas: &TextureAtlas) -> (GpuMem, SceGxmTexture) {
    let tex_bytes = ATLAS_SIZE * ATLAS_SIZE * 4;
    let mem = GpuMem::alloc_gpu(
        b"RC_GXM_Atlas\0",
        tex_bytes,
        SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
        SCE_GXM_MEMORY_ATTRIB_READ,
    ).expect("Nie udało się zaalokować tekstury atlasu w CDRAM");

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
pub fn atlas_uv(tex_id: TextureId) -> (f32, f32, f32, f32) {
    let tile_idx = match tex_id {
        TextureId::GrassTop => 0,
        TextureId::GrassSide => 1,
        TextureId::Dirt => 2,
        TextureId::Stone => 3,
        TextureId::Sand => 4,
        TextureId::WoodTop => 5,
        TextureId::WoodSide => 6,
        TextureId::Leaves => 7,
        TextureId::Water => 8,
        TextureId::Snow => 9,
        TextureId::CactusTop => 10,
        TextureId::CactusSide => 11,
        TextureId::CoalOre => 12,
        TextureId::IronOre => 13,
        TextureId::Bedrock => 14,
        TextureId::Lava => 15,
        TextureId::SnowLeaves => 16,
        TextureId::Planks => 17,
        TextureId::Door => 18,
        TextureId::DoorTop => 19,
        TextureId::Glass => 20,
        TextureId::Ice => 21,
        TextureId::TntTop => 22,
        TextureId::TntSide => 23,
        TextureId::PigSkin => 24,
        TextureId::PigFace => 25,
        TextureId::PigSnout => 26,
        TextureId::CowSkin => 27,
        TextureId::CowFace => 28,
        TextureId::CowSnout => 29,
        TextureId::CowHorns => 30,
        TextureId::CowUdder => 31,
        TextureId::CraftingTableTop => 32,
        TextureId::CraftingTableSide => 33,
        TextureId::CraftingTableFront => 34,
        TextureId::DestroyStage0 => 35,
        TextureId::DestroyStage1 => 36,
        TextureId::DestroyStage2 => 37,
        TextureId::DestroyStage3 => 38,
        TextureId::DestroyStage4 => 39,
        TextureId::DestroyStage5 => 40,
        TextureId::DestroyStage6 => 41,
        TextureId::DestroyStage7 => 42,
        TextureId::DestroyStage8 => 43,
        TextureId::DestroyStage9 => 44,
        TextureId::DamageFlash => 45,
        TextureId::UnderwaterTint => 46,
        TextureId::VignetteTint => 47,
        TextureId::LavaTint => 48,
        TextureId::CherryLeaves => 49,
        TextureId::AutumnLeaves => 50,
        TextureId::Mud => 51,
        TextureId::Torch => 52,
        TextureId::EntityShadow => 53,
        TextureId::GrassTopSwamp => 54,
        TextureId::GrassSideSwamp => 55,
        TextureId::LeavesSwamp => 56,
        TextureId::GrassTopSavanna => 57,
        TextureId::GrassSideSavanna => 58,
        TextureId::LeavesSavanna => 59,
        TextureId::GrassTopCherry => 60,
        TextureId::GrassSideCherry => 61,
        TextureId::GrassTopAutumn => 62,
        TextureId::GrassSideAutumn => 63,
        TextureId::GrassTopTaiga => 64,
        TextureId::GrassSideTaiga => 65,
    };
    const TILE_NORM: f32 = 1.0 / ATLAS_TILES_PER_ROW as f32;
    let col = tile_idx % ATLAS_TILES_PER_ROW;
    let row = tile_idx / ATLAS_TILES_PER_ROW;
    // Sub-texel inset (0.375 texela) zapobiega bleedingowi krawędzi kafelków atlasu
    let inset = 0.375 / ATLAS_SIZE as f32;
    let u0 = col as f32 * TILE_NORM + inset;
    let v0 = row as f32 * TILE_NORM + inset;
    let u1 = (col + 1) as f32 * TILE_NORM - inset;
    let v1 = (row + 1) as f32 * TILE_NORM - inset;
    (u0, v0, u1, v1)
}

// ============================================================
// 6. GŁÓWNY RENDERER GXM
// ============================================================

unsafe extern "C" fn patcher_host_alloc(_: *mut core::ffi::c_void, size: u32) -> *mut core::ffi::c_void {
    malloc(size as usize)
}

unsafe extern "C" fn patcher_host_free(_: *mut core::ffi::c_void, ptr: *mut core::ffi::c_void) {
    free(ptr);
}

pub struct GxmRenderer {
    context: *mut SceGxmContext,
    host_mem: *mut core::ffi::c_void,
    render_target: *mut SceGxmRenderTarget,

    _vdm_mem: GpuMem,
    _vertex_ring_mem: GpuMem,
    _fragment_ring_mem: GpuMem,
    _frag_usse_mem: GpuMem,

    display_bufs: [GpuMem; DISPLAY_BUFFER_COUNT],
    display_sync: [*mut SceGxmSyncObject; DISPLAY_BUFFER_COUNT],
    color_surfaces: [SceGxmColorSurface; DISPLAY_BUFFER_COUNT],
    display_data: [DisplayData; DISPLAY_BUFFER_COUNT],
    front_buf: usize,
    back_buf: usize,

    _depth_mem: GpuMem,
    depth_surface: SceGxmDepthStencilSurface,

    patcher: *mut SceGxmShaderPatcher,
    _patcher_buf_mem: GpuMem,
    _patcher_vert_usse: GpuMem,
    _patcher_frag_usse: GpuMem,

    vert_prog: *mut SceGxmVertexProgram,
    frag_prog: *mut SceGxmFragmentProgram,
    param_mvp: *const SceGxmProgramParameter,
    param_fog: *const SceGxmProgramParameter,

    clear_vert_prog: *mut SceGxmVertexProgram,
    clear_frag_prog: *mut SceGxmFragmentProgram,
    param_clear_color: *const SceGxmProgramParameter,
    clear_vb_mem: GpuMem,
    clear_ib_mem: GpuMem,

    _atlas_mem: GpuMem,
    atlas_tex: SceGxmTexture,

    vb_mem: GpuMem,
    ib_mem: GpuMem,

    crack_vb_mem: GpuMem,
    crack_ib_mem: GpuMem,

    entity_vb_mem: GpuMem,
    entity_ib_mem: GpuMem,

    chunk_slots: HashMap<(i32, i32), usize>,
    pub chunk_spawn_times: HashMap<(i32, i32), f32>,
    free_slots: Vec<usize>,
    slot_index_counts: [u32; GxmRenderer::MAX_CHUNK_SLOTS],
}

unsafe impl Send for GxmRenderer {}
unsafe impl Sync for GxmRenderer {}

impl GxmRenderer {
    pub const MAX_CHUNK_SLOTS: usize = 144;
    pub const MAX_VERTS_PER_SLOT: usize = 24576;
    pub const MAX_INDICES_PER_SLOT: usize = 36864;

    pub const MAX_ENTITY_VERTS: usize = 16384;
    pub const MAX_ENTITY_INDICES: usize = 24576;

    const VB_SIZE: usize = Self::MAX_CHUNK_SLOTS * Self::MAX_VERTS_PER_SLOT * core::mem::size_of::<GxmVertex>();
    const IB_SIZE: usize = Self::MAX_CHUNK_SLOTS * Self::MAX_INDICES_PER_SLOT * core::mem::size_of::<u16>();

    pub fn new(atlas: &TextureAtlas) -> Self {
        unsafe { Self::init_unsafe(atlas) }
    }

    unsafe fn init_unsafe(atlas: &TextureAtlas) -> Self {
        // --- 1. Inicjalizacja GXM Library ---
        let init_params = SceGxmInitializeParams {
            flags: 0,
            displayQueueMaxPendingCount: MAX_PENDING_SWAPS,
            displayQueueCallback: Some(display_callback),
            displayQueueCallbackDataSize: core::mem::size_of::<DisplayData>() as u32,
            parameterBufferSize: SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE as u32,
        };
        let rc = sceGxmInitialize(&init_params);
        assert_eq!(rc, SCE_OK, "sceGxmInitialize failed: 0x{:08X}", rc);

        // --- 2. Alokacja Ring Bufferów GPU ---
        let vdm_mem = GpuMem::alloc_gpu(
            b"RC_GXM_VDM\0",
            SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).expect("VDM alloc failed");

        let vertex_ring_mem = GpuMem::alloc_gpu(
            b"RC_GXM_VRing\0",
            SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).expect("VRing alloc failed");

        let fragment_ring_mem = GpuMem::alloc_gpu(
            b"RC_GXM_FRing\0",
            SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).expect("FRing alloc failed");

        let mut frag_usse_offset: u32 = 0;
        let frag_usse_mem = GpuMem::alloc_fragment_usse(
            b"RC_GXM_FUSSE\0",
            SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE,
            &mut frag_usse_offset,
        ).expect("FUSSE alloc failed");

        // --- 3. Tworzenie kontekstu GXM z hostMem (malloc) ---
        let host_mem = malloc(SCE_GXM_MINIMUM_CONTEXT_HOST_MEM_SIZE);
        assert!(!host_mem.is_null(), "malloc hostMem failed");

        let ctx_params = SceGxmContextParams {
            hostMem: host_mem,
            hostMemSize: SCE_GXM_MINIMUM_CONTEXT_HOST_MEM_SIZE as u32,
            vdmRingBufferMem: vdm_mem.ptr,
            vdmRingBufferMemSize: SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE as u32,
            vertexRingBufferMem: vertex_ring_mem.ptr,
            vertexRingBufferMemSize: SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE as u32,
            fragmentRingBufferMem: fragment_ring_mem.ptr,
            fragmentRingBufferMemSize: SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE as u32,
            fragmentUsseRingBufferMem: frag_usse_mem.ptr,
            fragmentUsseRingBufferMemSize: SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE as u32,
            fragmentUsseRingBufferOffset: frag_usse_offset,
        };

        let mut context: *mut SceGxmContext = core::ptr::null_mut();
        let rc = sceGxmCreateContext(&ctx_params, &mut context);
        assert_eq!(rc, SCE_OK, "sceGxmCreateContext failed: 0x{:08X}", rc);

        // --- 4. Render Target ---
        let rt_params = SceGxmRenderTargetParams {
            flags: 0,
            width: VITA_WIDTH as u16,
            height: VITA_HEIGHT as u16,
            scenesPerFrame: 1,
            multisampleMode: SCE_GXM_MULTISAMPLE_NONE as u16,
            multisampleLocations: 0,
            driverMemBlock: -1,
        };
        let mut render_target: *mut SceGxmRenderTarget = core::ptr::null_mut();
        let rc = sceGxmCreateRenderTarget(&rt_params, &mut render_target);
        assert_eq!(rc, SCE_OK, "sceGxmCreateRenderTarget failed: 0x{:08X}", rc);

        // --- 5. Triple Display Buffers w pamięci CDRAM (ze sprzętowym ditheringiem) ---
        let color_surface_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
        let names: [&[u8]; 3] = [b"RC_GXM_CB0\0", b"RC_GXM_CB1\0", b"RC_GXM_CB2\0"];

        let display_bufs: [GpuMem; DISPLAY_BUFFER_COUNT] = [
            GpuMem::alloc_gpu(names[0], color_surface_size, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_GXM_MEMORY_ATTRIB_RW).unwrap(),
            GpuMem::alloc_gpu(names[1], color_surface_size, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_GXM_MEMORY_ATTRIB_RW).unwrap(),
            GpuMem::alloc_gpu(names[2], color_surface_size, SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, SCE_GXM_MEMORY_ATTRIB_RW).unwrap(),
        ];

        let mut color_surfaces: [SceGxmColorSurface; DISPLAY_BUFFER_COUNT] = [
            SceGxmColorSurface { _private: [0u32; 12] },
            SceGxmColorSurface { _private: [0u32; 12] },
            SceGxmColorSurface { _private: [0u32; 12] },
        ];
        let mut display_sync: [*mut SceGxmSyncObject; DISPLAY_BUFFER_COUNT] = [core::ptr::null_mut(); DISPLAY_BUFFER_COUNT];
        let mut display_data: [DisplayData; DISPLAY_BUFFER_COUNT] = [
            DisplayData { address: core::ptr::null_mut() },
            DisplayData { address: core::ptr::null_mut() },
            DisplayData { address: core::ptr::null_mut() },
        ];

        for i in 0..DISPLAY_BUFFER_COUNT {
            let ptr = display_bufs[i].ptr;
            core::ptr::write_bytes(ptr as *mut u8, 0, color_surface_size);

            sceGxmColorSurfaceInit(
                &mut color_surfaces[i],
                SCE_GXM_COLOR_FORMAT_A8B8G8R8,
                SCE_GXM_COLOR_SURFACE_LINEAR,
                SCE_GXM_COLOR_SURFACE_SCALE_NONE,
                SCE_GXM_OUTPUT_REGISTER_SIZE_32BIT,
                VITA_WIDTH,
                VITA_HEIGHT,
                VITA_STRIDE,
                ptr,
            );
            // Sprzętowy Dithering eliminuje banding kolorów przy winietach, mgle i gradientach
            sceGxmColorSurfaceSetDitherMode(&mut color_surfaces[i], SCE_GXM_COLOR_SURFACE_DITHER_ENABLED);
            display_data[i].address = ptr;

            let mut sync: *mut SceGxmSyncObject = core::ptr::null_mut();
            sceGxmSyncObjectCreate(&mut sync);
            display_sync[i] = sync;
        }

        // --- 6. Depth Buffer ---
        let depth_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
        let depth_mem = GpuMem::alloc_gpu(
            b"RC_GXM_Depth\0",
            depth_size,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_RW,
        ).unwrap();

        let mut depth_surface = SceGxmDepthStencilSurface { _private: [0u32; 5] };
        sceGxmDepthStencilSurfaceInit(
            &mut depth_surface,
            SCE_GXM_DEPTH_STENCIL_FORMAT_DF32,
            SCE_GXM_DEPTH_STENCIL_SURFACE_LINEAR,
            VITA_WIDTH,
            depth_mem.ptr,
            core::ptr::null_mut(),
        );

        // --- 7. Shader Patcher ---
        let patcher_buf_mem = GpuMem::alloc_gpu(
            b"RC_GXM_PatchBuf\0",
            SCE_GXM_DEFAULT_PATCHER_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_RW,
        ).unwrap();

        let mut vert_usse_offset: u32 = 0;
        let patcher_vert_usse = GpuMem::alloc_vertex_usse(
            b"RC_GXM_PatchVUSS\0",
            SCE_GXM_DEFAULT_PATCHER_VERTEX_USSE_SIZE,
            &mut vert_usse_offset,
        ).unwrap();

        let mut frag_usse_offset: u32 = 0;
        let patcher_frag_usse = GpuMem::alloc_fragment_usse(
            b"RC_GXM_PatchFUSS\0",
            SCE_GXM_DEFAULT_PATCHER_FRAGMENT_USSE_SIZE,
            &mut frag_usse_offset,
        ).unwrap();

        let patcher_params = SceGxmShaderPatcherParams {
            userData: core::ptr::null_mut(),
            hostAllocCallback: Some(patcher_host_alloc),
            hostFreeCallback: Some(patcher_host_free),
            bufferAllocCallback: core::ptr::null(),
            bufferFreeCallback: core::ptr::null(),
            bufferMem: patcher_buf_mem.ptr,
            bufferMemSize: SCE_GXM_DEFAULT_PATCHER_BUFFER_SIZE as u32,
            vertexUsseAllocCallback: core::ptr::null(),
            vertexUsseFreeCallback: core::ptr::null(),
            vertexUsseMem: patcher_vert_usse.ptr,
            vertexUsseMemSize: SCE_GXM_DEFAULT_PATCHER_VERTEX_USSE_SIZE as u32,
            vertexUsseOffset: vert_usse_offset,
            fragmentUsseAllocCallback: core::ptr::null(),
            fragmentUsseFreeCallback: core::ptr::null(),
            fragmentUsseMem: patcher_frag_usse.ptr,
            fragmentUsseMemSize: SCE_GXM_DEFAULT_PATCHER_FRAGMENT_USSE_SIZE as u32,
            fragmentUsseOffset: frag_usse_offset,
        };

        let mut patcher: *mut SceGxmShaderPatcher = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreate(&patcher_params, &mut patcher);
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreate failed: 0x{:08X}", rc);

        // --- 8. Rejestracja Shaderów GXP i Linkowanie ---
        let vert_gxp = VOXEL_VERT_GXP.as_ptr() as *const SceGxmProgram;
        let frag_gxp = VOXEL_FRAG_GXP.as_ptr() as *const SceGxmProgram;

        let chk_v = sceGxmProgramCheck(vert_gxp);
        assert_eq!(chk_v, SCE_OK, "sceGxmProgramCheck(vert) failed: 0x{:08X}", chk_v);
        let chk_f = sceGxmProgramCheck(frag_gxp);
        assert_eq!(chk_f, SCE_OK, "sceGxmProgramCheck(frag) failed: 0x{:08X}", chk_f);

        let mut vert_prog_id: SceGxmShaderPatcherId = core::ptr::null_mut();
        let mut frag_prog_id: SceGxmShaderPatcherId = core::ptr::null_mut();
        sceGxmShaderPatcherRegisterProgram(patcher, vert_gxp, &mut vert_prog_id);
        sceGxmShaderPatcherRegisterProgram(patcher, frag_gxp, &mut frag_prog_id);

        let param_pos = sceGxmProgramFindParameterByName(vert_gxp, b"aPosition\0".as_ptr());
        let param_uv = sceGxmProgramFindParameterByName(vert_gxp, b"aUV\0".as_ptr());
        let param_ao = sceGxmProgramFindParameterByName(vert_gxp, b"aAO\0".as_ptr());

        let reg_pos = if !param_pos.is_null() { sceGxmProgramParameterGetResourceIndex(param_pos) as u16 } else { 0 };
        let reg_uv = if !param_uv.is_null() { sceGxmProgramParameterGetResourceIndex(param_uv) as u16 } else { 1 };
        let reg_ao = if !param_ao.is_null() { sceGxmProgramParameterGetResourceIndex(param_ao) as u16 } else { 2 };

        let attrs = [
            SceGxmVertexAttribute {
                streamIndex: 0,
                offset: 0,
                format: SCE_GXM_ATTRIBUTE_FORMAT_F32 as u8,
                componentCount: 3,
                regIndex: reg_pos,
            },
            SceGxmVertexAttribute {
                streamIndex: 0,
                offset: 12,
                format: SCE_GXM_ATTRIBUTE_FORMAT_F32 as u8,
                componentCount: 2,
                regIndex: reg_uv,
            },
            SceGxmVertexAttribute {
                streamIndex: 0,
                offset: 20,
                format: SCE_GXM_ATTRIBUTE_FORMAT_F32 as u8,
                componentCount: 1,
                regIndex: reg_ao,
            },
        ];

        let streams = [SceGxmVertexStream {
            stride: core::mem::size_of::<GxmVertex>() as u16,
            indexSource: SCE_GXM_INDEX_SOURCE_INDEX_16BIT,
        }];

        let mut vert_prog: *mut SceGxmVertexProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateVertexProgram(
            patcher,
            vert_prog_id,
            attrs.as_ptr(),
            3,
            streams.as_ptr(),
            1,
            &mut vert_prog,
        );
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreateVertexProgram failed: 0x{:08X}", rc);

        let blend_info = SceGxmBlendInfo::alpha_blend();
        let mut frag_prog: *mut SceGxmFragmentProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateFragmentProgram(
            patcher,
            frag_prog_id,
            SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4,
            SCE_GXM_MULTISAMPLE_NONE,
            &blend_info as *const _ as *const core::ffi::c_void,
            vert_gxp,
            &mut frag_prog,
        );
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreateFragmentProgram failed: 0x{:08X}", rc);

        let param_mvp = sceGxmProgramFindParameterByName(vert_gxp, b"uViewProj\0".as_ptr());
        let param_fog = sceGxmProgramFindParameterByName(vert_gxp, b"uCameraPosAndFog\0".as_ptr());

        // --- 8b. Rejestracja i Linkowanie Shaderów Hardware Clear ---
        let clear_vert_gxp = CLEAR_VERT_GXP.as_ptr() as *const SceGxmProgram;
        let clear_frag_gxp = CLEAR_FRAG_GXP.as_ptr() as *const SceGxmProgram;

        let mut clear_vert_id: SceGxmShaderPatcherId = core::ptr::null_mut();
        let mut clear_frag_id: SceGxmShaderPatcherId = core::ptr::null_mut();
        sceGxmShaderPatcherRegisterProgram(patcher, clear_vert_gxp, &mut clear_vert_id);
        sceGxmShaderPatcherRegisterProgram(patcher, clear_frag_gxp, &mut clear_frag_id);

        let param_clear_pos = sceGxmProgramFindParameterByName(clear_vert_gxp, b"aPosition\0".as_ptr());
        let reg_clear_pos = if !param_clear_pos.is_null() { sceGxmProgramParameterGetResourceIndex(param_clear_pos) as u16 } else { 0 };

        let clear_attr = [SceGxmVertexAttribute {
            streamIndex: 0,
            offset: 0,
            format: SCE_GXM_ATTRIBUTE_FORMAT_F32 as u8,
            componentCount: 2,
            regIndex: reg_clear_pos,
        }];

        let clear_stream = [SceGxmVertexStream {
            stride: (2 * core::mem::size_of::<f32>()) as u16,
            indexSource: SCE_GXM_INDEX_SOURCE_INDEX_16BIT,
        }];

        let mut clear_vert_prog: *mut SceGxmVertexProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateVertexProgram(
            patcher,
            clear_vert_id,
            clear_attr.as_ptr(),
            1,
            clear_stream.as_ptr(),
            1,
            &mut clear_vert_prog,
        );
        assert_eq!(rc, SCE_OK, "clear create vert failed: 0x{:08X}", rc);

        let mut clear_frag_prog: *mut SceGxmFragmentProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateFragmentProgram(
            patcher,
            clear_frag_id,
            SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4,
            SCE_GXM_MULTISAMPLE_NONE,
            core::ptr::null(),
            clear_vert_gxp,
            &mut clear_frag_prog,
        );
        assert_eq!(rc, SCE_OK, "clear create frag failed: 0x{:08X}", rc);

        let param_clear_color = sceGxmProgramFindParameterByName(clear_frag_gxp, b"uClearColor\0".as_ptr());

        // Bufor wierzchołków dla trójkąta czyszczącego (NDC [-1, 3])
        let clear_vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_ClearVB\0",
            3 * 2 * core::mem::size_of::<f32>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let clear_verts = clear_vb_mem.ptr as *mut f32;
        *clear_verts.add(0) = -1.0f32; *clear_verts.add(1) = -1.0f32;
        *clear_verts.add(2) =  3.0f32; *clear_verts.add(3) = -1.0f32;
        *clear_verts.add(4) = -1.0f32; *clear_verts.add(5) =  3.0f32;

        let clear_ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_ClearIB\0",
            3 * core::mem::size_of::<u16>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let clear_indices = clear_ib_mem.ptr as *mut u16;
        *clear_indices.add(0) = 0;
        *clear_indices.add(1) = 1;
        *clear_indices.add(2) = 2;

        // --- 9. Atlas Tekstury GPU ---
        let (atlas_mem, atlas_tex) = build_gpu_texture(atlas);

        // --- 10. Bufor GPU dla slotów chunków (Uncached RAM dla natychmiastowego zapisu CPU) ---
        let vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_VB\0",
            Self::VB_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_IB\0",
            Self::IB_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        // --- 10b. Dynamiczne bufory dla pęknięć niszczonego bloku ---
        let crack_vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_CrackVB\0",
            24 * core::mem::size_of::<GxmVertex>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let crack_ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_CrackIB\0",
            36 * core::mem::size_of::<u16>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let crack_ib = crack_ib_mem.ptr as *mut u16;
        for i in 0..6 {
            let base = (i * 4) as u16;
            let idx = i * 6;
            *crack_ib.add(idx) = base;
            *crack_ib.add(idx + 1) = base + 1;
            *crack_ib.add(idx + 2) = base + 2;
            *crack_ib.add(idx + 3) = base;
            *crack_ib.add(idx + 4) = base + 2;
            *crack_ib.add(idx + 5) = base + 3;
        }

        // --- 10c. Dynamiczne bufory dla bytów (wypadnięte itemy, moby, TNT) ---
        let entity_vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_EntVB\0",
            Self::MAX_ENTITY_VERTS * core::mem::size_of::<GxmVertex>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let entity_ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_EntIB\0",
            Self::MAX_ENTITY_INDICES * core::mem::size_of::<u16>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        ).unwrap();

        let free_slots = (0..Self::MAX_CHUNK_SLOTS).rev().collect();

        Self {
            context,
            host_mem,
            render_target,
            _vdm_mem: vdm_mem,
            _vertex_ring_mem: vertex_ring_mem,
            _fragment_ring_mem: fragment_ring_mem,
            _frag_usse_mem: frag_usse_mem,
            display_bufs,
            display_sync,
            color_surfaces,
            display_data,
            front_buf: DISPLAY_BUFFER_COUNT - 1,
            back_buf: 0,
            _depth_mem: depth_mem,
            depth_surface,
            patcher,
            _patcher_buf_mem: patcher_buf_mem,
            _patcher_vert_usse: patcher_vert_usse,
            _patcher_frag_usse: patcher_frag_usse,
            vert_prog,
            frag_prog,
            param_mvp,
            param_fog,
            clear_vert_prog,
            clear_frag_prog,
            param_clear_color,
            clear_vb_mem,
            clear_ib_mem,
            _atlas_mem: atlas_mem,
            atlas_tex,
            vb_mem,
            ib_mem,
            crack_vb_mem,
            crack_ib_mem,
            entity_vb_mem,
            entity_ib_mem,
            chunk_slots: HashMap::new(),
            chunk_spawn_times: HashMap::new(),
            free_slots,
            slot_index_counts: [0; Self::MAX_CHUNK_SLOTS],
        }
    }

    /// Oblicza macierz projekcji perspektywicznej dopasowaną do zakresu Z [0, 1] w GXM
    pub fn gxm_perspective_matrix(fov_y_rad: f32, aspect: f32, near: f32, far: f32) -> crate::core::math::Mat4 {
        let f = 1.0 / (fov_y_rad / 2.0).tan();
        let mut m = [0.0f32; 16];
        m[0] = f / aspect;
        m[5] = f;
        m[10] = far / (near - far);
        m[11] = -1.0;
        m[14] = (far * near) / (near - far);
        crate::core::math::Mat4 { m }
    }

    /// Rysuje 3D scenę wokół gracza na GPU Vita oraz nakłada 2D HUD (60 FPS)
    pub fn render_frame(
        &mut self,
        camera: &Camera,
        world: &mut World,
        hud_pixels: Option<&[u32]>,
        break_target: Option<crate::core::math::IVec3>,
        break_progress: f32,
        items: &[crate::gameplay::item::ItemEntity],
        mobs: &[crate::gameplay::mob::Mob],
        hand_block: Option<u8>,
        walk_phase: f32,
        speed: f32,
        attack_animation: f32,
        render_distance: i32,
        fog_enabled: bool,
        clouds_enabled: bool,
        chunk_anim: bool,
        total_time: f32,
        particles: &[crate::gameplay::particles::Particle],
        damage_flash: f32,
        is_underwater: bool,
        is_in_lava: bool,
        low_health: bool,
    ) -> u32 {
        let buf_idx = self.back_buf;
        let color_buf_ptr = self.display_bufs[buf_idx].ptr as *mut u32;

        unsafe {
            sceGxmBeginScene(
                self.context,
                0,
                self.render_target,
                core::ptr::null(),
                core::ptr::null_mut(),
                self.display_sync[buf_idx],
                &self.color_surfaces[buf_idx],
                &self.depth_surface,
            );

            // Pełnoekranowy viewport i obcinanie krawędzi (960x544)
            sceGxmSetDefaultRegionClipAndViewport(self.context, VITA_WIDTH - 1, VITA_HEIGHT - 1);
            sceGxmSetViewportEnable(self.context, SCE_GXM_VIEWPORT_ENABLED);
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_ENABLED);
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);

            // 1. Sprzętowe czyszczenie koloru nieba i głębokości (GPU Hardware Clear Triangle)
            sceGxmSetFrontDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
            sceGxmSetBackDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_ALWAYS_PASS);
            sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
            sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
            sceGxmSetFrontPolygonMode(self.context, SCE_GXM_POLYGON_MODE_TRIANGLE_FILL);
            sceGxmSetBackPolygonMode(self.context, SCE_GXM_POLYGON_MODE_TRIANGLE_FILL);

            sceGxmSetVertexProgram(self.context, self.clear_vert_prog);
            sceGxmSetFragmentProgram(self.context, self.clear_frag_prog);

            let mut clear_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
            sceGxmReserveFragmentDefaultUniformBuffer(self.context, &mut clear_uniform_buf);
            if !self.param_clear_color.is_null() && !clear_uniform_buf.is_null() {
                let (sky_top, _sky_bottom, _fog_col) = world.sky_colors();
                let sky_rgba: [f32; 4] = [sky_top[0], sky_top[1], sky_top[2], 1.0f32];
                sceGxmSetUniformDataF(clear_uniform_buf, self.param_clear_color, 0, 4, sky_rgba.as_ptr());
            }

            sceGxmSetVertexStream(self.context, 0, self.clear_vb_mem.ptr);
            sceGxmDraw(
                self.context,
                SCE_GXM_PRIMITIVE_TRIANGLES,
                SCE_GXM_INDEX_FORMAT_U16,
                self.clear_ib_mem.ptr,
                3,
            );

            // 2. Rysowanie terenu 3D wokół gracza z włączonym sprzętowym Backface Cullingiem
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_DISABLED);
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_CW);
            sceGxmSetFrontDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_LESS_EQUAL);
            sceGxmSetBackDepthFunc(self.context, SCE_GXM_DEPTH_FUNC_LESS_EQUAL);
            sceGxmSetFrontDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);
            sceGxmSetBackDepthWriteEnable(self.context, SCE_GXM_DEPTH_WRITE_ENABLED);

            sceGxmSetVertexProgram(self.context, self.vert_prog);
            sceGxmSetFragmentProgram(self.context, self.frag_prog);

            let aspect = VITA_WIDTH as f32 / VITA_HEIGHT as f32;
            let proj = Self::gxm_perspective_matrix(camera.fov_y, aspect, 0.05, 400.0);
            let view = camera.view_matrix();
            let mvp = proj.mul(&view);

            let mut uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
            sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut uniform_buf);
            if !self.param_mvp.is_null() && !uniform_buf.is_null() {
                sceGxmSetUniformDataF(
                    uniform_buf,
                    self.param_mvp,
                    0,
                    16,
                    mvp.m.as_ptr(),
                );
            }
            if !self.param_fog.is_null() && !uniform_buf.is_null() {
                let max_fog = if fog_enabled { (render_distance as f32 * 16.0).max(32.0) } else { -1.0 };
                let fog_data: [f32; 4] = [camera.position.x, camera.position.y, camera.position.z, max_fog];
                sceGxmSetUniformDataF(
                    uniform_buf,
                    self.param_fog,
                    0,
                    4,
                    fog_data.as_ptr(),
                );
            }

            sceGxmSetFragmentTexture(self.context, 0, &self.atlas_tex);

            let vb_base = self.vb_mem.as_mut_ptr::<GxmVertex>();
            let ib_base = self.ib_mem.as_mut_ptr::<u16>();

            let cam_chunk_x = (camera.position.x / CHUNK_SIZE_X as f32).floor() as i32;
            let cam_chunk_z = (camera.position.z / CHUNK_SIZE_Z as f32).floor() as i32;
            let render_dist = render_distance.clamp(2, 5);

            // 1. Usunięcie slotów dla chunków poza zasięgiem widzenia
            let max_dist = render_dist + 2;
            let mut freed = Vec::new();
            self.chunk_slots.retain(|&(cx, cz), &mut slot_idx| {
                let dx = cx - cam_chunk_x;
                let dz = cz - cam_chunk_z;
                if dx.abs() > max_dist || dz.abs() > max_dist {
                    freed.push(slot_idx);
                    false
                } else {
                    true
                }
            });
            self.free_slots.extend(freed);
            self.chunk_spawn_times.retain(|k, _| self.chunk_slots.contains_key(k));

            // 2. Rysowanie chunków z GPU z ograniczeniem liczby wysyłanych meshy na klatkę (likwiduje stutter)
            let mut tri_count: u32 = 0;
            let mut uploads_this_frame = 0;
            const MAX_UPLOADS_PER_FRAME: usize = 2;
            let mut last_had_offset = false;

            // 2. Sortowanie chunków od najbliższego do najdalszego (Front-to-Back)
            // Pozwala to sprzętowemu HSR / Early Depth Rejection GPU SGX543 odrzucać zasłonięte piksele przed cieniowaniem
            let mut chunk_coords = Vec::with_capacity(((render_dist * 2 + 1) * (render_dist * 2 + 1)) as usize);
            for dz in -render_dist..=render_dist {
                for dx in -render_dist..=render_dist {
                    chunk_coords.push((dx, dz));
                }
            }
            chunk_coords.sort_unstable_by_key(|&(dx, dz)| dx * dx + dz * dz);

            let is_occluded = world.is_view_fully_occluded(camera.position, camera.yaw, camera.pitch);

            for (dx, dz) in chunk_coords {
                let cx = cam_chunk_x + dx;
                let cz = cam_chunk_z + dz;
                let key = (cx, cz);

                // Jeśli gracz stoi twarzą bezpośrednio w litej ścianie (100% okluzji pola widzenia),
                // rysujemy tylko bieżący lokalny chunk (zawierający ścianę), a wszystkie pozostałe pomijamy!
                if is_occluded && (dx != 0 || dz != 0) {
                    continue;
                }

                    let slot_idx = if let Some(&s) = self.chunk_slots.get(&key) {
                        let is_dirty = world.chunks.get(&key).map(|c| c.dirty).unwrap_or(false);
                        if is_dirty && uploads_this_frame < MAX_UPLOADS_PER_FRAME {
                            if let Some(chunk) = world.chunks.get(&key) {
                                let idx_cnt = Self::upload_chunk(chunk, world, vb_base, ib_base, s);
                                self.slot_index_counts[s] = idx_cnt;
                                uploads_this_frame += 1;
                            }
                            if let Some(chunk) = world.chunks.get_mut(&key) {
                                chunk.dirty = false;
                            }
                        }
                        Some(s)
                    } else if world.chunks.contains_key(&key) {
                        if uploads_this_frame < MAX_UPLOADS_PER_FRAME {
                            if let Some(s) = self.free_slots.pop() {
                                if let Some(chunk) = world.chunks.get(&key) {
                                    let idx_cnt = Self::upload_chunk(chunk, world, vb_base, ib_base, s);
                                    self.slot_index_counts[s] = idx_cnt;
                                    uploads_this_frame += 1;
                                }
                                if let Some(chunk) = world.chunks.get_mut(&key) {
                                    chunk.dirty = false;
                                }
                                self.chunk_slots.insert(key, s);
                                self.chunk_spawn_times.insert(key, total_time);
                                Some(s)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(s) = slot_idx {
                        let idx_count = self.slot_index_counts[s];
                        if idx_count > 0 && Self::is_chunk_in_frustum(cx, cz, &mvp) {
                            let spawn_time = self.chunk_spawn_times.get(&key).copied().unwrap_or(0.0);
                            let elapsed = total_time - spawn_time;
                            let y_offset = if chunk_anim && elapsed >= 0.0 && elapsed < 0.35 {
                                let t = (elapsed / 0.35).clamp(0.0, 1.0);
                                let ease = 1.0 - (1.0 - t).powi(3);
                                (1.0 - ease) * -14.0
                            } else {
                                0.0
                            };

                            if y_offset != 0.0 {
                                let chunk_mvp = mvp.mul(&crate::core::math::Mat4::translation(0.0, y_offset, 0.0));
                                let mut chunk_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
                                sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut chunk_uniform_buf);
                                if !self.param_mvp.is_null() && !chunk_uniform_buf.is_null() {
                                    sceGxmSetUniformDataF(
                                        chunk_uniform_buf,
                                        self.param_mvp,
                                        0,
                                        16,
                                        chunk_mvp.m.as_ptr(),
                                    );
                                }
                                if !self.param_fog.is_null() && !chunk_uniform_buf.is_null() {
                                    let max_fog = if fog_enabled { (render_distance as f32 * 16.0).max(32.0) } else { -1.0 };
                                    let fog_data: [f32; 4] = [camera.position.x, camera.position.y, camera.position.z, max_fog];
                                    sceGxmSetUniformDataF(
                                        chunk_uniform_buf,
                                        self.param_fog,
                                        0,
                                        4,
                                        fog_data.as_ptr(),
                                    );
                                }
                                last_had_offset = true;
                            } else if last_had_offset {
                                let mut chunk_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
                                sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut chunk_uniform_buf);
                                if !self.param_mvp.is_null() && !chunk_uniform_buf.is_null() {
                                    sceGxmSetUniformDataF(
                                        chunk_uniform_buf,
                                        self.param_mvp,
                                        0,
                                        16,
                                        mvp.m.as_ptr(),
                                    );
                                }
                                if !self.param_fog.is_null() && !chunk_uniform_buf.is_null() {
                                    let max_fog = if fog_enabled { (render_distance as f32 * 16.0).max(32.0) } else { -1.0 };
                                    let fog_data: [f32; 4] = [camera.position.x, camera.position.y, camera.position.z, max_fog];
                                    sceGxmSetUniformDataF(
                                        chunk_uniform_buf,
                                        self.param_fog,
                                        0,
                                        4,
                                        fog_data.as_ptr(),
                                    );
                                }
                                last_had_offset = false;
                            }

                            let chunk_vb = vb_base.add(s * Self::MAX_VERTS_PER_SLOT);
                            let chunk_ib = ib_base.add(s * Self::MAX_INDICES_PER_SLOT);
                            sceGxmSetVertexStream(self.context, 0, chunk_vb as *const core::ffi::c_void);
                            sceGxmDraw(
                                self.context,
                                SCE_GXM_PRIMITIVE_TRIANGLES,
                                SCE_GXM_INDEX_FORMAT_U16,
                                chunk_ib as *const core::ffi::c_void,
                                idx_count,
                            );
                            tri_count += idx_count / 3;
                        }
                    }
                }

            if last_had_offset {
                let mut chunk_uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
                sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut chunk_uniform_buf);
                if !self.param_mvp.is_null() && !chunk_uniform_buf.is_null() {
                    sceGxmSetUniformDataF(
                        chunk_uniform_buf,
                        self.param_mvp,
                        0,
                        16,
                        mvp.m.as_ptr(),
                    );
                }
                if !self.param_fog.is_null() && !chunk_uniform_buf.is_null() {
                    let max_fog = if fog_enabled { (render_distance as f32 * 16.0).max(32.0) } else { -1.0 };
                    let fog_data: [f32; 4] = [camera.position.x, camera.position.y, camera.position.z, max_fog];
                    sceGxmSetUniformDataF(
                        chunk_uniform_buf,
                        self.param_fog,
                        0,
                        4,
                        fog_data.as_ptr(),
                    );
                }
            }

            // 2b. Rysowanie animacji pęknięć niszczonego bloku (3D block break cracks)
            if let Some(target) = break_target {
                if break_progress > 0.0 {
                    let stage = ((break_progress * 10.0).floor() as usize).min(9);
                    let crack_tex = TextureId::destroy_stage(stage);
                    let (u0, v0, u1, v1) = atlas_uv(crack_tex);

                    let eps = 0.004f32;
                    let x0 = target.x as f32 - eps;
                    let x1 = target.x as f32 + 1.0 + eps;
                    let y0 = target.y as f32 - eps;
                    let y1 = target.y as f32 + 1.0 + eps;
                    let z0 = target.z as f32 - eps;
                    let z1 = target.z as f32 + 1.0 + eps;

                    let crack_vb = self.crack_vb_mem.ptr as *mut GxmVertex;
                    let faces = [
                        // Top (+Y)
                        ([x0, y1, z1], [x1, y1, z1], [x1, y1, z0], [x0, y1, z0]),
                        // Bottom (-Y)
                        ([x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]),
                        // Front (+Z)
                        ([x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]),
                        // Back (-Z)
                        ([x1, y0, z0], [x0, y0, z0], [x0, y1, z0], [x1, y1, z0]),
                        // Left (-X)
                        ([x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]),
                        // Right (+X)
                        ([x1, y0, z1], [x1, y0, z0], [x1, y1, z0], [x1, y1, z1]),
                    ];

                    for (fi, &(c0, c1, c2, c3)) in faces.iter().enumerate() {
                        let base = fi * 4;
                        *crack_vb.add(base)     = GxmVertex { x: c0[0], y: c0[1], z: c0[2], u: u0, v: v1, ao: 1.0 };
                        *crack_vb.add(base + 1) = GxmVertex { x: c1[0], y: c1[1], z: c1[2], u: u1, v: v1, ao: 1.0 };
                        *crack_vb.add(base + 2) = GxmVertex { x: c2[0], y: c2[1], z: c2[2], u: u1, v: v0, ao: 1.0 };
                        *crack_vb.add(base + 3) = GxmVertex { x: c3[0], y: c3[1], z: c3[2], u: u0, v: v0, ao: 1.0 };
                    }

                    sceGxmSetVertexStream(self.context, 0, self.crack_vb_mem.ptr);
                    sceGxmDraw(
                        self.context,
                        SCE_GXM_PRIMITIVE_TRIANGLES,
                        SCE_GXM_INDEX_FORMAT_U16,
                        self.crack_ib_mem.ptr,
                        36,
                    );
                }
            }

            // 2c. Rysowanie obiektów dynamicznych (wypadnięte itemy na ziemi, moby, TNT) - wyłączamy culling dla dwustronnych billboardów/modeli
            sceGxmSetTwoSidedEnable(self.context, SCE_GXM_TWO_SIDED_ENABLED);
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_NONE);

            let mut ent_v_count = 0usize;
            let mut ent_i_count = 0usize;
            let ent_vb = self.entity_vb_mem.ptr as *mut GxmVertex;
            let ent_ib = self.entity_ib_mem.ptr as *mut u16;
            let max_ent_verts = Self::MAX_ENTITY_VERTS;
            let max_ent_idx = Self::MAX_ENTITY_INDICES;

            if clouds_enabled && !is_occluded {
                append_clouds(
                    camera.position,
                    total_time,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            append_particles(
                particles,
                camera,
                ent_vb,
                ent_ib,
                &mut ent_v_count,
                &mut ent_i_count,
                max_ent_verts,
                max_ent_idx,
            );

            for item in items {
                append_dropped_item(item, world, ent_vb, ent_ib, &mut ent_v_count, &mut ent_i_count, max_ent_verts, max_ent_idx);
            }

            for mob in mobs {
                append_mob(mob, camera.position, ent_vb, ent_ib, &mut ent_v_count, &mut ent_i_count, max_ent_verts, max_ent_idx);
            }

            for tnt in &world.primed_tnts {
                append_tnt(tnt, camera.position, ent_vb, ent_ib, &mut ent_v_count, &mut ent_i_count, max_ent_verts, max_ent_idx);
            }

            append_hand_3d(
                camera,
                hand_block,
                walk_phase,
                speed,
                attack_animation,
                ent_vb,
                ent_ib,
                &mut ent_v_count,
                &mut ent_i_count,
                max_ent_verts,
                max_ent_idx,
            );

            if is_underwater {
                append_screen_overlay(
                    camera,
                    TextureId::UnderwaterTint,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if is_in_lava {
                append_screen_overlay(
                    camera,
                    TextureId::LavaTint,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if damage_flash > 0.0 {
                append_screen_overlay(
                    camera,
                    TextureId::DamageFlash,
                    ent_vb,
                    ent_ib,
                    &mut ent_v_count,
                    &mut ent_i_count,
                    max_ent_verts,
                    max_ent_idx,
                );
            }

            if low_health {
                let pulse = (total_time * 6.5).sin() * 0.5 + 0.5;
                if pulse > 0.25 {
                    append_screen_overlay(
                        camera,
                        TextureId::VignetteTint,
                        ent_vb,
                        ent_ib,
                        &mut ent_v_count,
                        &mut ent_i_count,
                        max_ent_verts,
                        max_ent_idx,
                    );
                }
            }

            if ent_i_count > 0 {
                sceGxmSetVertexStream(self.context, 0, self.entity_vb_mem.ptr);
                sceGxmDraw(
                    self.context,
                    SCE_GXM_PRIMITIVE_TRIANGLES,
                    SCE_GXM_INDEX_FORMAT_U16,
                    self.entity_ib_mem.ptr,
                    ent_i_count as u32,
                );
                tri_count += ent_i_count as u32 / 3;
            }

            sceGxmEndScene(self.context, core::ptr::null(), core::ptr::null());

            // Synchronizacja GPU przed dostępem CPU do bufora klatki (eliminuje wszelkie glitche paska HUD)
            sceGxmFinish(self.context);

            // 3. Nałożenie 2D HUD (celownik, hotbar, wskaźnik niszczenia, panel debug)
            if let Some(hud) = hud_pixels {
                Self::blit_hud_overlay(color_buf_ptr, hud);
            }

            // 4. Asynchroniczne przekazanie do kolejki wyświetlania (Triple Buffering)
            sceGxmDisplayQueueAddEntry(
                self.display_sync[self.front_buf],
                self.display_sync[buf_idx],
                &self.display_data[buf_idx] as *const DisplayData as *const core::ffi::c_void,
            );

            self.front_buf = buf_idx;
            self.back_buf = (buf_idx + 1) % DISPLAY_BUFFER_COUNT;

            tri_count
        }
    }

    /// Kopiuje ostatnią wyświetloną klatkę 3D do bufora CPU i przyciemnia ją o 50% dla tła menu
    pub fn capture_frame_snapshot(&self, dst: &mut Option<Vec<u32>>) {
        let game_width = VITA_WIDTH as usize;
        let game_height = VITA_HEIGHT as usize;
        let y_offset = 0usize;
        let vec = dst.get_or_insert_with(|| vec![0u32; game_width * game_height]);
        if vec.len() != game_width * game_height {
            vec.resize(game_width * game_height, 0);
        }
        let src_buf = self.display_bufs[self.front_buf].ptr as *const u32;
        unsafe {
            for y in 0..game_height {
                let src_offset = (y + y_offset) * VITA_STRIDE as usize;
                let dst_offset = y * game_width;
                let src_ptr = src_buf.add(src_offset);
                let dst_ptr = vec.as_mut_ptr().add(dst_offset);
                for x in 0..game_width {
                    let abgr = *src_ptr.add(x);
                    let r = abgr & 0xFF;
                    let g = (abgr >> 8) & 0xFF;
                    let b = (abgr >> 16) & 0xFF;
                    // Format XRGB dla SoftwareRenderer z przyciemnieniem o 50%
                    let dimmed_r = r >> 1;
                    let dimmed_g = g >> 1;
                    let dimmed_b = b >> 1;
                    *dst_ptr.add(x) = (dimmed_r << 16) | (dimmed_g << 8) | dimmed_b;
                }
            }
        }
    }
}

fn append_quad(
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
    c0: Vec3,
    c1: Vec3,
    c2: Vec3,
    c3: Vec3,
    texture: TextureId,
    ao: f32,
) {
    if *v_count + 4 > max_verts || *i_count + 6 > max_idx {
        return;
    }
    let (u0, v0, u1, v1) = atlas_uv(texture);
    let base = *v_count as u16;
    unsafe {
        *vb.add(*v_count)     = GxmVertex { x: c0.x, y: c0.y, z: c0.z, u: u0, v: v1, ao };
        *vb.add(*v_count + 1) = GxmVertex { x: c1.x, y: c1.y, z: c1.z, u: u1, v: v1, ao };
        *vb.add(*v_count + 2) = GxmVertex { x: c2.x, y: c2.y, z: c2.z, u: u1, v: v0, ao };
        *vb.add(*v_count + 3) = GxmVertex { x: c3.x, y: c3.y, z: c3.z, u: u0, v: v0, ao };

        *ib.add(*i_count)     = base;
        *ib.add(*i_count + 1) = base + 1;
        *ib.add(*i_count + 2) = base + 2;
        *ib.add(*i_count + 3) = base;
        *ib.add(*i_count + 4) = base + 2;
        *ib.add(*i_count + 5) = base + 3;
    }
    *v_count += 4;
    *i_count += 6;
}

fn append_clouds(
    player_pos: Vec3,
    time: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let cloud_y = 75.0f32;
    let cloud_thickness = 4.0f32;
    let y_bottom = cloud_y;
    let y_top = cloud_y + cloud_thickness;

    if (player_pos.y - cloud_y).abs() > 100.0 {
        return;
    }

    let cloud_size = 12.0f32;
    let scroll = time * 1.5;

    let is_cloud = |gx: i32, gz: i32| -> bool {
        let hx = gx.wrapping_mul(374761393i32);
        let hz = gz.wrapping_mul(668265263i32);
        let h = (hx ^ hz ^ hx.wrapping_mul(hz >> 3)) as u32;
        let fx = gx as f32 * 0.16;
        let fz = gz as f32 * 0.16;
        let wave = (fx.sin() + fz.cos() + (fx * 0.7 + fz * 0.5).sin()) * 0.35;
        let r = ((h >> 16) % 100) as f32 / 100.0;
        (wave + r * 0.35) > 0.08
    };

    let cx = ((player_pos.x - scroll) / cloud_size).floor() as i32;
    let cz = (player_pos.z / cloud_size).floor() as i32;

    for dz in -5..=5 {
        for dx in -5..=5 {
            let gx = cx + dx;
            let gz = cz + dz;

            let x_min = gx as f32 * cloud_size + scroll;
            let x_max = x_min + cloud_size;
            let z_min = gz as f32 * cloud_size;
            let z_max = z_min + cloud_size;

            if !is_cloud(gx, gz) {
                continue;
            }

            let p000 = Vec3::new(x_min, y_bottom, z_min);
            let p100 = Vec3::new(x_max, y_bottom, z_min);
            let p110 = Vec3::new(x_max, y_top, z_min);
            let p010 = Vec3::new(x_min, y_top, z_min);
            let p001 = Vec3::new(x_min, y_bottom, z_max);
            let p101 = Vec3::new(x_max, y_bottom, z_max);
            let p111 = Vec3::new(x_max, y_top, z_max);
            let p011 = Vec3::new(x_min, y_top, z_max);

            // Top (+Y)
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p011, p111, p110, p010, TextureId::Snow, 1.0);
            // Bottom (-Y)
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p000, p100, p101, p001, TextureId::Snow, 0.7);
            // Sides
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p000, p010, p110, p100, TextureId::Snow, 0.85);
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p101, p111, p011, p001, TextureId::Snow, 0.85);
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p001, p011, p010, p000, TextureId::Snow, 0.75);
            append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p100, p110, p111, p101, TextureId::Snow, 0.75);
        }
    }
}

fn append_particles(
    particles: &[crate::gameplay::particles::Particle],
    camera: &Camera,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let right = camera.right();
    let up = camera.up();

    for p in particles {
        if *v_count + 4 > max_verts || *i_count + 6 > max_idx {
            break;
        }
        let half = p.size * 0.5;
        let p0 = p.pos - right * half - up * half;
        let p1 = p.pos + right * half - up * half;
        let p2 = p.pos + right * half + up * half;
        let p3 = p.pos - right * half + up * half;

        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, TextureId::Dirt, 1.0);
    }
}

fn append_screen_overlay(
    camera: &Camera,
    texture: TextureId,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    if *v_count + 4 > max_verts || *i_count + 6 > max_idx {
        return;
    }
    let fwd = camera.forward();
    let right = camera.right();
    let up = camera.up();
    let dist = 0.06; // Near-plane is 0.05
    let center = camera.position + fwd * dist;
    let half_h = dist * (camera.fov_y * 0.5).tan() * 1.5;
    let half_w = half_h * (960.0 / 544.0);

    let p0 = center - right * half_w - up * half_h;
    let p1 = center + right * half_w - up * half_h;
    let p2 = center + right * half_w + up * half_h;
    let p3 = center - right * half_w + up * half_h;

    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, texture, 1.0);
}

fn append_dropped_item(
    item: &crate::gameplay::item::ItemEntity,
    world: &crate::world::World,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    use crate::world::block::Face;

    // 1. Okrągły cień na ziemi pod bloczkiem (jak w Minecraft)
    let ipos_x = item.position.x.floor() as i32;
    let ipos_z = item.position.z.floor() as i32;
    let item_y = item.position.y;
    for dy in 0..=3 {
        let test_y = (item_y.floor() as i32) - dy;
        let b = world.get_block(crate::core::math::IVec3::new(ipos_x, test_y, ipos_z));
        if b.is_solid() {
            let top_y = test_y as f32 + 1.0 + 0.015;
            if item_y >= top_y - 0.2 {
                let s_rad = 0.22f32;
                let sx0 = item.position.x - s_rad;
                let sx1 = item.position.x + s_rad;
                let sz0 = item.position.z - s_rad;
                let sz1 = item.position.z + s_rad;
                let s_c0 = Vec3::new(sx0, top_y, sz1);
                let s_c1 = Vec3::new(sx1, top_y, sz1);
                let s_c2 = Vec3::new(sx1, top_y, sz0);
                let s_c3 = Vec3::new(sx0, top_y, sz0);
                append_quad(
                    vb, ib, v_count, i_count, max_verts, max_idx,
                    s_c0, s_c1, s_c2, s_c3,
                    TextureId::EntityShadow,
                    1.0,
                );
                break;
            }
        }
    }

    let y_offset = (item.age * 3.0).sin() * 0.08;
    let mut pos = item.position;
    pos.y += y_offset;
    let yaw = item.age * 2.5;

    let size = 0.28f32;
    let half = size * 0.5f32;
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let up = Vec3::new(0.0, 1.0, 0.0);
    let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());

    let corner = |x: f32, y: f32, z: f32| {
        pos + right * (x * half) + up * (y * half) + forward * (z * half)
    };

    let block = item.block;
    let face_data = [
        ([corner(-1.0, 1.0, 1.0), corner(1.0, 1.0, 1.0), corner(1.0, 1.0, -1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(Face::Top), 1.00),
        ([corner(-1.0, -1.0, -1.0), corner(1.0, -1.0, -1.0), corner(1.0, -1.0, 1.0), corner(-1.0, -1.0, 1.0)], block.texture_id(Face::Bottom), 0.55),
        ([corner(-1.0, -1.0, 1.0), corner(1.0, -1.0, 1.0), corner(1.0, 1.0, 1.0), corner(-1.0, 1.0, 1.0)], block.texture_id(Face::South), 0.85),
        ([corner(1.0, -1.0, -1.0), corner(-1.0, -1.0, -1.0), corner(-1.0, 1.0, -1.0), corner(1.0, 1.0, -1.0)], block.texture_id(Face::North), 0.85),
        ([corner(-1.0, -1.0, -1.0), corner(-1.0, -1.0, 1.0), corner(-1.0, 1.0, 1.0), corner(-1.0, 1.0, -1.0)], block.texture_id(Face::West), 0.70),
        ([corner(1.0, -1.0, 1.0), corner(1.0, -1.0, -1.0), corner(1.0, 1.0, -1.0), corner(1.0, 1.0, 1.0)], block.texture_id(Face::East), 0.70),
    ];

    for (corners, tex, ao) in face_data {
        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, corners[0], corners[1], corners[2], corners[3], tex, ao);
    }
}

struct GxmMobBox {
    center: Vec3,
    size: Vec3,
    yaw: f32,
    pitch: f32,
    roll: f32,
    origin: Vec3,
    tex_top: TextureId,
    tex_bottom: TextureId,
    tex_side: TextureId,
    tex_front: TextureId,
}

fn append_mob_box(
    b: &GxmMobBox,
    camera_pos: Vec3,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let half = Vec3::new(b.size.x * 0.5, b.size.y * 0.5, b.size.z * 0.5);

    let local_corners = [
        b.center + Vec3::new(-half.x, -half.y, -half.z),
        b.center + Vec3::new(half.x, -half.y, -half.z),
        b.center + Vec3::new(half.x, half.y, -half.z),
        b.center + Vec3::new(-half.x, half.y, -half.z),
        b.center + Vec3::new(-half.x, -half.y, half.z),
        b.center + Vec3::new(half.x, -half.y, half.z),
        b.center + Vec3::new(half.x, half.y, half.z),
        b.center + Vec3::new(-half.x, half.y, half.z),
    ];

    let transform_point = |p: Vec3| -> Vec3 {
        let p_pitch = if b.pitch != 0.0 {
            Vec3::new(
                p.x,
                p.y * b.pitch.cos() - p.z * b.pitch.sin(),
                p.y * b.pitch.sin() + p.z * b.pitch.cos(),
            )
        } else {
            p
        };

        let p_roll = if b.roll != 0.0 {
            Vec3::new(
                p_pitch.x * b.roll.cos() - p_pitch.y * b.roll.sin(),
                p_pitch.x * b.roll.sin() + p_pitch.y * b.roll.cos(),
                p_pitch.z,
            )
        } else {
            p_pitch
        };

        let p_yaw = Vec3::new(
            p_roll.x * b.yaw.cos() + p_roll.z * b.yaw.sin(),
            p_roll.y,
            -p_roll.x * b.yaw.sin() + p_roll.z * b.yaw.cos(),
        );

        b.origin + p_yaw
    };

    let world_pts: [Vec3; 8] = [
        transform_point(local_corners[0]),
        transform_point(local_corners[1]),
        transform_point(local_corners[2]),
        transform_point(local_corners[3]),
        transform_point(local_corners[4]),
        transform_point(local_corners[5]),
        transform_point(local_corners[6]),
        transform_point(local_corners[7]),
    ];

    let mut append_face = |i0: usize, i1: usize, i2: usize, i3: usize, tex: TextureId, shade: f32| {
        let p0 = world_pts[i0];
        let p1 = world_pts[i1];
        let p2 = world_pts[i2];
        let p3 = world_pts[i3];

        let edge1 = p1 - p0;
        let edge2 = p3 - p0;
        let normal = edge1.cross(edge2);
        let center = (p0 + p1 + p2 + p3) * 0.25;
        let to_face = center - camera_pos;
        if to_face.dot(normal) >= 0.0 {
            return;
        }

        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, p0, p1, p2, p3, tex, shade);
    };

    append_face(7, 6, 2, 3, b.tex_top, 1.00);
    append_face(4, 0, 1, 5, b.tex_bottom, 0.50);
    append_face(4, 5, 6, 7, b.tex_front, 0.85);
    append_face(1, 0, 3, 2, b.tex_side, 0.85);
    append_face(0, 4, 7, 3, b.tex_side, 0.70);
    append_face(5, 1, 2, 6, b.tex_side, 0.70);
}

fn append_mob(
    mob: &crate::gameplay::mob::Mob,
    camera_pos: Vec3,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    use crate::gameplay::mob::MobKind;

    let death_t = (mob.death_timer / 0.8).clamp(0.0, 1.0);
    let roll_angle = death_t * std::f32::consts::FRAC_PI_2;

    let base_pos = mob.pos;
    let yaw = mob.yaw;
    let leg_swing = mob.walk_phase.sin() * 0.65;

    match mob.kind {
        MobKind::Pig => {
            let body_box = GxmMobBox {
                center: Vec3::new(0.0, 0.45, 0.0),
                size: Vec3::new(0.65, 0.45, 0.90),
                yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: base_pos,
                tex_top: TextureId::PigSkin,
                tex_bottom: TextureId::PigSkin,
                tex_side: TextureId::PigSkin,
                tex_front: TextureId::PigSkin,
            };
            append_mob_box(&body_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            let head_pos = base_pos + Vec3::new(yaw.sin() * 0.48, 0.52, yaw.cos() * 0.48);
            let head_yaw = yaw + mob.head_yaw_offset;
            let head_box = GxmMobBox {
                center: Vec3::zero(),
                size: Vec3::new(0.48, 0.48, 0.48),
                yaw: head_yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: head_pos,
                tex_top: TextureId::PigSkin,
                tex_bottom: TextureId::PigSkin,
                tex_side: TextureId::PigSkin,
                tex_front: TextureId::PigFace,
            };
            append_mob_box(&head_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            let snout_pos = head_pos + Vec3::new(head_yaw.sin() * 0.28, -0.06, head_yaw.cos() * 0.28);
            let snout_box = GxmMobBox {
                center: Vec3::zero(),
                size: Vec3::new(0.24, 0.18, 0.12),
                yaw: head_yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: snout_pos,
                tex_top: TextureId::PigSkin,
                tex_bottom: TextureId::PigSkin,
                tex_side: TextureId::PigSkin,
                tex_front: TextureId::PigSnout,
            };
            append_mob_box(&snout_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            let leg_offsets = [
                (-0.22, -0.28, leg_swing),
                (0.22, -0.28, -leg_swing),
                (-0.22, 0.28, -leg_swing),
                (0.22, 0.28, leg_swing),
            ];

            for &(lx, lz, swing) in &leg_offsets {
                let rotated_x = lx * yaw.cos() + lz * yaw.sin();
                let rotated_z = -lx * yaw.sin() + lz * yaw.cos();
                let leg_origin = base_pos + Vec3::new(rotated_x, 0.30, rotated_z);

                let leg_box = GxmMobBox {
                    center: Vec3::new(0.0, -0.15, 0.0),
                    size: Vec3::new(0.20, 0.30, 0.20),
                    yaw,
                    pitch: swing,
                    roll: roll_angle,
                    origin: leg_origin,
                    tex_top: TextureId::PigSkin,
                    tex_bottom: TextureId::PigSkin,
                    tex_side: TextureId::PigSkin,
                    tex_front: TextureId::PigSkin,
                };
                append_mob_box(&leg_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);
            }
        }
        MobKind::Cow => {
            let body_box = GxmMobBox {
                center: Vec3::new(0.0, 0.78, 0.0),
                size: Vec3::new(0.75, 0.65, 1.15),
                yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: base_pos,
                tex_top: TextureId::CowSkin,
                tex_bottom: TextureId::CowSkin,
                tex_side: TextureId::CowSkin,
                tex_front: TextureId::CowSkin,
            };
            append_mob_box(&body_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            let udder_pos = base_pos + Vec3::new(0.0, 0.44, 0.0);
            let udder_box = GxmMobBox {
                center: Vec3::zero(),
                size: Vec3::new(0.28, 0.12, 0.36),
                yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: udder_pos,
                tex_top: TextureId::CowUdder,
                tex_bottom: TextureId::CowUdder,
                tex_side: TextureId::CowUdder,
                tex_front: TextureId::CowUdder,
            };
            append_mob_box(&udder_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            let head_pos = base_pos + Vec3::new(yaw.sin() * 0.62, 0.95, yaw.cos() * 0.62);
            let head_yaw = yaw + mob.head_yaw_offset;
            let head_box = GxmMobBox {
                center: Vec3::zero(),
                size: Vec3::new(0.48, 0.48, 0.46),
                yaw: head_yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: head_pos,
                tex_top: TextureId::CowSkin,
                tex_bottom: TextureId::CowSkin,
                tex_side: TextureId::CowSkin,
                tex_front: TextureId::CowFace,
            };
            append_mob_box(&head_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            let snout_pos = head_pos + Vec3::new(head_yaw.sin() * 0.26, -0.10, head_yaw.cos() * 0.26);
            let snout_box = GxmMobBox {
                center: Vec3::zero(),
                size: Vec3::new(0.34, 0.22, 0.16),
                yaw: head_yaw,
                pitch: 0.0,
                roll: roll_angle,
                origin: snout_pos,
                tex_top: TextureId::CowSkin,
                tex_bottom: TextureId::CowSkin,
                tex_side: TextureId::CowSkin,
                tex_front: TextureId::CowSnout,
            };
            append_mob_box(&snout_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);

            for side in [-1.0f32, 1.0f32] {
                let horn_x = side * 0.28 * head_yaw.cos();
                let horn_z = -side * 0.28 * head_yaw.sin();
                let horn_pos = head_pos + Vec3::new(horn_x, 0.26, horn_z);
                let horn_box = GxmMobBox {
                    center: Vec3::zero(),
                    size: Vec3::new(0.10, 0.18, 0.10),
                    yaw: head_yaw,
                    pitch: 0.0,
                    roll: roll_angle,
                    origin: horn_pos,
                    tex_top: TextureId::CowHorns,
                    tex_bottom: TextureId::CowHorns,
                    tex_side: TextureId::CowHorns,
                    tex_front: TextureId::CowHorns,
                };
                append_mob_box(&horn_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);
            }

            let leg_offsets = [
                (-0.25, -0.38, leg_swing),
                (0.25, -0.38, -leg_swing),
                (-0.25, 0.38, -leg_swing),
                (0.25, 0.38, leg_swing),
            ];

            for &(lx, lz, swing) in &leg_offsets {
                let rotated_x = lx * yaw.cos() + lz * yaw.sin();
                let rotated_z = -lx * yaw.sin() + lz * yaw.cos();
                let leg_origin = base_pos + Vec3::new(rotated_x, 0.50, rotated_z);

                let leg_box = GxmMobBox {
                    center: Vec3::new(0.0, -0.25, 0.0),
                    size: Vec3::new(0.22, 0.50, 0.22),
                    yaw,
                    pitch: swing,
                    roll: roll_angle,
                    origin: leg_origin,
                    tex_top: TextureId::CowSkin,
                    tex_bottom: TextureId::CowSkin,
                    tex_side: TextureId::CowSkin,
                    tex_front: TextureId::CowSkin,
                };
                append_mob_box(&leg_box, camera_pos, vb, ib, v_count, i_count, max_verts, max_idx);
            }
        }
    }
}

fn append_tnt(
    tnt: &crate::world::PrimedTnt,
    camera_pos: Vec3,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let p = tnt.pos;
    let (x0, x1) = (p.x, p.x + 1.0);
    let (y0, y1) = (p.y, p.y + 1.0);
    let (z0, z1) = (p.z, p.z + 1.0);

    let faces = [
        (Vec3::new(x0, y1, z1), Vec3::new(x1, y1, z1), Vec3::new(x1, y1, z0), Vec3::new(x0, y1, z0), TextureId::TntTop, 1.00),
        (Vec3::new(x0, y0, z0), Vec3::new(x1, y0, z0), Vec3::new(x1, y0, z1), Vec3::new(x0, y0, z1), TextureId::TntTop, 0.50),
        (Vec3::new(x0, y0, z1), Vec3::new(x1, y0, z1), Vec3::new(x1, y1, z1), Vec3::new(x0, y1, z1), TextureId::TntSide, 0.85),
        (Vec3::new(x1, y0, z0), Vec3::new(x0, y0, z0), Vec3::new(x0, y1, z0), Vec3::new(x1, y1, z0), TextureId::TntSide, 0.85),
        (Vec3::new(x0, y0, z0), Vec3::new(x0, y0, z1), Vec3::new(x0, y1, z1), Vec3::new(x0, y1, z0), TextureId::TntSide, 0.70),
        (Vec3::new(x1, y0, z1), Vec3::new(x1, y0, z0), Vec3::new(x1, y1, z0), Vec3::new(x1, y1, z1), TextureId::TntSide, 0.70),
    ];

    for (c0, c1, c2, c3, tex, ao) in faces {
        let edge1 = c1 - c0;
        let edge2 = c3 - c0;
        let normal = edge1.cross(edge2);
        let center = (c0 + c1 + c2 + c3) * 0.25;
        let to_face = center - camera_pos;
        if to_face.dot(normal) >= 0.0 {
            continue;
        }
        append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c0, c1, c2, c3, tex, ao);
    }
}

fn append_cube_oriented(
    pos: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
    size: [f32; 3],
    tex_top: TextureId,
    tex_bottom: TextureId,
    tex_side: TextureId,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let half_x = size[0] * 0.5;
    let half_y = size[1] * 0.5;
    let half_z = size[2] * 0.5;

    let corner = |x: f32, y: f32, z: f32| -> Vec3 {
        pos + right * (x * half_x) + up * (y * half_y) + forward * (z * half_z)
    };

    let c000 = corner(-1.0, -1.0, -1.0);
    let c100 = corner( 1.0, -1.0, -1.0);
    let c010 = corner(-1.0,  1.0, -1.0);
    let c110 = corner( 1.0,  1.0, -1.0);
    let c001 = corner(-1.0, -1.0,  1.0);
    let c101 = corner( 1.0, -1.0,  1.0);
    let c011 = corner(-1.0,  1.0,  1.0);
    let c111 = corner( 1.0,  1.0,  1.0);

    // Top (+Y)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c010, c110, c111, c011, tex_top, 1.00);
    // Bottom (-Y)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c001, c101, c100, c000, tex_bottom, 0.50);
    // Front (+Z)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c001, c011, c111, c101, tex_side, 0.85);
    // Back (-Z)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c100, c110, c010, c000, tex_side, 0.85);
    // Right (+X)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c101, c111, c110, c100, tex_side, 0.70);
    // Left (-X)
    append_quad(vb, ib, v_count, i_count, max_verts, max_idx, c000, c010, c011, c001, tex_side, 0.70);
}

fn append_procedural_tool_3d(
    block: BlockId,
    pos: Vec3,
    tool_right: Vec3,
    tool_up: Vec3,
    tool_forward: Vec3,
    scale: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let mat_tex = match block {
        BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword => TextureId::Planks,
        BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword => TextureId::Stone,
        BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword => TextureId::IronOre,
        _ => TextureId::WoodSide,
    };
    let handle_tex = TextureId::WoodSide;

    let mut cube = |offset: Vec3, sz: [f32; 3], tex: TextureId| {
        append_cube_oriented(
            pos + offset,
            tool_right,
            tool_up,
            tool_forward,
            sz,
            tex,
            tex,
            tex,
            vb,
            ib,
            v_count,
            i_count,
            max_verts,
            max_idx,
        );
    };

    match block {
        BlockId::Stick => {
            cube(Vec3::zero(), [0.016 * scale, 0.26 * scale, 0.016 * scale], handle_tex);
        }
        BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword => {
            // Pommel
            cube(-tool_up * (0.16 * scale), [0.024 * scale, 0.018 * scale, 0.024 * scale], handle_tex);
            // Grip
            cube(-tool_up * (0.08 * scale), [0.016 * scale, 0.065 * scale, 0.016 * scale], handle_tex);
            // Crossguard
            cube(Vec3::zero(), [0.085 * scale, 0.016 * scale, 0.026 * scale], mat_tex);
            // Blade
            cube(tool_up * (0.20 * scale), [0.038 * scale, 0.180 * scale, 0.010 * scale], mat_tex);
            // Tip
            cube(tool_up * (0.41 * scale), [0.020 * scale, 0.040 * scale, 0.008 * scale], mat_tex);
        }
        BlockId::WoodenPickaxe | BlockId::StonePickaxe | BlockId::IronPickaxe => {
            // Handle
            cube(Vec3::zero(), [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            // Head center mount
            cube(tool_up * (0.26 * scale), [0.032 * scale, 0.032 * scale, 0.032 * scale], mat_tex);
            // Head crossbar
            cube(tool_up * (0.27 * scale), [0.130 * scale, 0.024 * scale, 0.024 * scale], mat_tex);
            // Left & right pick tips
            cube(tool_up * (0.23 * scale) - tool_right * (0.12 * scale), [0.018 * scale, 0.032 * scale, 0.018 * scale], mat_tex);
            cube(tool_up * (0.23 * scale) + tool_right * (0.12 * scale), [0.018 * scale, 0.032 * scale, 0.018 * scale], mat_tex);
        }
        BlockId::WoodenAxe | BlockId::StoneAxe | BlockId::IronAxe => {
            // Handle
            cube(Vec3::zero(), [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            // Axe Head Main
            cube(tool_up * (0.22 * scale) + tool_right * (0.045 * scale), [0.048 * scale, 0.065 * scale, 0.022 * scale], mat_tex);
            // Blade edge
            cube(tool_up * (0.21 * scale) + tool_right * (0.088 * scale), [0.014 * scale, 0.078 * scale, 0.012 * scale], mat_tex);
            // Poll (rear)
            cube(tool_up * (0.25 * scale) - tool_right * (0.024 * scale), [0.020 * scale, 0.028 * scale, 0.020 * scale], mat_tex);
        }
        BlockId::WoodenShovel | BlockId::StoneShovel | BlockId::IronShovel => {
            // Handle
            cube(Vec3::zero(), [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            // Grip Topper
            cube(-tool_up * (0.27 * scale), [0.042 * scale, 0.014 * scale, 0.018 * scale], handle_tex);
            // Spade Blade
            cube(tool_up * (0.33 * scale), [0.065 * scale, 0.075 * scale, 0.012 * scale], mat_tex);
            // Spade Tip
            cube(tool_up * (0.42 * scale), [0.038 * scale, 0.018 * scale, 0.010 * scale], mat_tex);
        }
        _ => {}
    }
}

fn append_hand_3d(
    camera: &Camera,
    block_id: Option<u8>,
    walk_phase: f32,
    speed: f32,
    attack_animation: f32,
    vb: *mut GxmVertex,
    ib: *mut u16,
    v_count: &mut usize,
    i_count: &mut usize,
    max_verts: usize,
    max_idx: usize,
) {
    let forward = camera.forward();
    let right = camera.forward().cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(forward).normalized();
    let movement = (speed / 8.5).clamp(0.0, 1.0);
    let swing = walk_phase.sin() * 0.10 * movement;
    let attack = (attack_animation * core::f32::consts::PI).sin();
    let swing = swing + attack * 0.16;
    let bob = (walk_phase * 2.0).cos() * 0.035 * movement - attack.abs() * 0.08;
    let hand_pos = camera.position
        + forward * 0.68
        + right * (0.53 + swing)
        + up * (-0.40 + bob);
    let arm_pos = hand_pos - right * 0.10 - up * 0.13;

    // Always draw arm (sleeve/skin)
    append_cube_oriented(
        arm_pos,
        right,
        up,
        forward,
        [0.085, 0.085, 0.12],
        TextureId::Dirt,
        TextureId::Dirt,
        TextureId::Dirt,
        vb,
        ib,
        v_count,
        i_count,
        max_verts,
        max_idx,
    );

    // Draw held tool or block
    if let Some(id) = block_id {
        let block = BlockId::from_u8(id);
        if block.is_tool() {
            let swing_pitch = attack * 0.85;
            let tool_up = (up * (1.0 - swing_pitch * 0.5) + forward * (0.85 + swing_pitch) - right * 0.20).normalized();
            let tool_right = (right * 0.95 + up * 0.15).normalized();
            let tool_forward = tool_up.cross(tool_right).normalized();
            let tool_origin = hand_pos + forward * (0.10 + attack * 0.15) + up * (0.12 - attack * 0.12) - right * 0.02;

            append_procedural_tool_3d(
                block,
                tool_origin,
                tool_right,
                tool_up,
                tool_forward,
                1.0,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        } else {
            let top_tex = block.texture_id(crate::world::block::Face::Top);
            let side_tex = block.texture_id(crate::world::block::Face::North);
            let bot_tex = block.texture_id(crate::world::block::Face::Bottom);
            append_cube_oriented(
                hand_pos,
                right,
                up,
                forward,
                [0.13, 0.13, 0.13],
                top_tex,
                bot_tex,
                side_tex,
                vb,
                ib,
                v_count,
                i_count,
                max_verts,
                max_idx,
            );
        }
    }
}

impl GxmRenderer {
    unsafe fn upload_chunk(
        chunk: &crate::world::Chunk,
        world: &crate::world::World,
        vb_base: *mut GxmVertex,
        ib_base: *mut u16,
        slot_idx: usize,
    ) -> u32 {
        let tris = build_chunk_mesh(chunk, world);
        let max_verts = Self::MAX_VERTS_PER_SLOT;
        let max_idx = Self::MAX_INDICES_PER_SLOT;

        let slot_vb = vb_base.add(slot_idx * Self::MAX_VERTS_PER_SLOT);
        let slot_ib = ib_base.add(slot_idx * Self::MAX_INDICES_PER_SLOT);

        let mut v_count = 0usize;
        let mut i_count = 0usize;

        for tri in &tris {
            if v_count + 3 > max_verts || i_count + 3 > max_idx {
                break;
            }
            let (u0, v0, u1, v1) = atlas_uv(tri.texture);
            let base = v_count as u16;

            *slot_vb.add(v_count) = GxmVertex::from_vertex(&tri.v0, u0, v0, u1, v1);
            *slot_vb.add(v_count + 1) = GxmVertex::from_vertex(&tri.v1, u0, v0, u1, v1);
            *slot_vb.add(v_count + 2) = GxmVertex::from_vertex(&tri.v2, u0, v0, u1, v1);

            *slot_ib.add(i_count) = base;
            *slot_ib.add(i_count + 1) = base + 1;
            *slot_ib.add(i_count + 2) = base + 2;

            v_count += 3;
            i_count += 3;
        }

        i_count as u32
    }

    /// Prezentuje 2D bufor pikseli (np. Główne Menu, Pauza, Ekwipunek, Ustawienia)
    pub fn present_2d_buffer(&mut self, src_pixels: &[u32], game_width: usize, game_height: usize) {
        let buf_idx = self.back_buf;
        let dst_buf = self.display_bufs[buf_idx].ptr as *mut u32;

        let y_offset = if (VITA_HEIGHT as usize) > game_height {
            (VITA_HEIGHT as usize - game_height) / 2
        } else {
            0
        };

        let copy_width = game_width.min(VITA_WIDTH as usize);
        let copy_height = game_height.min(VITA_HEIGHT as usize);

        unsafe {
            if y_offset > 0 {
                let dst_slice = core::slice::from_raw_parts_mut(dst_buf, (VITA_WIDTH * VITA_HEIGHT) as usize);
                dst_slice.fill(0xFF000000);
            }

            for y in 0..copy_height {
                let src_offset = y * game_width;
                let dst_offset = (y + y_offset) * VITA_STRIDE as usize;
                let src_ptr = src_pixels.as_ptr().add(src_offset);
                let dst_ptr = dst_buf.add(dst_offset);
                for px in 0..copy_width {
                    let xrgb = *src_ptr.add(px);
                    let r = (xrgb >> 16) & 0xFF;
                    let g = (xrgb >> 8) & 0xFF;
                    let b = xrgb & 0xFF;
                    *dst_ptr.add(px) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                }
            }

            sceGxmPadHeartbeat(&self.color_surfaces[buf_idx], self.display_sync[buf_idx]);

            sceGxmDisplayQueueAddEntry(
                self.display_sync[self.front_buf],
                self.display_sync[buf_idx],
                &self.display_data[buf_idx] as *const DisplayData as *const core::ffi::c_void,
            );

            self.front_buf = buf_idx;
            self.back_buf = (buf_idx + 1) % DISPLAY_BUFFER_COUNT;
        }
    }

    /// Kopiuje elementy HUD na wyrenderowaną scenę 3D (zoptymalizowany zapis bez odczytu VRAM)
    fn blit_hud_overlay(dst_buf: *mut u32, src_pixels: &[u32]) {
        let game_width = 960usize;
        let game_height = 540usize.min(src_pixels.len() / game_width);
        let y_offset = 2usize;

        unsafe {
            for y in 0..game_height {
                let src_row = y * game_width;
                let dst_row = (y + y_offset) * VITA_STRIDE as usize;
                let src_ptr = src_pixels.as_ptr().add(src_row);
                let dst_ptr = dst_buf.add(dst_row);

                let mut x = 0;
                while x + 4 <= game_width {
                    let quad = (src_ptr.add(x) as *const [u64; 2]).read_unaligned();
                    if quad[0] == 0 && quad[1] == 0 {
                        x += 4;
                        continue;
                    }
                    for i in 0..4 {
                        let px = x + i;
                        let pixel = *src_ptr.add(px);
                        let a = (pixel >> 24) & 0xFF;
                        if a > 128 {
                            let r = (pixel >> 16) & 0xFF;
                            let g = (pixel >> 8) & 0xFF;
                            let b = pixel & 0xFF;
                            *dst_ptr.add(px) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                        }
                    }
                    x += 4;
                }
                while x < game_width {
                    let pixel = *src_ptr.add(x);
                    let a = (pixel >> 24) & 0xFF;
                    if a > 128 {
                        let r = (pixel >> 16) & 0xFF;
                        let g = (pixel >> 8) & 0xFF;
                        let b = pixel & 0xFF;
                        *dst_ptr.add(x) = (0xFF << 24) | (b << 16) | (g << 8) | r;
                    }
                    x += 1;
                }
            }
        }
    }

    pub fn invalidate_chunk(&mut self, cx: i32, cz: i32) {
        if let Some(slot) = self.chunk_slots.remove(&(cx, cz)) {
            self.free_slots.push(slot);
            self.chunk_spawn_times.remove(&(cx, cz));
        }
    }

    #[inline]
    fn is_chunk_in_frustum(chunk_x: i32, chunk_z: i32, mvp: &crate::core::math::Mat4) -> bool {
        let min_x = (chunk_x * CHUNK_SIZE_X as i32) as f32;
        let max_x = min_x + CHUNK_SIZE_X as f32;
        let min_y = 0.0f32;
        let max_y = CHUNK_SIZE_Y as f32;
        let min_z = (chunk_z * CHUNK_SIZE_Z as i32) as f32;
        let max_z = min_z + CHUNK_SIZE_Z as f32;

        let corners = [
            [min_x, min_y, min_z],
            [max_x, min_y, min_z],
            [min_x, max_y, min_z],
            [max_x, max_y, min_z],
            [min_x, min_y, max_z],
            [max_x, min_y, max_z],
            [min_x, max_y, max_z],
            [max_x, max_y, max_z],
        ];

        let m = &mvp.m;

        let mut out_left = true;
        let mut out_right = true;
        let mut out_bottom = true;
        let mut out_top = true;
        let mut out_near = true;
        let mut out_far = true;

        for corner in &corners {
            let x = corner[0];
            let y = corner[1];
            let z = corner[2];

            let clip_x = m[0] * x + m[4] * y + m[8] * z + m[12];
            let clip_y = m[1] * x + m[5] * y + m[9] * z + m[13];
            let clip_z = m[2] * x + m[6] * y + m[10] * z + m[14];
            let clip_w = m[3] * x + m[7] * y + m[11] * z + m[15];

            if clip_x >= -clip_w { out_left = false; }
            if clip_x <= clip_w { out_right = false; }
            if clip_y >= -clip_w { out_bottom = false; }
            if clip_y <= clip_w { out_top = false; }
            if clip_z >= 0.0 { out_near = false; }
            if clip_z <= clip_w { out_far = false; }
        }

        !(out_left || out_right || out_bottom || out_top || out_near || out_far)
    }

    /// Aktualizuje tekstury w pamięci CDRAM GPU po zmianie ustawień graficznych (np. przełącznik Fancy Leaves) oraz animacje
    pub fn update_textures(&mut self, atlas: &TextureAtlas) {
        let pixels = self._atlas_mem.ptr as *mut u32;
        unsafe {
            populate_atlas(pixels, atlas);
        }
    }

    /// Czyści wszystkie zarezerwowane sloty chunków na GPU, wymuszając ich ponowną przebudowę
    pub fn clear_mesh_cache(&mut self) {
        self.chunk_slots.clear();
        self.free_slots = (0..Self::MAX_CHUNK_SLOTS).rev().collect();
        self.slot_index_counts = [0; Self::MAX_CHUNK_SLOTS];
        self.chunk_spawn_times.clear();
    }

    /// Zwraca statystyki wydajności i pamięci (użyte sloty chunków, max slotów, bajty CDRAM, bajty RAM)
    pub fn get_perf_stats(&self) -> (usize, usize, usize, usize) {
        let used_slots = self.chunk_slots.len();
        let cdram = 3 * (VITA_WIDTH * VITA_HEIGHT * 4) as usize + (VITA_WIDTH * VITA_HEIGHT * 4) as usize; // Framebuffers + Depth
        let ram = Self::VB_SIZE + Self::IB_SIZE + 256 * 256 * 4 + Self::MAX_ENTITY_VERTS * core::mem::size_of::<GxmVertex>() + Self::MAX_ENTITY_INDICES * 2;
        (used_slots, Self::MAX_CHUNK_SLOTS, cdram, ram)
    }
}

impl Drop for GxmRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = sceGxmFinish(self.context);
            if !self.vert_prog.is_null() && !self.patcher.is_null() {
                let _ = sceGxmShaderPatcherReleaseVertexProgram(self.patcher, self.vert_prog);
            }
            if !self.frag_prog.is_null() && !self.patcher.is_null() {
                let _ = sceGxmShaderPatcherReleaseFragmentProgram(self.patcher, self.frag_prog);
            }
            if !self.clear_vert_prog.is_null() && !self.patcher.is_null() {
                let _ = sceGxmShaderPatcherReleaseVertexProgram(self.patcher, self.clear_vert_prog);
            }
            if !self.clear_frag_prog.is_null() && !self.patcher.is_null() {
                let _ = sceGxmShaderPatcherReleaseFragmentProgram(self.patcher, self.clear_frag_prog);
            }
            for sync in self.display_sync {
                if !sync.is_null() {
                    let _ = sceGxmSyncObjectDestroy(sync);
                }
            }
            if !self.render_target.is_null() {
                let _ = sceGxmDestroyRenderTarget(self.render_target);
            }
            if !self.patcher.is_null() {
                let _ = sceGxmShaderPatcherDestroy(self.patcher);
            }
            if !self.context.is_null() {
                let _ = sceGxmDestroyContext(self.context);
            }
            if !self.host_mem.is_null() {
                free(self.host_mem);
            }
            let _ = sceGxmTerminate();
        }
    }
}
