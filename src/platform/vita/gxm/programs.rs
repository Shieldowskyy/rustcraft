//! 16-byte aligned GXP shader bytecode, shader patcher helpers, and GXM program linking.
//! Registers all 12 Cg->GXP shaders + hardware clear pipelines and releases them cleanly on shutdown.

#![allow(dead_code)]

use super::ffi::*;
use super::programs_fx::{link_fx_and_entity_shaders, reg_index_or};
use super::types::GxmVertex;

// DLACZEGO: ARMv7 and sceGxmProgramCheck require SceGxmProgram headers to be
// at least 16-byte aligned; raw `include_bytes!` only has 1-byte alignment.
#[repr(C, align(16))]
pub(super) struct AlignedGxp<const N: usize>(pub(super) [u8; N]);

/// Copies GXP bytecode into an 8/16-byte aligned heap buffer preserved for the patcher's lifetime.
pub(super) fn alloc_aligned_gxp(bytes: &[u8]) -> Vec<u64> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let u64_len = (bytes.len() + 7) / 8;
    let mut aligned = vec![0u64; u64_len];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), aligned.as_mut_ptr() as *mut u8, bytes.len());
    }
    aligned
}

static OUT_VOXEL_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.vert.gxp"));
static OUT_VOXEL_OPAQUE_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel_opaque.frag.gxp"));
static OUT_VOXEL_CUTOUT_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel_cutout.frag.gxp"));
static OUT_VOXEL_TRANS_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel_trans.frag.gxp"));

// Precompiled hardware clear shaders (standard SDL2 GXM clear pipeline)
pub(super) static CLEAR_VERT_GXP: AlignedGxp<252> = AlignedGxp([
    0x47, 0x58, 0x50, 0x00, 0x01, 0x05, 0x50, 0x03, 0xfa, 0x00, 0x00, 0x00, 0xdc, 0x25, 0x34, 0x74,
    0x53, 0x4a, 0x7a, 0x5b, 0x04, 0x00, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xb8, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00,
    0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x74, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x78, 0x00, 0x00, 0x00, 0xc0, 0x3d, 0x03, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x6c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x5c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x4c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x04,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x44, 0xfa, 0x01, 0x00, 0x04, 0x90, 0x85, 0x11, 0xa5, 0x08,
    0x01, 0x80, 0x56, 0x90, 0x81, 0x11, 0x83, 0x08, 0x00, 0x00, 0x20, 0xa0, 0x00, 0x50, 0x27, 0xfb,
    0x10, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x61, 0x50, 0x6f, 0x73, 0x69, 0x74, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00,
]);

pub(super) static CLEAR_FRAG_GXP: AlignedGxp<232> = AlignedGxp([
    0x47, 0x58, 0x50, 0x00, 0x01, 0x05, 0x50, 0x03, 0xe8, 0x00, 0x00, 0x00, 0xa2, 0x55, 0x22, 0x3e,
    0xc6, 0x7e, 0x77, 0xf1, 0x01, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xa4, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00, 0x64, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x5c, 0x00, 0x00, 0x00, 0xc0, 0x3d, 0x03, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x50, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x44, 0xfa, 0x02, 0x80, 0x19, 0xf0,
    0x7e, 0x0d, 0x80, 0x40, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 0x10, 0x00, 0x00, 0x00,
    0x01, 0xe4, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x75, 0x43, 0x6c, 0x65,
    0x61, 0x72, 0x43, 0x6f, 0x6c, 0x6f, 0x72, 0x00,
]);

pub(super) unsafe extern "C" fn patcher_host_alloc(_: *mut core::ffi::c_void, size: u32) -> *mut core::ffi::c_void {
    malloc(size as usize)
}

pub(super) unsafe extern "C" fn patcher_host_free(_: *mut core::ffi::c_void, ptr: *mut core::ffi::c_void) {
    free(ptr);
}

pub(super) struct LinkedGxmShaders {
    pub(super) aligned_gxps: Vec<Vec<u64>>,
    pub(super) registered_ids: Vec<SceGxmShaderPatcherId>,

    // Voxel terrain pipeline (opaque / cutout / transparent)
    pub(super) vert_prog: *mut SceGxmVertexProgram,
    pub(super) frag_prog_opaque: *mut SceGxmFragmentProgram,
    pub(super) frag_prog_cutout: *mut SceGxmFragmentProgram,
    pub(super) frag_prog_trans: *mut SceGxmFragmentProgram,
    pub(super) param_voxel_mvp: *const SceGxmProgramParameter,
    pub(super) param_voxel_sky_daylight: *const SceGxmProgramParameter,
    pub(super) param_voxel_chunk_offset: *const SceGxmProgramParameter,
    pub(super) param_opaque_cam_fog: *const SceGxmProgramParameter,
    pub(super) param_opaque_fog_color: *const SceGxmProgramParameter,
    pub(super) param_cutout_cam_fog: *const SceGxmProgramParameter,
    pub(super) param_cutout_fog_color: *const SceGxmProgramParameter,
    pub(super) param_trans_cam_fog: *const SceGxmProgramParameter,
    pub(super) param_trans_fog_color: *const SceGxmProgramParameter,

