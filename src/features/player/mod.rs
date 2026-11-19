//! Player state domain facade (`Player`, `Camera`, `Inventory`, `ItemStack`, `Trading`, `Achievements`, `Quests`, `Recipes`).
//! Strictly isolated from platform-specific `#[cfg]` code.

pub mod achievements;
pub mod camera;
pub mod inventory;
pub mod quests;
pub mod recipes;
pub mod state;
pub mod trading;

#[cfg(test)]
mod tests_state;

pub use achievements::{AchievementId, AchievementProgress};
pub use camera::Camera;
pub use inventory::{Inventory, ItemStack};
pub use quests::{
    quests_in_category, QuestCategory, QuestDef, QuestObjective, QuestProgress, QUESTS,
    QUEST_COUNT,
};
pub use recipes::{Recipe, RecipeBook};
pub use state::{GameMode, Player, SavedPlayerData};
pub use trading::{
    consume_item_from_player, count_item_in_player, TradeOffer, VillagerProfession,
};
