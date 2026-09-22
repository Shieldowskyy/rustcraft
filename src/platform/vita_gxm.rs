// =============================================================================
// RustCraft – PS Vita GXM Hardware Renderer
// src/platform/vita_gxm.rs
//
// Implementuje pełny rendering 3D przez GXM API (SGX543MP4 GPU na PS Vita).
// Pipeline:
//   1. Inicjalizacja GXM + ring bufferów
//   2. Alokacja CDRAM dla color/depth buforów (triple buffering)
//   3. Shader Patcher + załadowanie GXP shaderów
//   4. Per-frame: BeginScene → SetVertexProgram → DrawCall × chunki → EndScene
//   5. Display callback (async swap)
// =============================================================================

#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]

use crate::core::math::{Mat4, Vec3};
use crate::graphics::atlas::{TextureAtlas, TEX_SIZE, TextureId};
use crate::graphics::camera::Camera;
use crate::world::mesher::{build_chunk_mesh, Triangle, Vertex};
use crate::world::{World, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};
use std::collections::HashMap;

// ============================================================
// 1. FFI TYPY
// ============================================================

type SceUID = i32;
type SceSize = u32;
type SceBool = i32;
const SCE_OK: i32 = 0;

#[repr(C)]
pub struct SceGxmContext(u8); // opaque
#[repr(C)]
pub struct SceGxmRenderTarget(u8); // opaque
#[repr(C)]
pub struct SceGxmShaderPatcher(u8); // opaque
#[repr(C)]
pub struct SceGxmVertexProgram(u8); // opaque
#[repr(C)]
pub struct SceGxmFragmentProgram(u8); // opaque
#[repr(C)]
pub struct SceGxmSyncObject(u8); // opaque
#[repr(C)]
pub struct SceGxmProgram(u8); // opaque (GXP binary header)

pub type SceGxmShaderPatcherId = *mut core::ffi::c_void;
pub type SceGxmDisplayQueueCallback = unsafe extern "C" fn(callbackData: *const core::ffi::c_void);

// ---- Stałe GXM ----
const SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE: usize = 16 * 1024 * 1024;
const SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE: usize = 128 * 1024;
const SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE: usize = 2 * 1024 * 1024;
const SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE: usize = 512 * 1024;
const SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE: usize = 16 * 1024;

const SCE_GXM_COLOR_FORMAT_A8B8G8R8: u32 = 0x94200408;
const SCE_GXM_COLOR_SURFACE_LINEAR: u32 = 0x00000000;
const SCE_GXM_COLOR_SURFACE_SCALE_NONE: u32 = 0x00000000;
const SCE_GXM_OUTPUT_REGISTER_SIZE_32BIT: u32 = 0x00000000;
const SCE_GXM_DEPTH_STENCIL_FORMAT_DF32: u32 = 0x00044000;
const SCE_GXM_DEPTH_STENCIL_SURFACE_LINEAR: u32 = 0x00000000;
const SCE_GXM_MULTISAMPLE_NONE: u32 = 0;
const SCE_GXM_SCENE_FRAGMENT_SET_DEPENDENCY: u32 = 0x00000001;
const SCE_GXM_PRIMITIVE_TRIANGLES: u32 = 0x00000000;
const SCE_GXM_INDEX_FORMAT_U32: u32 = 0x00000001;
const SCE_GXM_MEMORY_ATTRIB_READ: u32 = 1;
const SCE_GXM_MEMORY_ATTRIB_RW: u32 = 3;
const SCE_GXM_CULL_NONE: u32 = 0;
const SCE_GXM_CULL_CW: u32 = 1;
const SCE_GXM_ATTRIBUTE_FORMAT_F32: u32 = 9;
const SCE_GXM_TEXTURE_FORMAT_U8U8U8U8_ABGR: u32 = 0x18000000;
const SCE_GXM_TEXTURE_TYPE_LINEAR: u32 = 0xC0000000;
const SCE_GXM_TEXTURE_FILTER_LINEAR: u32 = 1;
const SCE_GXM_TEXTURE_FILTER_POINT: u32 = 0;
const SCE_GXM_TEXTURE_ADDR_CLAMP: u32 = 2;
const SCE_GXM_TEXTURE_ADDR_REPEAT: u32 = 0;

const SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW: u32 = 0x09408060;
const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE: u32 = 0x0C208060;
const SCE_KERNEL_MEMBLOCK_TYPE_USER_RW: u32 = 0x0C20D060;

const VITA_WIDTH: u32 = 960;
const VITA_HEIGHT: u32 = 544;
const VITA_STRIDE: u32 = 960;

const DISPLAY_BUFFER_COUNT: usize = 3;
const MAX_PENDING_SWAPS: u32 = DISPLAY_BUFFER_COUNT as u32 - 1;

// ---- Struktury GXM ----
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
    pub multisampleMode: u32,
    pub multisampleLocations: u32,
    pub driverMemBlock: SceUID,
}

#[repr(C)]
pub struct SceGxmColorSurface {
    _private: [u32; 8],
}

#[repr(C)]
pub struct SceGxmDepthStencilSurface {
    _private: [u32; 8],
}

