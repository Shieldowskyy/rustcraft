//! Secondary GXM shader registration and linking for Sky, Entity, Post-FX, Biome Tint, HUD, and Clear passes.
//! Keeps shader linking helpers modular alongside `programs.rs`.

#![allow(dead_code)]

use super::ffi::*;
use super::programs::{
    create_frag_program, create_fullscreen_vert_program, find_param, register_gxp_program,
    LinkedGxmShaders, CLEAR_FRAG_GXP, CLEAR_VERT_GXP,
};
use super::types::GxmEntityVertex;

static OUT_SKY_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sky.vert.gxp"));
static OUT_SKY_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/sky.frag.gxp"));
static OUT_ENTITY_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/entity.vert.gxp"));
static OUT_ENTITY_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/entity.frag.gxp"));
static OUT_POSTFX_VERT_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/postfx.vert.gxp"));
static OUT_POSTFX_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/postfx.frag.gxp"));
static OUT_BIOME_TINT_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/biome_tint.frag.gxp"));
static OUT_HUD_FRAG_GXP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/hud.frag.gxp"));

pub(super) unsafe fn link_fx_and_entity_shaders(
    patcher: *mut SceGxmShaderPatcher,
    shaders: &mut LinkedGxmShaders,
) {
    // --- 1. Early-Z Procedural Sky ---
    let (sky_vgxp, sky_v_id) = register_gxp_program(patcher, OUT_SKY_VERT_GXP, "sky.vert");
    let (sky_fgxp, sky_f_id) = register_gxp_program(patcher, OUT_SKY_FRAG_GXP, "sky.frag");
    let sky_v_hdr = sky_vgxp.as_ptr() as *const SceGxmProgram;
    let sky_f_hdr = sky_fgxp.as_ptr() as *const SceGxmProgram;

    shaders.sky_vert_prog = create_fullscreen_vert_program(patcher, sky_v_id, sky_v_hdr, "sky.vert");
    shaders.sky_frag_prog = create_frag_program(patcher, sky_f_id, sky_v_hdr, None, "sky.frag");
    shaders.param_sky_inv_vp = find_param(sky_f_hdr, b"uInvViewProj\0");
    shaders.param_sky_cam_daylight = find_param(sky_f_hdr, b"uCameraPosAndDaylight\0");
    shaders.param_sky_top = find_param(sky_f_hdr, b"uSkyTop\0");
    shaders.param_sky_bot = find_param(sky_f_hdr, b"uSkyBot\0");
    shaders.param_sky_sun_dir = find_param(sky_f_hdr, b"uSunDir\0");
    shaders.param_sky_moon_dir = find_param(sky_f_hdr, b"uMoonDir\0");
    shaders.aligned_gxps.push(sky_vgxp);
    shaders.aligned_gxps.push(sky_fgxp);
    shaders.registered_ids.push(sky_v_id);
    shaders.registered_ids.push(sky_f_id);

    // --- 2. Dynamic Entity / Player / Cloud / Weather / Hand Pipeline ---
    let (ent_vgxp, ent_v_id) = register_gxp_program(patcher, OUT_ENTITY_VERT_GXP, "entity.vert");
    let (ent_fgxp, ent_f_id) = register_gxp_program(patcher, OUT_ENTITY_FRAG_GXP, "entity.frag");
    let ent_v_hdr = ent_vgxp.as_ptr() as *const SceGxmProgram;
    let ent_f_hdr = ent_fgxp.as_ptr() as *const SceGxmProgram;

    let reg_pos = reg_index_or(ent_v_hdr, b"aPosition\0", 0);
    let reg_uv = reg_index_or(ent_v_hdr, b"aUV\0", 1);
    let reg_col = reg_index_or(ent_v_hdr, b"aColor\0", 2);
    let reg_tint = reg_index_or(ent_v_hdr, b"aTint\0", 3);

    let ent_attrs = [
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
            componentCount: 4,
            regIndex: reg_col,
        },
        SceGxmVertexAttribute {
            streamIndex: 0,
            offset: 36,
            format: SCE_GXM_ATTRIBUTE_FORMAT_U8N as u8,
            componentCount: 4,
            regIndex: reg_tint,
        },
    ];
    let ent_streams = [SceGxmVertexStream {
        stride: core::mem::size_of::<GxmEntityVertex>() as u16,
        indexSource: SCE_GXM_INDEX_SOURCE_INDEX_16BIT,
    }];
    let rc = sceGxmShaderPatcherCreateVertexProgram(
        patcher,
        ent_v_id,
        ent_attrs.as_ptr(),
        ent_attrs.len() as u32,
        ent_streams.as_ptr(),
        1,
        &mut shaders.entity_vert_prog,
    );
    assert_eq!(rc, SCE_OK, "CreateVertexProgram(entity) failed: 0x{:08X}", rc);

    let alpha_blend = SceGxmBlendInfo::alpha_blend();
    shaders.entity_frag_opaque =
        create_frag_program(patcher, ent_f_id, ent_v_hdr, None, "entity_opaque");
    shaders.entity_frag_blend =
        create_frag_program(patcher, ent_f_id, ent_v_hdr, Some(&alpha_blend), "entity_blend");
    shaders.param_entity_mvp = find_param(ent_v_hdr, b"uViewProj\0");
    shaders.param_entity_cam_fog = find_param(ent_f_hdr, b"uCameraPosAndFog\0");
    shaders.param_entity_fog_color = find_param(ent_f_hdr, b"uFogColor\0");
    shaders.aligned_gxps.push(ent_vgxp);
    shaders.aligned_gxps.push(ent_fgxp);
    shaders.registered_ids.push(ent_v_id);
    shaders.registered_ids.push(ent_f_id);

    // --- 3. Fullscreen Post-FX, Biome Tint & Premultiplied HUD Compositor ---
    let (pfx_vgxp, pfx_v_id) = register_gxp_program(patcher, OUT_POSTFX_VERT_GXP, "postfx.vert");
    let (pfx_fgxp, pfx_f_id) = register_gxp_program(patcher, OUT_POSTFX_FRAG_GXP, "postfx.frag");
    let (bio_fgxp, bio_f_id) = register_gxp_program(patcher, OUT_BIOME_TINT_FRAG_GXP, "biome_tint.frag");
    let (hud_fgxp, hud_f_id) = register_gxp_program(patcher, OUT_HUD_FRAG_GXP, "hud.frag");

    let pfx_v_hdr = pfx_vgxp.as_ptr() as *const SceGxmProgram;
    let pfx_f_hdr = pfx_fgxp.as_ptr() as *const SceGxmProgram;
    let bio_f_hdr = bio_fgxp.as_ptr() as *const SceGxmProgram;

    let premul_blend = SceGxmBlendInfo::premultiplied_alpha();
    let mul_blend = SceGxmBlendInfo::multiply();
    let mul_add_blend = SceGxmBlendInfo::multiply_add();

    shaders.postfx_vert_prog =
        create_fullscreen_vert_program(patcher, pfx_v_id, pfx_v_hdr, "postfx.vert");
    shaders.postfx_frag_prog =
        create_frag_program(patcher, pfx_f_id, pfx_v_hdr, Some(&premul_blend), "postfx.frag");
    shaders.param_postfx_p0 = find_param(pfx_f_hdr, b"uParams0\0");
    shaders.param_postfx_p1 = find_param(pfx_f_hdr, b"uParams1\0");

    shaders.biome_tint_mul_prog =
        create_frag_program(patcher, bio_f_id, pfx_v_hdr, Some(&mul_blend), "biome_mul");
    shaders.biome_tint_add_prog =
        create_frag_program(patcher, bio_f_id, pfx_v_hdr, Some(&mul_add_blend), "biome_add");
    shaders.param_biome_tint = find_param(bio_f_hdr, b"uBiomeTint\0");

    shaders.hud_frag_prog =
        create_frag_program(patcher, hud_f_id, pfx_v_hdr, Some(&premul_blend), "hud.frag");

    shaders.aligned_gxps.push(pfx_vgxp);
    shaders.aligned_gxps.push(pfx_fgxp);
    shaders.aligned_gxps.push(bio_fgxp);
    shaders.aligned_gxps.push(hud_fgxp);
    shaders.registered_ids.push(pfx_v_id);
    shaders.registered_ids.push(pfx_f_id);
    shaders.registered_ids.push(bio_f_id);
    shaders.registered_ids.push(hud_f_id);

    // --- 4. Hardware Fullscreen Clear & Depth-Only Clear (16-byte aligned static GXP) ---
    let clear_v_hdr = CLEAR_VERT_GXP.0.as_ptr() as *const SceGxmProgram;
    let clear_f_hdr = CLEAR_FRAG_GXP.0.as_ptr() as *const SceGxmProgram;
    let mut clear_v_id: SceGxmShaderPatcherId = core::ptr::null_mut();
    let mut clear_f_id: SceGxmShaderPatcherId = core::ptr::null_mut();
    let rc_cv = sceGxmShaderPatcherRegisterProgram(patcher, clear_v_hdr, &mut clear_v_id);
    assert_eq!(rc_cv, SCE_OK, "RegisterProgram(clear.vert) failed: 0x{:08X}", rc_cv);
    let rc_cf = sceGxmShaderPatcherRegisterProgram(patcher, clear_f_hdr, &mut clear_f_id);
    assert_eq!(rc_cf, SCE_OK, "RegisterProgram(clear.frag) failed: 0x{:08X}", rc_cf);

    let depth_only = SceGxmBlendInfo::depth_only_no_color();
    shaders.clear_vert_prog =
        create_fullscreen_vert_program(patcher, clear_v_id, clear_v_hdr, "clear.vert");
    shaders.clear_frag_prog =
        create_frag_program(patcher, clear_f_id, clear_v_hdr, None, "clear.frag");
    shaders.clear_frag_depth_only =
        create_frag_program(patcher, clear_f_id, clear_v_hdr, Some(&depth_only), "clear_depth");
    shaders.param_clear_color = find_param(clear_f_hdr, b"uClearColor\0");
    shaders.registered_ids.push(clear_v_id);
    shaders.registered_ids.push(clear_f_id);
}

#[inline]
pub(super) unsafe fn reg_index_or(
    prog: *const SceGxmProgram,
    name: &[u8],
    fallback: u16,
) -> u16 {
    let p = sceGxmProgramFindParameterByName(prog, name.as_ptr());
    if !p.is_null() {
        sceGxmProgramParameterGetResourceIndex(p) as u16
    } else {
        fallback
    }
}
