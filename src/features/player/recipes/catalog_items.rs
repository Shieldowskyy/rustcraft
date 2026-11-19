//! Default recipes for basics, cooking, metal/mineral compaction, tiered tools, armor, and weapons.

use crate::features::blocks::BlockId;

use super::types::{Recipe, RecipeBook};

pub(super) fn register_item_and_gear_recipes(book: &mut RecipeBook) {
    book.add(recipe!(Wood => Planks * 4));
    book.add(recipe!(1, 2, [Planks, Planks] => Stick * 4));
    book.add(recipe!(1, 2, [CoalOre, Stick] => Torch * 4));
    book.add(recipe!(2, 2, [
        Planks, Planks,
        Planks, Planks,
    ] => CraftingTable * 1));

    // Cooking over heat / torches
    book.add(recipe!(1, 2, [RawPorkchop, CoalOre] => CookedPorkchop * 1));
    book.add(recipe!(1, 2, [RawBeef, CoalOre] => CookedBeef * 1));
    book.add(recipe!(1, 2, [RawChicken, CoalOre] => CookedChicken * 1));
    book.add(recipe!(1, 2, [RawCod, CoalOre] => CookedCod * 1));
    book.add(recipe!(1, 2, [RawSalmon, CoalOre] => CookedSalmon * 1));
    book.add(recipe!(1, 2, [Kelp, CoalOre] => DriedKelp * 1));
    book.add(recipe!(1, 2, [RawPorkchop, Torch] => CookedPorkchop * 1));
    book.add(recipe!(1, 2, [RawBeef, Torch] => CookedBeef * 1));
    book.add(recipe!(1, 2, [RawChicken, Torch] => CookedChicken * 1));
    book.add(recipe!(1, 2, [RawCod, Torch] => CookedCod * 1));
    book.add(recipe!(1, 2, [RawSalmon, Torch] => CookedSalmon * 1));
    book.add(recipe!(1, 2, [Kelp, Torch] => DriedKelp * 1));

    // Storage & processing containers
    book.add(recipe!(3, 3, [
        Planks, Planks, Planks,
        Planks, _,      Planks,
        Planks, Planks, Planks,
    ] => Chest * 1));
    book.add(recipe!(3, 3, [
        Stone, Stone, Stone,
        Stone, _,     Stone,
        Stone, Stone, Stone,
    ] => Furnace * 1));
    book.add(recipe!(3, 3, [
        Cobblestone, Cobblestone, Cobblestone,
        Cobblestone, _,           Cobblestone,
        Cobblestone, Cobblestone, Cobblestone,
    ] => Furnace * 1));

    // Ounce <-> Ingot & Ingot/Gem <-> Block conversion
    book.add(recipe!(3, 3, [
        IronOunce, IronOunce, IronOunce,
        IronOunce, _,         IronOunce,
        IronOunce, IronOunce, IronOunce,
    ] => IronIngot * 1));
    book.add(recipe!(IronIngot => IronOunce * 8));

    book.add(recipe!(3, 3, [
        GoldOunce, GoldOunce, GoldOunce,
        GoldOunce, _,         GoldOunce,
        GoldOunce, GoldOunce, GoldOunce,
    ] => GoldIngot * 1));
    book.add(recipe!(GoldIngot => GoldOunce * 8));

    book.add(recipe!(3, 3, [
        IronIngot, IronIngot, IronIngot,
        IronIngot, IronIngot, IronIngot,
        IronIngot, IronIngot, IronIngot,
    ] => IronBlock * 1));
    book.add(recipe!(IronBlock => IronIngot * 9));

    book.add(recipe!(3, 3, [
        GoldIngot, GoldIngot, GoldIngot,
        GoldIngot, GoldIngot, GoldIngot,
        GoldIngot, GoldIngot, GoldIngot,
    ] => GoldBlock * 1));
    book.add(recipe!(GoldBlock => GoldIngot * 9));

    book.add(recipe!(3, 3, [
        CoalOre, CoalOre, CoalOre,
        CoalOre, CoalOre, CoalOre,
        CoalOre, CoalOre, CoalOre,
    ] => CoalBlock * 1));
    book.add(recipe!(CoalBlock => CoalOre * 9));

    book.add(recipe!(3, 3, [
        Diamond, Diamond, Diamond,
        Diamond, Diamond, Diamond,
        Diamond, Diamond, Diamond,
    ] => DiamondBlock * 1));
    book.add(recipe!(DiamondBlock => Diamond * 9));

    book.add(recipe!(3, 3, [
        Emerald, Emerald, Emerald,
        Emerald, Emerald, Emerald,
        Emerald, Emerald, Emerald,
    ] => EmeraldBlock * 1));
    book.add(recipe!(EmeraldBlock => Emerald * 9));

    register_tools(book);
    register_armor(book);
    register_ranged_and_food(book);
}

