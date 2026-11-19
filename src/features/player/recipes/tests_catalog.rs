//! Unit tests for diamond, architectural, ruststone, wool, building block, and combat recipes.

use crate::features::blocks::BlockId;

use super::super::inventory::ItemStack;
use super::RecipeBook;

#[test]
fn test_diamond_recipes_and_architecture() {
    let book = RecipeBook::new();
    let dia_block_grid = [Some(ItemStack::new(BlockId::Diamond, 1)); 9];
    assert_eq!(book.match_grid(&dia_block_grid, 3, 3), Some(ItemStack::new(BlockId::DiamondBlock, 1)));

    let decraft_dia = [Some(ItemStack::new(BlockId::DiamondBlock, 1)), None, None, None];
    assert_eq!(book.match_grid(&decraft_dia, 2, 2), Some(ItemStack::new(BlockId::Diamond, 9)));

    let dia_pick_grid = [
        Some(ItemStack::new(BlockId::Diamond, 1)),
        Some(ItemStack::new(BlockId::Diamond, 1)),
        Some(ItemStack::new(BlockId::Diamond, 1)),
        None, Some(ItemStack::new(BlockId::Stick, 1)), None,
        None, Some(ItemStack::new(BlockId::Stick, 1)), None,
    ];
    assert_eq!(book.match_grid(&dia_pick_grid, 3, 3), Some(ItemStack::new(BlockId::DiamondPickaxe, 1)));

    let bed_grid = [
        None, None, None,
        Some(ItemStack::new(BlockId::Feather, 1)),
        Some(ItemStack::new(BlockId::Feather, 1)),
        Some(ItemStack::new(BlockId::Feather, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
    ];
    assert_eq!(book.match_grid(&bed_grid, 3, 3), Some(ItemStack::new(BlockId::Bed, 1)));

    let slab_grid = [
        None, None, None,
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        None, None, None,
    ];
    assert_eq!(book.match_grid(&slab_grid, 3, 3), Some(ItemStack::new(BlockId::WoodenSlab, 6)));

    let ladder_grid = [
        Some(ItemStack::new(BlockId::Stick, 1)), None, Some(ItemStack::new(BlockId::Stick, 1)),
        Some(ItemStack::new(BlockId::Stick, 1)), Some(ItemStack::new(BlockId::Stick, 1)), Some(ItemStack::new(BlockId::Stick, 1)),
        Some(ItemStack::new(BlockId::Stick, 1)), None, Some(ItemStack::new(BlockId::Stick, 1)),
    ];
    assert_eq!(book.match_grid(&ladder_grid, 3, 3), Some(ItemStack::new(BlockId::Ladder, 3)));
}

#[test]
fn test_recipe_macro_and_custom_addition() {
    let mut book = RecipeBook::new();
    let initial_count = book.recipes().len();

    let custom_1x1 = recipe!(Dirt => Grass * 2);
    assert_eq!(custom_1x1.width, 1);
    assert_eq!(custom_1x1.height, 1);
    assert_eq!(custom_1x1.pattern, vec![Some(BlockId::Dirt)]);
    assert_eq!(custom_1x1.result, ItemStack::new(BlockId::Grass, 2));

    let custom_2x2 = recipe!(2, 2, [Dirt, _, _, Dirt] => Sand * 4);
    assert_eq!(custom_2x2.width, 2);
    assert_eq!(custom_2x2.height, 2);
    assert_eq!(custom_2x2.pattern, vec![Some(BlockId::Dirt), None, None, Some(BlockId::Dirt)]);
    assert_eq!(custom_2x2.result, ItemStack::new(BlockId::Sand, 4));

    book.add(custom_2x2);
    assert_eq!(book.recipes().len(), initial_count + 1);

    let grid = [Some(ItemStack::new(BlockId::Dirt, 1)), None, None, Some(ItemStack::new(BlockId::Dirt, 1))];
    assert_eq!(book.match_grid(&grid, 2, 2), Some(ItemStack::new(BlockId::Sand, 4)));
}

#[test]
fn test_ruststone_recipes() {
    let book = RecipeBook::new();
    let block_grid = [Some(ItemStack::new(BlockId::RustStone, 1)); 9];
    assert_eq!(book.match_grid(&block_grid, 3, 3), Some(ItemStack::new(BlockId::RustStoneBlock, 1)));

    let decraft_grid = [Some(ItemStack::new(BlockId::RustStoneBlock, 1)), None, None, None];
    assert_eq!(book.match_grid(&decraft_grid, 2, 2), Some(ItemStack::new(BlockId::RustStone, 9)));

    let lever_grid = [Some(ItemStack::new(BlockId::Stick, 1)), Some(ItemStack::new(BlockId::Stone, 1))];
    assert_eq!(book.match_grid(&lever_grid, 1, 2), Some(ItemStack::new(BlockId::Lever, 1)));

    let button_grid = [Some(ItemStack::new(BlockId::Stone, 1))];
    assert_eq!(book.match_grid(&button_grid, 1, 1), Some(ItemStack::new(BlockId::Button, 1)));

    let plate_grid = [Some(ItemStack::new(BlockId::Stone, 1)); 2];
    assert_eq!(book.match_grid(&plate_grid, 2, 1), Some(ItemStack::new(BlockId::PressurePlate, 1)));
}

#[test]
fn test_shears_and_wool_recipes() {
    let book = RecipeBook::new();
    let shears_grid = [None, Some(ItemStack::new(BlockId::IronIngot, 1)), Some(ItemStack::new(BlockId::IronIngot, 1)), None];
    assert_eq!(book.match_grid(&shears_grid, 2, 2), Some(ItemStack::new(BlockId::Shears, 1)));

    let bed_grid = [
        Some(ItemStack::new(BlockId::RedWool, 1)),
        Some(ItemStack::new(BlockId::RedWool, 1)),
        Some(ItemStack::new(BlockId::RedWool, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
    ];
    assert_eq!(book.match_grid(&bed_grid, 3, 2), Some(ItemStack::new(BlockId::Bed, 1)));

    let wool_grid = [Some(ItemStack::new(BlockId::Feather, 1)); 4];
    assert_eq!(book.match_grid(&wool_grid, 2, 2), Some(ItemStack::new(BlockId::WhiteWool, 1)));

    let dye_grid = [Some(ItemStack::new(BlockId::Apple, 1)), Some(ItemStack::new(BlockId::WhiteWool, 1))];
    assert_eq!(book.match_grid(&dye_grid, 2, 1), Some(ItemStack::new(BlockId::RedWool, 1)));
}

#[test]
fn test_new_building_blocks_recipes() {
    let book = RecipeBook::new();
    assert_eq!(book.match_grid(&[Some(ItemStack::new(BlockId::ClayBall, 1)); 4], 2, 2), Some(ItemStack::new(BlockId::Clay, 1)));
    assert_eq!(book.match_grid(&[Some(ItemStack::new(BlockId::Brick, 1)); 4], 2, 2), Some(ItemStack::new(BlockId::Bricks, 1)));
    assert_eq!(book.match_grid(&[Some(ItemStack::new(BlockId::Sand, 1)); 4], 2, 2), Some(ItemStack::new(BlockId::Sandstone, 1)));

    let furn_grid = [
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        None,
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
    ];
    assert_eq!(book.match_grid(&furn_grid, 3, 3), Some(ItemStack::new(BlockId::Furnace, 1)));

    let pick_grid = [
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        Some(ItemStack::new(BlockId::Cobblestone, 1)),
        None, Some(ItemStack::new(BlockId::Stick, 1)), None,
        None, Some(ItemStack::new(BlockId::Stick, 1)), None,
    ];
    assert_eq!(book.match_grid(&pick_grid, 3, 3), Some(ItemStack::new(BlockId::StonePickaxe, 1)));
}

#[test]
fn test_combat_and_armor_recipes() {
    let book = RecipeBook::new();
    let arrow_grid = [
        Some(ItemStack::new(BlockId::Flint, 1)),
        Some(ItemStack::new(BlockId::Stick, 1)),
        Some(ItemStack::new(BlockId::Feather, 1)),
    ];
    assert_eq!(book.match_grid(&arrow_grid, 1, 3), Some(ItemStack::new(BlockId::Arrow, 4)));

    let shield_grid = [
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::IronIngot, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        Some(ItemStack::new(BlockId::Planks, 1)),
        None, Some(ItemStack::new(BlockId::Planks, 1)), None,
    ];
    assert_eq!(book.match_grid(&shield_grid, 3, 3), Some(ItemStack::new(BlockId::Shield, 1)));

    let helmet_grid = [
        Some(ItemStack::new(BlockId::Diamond, 1)),
        Some(ItemStack::new(BlockId::Diamond, 1)),
        Some(ItemStack::new(BlockId::Diamond, 1)),
        Some(ItemStack::new(BlockId::Diamond, 1)),
        None,
        Some(ItemStack::new(BlockId::Diamond, 1)),
    ];
    assert_eq!(book.match_grid(&helmet_grid, 3, 2), Some(ItemStack::new(BlockId::DiamondHelmet, 1)));
}
