//! Unit tests verifying block metadata, tool mining tiers, ruststone, i18n, and table invariants.
//! Migrated from `src/world/block.rs` alongside `BlockId::VANILLA_ALL` structural checks.

use crate::features::rendering::atlas::TextureId;

use super::constants::VANILLA_BLOCK_COUNT;
use super::{ArmorSlot, BlockId, BlockRegistry, Face};

#[test]
fn test_meat_properties_and_textures() {
    let meats = [
        (BlockId::RawPorkchop, TextureId::RawPorkchop),
        (BlockId::CookedPorkchop, TextureId::CookedPorkchop),
        (BlockId::RawBeef, TextureId::RawBeef),
        (BlockId::CookedBeef, TextureId::CookedBeef),
        (BlockId::RawChicken, TextureId::RawChicken),
        (BlockId::CookedChicken, TextureId::CookedChicken),
    ];

    for (block, expected_tex) in meats {
        assert!(block.is_meat(), "{:?} should be classified as meat", block);
        assert!(block.is_food(), "{:?} should be classified as food", block);
        assert!(!block.is_placeable(), "{:?} must not be placeable as a block", block);
        assert_ne!(block.texture_id(Face::Top), TextureId::Planks);
        assert_eq!(block.texture_id(Face::Top), expected_tex);
    }

    assert!(!BlockId::Planks.is_meat());
    assert!(!BlockId::Dirt.is_meat());
    assert!(!BlockId::IronSword.is_meat());
}

