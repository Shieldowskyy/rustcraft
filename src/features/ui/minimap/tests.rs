//! Testy jednostkowe trybów minimapy, serializacji waypointów oraz geometrii przycisków mapy świata.

use super::*;
use crate::core::math::Vec3;

#[test]
fn test_minimap_mode_cycling() {
    let mode = MinimapMode::Circle;
    assert_eq!(mode.next(), MinimapMode::Square);
    assert_eq!(mode.next().next(), MinimapMode::Hidden);
    assert_eq!(mode.next().next().next(), MinimapMode::Circle);
}

#[test]
fn test_waypoint_dialog_button_bounds() {
    let (save, cancel) = waypoint_dialog_buttons(100, 100, 320, 180);
    assert!(save.0 >= 100);
    assert!(cancel.0 > save.0);
    assert_eq!(save.2, 110);
    assert_eq!(cancel.2, 110);
}

#[test]
fn test_waypoint_serialization_roundtrip() {
    let original = vec![
        Waypoint {
            name: "Base".to_string(),
            x: 10,
            y: 64,
            z: -25,
            color: 0xFFFF3333,
            enabled: true,
        },
        Waypoint {
            name: "Iron Mine".to_string(),
            x: 120,
            y: 32,
            z: 400,
            color: 0xFF33FF33,
            enabled: false,
        },
    ];

    let serialized = serialize_waypoints(&original);
    let parsed = parse_waypoints_str(&serialized);
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].name, "Base");
    assert_eq!(parsed[0].x, 10);
    assert_eq!(parsed[0].y, 64);
    assert_eq!(parsed[0].z, -25);
    assert_eq!(parsed[0].color, 0xFFFF3333);
    assert!(parsed[0].enabled);

    assert_eq!(parsed[1].name, "Iron Mine");
    assert_eq!(parsed[1].x, 120);
    assert_eq!(parsed[1].y, 32);
    assert_eq!(parsed[1].z, 400);
    assert_eq!(parsed[1].color, 0xFF33FF33);
    assert!(!parsed[1].enabled);
}

#[test]
fn test_worldmap_top_bar_buttons() {
    let (wp, new_wp, clear) = worldmap_top_bar_buttons();
    assert_eq!(wp.1, 4);
    assert_eq!(wp.3, 20);
    assert!(new_wp.0 > wp.0);
    assert!(clear.0 > new_wp.0);
}

#[test]
fn test_waypoint_manager_buttons() {
    let (header_close, new_wp, bottom_close, scroll_up, scroll_down) =
        waypoint_manager_buttons(100, 100, 540, 360);
    assert!(header_close.0 > 100);
    assert_eq!(new_wp.1, 100 + 360 - 36);
    assert!(bottom_close.0 > new_wp.0);
    assert!(scroll_down.1 > scroll_up.1);
}

#[test]
fn test_waypoint_row_buttons() {
    let (vis, tp, del) = waypoint_row_buttons(100, 150, 540, 0);
    assert!(tp.0 > vis.0);
    assert!(del.0 > tp.0);
}

#[test]
fn test_worldmap_state_selection_and_cache() {
    let mut state = WorldMapState::new(Vec3::new(0.0, 64.0, 0.0));
    assert!(state.selected_chunks.is_empty());
    assert!(!state.cache_valid);

    state.selected_chunks.insert((2, -3));
    assert!(state.selected_chunks.contains(&(2, -3)));
    assert_eq!(state.selected_chunks.len(), 1);

    state.selected_chunks.remove(&(2, -3));
    assert!(state.selected_chunks.is_empty());

    state.cache_valid = true;
    state.invalidate_cache();
    assert!(!state.cache_valid);
}

#[test]
fn test_worldmap_context_menu_rects() {
    let (card, item0, item1) = worldmap_context_menu_rects(50, 50, 800, 600);
    assert_eq!(card.0, 50);
    assert_eq!(card.1, 50);
    assert!(card.2 >= 180);
    assert!(card.3 >= 50);

    assert!(item0.0 >= card.0 && item0.0 + item0.2 <= card.0 + card.2);
    assert!(item1.0 >= card.0 && item1.0 + item1.2 <= card.0 + card.2);
    assert!(item1.1 > item0.1);

    let (card_edge, _, _) = worldmap_context_menu_rects(795, 595, 800, 600);
    assert!(card_edge.0 + card_edge.2 <= 796);
    assert!(card_edge.1 + card_edge.3 <= 596);
}
