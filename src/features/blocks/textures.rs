//! Compile-time face-to-texture mapping table (`FaceTexturePattern`) and `BlockId::texture_id`.
//! Maps each vanilla `BlockId` + `Face` into `TextureId` without a 180-arm `match` in hot paths.

use crate::features::rendering::atlas::TextureId;

use super::constants::VANILLA_BLOCK_COUNT;
use super::id::{BlockId, Face};

#[derive(Copy, Clone)]
enum FaceTexturePattern {
    Uniform(TextureId),
    TopBottomSides(TextureId, TextureId, TextureId),
    TopSides(TextureId, TextureId),
    TopBottomCap(TextureId, TextureId),
    CubeSides {
        top: TextureId,
        bottom: TextureId,
        front_back: TextureId,
        east_west: TextureId,
    },
}

impl FaceTexturePattern {
    #[inline(always)]
    const fn eval(self, face: Face) -> TextureId {
        match self {
            FaceTexturePattern::Uniform(tex) => tex,
            FaceTexturePattern::TopBottomSides(top, bottom, sides) => match face {
                Face::Top => top,
                Face::Bottom => bottom,
                _ => sides,
            },
            FaceTexturePattern::TopSides(top, sides) => match face {
                Face::Top => top,
                _ => sides,
            },
            FaceTexturePattern::TopBottomCap(cap, sides) => match face {
                Face::Top | Face::Bottom => cap,
                _ => sides,
            },
            FaceTexturePattern::CubeSides {
                top,
                bottom,
                front_back,
                east_west,
            } => match face {
                Face::Top => top,
                Face::Bottom => bottom,
                Face::North | Face::South => front_back,
                Face::East | Face::West => east_west,
            },
        }
    }
}