#[test]
fn test_new_blocks_and_tools() {
    assert_eq!(BlockId::VANILLA_ALL.len(), VANILLA_BLOCK_COUNT);
    for id in 0..=185u8 {
        let block = BlockId::from_u8(id);
        assert_eq!(block.to_u8(), id);
        assert_eq!(block.is_opaque(), block.is_solid() && !block.is_transparent());
    }

    let custom = BlockId::from_u8(186);
    assert_eq!(custom, BlockId::Custom(186));
    assert_eq!(custom.to_u8(), 186);

    assert_eq!(BlockId::DiamondChestplate.armor_defense(), 8.0);
    assert_eq!(BlockId::DiamondChestplate.armor_slot(), Some(ArmorSlot::Chestplate));
    assert!(BlockId::DiamondChestplate.is_armor());
    assert!(!BlockId::DiamondChestplate.is_solid());
    assert!(!BlockId::DiamondChestplate.is_placeable());
    assert!(BlockId::Shield.is_shield());
    assert!(BlockId::Bow.is_bow());
    assert!(BlockId::Arrow.is_arrow());

    assert!(BlockId::Clay.is_solid());
    assert!(BlockId::Clay.is_placeable());
    assert!(!BlockId::ClayBall.is_solid());
    assert!(!BlockId::ClayBall.is_placeable());
    assert!(BlockId::Bricks.is_solid());
    assert!(BlockId::BrickStairs.is_stairs());
    assert!(BlockId::BrickSlab.is_half_block());
    assert!(BlockId::Gravel.is_solid());
    assert!(!BlockId::Flint.is_solid());
    assert!(!BlockId::Flint.is_placeable());
    assert!(BlockId::Cobblestone.is_solid());
    assert!(BlockId::MossyCobblestone.is_solid());
    assert!(BlockId::CobblestoneStairs.is_stairs());
    assert!(BlockId::CobblestoneSlab.is_half_block());
    assert!(BlockId::Sandstone.is_solid());
    assert!(BlockId::SandstoneStairs.is_stairs());
    assert!(BlockId::SandstoneSlab.is_half_block());
    assert!(BlockId::SmoothStone.is_solid());
    assert!(BlockId::Obsidian.is_solid());
    assert_eq!(BlockId::Obsidian.break_time(), 9.0);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::DiamondPickaxe), BlockId::Obsidian), 10.0);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::StonePickaxe), BlockId::Obsidian), 1.0);
    assert!(BlockId::Bookshelf.is_solid());
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::IronAxe), BlockId::Bookshelf), 7.0);

    assert!(!BlockId::Emerald.is_placeable());
    assert!(!BlockId::Emerald.is_solid());
    assert!(BlockId::EmeraldOre.is_placeable());
    assert!(BlockId::EmeraldOre.is_solid());
    assert!(BlockId::EmeraldBlock.is_solid());
    assert_eq!(BlockId::VillagerSpawnEgg.spawn_mob_kind(), Some(crate::gameplay::mob::MobKind::Villager));
    assert_eq!(BlockId::IronGolemSpawnEgg.spawn_mob_kind(), Some(crate::gameplay::mob::MobKind::IronGolem));

    assert!(BlockId::Kelp.is_placeable());
    assert!(BlockId::Seagrass.is_placeable());
    assert!(!BlockId::RawCod.is_placeable());
    assert!(!BlockId::FishingRod.is_placeable());
    assert!(!BlockId::Kelp.is_solid());
    assert!(BlockId::Kelp.is_transparent());
    assert!(BlockId::RawCod.is_fish());
    assert!(BlockId::RawSalmon.is_meat());
    assert!(BlockId::FishingRod.is_tool());
    assert!(BlockId::DriedKelp.is_food());

    assert!(BlockId::WhiteWool.is_wool());
    assert!(BlockId::BlackWool.is_wool());
    assert!(BlockId::WhiteWool.is_solid());
    assert_eq!(BlockId::WhiteWool.break_time(), 0.8);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::Shears), BlockId::WhiteWool), 10.0);
    assert_eq!(BlockId::wool_color_index(BlockId::WhiteWool), Some(0));
    assert_eq!(BlockId::wool_color_index(BlockId::BlackWool), Some(15));
    assert_eq!(BlockId::wool_block_from_color(0), BlockId::WhiteWool);
    assert_eq!(BlockId::wool_block_from_color(15), BlockId::BlackWool);

    assert!(BlockId::Shears.is_tool());
    assert_eq!(BlockId::Shears.attack_damage(), 3.0);
    assert!(!BlockId::Shears.is_solid());

    assert!(BlockId::DiamondPickaxe.is_tool());
    assert_eq!(BlockId::DiamondPickaxe.attack_damage(), 8.0);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::DiamondPickaxe), BlockId::Stone), 10.0);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::GoldPickaxe), BlockId::Stone), 8.0);
    assert_eq!(BlockId::DiamondSword.attack_damage(), 13.0);

    assert!(BlockId::WoodenSlab.is_half_block());
    assert!(BlockId::StoneSlab.is_half_block());
    assert!(BlockId::Bed.is_half_block());
    assert!(BlockId::WoodenStairs.is_stairs());
    assert!(BlockId::StoneStairs.is_stairs());
    assert!(BlockId::Ladder.is_ladder());
    assert!(!BlockId::Ladder.is_solid());

    let registry = BlockRegistry::new();
    assert_eq!(registry.get_face_texture(BlockId::Grass, Face::Top), TextureId::GrassTop);
}