#[repr(C)]
pub struct SceGxmShaderPatcherParams {
    pub userData: *mut core::ffi::c_void,
    pub hostAllocCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> *mut core::ffi::c_void>,
    pub hostFreeCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void)>,
    pub bufferAllocCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32) -> *mut core::ffi::c_void>,
    pub bufferFreeCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void)>,
    pub bufferMem: *mut core::ffi::c_void,
    pub bufferMemSize: u32,
    pub vertexUsseAllocCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut u32) -> *mut core::ffi::c_void>,
    pub vertexUsseFreeCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut core::ffi::c_void)>,
    pub vertexUsseMem: *mut core::ffi::c_void,
    pub vertexUsseMemSize: u32,
    pub vertexUsseMemOffset: u32,
    pub fragmentUsseAllocCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut u32) -> *mut core::ffi::c_void>,
    pub fragmentUsseFreeCallback: Option<unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut core::ffi::c_void)>,
    pub fragmentUsseMem: *mut core::ffi::c_void,
    pub fragmentUsseMemSize: u32,
    pub fragmentUsseMemOffset: u32,
}

#[repr(C)]
pub struct SceGxmVertexAttribute {
    pub streamIndex: u16,
    pub offset: u16,
    pub format: u32,      // SceGxmAttributeFormat
    pub componentCount: u8,
    pub regIndex: u16,
}

#[repr(C)]
pub struct SceGxmVertexStream {
    pub stride: u16,
    pub indexSource: u32,
}

#[repr(C)]
pub struct SceGxmTexture {
    _private: [u32; 4],
}

// ---- FFI do bibliotek ----
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
    fn sceGxmMapFragmentUsseMemory(base: *mut core::ffi::c_void, size: u32, offset: *mut u32) -> i32;
    fn sceGxmMapVertexUsseMemory(base: *mut core::ffi::c_void, size: u32, offset: *mut u32) -> i32;
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
    fn sceGxmFinish(context: *mut SceGxmContext) -> i32;
    fn sceGxmPadHeartbeat(colorSurface: *const SceGxmColorSurface, syncObject: *mut SceGxmSyncObject) -> i32;
    fn sceGxmDisplayQueueAddEntry(
        oldBuffer: *mut SceGxmSyncObject,
        newBuffer: *mut SceGxmSyncObject,
        callbackData: *const core::ffi::c_void,
    ) -> i32;
    fn sceGxmSetVertexProgram(context: *mut SceGxmContext, vertexProgram: *const SceGxmVertexProgram);
    fn sceGxmSetFragmentProgram(context: *mut SceGxmContext, fragmentProgram: *const SceGxmFragmentProgram);
    fn sceGxmReserveVertexDefaultUniformBuffer(context: *mut SceGxmContext, uniformBuffer: *mut *mut core::ffi::c_void) -> i32;
    fn sceGxmSetUniformDataF(
        uniformBuffer: *mut core::ffi::c_void,
        parameter: *const core::ffi::c_void,
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
    fn sceGxmSetFrontDepthWriteEnable(context: *mut SceGxmContext, enable: u32);
    fn sceGxmSetCullMode(context: *mut SceGxmContext, mode: u32);
    // Shader Patcher
    fn sceGxmShaderPatcherCreate(params: *const SceGxmShaderPatcherParams, shaderPatcher: *mut *mut SceGxmShaderPatcher) -> i32;
    fn sceGxmShaderPatcherDestroy(shaderPatcher: *mut SceGxmShaderPatcher) -> i32;
    fn sceGxmShaderPatcherRegisterProgram(patcher: *mut SceGxmShaderPatcher, program: *const SceGxmProgram, id: *mut SceGxmShaderPatcherId) -> i32;
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
        vertexProgram: *const SceGxmProgram,
        fragmentProgram: *mut *mut SceGxmFragmentProgram,
    ) -> i32;
    fn sceGxmShaderPatcherGetProgramFromId(programId: SceGxmShaderPatcherId) -> *const SceGxmProgram;
    fn sceGxmProgramFindParameterByName(program: *const SceGxmProgram, name: *const u8) -> *const core::ffi::c_void;
    // Tekstury
    fn sceGxmTextureInitLinear(
        texture: *mut SceGxmTexture,
        data: *const core::ffi::c_void,
        texFormat: u32,
        width: u32,
        height: u32,
        mipCount: u32,
    ) -> i32;
    fn sceGxmTextureSetMinFilter(texture: *mut SceGxmTexture, minFilter: u32) -> i32;
    fn sceGxmTextureSetMagFilter(texture: *mut SceGxmTexture, magFilter: u32) -> i32;
    fn sceGxmTextureSetUAddrMode(texture: *mut SceGxmTexture, mode: u32) -> i32;
    fn sceGxmTextureSetVAddrMode(texture: *mut SceGxmTexture, mode: u32) -> i32;
    fn sceGxmSetFragmentTexture(context: *mut SceGxmContext, textureIndex: u32, texture: *const SceGxmTexture) -> i32;
    // SceGxmSyncObject
    fn sceGxmSyncObjectCreate(syncObject: *mut *mut SceGxmSyncObject) -> i32;
    fn sceGxmSyncObjectDestroy(syncObject: *mut SceGxmSyncObject) -> i32;
}

