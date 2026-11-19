//! Compile-time face color lookup table (`FaceColorPattern`) and `BlockId::face_color`.
//! Replaces the monolithic `match self` color switch with a static `VANILLA_BLOCK_COUNT` table.

use super::constants::VANILLA_BLOCK_COUNT;
use super::id::{BlockId, Face};

#[derive(Copy, Clone)]
enum FaceColorPattern {
    Uniform([f32; 3]),
    TopBottomSides([f32; 3], [f32; 3], [f32; 3]),
    TopSides([f32; 3], [f32; 3]),
    TopBottomCap([f32; 3], [f32; 3]),
    FrontBackSides([f32; 3], [f32; 3]),
}

impl FaceColorPattern {
    #[inline(always)]
    const fn eval(self, face: Face) -> [f32; 3] {
        match self {
            FaceColorPattern::Uniform(c) => c,
            FaceColorPattern::TopBottomSides(top, bottom, sides) => match face {
                Face::Top => top,
                Face::Bottom => bottom,
                _ => sides,
            },
            FaceColorPattern::TopSides(top, sides) => match face {
                Face::Top => top,
                _ => sides,
            },
            FaceColorPattern::TopBottomCap(cap, sides) => match face {
                Face::Top | Face::Bottom => cap,
                _ => sides,
            },
            FaceColorPattern::FrontBackSides(fb, other) => match face {
                Face::North | Face::South => fb,
                _ => other,
            },
        }
    }
}

