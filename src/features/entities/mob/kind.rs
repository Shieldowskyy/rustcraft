//! Mob taxonomy (`MobKind`) and declarative species descriptor table (`MobSpec`).
//! Replaces large per-property `match` statements with a single compile-time metadata table.

use crate::audio::SoundId;
use crate::features::blocks::BlockId;

/// Descriptor holding static stats, dimensions, and sound bindings for a `MobKind`.
pub(super) struct MobSpec {
    pub catalog_name_key: &'static str,
    pub display_name: &'static str,
    pub max_health: f32,
    pub half_width: f32,
    pub height: f32,
    pub is_hostile: bool,
    pub is_aquatic: bool,
    pub idle_sound: Option<SoundId>,
    pub hurt_sound: SoundId,
    pub death_sound: SoundId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MobKind {
    Pig,
    Cow,
    Chicken,
    Sheep,
    Zombie,
    Skeleton,
    Reaper,
    Dolphin,
    Cod,
    Salmon,
    Pufferfish,
    TropicalFish,
    Villager,
    IronGolem,
    Spider,
}

// Order matches `#[repr(u8)]` variants on `MobKind`.
const MOB_SPECS: [MobSpec; 15] = [
    MobSpec {
        catalog_name_key: "creature.pig",
        display_name: "Pig",
        max_health: 10.0,
        half_width: 0.4,
        height: 0.85,
        is_hostile: false,
        is_aquatic: false,
        idle_sound: Some(SoundId::PigIdle),
        hurt_sound: SoundId::PigHurt,
        death_sound: SoundId::PigDeath,
    },
    MobSpec {
        catalog_name_key: "creature.cow",
        display_name: "Cow",
        max_health: 10.0,
        half_width: 0.45,
        height: 1.35,
        is_hostile: false,
        is_aquatic: false,
        idle_sound: Some(SoundId::CowIdle),
        hurt_sound: SoundId::CowHurt,
        death_sound: SoundId::CowDeath,
    },
    MobSpec {
        catalog_name_key: "creature.chicken",
        display_name: "Chicken",
        max_health: 4.0,
        half_width: 0.25,
        height: 0.70,
        is_hostile: false,
        is_aquatic: false,
        idle_sound: Some(SoundId::ChickenIdle),
        hurt_sound: SoundId::ChickenHurt,
        death_sound: SoundId::ChickenDeath,
    },
    MobSpec {
        catalog_name_key: "creature.sheep",
        display_name: "Sheep",
        max_health: 8.0,
        half_width: 0.45,
        height: 1.30,
        is_hostile: false,
        is_aquatic: false,
        idle_sound: Some(SoundId::SheepIdle),
        hurt_sound: SoundId::SheepHurt,
        death_sound: SoundId::SheepDeath,
    },
    MobSpec {
        catalog_name_key: "creature.zombie",
        display_name: "Zombie",
        max_health: 20.0,
        half_width: 0.35,
        height: 1.85,
        is_hostile: true,
        is_aquatic: false,
        idle_sound: Some(SoundId::ZombieIdle),
        hurt_sound: SoundId::ZombieHurt,
        death_sound: SoundId::ZombieDeath,
    },
    MobSpec {
        catalog_name_key: "creature.skeleton",
        display_name: "Skeleton",
        max_health: 20.0,
        half_width: 0.30,
        height: 1.85,
        is_hostile: true,
        is_aquatic: false,
        idle_sound: Some(SoundId::SkeletonIdle),
        hurt_sound: SoundId::SkeletonHurt,
        death_sound: SoundId::SkeletonDeath,
    },
    MobSpec {
        catalog_name_key: "creature.reaper",
        display_name: "Reaper",
        max_health: 20.0,
        half_width: 0.30,
        height: 1.70,
        is_hostile: true,
        is_aquatic: false,
        idle_sound: None,
        hurt_sound: SoundId::ReaperHurt,
        death_sound: SoundId::ReaperDeath,
    },
    MobSpec {
        catalog_name_key: "creature.dolphin",
        display_name: "Dolphin",
        max_health: 10.0,
        half_width: 0.38,
        height: 0.60,
        is_hostile: false,
        is_aquatic: true,
        idle_sound: Some(SoundId::DolphinIdle),
        hurt_sound: SoundId::DolphinHurt,
        death_sound: SoundId::DolphinDeath,
    },
    MobSpec {
        catalog_name_key: "creature.cod",
        display_name: "Cod",
        max_health: 3.0,
        half_width: 0.20,
        height: 0.30,
        is_hostile: false,
        is_aquatic: true,
        idle_sound: None,
        hurt_sound: SoundId::FishFlop,
        death_sound: SoundId::FishFlop,
    },
    MobSpec {
        catalog_name_key: "creature.salmon",
        display_name: "Salmon",
        max_health: 3.0,
        half_width: 0.24,
        height: 0.35,
        is_hostile: false,
        is_aquatic: true,
        idle_sound: None,
        hurt_sound: SoundId::FishFlop,
        death_sound: SoundId::FishFlop,
    },
    MobSpec {
        catalog_name_key: "creature.pufferfish",
        display_name: "Pufferfish",
        max_health: 3.0,
        half_width: 0.25,
        height: 0.35,
        is_hostile: false,
        is_aquatic: true,
        idle_sound: None,
        hurt_sound: SoundId::FishFlop,
        death_sound: SoundId::FishFlop,
    },
    MobSpec {
        catalog_name_key: "creature.tropical_fish",
        display_name: "Tropical Fish",
        max_health: 3.0,
        half_width: 0.18,
        height: 0.28,
        is_hostile: false,
        is_aquatic: true,
        idle_sound: None,
        hurt_sound: SoundId::FishFlop,
        death_sound: SoundId::FishFlop,
    },
    MobSpec {
        catalog_name_key: "creature.villager",
        display_name: "Villager",
        max_health: 20.0,
        half_width: 0.32,
        height: 1.95,
        is_hostile: false,
        is_aquatic: false,
        idle_sound: Some(SoundId::VillagerIdle),
        hurt_sound: SoundId::VillagerHurt,
        death_sound: SoundId::VillagerDeath,
    },
    MobSpec {
        catalog_name_key: "creature.iron_golem",
        display_name: "Iron Golem",
        max_health: 100.0,
        half_width: 0.65,
        height: 2.70,
        is_hostile: false,
        is_aquatic: false,
        idle_sound: Some(SoundId::IronGolemIdle),
        hurt_sound: SoundId::IronGolemHurt,
        death_sound: SoundId::IronGolemDeath,
    },
    MobSpec {
        catalog_name_key: "creature.spider",
        display_name: "Spider",
        max_health: 16.0,
        half_width: 0.65,
        height: 0.50,
        is_hostile: true,
        is_aquatic: false,
        idle_sound: Some(SoundId::SpiderIdle),
        hurt_sound: SoundId::SpiderHurt,
        death_sound: SoundId::SpiderDeath,
    },
];

impl MobKind {
    pub const ALL: [Self; 15] = [
        Self::Pig,
        Self::Cow,
        Self::Chicken,
        Self::Sheep,
        Self::Zombie,
        Self::Skeleton,
        Self::Reaper,
        Self::Dolphin,
        Self::Cod,
        Self::Salmon,
        Self::Pufferfish,
        Self::TropicalFish,
        Self::Villager,
        Self::IronGolem,
        Self::Spider,
    ];

