use crate::gameplay::inventory::ItemStack;
use crate::world::block::BlockId;

#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<Option<BlockId>>,
    pub result: ItemStack,
}

#[macro_export]
macro_rules! recipe_slot {
    (_) => {
        None
    };
    ($item:ident) => {
        Some($crate::world::block::BlockId::$item)
    };
}

/// Declarative macro for defining crafting recipes concisely.
///
/// # Examples
/// ```ignore
/// // 1x1 crafting:
/// recipe!(Wood => Planks * 4);
///
/// // Shaped 2D crafting grid:
/// recipe!(2x2: [
///     Planks, Planks,
///     Planks, Planks,
/// ] => CraftingTable * 1);
///
/// // Grid with empty slots (_):
/// recipe!(3x3: [
///     Planks, Planks, Planks,
///     Planks, _,      Planks,
///     Planks, Planks, Planks,
/// ] => Chest * 1);
/// ```
#[macro_export]
macro_rules! recipe {
    // 1x1 shorthand: recipe!(Wood => Planks * 4)
    ($input:ident => $result:ident * $count:expr) => {
        $crate::gameplay::recipe::Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some($crate::world::block::BlockId::$input)],
            result: $crate::gameplay::inventory::ItemStack {
                block: $crate::world::block::BlockId::$result,
                count: $count,
            },
        }
    };

    // Shaped 2D grid: recipe!(3, 3, [A, B, C, ...] => Result * Count)
    ($w:expr, $h:expr, [ $($item:tt),* $(,)? ] => $result:ident * $count:expr) => {
        $crate::gameplay::recipe::Recipe {
            width: $w,
            height: $h,
            pattern: vec![ $( $crate::recipe_slot!($item) ),* ],
            result: $crate::gameplay::inventory::ItemStack {
                block: $crate::world::block::BlockId::$result,
                count: $count,
            },
        }
    };
}

impl Recipe {
    pub fn new(width: usize, height: usize, pattern: Vec<Option<BlockId>>, result: BlockId, count: u8) -> Self {
        Self {
            width,
            height,
            pattern,
            result: ItemStack { block: result, count },
        }
    }
}

pub struct RecipeBook {
    recipes: Vec<Recipe>,
}

impl RecipeBook {
    pub fn new() -> Self {
        let mut book = Self { recipes: Vec::new() };
        book.register_default_recipes();
        book
    }

    /// Add a recipe to the recipe book if not already present.
    pub fn add(&mut self, recipe: Recipe) {
        if !self.recipes.contains(&recipe) {
            self.recipes.push(recipe);
        }
    }