    // Early-Z Procedural Sky pipeline
    pub(super) sky_vert_prog: *mut SceGxmVertexProgram,
    pub(super) sky_frag_prog: *mut SceGxmFragmentProgram,
    pub(super) param_sky_inv_vp: *const SceGxmProgramParameter,
    pub(super) param_sky_cam_daylight: *const SceGxmProgramParameter,
    pub(super) param_sky_top: *const SceGxmProgramParameter,
    pub(super) param_sky_bot: *const SceGxmProgramParameter,
    pub(super) param_sky_sun_dir: *const SceGxmProgramParameter,
    pub(super) param_sky_moon_dir: *const SceGxmProgramParameter,

    // Dynamic entity, player, cloud, weather & hand pipeline
    pub(super) entity_vert_prog: *mut SceGxmVertexProgram,
    pub(super) entity_frag_opaque: *mut SceGxmFragmentProgram,
    pub(super) entity_frag_blend: *mut SceGxmFragmentProgram,
    pub(super) param_entity_mvp: *const SceGxmProgramParameter,
    pub(super) param_entity_cam_fog: *const SceGxmProgramParameter,
    pub(super) param_entity_fog_color: *const SceGxmProgramParameter,

    // Post-FX, Biome Tint & HUD compositor
    pub(super) postfx_vert_prog: *mut SceGxmVertexProgram,
    pub(super) postfx_frag_prog: *mut SceGxmFragmentProgram,
    pub(super) param_postfx_p0: *const SceGxmProgramParameter,
    pub(super) param_postfx_p1: *const SceGxmProgramParameter,
    pub(super) biome_tint_mul_prog: *mut SceGxmFragmentProgram,
    pub(super) biome_tint_add_prog: *mut SceGxmFragmentProgram,
    pub(super) param_biome_tint: *const SceGxmProgramParameter,
    pub(super) hud_frag_prog: *mut SceGxmFragmentProgram,

    // Hardware Clear & Depth-Only Clear
    pub(super) clear_vert_prog: *mut SceGxmVertexProgram,
    pub(super) clear_frag_prog: *mut SceGxmFragmentProgram,
    pub(super) clear_frag_depth_only: *mut SceGxmFragmentProgram,
    pub(super) param_clear_color: *const SceGxmProgramParameter,
}

impl LinkedGxmShaders {
    fn empty() -> Self {
        Self {
            aligned_gxps: Vec::with_capacity(12),
            registered_ids: Vec::with_capacity(14),
            vert_prog: core::ptr::null_mut(),
            frag_prog_opaque: core::ptr::null_mut(),
            frag_prog_cutout: core::ptr::null_mut(),
            frag_prog_trans: core::ptr::null_mut(),
            param_voxel_mvp: core::ptr::null(),
            param_voxel_sky_daylight: core::ptr::null(),
            param_voxel_chunk_offset: core::ptr::null(),
            param_opaque_cam_fog: core::ptr::null(),
            param_opaque_fog_color: core::ptr::null(),
            param_cutout_cam_fog: core::ptr::null(),
            param_cutout_fog_color: core::ptr::null(),
            param_trans_cam_fog: core::ptr::null(),
            param_trans_fog_color: core::ptr::null(),
            sky_vert_prog: core::ptr::null_mut(),
            sky_frag_prog: core::ptr::null_mut(),
            param_sky_inv_vp: core::ptr::null(),
            param_sky_cam_daylight: core::ptr::null(),
            param_sky_top: core::ptr::null(),
            param_sky_bot: core::ptr::null(),
            param_sky_sun_dir: core::ptr::null(),
            param_sky_moon_dir: core::ptr::null(),
            entity_vert_prog: core::ptr::null_mut(),
            entity_frag_opaque: core::ptr::null_mut(),
            entity_frag_blend: core::ptr::null_mut(),
            param_entity_mvp: core::ptr::null(),
            param_entity_cam_fog: core::ptr::null(),
            param_entity_fog_color: core::ptr::null(),
            postfx_vert_prog: core::ptr::null_mut(),
            postfx_frag_prog: core::ptr::null_mut(),
            param_postfx_p0: core::ptr::null(),
            param_postfx_p1: core::ptr::null(),
            biome_tint_mul_prog: core::ptr::null_mut(),
            biome_tint_add_prog: core::ptr::null_mut(),
            param_biome_tint: core::ptr::null(),
            hud_frag_prog: core::ptr::null_mut(),
            clear_vert_prog: core::ptr::null_mut(),
            clear_frag_prog: core::ptr::null_mut(),
            clear_frag_depth_only: core::ptr::null_mut(),
            param_clear_color: core::ptr::null(),
        }
    }

