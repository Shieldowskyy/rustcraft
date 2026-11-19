//! Unit tests for HUD icons, underwater hotbar bubbles, and biome color tint filters.

use crate::features::player::{GameMode, Player};
use crate::features::world::Biome;

use super::{
    BubbleState, HeartIconState, HungerIconState, apply_biome_color_filter, draw_bubble,
    draw_heart_icon, draw_hotbar, draw_hunger_icon,
};

#[test]
fn test_draw_bubble_renders_pixels() {
    let width = 32;
    let height = 32;
    let mut buffer = vec![0u32; width * height];
    draw_bubble(&mut buffer, width, height, 4, 4, BubbleState::Full);
    let filled_count = buffer.iter().filter(|&&p| p != 0).count();
    assert!(filled_count > 0, "Full bubble should write non-zero pixels");

    let mut buffer_pop = vec![0u32; width * height];
    draw_bubble(&mut buffer_pop, width, height, 4, 4, BubbleState::Pop);
    let pop_count = buffer_pop.iter().filter(|&&p| p != 0).count();
    assert!(pop_count > 0, "Pop bubble should write non-zero pixels");
}

#[test]
fn test_draw_hotbar_with_underwater_bubbles() {
    let width = 320;
    let height = 240;
    let mut buffer_surface = vec![0u32; width * height];
    let mut buffer_underwater = vec![0u32; width * height];

    let mut player = Player::new();
    player.gamemode = GameMode::Survival;
    player.head_underwater = false;
    player.air = 20.0;
    draw_hotbar(&mut buffer_surface, width, height, &player);

    player.head_underwater = true;
    player.air = 15.0;
    draw_hotbar(&mut buffer_underwater, width, height, &player);

    let count_surf = buffer_surface.iter().filter(|&&p| p != 0).count();
    let count_under = buffer_underwater.iter().filter(|&&p| p != 0).count();
    assert!(
        count_under > count_surf,
        "Underwater HUD should render bubble pixels above hearts"
    );
}

#[test]
fn test_draw_hunger_icon_renders_pixels() {
    let width = 32;
    let height = 32;
    let mut buffer_full = vec![0u32; width * height];
    draw_hunger_icon(&mut buffer_full, width, height, 4, 4, HungerIconState::Full);
    assert!(buffer_full.iter().filter(|&&p| p != 0).count() > 0);

    let mut buffer_half = vec![0u32; width * height];
    draw_hunger_icon(&mut buffer_half, width, height, 4, 4, HungerIconState::Half);
    assert!(buffer_half.iter().filter(|&&p| p != 0).count() > 0);

    let mut buffer_empty = vec![0u32; width * height];
    draw_hunger_icon(&mut buffer_empty, width, height, 4, 4, HungerIconState::Empty);
    assert!(buffer_empty.iter().filter(|&&p| p != 0).count() > 0);
}

#[test]
fn test_draw_heart_icon_renders_pixels() {
    let width = 32;
    let height = 32;
    let mut buffer_full = vec![0u32; width * height];
    draw_heart_icon(&mut buffer_full, width, height, 4, 4, HeartIconState::Full, false);
    assert!(buffer_full.iter().filter(|&&p| p != 0).count() > 0);

    let mut buffer_half = vec![0u32; width * height];
    draw_heart_icon(&mut buffer_half, width, height, 4, 4, HeartIconState::Half, false);
    assert!(buffer_half.iter().filter(|&&p| p != 0).count() > 0);

    let mut buffer_empty = vec![0u32; width * height];
    draw_heart_icon(&mut buffer_empty, width, height, 4, 4, HeartIconState::Empty, false);
    assert!(buffer_empty.iter().filter(|&&p| p != 0).count() > 0);

    let mut buffer_flash = vec![0u32; width * height];
    draw_heart_icon(&mut buffer_flash, width, height, 4, 4, HeartIconState::Full, true);
    assert!(buffer_flash.iter().filter(|&&p| p != 0).count() > 0);
}

#[test]
fn test_apply_biome_color_filter_neutral_noop() {
    let mut buffer = vec![0xFF808080u32; 16];
    let original = buffer.clone();
    apply_biome_color_filter(&mut buffer, [1.0, 1.0, 1.0]);
    assert_eq!(buffer, original, "Neutral filter [1.0, 1.0, 1.0] must not modify pixel buffer");
}

#[test]
fn test_apply_biome_color_filter_desert_warmth() {
    let mut buffer = vec![0xFF808080u32; 16];
    let filter = Biome::Desert.color_filter();
    apply_biome_color_filter(&mut buffer, filter);

    let p = buffer[0];
    let r = (p >> 16) & 0xFF;
    let g = (p >> 8) & 0xFF;
    let b = p & 0xFF;
    let a = (p >> 24) & 0xFF;

    assert_eq!(a, 0xFF);
    assert!(r > 128);
    assert!(b < 128);
    assert!(r > g && g > b);
}

#[test]
fn test_apply_biome_color_filter_snow_cool() {
    let mut buffer = vec![0xFF808080u32; 16];
    let filter = Biome::SnowyTundra.color_filter();
    apply_biome_color_filter(&mut buffer, filter);

    let p = buffer[0];
    let r = (p >> 16) & 0xFF;
    let b = p & 0xFF;
    let a = (p >> 24) & 0xFF;

    assert_eq!(a, 0xFF);
    assert!(b > 128);
    assert!(r < 128);
}
