//! Unit tests for `Inventory` stacking, slot click/drag semantics, crafting return, and durability.

use crate::features::blocks::BlockId;

use super::{Inventory, ItemStack};

#[test]
fn test_inventory_add_and_stack() {
    let mut inv = Inventory::new();
    assert_eq!(inv.add_item(BlockId::Dirt, 30), 0);
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockId::Dirt, 30)));

    assert_eq!(inv.add_item(BlockId::Dirt, 40), 0);
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockId::Dirt, 64)));
    assert_eq!(inv.hotbar[1], Some(ItemStack::new(BlockId::Dirt, 6)));
}

#[test]
fn test_slot_left_click_swap_and_stack() {
    let mut slot = Some(ItemStack::new(BlockId::Stone, 10));
    let mut held = Some(ItemStack::new(BlockId::Stone, 20));

    Inventory::handle_slot_left_click(&mut slot, &mut held);
    assert_eq!(slot, Some(ItemStack::new(BlockId::Stone, 30)));
    assert_eq!(held, None);

    Inventory::handle_slot_left_click(&mut slot, &mut held);
    assert_eq!(slot, None);
    assert_eq!(held, Some(ItemStack::new(BlockId::Stone, 30)));
}

#[test]
fn test_slot_right_click_split_and_drop_one() {
    let mut slot = Some(ItemStack::new(BlockId::Wood, 10));
    let mut held = None;

    Inventory::handle_slot_right_click(&mut slot, &mut held);
    assert_eq!(slot, Some(ItemStack::new(BlockId::Wood, 5)));
    assert_eq!(held, Some(ItemStack::new(BlockId::Wood, 5)));

    let mut empty_slot = None;
    Inventory::handle_slot_right_click(&mut empty_slot, &mut held);
    assert_eq!(empty_slot, Some(ItemStack::new(BlockId::Wood, 1)));
    assert_eq!(held, Some(ItemStack::new(BlockId::Wood, 4)));
}

#[test]
fn test_return_crafting_items() {
    let mut inv = Inventory::new();
    inv.crafting_2x2[0] = Some(ItemStack::new(BlockId::Wood, 2));
    inv.crafting_3x3[4] = Some(ItemStack::new(BlockId::Stick, 5));
    inv.held_item = Some(ItemStack::new(BlockId::IronPickaxe, 1));

    inv.return_crafting_items();
    assert!(inv.crafting_2x2.iter().all(|s| s.is_none()));
    assert!(inv.crafting_3x3.iter().all(|s| s.is_none()));
    assert!(inv.held_item.is_none());
    assert_eq!(inv.hotbar[0], Some(ItemStack::new(BlockId::Wood, 2)));
    assert_eq!(inv.hotbar[1], Some(ItemStack::new(BlockId::Stick, 5)));
    assert_eq!(inv.hotbar[2], Some(ItemStack::new(BlockId::IronPickaxe, 1)));
}

#[test]
fn count_item_includes_hotbar_main_and_offhand() {
    let mut inventory = Inventory::new();
    inventory.hotbar[0] = Some(ItemStack::new(BlockId::Wood, 2));
    inventory.main[0] = Some(ItemStack::new(BlockId::Wood, 3));
    inventory.offhand = Some(ItemStack::new(BlockId::Wood, 1));

    assert_eq!(inventory.count_item(BlockId::Wood), 6);
    assert_eq!(inventory.count_item(BlockId::Stone), 0);
}

#[test]
fn test_slot_right_drag_distribution() {
    let mut held = Some(ItemStack::new(BlockId::Planks, 4));
    let mut slot1 = None;
    let mut slot2 = None;
    let mut slot3 = Some(ItemStack::new(BlockId::Planks, 2));
    let mut slot4 = Some(ItemStack::new(BlockId::Stone, 5));

    Inventory::handle_slot_right_drag(&mut slot1, &mut held);
    assert_eq!(slot1, Some(ItemStack::new(BlockId::Planks, 1)));
    assert_eq!(held, Some(ItemStack::new(BlockId::Planks, 3)));

    Inventory::handle_slot_right_drag(&mut slot2, &mut held);
    assert_eq!(slot2, Some(ItemStack::new(BlockId::Planks, 1)));
    assert_eq!(held, Some(ItemStack::new(BlockId::Planks, 2)));

    Inventory::handle_slot_right_drag(&mut slot3, &mut held);
    assert_eq!(slot3, Some(ItemStack::new(BlockId::Planks, 3)));
    assert_eq!(held, Some(ItemStack::new(BlockId::Planks, 1)));

    Inventory::handle_slot_right_drag(&mut slot4, &mut held);
    assert_eq!(slot4, Some(ItemStack::new(BlockId::Stone, 5)));
    assert_eq!(held, Some(ItemStack::new(BlockId::Planks, 1)));
}

#[test]
fn test_item_stack_durability_and_wear() {
    let mut pick = ItemStack::new(BlockId::DiamondPickaxe, 1);
    assert_eq!(pick.durability, 1561);
    assert!(pick.block.is_damageable());
    assert_eq!(pick.durability_ratio(), 1.0);

    assert!(!pick.damage(10));
    assert_eq!(pick.durability, 1551);

    assert!(pick.damage(1551));
    assert_eq!(pick.durability, 0);

    let helm = ItemStack::new(BlockId::DiamondHelmet, 1);
    assert_eq!(helm.durability, 363);
    assert_eq!(BlockId::DiamondChestplate.max_durability(), 528);
    assert_eq!(BlockId::DiamondLeggings.max_durability(), 495);
    assert_eq!(BlockId::DiamondBoots.max_durability(), 429);
}
