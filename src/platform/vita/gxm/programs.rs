//! 16-byte aligned GXP shader bytecode, shader patcher callbacks, and GXM program linking.
//! Registers voxel and clear Cg->GXP programs and creates opaque/transparent GPU pipelines.

#![allow(dead_code)]

use super::ffi::*;
use super::types::GxmVertex;

// ============================================================
// 4. SHADERY GXP BYTECODE (16-BYTE ALIGNED FOR ARMV7 / GXM)
// ============================================================
// DLACZEGO: ARMv7 and sceGxmProgramCheck require SceGxmProgram headers to be
// at least 4-byte/16-byte aligned in memory. Raw `include_bytes!` or `&[u8]`
// only has 1-byte alignment and causes a Data Abort crash on PS Vita!

#[repr(C, align(16))]
struct AlignedGxp<const N: usize>(pub [u8; N]);

/// Copies any slice of GXP bytecode into a heap-allocated, 16-byte aligned buffer
/// that remains valid for the lifetime of the shader patcher.
fn alloc_aligned_gxp(bytes: &[u8]) -> Vec<u64> {
    let u64_len = (bytes.len() + 7) / 8;
    let mut aligned = vec![0u64; u64_len];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), aligned.as_mut_ptr() as *mut u8, bytes.len());
    }
    aligned
}

static OUT_VOXEL_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.vert.gxp"));
static OUT_VOXEL_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/voxel.frag.gxp"));
static PREBUILT_VOXEL_VERT_GXP: &[u8] = include_bytes!("../../../../vita/shaders/prebuilt/voxel.vert.gxp");
static PREBUILT_VOXEL_FRAG_GXP: &[u8] = include_bytes!("../../../../vita/shaders/prebuilt/voxel.frag.gxp");

// Precompiled hardware clear shaders (standard SDL2 GXM clear pipeline)
static CLEAR_VERT_GXP: AlignedGxp<252> = AlignedGxp([
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
]);

static CLEAR_FRAG_GXP: AlignedGxp<232> = AlignedGxp([
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
]);


pub(super) unsafe extern "C" fn patcher_host_alloc(_: *mut core::ffi::c_void, size: u32) -> *mut core::ffi::c_void {
    malloc(size as usize)
}

pub(super) unsafe extern "C" fn patcher_host_free(_: *mut core::ffi::c_void, ptr: *mut core::ffi::c_void) {
    free(ptr);
}

pub(super) struct LinkedGxmShaders {
    pub(super) aligned_voxel_vert_gxp: Vec<u64>,
    pub(super) aligned_voxel_frag_gxp: Vec<u64>,
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
}

pub(super) unsafe fn register_and_link_shaders(patcher: *mut SceGxmShaderPatcher) -> LinkedGxmShaders {
        // --- 8. Register and Link GXP Shaders (Using 8-byte aligned buffers!) ---
        let vert_bytes = if OUT_VOXEL_VERT_GXP.len() > 64 { OUT_VOXEL_VERT_GXP } else { PREBUILT_VOXEL_VERT_GXP };
        let aligned_voxel_vert_gxp = alloc_aligned_gxp(vert_bytes);
        let frag_bytes = if OUT_VOXEL_FRAG_GXP.len() > 64 { OUT_VOXEL_FRAG_GXP } else { PREBUILT_VOXEL_FRAG_GXP };
        let aligned_voxel_frag_gxp = alloc_aligned_gxp(frag_bytes);

        let vert_gxp = aligned_voxel_vert_gxp.as_ptr() as *const SceGxmProgram;
        let frag_gxp = aligned_voxel_frag_gxp.as_ptr() as *const SceGxmProgram;

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

        // Opaque / Cutout fragment program (no blending -> enables PowerVR SGX543 HSR / Early-Z)
        let mut frag_prog_opaque: *mut SceGxmFragmentProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateFragmentProgram(
            patcher,
            frag_prog_id,
            SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4,
            SCE_GXM_MULTISAMPLE_NONE,
            core::ptr::null(),
            vert_gxp,
            &mut frag_prog_opaque,
        );
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreateFragmentProgram(opaque) failed: 0x{:08X}", rc);

        // Transparent fragment program (alpha blending for water, glass, entities & overlays)
        let blend_info = SceGxmBlendInfo::alpha_blend();
        let mut frag_prog_trans: *mut SceGxmFragmentProgram = core::ptr::null_mut();
        let rc = sceGxmShaderPatcherCreateFragmentProgram(
            patcher,
            frag_prog_id,
            SCE_GXM_OUTPUT_REGISTER_FORMAT_UCHAR4,
            SCE_GXM_MULTISAMPLE_NONE,
            &blend_info as *const _ as *const core::ffi::c_void,
            vert_gxp,
            &mut frag_prog_trans,
        );
        assert_eq!(rc, SCE_OK, "sceGxmShaderPatcherCreateFragmentProgram(trans) failed: 0x{:08X}", rc);

        let param_mvp = sceGxmProgramFindParameterByName(vert_gxp, b"uViewProj\0".as_ptr());
        let param_fog = sceGxmProgramFindParameterByName(vert_gxp, b"uCameraPosAndFog\0".as_ptr());
        let param_fog_color = sceGxmProgramFindParameterByName(frag_gxp, b"uFogColor\0".as_ptr());

        // --- 8b. Register and Link Hardware Clear Shaders (16-byte aligned static) ---
        let clear_vert_gxp = CLEAR_VERT_GXP.0.as_ptr() as *const SceGxmProgram;
        let clear_frag_gxp = CLEAR_FRAG_GXP.0.as_ptr() as *const SceGxmProgram;

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

    LinkedGxmShaders {
        aligned_voxel_vert_gxp,
        aligned_voxel_frag_gxp,
        vert_prog_id,
        frag_prog_id,
        clear_vert_id,
        clear_frag_id,
        vert_prog,
        frag_prog_opaque,
        frag_prog_trans,
        param_mvp,
        param_fog,
        param_fog_color,
        clear_vert_prog,
        clear_frag_prog,
        param_clear_color,
    }
}