    #[inline]
    fn spec(&self) -> &'static MobSpec {
        &MOB_SPECS[*self as usize]
    }

    pub fn catalog_name_key(&self) -> &'static str {
        self.spec().catalog_name_key
    }

    pub fn is_aquatic(&self) -> bool {
        self.spec().is_aquatic
    }

    pub fn max_health(&self) -> f32 {
        self.spec().max_health
    }

    pub fn half_width(&self) -> f32 {
        self.spec().half_width
    }

    pub fn height(&self) -> f32 {
        self.spec().height
    }

    #[allow(dead_code)]
    pub fn is_hostile(&self) -> bool {
        self.spec().is_hostile
    }

    pub fn is_tempting_item(&self, item: BlockId) -> bool {
        match self {
            MobKind::Cow | MobKind::Sheep => item == BlockId::Wheat,
            MobKind::Pig => matches!(item, BlockId::Carrot | BlockId::Potato | BlockId::BakedPotato),
            MobKind::Chicken => item == BlockId::WheatSeeds,
            MobKind::Dolphin => matches!(item, BlockId::RawCod | BlockId::RawSalmon),
            MobKind::Cod | MobKind::Salmon | MobKind::TropicalFish | MobKind::Pufferfish => {
                matches!(item, BlockId::WheatSeeds | BlockId::Kelp | BlockId::Seagrass)
            }
            MobKind::Villager => matches!(item, BlockId::Emerald | BlockId::EmeraldBlock | BlockId::Bread),
            MobKind::IronGolem => matches!(item, BlockId::IronIngot | BlockId::IronBlock),
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        self.spec().display_name
    }

    pub fn idle_sound(&self) -> Option<SoundId> {
        self.spec().idle_sound
    }

    pub fn hurt_sound(&self) -> SoundId {
        self.spec().hurt_sound
    }

    pub fn death_sound(&self) -> SoundId {
        self.spec().death_sound
    }
}
