//! Low-level `libgxm`, `SceDisplay`, and `SceSysmem` FFI types, constants, and bindings.
//! Pure C ABI declarations used exclusively inside `platform::vita::gxm`.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

// ============================================================
// 1. FFI TYPES AND CONSTANTS FOR VITA GXM
// ============================================================

pub(super) type SceUID = i32;
pub(super) type SceSize = u32;
pub(super) type SceBool = i32;
pub(super) const SCE_OK: i32 = 0;

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
pub(super) const SCE_GXM_MINIMUM_CONTEXT_HOST_MEM_SIZE: usize = 2 * 1024;
pub(super) const SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE: usize = 16 * 1024 * 1024; // 16 MB standard VitaSDK parameter buffer
pub(super) const SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE: usize = 128 * 1024;
pub(super) const SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE: usize = 2 * 1024 * 1024;
pub(super) const SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE: usize = 512 * 1024;
pub(super) const SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE: usize = 16 * 1024;
pub(super) const SCE_GXM_DEFAULT_PATCHER_BUFFER_SIZE: usize = 64 * 1024;
pub(super) const SCE_GXM_DEFAULT_PATCHER_VERTEX_USSE_SIZE: usize = 64 * 1024;
pub(super) const SCE_GXM_DEFAULT_PATCHER_FRAGMENT_USSE_SIZE: usize = 64 * 1024;

// Formaty i flagi
pub(super) const SCE_GXM_COLOR_FORMAT_A8B8G8R8: u32 = 0x00000000;
pub(super) const SCE_GXM_COLOR_SURFACE_LINEAR: u32 = 0x00000000;
pub(super) const SCE_GXM_COLOR_SURFACE_SCALE_NONE: u32 = 0x00000000;
pub(super) const SCE_GXM_OUTPUT_REGISTER_SIZE_32BIT: u32 = 0x00000000;
pub(super) const SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4: u32 = 1;
pub(super) const SCE_GXM_DEPTH_STENCIL_FORMAT_DF32: u32 = 0x00044000;
pub(super) const SCE_GXM_DEPTH_STENCIL_SURFACE_LINEAR: u32 = 0x00000000;
pub(super) const SCE_GXM_DEPTH_STENCIL_SURFACE_TILED: u32 = 0x00011000; // VitaSDK enum value
pub(super) const SCE_GXM_MULTISAMPLE_NONE: u32 = 0;
pub(super) const SCE_GXM_COLOR_SURFACE_DITHER_ENABLED: u32 = 0x00000008;
pub(super) const SCE_GXM_PRIMITIVE_TRIANGLES: u32 = 0x00000000;

pub(super) const SCE_GXM_INDEX_FORMAT_U16: u32 = 0x00000000;
pub(super) const SCE_GXM_INDEX_SOURCE_INDEX_16BIT: u16 = 0;

pub(super) const SCE_GXM_MEMORY_ATTRIB_READ: u32 = 1;
pub(super) const SCE_GXM_MEMORY_ATTRIB_RW: u32 = 3;

pub(super) const SCE_GXM_CULL_NONE: u32 = 0;
pub(super) const SCE_GXM_CULL_CW: u32 = 1;
pub(super) const SCE_GXM_CULL_CCW: u32 = 2;
pub(super) const SCE_GXM_ATTRIBUTE_FORMAT_F32: u32 = 9;

pub(super) const SCE_GXM_DEPTH_FUNC_LESS_EQUAL: u32 = 0x00C00000;
pub(super) const SCE_GXM_DEPTH_FUNC_ALWAYS_PASS: u32 = 0x01C00000;

pub(super) const SCE_GXM_DEPTH_WRITE_ENABLED: u32 = 0x00000000;
pub(super) const SCE_GXM_DEPTH_WRITE_DISABLED: u32 = 0x00100000;
pub(super) const SCE_GXM_TWO_SIDED_DISABLED: u32 = 0x00000000;
pub(super) const SCE_GXM_TWO_SIDED_ENABLED: u32 = 0x00000800;
pub(super) const SCE_GXM_POLYGON_MODE_TRIANGLE_FILL: u32 = 0x00000000;
pub(super) const SCE_GXM_VIEWPORT_ENABLED: u32 = 0x00000000;

