//! Questbook player progression bitmasks, objective updates, reward claiming, and binary codec.

use crate::features::blocks::BlockId;
use crate::features::entities::mob::MobKind;

use super::super::inventory::Inventory;
use super::catalog::QUESTS;
use super::types::{QuestObjective, QUEST_COUNT};

#[inline]
fn is_hostile_mob(kind: MobKind) -> bool {
    matches!(kind, MobKind::Zombie | MobKind::Skeleton | MobKind::Reaper | MobKind::Spider)
}

pub(super) fn count_item_in_inventory(inventory: &Inventory, target: BlockId) -> u32 {
    inventory
        .hotbar
        .iter()
        .chain(inventory.main.iter())
        .chain(inventory.armor.iter())
        .chain(std::iter::once(&inventory.offhand))
        .chain(std::iter::once(&inventory.held_item))
        .flatten()
        .filter(|stack| stack.block == target)
        .map(|stack| stack.count as u32)
        .sum()
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
        let cur = if self.is_completed(idx) { target } else { self.progress[idx].min(target) };
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
        for (i, quest) in QUESTS.iter().enumerate() {
            match quest.objective {
                QuestObjective::MineBlock { block: target, .. } if target == block => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::TotalMined { .. } => self.add_quest(i, 1, &mut newly),
                _ => {}
            }
        }
        newly
    }

    pub fn record_placed(&mut self, block: BlockId) -> Vec<usize> {
        let mut newly = Vec::new();
        for (i, quest) in QUESTS.iter().enumerate() {
            match quest.objective {
                QuestObjective::PlaceBlock { block: target, .. } if target == block => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::TotalPlaced { .. } => self.add_quest(i, 1, &mut newly),
                _ => {}
            }
        }
        newly
    }

    pub fn record_crafted(&mut self, item: BlockId, count: u8) -> Vec<usize> {
        let mut newly = Vec::new();
        let delta = count as u32;
        for (i, quest) in QUESTS.iter().enumerate() {
            if let QuestObjective::CraftItem { item: target, .. } = quest.objective {
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
        for (i, quest) in QUESTS.iter().enumerate() {
            if let QuestObjective::SmeltItem { item: target, .. } = quest.objective {
                if target == item {
                    self.add_quest(i, delta, &mut newly);
                }
            }
        }
        newly
    }

    pub fn record_mob_killed(&mut self, kind: MobKind) -> Vec<usize> {
        let mut newly = Vec::new();
        for (i, quest) in QUESTS.iter().enumerate() {
            match quest.objective {
                QuestObjective::KillHostile { .. } if is_hostile_mob(kind) => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::KillSpecificMob { kind: target, .. } if target == kind => {
                    self.add_quest(i, 1, &mut newly);
                }
                QuestObjective::KillAnyMob { .. } => self.add_quest(i, 1, &mut newly),
                _ => {}
            }
        }
        newly
    }

    pub fn sync_biomes(&mut self, visited_biomes: usize) -> Vec<usize> {
        let mut newly = Vec::new();
        let count = visited_biomes as u32;
        for (i, quest) in QUESTS.iter().enumerate() {
            if let QuestObjective::VisitBiomes { .. } = quest.objective {
                self.advance_quest(i, count, &mut newly);
            }
        }
        newly
    }

    pub fn record_trade(&mut self) -> Vec<usize> {
        let mut newly = Vec::new();
        for (i, quest) in QUESTS.iter().enumerate() {
            if let QuestObjective::TradeVillager { .. } = quest.objective {
                self.add_quest(i, 1, &mut newly);
            }
        }
        newly
    }

    pub fn sync_inventory(&mut self, inventory: &Inventory) -> Vec<usize> {
        let mut newly = Vec::new();
        for (i, quest) in QUESTS.iter().enumerate() {
            if let QuestObjective::HaveItem { item, .. } = quest.objective {
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
        let enabled = if enabled_off < bytes.len() { bytes[enabled_off] != 0 } else { true };
        Ok(Self { enabled, progress, completed, claimed })
    }
}
