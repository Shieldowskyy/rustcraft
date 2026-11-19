//! `GxmRenderer` state struct, pool memory budgets, and public backend query methods.
//! Separates renderer state layout from hardware initialization (`init.rs`).

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};

use crate::graphics::atlas::TextureAtlas;
use crate::world::mesher::{AsyncMesher, Triangle};

use super::ffi::*;
use super::mem::{DisplayData, GpuMem};
use super::types::{ChunkSlotRanges, GxmVertex};

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

    pub(super) _depth_mem: GpuMem,
    pub(super) depth_surface: SceGxmDepthStencilSurface,

    pub(super) patcher: *mut SceGxmShaderPatcher,
    pub(super) _patcher_buf_mem: GpuMem,
    pub(super) _patcher_vert_usse: GpuMem,
    pub(super) _patcher_frag_usse: GpuMem,

    pub(super) _aligned_voxel_vert_gxp: Vec<u64>,
    pub(super) _aligned_voxel_frag_gxp: Vec<u64>,
    pub(super) vert_prog_id: SceGxmShaderPatcherId,
    pub(super) frag_prog_id: SceGxmShaderPatcherId,
    pub(super) clear_vert_id: SceGxmShaderPatcherId,
    pub(super) clear_frag_id: SceGxmShaderPatcherId,

    pub(super) vert_prog: *mut SceGxmVertexProgram,
    pub(super) frag_prog_opaque: *mut SceGxmFragmentProgram,
    pub(super) frag_prog_trans: *mut SceGxmFragmentProgram,
    pub(super) param_mvp: *const SceGxmProgramParameter,
    pub(super) param_fog: *const SceGxmProgramParameter,
    pub(super) param_fog_color: *const SceGxmProgramParameter,

    pub(super) clear_vert_prog: *mut SceGxmVertexProgram,
    pub(super) clear_frag_prog: *mut SceGxmFragmentProgram,
    pub(super) param_clear_color: *const SceGxmProgramParameter,
    pub(super) clear_vb_mem: GpuMem,
    pub(super) clear_ib_mem: GpuMem,

    pub(super) _atlas_mem: GpuMem,
    pub(super) atlas_tex: SceGxmTexture,

    pub(super) vb_mem: GpuMem,
    pub(super) ib_mem: GpuMem,

    // Cached CPU staging buffers for burst-copying meshes into uncached GPU memory
    pub(super) scratch_vb: Vec<GxmVertex>,
    pub(super) scratch_opaque_ib: Vec<u16>,
    pub(super) scratch_cutout_ib: Vec<u16>,
    pub(super) scratch_trans_ib: Vec<u16>,

    // Chunk geometry is built on worker threads; the render thread only uploads
    // at most one completed mesh per frame and never runs the mesher itself.
    pub(super) mesh_worker: AsyncMesher,
    pub(super) pending_meshes: HashSet<(i32, i32)>,
    pub(super) completed_meshes: Vec<((i32, i32), Vec<Triangle>)>,

    pub(super) crack_vb_mem: GpuMem,
    pub(super) crack_ib_mem: GpuMem,

    pub(super) entity_vb_mem: GpuMem,
    pub(super) entity_ib_mem: GpuMem,

    pub(super) chunk_slots: HashMap<(i32, i32), usize>,
    pub chunk_spawn_times: HashMap<(i32, i32), f32>,
    pub(super) free_slots: Vec<usize>,
    pub(super) slot_ranges: [ChunkSlotRanges; GxmRenderer::MAX_CHUNK_SLOTS],
    pub(super) freed_slots_scratch: Vec<usize>,
    pub(super) chunk_coords_scratch: Vec<(i32, i32)>,
    pub(super) last_daylight_bucket: i32,

    pub last_draw_calls: u32,
    pub last_triangles_rendered: u32,
    pub last_triangles_culled: u32,
    pub last_subchunks_rendered: u32,
    pub last_subchunks_culled: u32,
}

unsafe impl Send for GxmRenderer {}
unsafe impl Sync for GxmRenderer {}

impl GxmRenderer {
    // DLACZEGO: Chunk pool sizes fit alongside the 180 MB Newlib heap inside PS Vita's 256 MB RAM
    // (~18 MB VB + ~2.25 MB IB in uncached kernel memory).
    pub const MAX_CHUNK_SLOTS: usize = 64;
    pub const MAX_VERTS_PER_SLOT: usize = 18432;
    pub const MAX_INDICES_PER_SLOT: usize = 27648;

    pub const MAX_ENTITY_VERTS: usize = 8192;
    pub const MAX_ENTITY_INDICES: usize = 12288;

    pub(super) const VB_SIZE: usize = Self::MAX_CHUNK_SLOTS * Self::MAX_VERTS_PER_SLOT * core::mem::size_of::<GxmVertex>();
    pub(super) const IB_SIZE: usize = Self::MAX_CHUNK_SLOTS * Self::MAX_INDICES_PER_SLOT * core::mem::size_of::<u16>();

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
}
