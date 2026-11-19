//! Testy jednostkowe stanu czatu, autouzupełniania, zaznaczania tekstu, schowka i komend `/mobs` oraz `/weather`.

use super::*;
use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::entities::{MobKind, MobManager, ParticleSystem};
use crate::features::player::{Camera, Player};
use crate::features::rendering::font;
use crate::features::ui::draw::pack_color;
use crate::features::world::{WeatherType, World};
use crate::platform::clipboard::{CLIPBOARD_TEST_LOCK, internal_clipboard_text};

#[test]
fn test_chat_open_close_and_history() {
    let mut chat = ChatState::new();
    assert!(!chat.is_open);

    chat.open("hello");
    assert!(chat.is_open);
    assert_eq!(chat.input_text, "hello");
    assert_eq!(chat.cursor_pos, 5);

    chat.close();
    assert!(!chat.is_open);
    assert_eq!(chat.input_text, "");
}

#[test]
fn tab_completes_and_cycles_matching_player_names() {
    let mut chat = ChatState::new();
    chat.open("/tp Al");
    let names = vec!["Alex".to_string(), "Alice".to_string(), "Bob".to_string()];

    chat.complete_player_name(&names);
    assert_eq!(chat.input_text, "/tp Alex");

    chat.complete_player_name(&names);
    assert_eq!(chat.input_text, "/tp Alice");
}

#[test]
fn mouse_selection_copies_text_across_chat_lines() {
    let mut chat = ChatState::new();
    chat.add_message("first", pack_color(1.0, 1.0, 1.0));
    chat.add_message("second", pack_color(1.0, 1.0, 1.0));
    chat.open("");

    let first_line_y = 200 - 18 - 3 - 22;
    let latest_line_y = first_line_y + 11;
    let first_char_width = font::measure_text("f", 1, 1);
    let third_char_width = font::measure_text("sec", 1, 1);
    let mut input = crate::app::InputState {
        mouse_pos: Some((8.0 + first_char_width as f32, first_line_y as f32)),
        mouse_left: true,
        ..Default::default()
    };
    chat.update_text_selection(&input, 320, 200);

    input.mouse_pos = Some((8.0 + third_char_width as f32, latest_line_y as f32));
    chat.update_text_selection(&input, 320, 200);
    assert_eq!(chat.selected_text().as_deref(), Some("irst\nsec"));
}

#[test]
fn ctrl_a_selects_input_and_selection_can_be_replaced() {
    let mut chat = ChatState::new();
    chat.open("draft");
    let input = crate::app::InputState {
        key_ctrl: true,
        key_a: true,
        ..Default::default()
    };

    chat.handle_input(&input, &[]);
    assert_eq!(chat.selected_copy_text().as_deref(), Some("draft"));

    chat.paste_clipboard_text("replacement");
    assert_eq!(chat.input_text, "replacement");
}

#[test]
fn input_context_menu_select_all_then_paste_replaces_text() {
    let mut chat = ChatState::new();
    chat.open("draft");
    let mut input = crate::app::InputState {
        mouse_pos: Some((30.0, 182.0)),
        mouse_right: true,
        ..Default::default()
    };
    chat.update_text_selection(&input, 320, 200);
    input.mouse_right = false;
    chat.update_text_selection(&input, 320, 200);
    let menu = chat
        .context_menu
        .expect("right-clicking the input should open its context menu");

    input.mouse_left = true;
    input.mouse_pos = Some(((menu.x + 8) as f32, (menu.y + 43) as f32));
    chat.update_text_selection(&input, 320, 200);
    assert_eq!(chat.selected_copy_text().as_deref(), Some("draft"));

    chat.paste_clipboard_text("replacement");
    assert_eq!(chat.input_text, "replacement");
}

#[test]
fn ctrl_c_copies_selected_chat_text() {
    let _clipboard_lock = CLIPBOARD_TEST_LOCK
        .lock()
        .expect("clipboard test lock poisoned");
    let mut chat = ChatState::new();
    chat.add_message("selected", pack_color(1.0, 1.0, 1.0));
    chat.open("");
    chat.selection_anchor = Some((0, 0));
    chat.selection_focus = Some((0, 8));

    chat.handle_input(
        &crate::app::InputState {
            key_ctrl: true,
            key_c: true,
            ..Default::default()
        },
        &[],
    );

    assert_eq!(internal_clipboard_text(), "selected");
}

