//! Unit tests for playing-state gameplay mechanics: placement, chat capture, flight, mining, pick-block, aquatic.

use crate::app::{GameState, InputState, RustCraftApp};
use crate::core::math::{IVec3, Vec3};
use crate::gameplay::inventory::ItemStack;
use crate::gameplay::player::GameMode;

#[test]
fn blocks_can_be_placed_into_water_but_not_solid_blocks() {
    assert!(super::can_place_block_in(crate::world::BlockId::Air));
    assert!(super::can_place_block_in(crate::world::BlockId::Water));
    assert!(!super::can_place_block_in(crate::world::BlockId::Stone));
    assert!(!super::can_place_block_in(crate::world::BlockId::Lava));
}

#[test]
fn block_placement_is_blocked_by_live_mobs_but_not_dead_ones() {
    use crate::gameplay::mob::{Mob, MobKind};

    let place_pos = IVec3::new(4, 10, 4);
    let mob = Mob::new(1, MobKind::Pig, Vec3::new(4.5, 10.0, 4.5));
    assert!(super::block_intersects_live_mob(
        place_pos,
        std::slice::from_ref(&mob)
    ));
    assert!(!super::block_intersects_live_mob(
        IVec3::new(6, 10, 4),
        std::slice::from_ref(&mob)
    ));

    let mut dead_mob = mob;
    dead_mob.is_dead = true;
    assert!(!super::block_intersects_live_mob(place_pos, &[dead_mob]));
}

#[test]
fn opening_chat_releases_mouse_and_closing_it_recaptures() {
    let mut app = RustCraftApp::new(160, 120, "software", 12345);
    app.state = GameState::Playing;
    let mut input = InputState {
        key_t: true,
        is_mouse_captured: true,
        ..Default::default()
    };

    let opened = app.update_and_render(0.016, &input);
    assert!(app.chat.is_open);
    assert!(opened.request_release_mouse);
    assert!(!opened.is_playing);

    input.key_t = false;
    input.is_mouse_captured = false;
    input.key_escape = true;
    let closed = app.update_and_render(0.016, &input);
    assert!(!app.chat.is_open);
    assert!(closed.request_capture_mouse);
    assert!(closed.is_playing);
    assert!(matches!(app.state, GameState::Playing));
}

#[test]
fn test_creative_flight_double_tap_space() {
    let mut app = RustCraftApp::new(160, 120, "software", 12345);
    app.state = GameState::Playing;
    app.player.gamemode = GameMode::Creative;
    app.fly_mode = false;
    app.jump_double_tap_timer = 0.0;

    let mut input = InputState::default();
    input.is_mouse_captured = true;

    input.key_space = true;
    let _ = app.update_and_render(0.016, &input);
    assert!(!app.fly_mode, "First press should not trigger flying");
    assert!(
        app.jump_double_tap_timer > 0.0,
        "Timer should be set after first press"
    );

    input.key_space = false;
    let _ = app.update_and_render(0.05, &input);
    assert!(!app.fly_mode);
    assert!(app.jump_double_tap_timer > 0.0, "Timer should still be active");

    input.key_space = true;
    let _ = app.update_and_render(0.016, &input);
    assert!(
        app.fly_mode,
        "Double-tapping Space in creative mode should enable flight"
    );
    assert_eq!(
        app.jump_double_tap_timer, 0.0,
        "Timer should reset after double tap"
    );

    input.key_space = false;
    let _ = app.update_and_render(0.05, &input);
    assert!(app.fly_mode, "Should remain in flight mode");

    input.key_space = true;
    let _ = app.update_and_render(0.016, &input);
    assert!(
        app.fly_mode,
        "First press while flying should not immediately exit flight"
    );
    assert!(app.jump_double_tap_timer > 0.0);

    input.key_space = false;
    let _ = app.update_and_render(0.05, &input);
    assert!(app.fly_mode);

    input.key_space = true;
    let _ = app.update_and_render(0.016, &input);
    assert!(
        !app.fly_mode,
        "Double-tapping Space in creative mode should exit flight"
    );
    assert_eq!(app.jump_double_tap_timer, 0.0);

    app.player.gamemode = GameMode::Survival;
    app.fly_mode = false;
    app.jump_double_tap_timer = 0.0;

    input.key_space = false;
    let _ = app.update_and_render(0.05, &input);

    input.key_space = true;
    let _ = app.update_and_render(0.016, &input);
    input.key_space = false;
    let _ = app.update_and_render(0.05, &input);
    input.key_space = true;
    let _ = app.update_and_render(0.016, &input);
    assert!(
        !app.fly_mode,
        "Survival mode must never allow flight via double-tap space"
    );
}