const fn build_face_textures() -> [FaceTexturePattern; VANILLA_BLOCK_COUNT] {
    use BlockId as B;
    use FaceTexturePattern::*;
    use TextureId as T;

    let mut t = [Uniform(T::Planks); VANILLA_BLOCK_COUNT];
    macro_rules! set {
        ($blk:expr, $pat:expr) => {
            t[$blk.to_u8() as usize] = $pat;
        };
    }

    set!(B::Grass, TopBottomSides(T::GrassTop, T::Dirt, T::GrassSide));
    set!(B::Dirt, Uniform(T::Dirt));
    set!(B::Stone, Uniform(T::Stone));
    set!(B::StoneSlab, Uniform(T::Stone));
    set!(B::StoneStairs, Uniform(T::Stone));
    set!(B::Sand, Uniform(T::Sand));
    set!(B::Wood, TopBottomCap(T::WoodTop, T::WoodSide));
    set!(B::Leaves, Uniform(T::Leaves));
    set!(B::CherryLeaves, Uniform(T::CherryLeaves));
    set!(B::AutumnLeaves, Uniform(T::AutumnLeaves));
    set!(B::Mud, Uniform(T::Mud));
    set!(B::Water, Uniform(T::Water));
    set!(B::Snow, Uniform(T::Snow));
    set!(B::Cactus, TopBottomCap(T::CactusTop, T::CactusSide));
    set!(B::CoalOre, Uniform(T::CoalOre));
    set!(B::IronOre, Uniform(T::IronOre));
    set!(B::GoldOre, Uniform(T::GoldOre));
    set!(B::DiamondOre, Uniform(T::DiamondOre));
    set!(B::IronBlock, Uniform(T::IronBlock));
    set!(B::GoldBlock, Uniform(T::GoldBlock));
    set!(B::DiamondBlock, Uniform(T::DiamondBlock));
    set!(B::CoalBlock, Uniform(T::CoalBlock));
    set!(B::RawChicken, Uniform(T::RawChicken));
    set!(B::CookedChicken, Uniform(T::CookedChicken));
    set!(B::Feather, Uniform(T::Feather));
    set!(B::PigSpawnEgg, Uniform(T::PigSpawnEgg));
    set!(B::CowSpawnEgg, Uniform(T::CowSpawnEgg));
    set!(B::ChickenSpawnEgg, Uniform(T::ChickenSpawnEgg));
    set!(B::Bedrock, Uniform(T::Bedrock));
    set!(B::Lava, Uniform(T::Lava));
    set!(B::SnowLeaves, Uniform(T::SnowLeaves));
    set!(B::Planks, Uniform(T::Planks));
    set!(B::WoodenSlab, Uniform(T::Planks));
    set!(B::WoodenStairs, Uniform(T::Planks));
    set!(B::Door, Uniform(T::Door));
    set!(B::DoorOpen, Uniform(T::Door));
    set!(B::Glass, Uniform(T::Glass));
    set!(B::Ice, Uniform(T::Ice));
    set!(B::Tnt, TopBottomCap(T::TntTop, T::TntSide));
    set!(
        B::CraftingTable,
        CubeSides {
            top: T::CraftingTableTop,
            bottom: T::Planks,
            front_back: T::CraftingTableFront,
            east_west: T::CraftingTableSide,
        }
    );
    set!(
        B::Chest,
        CubeSides {
            top: T::ChestTop,
            bottom: T::ChestTop,
            front_back: T::ChestFront,
            east_west: T::ChestSide,
        }
    );
    set!(
        B::Furnace,
        CubeSides {
            top: T::FurnaceTop,
            bottom: T::FurnaceTop,
            front_back: T::FurnaceFront,
            east_west: T::FurnaceSide,
        }
    );
    set!(
        B::FurnaceLit,
        CubeSides {
            top: T::FurnaceTop,
            bottom: T::FurnaceTop,
            front_back: T::FurnaceFrontLit,
            east_west: T::FurnaceSide,
        }
    );
    set!(B::Torch, Uniform(T::Torch));
    set!(B::RawPorkchop, Uniform(T::RawPorkchop));
    set!(B::CookedPorkchop, Uniform(T::CookedPorkchop));
    set!(B::RawBeef, Uniform(T::RawBeef));
    set!(B::CookedBeef, Uniform(T::CookedBeef));
    set!(B::Apple, Uniform(T::Apple));
    set!(B::IronIngot, Uniform(T::IronIngot));
    set!(B::GoldIngot, Uniform(T::GoldIngot));
    set!(B::IronOunce, Uniform(T::IronOunce));
    set!(B::GoldOunce, Uniform(T::GoldOunce));
    set!(B::Diamond, Uniform(T::Diamond));
    set!(B::Bed, TopSides(T::BedTop, T::BedSide));
    set!(B::Ladder, Uniform(T::Ladder));
    set!(B::Bone, Uniform(T::Bone));
    set!(B::Gunpowder, Uniform(T::Gunpowder));
    set!(B::Farmland, TopBottomSides(T::Farmland, T::Dirt, T::FarmlandSide));
    set!(B::FarmlandWet, TopBottomSides(T::FarmlandWet, T::Dirt, T::FarmlandSide));
    set!(B::WheatSeeds, Uniform(T::WheatSeeds));
    set!(B::WheatCrop, Uniform(T::WheatCrop));
    set!(B::Wheat, Uniform(T::Wheat));
    set!(B::Bread, Uniform(T::Bread));
    set!(B::Carrot, Uniform(T::Carrot));
    set!(B::CarrotCrop, Uniform(T::CarrotCrop));
    set!(B::Potato, Uniform(T::Potato));
    set!(B::PotatoCrop, Uniform(T::PotatoCrop));
    set!(B::BakedPotato, Uniform(T::BakedPotato));
    set!(B::Sapling, Uniform(T::Sapling));
    set!(B::BoneMeal, Uniform(T::BoneMeal));
    set!(B::RustStoneOre, Uniform(T::RustStoneOre));
    set!(B::RustStone, Uniform(T::RustStone));
    set!(B::RustStoneWireLit, Uniform(T::RustStoneWireLit));
    set!(B::RustStoneBlock, Uniform(T::RustStoneBlock));
    set!(B::RustStoneLamp, Uniform(T::RustStoneLamp));
    set!(B::RustStoneLampLit, Uniform(T::RustStoneLampLit));
    set!(B::Lever, Uniform(T::Lever));
    set!(B::LeverOn, Uniform(T::Lever));
    set!(B::Button, Uniform(T::Button));
    set!(B::ButtonPressed, Uniform(T::Button));
    set!(B::PressurePlate, Uniform(T::PressurePlate));
    set!(B::PressurePlatePressed, Uniform(T::PressurePlate));
    set!(B::WhiteWool, Uniform(T::WhiteWool));
    set!(B::OrangeWool, Uniform(T::OrangeWool));
    set!(B::MagentaWool, Uniform(T::MagentaWool));
    set!(B::LightBlueWool, Uniform(T::LightBlueWool));
    set!(B::YellowWool, Uniform(T::YellowWool));
    set!(B::LimeWool, Uniform(T::LimeWool));
    set!(B::PinkWool, Uniform(T::PinkWool));
    set!(B::GrayWool, Uniform(T::GrayWool));
    set!(B::LightGrayWool, Uniform(T::LightGrayWool));
    set!(B::CyanWool, Uniform(T::CyanWool));
    set!(B::PurpleWool, Uniform(T::PurpleWool));
    set!(B::BlueWool, Uniform(T::BlueWool));
    set!(B::BrownWool, Uniform(T::BrownWool));
    set!(B::GreenWool, Uniform(T::GreenWool));
    set!(B::RedWool, Uniform(T::RedWool));
    set!(B::BlackWool, Uniform(T::BlackWool));
    set!(B::SheepSpawnEgg, Uniform(T::SheepSpawnEgg));
    set!(B::Shears, Uniform(T::Shears));
    set!(B::Kelp, Uniform(T::Kelp));
    set!(B::Seagrass, Uniform(T::Seagrass));
    set!(B::RawCod, Uniform(T::RawCod));
    set!(B::CookedCod, Uniform(T::CookedCod));
    set!(B::RawSalmon, Uniform(T::RawSalmon));
    set!(B::CookedSalmon, Uniform(T::CookedSalmon));
    set!(B::Pufferfish, Uniform(T::Pufferfish));
    set!(B::TropicalFish, Uniform(T::TropicalFish));
    set!(B::DriedKelp, Uniform(T::DriedKelp));
    set!(B::DolphinSpawnEgg, Uniform(T::DolphinSpawnEgg));
    set!(B::FishSpawnEgg, Uniform(T::FishSpawnEgg));
    set!(B::FishingRod, Uniform(T::FishingRod));
    set!(B::Emerald, Uniform(T::Emerald));
    set!(B::EmeraldOre, Uniform(T::EmeraldOre));
    set!(B::EmeraldBlock, Uniform(T::EmeraldBlock));
    set!(B::VillagerSpawnEgg, Uniform(T::VillagerSpawnEgg));
    set!(B::IronGolemSpawnEgg, Uniform(T::IronGolemSpawnEgg));
    set!(B::SpiderSpawnEgg, Uniform(T::SpiderSpawnEgg));
    set!(B::StringItem, Uniform(T::StringItem));
    set!(B::SpiderEye, Uniform(T::SpiderEye));
    set!(B::Clay, Uniform(T::Clay));
    set!(B::ClayBall, Uniform(T::ClayBall));
    set!(B::Brick, Uniform(T::Brick));
    set!(B::Bricks, Uniform(T::Bricks));
    set!(B::BrickStairs, Uniform(T::Bricks));
    set!(B::BrickSlab, Uniform(T::Bricks));
    set!(B::Gravel, Uniform(T::Gravel));
    set!(B::Flint, Uniform(T::Flint));
    set!(B::Cobblestone, Uniform(T::Cobblestone));
    set!(B::CobblestoneStairs, Uniform(T::Cobblestone));
    set!(B::CobblestoneSlab, Uniform(T::Cobblestone));
    set!(B::MossyCobblestone, Uniform(T::MossyCobblestone));
    set!(B::Sandstone, TopBottomSides(T::SandstoneTop, T::SandstoneBottom, T::SandstoneSide));
    set!(B::SandstoneStairs, TopBottomSides(T::SandstoneTop, T::SandstoneBottom, T::SandstoneSide));
    set!(B::SandstoneSlab, TopBottomSides(T::SandstoneTop, T::SandstoneBottom, T::SandstoneSide));
    set!(B::SmoothStone, Uniform(T::SmoothStone));
    set!(B::Obsidian, Uniform(T::Obsidian));
    set!(B::Bookshelf, TopBottomCap(T::BookshelfTop, T::BookshelfSide));

    t
}

const FACE_TEXTURES: [FaceTexturePattern; VANILLA_BLOCK_COUNT] = build_face_textures();

impl BlockId {
    /// Which procedural texture to use for a given face of this block.
    pub fn texture_id(self, face: Face) -> TextureId {
        match self {
            BlockId::Custom(id) => crate::engine::modding::get_custom_texture_id(id),
            _ => FACE_TEXTURES[self.to_u8() as usize].eval(face),
        }
    }
}
