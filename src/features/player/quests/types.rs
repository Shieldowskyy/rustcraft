//! Questbook data model types (`QuestCategory`, `QuestObjective`, `QuestDef`, `QUEST_COUNT`).

use crate::features::blocks::BlockId;
use crate::features::entities::mob::MobKind;

pub const QUEST_COUNT: usize = 45;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum QuestCategory {
    Basics = 0,
    Mining = 1,
    Combat = 2,
    Farming = 3,
    Exploration = 4,
}

impl QuestCategory {
    pub const ALL: [Self; 5] = [
        Self::Basics,
        Self::Mining,
        Self::Combat,
        Self::Farming,
        Self::Exploration,
    ];

    pub fn title_key(self) -> &'static str {
        match self {
            Self::Basics => "questbook.cat.basics",
            Self::Mining => "questbook.cat.mining",
            Self::Combat => "questbook.cat.combat",
            Self::Farming => "questbook.cat.farming",
            Self::Exploration => "questbook.cat.exploration",
        }
    }

    pub fn accent_color(self) -> u32 {
        match self {
            Self::Basics => 0xFF66BB6A,
            Self::Mining => 0xFF42A5F5,
            Self::Combat => 0xFFEF5350,
            Self::Farming => 0xFFFFCA28,
            Self::Exploration => 0xFFAB47BC,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestObjective {
    MineBlock { block: BlockId, target: u32 },
    PlaceBlock { block: BlockId, target: u32 },
    CraftItem { item: BlockId, target: u32 },
    SmeltItem { item: BlockId, target: u32 },
    HaveItem { item: BlockId, target: u32 },
    KillHostile { target: u32 },
    KillSpecificMob { kind: MobKind, target: u32 },
    KillAnyMob { target: u32 },
    VisitBiomes { target: u32 },
    TradeVillager { target: u32 },
    TotalMined { target: u32 },
    TotalPlaced { target: u32 },
}

impl QuestObjective {
    pub fn target(self) -> u32 {
        match self {
            Self::MineBlock { target, .. }
            | Self::PlaceBlock { target, .. }
            | Self::CraftItem { target, .. }
            | Self::SmeltItem { target, .. }
            | Self::HaveItem { target, .. }
            | Self::KillHostile { target }
            | Self::KillSpecificMob { target, .. }
            | Self::KillAnyMob { target }
            | Self::VisitBiomes { target }
            | Self::TradeVillager { target }
            | Self::TotalMined { target }
            | Self::TotalPlaced { target } => target,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct QuestDef {
    pub id: &'static str,
    pub category: QuestCategory,
    pub icon: BlockId,
    pub objective: QuestObjective,
    pub prerequisite: Option<usize>,
    pub rewards: &'static [(BlockId, u8)],
}

impl QuestDef {
    pub fn title_key(&self) -> String {
        format!("quest.{}.title", self.id)
    }

    pub fn desc_key(&self) -> String {
        format!("quest.{}.desc", self.id)
    }
}
