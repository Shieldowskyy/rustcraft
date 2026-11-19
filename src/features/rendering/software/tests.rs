//! Unit tests for software triangle orientation and water double-sided culling.

use crate::features::rendering::atlas::TextureId;

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
