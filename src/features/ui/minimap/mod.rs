//! Fasada minimapy, pełnoekranowej mapy świata oraz systemu znaczników (waypoints).
//! Całkowicie wolna od atrybutów `#[cfg]`.

mod radar;
mod storage;
#[cfg(test)]
mod tests;
mod types;
mod waypoints_ui;
mod worldmap;

pub use radar::{draw_clipped_rect_outline, draw_minimap};
pub use storage::{
    fast_calculate_map_color, load_waypoints, parse_waypoints_str, sample_chunk_column,
    sample_chunk_column_y, sample_surface, save_waypoints, serialize_waypoints,
};
pub use types::{
    MinimapMode, MinimapState, WAYPOINT_COLORS, Waypoint, WorldMapContextMenu, WorldMapState,
};
pub use waypoints_ui::{
    draw_waypoint_manager, draw_world_waypoint_markers, waypoint_dialog_buttons,
    waypoint_manager_buttons, waypoint_row_buttons, worldmap_context_menu_rects,
    worldmap_top_bar_buttons,
};
pub use worldmap::draw_worldmap;