    pub(super) unsafe fn release(&mut self, patcher: *mut SceGxmShaderPatcher) {
        if patcher.is_null() {
            return;
        }
        let vert_progs = [
            self.vert_prog,
            self.sky_vert_prog,
            self.entity_vert_prog,
            self.postfx_vert_prog,
            self.clear_vert_prog,
        ];
        for vp in vert_progs {
            if !vp.is_null() {
                let _ = sceGxmShaderPatcherReleaseVertexProgram(patcher, vp);
            }
        }
        let frag_progs = [
            self.frag_prog_opaque,
            self.frag_prog_cutout,
            self.frag_prog_trans,
            self.sky_frag_prog,
            self.entity_frag_opaque,
            self.entity_frag_blend,
            self.postfx_frag_prog,
            self.biome_tint_mul_prog,
            self.biome_tint_add_prog,
            self.hud_frag_prog,
            self.clear_frag_prog,
            self.clear_frag_depth_only,
        ];
        for fp in frag_progs {
            if !fp.is_null() {
                let _ = sceGxmShaderPatcherReleaseFragmentProgram(patcher, fp);
            }
        }
        for &id in &self.registered_ids {
            if !id.is_null() {
                let _ = sceGxmShaderPatcherUnregisterProgram(patcher, id);
            }
        }
        self.registered_ids.clear();
    }
}

pub(super) unsafe fn register_gxp_program(
    patcher: *mut SceGxmShaderPatcher,
    bytes: &[u8],
    name: &str,
) -> (Vec<u64>, SceGxmShaderPatcherId) {
    assert!(bytes.len() > 64, "Missing or placeholder GXP bytecode for {}", name);
    let aligned = alloc_aligned_gxp(bytes);
    let hdr = aligned.as_ptr() as *const SceGxmProgram;
    let chk = sceGxmProgramCheck(hdr);
    assert_eq!(chk, SCE_OK, "sceGxmProgramCheck({}) failed: 0x{:08X}", name, chk);
    let mut id: SceGxmShaderPatcherId = core::ptr::null_mut();
    let rc = sceGxmShaderPatcherRegisterProgram(patcher, hdr, &mut id);
    assert_eq!(rc, SCE_OK, "RegisterProgram({}) failed: 0x{:08X}", name, rc);
    (aligned, id)
}

pub(super) unsafe fn create_frag_program(
    patcher: *mut SceGxmShaderPatcher,
    frag_id: SceGxmShaderPatcherId,
    vert_gxp: *const SceGxmProgram,
    blend: Option<&SceGxmBlendInfo>,
    name: &str,
) -> *mut SceGxmFragmentProgram {
    let blend_ptr = blend
        .map(|b| b as *const SceGxmBlendInfo as *const core::ffi::c_void)
        .unwrap_or(core::ptr::null());
    let mut prog: *mut SceGxmFragmentProgram = core::ptr::null_mut();
    let rc = sceGxmShaderPatcherCreateFragmentProgram(
        patcher,
        frag_id,
        SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4,
        SCE_GXM_MULTISAMPLE_NONE,
        blend_ptr,
        vert_gxp,
        &mut prog,
    );
    assert_eq!(rc, SCE_OK, "CreateFragmentProgram({}) failed: 0x{:08X}", name, rc);
    prog
}

pub(super) unsafe fn create_fullscreen_vert_program(
    patcher: *mut SceGxmShaderPatcher,
    vert_id: SceGxmShaderPatcherId,
    vert_hdr: *const SceGxmProgram,
    name: &str,
) -> *mut SceGxmVertexProgram {
    let reg_pos = reg_index_or(vert_hdr, b"aPosition\0", 0);
    let attrs = [SceGxmVertexAttribute {
        streamIndex: 0,
        offset: 0,
        format: SCE_GXM_ATTRIBUTE_FORMAT_F32 as u8,
        componentCount: 2,
        regIndex: reg_pos,
    }];
    let streams = [SceGxmVertexStream {
        stride: (2 * core::mem::size_of::<f32>()) as u16,
        indexSource: SCE_GXM_INDEX_SOURCE_INDEX_16BIT,
    }];
    let mut prog: *mut SceGxmVertexProgram = core::ptr::null_mut();
    let rc = sceGxmShaderPatcherCreateVertexProgram(
        patcher,
        vert_id,
        attrs.as_ptr(),
        1,
        streams.as_ptr(),
        1,
        &mut prog,
    );
    assert_eq!(rc, SCE_OK, "CreateVertexProgram({}) failed: 0x{:08X}", name, rc);
    prog
}