fn register_tools(book: &mut RecipeBook) {
    let tool_materials = [
        (BlockId::Planks, BlockId::WoodenShovel, BlockId::WoodenSword, BlockId::WoodenPickaxe, BlockId::WoodenAxe, BlockId::WoodenHoe),
        (BlockId::Stone, BlockId::StoneShovel, BlockId::StoneSword, BlockId::StonePickaxe, BlockId::StoneAxe, BlockId::StoneHoe),
        (BlockId::Cobblestone, BlockId::StoneShovel, BlockId::StoneSword, BlockId::StonePickaxe, BlockId::StoneAxe, BlockId::StoneHoe),
        (BlockId::IronIngot, BlockId::IronShovel, BlockId::IronSword, BlockId::IronPickaxe, BlockId::IronAxe, BlockId::IronHoe),
        (BlockId::GoldIngot, BlockId::GoldShovel, BlockId::GoldSword, BlockId::GoldPickaxe, BlockId::GoldAxe, BlockId::GoldHoe),
        (BlockId::Diamond, BlockId::DiamondShovel, BlockId::DiamondSword, BlockId::DiamondPickaxe, BlockId::DiamondAxe, BlockId::DiamondHoe),
    ];

    for (mat, shovel, sword, pick, axe, hoe) in tool_materials {
        book.add(Recipe::new(1, 3, vec![Some(mat), Some(BlockId::Stick), Some(BlockId::Stick)], shovel, 1));
        book.add(Recipe::new(1, 3, vec![Some(mat), Some(mat), Some(BlockId::Stick)], sword, 1));
        book.add(Recipe::new(
            3, 3,
            vec![Some(mat), Some(mat), Some(mat), None, Some(BlockId::Stick), None, None, Some(BlockId::Stick), None],
            pick, 1,
        ));
        book.add(Recipe::new(
            2, 3,
            vec![Some(mat), Some(mat), Some(mat), Some(BlockId::Stick), None, Some(BlockId::Stick)],
            axe, 1,
        ));
        book.add(Recipe::new(
            2, 3,
            vec![Some(mat), Some(mat), Some(BlockId::Stick), Some(mat), Some(BlockId::Stick), None],
            axe, 1,
        ));
        book.add(Recipe::new(
            2, 3,
            vec![Some(mat), Some(mat), None, Some(BlockId::Stick), None, Some(BlockId::Stick)],
            hoe, 1,
        ));
        book.add(Recipe::new(
            2, 3,
            vec![Some(mat), Some(mat), Some(BlockId::Stick), None, Some(BlockId::Stick), None],
            hoe, 1,
        ));
    }
}