    /// Read-only access to all registered recipes.
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    fn register_default_recipes(&mut self) {
        // --- 1x1 Basic Crafting ---
        self.add(recipe!(Wood => Planks * 4));

        // --- 1x2 Vertical Recipes ---
        self.add(recipe!(1, 2, [Planks, Planks] => Stick * 4));
        self.add(recipe!(1, 2, [CoalOre, Stick] => Torch * 4));

        // --- 2x2 Crafting Grid ---
        self.add(recipe!(2, 2, [
            Planks, Planks,
            Planks, Planks,
        ] => CraftingTable * 1));

        // --- Cooking over Heat / Torches ---
        self.add(recipe!(1, 2, [RawPorkchop, CoalOre] => CookedPorkchop * 1));
        self.add(recipe!(1, 2, [RawBeef, CoalOre] => CookedBeef * 1));
        self.add(recipe!(1, 2, [RawChicken, CoalOre] => CookedChicken * 1));
        self.add(recipe!(1, 2, [RawCod, CoalOre] => CookedCod * 1));
        self.add(recipe!(1, 2, [RawSalmon, CoalOre] => CookedSalmon * 1));
        self.add(recipe!(1, 2, [Kelp, CoalOre] => DriedKelp * 1));
        self.add(recipe!(1, 2, [RawPorkchop, Torch] => CookedPorkchop * 1));
        self.add(recipe!(1, 2, [RawBeef, Torch] => CookedBeef * 1));
        self.add(recipe!(1, 2, [RawChicken, Torch] => CookedChicken * 1));
        self.add(recipe!(1, 2, [RawCod, Torch] => CookedCod * 1));
        self.add(recipe!(1, 2, [RawSalmon, Torch] => CookedSalmon * 1));
        self.add(recipe!(1, 2, [Kelp, Torch] => DriedKelp * 1));

        // --- 3x3 Hollow Rings (Storage & Processing) ---
        self.add(recipe!(3, 3, [
            Planks, Planks, Planks,
            Planks, _,      Planks,
            Planks, Planks, Planks,
        ] => Chest * 1));

        self.add(recipe!(3, 3, [
            Stone, Stone, Stone,
            Stone, _,     Stone,
            Stone, Stone, Stone,
        ] => Furnace * 1));

        self.add(recipe!(3, 3, [
            Cobblestone, Cobblestone, Cobblestone,
            Cobblestone, _,           Cobblestone,
            Cobblestone, Cobblestone, Cobblestone,
        ] => Furnace * 1));

        self.add(recipe!(3, 3, [
            IronOunce, IronOunce, IronOunce,
            IronOunce, _,         IronOunce,
            IronOunce, IronOunce, IronOunce,
        ] => IronIngot * 1));
        self.add(recipe!(IronIngot => IronOunce * 8));

        self.add(recipe!(3, 3, [
            GoldOunce, GoldOunce, GoldOunce,
            GoldOunce, _,         GoldOunce,
            GoldOunce, GoldOunce, GoldOunce,
        ] => GoldIngot * 1));
        self.add(recipe!(GoldIngot => GoldOunce * 8));

        // --- 3x3 Solid Metal/Mineral Blocks (and reverse decrafting) ---
        self.add(recipe!(3, 3, [
            IronIngot, IronIngot, IronIngot,
            IronIngot, IronIngot, IronIngot,
            IronIngot, IronIngot, IronIngot,
        ] => IronBlock * 1));
        self.add(recipe!(IronBlock => IronIngot * 9));

        self.add(recipe!(3, 3, [
            GoldIngot, GoldIngot, GoldIngot,
            GoldIngot, GoldIngot, GoldIngot,
            GoldIngot, GoldIngot, GoldIngot,
        ] => GoldBlock * 1));
        self.add(recipe!(GoldBlock => GoldIngot * 9));

        // --- 3x3 Solid Blocks ---
        self.add(recipe!(3, 3, [
            CoalOre, CoalOre, CoalOre,
            CoalOre, CoalOre, CoalOre,
            CoalOre, CoalOre, CoalOre,
        ] => CoalBlock * 1));
        self.add(recipe!(CoalBlock => CoalOre * 9));

        self.add(recipe!(3, 3, [
            Diamond, Diamond, Diamond,
            Diamond, Diamond, Diamond,
            Diamond, Diamond, Diamond,
        ] => DiamondBlock * 1));
        self.add(recipe!(DiamondBlock => Diamond * 9));

        self.add(recipe!(3, 3, [
            Emerald, Emerald, Emerald,
            Emerald, Emerald, Emerald,
            Emerald, Emerald, Emerald,
        ] => EmeraldBlock * 1));
        self.add(recipe!(EmeraldBlock => Emerald * 9));

        // --- Tools (Tiered Material Loops) ---
        let tool_materials = [
            (BlockId::Planks, BlockId::WoodenShovel, BlockId::WoodenSword, BlockId::WoodenPickaxe, BlockId::WoodenAxe, BlockId::WoodenHoe),
            (BlockId::Stone, BlockId::StoneShovel, BlockId::StoneSword, BlockId::StonePickaxe, BlockId::StoneAxe, BlockId::StoneHoe),
            (BlockId::Cobblestone, BlockId::StoneShovel, BlockId::StoneSword, BlockId::StonePickaxe, BlockId::StoneAxe, BlockId::StoneHoe),
            (BlockId::IronIngot, BlockId::IronShovel, BlockId::IronSword, BlockId::IronPickaxe, BlockId::IronAxe, BlockId::IronHoe),
            (BlockId::GoldIngot, BlockId::GoldShovel, BlockId::GoldSword, BlockId::GoldPickaxe, BlockId::GoldAxe, BlockId::GoldHoe),
            (BlockId::Diamond, BlockId::DiamondShovel, BlockId::DiamondSword, BlockId::DiamondPickaxe, BlockId::DiamondAxe, BlockId::DiamondHoe),
        ];

        for (mat, shovel, sword, pick, axe, hoe) in tool_materials {
            // Shovels (1x3: Material over 2 Sticks)
            self.add(Recipe::new(
                1, 3,
                vec![Some(mat), Some(BlockId::Stick), Some(BlockId::Stick)],
                shovel, 1,
            ));

            // Swords (1x3: 2 Materials over Stick)
            self.add(Recipe::new(
                1, 3,
                vec![Some(mat), Some(mat), Some(BlockId::Stick)],
                sword, 1,
            ));

            // Pickaxes (3x3: 3 Materials on top, 2 Sticks below center)
            self.add(Recipe::new(
                3, 3,
                vec![
                    Some(mat), Some(mat), Some(mat),
                    None, Some(BlockId::Stick), None,
                    None, Some(BlockId::Stick), None,
                ],
                pick, 1,
            ));

            // Axes (2x3: 3 Materials corner, 2 Sticks) - Right-facing
            self.add(Recipe::new(
                2, 3,
                vec![
                    Some(mat), Some(mat),
                    Some(mat), Some(BlockId::Stick),
                    None, Some(BlockId::Stick),
                ],
                axe, 1,
            ));

            // Axes (2x3: 3 Materials corner, 2 Sticks) - Left-facing
            self.add(Recipe::new(
                2, 3,
                vec![
                    Some(mat), Some(mat),
                    Some(BlockId::Stick), Some(mat),
                    Some(BlockId::Stick), None,
                ],
                axe, 1,
            ));

            // Hoes (2x3: 2 Materials on top, 2 Sticks below) - Right-facing
            self.add(Recipe::new(
                2, 3,
                vec![
                    Some(mat), Some(mat),
                    None, Some(BlockId::Stick),
                    None, Some(BlockId::Stick),
                ],
                hoe, 1,
            ));

            // Hoes (2x3: 2 Materials on top, 2 Sticks below) - Left-facing
            self.add(Recipe::new(
                2, 3,
                vec![
                    Some(mat), Some(mat),
                    Some(BlockId::Stick), None,
                    Some(BlockId::Stick), None,
                ],
                hoe, 1,
            ));
        }

        // --- Armor Recipes (Leather, Iron, Gold, Diamond) ---
        let armor_materials = [
            (BlockId::Leather, BlockId::LeatherHelmet, BlockId::LeatherChestplate, BlockId::LeatherLeggings, BlockId::LeatherBoots),
            (BlockId::IronIngot, BlockId::IronHelmet, BlockId::IronChestplate, BlockId::IronLeggings, BlockId::IronBoots),
            (BlockId::GoldIngot, BlockId::GoldHelmet, BlockId::GoldChestplate, BlockId::GoldLeggings, BlockId::GoldBoots),
            (BlockId::Diamond, BlockId::DiamondHelmet, BlockId::DiamondChestplate, BlockId::DiamondLeggings, BlockId::DiamondBoots),
        ];

        for (mat, helm, chest, legs, boots) in armor_materials {
            // Helmet (3x2: 3 on top row, 2 on sides of second row)
            self.add(Recipe::new(
                3, 2,
                vec![
                    Some(mat), Some(mat), Some(mat),
                    Some(mat), None,      Some(mat),
                ],
                helm, 1,
            ));

            // Chestplate (3x3: 8 materials leaving top center empty)
            self.add(Recipe::new(
                3, 3,
                vec![
                    Some(mat), None,      Some(mat),
                    Some(mat), Some(mat), Some(mat),
                    Some(mat), Some(mat), Some(mat),
                ],
                chest, 1,
            ));

            // Leggings (3x3: pants shape)
            self.add(Recipe::new(
                3, 3,
                vec![
                    Some(mat), Some(mat), Some(mat),
                    Some(mat), None,      Some(mat),
                    Some(mat), None,      Some(mat),
                ],
                legs, 1,
            ));

            // Boots (3x2: 2 on sides of two rows)
            self.add(Recipe::new(
                3, 2,
                vec![
                    Some(mat), None, Some(mat),
                    Some(mat), None, Some(mat),
                ],
                boots, 1,
            ));
            // Boots (2x2 compact variant)
            self.add(Recipe::new(
                2, 2,
                vec![
                    Some(mat), Some(mat),
                    Some(mat), Some(mat),
                ],
                boots, 1,
            ));
        }

        // --- Ranged & Defensive Equipment (Bow, Arrow, Shield) ---
        // Bow (3x3: 3 sticks and 3 wool / feathers) - Right-facing
        self.add(Recipe::new(
            3, 3,
            vec![
                None,                 Some(BlockId::Stick), Some(BlockId::WhiteWool),
                Some(BlockId::Stick), None,                 Some(BlockId::WhiteWool),
                None,                 Some(BlockId::Stick), Some(BlockId::WhiteWool),
            ],
            BlockId::Bow, 1,
        ));
        // Bow (3x3 with feathers)
        self.add(Recipe::new(
            3, 3,
            vec![
                None,                 Some(BlockId::Stick), Some(BlockId::Feather),
                Some(BlockId::Stick), None,                 Some(BlockId::Feather),
                None,                 Some(BlockId::Stick), Some(BlockId::Feather),
            ],
            BlockId::Bow, 1,
        ));
        // Bow - Left-facing
        self.add(Recipe::new(
            3, 3,
            vec![
                Some(BlockId::WhiteWool), Some(BlockId::Stick), None,
                Some(BlockId::WhiteWool), None,                 Some(BlockId::Stick),
                Some(BlockId::WhiteWool), Some(BlockId::Stick), None,
            ],
            BlockId::Bow, 1,
        ));

        // Arrow (1x3: Flint top, Stick middle, Feather bottom -> 4 Arrows)
        self.add(Recipe::new(
            1, 3,
            vec![
                Some(BlockId::Flint),
                Some(BlockId::Stick),
                Some(BlockId::Feather),
            ],
            BlockId::Arrow, 4,
        ));

        // Shield (3x3: Planks around Iron Ingot)
        self.add(Recipe::new(
            3, 3,
            vec![
                Some(BlockId::Planks), Some(BlockId::IronIngot), Some(BlockId::Planks),
                Some(BlockId::Planks), Some(BlockId::Planks),    Some(BlockId::Planks),
                None,                  Some(BlockId::Planks),    None,
            ],
            BlockId::Shield, 1,
        ));

        // --- Farming & Food Recipes ---
        // Bread (3x1: 3 Wheat)
        self.add(recipe!(3, 1, [
            Wheat, Wheat, Wheat,
        ] => Bread * 1));

        // Bone Meal (1x1: 1 Bone -> 3 BoneMeal)
        self.add(recipe!(Bone => BoneMeal * 3));

        // --- Architectural & Decorative Blocks ---
        // Shears (2x2: diagonal Iron Ingots)
        self.add(recipe!(2, 2, [
            _,         IronIngot,
            IronIngot, _,
        ] => Shears * 1));
        self.add(recipe!(2, 2, [
            IronIngot, _,
            _,         IronIngot,
        ] => Shears * 1));

        // Fishing Rod (3x3: 3 Sticks diagonally + 2 Feathers or Wool vertically)
        self.add(recipe!(3, 3, [
            _,     _,     Stick,
            _,     Stick, Feather,
            Stick, _,     Feather,
        ] => FishingRod * 1));
        self.add(recipe!(3, 3, [
            _,     _,     Stick,
            _,     Stick, WhiteWool,
            Stick, _,     WhiteWool,
        ] => FishingRod * 1));
        self.add(recipe!(2, 2, [
            Stick, Feather,
            Stick, Feather,
        ] => FishingRod * 1));
        self.add(recipe!(2, 2, [
            Stick, WhiteWool,
            Stick, WhiteWool,
        ] => FishingRod * 1));

        // Beds (3x2: 3 Wool, Feathers or Leaves over 3 Planks)
        for &wool in &crate::world::block::BlockId::WOOL_BLOCKS {
            self.add(Recipe::new(
                3, 2,
                vec![
                    Some(wool), Some(wool), Some(wool),
                    Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
                ],
                BlockId::Bed, 1,
            ));
        }
        self.add(recipe!(3, 2, [
            Feather, Feather, Feather,
            Planks,  Planks,  Planks,
        ] => Bed * 1));
        self.add(recipe!(3, 2, [
            Leaves, Leaves, Leaves,
            Planks, Planks, Planks,
        ] => Bed * 1));

        // Feathers to White Wool (2x2: 4 Feathers -> 1 White Wool)
        self.add(recipe!(2, 2, [
            Feather, Feather,
            Feather, Feather,
        ] => WhiteWool * 1));

        // Wool Dyeing Recipes (1x2 / 2x1)
        self.add(recipe!(1, 2, [Apple, WhiteWool] => RedWool * 1));
        self.add(recipe!(2, 1, [Apple, WhiteWool] => RedWool * 1));
        self.add(recipe!(1, 2, [Leaves, WhiteWool] => GreenWool * 1));
        self.add(recipe!(2, 1, [Leaves, WhiteWool] => GreenWool * 1));
        self.add(recipe!(1, 2, [Sapling, WhiteWool] => LimeWool * 1));
        self.add(recipe!(2, 1, [Sapling, WhiteWool] => LimeWool * 1));
        self.add(recipe!(1, 2, [CoalOre, WhiteWool] => BlackWool * 1));
        self.add(recipe!(2, 1, [CoalOre, WhiteWool] => BlackWool * 1));
        self.add(recipe!(1, 2, [Gunpowder, WhiteWool] => GrayWool * 1));
        self.add(recipe!(2, 1, [Gunpowder, WhiteWool] => GrayWool * 1));
        self.add(recipe!(1, 2, [BoneMeal, GrayWool] => LightGrayWool * 1));
        self.add(recipe!(2, 1, [BoneMeal, GrayWool] => LightGrayWool * 1));
        self.add(recipe!(1, 2, [Carrot, WhiteWool] => OrangeWool * 1));
        self.add(recipe!(2, 1, [Carrot, WhiteWool] => OrangeWool * 1));
        self.add(recipe!(1, 2, [Wheat, WhiteWool] => YellowWool * 1));
        self.add(recipe!(2, 1, [Wheat, WhiteWool] => YellowWool * 1));
        self.add(recipe!(1, 2, [BoneMeal, RedWool] => PinkWool * 1));
        self.add(recipe!(2, 1, [BoneMeal, RedWool] => PinkWool * 1));
        self.add(recipe!(1, 2, [Diamond, WhiteWool] => LightBlueWool * 1));
        self.add(recipe!(2, 1, [Diamond, WhiteWool] => LightBlueWool * 1));
        self.add(recipe!(1, 2, [Water, BlueWool] => CyanWool * 1));
        self.add(recipe!(2, 1, [Water, BlueWool] => CyanWool * 1));
        self.add(recipe!(1, 2, [RedWool, BlueWool] => PurpleWool * 1));
        self.add(recipe!(2, 1, [RedWool, BlueWool] => PurpleWool * 1));
        self.add(recipe!(1, 2, [Dirt, WhiteWool] => BrownWool * 1));
        self.add(recipe!(2, 1, [Dirt, WhiteWool] => BrownWool * 1));

        // Slabs (3x1: 3 Planks/Stone/Bricks/Cobblestone/Sandstone horizontal -> 6 Slabs)
        self.add(recipe!(3, 1, [Planks, Planks, Planks] => WoodenSlab * 6));
        self.add(recipe!(3, 1, [Stone, Stone, Stone] => StoneSlab * 6));
        self.add(recipe!(3, 1, [Bricks, Bricks, Bricks] => BrickSlab * 6));
        self.add(recipe!(3, 1, [Cobblestone, Cobblestone, Cobblestone] => CobblestoneSlab * 6));
        self.add(recipe!(3, 1, [Sandstone, Sandstone, Sandstone] => SandstoneSlab * 6));

        // Stairs (3x3 stair shape -> 4 Stairs, both orientations)
        self.add(recipe!(3, 3, [
            Planks, _,      _,
            Planks, Planks, _,
            Planks, Planks, Planks,
        ] => WoodenStairs * 4));
        self.add(recipe!(3, 3, [
            _,      _,      Planks,
            _,      Planks, Planks,
            Planks, Planks, Planks,
        ] => WoodenStairs * 4));

        self.add(recipe!(3, 3, [
            Stone, _,     _,
            Stone, Stone, _,
            Stone, Stone, Stone,
        ] => StoneStairs * 4));
        self.add(recipe!(3, 3, [
            _,     _,     Stone,
            _,     Stone, Stone,
            Stone, Stone, Stone,
        ] => StoneStairs * 4));

        self.add(recipe!(3, 3, [
            Bricks, _,      _,
            Bricks, Bricks, _,
            Bricks, Bricks, Bricks,
        ] => BrickStairs * 4));
        self.add(recipe!(3, 3, [
            _,      _,      Bricks,
            _,      Bricks, Bricks,
            Bricks, Bricks, Bricks,
        ] => BrickStairs * 4));

        self.add(recipe!(3, 3, [
            Cobblestone, _,           _,
            Cobblestone, Cobblestone, _,
            Cobblestone, Cobblestone, Cobblestone,
        ] => CobblestoneStairs * 4));
        self.add(recipe!(3, 3, [
            _,           _,           Cobblestone,
            _,           Cobblestone, Cobblestone,
            Cobblestone, Cobblestone, Cobblestone,
        ] => CobblestoneStairs * 4));

        self.add(recipe!(3, 3, [
            Sandstone, _,         _,
            Sandstone, Sandstone, _,
            Sandstone, Sandstone, Sandstone,
        ] => SandstoneStairs * 4));
        self.add(recipe!(3, 3, [
            _,         _,         Sandstone,
            _,         Sandstone, Sandstone,
            Sandstone, Sandstone, Sandstone,
        ] => SandstoneStairs * 4));

        // --- Clay, Bricks, Sandstone, Bookshelf Crafting ---
        self.add(recipe!(2, 2, [
            ClayBall, ClayBall,
            ClayBall, ClayBall,
        ] => Clay * 1));
        self.add(recipe!(Clay => ClayBall * 4));

        self.add(recipe!(2, 2, [
            Brick, Brick,
            Brick, Brick,
        ] => Bricks * 1));

        self.add(recipe!(2, 2, [
            Sand, Sand,
            Sand, Sand,
        ] => Sandstone * 1));

        self.add(recipe!(3, 3, [
            Planks, Planks, Planks,
            Wheat,  Wheat,  Wheat,
            Planks, Planks, Planks,
        ] => Bookshelf * 1));

        self.add(recipe!(1, 2, [Cobblestone, Leaves] => MossyCobblestone * 1));
        self.add(recipe!(2, 1, [Cobblestone, Leaves] => MossyCobblestone * 1));

        // Ladder (3x3: H-shape of Sticks -> 3 Ladders)
        self.add(recipe!(3, 3, [
            Stick, _,     Stick,
            Stick, Stick, Stick,
            Stick, _,     Stick,
        ] => Ladder * 3));

        // Doors (2x3: 6 Planks -> 1 Door)
        self.add(recipe!(2, 3, [
            Planks, Planks,
            Planks, Planks,
            Planks, Planks,
        ] => Door * 1));

        // TNT (2x2: Sand + Dirt or Gunpowder + Sand)
        self.add(recipe!(2, 2, [
            Sand, Dirt,
            Dirt, Sand,
        ] => Tnt * 1));
        self.add(recipe!(2, 2, [
            Gunpowder, Sand,
            Sand, Gunpowder,
        ] => Tnt * 1));

        // --- RustStone Mechanics ---
        // RustStone Block (3x3: 9 RustStone -> 1 RustStoneBlock)
        self.add(recipe!(3, 3, [
            RustStone, RustStone, RustStone,
            RustStone, RustStone, RustStone,
            RustStone, RustStone, RustStone,
        ] => RustStoneBlock * 1));

        // RustStone Block reverse (1x1: 1 RustStoneBlock -> 9 RustStone)
        self.add(recipe!(RustStoneBlock => RustStone * 9));

        // RustStone Lamp (3x3: Glass or Torch surrounded by RustStone)
        self.add(recipe!(3, 3, [
            _,         RustStone, _,
            RustStone, Glass,     RustStone,
            _,         RustStone, _,
        ] => RustStoneLamp * 1));
        self.add(recipe!(3, 3, [
            Stone,     RustStone, Stone,
            RustStone, Torch,     RustStone,
            Stone,     RustStone, Stone,
        ] => RustStoneLamp * 1));

        // Iron Golem (3x3: T-shape of 4 IronBlocks -> 1 IronGolemSpawnEgg)
        self.add(recipe!(3, 3, [
            IronBlock, IronBlock, IronBlock,
            _,         IronBlock, _,
            _,         _,         _,
        ] => IronGolemSpawnEgg * 1));
        self.add(recipe!(3, 3, [
            _,         _,         _,
            IronBlock, IronBlock, IronBlock,
            _,         IronBlock, _,
        ] => IronGolemSpawnEgg * 1));

        // Lever (1x2: Stick over Stone or Planks or Cobblestone)
        self.add(recipe!(1, 2, [
            Stick,
            Stone,
        ] => Lever * 1));
        self.add(recipe!(1, 2, [
            Stick,
            Cobblestone,
        ] => Lever * 1));
        self.add(recipe!(1, 2, [
            Stick,
            Planks,
        ] => Lever * 1));

        // Button (1x1: Stone or Planks or Cobblestone)
        self.add(recipe!(Stone => Button * 1));
        self.add(recipe!(Cobblestone => Button * 1));
        self.add(recipe!(Planks => Button * 1));

        // Pressure Plate (2x1: 2 Stone or 2 Planks or 2 Cobblestone)
        self.add(recipe!(2, 1, [
            Stone, Stone,
        ] => PressurePlate * 1));
        self.add(recipe!(2, 1, [
            Cobblestone, Cobblestone,
        ] => PressurePlate * 1));
        self.add(recipe!(2, 1, [
            Planks, Planks,
        ] => PressurePlate * 1));
    }