pub(super) unsafe fn find_param(prog: *const SceGxmProgram, name: &[u8]) -> *const SceGxmProgramParameter {
    sceGxmProgramFindParameterByName(prog, name.as_ptr())
}

pub(super) unsafe fn register_and_link_shaders(patcher: *mut SceGxmShaderPatcher) -> LinkedGxmShaders {
    let mut shaders = LinkedGxmShaders::empty();

    let (vox_vgxp, vox_v_id) = register_gxp_program(patcher, OUT_VOXEL_VERT_GXP, "voxel.vert");
    let (op_fgxp, op_f_id) = register_gxp_program(patcher, OUT_VOXEL_OPAQUE_FRAG_GXP, "voxel_opaque.frag");
    let (cut_fgxp, cut_f_id) = register_gxp_program(patcher, OUT_VOXEL_CUTOUT_FRAG_GXP, "voxel_cutout.frag");
    let (tr_fgxp, tr_f_id) = register_gxp_program(patcher, OUT_VOXEL_TRANS_FRAG_GXP, "voxel_trans.frag");

    let vox_v_hdr = vox_vgxp.as_ptr() as *const SceGxmProgram;
    let op_f_hdr = op_fgxp.as_ptr() as *const SceGxmProgram;
    let cut_f_hdr = cut_fgxp.as_ptr() as *const SceGxmProgram;
    let tr_f_hdr = tr_fgxp.as_ptr() as *const SceGxmProgram;

    let reg_pos = reg_index_or(vox_v_hdr, b"aPosition\0", 0);
    let reg_uv = reg_index_or(vox_v_hdr, b"aUV\0", 1);
    let reg_light = reg_index_or(vox_v_hdr, b"aLight\0", 2);

    let vox_attrs = [
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
            format: SCE_GXM_ATTRIBUTE_FORMAT_U8N as u8,
            componentCount: 4,
            regIndex: reg_light,
        },
    ];
    let vox_streams = [SceGxmVertexStream {
        stride: core::mem::size_of::<GxmVertex>() as u16,
        indexSource: SCE_GXM_INDEX_SOURCE_INDEX_16BIT,
    }];
    let rc = sceGxmShaderPatcherCreateVertexProgram(
        patcher,
        vox_v_id,
        vox_attrs.as_ptr(),
        vox_attrs.len() as u32,
        vox_streams.as_ptr(),
        1,
        &mut shaders.vert_prog,
    );
    assert_eq!(rc, SCE_OK, "CreateVertexProgram(voxel) failed: 0x{:08X}", rc);

    let alpha_blend = SceGxmBlendInfo::alpha_blend();
    shaders.frag_prog_opaque = create_frag_program(patcher, op_f_id, vox_v_hdr, None, "voxel_opaque");
    shaders.frag_prog_cutout = create_frag_program(patcher, cut_f_id, vox_v_hdr, None, "voxel_cutout");
    shaders.frag_prog_trans = create_frag_program(patcher, tr_f_id, vox_v_hdr, Some(&alpha_blend), "voxel_trans");

    shaders.param_voxel_mvp = find_param(vox_v_hdr, b"uViewProj\0");
    shaders.param_voxel_sky_daylight = find_param(vox_v_hdr, b"uSkyBotAndDaylight\0");
    shaders.param_voxel_chunk_offset = find_param(vox_v_hdr, b"uChunkOffset\0");

    shaders.param_opaque_cam_fog = find_param(op_f_hdr, b"uCameraPosAndFog\0");
    shaders.param_opaque_fog_color = find_param(op_f_hdr, b"uFogColor\0");
    shaders.param_cutout_cam_fog = find_param(cut_f_hdr, b"uCameraPosAndFog\0");
    shaders.param_cutout_fog_color = find_param(cut_f_hdr, b"uFogColor\0");
    shaders.param_trans_cam_fog = find_param(tr_f_hdr, b"uCameraPosAndFog\0");
    shaders.param_trans_fog_color = find_param(tr_f_hdr, b"uFogColor\0");

    shaders.aligned_gxps.push(vox_vgxp);
    shaders.aligned_gxps.push(op_fgxp);
    shaders.aligned_gxps.push(cut_fgxp);
    shaders.aligned_gxps.push(tr_fgxp);
    shaders.registered_ids.push(vox_v_id);
    shaders.registered_ids.push(op_f_id);
    shaders.registered_ids.push(cut_f_id);
    shaders.registered_ids.push(tr_f_id);

    link_fx_and_entity_shaders(patcher, &mut shaders);
    shaders
}
