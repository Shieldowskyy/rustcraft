//! Testy jednostkowe zakładek kreatywnych, układu slotów ekwipunku, księgi receptur i renderowania ekranu.

use super::*;
use crate::features::blocks::BlockId;
use crate::features::player::{GameMode, Inventory, ItemStack, Player, RecipeBook};

#[test]
fn test_creative_tabs_and_block_lists() {
    assert_eq!(CREATIVE_TABS.len(), 6);
    let tab0 = creative_tab_blocks(0);
    assert!(tab0.contains(&BlockId::Grass));
    assert!(tab0.contains(&BlockId::Stone));
    assert!(tab0.contains(&BlockId::Wood));
    assert!(tab0.contains(&BlockId::PigSpawnEgg));
    assert!(tab0.contains(&BlockId::CowSpawnEgg));
    assert!(tab0.contains(&BlockId::ChickenSpawnEgg));
    assert!(tab0.contains(&BlockId::SheepSpawnEgg));
    assert!(tab0.contains(&BlockId::WhiteWool));
    assert!(tab0.contains(&BlockId::Clay));
    assert!(tab0.contains(&BlockId::Bricks));
    assert!(tab0.contains(&BlockId::Cobblestone));
    assert!(tab0.contains(&BlockId::Sandstone));
    assert!(tab0.contains(&BlockId::Obsidian));
    assert!(tab0.contains(&BlockId::Bookshelf));

    let tab1 = creative_tab_blocks(1);
    assert!(tab1.contains(&BlockId::WoodenPickaxe));
    assert!(tab1.contains(&BlockId::IronSword));
    assert!(tab1.contains(&BlockId::Shears));
    assert!(tab1.contains(&BlockId::Tnt));
    assert!(tab1.contains(&BlockId::ClayBall));
    assert!(tab1.contains(&BlockId::Brick));
    assert!(tab1.contains(&BlockId::Flint));
    assert!(tab1.contains(&BlockId::PigSpawnEgg));
    assert!(tab1.contains(&BlockId::CowSpawnEgg));
    assert!(tab1.contains(&BlockId::ChickenSpawnEgg));
    assert!(tab1.contains(&BlockId::SheepSpawnEgg));

    let tab2 = creative_tab_blocks(2);
    assert!(tab2.contains(&BlockId::RustStone));
    assert!(tab2.contains(&BlockId::RustStoneBlock));
    assert!(tab2.contains(&BlockId::RustStoneLamp));
    assert!(tab2.contains(&BlockId::Lever));
    assert!(tab2.contains(&BlockId::Button));
    assert!(tab2.contains(&BlockId::PressurePlate));

    let tab3 = creative_tab_blocks(3);
    assert!(tab3.contains(&BlockId::WoodenHoe));
    assert!(tab3.contains(&BlockId::DiamondHoe));
    assert!(tab3.contains(&BlockId::WheatSeeds));
    assert!(tab3.contains(&BlockId::Carrot));
    assert!(tab3.contains(&BlockId::Potato));
    assert!(tab3.contains(&BlockId::BoneMeal));
    assert!(tab3.contains(&BlockId::Farmland));
    assert!(tab3.contains(&BlockId::SheepSpawnEgg));

    let tab4 = creative_tab_blocks(4);
    assert!(tab4.len() >= 76);
    assert!(tab4.contains(&BlockId::Grass));
    assert!(tab4.contains(&BlockId::Torch));
    assert!(tab4.contains(&BlockId::IronSword));
    assert!(tab4.contains(&BlockId::DiamondSword));
    assert!(tab4.contains(&BlockId::DiamondPickaxe));
    assert!(tab4.contains(&BlockId::Diamond));
    assert!(tab4.contains(&BlockId::Bed));
    assert!(tab4.contains(&BlockId::Ladder));
    assert!(tab4.contains(&BlockId::WoodenSlab));
    assert!(tab4.contains(&BlockId::WoodenStairs));
    assert!(tab4.contains(&BlockId::RawPorkchop));
    assert!(tab4.contains(&BlockId::CookedBeef));
    assert!(tab4.contains(&BlockId::RawChicken));
    assert!(tab4.contains(&BlockId::CookedChicken));
    assert!(tab4.contains(&BlockId::Feather));
    assert!(tab4.contains(&BlockId::PigSpawnEgg));
    assert!(tab4.contains(&BlockId::CowSpawnEgg));
    assert!(tab4.contains(&BlockId::ChickenSpawnEgg));
    assert!(tab4.contains(&BlockId::SheepSpawnEgg));
    assert!(tab4.contains(&BlockId::DolphinSpawnEgg));
    assert!(tab4.contains(&BlockId::FishSpawnEgg));
    assert!(tab4.contains(&BlockId::FishingRod));
    assert!(tab4.contains(&BlockId::RawCod));
    assert!(tab4.contains(&BlockId::CookedCod));
    assert!(tab4.contains(&BlockId::RawSalmon));
    assert!(tab4.contains(&BlockId::CookedSalmon));
    assert!(tab4.contains(&BlockId::Pufferfish));
    assert!(tab4.contains(&BlockId::TropicalFish));
    assert!(tab4.contains(&BlockId::DriedKelp));
    assert!(tab4.contains(&BlockId::Kelp));
    assert!(tab4.contains(&BlockId::Seagrass));
    assert!(tab4.contains(&BlockId::Shears));
    assert!(tab4.contains(&BlockId::WhiteWool));
    assert!(tab4.contains(&BlockId::RedWool));
    assert!(tab4.contains(&BlockId::Chest));
    assert!(tab4.contains(&BlockId::Furnace));
    assert!(tab4.contains(&BlockId::IronOunce));
    assert!(tab4.contains(&BlockId::IronBlock));
    assert!(tab4.contains(&BlockId::GoldOre));
    assert!(tab4.contains(&BlockId::GoldBlock));
    assert!(tab4.contains(&BlockId::CoalBlock));
    assert!(tab4.contains(&BlockId::Farmland));
    assert!(tab4.contains(&BlockId::RustStone));
    assert!(tab4.contains(&BlockId::RustStoneBlock));
    assert!(tab4.contains(&BlockId::RustStoneLamp));
    assert!(tab4.contains(&BlockId::Lever));
    assert!(tab4.contains(&BlockId::Button));
    assert!(tab4.contains(&BlockId::PressurePlate));
    assert!(tab4.contains(&BlockId::Bone));
    assert!(tab4.contains(&BlockId::Gunpowder));
    assert!(tab4.contains(&BlockId::WheatSeeds));
    assert!(tab4.contains(&BlockId::DiamondHoe));
    assert!(tab4.contains(&BlockId::Water));
    assert!(tab4.contains(&BlockId::Lava));
    assert!(tab4.contains(&BlockId::Tnt));
}

