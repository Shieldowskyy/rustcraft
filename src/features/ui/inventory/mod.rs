//! Fasada interfejsu ekwipunku, podglądu postaci, układu slotów i proceduralnych ikon przedmiotów.

pub mod icons;
mod layout;
mod preview;
mod screen;
#[cfg(test)]
mod tests;

pub use icons::draw_item_icon;
pub use layout::{creative_tab_blocks, InventoryLayout, InventorySlotTarget, CREATIVE_TABS};
pub use preview::{draw_armor_and_offhand_slots, draw_player_inventory_preview};
pub use screen::{draw_inventory_screen, draw_trash_icon};
