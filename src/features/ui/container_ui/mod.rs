//! Fasada interfejsów kontenerów (skrzynia, piec) oraz handlu z osadnikami.

mod chest_furnace;
mod villager_trade;

pub use chest_furnace::{
    ChestLayout, ChestSlotTarget, FurnaceLayout, FurnaceSlotTarget, SLOT_GAP, SLOT_SIZE, SLOT_STEP,
    draw_chest_ui, draw_furnace_ui, draw_slot,
};
pub use villager_trade::{VillagerTradeLayout, VillagerTradeSlotTarget, draw_villager_trade_ui};