    pub fn match_grid(&self, grid: &[Option<ItemStack>], grid_w: usize, grid_h: usize) -> Option<ItemStack> {
        let mut min_r = None;
        let mut max_r = 0;
        let mut min_c = None;
        let mut max_c = 0;

        for r in 0..grid_h {
            for c in 0..grid_w {
                let idx = r * grid_w + c;
                if grid[idx].is_some() {
                    if min_r.is_none() { min_r = Some(r); }
                    max_r = r;
                    if min_c.is_none() || c < min_c.unwrap() { min_c = Some(c); }
                    if c > max_c { max_c = c; }
                }
            }
        }

        let (min_r, min_c) = match (min_r, min_c) {
            (Some(r), Some(c)) => (r, c),
            _ => return None,
        };

        let sub_w = max_c - min_c + 1;
        let sub_h = max_r - min_r + 1;

        let mut sub_pattern = Vec::with_capacity(sub_w * sub_h);
        for r in min_r..=max_r {
            for c in min_c..=max_c {
                let idx = r * grid_w + c;
                sub_pattern.push(grid[idx].map(|s| s.block));
            }
        }

        for recipe in &self.recipes {
            if recipe.width == sub_w && recipe.height == sub_h && recipe.pattern == sub_pattern {
                return Some(recipe.result);
            }
        }

        None
    }

