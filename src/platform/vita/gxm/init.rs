//! Hardware GXM GPU initialization (`sceGxmInitialize` -> render target -> CDRAM -> buffers).
//! Allocates triple display/HUD surfaces, tiled depth, shader patcher, and CDRAM chunk/entity pools.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::graphics::atlas::TextureAtlas;
use crate::world::mesher::AsyncMesher;

use super::ffi::*;
use super::mem::{display_callback, DisplayData, GpuMem};
use super::programs::register_and_link_shaders;
use super::renderer::GxmRenderer;
use super::textures::{build_gpu_texture, build_hud_textures};
use super::types::ChunkSlotRanges;

impl GxmRenderer {
    pub(super) unsafe fn init_unsafe(atlas: &TextureAtlas) -> Self {
        let init_params = SceGxmInitializeParams {
            flags: 0,
            displayQueueMaxPendingCount: MAX_PENDING_SWAPS,
            displayQueueCallback: Some(display_callback),
            displayQueueCallbackDataSize: core::mem::size_of::<DisplayData>() as u32,
            parameterBufferSize: SCE_GXM_DEFAULT_PARAMETER_BUFFER_SIZE as u32,
        };
        let rc = sceGxmInitialize(&init_params);
        assert_eq!(rc, SCE_OK, "sceGxmInitialize failed: 0x{:08X}", rc);

        let vdm_mem = GpuMem::alloc_gpu(
            b"RC_GXM_VDM\0",
            SCE_GXM_DEFAULT_VDM_RING_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .expect("VDM alloc failed");

        let vertex_ring_mem = GpuMem::alloc_gpu(
            b"RC_GXM_VRing\0",
            SCE_GXM_DEFAULT_VERTEX_RING_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .expect("VRing alloc failed");

        let fragment_ring_mem = GpuMem::alloc_gpu(
            b"RC_GXM_FRing\0",
            SCE_GXM_DEFAULT_FRAGMENT_RING_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .expect("FRing alloc failed");

        let mut frag_usse_offset: u32 = 0;
        let frag_usse_mem = GpuMem::alloc_fragment_usse(
            b"RC_GXM_FUSSE\0",
            SCE_GXM_DEFAULT_FRAGMENT_USSE_RING_BUFFER_SIZE,
            &mut frag_usse_offset,
        )
        .expect("FUSSE alloc failed");

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

        // Triple Display Buffers in CDRAM (with hardware dithering)
        let color_surface_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
        let names: [&[u8]; DISPLAY_BUFFER_COUNT] = [b"RC_GXM_CB0\0", b"RC_GXM_CB1\0", b"RC_GXM_CB2\0"];

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
            sceGxmColorSurfaceSetDitherMode(&mut color_surfaces[i], SCE_GXM_COLOR_SURFACE_DITHER_ENABLED);
            display_data[i].address = ptr;

            let mut sync: *mut SceGxmSyncObject = core::ptr::null_mut();
            sceGxmSyncObjectCreate(&mut sync);
            display_sync[i] = sync;
        }

        // Tiled DF32 Depth Buffer in CDRAM
        let depth_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
        let depth_mem = GpuMem::alloc_gpu(
            b"RC_GXM_Depth\0",
            depth_size,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
            SCE_GXM_MEMORY_ATTRIB_RW,
        )
        .unwrap();

        let mut depth_surface = SceGxmDepthStencilSurface { _private: [0u32; 5] };
        let depth_init_rc = sceGxmDepthStencilSurfaceInit(
            &mut depth_surface,
            SCE_GXM_DEPTH_STENCIL_FORMAT_DF32,
            SCE_GXM_DEPTH_STENCIL_SURFACE_TILED,
            VITA_WIDTH,
            depth_mem.ptr,
            core::ptr::null_mut(),
        );
        assert_eq!(depth_init_rc, SCE_OK, "sceGxmDepthStencilSurfaceInit failed: 0x{:08X}", depth_init_rc);

        // Shader Patcher & Linking
        let patcher_buf_mem = GpuMem::alloc_gpu(
            b"RC_GXM_PatchBuf\0",
            SCE_GXM_DEFAULT_PATCHER_BUFFER_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_RW,
        )
        .unwrap();

        let mut vert_usse_offset: u32 = 0;
        let patcher_vert_usse = GpuMem::alloc_vertex_usse(
            b"RC_GXM_PatchVUSS\0",
            SCE_GXM_DEFAULT_PATCHER_VERTEX_USSE_SIZE,
            &mut vert_usse_offset,
        )
        .unwrap();

        let mut frag_usse_offset: u32 = 0;
        let patcher_frag_usse = GpuMem::alloc_fragment_usse(
            b"RC_GXM_PatchFUSS\0",
            SCE_GXM_DEFAULT_PATCHER_FRAGMENT_USSE_SIZE,
            &mut frag_usse_offset,
        )
        .unwrap();

        let patcher_params = SceGxmShaderPatcherParams {
            userData: core::ptr::null_mut(),
            hostAllocCallback: Some(super::programs::patcher_host_alloc),
            hostFreeCallback: Some(super::programs::patcher_host_free),
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

        let shaders = register_and_link_shaders(patcher);

        // Shared fullscreen triangle VB/IB for clear, Early-Z sky, Post-FX, biome tint & HUD
        let clear_vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_ClearVB\0",
            3 * 2 * core::mem::size_of::<f32>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .unwrap();

        let clear_verts = clear_vb_mem.ptr as *mut f32;
        *clear_verts.add(0) = -1.0f32;
        *clear_verts.add(1) = -1.0f32;
        *clear_verts.add(2) = 3.0f32;
        *clear_verts.add(3) = -1.0f32;
        *clear_verts.add(4) = -1.0f32;
        *clear_verts.add(5) = 3.0f32;

        let clear_ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_ClearIB\0",
            3 * core::mem::size_of::<u16>(),
            SCE_KERNEL_MEMBLOCK_TYPE_USER_RW_UNCACHE,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .unwrap();

        let clear_indices = clear_ib_mem.ptr as *mut u16;
        *clear_indices.add(0) = 0;
        *clear_indices.add(1) = 1;
        *clear_indices.add(2) = 2;

        let (atlas_mem, atlas_tex) = build_gpu_texture(atlas);
        let (hud_bufs, hud_texs) = build_hud_textures();

        // DLACZEGO: Chunk and triple-buffered entity buffers reside in CDRAM (with automatic
        // GpuMem fallback to uncached RAM) to free ~31 MB Main RAM for world chunks and audio.
        let vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_VB\0",
            Self::VB_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .unwrap();

        let ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_IB\0",
            Self::IB_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .unwrap();

        let entity_vb_mem = GpuMem::alloc_gpu(
            b"RC_GXM_EntVB\0",
            Self::ENTITY_VB_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .unwrap();

        let entity_ib_mem = GpuMem::alloc_gpu(
            b"RC_GXM_EntIB\0",
            Self::ENTITY_IB_SIZE,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
            SCE_GXM_MEMORY_ATTRIB_READ,
        )
        .unwrap();

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
            last_was_3d: false,
            rendered_3d_frames: 0,
            _depth_mem: depth_mem,
            depth_surface,
            patcher,
            _patcher_buf_mem: patcher_buf_mem,
            _patcher_vert_usse: patcher_vert_usse,
            _patcher_frag_usse: patcher_frag_usse,
            shaders,
            clear_vb_mem,
            clear_ib_mem,
            _atlas_mem: atlas_mem,
            atlas_tex,
            hud_bufs,
            hud_texs,
            vb_mem,
            ib_mem,
            entity_vb_mem,
            entity_ib_mem,
            scratch_vb: Vec::with_capacity(Self::MAX_VERTS_PER_SLOT),
            scratch_opaque_ib: Vec::with_capacity(Self::MAX_INDICES_PER_SLOT),
            scratch_cutout_ib: Vec::with_capacity(Self::MAX_INDICES_PER_SLOT / 4),
            scratch_trans_ib: Vec::with_capacity(Self::MAX_INDICES_PER_SLOT / 4),
            mesh_worker: AsyncMesher::new(),
            pending_meshes: HashSet::with_capacity(8),
            completed_meshes: Vec::with_capacity(8),
            chunk_slots: HashMap::new(),
            chunk_spawn_times: HashMap::new(),
            free_slots,
            evicted_slots_ring: [
                Vec::with_capacity(16),
                Vec::with_capacity(16),
                Vec::with_capacity(16),
            ],
            slot_ranges: [ChunkSlotRanges::default(); Self::MAX_CHUNK_SLOTS],
            freed_slots_scratch: Vec::with_capacity(Self::MAX_CHUNK_SLOTS),
            chunk_coords_scratch: Vec::with_capacity(121),
            last_draw_calls: 0,
            last_triangles_rendered: 0,
            last_triangles_culled: 0,
            last_subchunks_rendered: 0,
            last_subchunks_culled: 0,
        }
    }
}
