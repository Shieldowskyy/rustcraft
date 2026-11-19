//! Hardware GXM GPU initialization (`sceGxmInitialize` -> render target -> CDRAM -> buffers).
//! Allocates triple framebuffers, tiled depth buffer, shader patcher, and chunk slot pools.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::graphics::atlas::TextureAtlas;
use crate::world::mesher::AsyncMesher;

use super::ffi::*;
use super::mem::{display_callback, DisplayData, GpuMem};
use super::programs::register_and_link_shaders;
use super::renderer::GxmRenderer;
use super::textures::build_gpu_texture;
use super::types::{ChunkSlotRanges, GxmVertex};

impl GxmRenderer {
    pub(super) unsafe fn init_unsafe(atlas: &TextureAtlas) -> Self {
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

        // --- 2. Allocate GPU Ring Buffers ---
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

        // --- 3. Create GXM Context with hostMem (malloc) ---
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

        // --- 5. Triple Display Buffers in CDRAM (with Hardware Dithering) ---
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
            // Hardware Dithering eliminates color banding in vignettes, fog, and sky gradients
            sceGxmColorSurfaceSetDitherMode(&mut color_surfaces[i], SCE_GXM_COLOR_SURFACE_DITHER_ENABLED);
            display_data[i].address = ptr;

            let mut sync: *mut SceGxmSyncObject = core::ptr::null_mut();
            sceGxmSyncObjectCreate(&mut sync);
            display_sync[i] = sync;
        }

        // --- 6. Depth Buffer (Tiled for PowerVR SGX543MP4 TBDR bandwidth efficiency) ---
        let depth_size = (VITA_WIDTH * VITA_HEIGHT * 4) as usize;
        let depth_mem = GpuMem::alloc_gpu(
            b"RC_GXM_Depth\0",
            depth_size,
            SCE_KERNEL_MEMBLOCK_TYPE_USER_CDRAM_RW,
            SCE_GXM_MEMORY_ATTRIB_RW,
        ).unwrap();

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

        // Vertex buffer for full-screen clear triangle (NDC [-1, 3])
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

        // --- 9. GPU Texture Atlas ---
        let (atlas_mem, atlas_tex) = build_gpu_texture(atlas);

        // --- 10. GPU Buffer for Chunk Slots (Uncached RAM for immediate CPU writes) ---
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

        // --- 10b. Dynamic buffers for block destruction cracks ---
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

        // --- 10c. Dynamic buffers for entities (dropped items, mobs, TNT) ---
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
            _aligned_voxel_vert_gxp: shaders.aligned_voxel_vert_gxp,
            _aligned_voxel_frag_gxp: shaders.aligned_voxel_frag_gxp,
            vert_prog_id: shaders.vert_prog_id,
            frag_prog_id: shaders.frag_prog_id,
            clear_vert_id: shaders.clear_vert_id,
            clear_frag_id: shaders.clear_frag_id,
            vert_prog: shaders.vert_prog,
            frag_prog_opaque: shaders.frag_prog_opaque,
            frag_prog_trans: shaders.frag_prog_trans,
            param_mvp: shaders.param_mvp,
            param_fog: shaders.param_fog,
            param_fog_color: shaders.param_fog_color,
            clear_vert_prog: shaders.clear_vert_prog,
            clear_frag_prog: shaders.clear_frag_prog,
            param_clear_color: shaders.param_clear_color,
            clear_vb_mem,
            clear_ib_mem,
            _atlas_mem: atlas_mem,
            atlas_tex,
            vb_mem,
            ib_mem,
            scratch_vb: Vec::with_capacity(Self::MAX_VERTS_PER_SLOT),
            scratch_opaque_ib: Vec::with_capacity(Self::MAX_INDICES_PER_SLOT),
            scratch_cutout_ib: Vec::with_capacity(Self::MAX_INDICES_PER_SLOT / 4),
            scratch_trans_ib: Vec::with_capacity(Self::MAX_INDICES_PER_SLOT / 4),
            mesh_worker: AsyncMesher::new(),
            pending_meshes: HashSet::with_capacity(8),
            completed_meshes: Vec::with_capacity(8),
            crack_vb_mem,
            crack_ib_mem,
            entity_vb_mem,
            entity_ib_mem,
            chunk_slots: HashMap::new(),
            chunk_spawn_times: HashMap::new(),
            free_slots,
            slot_ranges: [ChunkSlotRanges::default(); Self::MAX_CHUNK_SLOTS],
            freed_slots_scratch: Vec::with_capacity(Self::MAX_CHUNK_SLOTS),
            chunk_coords_scratch: Vec::with_capacity(121),
            last_daylight_bucket: 20,
            last_draw_calls: 0,
            last_triangles_rendered: 0,
            last_triangles_culled: 0,
            last_subchunks_rendered: 0,
            last_subchunks_culled: 0,
        }
    }

}
