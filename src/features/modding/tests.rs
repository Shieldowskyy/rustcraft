use std::path::Path;

use crate::core::math::Vec3;
use crate::features::blocks::BlockId;
use crate::features::entities::{MobManager, ParticleSystem};
use crate::features::player::{Camera, ItemStack, Player, RecipeBook};
use crate::features::rendering::atlas::TextureId;
use crate::features::ui::chat::{ChatState, CommandContext};
use crate::features::world::World;

use super::registry::get_registry;
use super::*;

#[test]
fn test_json_parser_basic() {
    let json = r#"
    // This is a comment
    {
        "id": "emerald_mod",
        "name": "Emerald Mod",
        "version": "1.0",
        "blocks": [
            {
                "id": 85,
                "identifier": "emerald_ore",
                "solid": true,
                "hardness": 3.0
            }
        ],
        "commands": [
            {
                "name": "emerald_kit",
                "actions": [
                    { "type": "give", "item": "emerald_ore", "count": 16 }
                ]
            }
        ]
    }
    "#;
    let root = parse_json_value_root(json).expect("JSON should parse");
    let obj = root.as_object().unwrap();
    assert_eq!(obj.get("id").unwrap().as_str().unwrap(), "emerald_mod");
    let blocks = obj.get("blocks").unwrap().as_array().unwrap();
    assert_eq!(blocks.len(), 1);
    let ore = blocks[0].as_object().unwrap();
    assert_eq!(ore.get("id").unwrap().as_u8().unwrap(), 85);
    assert_eq!(ore.get("hardness").unwrap().as_f32().unwrap(), 3.0);
}

#[test]
fn test_mod_registration_and_queries() {
    let json = r#"{
        "id": "test_mod",
        "name": "Test Mod",
        "blocks": [
            {
                "id": 212,
                "identifier": "test_ruby_ore",
                "name": "Ruby Ore",
                "solid": true,
                "hardness": 4.5,
                "texture": {
                    "pattern": "ore",
                    "ore_color": [1.0, 0.0, 0.2]
                }
            }
        ],
        "items": [
            {
                "id": 213,
                "identifier": "test_ruby",
                "name": "Ruby Gem",
                "is_food": false
            }
        ],
        "smelting": [
            { "input": "test_ruby_ore", "output": "test_ruby" }
        ]
    }"#;

    let mut book = RecipeBook::new();
    let manifest =
        ModManager::parse_and_register_mod(json, Path::new("mods/test"), &mut book).unwrap();
    assert_eq!(manifest.id, "test_mod");

    assert_eq!(
        find_custom_block_by_name("test_ruby_ore"),
        Some(BlockId::Custom(212))
    );
    assert_eq!(
        find_custom_block_by_name("test_ruby"),
        Some(BlockId::Custom(213))
    );
    assert_eq!(get_custom_name(212), "Ruby Ore");
    assert_eq!(get_custom_name(213), "Ruby Gem");
    assert_eq!(get_custom_texture_id(212), TextureId::Custom(26));
    let expected_texture = get_registry()
        .read()
        .unwrap()
        .blocks_by_id
        .get(&212)
        .unwrap()
        .texture
        .texels
        .clone();
    assert_eq!(generate_mod_textures()[26].texels, expected_texture);
    assert!(is_custom_solid(212));
    assert_eq!(
        get_custom_smelting(BlockId::Custom(212)),
        Some(BlockId::Custom(213))
    );
}

#[test]
fn test_multiplayer_mod_sync() {
    let json = r#"{
        "id": "net_mod",
        "name": "Network Sync Mod",
        "blocks": [
            {
                "id": 214,
                "identifier": "net_block",
                "name": "Network Block"
            }
        ]
    }"#;

    let mut book = RecipeBook::new();
    let manifest =
        ModManager::register_mod_from_str(json, Path::new("mods/net_mod"), &mut book).unwrap();
    assert_eq!(manifest.id, "net_mod");

    let active_files = ModManager::get_active_mod_files();
    assert!(active_files
        .iter()
        .any(|m| m.mod_id == "net_mod" && m.content.contains("Network Block")));
    assert_eq!(
        find_custom_block_by_name("net_block"),
        Some(BlockId::Custom(214))
    );
}

#[test]
fn test_mod_food_and_nutrition() {
    let json = r#"{
        "id": "food_mod",
        "name": "Food Mod",
        "items": [
            {
                "id": 186,
                "identifier": "super_pie",
                "name": "Super Pie",
                "is_food": true,
                "nutrition": [8.0, 5.0]
            }
        ]
    }"#;

    let mut book = RecipeBook::new();
    let _ = ModManager::register_mod_from_str(json, Path::new("mods/food"), &mut book).unwrap();

    assert!(is_custom_food(186));
    assert_eq!(get_custom_nutrition(186), Some((8.0, 5.0)));
}

