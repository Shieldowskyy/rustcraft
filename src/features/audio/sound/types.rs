//! Identyfikatory efektów dźwiękowych (`SoundId`), kategorie głośności (`SoundCategory`) i bufor próbki (`SoundSample`).

use std::sync::Arc;

use crate::features::blocks::BlockId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundCategory {
    Master,
    Sfx,
    Music,
    Ambient,
    Ui,
}

impl SoundCategory {
    pub const ALL: [SoundCategory; 5] = [
        SoundCategory::Master,
        SoundCategory::Sfx,
        SoundCategory::Music,
        SoundCategory::Ambient,
        SoundCategory::Ui,
    ];

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        match self {
            SoundCategory::Master => "Master",
            SoundCategory::Sfx => "SFX",
            SoundCategory::Music => "Music",
            SoundCategory::Ambient => "Ambient",
            SoundCategory::Ui => "UI",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SoundId {
    StepGrass,
    StepDirt,
    StepStone,
    StepWood,
    StepSand,
    StepGravel,
    StepWater,
    DigGrass,
    DigDirt,
    DigStone,
    DigWood,
    DigSand,
    BreakGrass,
    BreakDirt,
    BreakStone,
    BreakWood,
    BreakGlass,
    BreakSand,
    PlaceGrass,
    PlaceDirt,
    PlaceStone,
    PlaceWood,
    PlaceSand,
    PlaceGeneric,
    PlayerJump,
    PlayerLand,
    PlayerHurt,
    PlayerSplash,
    PlayerEat,
    ItemPop,
    UiClick,
    UiHover,
    ChestOpen,
    ChestClose,
    DoorOpen,
    DoorClose,
    Explosion,
    PigIdle,
    PigHurt,
    PigDeath,
    CowIdle,
    CowHurt,
    CowDeath,
    ChickenIdle,
    ChickenHurt,
    ChickenDeath,
    ZombieIdle,
    ZombieHurt,
    ZombieDeath,
    SkeletonIdle,
    SkeletonHurt,
    SkeletonDeath,
    ReaperHiss,
    ReaperHurt,
    ReaperDeath,
    SheepIdle,
    SheepHurt,
    SheepDeath,
    SheepShear,
    VillagerIdle,
    VillagerHurt,
    VillagerDeath,
    VillagerTrade,
    VillagerNo,
    IronGolemIdle,
    IronGolemHurt,
    IronGolemDeath,
    IronGolemAttack,
    FishFlop,
    DolphinIdle,
    DolphinHurt,
    DolphinDeath,
    DolphinJump,
    FishingCast,
    FishingReel,
    FishCatch,
    AmbientWind,
    AmbientCave,
    AmbientWater,
    AmbientLeaves,
    AmbientCaveRocks,
    StepWoodCreak,
    AmbientDesertSand,
    Rain,
    ThunderClose,
    ThunderDistant,
    BowShoot,
    BowHit,
    ShieldBlock,
    ArmorEquip,
    SpiderIdle,
    SpiderHurt,
    SpiderDeath,
    ToolBreak,
}

impl SoundId {
    #[inline(always)]
    pub fn category(&self) -> SoundCategory {
        match self {
            SoundId::UiClick | SoundId::UiHover => SoundCategory::Ui,
            SoundId::AmbientWind
            | SoundId::AmbientCave
            | SoundId::AmbientWater
            | SoundId::AmbientLeaves
            | SoundId::AmbientCaveRocks
            | SoundId::AmbientDesertSand
            | SoundId::Rain
            | SoundId::ThunderClose
            | SoundId::ThunderDistant => SoundCategory::Ambient,
            _ => SoundCategory::Sfx,
        }
    }

    #[inline]
    pub fn from_block_step(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::StepGrass;
        }
        match block {
            BlockId::Grass
            | BlockId::Leaves
            | BlockId::CherryLeaves
            | BlockId::AutumnLeaves
            | BlockId::SnowLeaves => SoundId::StepGrass,
            BlockId::Dirt | BlockId::Mud | BlockId::Clay => SoundId::StepDirt,
            BlockId::Gravel => SoundId::StepGravel,
            BlockId::Stone
            | BlockId::Bedrock
            | BlockId::CoalOre
            | BlockId::IronOre
            | BlockId::GoldOre
            | BlockId::Furnace
            | BlockId::FurnaceLit
            | BlockId::IronBlock
            | BlockId::GoldBlock
            | BlockId::CoalBlock => SoundId::StepStone,
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::DoorOpen
            | BlockId::Bookshelf => SoundId::StepWood,
            BlockId::Sand => SoundId::StepSand,
            BlockId::Water => SoundId::StepWater,
            _ => SoundId::StepStone,
        }
    }

    #[inline]
    pub fn from_block_dig(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::DigGrass;
        }
        match block {
            BlockId::Grass
            | BlockId::Leaves
            | BlockId::CherryLeaves
            | BlockId::AutumnLeaves
            | BlockId::SnowLeaves => SoundId::DigGrass,
            BlockId::Dirt | BlockId::Mud | BlockId::Clay | BlockId::Gravel => SoundId::DigDirt,
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::DoorOpen
            | BlockId::Bookshelf => SoundId::DigWood,
            BlockId::Glass => SoundId::BreakGlass,
            BlockId::Sand => SoundId::DigSand,
            _ => SoundId::DigStone,
        }
    }

    #[inline]
    pub fn from_block_break(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::BreakGrass;
        }
        match block {
            BlockId::Grass
            | BlockId::Leaves
            | BlockId::CherryLeaves
            | BlockId::AutumnLeaves
            | BlockId::SnowLeaves => SoundId::BreakGrass,
            BlockId::Dirt | BlockId::Mud | BlockId::Clay | BlockId::Gravel => SoundId::BreakDirt,
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::DoorOpen
            | BlockId::Bookshelf => SoundId::BreakWood,
            BlockId::Glass => SoundId::BreakGlass,
            BlockId::Sand => SoundId::BreakSand,
            _ => SoundId::BreakStone,
        }
    }

    #[inline]
    pub fn from_block_place(block: BlockId) -> Self {
        if block.is_wool() {
            return SoundId::PlaceGrass;
        }
        match block {
            BlockId::Grass
            | BlockId::Leaves
            | BlockId::CherryLeaves
            | BlockId::AutumnLeaves
            | BlockId::SnowLeaves => SoundId::PlaceGrass,
            BlockId::Dirt | BlockId::Mud | BlockId::Clay | BlockId::Gravel => SoundId::PlaceDirt,
            BlockId::Wood
            | BlockId::Planks
            | BlockId::CraftingTable
            | BlockId::Chest
            | BlockId::Door
            | BlockId::Bookshelf => SoundId::PlaceWood,
            BlockId::Sand => SoundId::PlaceSand,
            _ => SoundId::PlaceStone,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SoundSample {
    pub data: Arc<[f32]>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl SoundSample {
    pub fn new(data: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self {
            data: data.into(),
            sample_rate,
            channels: channels.max(1),
        }
    }

    #[inline]
    pub fn duration_seconds(&self) -> f32 {
        if self.sample_rate == 0 || self.channels == 0 {
            0.0
        } else {
            (self.data.len() / self.channels as usize) as f32 / self.sample_rate as f32
        }
    }
}