#[test]
fn test_ore_mining_tool_tiers() {
    assert!(!BlockId::Stone.can_harvest_with_tool(None));
    assert!(!BlockId::Cobblestone.can_harvest_with_tool(None));
    assert!(BlockId::Stone.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
    assert!(BlockId::Cobblestone.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
    assert!(
        BlockId::Stone.break_time_with_tool(None)
            > BlockId::Stone.break_time_with_tool(Some(BlockId::WoodenPickaxe))
    );

    assert!(BlockId::CoalOre.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
    assert!(BlockId::CoalOre.can_harvest_with_tool(Some(BlockId::GoldPickaxe)));
    assert!(!BlockId::CoalOre.can_harvest_with_tool(None));

    assert!(!BlockId::IronOre.can_harvest_with_tool(Some(BlockId::WoodenPickaxe)));
    assert!(!BlockId::IronOre.can_harvest_with_tool(Some(BlockId::GoldPickaxe)));
    assert!(BlockId::IronOre.can_harvest_with_tool(Some(BlockId::StonePickaxe)));

    for ore in [BlockId::GoldOre, BlockId::DiamondOre, BlockId::EmeraldOre, BlockId::RustStoneOre] {
        assert!(!ore.can_harvest_with_tool(Some(BlockId::StonePickaxe)));
        assert!(ore.can_harvest_with_tool(Some(BlockId::IronPickaxe)));
        assert!(ore.can_harvest_with_tool(Some(BlockId::DiamondPickaxe)));
    }

    assert!(BlockId::Dirt.can_harvest_with_tool(None));
    assert!(
        BlockId::DiamondOre.break_time_with_tool(Some(BlockId::StonePickaxe))
            > BlockId::DiamondOre.break_time_with_tool(Some(BlockId::IronPickaxe))
    );
}

#[test]
fn test_ruststone_properties() {
    assert!(BlockId::RustStoneOre.is_solid());
    assert!(BlockId::RustStoneBlock.is_solid());
    assert!(BlockId::RustStoneLamp.is_solid());
    assert!(BlockId::RustStoneLampLit.is_solid());

    assert!(!BlockId::RustStone.is_solid());
    assert!(!BlockId::RustStoneWireLit.is_solid());
    assert!(!BlockId::Lever.is_solid());
    assert!(!BlockId::LeverOn.is_solid());
    assert!(!BlockId::Button.is_solid());
    assert!(!BlockId::ButtonPressed.is_solid());
    assert!(!BlockId::PressurePlate.is_solid());
    assert!(!BlockId::PressurePlatePressed.is_solid());

    assert!(BlockId::RustStone.is_targetable());
    assert!(BlockId::RustStoneWireLit.is_targetable());
    assert!(BlockId::Lever.is_targetable());
    assert!(BlockId::Button.is_targetable());
    assert!(BlockId::PressurePlate.is_targetable());

    assert!(BlockId::RustStone.is_transparent());
    assert!(BlockId::RustStoneWireLit.is_transparent());
    assert!(BlockId::Lever.is_transparent());
    assert!(BlockId::Button.is_transparent());
    assert!(BlockId::PressurePlate.is_transparent());

    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::IronPickaxe), BlockId::RustStoneOre), 7.0);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::DiamondPickaxe), BlockId::RustStoneBlock), 10.0);
    assert_eq!(BlockId::mining_speed_multiplier(Some(BlockId::StonePickaxe), BlockId::RustStoneLamp), 4.5);

    assert_eq!(BlockId::RustStoneOre.name(), "ruda rdzawnika");
    assert_eq!(BlockId::RustStone.name(), "rdzawnik");
    assert_eq!(BlockId::RustStoneBlock.name(), "blok rdzawnika");
    assert_eq!(BlockId::RustStoneLamp.name(), "lampa rdzawnikowa");
    assert_eq!(BlockId::Lever.name(), "dzwignia");
    assert_eq!(BlockId::Button.name(), "przycisk");
    assert_eq!(BlockId::PressurePlate.name(), "plytka naciskowa");

    assert_eq!(BlockId::RustStoneOre.name_en(), "Ruststone Ore");
    assert_eq!(BlockId::RustStone.name_en(), "Ruststone");
    assert_eq!(BlockId::RustStoneBlock.name_en(), "Block of Ruststone");
    assert_eq!(BlockId::RustStoneLamp.name_en(), "Ruststone Lamp");
    assert_eq!(BlockId::Grass.name_en(), "Grass Block");
    assert_eq!(BlockId::Dirt.name_en(), "Dirt");
    assert_eq!(BlockId::Stone.name_en(), "Stone");
    assert_eq!(BlockId::DiamondSword.name_en(), "Diamond Sword");

    crate::core::i18n::set_language(crate::core::i18n::Language::English);
    assert_eq!(BlockId::Grass.display_name(), "Grass Block");
    assert_eq!(BlockId::RustStone.display_name(), "Ruststone");

    crate::core::i18n::set_language(crate::core::i18n::Language::Polish);
    assert_eq!(BlockId::Grass.display_name(), "trawa");
    assert_eq!(BlockId::RustStone.display_name(), "rdzawnik");

    crate::core::i18n::set_language(crate::core::i18n::Language::English);
}
