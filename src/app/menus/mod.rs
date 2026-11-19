pub mod controls;
pub mod create_world;
pub mod main_menu;
pub mod overlays;
pub mod pause;
pub mod saves;
pub mod settings;
pub mod worldmap;

use crate::ui::Rect;

/// Helper to find the index of the first rectangle containing the mouse position.
#[inline]
pub(crate) fn find_hovered_rect(
    mouse_pos: Option<(f32, f32)>,
    rects: &[(i32, i32, i32, i32)],
) -> Option<usize> {
    rects
        .iter()
        .position(|&r| Rect::from_tuple(r).contains_mouse(mouse_pos))
}

/// Helper to check if a 2D integer point lies within a bounding box `(x, y, w, h)`.
#[inline]
pub(crate) fn point_in_rect(px: i32, py: i32, rect: (i32, i32, i32, i32)) -> bool {
    Rect::from_tuple(rect).contains_point(px, py)
}
