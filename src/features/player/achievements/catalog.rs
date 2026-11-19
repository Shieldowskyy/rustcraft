//! Achievement catalog (`AchievementId`), dependency graph, and biome bitmask mapping.

use crate::features::world::Biome;

struct AchievementSpec {
    name_key: &'static str,
    description_key: &'static str,
    prerequisites: &'static [AchievementId],
}

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

const SPECS: [AchievementSpec; 9] = [
    AchievementSpec {
        name_key: "achievement.first_mark.name",
        description_key: "achievement.first_mark.description",
        prerequisites: &[],
    },
    AchievementSpec {
        name_key: "achievement.builder.name",
        description_key: "achievement.builder.description",
        prerequisites: &[AchievementId::FirstMark],
    },
    AchievementSpec {
        name_key: "achievement.deep_memory.name",
        description_key: "achievement.deep_memory.description",
        prerequisites: &[AchievementId::FirstMark],
    },
    AchievementSpec {
        name_key: "achievement.wayfinder.name",
        description_key: "achievement.wayfinder.description",
        prerequisites: &[AchievementId::FirstMark],
    },
    AchievementSpec {
        name_key: "achievement.crafter.name",
        description_key: "achievement.crafter.description",
        prerequisites: &[AchievementId::Builder],
    },
    AchievementSpec {
        name_key: "achievement.ore_whisperer.name",
        description_key: "achievement.ore_whisperer.description",
        prerequisites: &[AchievementId::DeepMemory],
    },
    AchievementSpec {
        name_key: "achievement.color_atlas.name",
        description_key: "achievement.color_atlas.description",
        prerequisites: &[AchievementId::Wayfinder],
    },
    AchievementSpec {
        name_key: "achievement.cave_guardian.name",
        description_key: "achievement.cave_guardian.description",
        prerequisites: &[AchievementId::OreWhisperer],
    },
    AchievementSpec {
        name_key: "achievement.path_weaver.name",
        description_key: "achievement.path_weaver.description",
        prerequisites: &[
            AchievementId::Crafter,
            AchievementId::ColorAtlas,
            AchievementId::CaveGuardian,
        ],
    },
];

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

    #[inline]
    fn spec(self) -> &'static AchievementSpec {
        &SPECS[self as usize]
    }

    pub fn name_key(self) -> &'static str {
        self.spec().name_key
    }

    pub fn description_key(self) -> &'static str {
        self.spec().description_key
    }

    pub(super) fn prerequisites(self) -> &'static [Self] {
        self.spec().prerequisites
    }
}

pub(super) fn biome_bit(biome: Biome) -> u32 {
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
