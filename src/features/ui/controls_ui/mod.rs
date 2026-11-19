//! Fasada menu przypisywania klawiszy, przycisków myszy oraz kontrolera (rebind UI).

mod draw;
mod layout;

pub use draw::draw_controls_rebind_menu;
pub use layout::{
    ControlsRebindLayout, RebindSlot, VISIBLE_REBIND_ROWS, filter_actions, normalize_search,
};
