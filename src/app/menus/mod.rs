pub mod controls;
pub mod main_menu;
pub mod pause;
pub mod saves;
pub mod settings;
pub mod worldmap;

/// Helper to find the index of the first rectangle containing the mouse position.
#[inline]
pub(crate) fn find_hovered_rect(
    mouse_pos: Option<(f32, f32)>,
    rects: &[(i32, i32, i32, i32)],
) -> Option<usize> {
    let (mx, my) = mouse_pos?;
    let px = mx as i32;
    let py = my as i32;
    rects.iter().enumerate().find_map(|(i, &(bx, by, bw, bh))| {
        if px >= bx && px < bx + bw && py >= by && py < by + bh {
            Some(i)
        } else {
            None
        }
    })
}

/// Helper to check if a 2D integer point lies within a bounding box `(x, y, w, h)`.
#[inline]
pub(crate) fn point_in_rect(px: i32, py: i32, rect: (i32, i32, i32, i32)) -> bool {
    let (rx, ry, rw, rh) = rect;
    px >= rx && px < rx + rw && py >= ry && py < ry + rh
}
