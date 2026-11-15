use crate::gameplay::inventory::Inventory;
use crate::gameplay::mob::MobKind;
use crate::world::block::BlockId;

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

pub const QUESTS: [QuestDef; QUEST_COUNT] = [
    // --- 0..5: Basics ---
    QuestDef {
        id: "wood_gatherer",
        category: QuestCategory::Basics,
        icon: BlockId::Wood,
        objective: QuestObjective::MineBlock { block: BlockId::Wood, target: 8 },
        prerequisite: None,
        rewards: &[(BlockId::Planks, 8), (BlockId::Stick, 4)],
    },
    QuestDef {
        id: "workbench",
        category: QuestCategory::Basics,
        icon: BlockId::CraftingTable,
        objective: QuestObjective::CraftItem { item: BlockId::CraftingTable, target: 1 },
        prerequisite: Some(0),
        rewards: &[(BlockId::Torch, 4), (BlockId::Apple, 2)],
    },
    QuestDef {
        id: "stone_age",
        category: QuestCategory::Basics,
        icon: BlockId::StonePickaxe,
        objective: QuestObjective::MineBlock { block: BlockId::Stone, target: 16 },
        prerequisite: Some(1),
        rewards: &[(BlockId::StonePickaxe, 1), (BlockId::Torch, 4)],
    },
    QuestDef {
        id: "light_up",
        category: QuestCategory::Basics,
        icon: BlockId::Torch,
        objective: QuestObjective::CraftItem { item: BlockId::Torch, target: 8 },
        prerequisite: Some(1),
        rewards: &[(BlockId::CoalBlock, 1)],
    },
    QuestDef {
        id: "home_sweet_home",
        category: QuestCategory::Basics,
        icon: BlockId::Bed,
        objective: QuestObjective::PlaceBlock { block: BlockId::Bed, target: 1 },
        prerequisite: Some(2),
        rewards: &[(BlockId::Door, 1), (BlockId::Glass, 4)],
    },
    // --- 5..10: Mining ---
    QuestDef {
        id: "coal_miner",
        category: QuestCategory::Mining,
        icon: BlockId::CoalOre,
        objective: QuestObjective::MineBlock { block: BlockId::CoalOre, target: 12 },
        prerequisite: None,
        rewards: &[(BlockId::Furnace, 1), (BlockId::Torch, 8)],
    },
    QuestDef {
        id: "iron_vein",
        category: QuestCategory::Mining,
        icon: BlockId::IronOre,
        objective: QuestObjective::MineBlock { block: BlockId::IronOre, target: 12 },
        prerequisite: Some(5),
        rewards: &[(BlockId::IronIngot, 4)],
    },
    QuestDef {
        id: "smelt_iron",
        category: QuestCategory::Mining,
        icon: BlockId::IronIngot,
        objective: QuestObjective::SmeltItem { item: BlockId::IronIngot, target: 8 },
        prerequisite: Some(6),
        rewards: &[(BlockId::IronPickaxe, 1)],
    },
    QuestDef {
        id: "gold_rush",
        category: QuestCategory::Mining,
        icon: BlockId::GoldOre,
        objective: QuestObjective::MineBlock { block: BlockId::GoldOre, target: 6 },
        prerequisite: Some(7),
        rewards: &[(BlockId::GoldIngot, 2), (BlockId::Apple, 2)],
    },
    QuestDef {
        id: "diamond_hunter",
        category: QuestCategory::Mining,
        icon: BlockId::Diamond,
        objective: QuestObjective::MineBlock { block: BlockId::DiamondOre, target: 3 },
        prerequisite: Some(7),
        rewards: &[(BlockId::Diamond, 2), (BlockId::Obsidian, 2)],
    },
    // --- 10..15: Combat ---
    QuestDef {
        id: "first_blade",
        category: QuestCategory::Combat,
        icon: BlockId::StoneSword,
        objective: QuestObjective::HaveItem { item: BlockId::StoneSword, target: 1 },
        prerequisite: None,
        rewards: &[(BlockId::CookedBeef, 3)],
    },
    QuestDef {
        id: "night_watch",
        category: QuestCategory::Combat,
        icon: BlockId::IronSword,
        objective: QuestObjective::KillHostile { target: 5 },
        prerequisite: Some(10),
        rewards: &[(BlockId::Shield, 1)],
    },
    QuestDef {
        id: "bone_collector",
        category: QuestCategory::Combat,
        icon: BlockId::Bone,
        objective: QuestObjective::KillSpecificMob { kind: MobKind::Skeleton, target: 3 },
        prerequisite: Some(11),
        rewards: &[(BlockId::Bow, 1), (BlockId::Arrow, 16)],
    },
    QuestDef {
        id: "iron_armor",
        category: QuestCategory::Combat,
        icon: BlockId::IronChestplate,
        objective: QuestObjective::HaveItem { item: BlockId::IronChestplate, target: 1 },
        prerequisite: Some(11),
        rewards: &[(BlockId::IronHelmet, 1), (BlockId::IronBoots, 1)],
    },
    QuestDef {
        id: "apex_slayer",
        category: QuestCategory::Combat,
        icon: BlockId::DiamondSword,
        objective: QuestObjective::KillAnyMob { target: 15 },
        prerequisite: Some(12),
        rewards: &[(BlockId::DiamondSword, 1)],
    },
    // --- 15..20: Farming & Building ---
    QuestDef {
        id: "storage_room",
        category: QuestCategory::Farming,
        icon: BlockId::Chest,
        objective: QuestObjective::CraftItem { item: BlockId::Chest, target: 2 },
        prerequisite: None,
        rewards: &[(BlockId::Planks, 8), (BlockId::Torch, 4)],
    },
    QuestDef {
        id: "baker",
        category: QuestCategory::Farming,
        icon: BlockId::Bread,
        objective: QuestObjective::CraftItem { item: BlockId::Bread, target: 4 },
        prerequisite: Some(15),
        rewards: &[(BlockId::Carrot, 4), (BlockId::Potato, 4)],
    },
    QuestDef {
        id: "master_builder",
        category: QuestCategory::Farming,
        icon: BlockId::Bricks,
        objective: QuestObjective::TotalPlaced { target: 64 },
        prerequisite: Some(15),
        rewards: &[(BlockId::Bricks, 16), (BlockId::Glass, 8)],
    },
    QuestDef {
        id: "library",
        category: QuestCategory::Farming,
        icon: BlockId::Bookshelf,
        objective: QuestObjective::CraftItem { item: BlockId::Bookshelf, target: 3 },
        prerequisite: Some(17),
        rewards: &[(BlockId::Emerald, 2)],
    },
    QuestDef {
        id: "redstone_engineer",
        category: QuestCategory::Farming,
        icon: BlockId::RustStoneLamp,
        objective: QuestObjective::MineBlock { block: BlockId::RustStoneOre, target: 6 },
        prerequisite: Some(17),
        rewards: &[(BlockId::RustStoneLamp, 4), (BlockId::Lever, 2)],
    },
    // --- 20..25: Exploration ---
    QuestDef {
        id: "wanderer",
        category: QuestCategory::Exploration,
        icon: BlockId::Grass,
        objective: QuestObjective::VisitBiomes { target: 3 },
        prerequisite: None,
        rewards: &[(BlockId::CookedPorkchop, 4), (BlockId::Torch, 8)],
    },
    QuestDef {
        id: "world_traveler",
        category: QuestCategory::Exploration,
        icon: BlockId::GoldBlock,
        objective: QuestObjective::VisitBiomes { target: 6 },
        prerequisite: Some(20),
        rewards: &[(BlockId::GoldBlock, 1)],
    },
    QuestDef {
        id: "angler",
        category: QuestCategory::Exploration,
        icon: BlockId::RawCod,
        objective: QuestObjective::HaveItem { item: BlockId::RawCod, target: 4 },
        prerequisite: Some(20),
        rewards: &[(BlockId::FishingRod, 1), (BlockId::CookedSalmon, 2)],
    },
    QuestDef {
        id: "emerald_fortune",
        category: QuestCategory::Exploration,
        icon: BlockId::Emerald,
        objective: QuestObjective::MineBlock { block: BlockId::EmeraldOre, target: 2 },
        prerequisite: Some(21),
        rewards: &[(BlockId::Emerald, 3)],
    },
    QuestDef {
        id: "rustcraft_legend",
        category: QuestCategory::Exploration,
        icon: BlockId::DiamondBlock,
        objective: QuestObjective::TotalMined { target: 500 },
        prerequisite: Some(21),
        rewards: &[(BlockId::DiamondBlock, 1), (BlockId::Emerald, 4)],
    },
    // --- 25..29: Extra Basics ---
    QuestDef {
        id: "glassmaker",
        category: QuestCategory::Basics,
        icon: BlockId::Glass,
        objective: QuestObjective::SmeltItem { item: BlockId::Glass, target: 8 },
        prerequisite: Some(2),
        rewards: &[(BlockId::Glass, 8), (BlockId::Torch, 4)],
    },
    QuestDef {
        id: "carpenter",
        category: QuestCategory::Basics,
        icon: BlockId::WoodenStairs,
        objective: QuestObjective::CraftItem { item: BlockId::WoodenStairs, target: 16 },
        prerequisite: Some(1),
        rewards: &[(BlockId::Planks, 8), (BlockId::Stick, 4)],
    },
    QuestDef {
        id: "first_shelter",
        category: QuestCategory::Basics,
        icon: BlockId::Door,
        objective: QuestObjective::PlaceBlock { block: BlockId::Door, target: 3 },
        prerequisite: Some(4),
        rewards: &[(BlockId::Torch, 6), (BlockId::Bread, 2)],
    },
    QuestDef {
        id: "lumberjack",
        category: QuestCategory::Basics,
        icon: BlockId::IronAxe,
        objective: QuestObjective::MineBlock { block: BlockId::Wood, target: 64 },
        prerequisite: Some(0),
        rewards: &[(BlockId::IronAxe, 1), (BlockId::Apple, 4)],
    },
    // --- 29..33: Extra Mining ---
    QuestDef {
        id: "smelt_gold",
        category: QuestCategory::Mining,
        icon: BlockId::GoldIngot,
        objective: QuestObjective::SmeltItem { item: BlockId::GoldIngot, target: 6 },
        prerequisite: Some(8),
        rewards: &[(BlockId::GoldIngot, 2), (BlockId::GoldPickaxe, 1)],
    },
    QuestDef {
        id: "diamond_tools",
        category: QuestCategory::Mining,
        icon: BlockId::DiamondPickaxe,
        objective: QuestObjective::CraftItem { item: BlockId::DiamondPickaxe, target: 1 },
        prerequisite: Some(9),
        rewards: &[(BlockId::Diamond, 1), (BlockId::Obsidian, 4)],
    },
    QuestDef {
        id: "obsidian_breaker",
        category: QuestCategory::Mining,
        icon: BlockId::Obsidian,
        objective: QuestObjective::MineBlock { block: BlockId::Obsidian, target: 4 },
        prerequisite: Some(30),
        rewards: &[(BlockId::Diamond, 2), (BlockId::Emerald, 2)],
    },
    QuestDef {
        id: "iron_fortress",
        category: QuestCategory::Mining,
        icon: BlockId::IronBlock,
        objective: QuestObjective::CraftItem { item: BlockId::IronBlock, target: 1 },
        prerequisite: Some(7),
        rewards: &[(BlockId::IronIngot, 4), (BlockId::CoalOre, 8)],
    },
    // --- 33..37: Extra Combat ---
    QuestDef {
        id: "zombie_hunter",
        category: QuestCategory::Combat,
        icon: BlockId::IronSword,
        objective: QuestObjective::KillSpecificMob { kind: MobKind::Zombie, target: 5 },
        prerequisite: Some(11),
        rewards: &[(BlockId::CookedBeef, 4), (BlockId::IronSword, 1)],
    },
    QuestDef {
        id: "spider_slayer",
        category: QuestCategory::Combat,
        icon: BlockId::SpiderEye,
        objective: QuestObjective::KillSpecificMob { kind: MobKind::Spider, target: 4 },
        prerequisite: Some(11),
        rewards: &[(BlockId::StringItem, 4), (BlockId::Bow, 1)],
    },
    QuestDef {
        id: "reaper_bane",
        category: QuestCategory::Combat,
        icon: BlockId::Shield,
        objective: QuestObjective::KillSpecificMob { kind: MobKind::Reaper, target: 2 },
        prerequisite: Some(14),
        rewards: &[(BlockId::Diamond, 2), (BlockId::Shield, 1)],
    },
    QuestDef {
        id: "diamond_armor",
        category: QuestCategory::Combat,
        icon: BlockId::DiamondChestplate,
        objective: QuestObjective::HaveItem { item: BlockId::DiamondChestplate, target: 1 },
        prerequisite: Some(13),
        rewards: &[(BlockId::DiamondHelmet, 1), (BlockId::DiamondBoots, 1)],
    },
    // --- 37..41: Extra Farming & Building ---
    QuestDef {
        id: "master_chef",
        category: QuestCategory::Farming,
        icon: BlockId::CookedBeef,
        objective: QuestObjective::SmeltItem { item: BlockId::CookedBeef, target: 8 },
        prerequisite: Some(16),
        rewards: &[(BlockId::BakedPotato, 4), (BlockId::Bread, 4)],
    },
    QuestDef {
        id: "bricklayer",
        category: QuestCategory::Farming,
        icon: BlockId::Brick,
        objective: QuestObjective::SmeltItem { item: BlockId::Brick, target: 16 },
        prerequisite: Some(17),
        rewards: &[(BlockId::Bricks, 16)],
    },
    QuestDef {
        id: "demolition",
        category: QuestCategory::Farming,
        icon: BlockId::Tnt,
        objective: QuestObjective::CraftItem { item: BlockId::Tnt, target: 2 },
        prerequisite: Some(19),
        rewards: &[(BlockId::Gunpowder, 4), (BlockId::Sand, 8)],
    },
    QuestDef {
        id: "shepherd",
        category: QuestCategory::Farming,
        icon: BlockId::Shears,
        objective: QuestObjective::CraftItem { item: BlockId::Shears, target: 1 },
        prerequisite: Some(15),
        rewards: &[(BlockId::WhiteWool, 8), (BlockId::RedWool, 4)],
    },
    // --- 41..45: Extra Exploration & Mastery ---
    QuestDef {
        id: "merchant",
        category: QuestCategory::Exploration,
        icon: BlockId::Emerald,
        objective: QuestObjective::TradeVillager { target: 3 },
        prerequisite: Some(20),
        rewards: &[(BlockId::Emerald, 3)],
    },
    QuestDef {
        id: "grand_trader",
        category: QuestCategory::Exploration,
        icon: BlockId::EmeraldBlock,
        objective: QuestObjective::TradeVillager { target: 10 },
        prerequisite: Some(41),
        rewards: &[(BlockId::EmeraldBlock, 1)],
    },
    QuestDef {
        id: "biome_master",
        category: QuestCategory::Exploration,
        icon: BlockId::Sapling,
        objective: QuestObjective::VisitBiomes { target: 12 },
        prerequisite: Some(21),
        rewards: &[(BlockId::Diamond, 2), (BlockId::Emerald, 4)],
    },
    QuestDef {
        id: "grand_architect",
        category: QuestCategory::Exploration,
        icon: BlockId::EmeraldBlock,
        objective: QuestObjective::TotalPlaced { target: 250 },
        prerequisite: Some(24),
        rewards: &[(BlockId::DiamondBlock, 1), (BlockId::EmeraldBlock, 1)],
    },
];

