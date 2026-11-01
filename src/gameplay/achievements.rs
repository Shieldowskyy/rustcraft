use crate::gameplay::mob::MobKind;
use crate::world::block::BlockId;
use crate::world::Biome;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AchievementId {
    FirstMark,
    Builder,
    DeepMemory,
    Wayfinder,
    Crafter,
    OreWhisperer,
    ColorAtlas,
    CaveGuardian,
    PathWeaver,
}

impl AchievementId {
    pub const ALL: [Self; 9] = [
        Self::FirstMark,
        Self::Builder,
        Self::DeepMemory,
        Self::Wayfinder,
        Self::Crafter,
        Self::OreWhisperer,
        Self::ColorAtlas,
        Self::CaveGuardian,
        Self::PathWeaver,
    ];

    pub fn name_key(self) -> &'static str {
        match self {
            Self::FirstMark => "achievement.first_mark.name",
            Self::Builder => "achievement.builder.name",
            Self::DeepMemory => "achievement.deep_memory.name",
            Self::Wayfinder => "achievement.wayfinder.name",
            Self::Crafter => "achievement.crafter.name",
            Self::OreWhisperer => "achievement.ore_whisperer.name",
            Self::ColorAtlas => "achievement.color_atlas.name",
            Self::CaveGuardian => "achievement.cave_guardian.name",
            Self::PathWeaver => "achievement.path_weaver.name",
        }
    }

    pub fn description_key(self) -> &'static str {
        match self {
            Self::FirstMark => "achievement.first_mark.description",
            Self::Builder => "achievement.builder.description",
            Self::DeepMemory => "achievement.deep_memory.description",
            Self::Wayfinder => "achievement.wayfinder.description",
            Self::Crafter => "achievement.crafter.description",
            Self::OreWhisperer => "achievement.ore_whisperer.description",
            Self::ColorAtlas => "achievement.color_atlas.description",
            Self::CaveGuardian => "achievement.cave_guardian.description",
            Self::PathWeaver => "achievement.path_weaver.description",
        }
    }

    fn prerequisites(self) -> &'static [Self] {
        match self {
            Self::FirstMark => &[],
            Self::Builder | Self::DeepMemory | Self::Wayfinder => &[Self::FirstMark],
            Self::Crafter => &[Self::Builder],
            Self::OreWhisperer => &[Self::DeepMemory],
            Self::ColorAtlas => &[Self::Wayfinder],
            Self::CaveGuardian => &[Self::OreWhisperer],
            Self::PathWeaver => &[Self::Crafter, Self::ColorAtlas, Self::CaveGuardian],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AchievementProgress {
    unlocked: u16,
    mined_blocks: u16,
    mined_ores: u16,
    placed_blocks: u16,
    crafted_items: u16,
    defeated_mobs: u16,
    visited_biomes: u64,
    discovered_mobs: u16,
    discovered_blocks: [u8; 32],
    collected_items: [u8; 32],
}

impl AchievementProgress {
    const MAGIC: &'static [u8; 4] = b"RCA1";
    pub const LEGACY_ENCODED_LEN: usize = 24;
    pub const DISCOVERIES_ENCODED_LEN: usize = 58;
    pub const ENCODED_LEN: usize = 90;

    pub fn is_unlocked(&self, id: AchievementId) -> bool {
        self.unlocked & (1 << id as u8) != 0
    }

    pub fn unlocked_count(&self) -> usize {
        self.unlocked.count_ones() as usize
    }

    pub fn has_seen_mob(&self, mob: MobKind) -> bool {
        self.discovered_mobs & (1 << mob as u8) != 0
    }

    pub fn record_mob_seen(&mut self, mob: MobKind) -> bool {
        let flag = 1 << mob as u8;
        let was_new = self.discovered_mobs & flag == 0;
        self.discovered_mobs |= flag;
        was_new
    }

    pub fn has_discovered_block(&self, block: BlockId) -> bool {
        let id = block.to_u8() as usize;
        self.discovered_blocks[id / 8] & (1 << (id % 8)) != 0
    }

    pub fn record_block_discovered(&mut self, block: BlockId) -> bool {
        if !block.is_placeable() {
            return false;
        }
        let id = block.to_u8() as usize;
        let flag = 1 << (id % 8);
        let was_new = self.discovered_blocks[id / 8] & flag == 0;
        self.discovered_blocks[id / 8] |= flag;
        was_new
    }

    pub fn has_collected_item(&self, block: BlockId) -> bool {
        let id = block.to_u8() as usize;
        id != 0 && self.collected_items[id / 8] & (1 << (id % 8)) != 0
    }

    pub fn record_item_collected(&mut self, block: BlockId) -> bool {
        let id = block.to_u8() as usize;
        if id == 0 {
            return false;
        }
        let flag = 1 << (id % 8);
        let was_new = self.collected_items[id / 8] & flag == 0;
        self.collected_items[id / 8] |= flag;
        was_new
    }

    pub fn discovered_mob_count(&self) -> usize {
        self.discovered_mobs.count_ones() as usize
    }

    pub fn discovered_block_count(&self) -> usize {
        (1..=u8::MAX)
            .map(BlockId::from_u8)
            .filter(|block| block.is_placeable() && self.has_discovered_block(*block))
            .count()
    }

    pub fn counts_for(&self, id: AchievementId) -> (u32, u32) {
        match id {
            AchievementId::FirstMark => (self.mined_blocks as u32, 1),
            AchievementId::Builder => (self.placed_blocks as u32, 16),
            AchievementId::DeepMemory => (self.mined_blocks as u32, 32),
            AchievementId::Wayfinder => (self.visited_biomes.count_ones(), 5),
            AchievementId::Crafter => (self.crafted_items as u32, 1),
            AchievementId::OreWhisperer => (self.mined_ores as u32, 1),
            AchievementId::ColorAtlas => (self.visited_biomes.count_ones(), 12),
            AchievementId::CaveGuardian => (self.defeated_mobs as u32, 3),
            AchievementId::PathWeaver => (
                self.is_unlocked(AchievementId::Crafter) as u32
                    + self.is_unlocked(AchievementId::ColorAtlas) as u32
                    + self.is_unlocked(AchievementId::CaveGuardian) as u32,
                3,
            ),
        }
    }

    pub fn record_mined(&mut self, block: BlockId) -> Vec<AchievementId> {
        self.mined_blocks = self.mined_blocks.saturating_add(1);
        if matches!(
            block,
            BlockId::CoalOre
                | BlockId::IronOre
                | BlockId::GoldOre
                | BlockId::DiamondOre
                | BlockId::EmeraldOre
                | BlockId::RustStoneOre
        ) {
            self.mined_ores = self.mined_ores.saturating_add(1);
        }
        self.evaluate()
    }

    pub fn record_placed(&mut self) -> Vec<AchievementId> {
        self.placed_blocks = self.placed_blocks.saturating_add(1);
        self.evaluate()
    }

    pub fn record_crafted(&mut self, count: u8) -> Vec<AchievementId> {
        self.crafted_items = self.crafted_items.saturating_add(count as u16);
        self.evaluate()
    }

    pub fn record_defeated_mob(&mut self) -> Vec<AchievementId> {
        self.defeated_mobs = self.defeated_mobs.saturating_add(1);
        self.evaluate()
    }

    pub fn visit_biome(&mut self, biome: Biome) -> Vec<AchievementId> {
        self.visited_biomes |= 1u64 << biome_bit(biome);
        self.evaluate()
    }

    fn evaluate(&mut self) -> Vec<AchievementId> {
        let mut unlocked = Vec::new();
        loop {
            let mut changed = false;
            for id in AchievementId::ALL {
                if self.is_unlocked(id)
                    || !id
                        .prerequisites()
                        .iter()
                        .all(|&parent| self.is_unlocked(parent))
                {
                    continue;
                }
                let (progress, goal) = self.counts_for(id);
                if progress >= goal {
                    self.unlocked |= 1 << id as u8;
                    unlocked.push(id);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        unlocked
    }

    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        let mut bytes = [0; Self::ENCODED_LEN];
        bytes[..4].copy_from_slice(Self::MAGIC);
        bytes[4..6].copy_from_slice(&self.unlocked.to_le_bytes());
        bytes[6..8].copy_from_slice(&self.mined_blocks.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.mined_ores.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.placed_blocks.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.crafted_items.to_le_bytes());
        bytes[14..16].copy_from_slice(&self.defeated_mobs.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.visited_biomes.to_le_bytes());
        bytes[24..26].copy_from_slice(&self.discovered_mobs.to_le_bytes());
        bytes[26..58].copy_from_slice(&self.discovered_blocks);
        bytes[58..90].copy_from_slice(&self.collected_items);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if (bytes.len() != Self::ENCODED_LEN
            && bytes.len() != Self::DISCOVERIES_ENCODED_LEN
            && bytes.len() != Self::LEGACY_ENCODED_LEN)
            || &bytes[..4] != Self::MAGIC
        {
            return Err("Invalid achievement progress file".to_string());
        }
        let u16_at = |start| u16::from_le_bytes([bytes[start], bytes[start + 1]]);
        let unlocked = u16_at(4);
        if unlocked & !((1 << AchievementId::ALL.len()) - 1) != 0 {
            return Err("Achievement progress contains unknown unlock flags".to_string());
        }
        let visited_biomes = u64::from_le_bytes([
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
        ]);
        if visited_biomes >> 37 != 0 {
            return Err("Achievement progress contains unknown biome flags".to_string());
        }
        let discovered_mobs = if bytes.len() >= Self::DISCOVERIES_ENCODED_LEN {
            u16_at(24)
        } else {
            0
        };
        if discovered_mobs >> MobKind::ALL.len() != 0 {
            return Err("Achievement progress contains unknown creature flags".to_string());
        }
        let mut discovered_blocks = [0; 32];
        if bytes.len() >= Self::DISCOVERIES_ENCODED_LEN {
            discovered_blocks.copy_from_slice(&bytes[26..58]);
            if discovered_blocks[0] & 1 != 0 {
                return Err("Achievement progress contains an invalid air discovery".to_string());
            }
        }
        let mut collected_items = [0; 32];
        if bytes.len() == Self::ENCODED_LEN {
            collected_items.copy_from_slice(&bytes[58..90]);
            if collected_items[0] & 1 != 0 {
                return Err("Achievement progress contains an invalid air collection".to_string());
            }
        }
        Ok(Self {
            unlocked,
            mined_blocks: u16_at(6),
            mined_ores: u16_at(8),
            placed_blocks: u16_at(10),
            crafted_items: u16_at(12),
            defeated_mobs: u16_at(14),
            visited_biomes,
            discovered_mobs,
            discovered_blocks,
            collected_items,
        })
    }
}

fn biome_bit(biome: Biome) -> u32 {
    match biome {
        Biome::DeepOcean => 0,
        Biome::Ocean => 1,
        Biome::SnowyTaiga => 2,
        Biome::SnowyTundra => 3,
        Biome::AutumnForest => 4,
        Biome::Forest => 5,
        Biome::CherryGrove => 6,
        Biome::Plains => 7,
        Biome::Swamp => 8,
        Biome::Savanna => 9,
        Biome::Desert => 10,
        Biome::Mountain => 11,
        Biome::Jungle => 12,
        Biome::Badlands => 13,
        Biome::DarkForest => 14,
        Biome::Taiga => 15,
        Biome::BirchForest => 16,
        Biome::IceSpikes => 17,
        Biome::MushroomIsland => 18,
        Biome::Meadow => 19,
        Biome::JaggedPeaks => 20,
        Biome::FrozenPeaks => 21,
        Biome::SnowySlopes => 22,
        Biome::River => 23,
        Biome::FrozenRiver => 24,
        Biome::Beach => 25,
        Biome::ColdBeach => 26,
        Biome::StoneBeach => 27,
        Biome::SunflowerPlains => 28,
        Biome::FlowerForest => 29,
        Biome::MegaTaiga => 30,
        Biome::JungleEdge => 31,
        Biome::SavannaPlateau => 32,
        Biome::Mesa => 33,
        Biome::MesaPlateau => 34,
        Biome::MesaPlateauForest => 35,
        Biome::MesaBryce => 36,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn achievements_unlock_in_order_and_require_each_branch() {
        let mut progress = AchievementProgress::default();
        assert_eq!(
            progress.record_mined(BlockId::Stone),
            vec![AchievementId::FirstMark]
        );
        for _ in 0..16 {
            progress.record_placed();
        }
        assert!(progress.is_unlocked(AchievementId::Builder));
        assert!(!progress.is_unlocked(AchievementId::PathWeaver));
        for _ in 0..31 {
            progress.record_mined(BlockId::Stone);
        }
        assert!(progress.is_unlocked(AchievementId::DeepMemory));
        progress.record_mined(BlockId::IronOre);
        assert!(progress.is_unlocked(AchievementId::OreWhisperer));
        for _ in 0..3 {
            progress.record_defeated_mob();
        }
        assert!(progress.is_unlocked(AchievementId::CaveGuardian));
        progress.record_crafted(1);
        assert!(progress.is_unlocked(AchievementId::Crafter));
        for biome in [
            Biome::Ocean,
            Biome::Forest,
            Biome::Desert,
            Biome::Taiga,
            Biome::Swamp,
            Biome::Plains,
            Biome::Jungle,
            Biome::River,
            Biome::Beach,
            Biome::Savanna,
            Biome::Mountain,
            Biome::Meadow,
        ] {
            progress.visit_biome(biome);
        }
        assert!(progress.is_unlocked(AchievementId::PathWeaver));
    }

    #[test]
    fn progress_round_trips_and_biomes_only_count_once() {
        let mut progress = AchievementProgress::default();
        progress.visit_biome(Biome::Forest);
        progress.visit_biome(Biome::Forest);
        progress.record_mined(BlockId::IronOre);
        let decoded = AchievementProgress::decode(&progress.encode()).unwrap();
        assert_eq!(decoded, progress);
        assert_eq!(decoded.counts_for(AchievementId::Wayfinder), (1, 5));
    }

    #[test]
    fn creature_and_block_discoveries_are_unique_and_persisted() {
        let mut progress = AchievementProgress::default();
        assert!(progress.record_mob_seen(MobKind::Spider));
        assert!(!progress.record_mob_seen(MobKind::Spider));
        assert!(progress.record_block_discovered(BlockId::Obsidian));
        assert!(!progress.record_block_discovered(BlockId::Obsidian));
        assert!(!progress.record_block_discovered(BlockId::Apple));
        assert!(progress.record_item_collected(BlockId::CoalOre));
        assert!(!progress.record_item_collected(BlockId::CoalOre));
        assert!(!progress.record_item_collected(BlockId::Air));

        let decoded = AchievementProgress::decode(&progress.encode()).unwrap();
        assert_eq!(decoded, progress);
        assert_eq!(decoded.discovered_mob_count(), 1);
        assert!(decoded.has_seen_mob(MobKind::Spider));
        assert!(decoded.has_discovered_block(BlockId::Obsidian));
        assert_eq!(decoded.discovered_block_count(), 1);
        assert!(decoded.has_collected_item(BlockId::CoalOre));
        assert!(!decoded.has_collected_item(BlockId::Air));

        let discoveries_only = AchievementProgress::decode(
            &progress.encode()[..AchievementProgress::DISCOVERIES_ENCODED_LEN],
        )
        .unwrap();
        assert!(!discoveries_only.has_collected_item(BlockId::CoalOre));
    }

    #[test]
    fn old_achievement_records_still_load_without_catalog_entries() {
        let mut bytes = [0; AchievementProgress::LEGACY_ENCODED_LEN];
        bytes[..4].copy_from_slice(AchievementProgress::MAGIC);
        assert_eq!(
            AchievementProgress::decode(&bytes).unwrap(),
            AchievementProgress::default()
        );
    }
}