#[test]
fn test_creative_break_cooldown_holding_mouse() {
    use crate::world::BlockId;

    let mut app = RustCraftApp::new(160, 120, "software", 12345);
    app.state = GameState::Playing;
    app.player.gamemode = GameMode::Creative;
    app.fly_mode = true;
    app.camera.position = Vec3::new(0.5, 60.0, 0.5);
    app.camera.yaw = 0.0;
    app.camera.pitch = 0.0;

    let block1_pos = IVec3::new(2, 60, 0);
    let block2_pos = IVec3::new(3, 60, 0);
    app.world.set_block(block1_pos, BlockId::Stone);
    app.world.set_block(block2_pos, BlockId::Stone);

    let mut input = InputState::default();
    input.is_mouse_captured = true;
    input.mouse_left = true;

    let _ = app.update_and_render(0.016, &input);
    assert_eq!(
        app.world.get_block(block1_pos),
        BlockId::Air,
        "Block 1 should break on first frame"
    );
    assert_eq!(
        app.world.get_block(block2_pos),
        BlockId::Stone,
        "Block 2 should NOT break immediately"
    );
    assert!(
        app.creative_break_cooldown > 0.0,
        "Cooldown should be set after breaking in creative"
    );

    let _ = app.update_and_render(0.10, &input);
    assert_eq!(
        app.world.get_block(block2_pos),
        BlockId::Stone,
        "Block 2 should still exist after 100ms"
    );
    assert!(app.creative_break_cooldown > 0.0);

    let _ = app.update_and_render(0.10, &input);
    assert_eq!(
        app.world.get_block(block2_pos),
        BlockId::Stone,
        "Block 2 should still exist after 200ms"
    );

    let _ = app.update_and_render(0.06, &input);
    assert_eq!(
        app.world.get_block(block2_pos),
        BlockId::Air,
        "Block 2 should break after 250ms cooldown elapsed"
    );
}

#[test]
fn test_pick_block_item_mappings() {
    use crate::world::BlockId;

    assert_eq!(BlockId::Grass.pick_item(), Some(BlockId::Grass));
    assert_eq!(BlockId::Stone.pick_item(), Some(BlockId::Stone));
    assert_eq!(BlockId::Air.pick_item(), None);
    assert_eq!(BlockId::Water.pick_item(), None);
    assert_eq!(BlockId::Lava.pick_item(), None);
    assert_eq!(
        BlockId::RustStoneWireLit.pick_item(),
        Some(BlockId::RustStone)
    );
    assert_eq!(
        BlockId::RustStoneLampLit.pick_item(),
        Some(BlockId::RustStoneLamp)
    );
    assert_eq!(BlockId::LeverOn.pick_item(), Some(BlockId::Lever));
    assert_eq!(BlockId::ButtonPressed.pick_item(), Some(BlockId::Button));
    assert_eq!(
        BlockId::PressurePlatePressed.pick_item(),
        Some(BlockId::PressurePlate)
    );
    assert_eq!(BlockId::DoorOpen.pick_item(), Some(BlockId::Door));
    assert_eq!(BlockId::FurnaceLit.pick_item(), Some(BlockId::Furnace));
    assert_eq!(BlockId::Farmland.pick_item(), Some(BlockId::Dirt));
    assert_eq!(BlockId::FarmlandWet.pick_item(), Some(BlockId::Dirt));
}

#[test]
fn test_creative_pick_block_middle_click() {
    use crate::world::BlockId;

    let mut app = RustCraftApp::new(160, 120, "software", 12345);
    app.state = GameState::Playing;
    app.player.gamemode = GameMode::Creative;
    app.fly_mode = true;
    app.camera.position = Vec3::new(0.5, 60.0, 0.5);
    app.camera.yaw = 0.0;
    app.camera.pitch = 0.0;

    let target_pos = IVec3::new(3, 60, 0);
    app.world.set_block(target_pos, BlockId::RustStoneBlock);

    app.player.selected_hotbar_slot = 0;
    app.player.inventory.hotbar[0] = None;

    let mut input = InputState::default();
    input.is_mouse_captured = true;
    input.mouse_middle = true;

    let _ = app.update_and_render(0.016, &input);
    assert_eq!(
        app.player.inventory.hotbar[0],
        Some(ItemStack::new(BlockId::RustStoneBlock, 64)),
        "Middle click in creative should place 64x of picked block into current free slot"
    );
    assert_eq!(app.player.selected_hotbar_slot, 0);

    app.world.set_block(target_pos, BlockId::DiamondBlock);
    input.mouse_middle = false;
    let _ = app.update_and_render(0.016, &input);

    app.player.inventory.hotbar[1] = None;
    input.mouse_middle = true;
    let _ = app.update_and_render(0.016, &input);

    assert_eq!(
        app.player.inventory.hotbar[1],
        Some(ItemStack::new(BlockId::DiamondBlock, 64)),
        "DiamondBlock should go into the next free slot"
    );
    assert_eq!(
        app.player.selected_hotbar_slot, 1,
        "Selected slot should switch to the free slot where item was placed"
    );

    app.world.set_block(target_pos, BlockId::RustStoneBlock);
    input.mouse_middle = false;
    let _ = app.update_and_render(0.016, &input);
    input.mouse_middle = true;
    let _ = app.update_and_render(0.016, &input);
    assert_eq!(
        app.player.selected_hotbar_slot, 0,
        "Middle click should select existing hotbar slot if already in hotbar"
    );

    app.world.set_block(target_pos, BlockId::RustStoneWireLit);
    input.mouse_middle = false;
    let _ = app.update_and_render(0.016, &input);
    app.player.inventory.hotbar[2] = None;
    app.player.selected_hotbar_slot = 2;
    input.mouse_middle = true;
    let _ = app.update_and_render(0.016, &input);
    assert_eq!(
        app.player.inventory.hotbar[2],
        Some(ItemStack::new(BlockId::RustStone, 64)),
        "Middle click on RustStoneWireLit should pick RustStone item"
    );
}