const fn build_face_colors() -> [FaceColorPattern; VANILLA_BLOCK_COUNT] {
    use BlockId as B;
    use FaceColorPattern::*;

    let mut t = [Uniform([0.0, 0.0, 0.0]); VANILLA_BLOCK_COUNT];
    macro_rules! set {
        ($blk:expr, $pat:expr) => {
            t[$blk.to_u8() as usize] = $pat;
        };
    }

    set!(B::Grass, TopBottomSides([0.36, 0.68, 0.24], [0.45, 0.33, 0.20], [0.42, 0.34, 0.20]));
    set!(B::Dirt, Uniform([0.45, 0.33, 0.20]));
    set!(B::Stone, Uniform([0.55, 0.55, 0.55]));
    set!(B::Sand, Uniform([0.87, 0.80, 0.55]));
    set!(B::Wood, TopBottomCap([0.55, 0.42, 0.25], [0.42, 0.30, 0.16]));
    set!(B::Leaves, Uniform([0.20, 0.50, 0.16]));
    set!(B::CherryLeaves, Uniform([0.92, 0.55, 0.72]));
    set!(B::AutumnLeaves, Uniform([0.78, 0.35, 0.10]));
    set!(B::Mud, Uniform([0.35, 0.25, 0.15]));
    set!(B::Water, Uniform([0.15, 0.35, 0.75]));
    set!(B::Snow, Uniform([0.93, 0.95, 0.98]));
    set!(B::Cactus, TopBottomCap([0.18, 0.55, 0.15], [0.15, 0.48, 0.12]));
    set!(B::CoalOre, Uniform([0.48, 0.48, 0.48]));
    set!(B::IronOre, Uniform([0.60, 0.55, 0.50]));
    set!(B::GoldOre, Uniform([0.58, 0.55, 0.48]));
    set!(B::DiamondOre, Uniform([0.45, 0.65, 0.68]));
    set!(B::IronBlock, Uniform([0.88, 0.88, 0.90]));
    set!(B::GoldBlock, Uniform([0.96, 0.85, 0.22]));
    set!(B::DiamondBlock, Uniform([0.35, 0.95, 0.95]));
    set!(B::CoalBlock, Uniform([0.15, 0.15, 0.15]));
    set!(B::Bedrock, Uniform([0.22, 0.22, 0.22]));
    set!(B::Lava, Uniform([0.95, 0.42, 0.05]));
    set!(B::SnowLeaves, Uniform([0.55, 0.72, 0.80]));
    set!(B::Planks, Uniform([0.62, 0.42, 0.20]));
    set!(B::WoodenSlab, Uniform([0.62, 0.42, 0.20]));
    set!(B::WoodenStairs, Uniform([0.62, 0.42, 0.20]));
    set!(B::StoneSlab, Uniform([0.55, 0.55, 0.55]));
    set!(B::StoneStairs, Uniform([0.55, 0.55, 0.55]));
    set!(B::Bed, TopSides([0.85, 0.20, 0.20], [0.58, 0.35, 0.18]));
    set!(B::Ladder, Uniform([0.58, 0.40, 0.22]));
    set!(B::Door, Uniform([0.50, 0.30, 0.12]));
    set!(B::DoorOpen, Uniform([0.50, 0.30, 0.12]));
    set!(B::Glass, Uniform([0.55, 0.82, 0.95]));
    set!(B::Ice, Uniform([0.70, 0.88, 1.0]));
    set!(B::Tnt, TopBottomCap([0.75, 0.70, 0.60], [0.85, 0.15, 0.10]));
    set!(B::CraftingTable, TopBottomSides([0.68, 0.48, 0.26], [0.58, 0.38, 0.18], [0.54, 0.36, 0.18]));
    set!(B::Chest, Uniform([0.55, 0.40, 0.20]));
    set!(B::Furnace, Uniform([0.52, 0.52, 0.52]));
    set!(B::FurnaceLit, FrontBackSides([0.95, 0.55, 0.15], [0.52, 0.52, 0.52]));
    set!(B::Stick, Uniform([0.55, 0.38, 0.18]));
    set!(B::Torch, Uniform([0.98, 0.72, 0.22]));
    set!(B::RawPorkchop, Uniform([0.92, 0.55, 0.60]));
    set!(B::CookedPorkchop, Uniform([0.72, 0.45, 0.25]));
    set!(B::RawBeef, Uniform([0.85, 0.25, 0.25]));
    set!(B::CookedBeef, Uniform([0.55, 0.30, 0.18]));
    set!(B::RawChicken, Uniform([0.90, 0.70, 0.65]));
    set!(B::CookedChicken, Uniform([0.72, 0.45, 0.20]));
    set!(B::Feather, Uniform([0.95, 0.95, 0.95]));
    set!(B::PigSpawnEgg, Uniform([0.95, 0.62, 0.68]));
    set!(B::CowSpawnEgg, Uniform([0.45, 0.35, 0.28]));
    set!(B::ChickenSpawnEgg, Uniform([0.95, 0.95, 0.90]));
    set!(B::Apple, Uniform([0.90, 0.15, 0.15]));
    set!(B::IronIngot, Uniform([0.85, 0.85, 0.88]));
    set!(B::GoldIngot, Uniform([0.95, 0.85, 0.25]));
    set!(B::Diamond, Uniform([0.25, 0.90, 0.95]));
    set!(B::IronOunce, Uniform([0.82, 0.82, 0.85]));
    set!(B::GoldOunce, Uniform([0.95, 0.85, 0.25]));
    set!(B::WoodenPickaxe, Uniform([0.60, 0.42, 0.22]));
    set!(B::WoodenAxe, Uniform([0.60, 0.42, 0.22]));
    set!(B::WoodenShovel, Uniform([0.60, 0.42, 0.22]));
    set!(B::WoodenSword, Uniform([0.60, 0.42, 0.22]));
    set!(B::WoodenHoe, Uniform([0.60, 0.42, 0.22]));
    set!(B::StonePickaxe, Uniform([0.55, 0.55, 0.55]));
    set!(B::StoneAxe, Uniform([0.55, 0.55, 0.55]));
    set!(B::StoneShovel, Uniform([0.55, 0.55, 0.55]));
    set!(B::StoneSword, Uniform([0.55, 0.55, 0.55]));
    set!(B::StoneHoe, Uniform([0.55, 0.55, 0.55]));
    set!(B::IronPickaxe, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronAxe, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronShovel, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronSword, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronHoe, Uniform([0.85, 0.85, 0.88]));
    set!(B::GoldPickaxe, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldAxe, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldShovel, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldSword, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldHoe, Uniform([0.95, 0.85, 0.25]));
    set!(B::DiamondPickaxe, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondAxe, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondShovel, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondSword, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondHoe, Uniform([0.25, 0.90, 0.95]));
    set!(B::Farmland, TopSides([0.42, 0.28, 0.16], [0.45, 0.33, 0.20]));
    set!(B::FarmlandWet, TopSides([0.24, 0.16, 0.08], [0.35, 0.24, 0.14]));
    set!(B::WheatSeeds, Uniform([0.65, 0.60, 0.30]));
    set!(B::WheatCrop, Uniform([0.80, 0.75, 0.22]));
    set!(B::Wheat, Uniform([0.85, 0.75, 0.25]));
    set!(B::Bread, Uniform([0.75, 0.50, 0.20]));
    set!(B::Carrot, Uniform([0.95, 0.45, 0.05]));
    set!(B::CarrotCrop, Uniform([0.25, 0.65, 0.20]));
    set!(B::Potato, Uniform([0.75, 0.62, 0.35]));
    set!(B::PotatoCrop, Uniform([0.28, 0.60, 0.22]));
    set!(B::BakedPotato, Uniform([0.82, 0.65, 0.30]));
    set!(B::Sapling, Uniform([0.30, 0.62, 0.22]));
    set!(B::BoneMeal, Uniform([0.92, 0.92, 0.88]));
    set!(B::Bone, Uniform([0.88, 0.88, 0.84]));
    set!(B::Gunpowder, Uniform([0.28, 0.28, 0.28]));
    set!(B::RustStoneOre, Uniform([0.65, 0.25, 0.15]));
    set!(B::RustStone, Uniform([0.75, 0.22, 0.10]));
    set!(B::RustStoneWireLit, Uniform([1.0, 0.38, 0.14]));
    set!(B::RustStoneBlock, Uniform([0.78, 0.24, 0.12]));
    set!(B::RustStoneLamp, Uniform([0.38, 0.22, 0.14]));
    set!(B::RustStoneLampLit, Uniform([0.98, 0.78, 0.35]));
    set!(B::Lever, Uniform([0.55, 0.50, 0.45]));
    set!(B::LeverOn, Uniform([0.55, 0.50, 0.45]));
    set!(B::Button, Uniform([0.58, 0.58, 0.58]));
    set!(B::ButtonPressed, Uniform([0.58, 0.58, 0.58]));
    set!(B::PressurePlate, Uniform([0.60, 0.60, 0.60]));
    set!(B::PressurePlatePressed, Uniform([0.60, 0.60, 0.60]));
    set!(B::WhiteWool, Uniform([0.91, 0.92, 0.92]));
    set!(B::OrangeWool, Uniform([0.94, 0.46, 0.07]));
    set!(B::MagentaWool, Uniform([0.74, 0.27, 0.70]));
    set!(B::LightBlueWool, Uniform([0.23, 0.69, 0.85]));
    set!(B::YellowWool, Uniform([0.97, 0.78, 0.15]));
    set!(B::LimeWool, Uniform([0.44, 0.73, 0.10]));
    set!(B::PinkWool, Uniform([0.93, 0.55, 0.67]));
    set!(B::GrayWool, Uniform([0.24, 0.27, 0.28]));
    set!(B::LightGrayWool, Uniform([0.56, 0.56, 0.53]));
    set!(B::CyanWool, Uniform([0.08, 0.54, 0.57]));
    set!(B::PurpleWool, Uniform([0.47, 0.16, 0.67]));
    set!(B::BlueWool, Uniform([0.21, 0.22, 0.62]));
    set!(B::BrownWool, Uniform([0.45, 0.28, 0.16]));
    set!(B::GreenWool, Uniform([0.33, 0.43, 0.11]));
    set!(B::RedWool, Uniform([0.63, 0.15, 0.13]));
    set!(B::BlackWool, Uniform([0.08, 0.08, 0.10]));
    set!(B::SheepSpawnEgg, Uniform([0.88, 0.88, 0.86]));
    set!(B::Shears, Uniform([0.75, 0.75, 0.78]));
    set!(B::Kelp, Uniform([0.18, 0.44, 0.16]));
    set!(B::Seagrass, Uniform([0.22, 0.58, 0.22]));
    set!(B::RawCod, Uniform([0.72, 0.68, 0.58]));
    set!(B::CookedCod, Uniform([0.82, 0.70, 0.45]));
    set!(B::RawSalmon, Uniform([0.88, 0.42, 0.38]));
    set!(B::CookedSalmon, Uniform([0.92, 0.55, 0.32]));
    set!(B::Pufferfish, Uniform([0.88, 0.80, 0.25]));
    set!(B::TropicalFish, Uniform([0.96, 0.48, 0.15]));
    set!(B::DriedKelp, Uniform([0.14, 0.28, 0.12]));
    set!(B::DolphinSpawnEgg, Uniform([0.55, 0.68, 0.78]));
    set!(B::FishSpawnEgg, Uniform([0.78, 0.55, 0.35]));
    set!(B::FishingRod, Uniform([0.55, 0.38, 0.22]));
    set!(B::Emerald, Uniform([0.10, 0.82, 0.35]));
    set!(B::EmeraldOre, Uniform([0.35, 0.55, 0.35]));
    set!(B::EmeraldBlock, Uniform([0.08, 0.85, 0.30]));
    set!(B::VillagerSpawnEgg, Uniform([0.55, 0.38, 0.24]));
    set!(B::IronGolemSpawnEgg, Uniform([0.84, 0.84, 0.82]));
    set!(B::Clay, Uniform([0.62, 0.65, 0.70]));
    set!(B::ClayBall, Uniform([0.68, 0.71, 0.76]));
    set!(B::Brick, Uniform([0.70, 0.35, 0.25]));
    set!(B::Bricks, Uniform([0.65, 0.30, 0.22]));
    set!(B::BrickStairs, Uniform([0.65, 0.30, 0.22]));
    set!(B::BrickSlab, Uniform([0.65, 0.30, 0.22]));
    set!(B::Gravel, Uniform([0.52, 0.50, 0.50]));
    set!(B::Flint, Uniform([0.25, 0.25, 0.27]));
    set!(B::Cobblestone, Uniform([0.48, 0.48, 0.48]));
    set!(B::CobblestoneStairs, Uniform([0.48, 0.48, 0.48]));
    set!(B::CobblestoneSlab, Uniform([0.48, 0.48, 0.48]));
    set!(B::MossyCobblestone, Uniform([0.42, 0.50, 0.40]));
    set!(B::Sandstone, TopBottomSides([0.86, 0.82, 0.65], [0.82, 0.77, 0.60], [0.84, 0.80, 0.62]));
    set!(B::SandstoneStairs, TopBottomSides([0.86, 0.82, 0.65], [0.82, 0.77, 0.60], [0.84, 0.80, 0.62]));
    set!(B::SandstoneSlab, TopBottomSides([0.86, 0.82, 0.65], [0.82, 0.77, 0.60], [0.84, 0.80, 0.62]));
    set!(B::SmoothStone, Uniform([0.62, 0.62, 0.62]));
    set!(B::Obsidian, Uniform([0.12, 0.08, 0.18]));
    set!(B::Bookshelf, TopBottomCap([0.55, 0.42, 0.25], [0.45, 0.35, 0.25]));
    set!(B::Leather, Uniform([0.65, 0.42, 0.22]));
    set!(B::LeatherHelmet, Uniform([0.62, 0.38, 0.20]));
    set!(B::LeatherChestplate, Uniform([0.62, 0.38, 0.20]));
    set!(B::LeatherLeggings, Uniform([0.62, 0.38, 0.20]));
    set!(B::LeatherBoots, Uniform([0.62, 0.38, 0.20]));
    set!(B::IronHelmet, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronChestplate, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronLeggings, Uniform([0.85, 0.85, 0.88]));
    set!(B::IronBoots, Uniform([0.85, 0.85, 0.88]));
    set!(B::GoldHelmet, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldChestplate, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldLeggings, Uniform([0.95, 0.85, 0.25]));
    set!(B::GoldBoots, Uniform([0.95, 0.85, 0.25]));
    set!(B::DiamondHelmet, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondChestplate, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondLeggings, Uniform([0.25, 0.90, 0.95]));
    set!(B::DiamondBoots, Uniform([0.25, 0.90, 0.95]));
    set!(B::Bow, Uniform([0.55, 0.38, 0.18]));
    set!(B::Arrow, Uniform([0.75, 0.75, 0.72]));
    set!(B::Shield, Uniform([0.60, 0.45, 0.25]));
    set!(B::StringItem, Uniform([0.92, 0.92, 0.95]));
    set!(B::SpiderEye, Uniform([0.65, 0.12, 0.15]));
    set!(B::SpiderSpawnEgg, Uniform([0.18, 0.14, 0.14]));

    t
}

const FACE_COLORS: [FaceColorPattern; VANILLA_BLOCK_COUNT] = build_face_colors();

impl BlockId {
    /// Base RGB color for each face; grass has a green top and dirt sides.
    pub fn face_color(self, face: Face) -> [f32; 3] {
        match self {
            BlockId::Custom(id) => crate::engine::modding::get_custom_face_color(id, face),
            _ => FACE_COLORS[self.to_u8() as usize].eval(face),
        }
    }
}
