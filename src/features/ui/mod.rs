//! User interface domain facade (primitives, widgets, HUD, menus, inventory, minimap, containers, settings, and chat).
//! Strictly isolated from platform-specific `#[cfg]` code.

pub mod achievements_ui;
pub mod chat;
pub mod container_ui;
pub mod controls_ui;
pub mod create_world;
pub mod dev_logo;
pub mod draw;
pub mod hud;
pub mod inventory;
pub mod menu;
pub mod minimap;
pub mod multiplayer_menu;
pub mod nickname_ui;
pub mod player_list_ui;
pub mod quests_ui;
pub mod save_select;
pub mod settings_ui;
pub mod splashes;
pub mod widgets;

pub use achievements_ui::{
    block_catalog_entry_count, block_catalog_page_size, draw_achievement_toast,
    draw_achievement_tree, draw_field_guide, field_guide_tab_at, field_guide_tab_rects,
};
pub use chat::{
    ChatAction, ChatMessage, ChatState, CommandContext, parse_block_id, parse_vanilla_block_id,
};
pub use container_ui::{
    ChestLayout, ChestSlotTarget, FurnaceLayout, FurnaceSlotTarget, SLOT_GAP, SLOT_SIZE, SLOT_STEP,
    VillagerTradeLayout, VillagerTradeSlotTarget, draw_chest_ui, draw_furnace_ui, draw_slot,
    draw_villager_trade_ui,
};
pub use controls_ui::{
    ControlsRebindLayout, RebindSlot, VISIBLE_REBIND_ROWS, draw_controls_rebind_menu,
    filter_actions, normalize_search,
};
pub use create_world::{create_world_rects, draw_create_world_menu};
pub use draw::{
    draw_cursor_pointer, draw_text_colored_shadowed, draw_text_light_colored_shadowed,
    draw_text_light_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color, set_px,
    unpack_color,
};
pub use hud::{
    ArmorIconState, BubbleState, HeartIconState, HungerIconState, apply_biome_color_filter,
    death_screen_button_rects, draw_advanced_debug_overlay, draw_armor_icon, draw_break_progress,
    draw_bubble, draw_burning_fire_overlay, draw_capture_prompt, draw_crosshair, draw_damage_flash,
    draw_death_screen, draw_debug_panel, draw_heart_icon, draw_hotbar, draw_hunger_icon,
    draw_lava_overlay, draw_low_health_vignette, draw_multiplayer_toast, draw_pause_menu,
    draw_pie_chart, draw_underwater_overlay, pause_menu_button_rects,
};
pub use inventory::{
    CREATIVE_TABS, InventoryLayout, InventorySlotTarget, creative_tab_blocks,
    draw_armor_and_offhand_slots, draw_inventory_screen, draw_item_icon,
    draw_player_inventory_preview, draw_trash_icon,
};
pub use menu::{draw_main_menu, draw_rustcraft_logo, main_menu_button_rects};
pub use minimap::{
    MinimapMode, MinimapState, WAYPOINT_COLORS, Waypoint, WorldMapContextMenu, WorldMapState,
    draw_clipped_rect_outline, draw_minimap, draw_waypoint_manager, draw_world_waypoint_markers,
    draw_worldmap, fast_calculate_map_color, load_waypoints, parse_waypoints_str,
    sample_chunk_column, sample_chunk_column_y, sample_surface, save_waypoints,
    serialize_waypoints, waypoint_dialog_buttons, waypoint_manager_buttons, waypoint_row_buttons,
    worldmap_context_menu_rects, worldmap_top_bar_buttons,
};
pub use multiplayer_menu::{
    MAX_LAN_SERVERS_DISPLAY, MULTIPLAYER_RECT_COUNT, draw_multiplayer_menu, multiplayer_menu_rects,
};
pub use nickname_ui::{draw_nickname_dialog, nickname_dialog_rects};
pub use player_list_ui::draw_player_list_tab;
pub use quests_ui::{
    QUESTBOOK_VISIBLE_CARDS, draw_quest_toast, draw_questbook, questbook_card_at,
    questbook_claim_hit, questbook_close_hit, questbook_tab_at,
};
pub use save_select::{
    MAX_SAVE_SLOTS, VISIBLE_SAVE_ROWS, draw_save_select_menu, save_bottom_button_rects,
    save_card_rects,
};
pub use settings_ui::{
    draw_settings_menu, settings_audio_rects, settings_controls_rects, settings_debug_rects,
    settings_graphics_rects, settings_option_rects, settings_tab_rects,
};
pub use widgets::{Button, ButtonVariant, Grid, HStack, Rect, TextInput, Toast, UiCanvas, VStack};