pub(super) const SCE_GXM_TEXTURE_BASE_FORMAT_U8U8U8U8: u32 = 0x0C000000;
pub(super) const SCE_GXM_TEXTURE_SWIZZLE4_ABGR: u32 = 0x00000000;
pub(super) const SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR: u32 = SCE_GXM_TEXTURE_BASE_FORMAT_U8U8U8U8 | SCE_GXM_TEXTURE_SWIZZLE4_ABGR;
pub(super) const SCE_GXM_TEXTURE_FILTER_POINT: u32 = 0;
pub(super) const SCE_GXM_TEXTURE_ADDR_REPEAT: u32 = 0;

pub(super) const SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW: u32 = 0x09408060;
pub(super) const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE: u32 = 0x0C208060;

pub(super) const VITA_WIDTH: u32 = 960;
pub(super) const VITA_HEIGHT: u32 = 544;
pub(super) const VITA_STRIDE: u32 = 960;
pub(super) const DISPLAY_BUFFER_COUNT: usize = 3;
pub(super) const MAX_PENDING_SWAPS: u32 = DISPLAY_BUFFER_COUNT as u32 - 1;

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
    pub(super) _private: [u32; 4], // 16 bytes (0x10)
}

#[repr(C)]
pub struct SceGxmColorSurface {
    pub(super) _private: [u32; 12], // 48 bytes (0x30)
}