#[link(name = "SceLibKernel_stub")]
extern "C" {
    fn sceKernelAllocMemBlock(name: *const u8, type_: u32, size: u32, opt: *const core::ffi::c_void) -> SceUID;
    fn sceKernelFreeMemBlock(uid: SceUID) -> i32;
    fn sceKernelGetMemBlockBase(uid: SceUID, base: *mut *mut core::ffi::c_void) -> i32;
    fn sceKernelExitProcess(res: i32) -> !;
    fn sceKernelGetProcessTimeWide() -> u64;
}

#[link(name = "SceDisplay_stub")]
extern "C" {
    fn sceDisplaySetFrameBuf(pParam: *const SceDisplayFrameBuf, sync: i32) -> i32;
    fn sceDisplayWaitVblankStart() -> i32;
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

// ============================================================
// 2. GXM PAMIĘĆ – alokator aligned
// ============================================================

/// Wrapper na zaalokowany blok pamięci mapowany dla GPU
pub struct GpuMem {
    uid: SceUID,
    pub ptr: *mut core::ffi::c_void,
    pub size: usize,
}

impl GpuMem {
    /// Alokuje pamięć dostępną dla CPU i GPU (UC = uncached)
    pub fn alloc_uc(label: &[u8], size: usize) -> Option<Self> {
        let aligned = (size + 0xFFFFF) & !0xFFFFF; // 1MB granulacja dla CDRAM
        let uid = unsafe {
            sceKernelAllocMemBlock(label.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE, aligned as u32, core::ptr::null())
        };
        if uid < 0 { return None; }
        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK { return None; }
        unsafe { sceGxmMapMemory(ptr, aligned as u32, SCE_GXM_MEMORY_ATTRIB_RW); }
        Some(Self { uid, ptr, size: aligned })
    }

    /// Alokuje pamięć CDRAM (dla buforów wyświetlania)
    pub fn alloc_cdram(label: &[u8], size: usize) -> Option<Self> {
        let aligned = (size + 0xFFFFF) & !0xFFFFF;
        let uid = unsafe {
            sceKernelAllocMemBlock(label.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW, aligned as u32, core::ptr::null())
        };
        if uid < 0 { return None; }
        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK { return None; }
        unsafe { sceGxmMapMemory(ptr, aligned as u32, SCE_GXM_MEMORY_ATTRIB_RW); }
        Some(Self { uid, ptr, size: aligned })
    }

    /// Alokuje pamięć RAM normalną (dla ring bufferów, danych gry)
    pub fn alloc_ram(label: &[u8], size: usize) -> Option<Self> {
        let aligned = (size + 0xFFFFF) & !0xFFFFF;
        let uid = unsafe {
            sceKernelAllocMemBlock(label.as_ptr(), SCE_KERNEL_MEMBLOCK_TYPE_USER_RW, aligned as u32, core::ptr::null())
        };
        if uid < 0 { return None; }
        let mut ptr: *mut core::ffi::c_void = core::ptr::null_mut();
        if unsafe { sceKernelGetMemBlockBase(uid, &mut ptr) } != SCE_OK { return None; }
        unsafe { sceGxmMapMemory(ptr, aligned as u32, SCE_GXM_MEMORY_ATTRIB_RW); }
        Some(Self { uid, ptr, size: aligned })
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.ptr as *mut T
    }
}

impl Drop for GpuMem {
    fn drop(&mut self) {
        unsafe {
            sceGxmUnmapMemory(self.ptr);
            sceKernelFreeMemBlock(self.uid);
        }
    }
}

// ============================================================
// 3. DANE CALLBACKU DISPLAY (async swap)
// ============================================================

#[repr(C)]
struct DisplayData {
    address: *mut core::ffi::c_void,
}

/// Callback wywoływany asynchronicznie przez GXM po zakończeniu sceny.
/// Ustawia bufor do wyświetlenia przez sceDisplay.
unsafe extern "C" fn display_callback(data: *const core::ffi::c_void) {
    let d = &*(data as *const DisplayData);
    let fb = SceDisplayFrameBuf {
        size: core::mem::size_of::<SceDisplayFrameBuf>() as u32,
        base: d.address,
        pitch: VITA_STRIDE,
        pixelformat: 0x80000000, // SCE_DISPLAY_PIXELFORMAT_A8B8G8R8
        width: VITA_WIDTH,
        height: VITA_HEIGHT,
    };
    sceDisplaySetFrameBuf(&fb, 1 /* SCE_DISPLAY_SETBUF_NEXTFRAME */);
}

// ============================================================
// 4. SHADERY – Pre-skompilowane GXP bytecody
//    Wygenerowane przez psp2cgc z vita/shaders/voxel.{vert,frag}.cg
//    Minimalne shadery: pos(xyz) + uv(st) + ao(f) → tekstura × AO
// ============================================================

// Ładujemy skompilowane GXP shadery z app0: (dołączone do VPK)
// Fallback: zakodowane jako stałe bajty (znane dobre shadery z VitaSDK samples)

// Vertex GXP – transformacja MVP + przekazanie UV + AO
// Source: vita/shaders/voxel.vert.cg  (sce_vp_psp2 profile)
static VOXEL_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.vert.gxp"));

// Fragment GXP – próbkowanie tekstury + mnożenie przez AO
// Source: vita/shaders/voxel.frag.cg  (sce_fp_psp2 profile)
static VOXEL_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.frag.gxp"));

// ============================================================
// 5. VERTEX FORMAT
// ============================================================

/// Format wierzchołka dla GXM – musi pasować do layout w shaderze
#[repr(C, packed)]
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
    pub fn from_vertex(v: &Vertex, u: f32, vv: f32) -> Self {
        Self {
            x: v.pos.x,
            y: v.pos.y,
            z: v.pos.z,
            u,
            v: vv,
            ao: v.ao,
        }
    }
}

