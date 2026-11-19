//! Low-level `libgxm`, `SceDisplay`, and `SceSysmem` C ABI extern bindings.
//! Re-exports constants and C structs from `ffi_defs` exclusively for `platform::vita::gxm`.

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

pub(super) use super::ffi_defs::*;

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
    pub(super) fn sceGxmTextureInitLinearStrided(
        texture: *mut SceGxmTexture,
        data: *const core::ffi::c_void,
        format: u32,
        width: u32,
        height: u32,
        byteStride: u32,
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

