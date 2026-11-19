//! Unit tests for UI layout primitives and `UiCanvas` widget rendering.

use super::{Button, Grid, Rect, TextInput, UiCanvas, VStack};

#[test]
fn test_rect_geometry_and_layout_stacks() {
    let r = Rect::centered_in(800, 600, 400, 300);
    assert_eq!(r.to_tuple(), (200, 150, 400, 300));
    assert!(r.contains_point(250, 200));
    assert!(!r.contains_point(100, 200));
    assert!(r.contains_mouse(Some((399.0, 299.0))));

    let inset = r.inset(10);
    assert_eq!(inset.to_tuple(), (210, 160, 380, 280));

    let cols: [Rect; 4] = Rect::new(10, 20, 430, 30).split_cols(10);
    assert_eq!(cols[0].to_tuple(), (10, 20, 100, 30));
    assert_eq!(cols[1].to_tuple(), (120, 20, 100, 30));
    assert_eq!(cols[3].to_tuple(), (340, 20, 100, 30));

    let v_arr: [(i32, i32, i32, i32); 3] = VStack::centered(800, 100, 200, 10).array(40);
    assert_eq!(v_arr[0], (300, 100, 200, 40));
    assert_eq!(v_arr[1], (300, 150, 200, 40));
    assert_eq!(v_arr[2], (300, 200, 200, 40));

    let grid = Grid::square(20, 30, 9, 3, 38, 4);
    assert_eq!(grid.slot(0).to_tuple(), (20, 30, 38, 38));
    assert_eq!(grid.slot(10).to_tuple(), (62, 72, 38, 38));
    assert_eq!(grid.hit_test(25, 35), Some(0));
    assert_eq!(grid.hit_test(59, 35), None);
    assert_eq!(grid.hit_test(65, 75), Some(10));
    assert_eq!(grid.hit_test(500, 500), None);
}

#[test]
fn test_ui_canvas_widgets_render() {
    let (w, h) = (320, 240);
    let mut buf = vec![0u32; w * h];
    let mut ui = UiCanvas::new(&mut buf, w, h);

    ui.backdrop(Some(0xFF101010), 0xFF000000, 0.5);
    ui.button(
        Rect::new(20, 20, 120, 32),
        Button::new("Play").primary().hovered(true),
    );
    ui.text_input(
        Rect::new(20, 60, 160, 28),
        TextInput::new("World 1").focused(true).cursor(true),
    );
    ui.progress_bar(Rect::new(20, 100, 100, 6), 0.5, 0xFF222222, 0xFF00FF00);
    ui.scrollbar(Rect::new(200, 20, 6, 100), 4, 12, 2, 0xFF222222, 0xFF88AAFF);

    assert!(buf.iter().any(|&px| px != 0xFF101010));
}