#[test]
fn test_mod_ids_reject_vanilla_and_unmapped_texture_ranges() {
    for id in [185, 250] {
        let json = format!(
            r#"{{"id":"invalid_id","blocks":[{{"id":{id},"identifier":"invalid","name":"Invalid"}}]}}"#
        );
        let mut book = RecipeBook::new();
        let result =
            ModManager::register_mod_from_str(&json, Path::new("mods/invalid"), &mut book);
        assert!(result.is_err(), "ID {id} should be rejected");
    }
}

#[test]
fn test_mod_crafting_recipes_match_grid() {
    let json = r#"{
        "id": "craft_mod",
        "name": "Craft Mod",
        "blocks": [
            { "id": 220, "identifier": "craft_block", "name": "Craft Block" }
        ],
        "items": [
            { "id": 221, "identifier": "craft_gem", "name": "Craft Gem" }
        ],
        "recipes": [
            {
                "type": "shaped",
                "grid": [
                    ["craft_gem", "craft_gem"],
                    ["craft_gem", "craft_gem"]
                ],
                "result": "craft_block",
                "count": 1
            }
        ]
    }"#;

    let mut book = RecipeBook::new();
    let _ = ModManager::register_mod_from_str(json, Path::new("mods/craft"), &mut book).unwrap();

    let gem_item = Some(ItemStack::new(BlockId::Custom(221), 1));
    let grid_2x2 = [gem_item, gem_item, gem_item, gem_item];
    let result = book.match_grid(&grid_2x2, 2, 2);
    assert!(result.is_some());
    let res_stack = result.unwrap();
    assert_eq!(res_stack.block, BlockId::Custom(220));
    assert_eq!(res_stack.count, 1);
}

#[test]
fn test_mod_block_custom_drops_and_mob_loot() {
    let json = r#"{
        "id": "loot_mod",
        "name": "Loot Mod",
        "blocks": [
            {
                "id": 225,
                "identifier": "crystal_ore",
                "name": "Crystal Ore",
                "drops": { "item": "crystal_shard", "count": 2 }
            }
        ],
        "items": [
            { "id": 226, "identifier": "crystal_shard", "name": "Crystal Shard" }
        ],
        "loot": [
            { "target": "zombie", "item": "crystal_shard", "count": 3, "chance": 1.0 }
        ]
    }"#;

    let mut book = RecipeBook::new();
    let _ = ModManager::register_mod_from_str(json, Path::new("mods/loot"), &mut book).unwrap();

    let drop = get_custom_block_drop(BlockId::Custom(225));
    assert_eq!(drop, Some((BlockId::Custom(226), 2)));

    let mob_drops = get_custom_mob_drops("zombie");
    assert!(mob_drops
        .iter()
        .any(|&(b, count)| b == BlockId::Custom(226) && count == 3));
}

#[test]
fn test_mod_command_dispatch() {
    let json = r#"{
        "id": "cmd_mod",
        "name": "Cmd Mod",
        "commands": [
            {
                "name": "super_heal",
                "description": "Heals player to max",
                "actions": [
                    { "type": "heal" },
                    { "type": "message", "text": "Super healed!" }
                ]
            }
        ]
    }"#;

    let mut book = RecipeBook::new();
    let _ = ModManager::register_mod_from_str(json, Path::new("mods/cmd"), &mut book).unwrap();

    let mut chat = ChatState::new();
    let mut mob_manager = MobManager::new();
    let mut player = Player::new();
    player.health = 5.0;
    player.hunger = 2.0;

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

    chat.execute_command("/super_heal", &mut ctx);
    assert_eq!(player.health, player.max_health);
    assert_eq!(player.hunger, player.max_hunger);
    assert!(chat.messages.iter().any(|m| m.text.contains("Super healed!")));
}

#[test]
fn test_mod_bundled_emerald_expansion_file_validity() {
    let path = Path::new("mods/emerald_expansion/mod.json");
    assert!(
        path.exists(),
        "Reference mod file mods/emerald_expansion/mod.json must exist"
    );
    let content = std::fs::read_to_string(path).expect("Must read emerald_expansion mod.json");

    let mut book = RecipeBook::new();
    let manifest = ModManager::register_mod_from_str(&content, path.parent().unwrap(), &mut book)
        .expect("Reference mod must register cleanly without errors");

    assert_eq!(manifest.id, "emerald_expansion");
    assert!(find_custom_block_by_name("emerald_ore").is_some());
    assert!(find_custom_block_by_name("emerald").is_some());
    assert!(find_custom_block_by_name("emerald_apple").is_some());
    assert!(find_custom_block_by_name("ruby_ore").is_some());
    assert!(find_custom_block_by_name("ruby").is_some());
    assert!(find_custom_block_by_name("ruby_block").is_some());
}
