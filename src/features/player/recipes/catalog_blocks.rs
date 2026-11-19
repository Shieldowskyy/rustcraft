//! Default recipes for wool, beds, architectural slabs/stairs, doors, TNT, and RustStone circuits.

use crate::features::blocks::BlockId;

use super::types::{Recipe, RecipeBook};

pub(super) fn register_building_and_mechanic_recipes(book: &mut RecipeBook) {
    // Shears
    book.add(recipe!(2, 2, [_, IronIngot, IronIngot, _] => Shears * 1));
    book.add(recipe!(2, 2, [IronIngot, _, _, IronIngot] => Shears * 1));

    // Fishing rod
    book.add(recipe!(3, 3, [_, _, Stick, _, Stick, StringItem, Stick, _, StringItem] => FishingRod * 1));
    book.add(recipe!(2, 2, [Stick, StringItem, Stick, StringItem] => FishingRod * 1));
    book.add(recipe!(3, 3, [_, _, Stick, _, Stick, Feather, Stick, _, Feather] => FishingRod * 1));
    book.add(recipe!(3, 3, [_, _, Stick, _, Stick, WhiteWool, Stick, _, WhiteWool] => FishingRod * 1));
    book.add(recipe!(2, 2, [Stick, Feather, Stick, Feather] => FishingRod * 1));
    book.add(recipe!(2, 2, [Stick, WhiteWool, Stick, WhiteWool] => FishingRod * 1));

    // Wool & beds
    book.add(recipe!(2, 2, [StringItem, StringItem, StringItem, StringItem] => WhiteWool * 1));
    for &wool in &BlockId::WOOL_BLOCKS {
        book.add(Recipe::new(
            3, 2,
            vec![
                Some(wool), Some(wool), Some(wool),
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            BlockId::Bed, 1,
        ));
    }
    book.add(recipe!(3, 2, [Feather, Feather, Feather, Planks, Planks, Planks] => Bed * 1));
    book.add(recipe!(3, 2, [Leaves, Leaves, Leaves, Planks, Planks, Planks] => Bed * 1));
    book.add(recipe!(2, 2, [Feather, Feather, Feather, Feather] => WhiteWool * 1));

    // Wool dyeing
    book.add(recipe!(1, 2, [Apple, WhiteWool] => RedWool * 1));
    book.add(recipe!(2, 1, [Apple, WhiteWool] => RedWool * 1));
    book.add(recipe!(1, 2, [Leaves, WhiteWool] => GreenWool * 1));
    book.add(recipe!(2, 1, [Leaves, WhiteWool] => GreenWool * 1));
    book.add(recipe!(1, 2, [Sapling, WhiteWool] => LimeWool * 1));
    book.add(recipe!(2, 1, [Sapling, WhiteWool] => LimeWool * 1));
    book.add(recipe!(1, 2, [CoalOre, WhiteWool] => BlackWool * 1));
    book.add(recipe!(2, 1, [CoalOre, WhiteWool] => BlackWool * 1));
    book.add(recipe!(1, 2, [Gunpowder, WhiteWool] => GrayWool * 1));
    book.add(recipe!(2, 1, [Gunpowder, WhiteWool] => GrayWool * 1));
    book.add(recipe!(1, 2, [BoneMeal, GrayWool] => LightGrayWool * 1));
    book.add(recipe!(2, 1, [BoneMeal, GrayWool] => LightGrayWool * 1));
    book.add(recipe!(1, 2, [Carrot, WhiteWool] => OrangeWool * 1));
    book.add(recipe!(2, 1, [Carrot, WhiteWool] => OrangeWool * 1));
    book.add(recipe!(1, 2, [Wheat, WhiteWool] => YellowWool * 1));
    book.add(recipe!(2, 1, [Wheat, WhiteWool] => YellowWool * 1));
    book.add(recipe!(1, 2, [BoneMeal, RedWool] => PinkWool * 1));
    book.add(recipe!(2, 1, [BoneMeal, RedWool] => PinkWool * 1));
    book.add(recipe!(1, 2, [Diamond, WhiteWool] => LightBlueWool * 1));
    book.add(recipe!(2, 1, [Diamond, WhiteWool] => LightBlueWool * 1));
    book.add(recipe!(1, 2, [Water, BlueWool] => CyanWool * 1));
    book.add(recipe!(2, 1, [Water, BlueWool] => CyanWool * 1));
    book.add(recipe!(1, 2, [RedWool, BlueWool] => PurpleWool * 1));
    book.add(recipe!(2, 1, [RedWool, BlueWool] => PurpleWool * 1));
    book.add(recipe!(1, 2, [Dirt, WhiteWool] => BrownWool * 1));
    book.add(recipe!(2, 1, [Dirt, WhiteWool] => BrownWool * 1));

    // Slabs & stairs
    book.add(recipe!(3, 1, [Planks, Planks, Planks] => WoodenSlab * 6));
    book.add(recipe!(3, 1, [Stone, Stone, Stone] => StoneSlab * 6));
    book.add(recipe!(3, 1, [Bricks, Bricks, Bricks] => BrickSlab * 6));
    book.add(recipe!(3, 1, [Cobblestone, Cobblestone, Cobblestone] => CobblestoneSlab * 6));
    book.add(recipe!(3, 1, [Sandstone, Sandstone, Sandstone] => SandstoneSlab * 6));

    book.add(recipe!(3, 3, [Planks, _, _, Planks, Planks, _, Planks, Planks, Planks] => WoodenStairs * 4));
    book.add(recipe!(3, 3, [_, _, Planks, _, Planks, Planks, Planks, Planks, Planks] => WoodenStairs * 4));
    book.add(recipe!(3, 3, [Stone, _, _, Stone, Stone, _, Stone, Stone, Stone] => StoneStairs * 4));
    book.add(recipe!(3, 3, [_, _, Stone, _, Stone, Stone, Stone, Stone, Stone] => StoneStairs * 4));
    book.add(recipe!(3, 3, [Bricks, _, _, Bricks, Bricks, _, Bricks, Bricks, Bricks] => BrickStairs * 4));
    book.add(recipe!(3, 3, [_, _, Bricks, _, Bricks, Bricks, Bricks, Bricks, Bricks] => BrickStairs * 4));
    book.add(recipe!(3, 3, [Cobblestone, _, _, Cobblestone, Cobblestone, _, Cobblestone, Cobblestone, Cobblestone] => CobblestoneStairs * 4));
    book.add(recipe!(3, 3, [_, _, Cobblestone, _, Cobblestone, Cobblestone, Cobblestone, Cobblestone, Cobblestone] => CobblestoneStairs * 4));
    book.add(recipe!(3, 3, [Sandstone, _, _, Sandstone, Sandstone, _, Sandstone, Sandstone, Sandstone] => SandstoneStairs * 4));
    book.add(recipe!(3, 3, [_, _, Sandstone, _, Sandstone, Sandstone, Sandstone, Sandstone, Sandstone] => SandstoneStairs * 4));

    // Clay, bricks, sandstone, bookshelf, mossy stone, ladder, door, TNT
    book.add(recipe!(2, 2, [ClayBall, ClayBall, ClayBall, ClayBall] => Clay * 1));
    book.add(recipe!(Clay => ClayBall * 4));
    book.add(recipe!(2, 2, [Brick, Brick, Brick, Brick] => Bricks * 1));
    book.add(recipe!(2, 2, [Sand, Sand, Sand, Sand] => Sandstone * 1));
    book.add(recipe!(3, 3, [Planks, Planks, Planks, Wheat, Wheat, Wheat, Planks, Planks, Planks] => Bookshelf * 1));
    book.add(recipe!(1, 2, [Cobblestone, Leaves] => MossyCobblestone * 1));
    book.add(recipe!(2, 1, [Cobblestone, Leaves] => MossyCobblestone * 1));
    book.add(recipe!(3, 3, [Stick, _, Stick, Stick, Stick, Stick, Stick, _, Stick] => Ladder * 3));
    book.add(recipe!(2, 3, [Planks, Planks, Planks, Planks, Planks, Planks] => Door * 1));
    book.add(recipe!(2, 2, [Sand, Dirt, Dirt, Sand] => Tnt * 1));
    book.add(recipe!(2, 2, [Gunpowder, Sand, Sand, Gunpowder] => Tnt * 1));

    // RustStone circuits & mechanics
    book.add(recipe!(3, 3, [RustStone, RustStone, RustStone, RustStone, RustStone, RustStone, RustStone, RustStone, RustStone] => RustStoneBlock * 1));
    book.add(recipe!(RustStoneBlock => RustStone * 9));
    book.add(recipe!(3, 3, [_, RustStone, _, RustStone, Glass, RustStone, _, RustStone, _] => RustStoneLamp * 1));
    book.add(recipe!(3, 3, [Stone, RustStone, Stone, RustStone, Torch, RustStone, Stone, RustStone, Stone] => RustStoneLamp * 1));
    book.add(recipe!(3, 3, [IronBlock, IronBlock, IronBlock, _, IronBlock, _, _, _, _] => IronGolemSpawnEgg * 1));
    book.add(recipe!(3, 3, [_, _, _, IronBlock, IronBlock, IronBlock, _, IronBlock, _] => IronGolemSpawnEgg * 1));
    book.add(recipe!(1, 2, [Stick, Stone] => Lever * 1));
    book.add(recipe!(1, 2, [Stick, Cobblestone] => Lever * 1));
    book.add(recipe!(1, 2, [Stick, Planks] => Lever * 1));
    book.add(recipe!(Stone => Button * 1));
    book.add(recipe!(Cobblestone => Button * 1));
    book.add(recipe!(Planks => Button * 1));
    book.add(recipe!(2, 1, [Stone, Stone] => PressurePlate * 1));
    book.add(recipe!(2, 1, [Cobblestone, Cobblestone] => PressurePlate * 1));
    book.add(recipe!(2, 1, [Planks, Planks] => PressurePlate * 1));
}