    pub fn consume_grid(grid: &mut [Option<ItemStack>]) {
        for slot in grid.iter_mut() {
            if let Some(stack) = slot {
                if stack.count > 1 {
                    stack.count -= 1;
                } else {
                    *slot = None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planks_crafting_2x2() {
        let book = RecipeBook::new();
        let mut grid = [None; 4];
        grid[0] = Some(ItemStack { block: BlockId::Wood, count: 1 });
        let result = book.match_grid(&grid, 2, 2);
        assert_eq!(result, Some(ItemStack { block: BlockId::Planks, count: 4 }));
    }

    #[test]
    fn test_crafting_table_2x2() {
        let book = RecipeBook::new();
        let grid = [
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
        ];
        let result = book.match_grid(&grid, 2, 2);
        assert_eq!(result, Some(ItemStack { block: BlockId::CraftingTable, count: 1 }));
    }

    #[test]
    fn test_pickaxe_3x3() {
        let book = RecipeBook::new();
        let grid = [
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
        ];
        let result = book.match_grid(&grid, 3, 3);
        assert_eq!(result, Some(ItemStack { block: BlockId::StonePickaxe, count: 1 }));
    }

    #[test]
    fn test_stick_recipe_in_2x2_and_consumption() {
        let book = RecipeBook::new();
        let mut grid = [
            Some(ItemStack { block: BlockId::Planks, count: 2 }),
            None,
            Some(ItemStack { block: BlockId::Planks, count: 2 }),
            None,
        ];
        let result = book.match_grid(&grid, 2, 2);
        assert_eq!(result, Some(ItemStack { block: BlockId::Stick, count: 4 }));

        RecipeBook::consume_grid(&mut grid);
        assert_eq!(grid[0], Some(ItemStack { block: BlockId::Planks, count: 1 }));
        assert_eq!(grid[2], Some(ItemStack { block: BlockId::Planks, count: 1 }));

        RecipeBook::consume_grid(&mut grid);
        assert_eq!(grid[0], None);
        assert_eq!(grid[2], None);
    }

    #[test]
    fn test_sword_and_shovel_3x3() {
        let book = RecipeBook::new();
        // Iron Sword
        let sword_grid = [
            None, Some(ItemStack { block: BlockId::IronIngot, count: 1 }), None,
            None, Some(ItemStack { block: BlockId::IronIngot, count: 1 }), None,
            None, Some(ItemStack { block: BlockId::Stick, count: 1 }), None,
        ];
        assert_eq!(
            book.match_grid(&sword_grid, 3, 3),
            Some(ItemStack { block: BlockId::IronSword, count: 1 })
        );

        // Wooden Shovel
        let shovel_grid = [
            Some(ItemStack { block: BlockId::Planks, count: 1 }), None, None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }), None, None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }), None, None,
        ];
        assert_eq!(
            book.match_grid(&shovel_grid, 3, 3),
            Some(ItemStack { block: BlockId::WoodenShovel, count: 1 })
        );
    }