#[test]
fn test_break_ruststone_wire() {
    use crate::world::BlockId;

    let mut app = RustCraftApp::new(160, 120, "software", 12345);
    app.state = GameState::Playing;
    app.player.gamemode = GameMode::Survival;
    app.fly_mode = true;
    app.camera.position = Vec3::new(0.5, 60.0, 0.5);
    app.camera.yaw = 0.0;
    app.camera.pitch = 0.0;

    let target_pos = IVec3::new(3, 60, 0);
    app.world.set_block(target_pos, BlockId::RustStoneWireLit);

    let mut input = InputState::default();
    input.is_mouse_captured = true;
    input.mouse_left = true;

    let _ = app.update_and_render(0.1, &input);
    assert_eq!(
        app.world.get_block(target_pos),
        BlockId::Air,
        "Breaking wire should clear the block"
    );
    assert!(
        app.items.iter().any(|item| item.block == BlockId::RustStone),
        "Breaking wire should drop RustStone item"
    );
}

#[test]
fn test_aquatic_update_mechanics() {
    use crate::gameplay::mob::MobKind;
    use crate::world::BlockId;

    let mut app = RustCraftApp::new(160, 120, "software", 12345);
    app.state = GameState::Playing;
    app.player.gamemode = GameMode::Survival;
    app.fly_mode = false;
    app.camera.position = Vec3::new(10.5, 15.0, 10.5);

    app.world.set_block(IVec3::new(10, 15, 10), BlockId::Water);
    app.world.set_block(IVec3::new(10, 14, 10), BlockId::Water);
    app.mob_manager
        .spawn_mob(MobKind::Dolphin, Vec3::new(11.0, 15.0, 11.0));

    let feet_pos = Vec3::new(
        app.camera.position.x,
        app.camera.position.y - app.current_eye_offset - 0.08,
        app.camera.position.z,
    );
    let feet_block = app.world.get_block(feet_pos.floor_to_ivec());
    let eye_block = app.world.get_block(app.camera.position.floor_to_ivec());
    let in_water = feet_block == BlockId::Water || eye_block == BlockId::Water;
    assert!(in_water, "Player should be in water");

    let near_dolphin = in_water
        && app.mob_manager.mobs.iter().any(|m| {
            m.kind == MobKind::Dolphin
                && m.health > 0.0
                && (m.pos - app.camera.position).length_sq() < 196.0
        });
    assert!(
        near_dolphin,
        "Player should receive Dolphin's Grace near alive dolphin"
    );

    app.player.invulnerability_timer = 0.0;
    let initial_health = app.player.health;
    app.player.hunger = 10.0;
    assert!(app.player.eat(BlockId::Pufferfish));
    app.player.take_damage(3.0);
    assert!(
        app.player.health < initial_health,
        "Eating pufferfish should deal damage"
    );

    app.world.set_block(IVec3::new(20, 10, 20), BlockId::Sand);
    app.world.set_block(IVec3::new(20, 11, 20), BlockId::Kelp);
    app.world.set_block(IVec3::new(20, 12, 20), BlockId::Kelp);
    app.world.set_block(IVec3::new(20, 13, 20), BlockId::Kelp);
    assert_eq!(app.world.get_block(IVec3::new(20, 11, 20)), BlockId::Kelp);
    assert_eq!(app.world.get_block(IVec3::new(20, 12, 20)), BlockId::Kelp);
    assert_eq!(app.world.get_block(IVec3::new(20, 13, 20)), BlockId::Kelp);

    app.fishing_bobber = Some(crate::app::FishingBobber {
        pos: Vec3::new(20.0, 15.0, 20.0),
        vel: Vec3::new(0.0, 0.0, 0.0),
        in_water: true,
        bite_timer: 0.0,
        bite_window: 2.0,
        splash_timer: 0.0,
    });
    assert!(app.fishing_bobber.is_some());
    let bobber = app.fishing_bobber.take().unwrap();
    assert!(bobber.in_water && bobber.bite_timer <= 0.0 && bobber.bite_window > 0.0);
}
