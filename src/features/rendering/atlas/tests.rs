//! Unit tests for procedural atlas textures and 256-layer WGPU texture array invariants.

use super::blocks_containers::ladder;
use super::{TextureAtlas, TextureId, BUILTIN_TEXTURE_COUNT, CUSTOM_TEXTURE_LAYERS, TEX_SIZE};

#[test]
fn ladder_texture_background_is_transparent() {
    let texture = ladder();

    assert!(texture.sample(0.0, 0.0)[0] < 0.0);
    assert!(texture.sample(0.1875, 0.1875)[0] >= 0.0);
}

#[test]
fn atlas_layer_roundtrip_and_counts() {
    assert_eq!(BUILTIN_TEXTURE_COUNT as u32 + CUSTOM_TEXTURE_LAYERS, 256);
    let atlas = TextureAtlas::generate();
    assert_eq!(atlas.builtin.len(), BUILTIN_TEXTURE_COUNT);
    for tex in atlas.builtin.iter() {
        assert_eq!(tex.texels.len(), TEX_SIZE * TEX_SIZE);
    }
    for layer in 0..(BUILTIN_TEXTURE_COUNT as u32 + CUSTOM_TEXTURE_LAYERS) {
        let id = TextureId::from_layer(layer);
        assert_eq!(id.layer(), layer);
    }
}
