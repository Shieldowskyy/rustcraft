//! HUD, status bars, screen overlays, pause menu, death screen, and F3 diagnostic overlay facade.
//! Strictly isolated from platform-specific `#[cfg]` code.

mod debug;
mod hotbar;
mod icons;
mod overlays;
mod pause_death;

#[cfg(test)]
mod tests;

pub use debug::{
    draw_advanced_debug_overlay, draw_capture_prompt, draw_debug_panel, draw_multiplayer_toast,
    draw_pie_chart,
};
pub use hotbar::draw_hotbar;
pub use icons::{
    ArmorIconState, BubbleState, HeartIconState, HungerIconState, draw_armor_icon, draw_bubble,
    draw_heart_icon, draw_hunger_icon,
};
pub use overlays::{
    apply_biome_color_filter, draw_break_progress, draw_burning_fire_overlay, draw_crosshair,
    draw_damage_flash, draw_lava_overlay, draw_low_health_vignette, draw_underwater_overlay,
};
pub use pause_death::{
    death_screen_button_rects, draw_death_screen, draw_pause_menu, pause_menu_button_rects,
};