    #[test]
    fn test_torch_crafting() {
        let book = RecipeBook::new();
        let mut grid = [None; 4];
        grid[0] = Some(ItemStack { block: BlockId::CoalOre, count: 1 });
        grid[2] = Some(ItemStack { block: BlockId::Stick, count: 1 });
        let result = book.match_grid(&grid, 2, 2);
        assert_eq!(result, Some(ItemStack { block: BlockId::Torch, count: 4 }));
    }

    #[test]
    fn test_ounces_to_ingots_and_reverse() {
        let book = RecipeBook::new();
        // 8 ounces in hollow 3x3 ring -> 1 ingot
        let iron_ring = [
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
            Some(ItemStack { block: BlockId::IronOunce, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&iron_ring, 3, 3),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 })
        );

        // 1 ingot -> 8 ounces (1x1 in 2x2 or 3x3)
        let ingot_grid = [
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }), None,
            None, None,
        ];
        assert_eq!(
            book.match_grid(&ingot_grid, 2, 2),
            Some(ItemStack { block: BlockId::IronOunce, count: 8 })
        );

        // Gold ring -> gold ingot
        let gold_ring = [
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
            Some(ItemStack { block: BlockId::GoldOunce, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&gold_ring, 3, 3),
            Some(ItemStack { block: BlockId::GoldIngot, count: 1 })
        );

        // 1 gold ingot -> 8 gold ounces
        let gold_ingot_grid = [
            Some(ItemStack { block: BlockId::GoldIngot, count: 1 }), None,
            None, None,
        ];
        assert_eq!(
            book.match_grid(&gold_ingot_grid, 2, 2),
            Some(ItemStack { block: BlockId::GoldOunce, count: 8 })
        );
    }

    #[test]
    fn test_ingots_to_blocks_and_reverse() {
        let book = RecipeBook::new();
        // 9 ingots -> 1 block
        let block_grid = [
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&block_grid, 3, 3),
            Some(ItemStack { block: BlockId::IronBlock, count: 1 })
        );

        // 1 block -> 9 ingots
        let decraft_block = [
            Some(ItemStack { block: BlockId::IronBlock, count: 1 }), None,
            None, None,
        ];
        assert_eq!(
            book.match_grid(&decraft_block, 2, 2),
            Some(ItemStack { block: BlockId::IronIngot, count: 9 })
        );
    }

    #[test]
    fn test_coal_block_and_reverse() {
        let book = RecipeBook::new();
        let coal_grid = [
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
            Some(ItemStack { block: BlockId::CoalOre, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&coal_grid, 3, 3),
            Some(ItemStack { block: BlockId::CoalBlock, count: 1 })
        );

        let decraft_coal = [
            Some(ItemStack { block: BlockId::CoalBlock, count: 1 }), None,
            None, None,
        ];
        assert_eq!(
            book.match_grid(&decraft_coal, 2, 2),
            Some(ItemStack { block: BlockId::CoalOre, count: 9 })
        );
    }

    #[test]
    fn test_diamond_recipes_and_architecture() {
        let book = RecipeBook::new();

        // 9x Diamond -> Diamond Block
        let dia_block_grid = [
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&dia_block_grid, 3, 3),
            Some(ItemStack { block: BlockId::DiamondBlock, count: 1 })
        );

        // 1x Diamond Block -> 9x Diamond
        let decraft_dia = [
            Some(ItemStack { block: BlockId::DiamondBlock, count: 1 }), None,
            None, None,
        ];
        assert_eq!(
            book.match_grid(&decraft_dia, 2, 2),
            Some(ItemStack { block: BlockId::Diamond, count: 9 })
        );

        // Diamond Pickaxe (3 Diamonds + 2 Sticks)
        let dia_pick_grid = [
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
        ];
        assert_eq!(
            book.match_grid(&dia_pick_grid, 3, 3),
            Some(ItemStack { block: BlockId::DiamondPickaxe, count: 1 })
        );

        // Bed (3 Feathers over 3 Planks)
        let bed_grid = [
            None, None, None,
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&bed_grid, 3, 3),
            Some(ItemStack { block: BlockId::Bed, count: 1 })
        );

        // Wooden Slab (3 Planks horizontal -> 6 Wooden Slabs)
        let slab_grid = [
            None, None, None,
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            None, None, None,
        ];
        assert_eq!(
            book.match_grid(&slab_grid, 3, 3),
            Some(ItemStack { block: BlockId::WoodenSlab, count: 6 })
        );

        // Ladder (7 Sticks in H shape -> 3 Ladders)
        let ladder_grid = [
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&ladder_grid, 3, 3),
            Some(ItemStack { block: BlockId::Ladder, count: 3 })
        );
    }

    #[test]
    fn test_recipe_macro_and_custom_addition() {
        let mut book = RecipeBook::new();
        let initial_count = book.recipes().len();

        // 1x1 test
        let custom_1x1 = recipe!(Dirt => Grass * 2);
        assert_eq!(custom_1x1.width, 1);
        assert_eq!(custom_1x1.height, 1);
        assert_eq!(custom_1x1.pattern, vec![Some(BlockId::Dirt)]);
        assert_eq!(custom_1x1.result, ItemStack { block: BlockId::Grass, count: 2 });

        // Shaped 2x2 with slot wildcard _
        let custom_2x2 = recipe!(2, 2, [
            Dirt, _,
            _,    Dirt,
        ] => Sand * 4);
        assert_eq!(custom_2x2.width, 2);
        assert_eq!(custom_2x2.height, 2);
        assert_eq!(custom_2x2.pattern, vec![Some(BlockId::Dirt), None, None, Some(BlockId::Dirt)]);
        assert_eq!(custom_2x2.result, ItemStack { block: BlockId::Sand, count: 4 });

        book.add(custom_2x2);
        assert_eq!(book.recipes().len(), initial_count + 1);

        let grid = [
            Some(ItemStack { block: BlockId::Dirt, count: 1 }), None,
            None, Some(ItemStack { block: BlockId::Dirt, count: 1 }),
        ];
        assert_eq!(book.match_grid(&grid, 2, 2), Some(ItemStack { block: BlockId::Sand, count: 4 }));
    }

    #[test]
    fn test_ruststone_recipes() {
        let book = RecipeBook::new();

        // 9x RustStone -> RustStoneBlock
        let block_grid = [
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
            Some(ItemStack { block: BlockId::RustStone, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&block_grid, 3, 3),
            Some(ItemStack { block: BlockId::RustStoneBlock, count: 1 })
        );

        // 1x RustStoneBlock -> 9x RustStone
        let decraft_grid = [
            Some(ItemStack { block: BlockId::RustStoneBlock, count: 1 }), None,
            None, None,
        ];
        assert_eq!(
            book.match_grid(&decraft_grid, 2, 2),
            Some(ItemStack { block: BlockId::RustStone, count: 9 })
        );

        // Lever: Stick over Stone
        let lever_grid = [
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&lever_grid, 1, 2),
            Some(ItemStack { block: BlockId::Lever, count: 1 })
        );

        // Button: 1 Stone
        let button_grid = [
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&button_grid, 1, 1),
            Some(ItemStack { block: BlockId::Button, count: 1 })
        );

        // Pressure Plate: 2 Stone
        let plate_grid = [
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
            Some(ItemStack { block: BlockId::Stone, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&plate_grid, 2, 1),
            Some(ItemStack { block: BlockId::PressurePlate, count: 1 })
        );
    }

    #[test]
    fn test_shears_and_wool_recipes() {
        let book = RecipeBook::new();

        // 1. Shears: 2 Iron Ingots diagonally
        let shears_grid = [
            None, Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }), None,
        ];
        assert_eq!(
            book.match_grid(&shears_grid, 2, 2),
            Some(ItemStack { block: BlockId::Shears, count: 1 })
        );

        // 2. Bed: 3 Wool over 3 Planks
        let bed_grid = [
            Some(ItemStack { block: BlockId::RedWool, count: 1 }),
            Some(ItemStack { block: BlockId::RedWool, count: 1 }),
            Some(ItemStack { block: BlockId::RedWool, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&bed_grid, 3, 2),
            Some(ItemStack { block: BlockId::Bed, count: 1 })
        );

        // 3. 4 Feathers -> 1 White Wool
        let wool_grid = [
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&wool_grid, 2, 2),
            Some(ItemStack { block: BlockId::WhiteWool, count: 1 })
        );

        // 4. Dyeing: Apple + WhiteWool -> RedWool
        let dye_grid = [
            Some(ItemStack { block: BlockId::Apple, count: 1 }),
            Some(ItemStack { block: BlockId::WhiteWool, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&dye_grid, 2, 1),
            Some(ItemStack { block: BlockId::RedWool, count: 1 })
        );
    }

    #[test]
    fn test_new_building_blocks_recipes() {
        let book = RecipeBook::new();

        // 4 ClayBall -> 1 Clay
        let clay_grid = [
            Some(ItemStack { block: BlockId::ClayBall, count: 1 }),
            Some(ItemStack { block: BlockId::ClayBall, count: 1 }),
            Some(ItemStack { block: BlockId::ClayBall, count: 1 }),
            Some(ItemStack { block: BlockId::ClayBall, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&clay_grid, 2, 2),
            Some(ItemStack { block: BlockId::Clay, count: 1 })
        );

        // 4 Brick -> 1 Bricks
        let brick_grid = [
            Some(ItemStack { block: BlockId::Brick, count: 1 }),
            Some(ItemStack { block: BlockId::Brick, count: 1 }),
            Some(ItemStack { block: BlockId::Brick, count: 1 }),
            Some(ItemStack { block: BlockId::Brick, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&brick_grid, 2, 2),
            Some(ItemStack { block: BlockId::Bricks, count: 1 })
        );

        // 4 Sand -> 1 Sandstone
        let sand_grid = [
            Some(ItemStack { block: BlockId::Sand, count: 1 }),
            Some(ItemStack { block: BlockId::Sand, count: 1 }),
            Some(ItemStack { block: BlockId::Sand, count: 1 }),
            Some(ItemStack { block: BlockId::Sand, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&sand_grid, 2, 2),
            Some(ItemStack { block: BlockId::Sandstone, count: 1 })
        );

        // Cobblestone Furnace
        let furn_grid = [
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&furn_grid, 3, 3),
            Some(ItemStack { block: BlockId::Furnace, count: 1 })
        );

        // Cobblestone Pickaxe (3 Cobblestone on top, 2 Sticks below)
        let pick_grid = [
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            Some(ItemStack { block: BlockId::Cobblestone, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
            None,
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            None,
        ];
        assert_eq!(
            book.match_grid(&pick_grid, 3, 3),
            Some(ItemStack { block: BlockId::StonePickaxe, count: 1 })
        );
    }

    #[test]
    fn test_combat_and_armor_recipes() {
        let book = RecipeBook::new();

        // 1. Arrow recipe (Flint, Stick, Feather)
        let arrow_grid = [
            Some(ItemStack { block: BlockId::Flint, count: 1 }),
            Some(ItemStack { block: BlockId::Stick, count: 1 }),
            Some(ItemStack { block: BlockId::Feather, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&arrow_grid, 1, 3),
            Some(ItemStack { block: BlockId::Arrow, count: 4 })
        );

        // 2. Shield recipe (Planks, IronIngot, Planks...)
        let shield_grid = [
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::IronIngot, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Planks, count: 1 }),
            None,
        ];
        assert_eq!(
            book.match_grid(&shield_grid, 3, 3),
            Some(ItemStack { block: BlockId::Shield, count: 1 })
        );

        // 3. Diamond Helmet (3 Diamond on top, 2 on sides)
        let helmet_grid = [
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
            None,
            Some(ItemStack { block: BlockId::Diamond, count: 1 }),
        ];
        assert_eq!(
            book.match_grid(&helmet_grid, 3, 2),
            Some(ItemStack { block: BlockId::DiamondHelmet, count: 1 })
        );
    }
}

