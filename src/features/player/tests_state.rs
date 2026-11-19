//! Unit tests for player air, hunger, eating, fire state, and save/restore snapshots.

use crate::features::blocks::BlockId;

use super::inventory::ItemStack;
use super::state::{GameMode, Player};

#[test]
fn test_player_air_initialization() {
    let player = Player::new();
    assert_eq!(player.air, 20.0);
    assert_eq!(player.max_air, 20.0);
    assert!(!player.head_underwater);
}

#[test]
fn test_player_hunger_and_eating() {
    let mut player = Player::new();
    player.gamemode = GameMode::Survival;
    player.hunger = 10.0;
    player.saturation = 0.0;

    assert!(player.can_eat(BlockId::CookedBeef));
    assert!(!player.can_eat(BlockId::Stone));

    assert!(player.eat(BlockId::CookedBeef));
    assert_eq!(player.hunger, 18.0);
    assert!(player.saturation > 0.0);

    player.eat(BlockId::Apple);
    assert_eq!(player.hunger, 20.0);
    assert!(!player.can_eat(BlockId::Apple));

    let mut creative_player = Player::new();
    creative_player.gamemode = GameMode::Creative;
    creative_player.hunger = 10.0;
    assert!(!creative_player.can_eat(BlockId::CookedBeef));
    assert!(!creative_player.can_eat(BlockId::Apple));
}

#[test]
fn test_player_save_and_apply_state() {
    let mut player = Player::new();
    player.health = 14.0;
    player.hunger = 12.0;
    player.saturation = 3.5;
    player.exhaustion = 1.2;
    player.air = 15.0;
    player.selected_hotbar_slot = 3;
    player.inventory.hotbar[3] = Some(ItemStack::new(BlockId::IronPickaxe, 1));

    let saved = player.save_state();
    assert_eq!(saved.health, 14.0);
    assert_eq!(saved.hunger, 12.0);
    assert_eq!(saved.selected_hotbar_slot, 3);
    assert_eq!(saved.hotbar[3].unwrap().block, BlockId::IronPickaxe);

    player.reset(GameMode::Creative);
    assert_eq!(player.health, 20.0);
    assert_eq!(player.hunger, 20.0);
    assert_eq!(player.selected_hotbar_slot, 0);
    assert!(player.inventory.hotbar[3].is_none());

    player.apply_saved_state(&saved);
    assert_eq!(player.health, 14.0);
    assert_eq!(player.hunger, 12.0);
    assert_eq!(player.selected_hotbar_slot, 3);
    assert_eq!(player.inventory.hotbar[3].unwrap().block, BlockId::IronPickaxe);
}

#[test]
fn test_player_fire_state() {
    let mut player = Player::new();
    assert_eq!(player.fire_timer, 0.0);
    assert_eq!(player.fire_damage_timer, 0.0);

    player.fire_timer = 5.0;
    player.reset(GameMode::Survival);
    assert_eq!(player.fire_timer, 0.0);
}
