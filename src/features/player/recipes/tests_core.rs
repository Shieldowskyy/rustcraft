//! Unit tests for recipe book unlocking, 2x2/3x3 crafting grid matching, and ingot/block conversions.

use crate::features::blocks::BlockId;

use super::super::achievements::AchievementProgress;
use super::super::inventory::ItemStack;
use super::RecipeBook;

#[test]
fn recipes_unlock_after_collecting_each_ingredient_type() {
    let book = RecipeBook::new();
    let mut progress = AchievementProgress::default();
    assert!(!book.unlocked_recipe_indices(&progress).contains(&0));

    progress.record_block_discovered(BlockId::Wood);
    assert!(!book.unlocked_recipe_indices(&progress).contains(&0));

    progress.record_item_collected(BlockId::Wood);
    let unlocked = book.unlocked_recipe_indices(&progress);
    assert!(unlocked.contains(&0));
    assert!(!unlocked.contains(&1));

    progress.record_item_collected(BlockId::Planks);
    assert!(book.unlocked_recipe_indices(&progress).contains(&1));
}

#[test]
fn test_planks_crafting_2x2() {
    let book = RecipeBook::new();
    let mut grid = [None; 4];
    grid[0] = Some(ItemStack::new(BlockId::Wood, 1));
    assert_eq!(book.match_grid(&grid, 2, 2), Some(ItemStack::new(BlockId::Planks, 4)));
}

#[test]
fn test_crafting_table_2x2() {
    let book = RecipeBook::new();
    let grid = [Some(ItemStack::new(BlockId::Planks, 1)); 4];
    assert_eq!(book.match_grid(&grid, 2, 2), Some(ItemStack::new(BlockId::CraftingTable, 1)));
}

#[test]
fn test_pickaxe_3x3() {
    let book = RecipeBook::new();
    let grid = [
        Some(ItemStack::new(BlockId::Stone, 1)),
        Some(ItemStack::new(BlockId::Stone, 1)),
        Some(ItemStack::new(BlockId::Stone, 1)),
        None,
        Some(ItemStack::new(BlockId::Stick, 1)),
        None,
        None,
        Some(ItemStack::new(BlockId::Stick, 1)),
        None,
    ];
    assert_eq!(book.match_grid(&grid, 3, 3), Some(ItemStack::new(BlockId::StonePickaxe, 1)));
}

#[test]
fn test_stick_recipe_in_2x2_and_consumption() {
    let book = RecipeBook::new();
    let mut grid = [
        Some(ItemStack::new(BlockId::Planks, 2)),
        None,
        Some(ItemStack::new(BlockId::Planks, 2)),
        None,
    ];
    assert_eq!(book.match_grid(&grid, 2, 2), Some(ItemStack::new(BlockId::Stick, 4)));

    RecipeBook::consume_grid(&mut grid);
    assert_eq!(grid[0], Some(ItemStack::new(BlockId::Planks, 1)));
    assert_eq!(grid[2], Some(ItemStack::new(BlockId::Planks, 1)));

    RecipeBook::consume_grid(&mut grid);
    assert_eq!(grid[0], None);
    assert_eq!(grid[2], None);
}

#[test]
fn test_sword_and_shovel_3x3() {
    let book = RecipeBook::new();
    let sword_grid = [
        None, Some(ItemStack::new(BlockId::IronIngot, 1)), None,
        None, Some(ItemStack::new(BlockId::IronIngot, 1)), None,
        None, Some(ItemStack::new(BlockId::Stick, 1)), None,
    ];
    assert_eq!(book.match_grid(&sword_grid, 3, 3), Some(ItemStack::new(BlockId::IronSword, 1)));

    let shovel_grid = [
        Some(ItemStack::new(BlockId::Planks, 1)), None, None,
        Some(ItemStack::new(BlockId::Stick, 1)), None, None,
        Some(ItemStack::new(BlockId::Stick, 1)), None, None,
    ];
    assert_eq!(book.match_grid(&shovel_grid, 3, 3), Some(ItemStack::new(BlockId::WoodenShovel, 1)));
}

#[test]
fn test_torch_crafting() {
    let book = RecipeBook::new();
    let mut grid = [None; 4];
    grid[0] = Some(ItemStack::new(BlockId::CoalOre, 1));
    grid[2] = Some(ItemStack::new(BlockId::Stick, 1));
    assert_eq!(book.match_grid(&grid, 2, 2), Some(ItemStack::new(BlockId::Torch, 4)));
}

#[test]
fn test_ounces_to_ingots_and_reverse() {
    let book = RecipeBook::new();
    let iron_ring = [
        Some(ItemStack::new(BlockId::IronOunce, 1)), Some(ItemStack::new(BlockId::IronOunce, 1)), Some(ItemStack::new(BlockId::IronOunce, 1)),
        Some(ItemStack::new(BlockId::IronOunce, 1)), None, Some(ItemStack::new(BlockId::IronOunce, 1)),
        Some(ItemStack::new(BlockId::IronOunce, 1)), Some(ItemStack::new(BlockId::IronOunce, 1)), Some(ItemStack::new(BlockId::IronOunce, 1)),
    ];
    assert_eq!(book.match_grid(&iron_ring, 3, 3), Some(ItemStack::new(BlockId::IronIngot, 1)));

    let ingot_grid = [Some(ItemStack::new(BlockId::IronIngot, 1)), None, None, None];
    assert_eq!(book.match_grid(&ingot_grid, 2, 2), Some(ItemStack::new(BlockId::IronOunce, 8)));

    let gold_ring = [
        Some(ItemStack::new(BlockId::GoldOunce, 1)), Some(ItemStack::new(BlockId::GoldOunce, 1)), Some(ItemStack::new(BlockId::GoldOunce, 1)),
        Some(ItemStack::new(BlockId::GoldOunce, 1)), None, Some(ItemStack::new(BlockId::GoldOunce, 1)),
        Some(ItemStack::new(BlockId::GoldOunce, 1)), Some(ItemStack::new(BlockId::GoldOunce, 1)), Some(ItemStack::new(BlockId::GoldOunce, 1)),
    ];
    assert_eq!(book.match_grid(&gold_ring, 3, 3), Some(ItemStack::new(BlockId::GoldIngot, 1)));

    let gold_ingot_grid = [Some(ItemStack::new(BlockId::GoldIngot, 1)), None, None, None];
    assert_eq!(book.match_grid(&gold_ingot_grid, 2, 2), Some(ItemStack::new(BlockId::GoldOunce, 8)));
}

#[test]
fn test_ingots_to_blocks_and_reverse() {
    let book = RecipeBook::new();
    let block_grid = [Some(ItemStack::new(BlockId::IronIngot, 1)); 9];
    assert_eq!(book.match_grid(&block_grid, 3, 3), Some(ItemStack::new(BlockId::IronBlock, 1)));

    let decraft_block = [Some(ItemStack::new(BlockId::IronBlock, 1)), None, None, None];
    assert_eq!(book.match_grid(&decraft_block, 2, 2), Some(ItemStack::new(BlockId::IronIngot, 9)));
}

#[test]
fn test_coal_block_and_reverse() {
    let book = RecipeBook::new();
    let coal_grid = [Some(ItemStack::new(BlockId::CoalOre, 1)); 9];
    assert_eq!(book.match_grid(&coal_grid, 3, 3), Some(ItemStack::new(BlockId::CoalBlock, 1)));

    let decraft_coal = [Some(ItemStack::new(BlockId::CoalBlock, 1)), None, None, None];
    assert_eq!(book.match_grid(&decraft_coal, 2, 2), Some(ItemStack::new(BlockId::CoalOre, 9)));
}
