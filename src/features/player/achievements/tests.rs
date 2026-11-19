//! Unit tests for achievement unlock DAGs, discovery bitsets, and serialization versioning.

use crate::features::blocks::BlockId;
use crate::features::entities::mob::MobKind;
use crate::features::world::Biome;

use super::{AchievementId, AchievementProgress};

#[test]
fn achievements_unlock_in_order_and_require_each_branch() {
    let mut progress = AchievementProgress::default();
    assert_eq!(progress.record_mined(BlockId::Stone), vec![AchievementId::FirstMark]);
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

    let discoveries_only =
        AchievementProgress::decode(&progress.encode()[..AchievementProgress::DISCOVERIES_ENCODED_LEN])
            .unwrap();
    assert!(!discoveries_only.has_collected_item(BlockId::CoalOre));
}

#[test]
fn old_achievement_records_still_load_without_catalog_entries() {
    let mut bytes = [0; AchievementProgress::LEGACY_ENCODED_LEN];
    bytes[..4].copy_from_slice(AchievementProgress::MAGIC);
    assert_eq!(AchievementProgress::decode(&bytes).unwrap(), AchievementProgress::default());
}
