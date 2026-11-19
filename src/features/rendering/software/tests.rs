//! Unit tests for software triangle orientation and shared PC/Vita rendering primitives.

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::player::ItemStack;
use crate::features::rendering::atlas::{
    TextureId, TextureRenderPass, BUILTIN_TEXTURE_COUNT, CUSTOM_TEXTURE_LAYERS,
};
use crate::features::rendering::clouds::compute_cloud_color;
use crate::features::rendering::{
    chunk_spawn_y_offset, compute_entity_light_rgb, emit_player_model_parts,
};

use super::clip::orient_screen_triangle;
use super::types::ScreenVertAO;

fn screen_vert(x: f32, y: f32) -> ScreenVertAO {
    ScreenVertAO {
        x,
        y,
        inv_w: 1.0,
        uv_over_w: (0.0, 0.0),
        ao_over_w: 1.0,
        sky_light_over_w: 1.0,
        block_light_over_w: 0.0,
    }
}

#[test]
fn water_triangles_are_visible_from_both_sides() {
    let a = screen_vert(0.0, 0.0);
    let b = screen_vert(1.0, 0.0);
    let c = screen_vert(0.0, 1.0);

    assert!(orient_screen_triangle(a, b, c, TextureId::Water).is_some());
    assert!(orient_screen_triangle(a, c, b, TextureId::Water).is_some());
    assert!(orient_screen_triangle(a, b, c, TextureId::Glass).is_none());
}

#[test]
fn texture_id_layer_roundtrip_and_render_passes() {
    let total = BUILTIN_TEXTURE_COUNT as u32 + CUSTOM_TEXTURE_LAYERS;
    assert_eq!(total, 256);
    for layer in 0..total {
        assert_eq!(TextureId::from_layer(layer).layer(), layer);
    }

    assert_eq!(TextureId::Water.render_pass(), TextureRenderPass::Transparent);
    assert_eq!(TextureId::Glass.render_pass(), TextureRenderPass::Transparent);
    assert_eq!(TextureId::Leaves.render_pass(), TextureRenderPass::Cutout);
    assert_eq!(TextureId::Torch.render_pass(), TextureRenderPass::Cutout);
    assert_eq!(TextureId::Kelp.render_pass(), TextureRenderPass::Cutout);
    assert_eq!(TextureId::Seagrass.render_pass(), TextureRenderPass::Cutout);
    assert_eq!(TextureId::Stone.render_pass(), TextureRenderPass::Opaque);
    assert_eq!(TextureId::Dirt.render_pass(), TextureRenderPass::Opaque);
    assert_eq!(TextureId::Planks.render_pass(), TextureRenderPass::Opaque);
}

#[test]
fn shared_cloud_color_day_night_and_storm_curves() {
    let day = compute_cloud_color(1.0, [0.7, 0.8, 0.9], 0.0, 0.0);
    let night = compute_cloud_color(0.0, [0.05, 0.06, 0.12], 0.0, 0.0);
    let storm = compute_cloud_color(1.0, [0.5, 0.55, 0.6], 1.0, 1.0);

    assert!(day[0] > night[0] && day[1] > night[1] && day[2] > night[2]);
    assert!(storm[0] < day[0] && storm[1] < day[1] && storm[2] < day[2]);
}

#[test]
fn shared_entity_light_and_chunk_spawn_curves() {
    let min_lit = compute_entity_light_rgb(0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(min_lit, [0.04, 0.04, 0.04]);

    let bright = compute_entity_light_rgb(1.0, 1.0, 1.0, 1.0, 1.0);
    assert_eq!(bright, [1.45, 1.45, 1.45]);

    let torch = compute_entity_light_rgb(1.0, 0.0, 0.7, 0.0, 0.0);
    assert!(torch.iter().all(|&c| (0.04..=1.45).contains(&c)));
    assert!(torch[0] > torch[1] && torch[1] > torch[2]);

    assert_eq!(chunk_spawn_y_offset(false, 0.1, 0.0), 0.0);
    assert!((chunk_spawn_y_offset(true, 0.0, 0.0) - (-14.0)).abs() < 1e-4);
    assert_eq!(chunk_spawn_y_offset(true, 0.35, 0.0), 0.0);
    let mid = chunk_spawn_y_offset(true, 0.175, 0.0);
    assert!(mid > -14.0 && mid < 0.0);
}

#[test]
fn shared_player_model_parts_and_held_pose() {
    let mut bare_parts = Vec::new();
    let none_held = emit_player_model_parts(
        Vec3::zero(),
        0.0,
        0.0,
        0.0,
        0.0,
        false,
        Some(BlockId::Air),
        &[None, None, None, None],
        |p| bare_parts.push(p),
    );
    assert_eq!(bare_parts.len(), 6);
    assert!(none_held.is_none());

    let full_armor = [
        Some(ItemStack::new(BlockId::DiamondHelmet, 1)),
        Some(ItemStack::new(BlockId::DiamondChestplate, 1)),
        Some(ItemStack::new(BlockId::DiamondLeggings, 1)),
        Some(ItemStack::new(BlockId::DiamondBoots, 1)),
    ];
    let mut armored_parts = Vec::new();
    let sword_held = emit_player_model_parts(
        Vec3::zero(),
        0.5,
        0.1,
        1.0,
        0.5,
        true,
        Some(BlockId::DiamondSword),
        &full_armor,
        |p| armored_parts.push(p),
    );
    assert_eq!(armored_parts.len(), 15);
    assert_eq!(sword_held.map(|h| h.block), Some(BlockId::DiamondSword));
}