pub fn quests_in_category(category: QuestCategory) -> Vec<usize> {
    QUESTS
        .iter()
        .enumerate()
        .filter(|(_, q)| q.category == category)
        .map(|(i, _)| i)
        .collect()
}

fn is_hostile_mob(kind: MobKind) -> bool {
    matches!(
        kind,
        MobKind::Zombie | MobKind::Skeleton | MobKind::Reaper | MobKind::Spider
    )
}

fn count_item_in_inventory(inventory: &Inventory, target: BlockId) -> u32 {
    let mut total: u32 = 0;
    for slot in inventory
        .hotbar
        .iter()
        .chain(inventory.main.iter())
        .chain(inventory.armor.iter())
        .chain(std::iter::once(&inventory.offhand))
        .chain(std::iter::once(&inventory.held_item))
    {
        if let Some(stack) = slot {
            if stack.block == target {
                total += stack.count as u32;
            }
        }
    }
    total
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestProgress {
    pub enabled: bool,
    pub progress: [u32; QUEST_COUNT],
    pub completed: u64,
    pub claimed: u64,
}

impl Default for QuestProgress {
    fn default() -> Self {
        Self {
            enabled: true,
            progress: [0; QUEST_COUNT],
            completed: 0,
            claimed: 0,
        }
    }
}

impl QuestProgress {
    const MAGIC: &'static [u8; 4] = b"RCQ1";
    pub const ENCODED_LEN: usize = 4 + 8 + 8 + QUEST_COUNT * 4 + 1;

    pub fn is_completed(&self, idx: usize) -> bool {
        idx < QUEST_COUNT && (self.completed & (1u64 << idx)) != 0
    }

    pub fn is_claimed(&self, idx: usize) -> bool {
        idx < QUEST_COUNT && (self.claimed & (1u64 << idx)) != 0
    }

    pub fn is_unlocked(&self, idx: usize) -> bool {
        if idx >= QUEST_COUNT {
            return false;
        }
        match QUESTS[idx].prerequisite {
            None => true,
            Some(prereq) => self.is_completed(prereq),
        }
    }

    pub fn completed_count(&self) -> usize {
        (self.completed & ((1u64 << QUEST_COUNT) - 1)).count_ones() as usize
    }

    pub fn claimable_count(&self) -> usize {
        let mask = (1u64 << QUEST_COUNT) - 1;
        ((self.completed & !self.claimed) & mask).count_ones() as usize
    }

    pub fn quest_progress(&self, idx: usize) -> (u32, u32) {
        if idx >= QUEST_COUNT {
            return (0, 1);
        }
        let target = QUESTS[idx].objective.target().max(1);
        let cur = if self.is_completed(idx) {
            target
        } else {
            self.progress[idx].min(target)
        };
        (cur, target)
    }

    fn advance_quest(&mut self, idx: usize, new_val: u32, newly_completed: &mut Vec<usize>) {
        if !self.enabled || idx >= QUEST_COUNT || self.is_completed(idx) || !self.is_unlocked(idx) {
            return;
        }
        let target = QUESTS[idx].objective.target();
        let clamped = new_val.min(target);
        if clamped > self.progress[idx] {
            self.progress[idx] = clamped;
        }
        if self.progress[idx] >= target {
            self.completed |= 1u64 << idx;
            newly_completed.push(idx);
        }
    }

    fn add_quest(&mut self, idx: usize, delta: u32, newly_completed: &mut Vec<usize>) {
        if idx >= QUEST_COUNT || self.is_completed(idx) || !self.is_unlocked(idx) {
            return;
        }
        let next = self.progress[idx].saturating_add(delta);
        self.advance_quest(idx, next, newly_completed);
    }

    pub fn record_mined(&mut self, block: BlockId) -> Vec<usize> {
        let mut newly = Vec::new();
        for i in 0..QUEST_COUNT {
            match QUESTS[i].objective {
                QuestObjective::MineBlock { block: target, .. } if target == block => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::TotalMined { .. } => {
                    self.add_quest(i, 1, &mut newly);
                }
                _ => {}
            }
        }
        newly
    }

    pub fn record_placed(&mut self, block: BlockId) -> Vec<usize> {
        let mut newly = Vec::new();
        for i in 0..QUEST_COUNT {
            match QUESTS[i].objective {
                QuestObjective::PlaceBlock { block: target, .. } if target == block => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::TotalPlaced { .. } => {
                    self.add_quest(i, 1, &mut newly);
                }
                _ => {}
            }
        }
        newly
    }

    pub fn record_crafted(&mut self, item: BlockId, count: u8) -> Vec<usize> {
        let mut newly = Vec::new();
        let delta = count as u32;
        for i in 0..QUEST_COUNT {
            if let QuestObjective::CraftItem { item: target, .. } = QUESTS[i].objective {
                if target == item {
                    self.add_quest(i, delta, &mut newly);
                }
            }
        }
        newly
    }

    pub fn record_smelted(&mut self, item: BlockId, count: u8) -> Vec<usize> {
        let mut newly = Vec::new();
        let delta = count as u32;
        for i in 0..QUEST_COUNT {
            if let QuestObjective::SmeltItem { item: target, .. } = QUESTS[i].objective {
                if target == item {
                    self.add_quest(i, delta, &mut newly);
                }
            }
        }
        newly
    }

    pub fn record_mob_killed(&mut self, kind: MobKind) -> Vec<usize> {
        let mut newly = Vec::new();
        for i in 0..QUEST_COUNT {
            match QUESTS[i].objective {
                QuestObjective::KillHostile { .. } if is_hostile_mob(kind) => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::KillSpecificMob { kind: target, .. } if target == kind => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::KillAnyMob { .. } => {
                    self.add_quest(i, 1, &mut newly);
                }
                _ => {}
            }
        }
        newly
    }

    pub fn sync_biomes(&mut self, visited_biomes: usize) -> Vec<usize> {
        let mut newly = Vec::new();
        let count = visited_biomes as u32;
        for i in 0..QUEST_COUNT {
            if let QuestObjective::VisitBiomes { .. } = QUESTS[i].objective {
                self.advance_quest(i, count, &mut newly);
            }
        }
        newly
    }

    pub fn record_trade(&mut self) -> Vec<usize> {
        let mut newly = Vec::new();
        for i in 0..QUEST_COUNT {
            if let QuestObjective::TradeVillager { .. } = QUESTS[i].objective {
                self.add_quest(i, 1, &mut newly);
            }
        }
        newly
    }

    pub fn sync_inventory(&mut self, inventory: &Inventory) -> Vec<usize> {
        let mut newly = Vec::new();
        for i in 0..QUEST_COUNT {
            if let QuestObjective::HaveItem { item, .. } = QUESTS[i].objective {
                let count = count_item_in_inventory(inventory, item);
                self.advance_quest(i, count, &mut newly);
            }
        }
        newly
    }

    pub fn claim_reward(&mut self, idx: usize, inventory: &mut Inventory) -> bool {
        if !self.enabled || idx >= QUEST_COUNT || !self.is_completed(idx) || self.is_claimed(idx) {
            return false;
        }
        for &(block, count) in QUESTS[idx].rewards {
            let _rem = inventory.add_item(block, count);
        }
        self.claimed |= 1u64 << idx;
        true
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::ENCODED_LEN);
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&self.completed.to_le_bytes());
        out.extend_from_slice(&self.claimed.to_le_bytes());
        for &p in &self.progress {
            out.extend_from_slice(&p.to_le_bytes());
        }
        out.push(if self.enabled { 1 } else { 0 });
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 20 || &bytes[0..4] != Self::MAGIC {
            return Err("Invalid quest progress header".to_string());
        }
        let completed = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
        let claimed = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
        let mut progress = [0u32; QUEST_COUNT];
        for (i, slot) in progress.iter_mut().enumerate() {
            let off = 20 + i * 4;
            if off + 4 <= bytes.len() {
                *slot = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
            }
        }
        let enabled_off = 20 + QUEST_COUNT * 4;
        let enabled = if enabled_off < bytes.len() {
            bytes[enabled_off] != 0
        } else {
            true
        };
        Ok(Self {
            enabled,
            progress,
            completed,
            claimed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_progression_and_prerequisites() {
        let mut qp = QuestProgress::default();
        assert!(qp.is_unlocked(0));
        assert!(!qp.is_unlocked(1));

        // Mining wood completes quest 0 and unlocks quest 1
        for _ in 0..8 {
            qp.record_mined(BlockId::Wood);
        }
        assert!(qp.is_completed(0));
        assert!(qp.is_unlocked(1));
        assert_eq!(qp.claimable_count(), 1);

        let mut inv = Inventory::new();
        assert!(qp.claim_reward(0, &mut inv));
        assert!(qp.is_claimed(0));
        assert!(!qp.claim_reward(0, &mut inv));
        assert_eq!(qp.claimable_count(), 0);
        assert!(count_item_in_inventory(&inv, BlockId::Planks) >= 8);
    }

    #[test]
    fn quest_progress_encode_decode_roundtrip() {
        let mut qp = QuestProgress::default();
        qp.record_mined(BlockId::Wood);
        qp.completed |= 1u64 << 0;
        qp.claimed |= 1u64 << 0;
        qp.progress[5] = 7;

        let encoded = qp.encode();
        assert_eq!(encoded.len(), QuestProgress::ENCODED_LEN);
        let decoded = QuestProgress::decode(&encoded).expect("decode should succeed");
        assert_eq!(qp, decoded);
    }
}
