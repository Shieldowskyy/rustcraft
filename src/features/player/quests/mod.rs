//! Player questbook domain facade (`QUESTS`, `QuestCategory`, `QuestDef`, `QuestObjective`, `QuestProgress`).
//! Tracks mining, placement, crafting, smelting, combat, exploration, and trade milestones.

mod catalog;
mod tracker;
mod types;

#[cfg(test)]
mod tests;

pub use catalog::{quests_in_category, QUESTS};
pub use tracker::QuestProgress;
pub use types::{QuestCategory, QuestDef, QuestObjective, QUEST_COUNT};