#[repr(C)]
pub struct SceGxmDepthStencilSurface {
    pub(super) _private: [u32; 5], // 20 bytes (0x14)
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
pub(super) struct SceDisplayFrameBuf {
    pub(super) size: u32,
    pub(super) base: *mut core::ffi::c_void,
    pub(super) pitch: u32,
    pub(super) pixelformat: u32,
    pub(super) width: u32,
    pub(super) height: u32,
}

// ---- FFI do libgxm i libkernel ----
#[link(name = "SceGxm_stub")]
extern "C" {
    pub(super) fn sceGxmInitialize(params: *const SceGxmInitializeParams) -> i32;
    pub(super) fn sceGxmTerminate() -> i32;
    pub(super) fn sceGxmCreateContext(params: *const SceGxmContextParams, context: *mut *mut SceGxmContext) -> i32;
    pub(super) fn sceGxmDestroyContext(context: *mut SceGxmContext) -> i32;
    pub(super) fn sceGxmCreateRenderTarget(params: *const SceGxmRenderTargetParams, renderTarget: *mut *mut SceGxmRenderTarget) -> i32;
    pub(super) fn sceGxmDestroyRenderTarget(renderTarget: *mut SceGxmRenderTarget) -> i32;
    pub(super) fn sceGxmMapMemory(base: *mut core::ffi::c_void, size: u32, attr: u32) -> i32;
    pub(super) fn sceGxmUnmapMemory(base: *mut core::ffi::c_void) -> i32;
    pub(super) fn sceGxmMapVertexUsseMemory(base: *mut core::ffi::c_void, size: u32, offset: *mut u32) -> i32;
    pub(super) fn sceGxmUnmapVertexUsseMemory(base: *mut core::ffi::c_void) -> i32;
    pub(super) fn sceGxmMapFragmentUsseMemory(base: *mut core::ffi::c_void, size: u32, offset: *mut u32) -> i32;
    pub(super) fn sceGxmUnmapFragmentUsseMemory(base: *mut core::ffi::c_void) -> i32;

    pub(super) fn sceGxmColorSurfaceInit(
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

    pub(super) fn sceGxmDepthStencilSurfaceInit(
        surface: *mut SceGxmDepthStencilSurface,
        depthStencilFormat: u32,
        surfaceType: u32,
        strideInSamples: u32,
        depthData: *mut core::ffi::c_void,
        stencilData: *mut core::ffi::c_void,
    ) -> i32;

    pub(super) fn sceGxmBeginScene(
        context: *mut SceGxmContext,
        flags: u32,
        renderTarget: *const SceGxmRenderTarget,
        validRegion: *const core::ffi::c_void,
        vertexSyncObject: *mut SceGxmSyncObject,
        fragmentSyncObject: *mut SceGxmSyncObject,
        colorSurface: *const SceGxmColorSurface,
        depthStencil: *const SceGxmDepthStencilSurface,
    ) -> i32;

    pub(super) fn sceGxmEndScene(
        context: *mut SceGxmContext,
        vertexNotification: *const core::ffi::c_void,
        fragmentNotification: *const core::ffi::c_void,
    ) -> i32;

    pub(super) fn sceGxmPadHeartbeat(colorSurface: *const SceGxmColorSurface, syncObject: *mut SceGxmSyncObject) -> i32;
    pub(super) fn sceGxmDisplayQueueAddEntry(
        oldBuffer: *mut SceGxmSyncObject,
        newBuffer: *mut SceGxmSyncObject,
        callbackData: *const core::ffi::c_void,
    ) -> i32;

    pub(super) fn sceGxmSetVertexProgram(context: *mut SceGxmContext, vertexProgram: *const SceGxmVertexProgram);
    pub(super) fn sceGxmSetFragmentProgram(context: *mut SceGxmContext, fragmentProgram: *const SceGxmFragmentProgram);
    pub(super) fn sceGxmReserveVertexDefaultUniformBuffer(context: *mut SceGxmContext, uniformBuffer: *mut *mut core::ffi::c_void) -> i32;
    pub(super) fn sceGxmReserveFragmentDefaultUniformBuffer(context: *mut SceGxmContext, uniformBuffer: *mut *mut core::ffi::c_void) -> i32;
    pub(super) fn sceGxmSetUniformDataF(
        uniformBuffer: *mut core::ffi::c_void,
        parameter: *const SceGxmProgramParameter,
        componentOffset: u32,
        componentCount: u32,
        sourceData: *const f32,
    ) -> i32;
    pub(super) fn sceGxmSetVertexStream(context: *mut SceGxmContext, streamIndex: u32, streamData: *const core::ffi::c_void);
    pub(super) fn sceGxmDraw(
        context: *mut SceGxmContext,
        primType: u32,
        indexType: u32,
        indexData: *const core::ffi::c_void,
        indexCount: u32,
    ) -> i32;

    pub(super) fn sceGxmSetFrontDepthFunc(context: *mut SceGxmContext, depthFunc: u32);
    pub(super) fn sceGxmSetBackDepthFunc(context: *mut SceGxmContext, depthFunc: u32);
    pub(super) fn sceGxmSetFrontDepthWriteEnable(context: *mut SceGxmContext, enable: u32);
    pub(super) fn sceGxmSetBackDepthWriteEnable(context: *mut SceGxmContext, enable: u32);
    pub(super) fn sceGxmSetFrontPolygonMode(context: *mut SceGxmContext, mode: u32);
    pub(super) fn sceGxmSetBackPolygonMode(context: *mut SceGxmContext, mode: u32);
    pub(super) fn sceGxmSetTwoSidedEnable(context: *mut SceGxmContext, enable: u32);
    pub(super) fn sceGxmSetDefaultRegionClipAndViewport(context: *mut SceGxmContext, xMax: u32, yMax: u32);
    pub(super) fn sceGxmSetViewportEnable(context: *mut SceGxmContext, enable: u32);
    pub(super) fn sceGxmSetViewport(
        context: *mut SceGxmContext,
        xOffset: f32,
        xScale: f32,
        yOffset: f32,
        yScale: f32,
        zOffset: f32,
        zScale: f32,
    );
    pub(super) fn sceGxmSetCullMode(context: *mut SceGxmContext, mode: u32);
    pub(super) fn sceGxmFinish(context: *mut SceGxmContext);

    pub(super) fn sceGxmColorSurfaceSetDitherMode(surface: *mut SceGxmColorSurface, ditherMode: u32) -> i32;

    pub(super) fn sceGxmTextureInitLinear(
        texture: *mut SceGxmTexture,
        data: *const core::ffi::c_void,
        format: u32,
        width: u32,
        height: u32,
        mipCount: u32,
    ) -> i32;
    pub(super) fn sceGxmTextureSetMinFilter(texture: *mut SceGxmTexture, filter: u32) -> i32;
    pub(super) fn sceGxmTextureSetMagFilter(texture: *mut SceGxmTexture, filter: u32) -> i32;
    pub(super) fn sceGxmTextureSetUAddrMode(texture: *mut SceGxmTexture, mode: u32) -> i32;
    pub(super) fn sceGxmTextureSetVAddrMode(texture: *mut SceGxmTexture, mode: u32) -> i32;
    pub(super) fn sceGxmSetFragmentTexture(context: *mut SceGxmContext, textureIndex: u32, texture: *const SceGxmTexture) -> i32;

    pub(super) fn sceGxmShaderPatcherCreate(params: *const SceGxmShaderPatcherParams, patcher: *mut *mut SceGxmShaderPatcher) -> i32;
    pub(super) fn sceGxmShaderPatcherDestroy(patcher: *mut SceGxmShaderPatcher) -> i32;
    pub(super) fn sceGxmShaderPatcherRegisterProgram(
        patcher: *mut SceGxmShaderPatcher,
        programHeader: *const SceGxmProgram,
        programId: *mut SceGxmShaderPatcherId,
    ) -> i32;
    pub(super) fn sceGxmShaderPatcherUnregisterProgram(
        patcher: *mut SceGxmShaderPatcher,
        programId: SceGxmShaderPatcherId,
    ) -> i32;
    pub(super) fn sceGxmShaderPatcherReleaseVertexProgram(
        patcher: *mut SceGxmShaderPatcher,
        vertexProgram: *mut SceGxmVertexProgram,
    ) -> i32;
    pub(super) fn sceGxmShaderPatcherReleaseFragmentProgram(
        patcher: *mut SceGxmShaderPatcher,
        fragmentProgram: *mut SceGxmFragmentProgram,
    ) -> i32;
    pub(super) fn sceGxmShaderPatcherCreateVertexProgram(
        patcher: *mut SceGxmShaderPatcher,
        programId: SceGxmShaderPatcherId,
        attributes: *const SceGxmVertexAttribute,
        attributeCount: u32,
        streams: *const SceGxmVertexStream,
        streamCount: u32,
        vertexProgram: *mut *mut SceGxmVertexProgram,
    ) -> i32;
    pub(super) fn sceGxmShaderPatcherCreateFragmentProgram(
        patcher: *mut SceGxmShaderPatcher,
        programId: SceGxmShaderPatcherId,
        outputFormat: u32,
        multisampleMode: u32,
        blendInfo: *const core::ffi::c_void,
        vertexProgramHeader: *const SceGxmProgram,
        fragmentProgram: *mut *mut SceGxmFragmentProgram,
    ) -> i32;

    pub(super) fn sceGxmProgramCheck(program: *const SceGxmProgram) -> i32;
    pub(super) fn sceGxmProgramFindParameterByName(program: *const SceGxmProgram, name: *const u8) -> *const SceGxmProgramParameter;
    pub(super) fn sceGxmProgramParameterGetResourceIndex(parameter: *const SceGxmProgramParameter) -> u32;

    pub(super) fn sceGxmSyncObjectCreate(syncObject: *mut *mut SceGxmSyncObject) -> i32;
    pub(super) fn sceGxmSyncObjectDestroy(syncObject: *mut SceGxmSyncObject) -> i32;
}

#[link(name = "SceSysmem_stub")]
extern "C" {
    pub(super) fn sceKernelAllocMemBlock(name: *const u8, type_: u32, size: u32, opt: *const core::ffi::c_void) -> SceUID;
    pub(super) fn sceKernelFreeMemBlock(uid: SceUID) -> i32;
    pub(super) fn sceKernelGetMemBlockBase(uid: SceUID, base: *mut *mut core::ffi::c_void) -> i32;
}

#[link(name = "SceDisplay_stub")]
extern "C" {
    pub(super) fn sceDisplaySetFrameBuf(pParam: *const SceDisplayFrameBuf, sync: i32) -> i32;
}

extern "C" {
    pub(super) fn malloc(size: usize) -> *mut core::ffi::c_void;
    pub(super) fn free(ptr: *mut core::ffi::c_void);
}