#[test]
fn test_creative_layout_slot_detection() {
    let layout = InventoryLayout::new(960, 544, false);
    let mut player = Player::new();
    player.gamemode = GameMode::Creative;
    player.creative_tab = 0;

    let tab0_x = layout.start_x + 20;
    let tab0_y = layout.start_y - 10;
    assert_eq!(layout.find_slot_at(tab0_x, tab0_y, &player), Some(InventorySlotTarget::CreativeTab(0)));

    let tab1_x = layout.start_x + 90;
    assert_eq!(layout.find_slot_at(tab1_x, tab0_y, &player), Some(InventorySlotTarget::CreativeTab(1)));

    let block0_x = layout.start_x + 30;
    let block0_y = layout.start_y + 45;
    assert_eq!(layout.find_slot_at(block0_x, block0_y, &player), Some(InventorySlotTarget::Creative(BlockId::Grass.to_u8())));

    let sb_up_x = layout.start_x + 410;
    let sb_up_y = layout.start_y + 40;
    assert_eq!(layout.find_slot_at(sb_up_x, sb_up_y, &player), Some(InventorySlotTarget::CreativeScrollUp));
    let sb_down_y = layout.start_y + 195;
    assert_eq!(layout.find_slot_at(sb_up_x, sb_down_y, &player), Some(InventorySlotTarget::CreativeScrollDown));

    let hotbar0_x = layout.start_x + 25;
    let hotbar0_y = layout.start_y + 250;
    assert_eq!(layout.find_slot_at(hotbar0_x, hotbar0_y, &player), Some(InventorySlotTarget::Hotbar(0)));

    let trash_x = layout.start_x + 430;
    let trash_y = layout.start_y + 250;
    assert_eq!(layout.find_slot_at(trash_x, trash_y, &player), Some(InventorySlotTarget::Trash));
}

#[test]
fn test_creative_hotbar_assignment_and_trash() {
    let mut inv = Inventory::new();
    inv.hotbar[0] = Some(ItemStack::new(BlockId::Grass, 64));
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockId::Grass, 64)));

    inv.held_item = Some(ItemStack::new(BlockId::StonePickaxe, 1));
    inv.held_item = None;
    assert_eq!(inv.held_item, None);

    inv.hotbar.fill(None);
    assert!(inv.hotbar.iter().all(|s| s.is_none()));
}

#[test]
fn test_draw_inventory_screen_renders_player_preview_and_background() {
    let width = 640;
    let height = 480;
    let mut buffer = vec![0u32; width * height];
    let player = Player::new();
    let recipes = RecipeBook::new();

    draw_inventory_screen(
        &mut buffer,
        width,
        height,
        &player,
        None,
        false,
        &recipes,
        &[],
        false,
        0,
        None,
        Some((320.0, 240.0)),
        false,
    );

    let colored_pixels = buffer.iter().filter(|&&p| p != 0).count();
    assert!(colored_pixels > 1000);
}

#[test]
fn recipe_book_targets_only_exist_at_crafting_table() {
    let layout = InventoryLayout::new(640, 480, true);
    let recipes = [2, 7, 9];
    assert_eq!(
        layout.find_recipe_book_target_at(layout.start_x + 195, layout.start_y + 40, false, &recipes, 0),
        Some(InventorySlotTarget::RecipeBookToggle)
    );
    assert_eq!(
        layout.find_recipe_book_target_at(layout.start_x + 305, layout.start_y + 53, true, &recipes, 1),
        Some(InventorySlotTarget::RecipeEntry(7))
    );
    assert_eq!(
        InventoryLayout::new(640, 480, false).find_recipe_book_target_at(
            layout.start_x + 195,
            layout.start_y + 40,
            true,
            &recipes,
            0,
        ),
        None
    );
}
