//! `GxmRenderer` state struct, pool memory budgets, and public backend query methods.
//! Separates renderer state layout from hardware initialization (`init.rs`).

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::graphics::atlas::TextureAtlas;
use crate::world::mesher::{AsyncMesher, Triangle};

use super::ffi::*;
use super::mem::{DisplayData, GpuMem};
use super::programs::LinkedGxmShaders;
use super::types::{ChunkSlotRanges, GxmEntityVertex, GxmVertex};

pub struct GxmRenderer {
    pub(super) context: *mut SceGxmContext,
    pub(super) host_mem: *mut core::ffi::c_void,
    pub(super) render_target: *mut SceGxmRenderTarget,

    pub(super) _vdm_mem: GpuMem,
    pub(super) _vertex_ring_mem: GpuMem,
    pub(super) _fragment_ring_mem: GpuMem,
    pub(super) _frag_usse_mem: GpuMem,

    pub(super) display_bufs: [GpuMem; DISPLAY_BUFFER_COUNT],
    pub(super) display_sync: [*mut SceGxmSyncObject; DISPLAY_BUFFER_COUNT],
    pub(super) color_surfaces: [SceGxmColorSurface; DISPLAY_BUFFER_COUNT],
    pub(super) display_data: [DisplayData; DISPLAY_BUFFER_COUNT],
    pub(super) front_buf: usize,
    pub(super) back_buf: usize,
    pub(super) last_was_3d: bool,
    pub(super) rendered_3d_frames: u32,

    pub(super) _depth_mem: GpuMem,
    pub(super) depth_surface: SceGxmDepthStencilSurface,

    pub(super) patcher: *mut SceGxmShaderPatcher,
    pub(super) _patcher_buf_mem: GpuMem,
    pub(super) _patcher_vert_usse: GpuMem,
    pub(super) _patcher_frag_usse: GpuMem,
    pub(super) shaders: LinkedGxmShaders,

    pub(super) clear_vb_mem: GpuMem,
    pub(super) clear_ib_mem: GpuMem,

    pub(super) _atlas_mem: GpuMem,
    pub(super) atlas_tex: SceGxmTexture,
    pub(super) hud_bufs: [GpuMem; DISPLAY_BUFFER_COUNT],
    pub(super) hud_texs: [SceGxmTexture; DISPLAY_BUFFER_COUNT],
    pub(super) hud_row_bounds: [[(u16, u16); VITA_HEIGHT as usize]; DISPLAY_BUFFER_COUNT],

    pub(super) vb_mem: GpuMem,
    pub(super) ib_mem: GpuMem,
    pub(super) entity_vb_mem: GpuMem,
    pub(super) entity_ib_mem: GpuMem,

    // Cached CPU staging buffers for burst-copying chunk meshes into GPU memory
    pub(super) scratch_vb: Vec<GxmVertex>,
    pub(super) scratch_opaque_ib: Vec<u16>,
    pub(super) scratch_cutout_ib: Vec<u16>,
    pub(super) scratch_trans_ib: Vec<u16>,

    // Chunk geometry is built on worker threads; the render thread only uploads
    // at most one completed mesh per frame and never runs the mesher itself.
    pub(super) mesh_worker: AsyncMesher,
    pub(super) pending_meshes: HashSet<(i32, i32)>,
    pub(super) completed_meshes: Vec<((i32, i32), Vec<Triangle>)>,

    pub(super) chunk_slots: HashMap<(i32, i32), usize>,
    pub chunk_spawn_times: HashMap<(i32, i32), f32>,
    pub(super) free_slots: Vec<usize>,
    pub(super) evicted_slots_ring: [Vec<usize>; DISPLAY_BUFFER_COUNT],
    pub(super) slot_ranges: [ChunkSlotRanges; GxmRenderer::MAX_CHUNK_SLOTS],
    pub(super) freed_slots_scratch: Vec<usize>,
    pub(super) chunk_coords_scratch: Vec<(i32, i32)>,

    pub last_draw_calls: u32,
    pub last_triangles_rendered: u32,
    pub last_triangles_culled: u32,
    pub last_subchunks_rendered: u32,
    pub last_subchunks_culled: u32,
    pub(super) last_timings: crate::core::profiler::ExternalRenderTimings,
}

unsafe impl Send for GxmRenderer {}
unsafe impl Sync for GxmRenderer {}

impl GxmRenderer {
    // DLACZEGO: 96 slots cover all 81 radius-5 circular chunks with 15 slots of eviction headroom;
    // paired with 18432 indices/slot (~44 MB chunk pool + display/HUD/entity buffers <= 66 MB CDRAM).
    pub const MAX_CHUNK_SLOTS: usize = 96;
    pub const MAX_VERTS_PER_SLOT: usize = 18432;
    pub const MAX_INDICES_PER_SLOT: usize = 18432;
    const _MATCH_VITA_TRI_CAP: () = assert!(
        Self::MAX_VERTS_PER_SLOT / 3 == crate::features::meshing::constants::VITA_MAX_MESH_TRIANGLES
    );

    pub const MAX_ENTITY_VERTS: usize = 12288;
    pub const MAX_ENTITY_INDICES: usize = 18432;

    pub(super) const VB_SIZE: usize = Self::MAX_CHUNK_SLOTS * Self::MAX_VERTS_PER_SLOT * core::mem::size_of::<GxmVertex>();
    pub(super) const IB_SIZE: usize = Self::MAX_CHUNK_SLOTS * Self::MAX_INDICES_PER_SLOT * core::mem::size_of::<u16>();
    pub(super) const ENTITY_VB_SIZE: usize = DISPLAY_BUFFER_COUNT * Self::MAX_ENTITY_VERTS * core::mem::size_of::<GxmEntityVertex>();
    pub(super) const ENTITY_IB_SIZE: usize = DISPLAY_BUFFER_COUNT * Self::MAX_ENTITY_INDICES * core::mem::size_of::<u16>();

    pub fn new(atlas: &TextureAtlas) -> Self {
        unsafe { Self::init_unsafe(atlas) }
    }

    #[inline(always)]
    pub fn backend_name(&self) -> &'static str {
        "libgxm"
    }

    #[inline(always)]
    pub fn gpu_info(&self) -> (&'static str, &'static str, &'static str) {
        ("PowerVR SGX543MP4+", "SCE libgxm", "PS Vita Hardware")
    }

    #[inline(always)]
    pub fn last_frame_timings(&self) -> crate::core::profiler::ExternalRenderTimings {
        self.last_timings
    }
}