fn register_armor(book: &mut RecipeBook) {
    let armor_materials = [
        (BlockId::Leather, BlockId::LeatherHelmet, BlockId::LeatherChestplate, BlockId::LeatherLeggings, BlockId::LeatherBoots),
        (BlockId::IronIngot, BlockId::IronHelmet, BlockId::IronChestplate, BlockId::IronLeggings, BlockId::IronBoots),
        (BlockId::GoldIngot, BlockId::GoldHelmet, BlockId::GoldChestplate, BlockId::GoldLeggings, BlockId::GoldBoots),
        (BlockId::Diamond, BlockId::DiamondHelmet, BlockId::DiamondChestplate, BlockId::DiamondLeggings, BlockId::DiamondBoots),
    ];

    for (mat, helm, chest, legs, boots) in armor_materials {
        book.add(Recipe::new(
            3, 2,
            vec![Some(mat), Some(mat), Some(mat), Some(mat), None, Some(mat)],
            helm, 1,
        ));
        book.add(Recipe::new(
            3, 3,
            vec![Some(mat), None, Some(mat), Some(mat), Some(mat), Some(mat), Some(mat), Some(mat), Some(mat)],
            chest, 1,
        ));
        book.add(Recipe::new(
            3, 3,
            vec![Some(mat), Some(mat), Some(mat), Some(mat), None, Some(mat), Some(mat), None, Some(mat)],
            legs, 1,
        ));
        book.add(Recipe::new(
            3, 2,
            vec![Some(mat), None, Some(mat), Some(mat), None, Some(mat)],
            boots, 1,
        ));
        book.add(Recipe::new(
            2, 2,
            vec![Some(mat), Some(mat), Some(mat), Some(mat)],
            boots, 1,
        ));
    }
}

fn register_ranged_and_food(book: &mut RecipeBook) {
    book.add(Recipe::new(
        3, 3,
        vec![
            None, Some(BlockId::Stick), Some(BlockId::StringItem),
            Some(BlockId::Stick), None, Some(BlockId::StringItem),
            None, Some(BlockId::Stick), Some(BlockId::StringItem),
        ],
        BlockId::Bow, 1,
    ));
    book.add(Recipe::new(
        3, 3,
        vec![
            Some(BlockId::StringItem), Some(BlockId::Stick), None,
            Some(BlockId::StringItem), None, Some(BlockId::Stick),
            Some(BlockId::StringItem), Some(BlockId::Stick), None,
        ],
        BlockId::Bow, 1,
    ));
    book.add(Recipe::new(
        3, 3,
        vec![
            None, Some(BlockId::Stick), Some(BlockId::WhiteWool),
            Some(BlockId::Stick), None, Some(BlockId::WhiteWool),
            None, Some(BlockId::Stick), Some(BlockId::WhiteWool),
        ],
        BlockId::Bow, 1,
    ));
    book.add(Recipe::new(
        3, 3,
        vec![
            None, Some(BlockId::Stick), Some(BlockId::Feather),
            Some(BlockId::Stick), None, Some(BlockId::Feather),
            None, Some(BlockId::Stick), Some(BlockId::Feather),
        ],
        BlockId::Bow, 1,
    ));
    book.add(Recipe::new(
        3, 3,
        vec![
            Some(BlockId::WhiteWool), Some(BlockId::Stick), None,
            Some(BlockId::WhiteWool), None, Some(BlockId::Stick),
            Some(BlockId::WhiteWool), Some(BlockId::Stick), None,
        ],
        BlockId::Bow, 1,
    ));

    book.add(Recipe::new(
        1, 3,
        vec![Some(BlockId::Flint), Some(BlockId::Stick), Some(BlockId::Feather)],
        BlockId::Arrow, 4,
    ));
    book.add(Recipe::new(
        3, 3,
        vec![
            Some(BlockId::Planks), Some(BlockId::IronIngot), Some(BlockId::Planks),
            Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            None, Some(BlockId::Planks), None,
        ],
        BlockId::Shield, 1,
    ));

    book.add(recipe!(3, 1, [Wheat, Wheat, Wheat] => Bread * 1));
    book.add(recipe!(Bone => BoneMeal * 3));
}