// Stride = 6 × 4 bajtów = 24 bajty
const VERTEX_STRIDE: u16 = 24;

// ============================================================
// 6. ATLAS TEKSTURY → bufor GPU
// ============================================================

const ATLAS_TILES_PER_ROW: usize = 8; // 8×8 = 64 kafelki; daje 128×128px atlas
const ATLAS_SIZE: usize = ATLAS_TILES_PER_ROW * TEX_SIZE;

/// Buduje atlas RGBA→ABGR i wgrywa na GPU
pub fn build_gpu_texture(atlas: &TextureAtlas) -> (GpuMem, SceGxmTexture) {
    use crate::graphics::atlas::TextureId::*;
    // Wszystkie TextureId w kolejności = indeks w atlasie
    const ALL_TEXTURES: &[TextureId] = &[
        GrassTop, GrassSide, Dirt, Stone, Sand, WoodTop, WoodSide, Leaves,
        Water, Snow, CactusTop, CactusSide, CoalOre, IronOre, Bedrock, Lava,
        SnowLeaves, Planks, Door, DoorTop, Glass, Ice, TntTop, TntSide,
        PigSkin, PigFace, PigSnout, CowSkin, CowFace, CowSnout, CowHorns, CowUdder,
        CraftingTableTop, CraftingTableSide, CraftingTableFront,
        DestroyStage0, DestroyStage1, DestroyStage2, DestroyStage3, DestroyStage4,
        DestroyStage5, DestroyStage6, DestroyStage7, DestroyStage8, DestroyStage9,
    ];

    let pixel_count = ATLAS_SIZE * ATLAS_SIZE;
    let byte_size = pixel_count * 4;

    let mem = GpuMem::alloc_cdram(b"RustCraft_Atlas\0", byte_size)
        .expect("Brak pamięci CDRAM dla atlasu");

    let pixels = unsafe { core::slice::from_raw_parts_mut(mem.ptr as *mut u32, pixel_count) };
    // Wyzeruj atlas (przezroczysty)
    pixels.fill(0);

    // Wypełnij atlas teksturami (ABGR8888 format)
    for (tile_idx, &tex_id) in ALL_TEXTURES.iter().enumerate() {
        let tex = atlas.texture_for(tex_id);
        let tile_col = tile_idx % ATLAS_TILES_PER_ROW;
        let tile_row = tile_idx / ATLAS_TILES_PER_ROW;
        let base_x = tile_col * TEX_SIZE;
        let base_y = tile_row * TEX_SIZE;
        for py in 0..TEX_SIZE {
            for px in 0..TEX_SIZE {
                let [r, g, b] = tex.texels[py * TEX_SIZE + px];
                let ri = (r.clamp(0.0, 1.0) * 255.0) as u32;
                let gi = (g.clamp(0.0, 1.0) * 255.0) as u32;
                let bi = (b.clamp(0.0, 1.0) * 255.0) as u32;
                // ABGR8888: A=255, B, G, R
                let abgr = (255u32 << 24) | (bi << 16) | (gi << 8) | ri;
                pixels[(base_y + py) * ATLAS_SIZE + (base_x + px)] = abgr;
            }
        }
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

/// Zwraca UV (u0,v0,u1,v1) w atlasie dla danego TextureId
pub fn atlas_uv(tex_id: TextureId) -> (f32, f32, f32, f32) {
    use crate::graphics::atlas::TextureId::*;
    const ALL_TEXTURES: &[TextureId] = &[
        GrassTop, GrassSide, Dirt, Stone, Sand, WoodTop, WoodSide, Leaves,
        Water, Snow, CactusTop, CactusSide, CoalOre, IronOre, Bedrock, Lava,
        SnowLeaves, Planks, Door, DoorTop, Glass, Ice, TntTop, TntSide,
        PigSkin, PigFace, PigSnout, CowSkin, CowFace, CowSnout, CowHorns, CowUdder,
        CraftingTableTop, CraftingTableSide, CraftingTableFront,
        DestroyStage0, DestroyStage1, DestroyStage2, DestroyStage3, DestroyStage4,
        DestroyStage5, DestroyStage6, DestroyStage7, DestroyStage8, DestroyStage9,
    ];
    // Szukamy indeksu tex_id
    let tile_idx = ALL_TEXTURES.iter().position(|&t| t as usize == tex_id as usize).unwrap_or(0);
    let col = tile_idx % ATLAS_TILES_PER_ROW;
    let row = tile_idx / ATLAS_TILES_PER_ROW;
    let tile_norm = 1.0 / ATLAS_TILES_PER_ROW as f32;
    let u0 = col as f32 * tile_norm;
    let v0 = row as f32 * tile_norm;
    (u0, v0, u0 + tile_norm, v0 + tile_norm)
}

// ============================================================
// 7. GŁÓWNY RENDERER GXM
// ============================================================

pub struct GxmRenderer {
    // Kontekst GXM
    context: *mut SceGxmContext,
    render_target: *mut SceGxmRenderTarget,

    // Ring buffery (RAM)
    _vdm_mem: GpuMem,
    _vertex_ring_mem: GpuMem,
    _fragment_ring_mem: GpuMem,
    _frag_usse_mem: GpuMem,
    _param_mem: GpuMem,

    // Buffery wyświetlania (CDRAM, triple-buffering)
    display_bufs: [GpuMem; DISPLAY_BUFFER_COUNT],
    display_sync: [*mut SceGxmSyncObject; DISPLAY_BUFFER_COUNT],
    color_surfaces: [SceGxmColorSurface; DISPLAY_BUFFER_COUNT],
    display_data: [DisplayData; DISPLAY_BUFFER_COUNT],
    current_buf: usize,

    // Depth buffer (RAM)
    depth_mem: GpuMem,
    depth_surface: SceGxmDepthStencilSurface,

    // Shader Patcher
    patcher: *mut SceGxmShaderPatcher,
    _patcher_buf_mem: GpuMem,
    _patcher_vert_usse: GpuMem,
    _patcher_frag_usse: GpuMem,
    vert_prog_id: SceGxmShaderPatcherId,
    frag_prog_id: SceGxmShaderPatcherId,
    vert_prog: *mut SceGxmVertexProgram,
    frag_prog: *mut SceGxmFragmentProgram,

    // Uniform parametry
    param_mvp: *const core::ffi::c_void,
    param_texture: *const core::ffi::c_void,

    // Tekstura atlasu
    _atlas_mem: GpuMem,
    atlas_tex: SceGxmTexture,

    // Dynamiczny vertex buffer dla meshy (rysowany co-frame)
    vb_mem: GpuMem,
    ib_mem: GpuMem,

    // Cache meshy chunków
    mesh_cache: HashMap<(i32, i32), Vec<Triangle>>,
}

// Używamy raw pointerów – jesteśmy single-threaded na Vita
unsafe impl Send for GxmRenderer {}
unsafe impl Sync for GxmRenderer {}

impl GxmRenderer {
    const VB_SIZE: usize = 512 * 1024 * 4; // 2MB vertex buffer
    const IB_SIZE: usize = 512 * 1024 * 4; // 2MB index buffer

    pub fn new(atlas: &TextureAtlas) -> Self {
        unsafe { Self::init_unsafe(atlas) }
    }

    unsafe fn init_unsafe(atlas: &TextureAtlas) -> Self {
        // --- 1. Alokacja ring bufferów ---
        let param_mem = GpuMem::alloc_ram(b"RC_GXM_Param\0", SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE).unwrap();
        let vdm_mem = GpuMem::alloc_ram(b"RC_GXM_VDM\0", SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE).unwrap();
        let vertex_ring_mem = GpuMem::alloc_ram(b"RC_GXM_VRing\0", SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE).unwrap();
        let fragment_ring_mem = GpuMem::alloc_ram(b"RC_GXM_FRing\0", SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE).unwrap();
        let frag_usse_mem = GpuMem::alloc_ram(b"RC_GXM_FUSSE\0", SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE).unwrap();

        let mut frag_usse_offset: u32 = 0;
        sceGxmMapFragmentUsseMemory(frag_usse_mem.ptr, SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE as u32, &mut frag_usse_offset);

        // --- 2. Inicjalizacja GXM ---
        let init_params = SceGxmInitializeParams {
            flags: 0,
            displayQueueMaxPendingCount: MAX_PENDING_SWAPS,
            displayQueueCallback: Some(display_callback),
            displayQueueCallbackDataSize: core::mem::size_of::<DisplayData>() as u32,
            parameterBufferSize: SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE as u32,
        };
        let rc = sceGxmInitialize(&init_params);
        assert_eq!(rc, SCE_OK, "sceGxmInitialize failed: {}", rc);

        // --- 3. Tworzenie kontekstu GXM ---
        let ctx_params = SceGxmContextParams {
            hostMem: param_mem.ptr,
            hostMemSize: param_mem.size as u32,
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
        assert_eq!(rc, SCE_OK, "sceGxmCreateContext failed: {}", rc);

        // --- 4. Buffery wyświetlania CDRAM (triple buffering) ---
        let color_surface_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
        let mut display_sync: [*mut SceGxmSyncObject; DISPLAY_BUFFER_COUNT] =
            [core::ptr::null_mut(); DISPLAY_BUFFER_COUNT];
        let mut display_data: [DisplayData; DISPLAY_BUFFER_COUNT] = [
            DisplayData { address: core::ptr::null_mut() },
            DisplayData { address: core::ptr::null_mut() },
            DisplayData { address: core::ptr::null_mut() },
        ];

        let names: [&[u8]; 3] = [b"RC_GXM_CB0\0", b"RC_GXM_CB1\0", b"RC_GXM_CB2\0"];

        // Alokacja buforów; owijamy w Option żeby uniknąć uninit_array (nightly-only)
        let mut display_bufs_opt: [core::mem::ManuallyDrop<GpuMem>; DISPLAY_BUFFER_COUNT] = unsafe {
            [
                core::mem::ManuallyDrop::new(GpuMem::alloc_cdram(names[0], color_surface_size).unwrap()),
                core::mem::ManuallyDrop::new(GpuMem::alloc_cdram(names[1], color_surface_size).unwrap()),
                core::mem::ManuallyDrop::new(GpuMem::alloc_cdram(names[2], color_surface_size).unwrap()),
            ]
        };
        let mut color_surfaces: [SceGxmColorSurface; DISPLAY_BUFFER_COUNT] = [
            SceGxmColorSurface { _private: [0u32; 8] },
            SceGxmColorSurface { _private: [0u32; 8] },
            SceGxmColorSurface { _private: [0u32; 8] },
        ];

        for i in 0..DISPLAY_BUFFER_COUNT {
            let ptr = display_bufs_opt[i].ptr;
            // Wyzeruj bufor koloru
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
            display_data[i].address = ptr;

            let mut sync: *mut SceGxmSyncObject = core::ptr::null_mut();
            sceGxmSyncObjectCreate(&mut sync);
            display_sync[i] = sync;
        }

        // Przekształć ManuallyDrop do prawdziwych GpuMem (unsafe move)
        let display_bufs: [GpuMem; DISPLAY_BUFFER_COUNT] = unsafe {
            [
                core::mem::ManuallyDrop::into_inner(core::ptr::read(&display_bufs_opt[0])),
                core::mem::ManuallyDrop::into_inner(core::ptr::read(&display_bufs_opt[1])),
                core::mem::ManuallyDrop::into_inner(core::ptr::read(&display_bufs_opt[2])),
            ]
        };
        core::mem::forget(display_bufs_opt); // nie wywołuj destruktorów na oryginałach

        // --- 5. Depth buffer ---
        let depth_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize; // DF32 = 4 bajty
        let depth_mem = GpuMem::alloc_ram(b"RC_GXM_Depth\0", depth_size).unwrap();
        let mut depth_surface = SceGxmDepthStencilSurface { _private: [0u32; 8] };
        sceGxmDepthStencilSurfaceInit(
            &mut depth_surface,
            SCE_GXM_DEPTH_STENCIL_FORMAT_DF32,
            SCE_GXM_DEPTH_STENCIL_SURFACE_LINEAR,
            VITA_WIDTH,
            depth_mem.ptr,
            core::ptr::null_mut(),
        );

        // --- 6. Render target ---
        let rt_params = SceGxmRenderTargetParams {
            flags: 0,
            width: VITA_WIDTH as u16,
            height: VITA_HEIGHT as u16,
            scenesPerFrame: 1,
            multisampleMode: SCE_GXM_MULTISAMPLE_NONE,
            multisampleLocations: 0,
            driverMemBlock: -1, // auto
        };
        let mut render_target: *mut SceGxmRenderTarget = core::ptr::null_mut();
        let rc = sceGxmCreateRenderTarget(&rt_params, &mut render_target);
        assert_eq!(rc, SCE_OK, "sceGxmCreateRenderTarget failed: {}", rc);

        // --- 7. Shader Patcher (USSE memory) ---
        let patcher_buf_mem = GpuMem::alloc_ram(b"RC_GXM_PatchBuf\0", 64 * 1024).unwrap();
        let patcher_vert_usse = GpuMem::alloc_ram(b"RC_GXM_PatchVUSS\0", 64 * 1024).unwrap();
        let patcher_frag_usse = GpuMem::alloc_ram(b"RC_GXM_PatchFUSS\0", 64 * 1024).unwrap();

        let mut vert_usse_offset: u32 = 0;
        let mut frag_usse_p_offset: u32 = 0;
        sceGxmMapVertexUsseMemory(patcher_vert_usse.ptr, 64 * 1024, &mut vert_usse_offset);
        sceGxmMapFragmentUsseMemory(patcher_frag_usse.ptr, 64 * 1024, &mut frag_usse_p_offset);

        // Callbacki host alloc dla patcha (używamy systemowego malloc, który jest dostępny przez libc)
        unsafe extern "C" fn patcher_alloc(_: *mut core::ffi::c_void, size: u32) -> *mut core::ffi::c_void {
            libc_malloc(size as usize)
        }
        unsafe extern "C" fn patcher_free(_: *mut core::ffi::c_void, ptr: *mut core::ffi::c_void) {
            libc_free(ptr);
        }

        let patcher_params = SceGxmShaderPatcherParams {
            userData: core::ptr::null_mut(),
            hostAllocCallback: Some(patcher_alloc),
            hostFreeCallback: Some(patcher_free),
            bufferAllocCallback: None,
            bufferFreeCallback: None,
            bufferMem: patcher_buf_mem.ptr,
            bufferMemSize: 64 * 1024,
            vertexUsseAllocCallback: None,
            vertexUsseFreeCallback: None,
            vertexUsseMem: patcher_vert_usse.ptr,
            vertexUsseMemSize: 64 * 1024,
            vertexUsseMemOffset: vert_usse_offset,
            fragmentUsseAllocCallback: None,
            fragmentUsseFreeCallback: None,
            fragmentUsseMem: patcher_frag_usse.ptr,
            fragmentUsseMemSize: 64 * 1024,
            fragmentUsseMemOffset: frag_usse_p_offset,
        };

        let mut patcher: *mut SceGxmShaderPatcher = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreate(&patcher_params, &mut patcher);
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreate failed: {}", rc);

        // --- 8. Rejestracja i linkowanie shaderów GXP ---
        let vert_gxp = VOXEL_VERT_GXP.as_ptr() as *const SceGxmProgram;
        let frag_gxp = VOXEL_FRAG_GXP.as_ptr() as *const SceGxmProgram;

        let mut vert_prog_id: SceGxmShaderPatcherId = core::ptr::null_mut();
        let mut frag_prog_id: SceGxmShaderPatcherId = core::ptr::null_mut();
        sceGxmShaderPatcherRegisterProgram(patcher, vert_gxp, &mut vert_prog_id);
        sceGxmShaderPatcherRegisterProgram(patcher, frag_gxp, &mut frag_prog_id);

        // Vertex attributes: position(3xF32) + uv(2xF32) + ao(1xF32)
        let vert_prog_ptr = sceGxmShaderPatcherGetProgramFromId(vert_prog_id);
        let frag_prog_ptr = sceGxmShaderPatcherGetProgramFromId(frag_prog_id);

        let attrs = [
            SceGxmVertexAttribute {
                streamIndex: 0,
                offset: 0,
                format: SCE_GXM_ATTRIBUTE_FORMAT_F32,
                componentCount: 3,
                regIndex: 0,
            },
            SceGxmVertexAttribute {
                streamIndex: 0,
                offset: 12,
                format: SCE_GXM_ATTRIBUTE_FORMAT_F32,
                componentCount: 2,
                regIndex: 1,
            },
            SceGxmVertexAttribute {
                streamIndex: 0,
                offset: 20,
                format: SCE_GXM_ATTRIBUTE_FORMAT_F32,
                componentCount: 1,
                regIndex: 2,
            },
        ];
        let streams = [SceGxmVertexStream {
            stride: VERTEX_STRIDE,
            indexSource: 0,
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
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreateVertexProgram failed: {}", rc);

        let mut frag_prog: *mut SceGxmFragmentProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateFragmentProgram(
            patcher,
            frag_prog_id,
            0x94200408, // SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4 / ABGR8
            SCE_GXM_MULTISAMPLE_NONE,
            core::ptr::null(),
            vert_prog_ptr,
            &mut frag_prog,
        );
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreateFragmentProgram failed: {}", rc);

        // Parametry uniformów
        let param_mvp = sceGxmProgramFindParameterByName(vert_prog_ptr, b"uViewProj\0".as_ptr());
        let param_texture = sceGxmProgramFindParameterByName(frag_prog_ptr, b"uTexture\0".as_ptr());

        // --- 9. Atlas tekstury ---
        let (atlas_mem, atlas_tex) = build_gpu_texture(atlas);

        // --- 10. Bufor werteksów / indeksów ---
        let vb_mem = GpuMem::alloc_uc(b"RC_GXM_VB\0", Self::VB_SIZE).unwrap();
        let ib_mem = GpuMem::alloc_uc(b"RC_GXM_IB\0", Self::IB_SIZE).unwrap();

        Self {
            context,
            render_target,
            _vdm_mem: vdm_mem,
            _vertex_ring_mem: vertex_ring_mem,
            _fragment_ring_mem: fragment_ring_mem,
            _frag_usse_mem: frag_usse_mem,
            _param_mem: param_mem,
            display_bufs,
            display_sync,
            color_surfaces,
            display_data,
            current_buf: 0,
            depth_mem,
            depth_surface,
            patcher,
            _patcher_buf_mem: patcher_buf_mem,
            _patcher_vert_usse: patcher_vert_usse,
            _patcher_frag_usse: patcher_frag_usse,
            vert_prog_id,
            frag_prog_id,
            vert_prog,
            frag_prog,
            param_mvp,
            param_texture,
            _atlas_mem: atlas_mem,
            atlas_tex,
            vb_mem,
            ib_mem,
            mesh_cache: HashMap::new(),
        }
    }

    /// Renderuje jedną klatkę świata vokseli.
    /// Zwraca liczbę narysowanych trójkątów (debug).
    pub fn render_frame(&mut self, camera: &Camera, world: &mut World) -> u32 {
        let buf_idx = self.current_buf;
        let prev_buf = (buf_idx + DISPLAY_BUFFER_COUNT - 1) % DISPLAY_BUFFER_COUNT;

        unsafe {
            // Rozpocznij scenę GXM
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

            // Stan GPU
            sceGxmSetCullMode(self.context, SCE_GXM_CULL_CW); // backface culling
            sceGxmSetFrontDepthFunc(self.context, 0x00C00000u32); // LESS_EQUAL
            sceGxmSetFrontDepthWriteEnable(self.context, 1);

            // Programy shaderów
            sceGxmSetVertexProgram(self.context, self.vert_prog);
            sceGxmSetFragmentProgram(self.context, self.frag_prog);

            // MVP matrix: Projection × View
            let aspect = VITA_WIDTH as f32 / VITA_HEIGHT as f32;
            let proj = camera.projection_matrix(aspect);
            let view = camera.view_matrix();
            let mvp = proj.mul(&view);
            let mut uniform_buf: *mut core::ffi::c_void = core::ptr::null_mut();
            sceGxmReserveVertexDefaultUniformBuffer(self.context, &mut uniform_buf);
            sceGxmSetUniformDataF(
                uniform_buf,
                self.param_mvp,
                0,
                16, // 4×4 = 16 floatów
                mvp.m.as_ptr(),
            );

            // Tekstura atlasu do sloту 0
            sceGxmSetFragmentTexture(self.context, 0, &self.atlas_tex);

            // --- Budowanie vertex/index bufferów z chunków ---
            let vb_ptr = self.vb_mem.as_mut_ptr::<GxmVertex>();
            let ib_ptr = self.ib_mem.as_mut_ptr::<u32>();
            let max_verts = Self::VB_SIZE / core::mem::size_of::<GxmVertex>();
            let max_idx = Self::IB_SIZE / core::mem::size_of::<u32>();

            let cam_chunk_x = (camera.position.x / CHUNK_SIZE_X as f32).floor() as i32;
            let cam_chunk_z = (camera.position.z / CHUNK_SIZE_Z as f32).floor() as i32;
            let render_dist: i32 = 4;

            let mut vb_count: usize = 0;
            let mut ib_count: usize = 0;
            let mut tri_count: u32 = 0;

            for dz in -render_dist..=render_dist {
                for dx in -render_dist..=render_dist {
                    let cx = cam_chunk_x + dx;
                    let cz = cam_chunk_z + dz;
                    let key = (cx, cz);

                    // Buduj/aktualizuj mesh jeśli potrzeba
                    if !self.mesh_cache.contains_key(&key) {
                        if let Some(chunk) = world.chunks.get(&key) {
                            let tris = build_chunk_mesh(chunk, world);
                            self.mesh_cache.insert(key, tris);
                        }
                    }

                    if let Some(triangles) = self.mesh_cache.get(&key) {
                        for tri in triangles {
                            if vb_count + 3 >= max_verts || ib_count + 3 >= max_idx {
                                // Flush przed overflow
                                Self::flush_draw(
                                    self.context,
                                    vb_ptr,
                                    ib_ptr,
                                    vb_count,
                                    ib_count,
                                );
                                tri_count += ib_count as u32 / 3;
                                vb_count = 0;
                                ib_count = 0;
                            }

                            let (u0, v0, u1, v1) = atlas_uv(tri.texture);
                            // Trójkąt: v0(u0,v0) v1(u1,v0) v2(u0,v1) – zakładamy quad-split
                            let base = vb_count as u32;

                            *vb_ptr.add(vb_count) = GxmVertex::from_vertex(&tri.v0, u0, v0);
                            *vb_ptr.add(vb_count + 1) = GxmVertex::from_vertex(&tri.v1, u1, v0);
                            *vb_ptr.add(vb_count + 2) = GxmVertex::from_vertex(&tri.v2, u0, v1);

                            *ib_ptr.add(ib_count) = base;
                            *ib_ptr.add(ib_count + 1) = base + 1;
                            *ib_ptr.add(ib_count + 2) = base + 2;

                            vb_count += 3;
                            ib_count += 3;
                        }
                    }
                }
            }

            // Ostatni flush
            if ib_count > 0 {
                Self::flush_draw(self.context, vb_ptr, ib_ptr, vb_count, ib_count);
                tri_count += ib_count as u32 / 3;
            }

            // Zakończ scenę
            sceGxmEndScene(self.context, core::ptr::null(), core::ptr::null());

            // Display swap – asynchroniczny callback
            sceGxmPadHeartbeat(&self.color_surfaces[buf_idx], self.display_sync[buf_idx]);
            sceGxmDisplayQueueAddEntry(
                self.display_sync[prev_buf],
                self.display_sync[buf_idx],
                &self.display_data[buf_idx] as *const DisplayData as *const core::ffi::c_void,
            );

            tri_count
        }
    }

    unsafe fn flush_draw(
        ctx: *mut SceGxmContext,
        vb: *mut GxmVertex,
        ib: *mut u32,
        vb_count: usize,
        ib_count: usize,
    ) {
        if ib_count == 0 { return; }
        sceGxmSetVertexStream(ctx, 0, vb as *const core::ffi::c_void);
        sceGxmDraw(ctx, SCE_GXM_PRIMITIVE_TRIANGLES, SCE_GXM_INDEX_FORMAT_U32, ib as *const core::ffi::c_void, ib_count as u32);
    }

    pub fn next_buffer(&mut self) {
        self.current_buf = (self.current_buf + 1) % DISPLAY_BUFFER_COUNT;
    }

    pub fn invalidate_chunk(&mut self, cx: i32, cz: i32) {
        self.mesh_cache.remove(&(cx, cz));
    }
}

// malloc/free z libc (newlib na vita)
extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}

unsafe fn libc_malloc(size: usize) -> *mut core::ffi::c_void {
    malloc(size)
}

unsafe fn libc_free(ptr: *mut core::ffi::c_void) {
    free(ptr)
}