#[test]
fn right_click_chat_context_menu_can_copy_and_select_all() {
    let _clipboard_lock = CLIPBOARD_TEST_LOCK
        .lock()
        .expect("clipboard test lock poisoned");
    let mut chat = ChatState::new();
    chat.add_message("first", pack_color(1.0, 1.0, 1.0));
    chat.add_message("second", pack_color(1.0, 1.0, 1.0));
    chat.open("");

    let mut input = crate::app::InputState {
        mouse_pos: Some((12.0, 168.0)),
        mouse_right: true,
        ..Default::default()
    };
    chat.update_text_selection(&input, 320, 200);
    assert!(chat.context_menu.is_some());

    input.mouse_right = false;
    chat.update_text_selection(&input, 320, 200);
    let menu = chat
        .context_menu
        .expect("right-click menu should remain open");
    input.mouse_pos = Some(((menu.x + 8) as f32, (menu.y + 5) as f32));
    input.mouse_left = true;
    chat.update_text_selection(&input, 320, 200);
    assert_eq!(internal_clipboard_text(), "second");
    assert!(chat.context_menu.is_none());

    input.mouse_left = false;
    input.mouse_right = true;
    input.mouse_pos = Some((12.0, 168.0));
    chat.update_text_selection(&input, 320, 200);
    input.mouse_right = false;
    chat.update_text_selection(&input, 320, 200);
    let menu = chat
        .context_menu
        .expect("right-click menu should remain open");
    input.mouse_left = true;
    input.mouse_pos = Some(((menu.x + 8) as f32, (menu.y + 43) as f32));
    chat.update_text_selection(&input, 320, 200);
    assert_eq!(chat.selected_text().as_deref(), Some("first\nsecond"));
}

#[test]
fn test_parse_block_id_items() {
    assert_eq!(parse_block_id("diamond_sword"), Some(BlockId::IronSword));
    assert_eq!(parse_block_id("wood"), Some(BlockId::Wood));
    assert_eq!(parse_block_id("raw_porkchop"), Some(BlockId::RawPorkchop));
    assert_eq!(parse_block_id("cooked_beef"), Some(BlockId::CookedBeef));
    assert_eq!(parse_block_id("apple"), Some(BlockId::Apple));
    assert_eq!(parse_block_id("torch"), Some(BlockId::Torch));
    assert_eq!(parse_block_id("bone"), Some(BlockId::Bone));
    assert_eq!(parse_block_id("kosc"), Some(BlockId::Bone));
    assert_eq!(parse_block_id("gunpowder"), Some(BlockId::Gunpowder));
    assert_eq!(parse_block_id("proch"), Some(BlockId::Gunpowder));
}

#[test]
fn test_chat_mobs_debug_command() {
    let mut chat = ChatState::new();
    let mut mob_manager = MobManager::new();
    mob_manager.spawn_mob(MobKind::Zombie, Vec3::new(10.0, 64.0, 10.0));
    mob_manager.spawn_mob(MobKind::Skeleton, Vec3::new(15.0, 64.0, 15.0));
    mob_manager.spawn_mob(MobKind::Reaper, Vec3::new(8.0, 64.0, 8.0));

    let mut player = Player::new();
    let mut camera = Camera::new(Vec3::new(0.0, 64.0, 0.0));
    let mut world = World::new();
    let mut items = Vec::new();
    let mut particles = ParticleSystem::new();
    let mut is_flying = false;
    let mut fly_vel = Vec3::zero();
    let mut horiz_vel = Vec3::zero();
    let mut reload_chunks = false;

    let mut ctx = CommandContext {
        player: &mut player,
        camera: &mut camera,
        world: &mut world,
        mob_manager: &mut mob_manager,
        items: &mut items,
        particles: &mut particles,
        is_flying: &mut is_flying,
        fly_velocity: &mut fly_vel,
        horiz_velocity: &mut horiz_vel,
        reload_chunks_requested: &mut reload_chunks,
    };

    chat.execute_command("/mobs", &mut ctx);
    assert!(!chat.messages.is_empty());
    assert!(chat.messages.iter().any(|m| m.text.contains("Spawned Mobs")));
    assert!(chat.messages.iter().any(|m| m.text.contains("Zombie")));
    assert!(chat.messages.iter().any(|m| m.text.contains("Skeleton")));
}

#[test]
fn test_chat_weather_commands() {
    let mut chat = ChatState::new();
    let mut mob_manager = MobManager::new();
    let mut player = Player::new();
    let mut camera = Camera::new(Vec3::new(0.0, 64.0, 0.0));
    let mut world = World::new();
    let mut items = Vec::new();
    let mut particles = ParticleSystem::new();
    let mut is_flying = false;
    let mut fly_vel = Vec3::zero();
    let mut horiz_vel = Vec3::zero();
    let mut reload_chunks = false;

    let mut ctx = CommandContext {
        player: &mut player,
        camera: &mut camera,
        world: &mut world,
        mob_manager: &mut mob_manager,
        items: &mut items,
        particles: &mut particles,
        is_flying: &mut is_flying,
        fly_velocity: &mut fly_vel,
        horiz_velocity: &mut horiz_vel,
        reload_chunks_requested: &mut reload_chunks,
    };

    chat.execute_command("/weather rain 120", &mut ctx);
    assert_eq!(ctx.world.weather.current_type, WeatherType::Rain);
    assert_eq!(ctx.world.weather.timer, 120.0);

    chat.execute_command("/weather thunder 60", &mut ctx);
    assert_eq!(ctx.world.weather.current_type, WeatherType::Thunder);
    assert_eq!(ctx.world.weather.timer, 60.0);

    chat.execute_command("/weather query", &mut ctx);
    assert!(chat
        .messages
        .iter()
        .any(|m| m.text.contains("weather is currently thunder")));

    chat.execute_command("/weather clear", &mut ctx);
    assert_eq!(ctx.world.weather.current_type, WeatherType::Clear);
}
