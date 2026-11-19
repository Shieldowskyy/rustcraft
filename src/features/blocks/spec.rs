//! Compile-time physical, optical, and mining metadata table (`BlockSpec`, `VANILLA_SPECS`).
//! Centralizes block attributes (`is_solid`, `is_transparent`, `break_time`, etc.) outside giant `match` blocks.

use super::constants::{VANILLA_BLOCK_COUNT, WOOL_BREAK_TIME_SECS};
use super::id::BlockId;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MiningCategory {
    None,
    PickaxeTier0,
    PickaxeTier2,
    PickaxeObsidian,
    AxeTarget,
    ShovelTarget,
    HoeCropTarget,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlockSpec {
    pub solid: bool,
    pub transparent: bool,
    pub placeable: bool,
    pub targetable: bool,
    pub leaves: bool,
    pub half_block: bool,
    pub stairs: bool,
    pub crop: bool,
    pub break_time: f32,
    pub required_pickaxe_tier: Option<u8>,
    pub mining_category: MiningCategory,
}

const DEFAULT_SPEC: BlockSpec = BlockSpec {
    solid: true,
    transparent: false,
    placeable: true,
    targetable: true,
    leaves: false,
    half_block: false,
    stairs: false,
    crop: false,
    break_time: 0.0,
    required_pickaxe_tier: None,
    mining_category: MiningCategory::None,
};

const fn set_non_solid(specs: &mut [BlockSpec; VANILLA_BLOCK_COUNT]) {
    use BlockId as B;
    let non_solid = [
        B::Air, B::Water, B::Lava, B::Stick,
        B::WoodenPickaxe, B::StonePickaxe, B::IronPickaxe,
        B::WoodenAxe, B::StoneAxe, B::IronAxe,
        B::WoodenShovel, B::StoneShovel, B::IronShovel,
        B::WoodenSword, B::StoneSword, B::IronSword,
        B::Torch, B::RawPorkchop, B::CookedPorkchop, B::RawBeef, B::CookedBeef, B::Apple, B::DoorOpen,
        B::IronIngot, B::GoldIngot, B::IronOunce, B::GoldOunce,
        B::RawChicken, B::CookedChicken, B::Feather, B::PigSpawnEgg, B::CowSpawnEgg, B::ChickenSpawnEgg,
        B::Diamond, B::DiamondPickaxe, B::DiamondAxe, B::DiamondShovel, B::DiamondSword,
        B::GoldPickaxe, B::GoldAxe, B::GoldShovel, B::GoldSword,
        B::Ladder, B::WoodenHoe, B::StoneHoe, B::IronHoe, B::DiamondHoe, B::GoldHoe,
        B::WheatSeeds, B::WheatCrop, B::Wheat, B::Bread, B::Carrot, B::CarrotCrop,
        B::Potato, B::PotatoCrop, B::BakedPotato, B::Sapling, B::BoneMeal,
        B::RustStone, B::RustStoneWireLit,
        B::Lever, B::LeverOn, B::Button, B::ButtonPressed, B::PressurePlate, B::PressurePlatePressed,
        B::SheepSpawnEgg, B::Shears, B::Kelp, B::Seagrass,
        B::RawCod, B::CookedCod, B::RawSalmon, B::CookedSalmon, B::Pufferfish, B::TropicalFish,
        B::DriedKelp, B::DolphinSpawnEgg, B::FishSpawnEgg, B::FishingRod,
        B::Emerald, B::VillagerSpawnEgg, B::IronGolemSpawnEgg, B::ClayBall, B::Brick, B::Flint,
        B::Leather, B::LeatherHelmet, B::LeatherChestplate, B::LeatherLeggings, B::LeatherBoots,
        B::IronHelmet, B::IronChestplate, B::IronLeggings, B::IronBoots,
        B::GoldHelmet, B::GoldChestplate, B::GoldLeggings, B::GoldBoots,
        B::DiamondHelmet, B::DiamondChestplate, B::DiamondLeggings, B::DiamondBoots,
        B::Bow, B::Arrow, B::Shield, B::StringItem, B::SpiderEye, B::SpiderSpawnEgg,
    ];
    let mut i = 0;
    while i < non_solid.len() {
        let idx = non_solid[i].to_u8() as usize;
        specs[idx].solid = false;
        specs[idx].targetable = false;
        i += 1;
    }
}

const fn set_transparent_and_flags(specs: &mut [BlockSpec; VANILLA_BLOCK_COUNT]) {
    use BlockId as B;
    let transparent = [
        B::Air, B::Leaves, B::Water, B::Lava, B::SnowLeaves, B::Door, B::Glass,
        B::CherryLeaves, B::AutumnLeaves, B::Torch, B::DoorOpen, B::Bed,
        B::WoodenSlab, B::StoneSlab, B::WoodenStairs, B::StoneStairs, B::Ladder,
        B::WheatCrop, B::CarrotCrop, B::PotatoCrop, B::Sapling,
        B::RustStone, B::RustStoneWireLit,
        B::Lever, B::LeverOn, B::Button, B::ButtonPressed, B::PressurePlate, B::PressurePlatePressed,
        B::Kelp, B::Seagrass, B::RawCod, B::CookedCod, B::RawSalmon, B::CookedSalmon,
        B::Pufferfish, B::TropicalFish, B::DriedKelp, B::DolphinSpawnEgg, B::FishSpawnEgg, B::FishingRod,
        B::ClayBall, B::Brick, B::Flint, B::BrickStairs, B::BrickSlab,
        B::CobblestoneStairs, B::CobblestoneSlab, B::SandstoneStairs, B::SandstoneSlab,
        B::Leather, B::LeatherHelmet, B::LeatherChestplate, B::LeatherLeggings, B::LeatherBoots,
        B::IronHelmet, B::IronChestplate, B::IronLeggings, B::IronBoots,
        B::GoldHelmet, B::GoldChestplate, B::GoldLeggings, B::GoldBoots,
        B::DiamondHelmet, B::DiamondChestplate, B::DiamondLeggings, B::DiamondBoots,
        B::Bow, B::Arrow, B::Shield,
    ];
    let mut i = 0;
    while i < transparent.len() {
        specs[transparent[i].to_u8() as usize].transparent = true;
        i += 1;
    }

    let extra_targetable = [
        B::DoorOpen, B::Torch, B::Ladder, B::WheatCrop, B::CarrotCrop, B::PotatoCrop, B::Sapling,
        B::RustStone, B::RustStoneWireLit,
        B::Lever, B::LeverOn, B::Button, B::ButtonPressed, B::PressurePlate, B::PressurePlatePressed,
        B::Kelp, B::Seagrass,
    ];
    i = 0;
    while i < extra_targetable.len() {
        specs[extra_targetable[i].to_u8() as usize].targetable = true;
        i += 1;
    }

    let non_placeable = [
        B::Air, B::DoorOpen, B::Stick,
        B::WoodenPickaxe, B::StonePickaxe, B::IronPickaxe,
        B::WoodenAxe, B::StoneAxe, B::IronAxe,
        B::WoodenShovel, B::StoneShovel, B::IronShovel,
        B::WoodenSword, B::StoneSword, B::IronSword,
        B::WoodenHoe, B::StoneHoe, B::IronHoe, B::DiamondHoe, B::GoldHoe,
        B::RawPorkchop, B::CookedPorkchop, B::RawBeef, B::CookedBeef, B::Apple,
        B::IronIngot, B::GoldIngot, B::IronOunce, B::GoldOunce,
        B::RawChicken, B::CookedChicken, B::Feather, B::PigSpawnEgg, B::CowSpawnEgg, B::ChickenSpawnEgg,
        B::Diamond, B::DiamondPickaxe, B::DiamondAxe, B::DiamondShovel, B::DiamondSword,
        B::GoldPickaxe, B::GoldAxe, B::GoldShovel, B::GoldSword,
        B::Bone, B::Gunpowder, B::WheatSeeds, B::Wheat, B::Bread, B::Carrot, B::Potato, B::BakedPotato, B::BoneMeal,
        B::RustStoneWireLit, B::RustStoneLampLit, B::LeverOn, B::ButtonPressed, B::PressurePlatePressed,
        B::Shears, B::SheepSpawnEgg,
        B::RawCod, B::CookedCod, B::RawSalmon, B::CookedSalmon, B::Pufferfish, B::TropicalFish,
        B::DriedKelp, B::DolphinSpawnEgg, B::FishSpawnEgg, B::FishingRod,
        B::Emerald, B::VillagerSpawnEgg, B::IronGolemSpawnEgg, B::ClayBall, B::Brick, B::Flint,
        B::Leather, B::LeatherHelmet, B::LeatherChestplate, B::LeatherLeggings, B::LeatherBoots,
        B::IronHelmet, B::IronChestplate, B::IronLeggings, B::IronBoots,
        B::GoldHelmet, B::GoldChestplate, B::GoldLeggings, B::GoldBoots,
        B::DiamondHelmet, B::DiamondChestplate, B::DiamondLeggings, B::DiamondBoots,
        B::Bow, B::Arrow, B::Shield, B::StringItem, B::SpiderEye, B::SpiderSpawnEgg,
    ];
    i = 0;
    while i < non_placeable.len() {
        specs[non_placeable[i].to_u8() as usize].placeable = false;
        i += 1;
    }

    let mut k = 0;
    let leaves = [B::Leaves, B::SnowLeaves, B::CherryLeaves, B::AutumnLeaves];
    while k < leaves.len() { specs[leaves[k].to_u8() as usize].leaves = true; k += 1; }

    k = 0;
    let slabs = [B::WoodenSlab, B::StoneSlab, B::BrickSlab, B::CobblestoneSlab, B::SandstoneSlab, B::Bed];
    while k < slabs.len() { specs[slabs[k].to_u8() as usize].half_block = true; k += 1; }

    k = 0;
    let stairs = [B::WoodenStairs, B::StoneStairs, B::BrickStairs, B::CobblestoneStairs, B::SandstoneStairs];
    while k < stairs.len() { specs[stairs[k].to_u8() as usize].stairs = true; k += 1; }

    k = 0;
    let crops = [B::WheatCrop, B::CarrotCrop, B::PotatoCrop];
    while k < crops.len() { specs[crops[k].to_u8() as usize].crop = true; k += 1; }
}

const fn set_break_and_mining(specs: &mut [BlockSpec; VANILLA_BLOCK_COUNT]) {
    use BlockId as B;
    use MiningCategory as M;

    macro_rules! assign_field {
        ($field:ident = $val:expr; [$($b:expr),* $(,)?]) => {{
            let arr = [$($b),*];
            let mut idx = 0;
            while idx < arr.len() {
                specs[arr[idx].to_u8() as usize].$field = $val;
                idx += 1;
            }
        }};
    }

    let mut w = B::WhiteWool.to_u8() as usize;
    while w <= B::BlackWool.to_u8() as usize {
        specs[w].break_time = WOOL_BREAK_TIME_SECS;
        w += 1;
    }

    let break_times: [(BlockId, f32); 67] = [
        (B::Grass, 0.6), (B::Dirt, 0.5), (B::Stone, 1.5), (B::Sand, 0.5), (B::Wood, 2.0),
        (B::Leaves, 0.2), (B::CherryLeaves, 0.2), (B::AutumnLeaves, 0.2), (B::Snow, 0.2), (B::Cactus, 0.4),
        (B::CoalOre, 1.5), (B::IronOre, 1.5), (B::GoldOre, 3.0), (B::DiamondOre, 3.0), (B::EmeraldOre, 3.0),
        (B::IronBlock, 5.0), (B::GoldBlock, 3.0), (B::DiamondBlock, 5.0), (B::EmeraldBlock, 5.0), (B::CoalBlock, 5.0),
        (B::Bedrock, f32::INFINITY), (B::SnowLeaves, 0.2), (B::Planks, 1.5), (B::CraftingTable, 1.5), (B::Chest, 2.0),
        (B::Furnace, 3.5), (B::FurnaceLit, 3.5), (B::Door, 1.5), (B::DoorOpen, 1.5), (B::Glass, 0.3), (B::Ice, 0.5),
        (B::Mud, 0.6), (B::Bed, 0.4), (B::WoodenSlab, 1.0), (B::StoneSlab, 1.0), (B::WoodenStairs, 1.2), (B::StoneStairs, 1.2),
        (B::Ladder, 0.4), (B::Farmland, 0.6), (B::FarmlandWet, 0.6),
        (B::WheatCrop, 0.05), (B::CarrotCrop, 0.05), (B::PotatoCrop, 0.05), (B::Sapling, 0.05),
        (B::Kelp, 0.05), (B::Seagrass, 0.05), (B::RustStoneOre, 3.0), (B::RustStoneBlock, 5.0),
        (B::RustStoneLamp, 0.3), (B::RustStoneLampLit, 0.3), (B::RustStone, 0.05), (B::RustStoneWireLit, 0.05),
        (B::Lever, 0.5), (B::LeverOn, 0.5), (B::Button, 0.5), (B::ButtonPressed, 0.5),
        (B::PressurePlate, 0.5), (B::PressurePlatePressed, 0.5), (B::Clay, 0.6), (B::Gravel, 0.6),
        (B::Cobblestone, 2.0), (B::MossyCobblestone, 2.0), (B::CobblestoneStairs, 2.0), (B::CobblestoneSlab, 2.0),
        (B::Bricks, 2.0), (B::BrickStairs, 2.0), (B::BrickSlab, 2.0),
    ];
    let mut i = 0;
    while i < break_times.len() {
        specs[break_times[i].0.to_u8() as usize].break_time = break_times[i].1;
        i += 1;
    }
    assign_field!(break_time = 0.8; [B::Sandstone, B::SandstoneStairs, B::SandstoneSlab]);
    specs[B::SmoothStone.to_u8() as usize].break_time = 2.0;
    specs[B::Obsidian.to_u8() as usize].break_time = 9.0;
    specs[B::Bookshelf.to_u8() as usize].break_time = 1.5;

    assign_field!(required_pickaxe_tier = Some(0); [B::Stone, B::Cobblestone, B::CoalOre]);
    specs[B::IronOre.to_u8() as usize].required_pickaxe_tier = Some(1);
    assign_field!(required_pickaxe_tier = Some(2); [B::GoldOre, B::DiamondOre, B::EmeraldOre, B::RustStoneOre]);

    assign_field!(mining_category = M::PickaxeTier0; [
        B::Stone, B::CoalOre, B::IronOre, B::Ice, B::Furnace, B::FurnaceLit, B::IronBlock, B::GoldBlock, B::GoldOre,
        B::CoalBlock, B::StoneSlab, B::StoneStairs, B::RustStoneOre, B::RustStoneBlock, B::RustStoneLamp, B::RustStoneLampLit,
        B::Cobblestone, B::MossyCobblestone, B::CobblestoneStairs, B::CobblestoneSlab, B::Bricks, B::BrickStairs, B::BrickSlab,
        B::Sandstone, B::SandstoneStairs, B::SandstoneSlab, B::SmoothStone,
    ]);
    assign_field!(mining_category = M::PickaxeTier2; [B::DiamondOre, B::DiamondBlock, B::EmeraldOre, B::EmeraldBlock]);
    specs[B::Obsidian.to_u8() as usize].mining_category = M::PickaxeObsidian;
    assign_field!(mining_category = M::AxeTarget; [B::Wood, B::Planks, B::CraftingTable, B::Door, B::Cactus, B::Chest, B::WoodenSlab, B::WoodenStairs, B::Bed, B::Ladder, B::Bookshelf]);
    assign_field!(mining_category = M::ShovelTarget; [B::Dirt, B::Grass, B::Sand, B::Snow, B::Mud, B::Farmland, B::FarmlandWet, B::Clay, B::Gravel]);
    assign_field!(mining_category = M::HoeCropTarget; [B::WheatCrop, B::CarrotCrop, B::PotatoCrop, B::Sapling]);
}

const fn build_vanilla_specs() -> [BlockSpec; VANILLA_BLOCK_COUNT] {
    let mut specs = [DEFAULT_SPEC; VANILLA_BLOCK_COUNT];
    set_non_solid(&mut specs);
    set_transparent_and_flags(&mut specs);
    set_break_and_mining(&mut specs);
    specs
}

pub const VANILLA_SPECS: [BlockSpec; VANILLA_BLOCK_COUNT] = build_vanilla_specs();

impl BlockId {
    #[inline(always)]
    pub const fn spec(self) -> &'static BlockSpec {
        let id = self.to_u8() as usize;
        if id < VANILLA_BLOCK_COUNT {
            &VANILLA_SPECS[id]
        } else {
            &DEFAULT_SPEC
        }
    }

    pub fn pick_item(self) -> Option<BlockId> {
        if matches!(self, BlockId::Air | BlockId::Water | BlockId::Lava) {
            return None;
        }
        let item = match self {
            BlockId::RustStoneWireLit => BlockId::RustStone,
            BlockId::RustStoneLampLit => BlockId::RustStoneLamp,
            BlockId::LeverOn => BlockId::Lever,
            BlockId::ButtonPressed => BlockId::Button,
            BlockId::PressurePlatePressed => BlockId::PressurePlate,
            BlockId::DoorOpen => BlockId::Door,
            BlockId::FurnaceLit => BlockId::Furnace,
            BlockId::Farmland | BlockId::FarmlandWet => BlockId::Dirt,
            BlockId::WheatCrop => BlockId::WheatSeeds,
            BlockId::CarrotCrop => BlockId::Carrot,
            BlockId::PotatoCrop => BlockId::Potato,
            other => other,
        };
        Some(item)
    }

    pub fn is_placeable(self) -> bool {
        match self {
            BlockId::Custom(id) => !crate::engine::modding::is_custom_item_only(id),
            _ => self.spec().placeable,
        }
    }

    #[inline(always)]
    pub fn is_leaves(self) -> bool { self.spec().leaves }

    #[inline(always)]
    pub fn is_wood(self) -> bool { matches!(self, BlockId::Wood) }

    #[inline(always)]
    pub fn is_half_block(self) -> bool { self.spec().half_block }

    #[inline(always)]
    pub fn is_stairs(self) -> bool { self.spec().stairs }

    #[inline(always)]
    pub fn is_ladder(self) -> bool { matches!(self, BlockId::Ladder) }

    #[inline(always)]
    pub fn is_crop(self) -> bool { self.spec().crop }

    #[inline(always)]
    pub fn is_bedrock(self) -> bool { matches!(self, BlockId::Bedrock) }

    #[inline(always)]
    pub fn is_solid(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_solid(id),
            _ => self.spec().solid,
        }
    }

    #[inline(always)]
    pub fn is_targetable(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_solid(id),
            _ => self.spec().targetable,
        }
    }

    #[inline(always)]
    pub fn is_transparent(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_transparent(id),
            _ => self.spec().transparent,
        }
    }

    #[inline(always)]
    pub fn is_opaque(self) -> bool {
        self.is_solid() && !self.is_transparent()
    }

    #[inline(always)]
    pub fn is_leaf(self) -> bool { self.spec().leaves }

    /// Break time in seconds (like Minecraft hardness). Creative breaks instantly.
    pub fn break_time(self) -> f32 {
        match self {
            BlockId::Custom(_) => 0.0,
            _ => self.spec().break_time,
        }
    }

    pub fn spawn_mob_kind(self) -> Option<crate::gameplay::mob::MobKind> {
        use crate::gameplay::mob::MobKind;
        match self {
            BlockId::PigSpawnEgg => Some(MobKind::Pig),
            BlockId::CowSpawnEgg => Some(MobKind::Cow),
            BlockId::ChickenSpawnEgg => Some(MobKind::Chicken),
            BlockId::SheepSpawnEgg => Some(MobKind::Sheep),
            BlockId::DolphinSpawnEgg => Some(MobKind::Dolphin),
            BlockId::FishSpawnEgg => Some(MobKind::Cod),
            BlockId::VillagerSpawnEgg => Some(MobKind::Villager),
            BlockId::IronGolemSpawnEgg => Some(MobKind::IronGolem),
            BlockId::SpiderSpawnEgg => Some(MobKind::Spider),
            _ => None,
        }
    }
}
