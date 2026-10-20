use crate::gameplay::inventory::ItemStack;
use crate::world::block::BlockId;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub width: usize,
    pub height: usize,
    pub pattern: Vec<Option<BlockId>>,
    pub result: ItemStack,
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

    fn register_default_recipes(&mut self) {
        // 1x Wood -> 4x Planks (1x1)
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::Wood)],
            result: ItemStack { block: BlockId::Planks, count: 4 },
        });

        // 2x Planks (vertical 1x2) -> 4x Stick
        self.recipes.push(Recipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(BlockId::Planks),
                Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::Stick, count: 4 },
        });

        // 1x CoalOre over 1x Stick (vertical 1x2) -> 4x Torch
        self.recipes.push(Recipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(BlockId::CoalOre),
                Some(BlockId::Stick),
            ],
            result: ItemStack { block: BlockId::Torch, count: 4 },
        });

        // 4x Planks (2x2) -> 1x CraftingTable
        self.recipes.push(Recipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(BlockId::Planks), Some(BlockId::Planks),
                Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::CraftingTable, count: 1 },
        });

        // 1x RawPorkchop over 1x CoalOre (vertical 1x2) -> 1x CookedPorkchop
        self.recipes.push(Recipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(BlockId::RawPorkchop),
                Some(BlockId::CoalOre),
            ],
            result: ItemStack { block: BlockId::CookedPorkchop, count: 1 },
        });

        // 1x RawBeef over 1x CoalOre (vertical 1x2) -> 1x CookedBeef
        self.recipes.push(Recipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(BlockId::RawBeef),
                Some(BlockId::CoalOre),
            ],
            result: ItemStack { block: BlockId::CookedBeef, count: 1 },
        });

        // 1x RawPorkchop over 1x Torch (vertical 1x2) -> 1x CookedPorkchop
        self.recipes.push(Recipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(BlockId::RawPorkchop),
                Some(BlockId::Torch),
            ],
            result: ItemStack { block: BlockId::CookedPorkchop, count: 1 },
        });

        // 1x RawBeef over 1x Torch (vertical 1x2) -> 1x CookedBeef
        self.recipes.push(Recipe {
            width: 1,
            height: 2,
            pattern: vec![
                Some(BlockId::RawBeef),
                Some(BlockId::Torch),
            ],
            result: ItemStack { block: BlockId::CookedBeef, count: 1 },
        });

        // Chest (3x3 ring: 8 Planks)
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
                Some(BlockId::Planks), None,                  Some(BlockId::Planks),
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::Chest, count: 1 },
        });

        // Furnace (3x3 ring: 8 Stone)
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::Stone), Some(BlockId::Stone), Some(BlockId::Stone),
                Some(BlockId::Stone), None,                 Some(BlockId::Stone),
                Some(BlockId::Stone), Some(BlockId::Stone), Some(BlockId::Stone),
            ],
            result: ItemStack { block: BlockId::Furnace, count: 1 },
        });

        // 8x Iron Ounces (3x3 ring: 8 Ounces) -> 1x Iron Ingot
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::IronOunce), Some(BlockId::IronOunce), Some(BlockId::IronOunce),
                Some(BlockId::IronOunce), None,                    Some(BlockId::IronOunce),
                Some(BlockId::IronOunce), Some(BlockId::IronOunce), Some(BlockId::IronOunce),
            ],
            result: ItemStack { block: BlockId::IronIngot, count: 1 },
        });

        // 1x Iron Ingot (1x1) -> 8x Iron Ounces
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::IronIngot)],
            result: ItemStack { block: BlockId::IronOunce, count: 8 },
        });

        // 8x Gold Ounces (3x3 ring: 8 Ounces) -> 1x Gold Ingot
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::GoldOunce), Some(BlockId::GoldOunce), Some(BlockId::GoldOunce),
                Some(BlockId::GoldOunce), None,                    Some(BlockId::GoldOunce),
                Some(BlockId::GoldOunce), Some(BlockId::GoldOunce), Some(BlockId::GoldOunce),
            ],
            result: ItemStack { block: BlockId::GoldIngot, count: 1 },
        });

        // 1x Gold Ingot (1x1) -> 8x Gold Ounces
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::GoldIngot)],
            result: ItemStack { block: BlockId::GoldOunce, count: 8 },
        });

        // 9x Iron Ingots (3x3 full) -> 1x Iron Block
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::IronIngot), Some(BlockId::IronIngot), Some(BlockId::IronIngot),
                Some(BlockId::IronIngot), Some(BlockId::IronIngot), Some(BlockId::IronIngot),
                Some(BlockId::IronIngot), Some(BlockId::IronIngot), Some(BlockId::IronIngot),
            ],
            result: ItemStack { block: BlockId::IronBlock, count: 1 },
        });

        // 1x Iron Block (1x1) -> 9x Iron Ingots
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::IronBlock)],
            result: ItemStack { block: BlockId::IronIngot, count: 9 },
        });

        // 9x Gold Ingots (3x3 full) -> 1x Gold Block
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::GoldIngot), Some(BlockId::GoldIngot), Some(BlockId::GoldIngot),
                Some(BlockId::GoldIngot), Some(BlockId::GoldIngot), Some(BlockId::GoldIngot),
                Some(BlockId::GoldIngot), Some(BlockId::GoldIngot), Some(BlockId::GoldIngot),
            ],
            result: ItemStack { block: BlockId::GoldBlock, count: 1 },
        });

        // 1x Gold Block (1x1) -> 9x Gold Ingots
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::GoldBlock)],
            result: ItemStack { block: BlockId::GoldIngot, count: 9 },
        });

        // 9x Coal Ore (3x3 full) -> 1x Coal Block
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::CoalOre), Some(BlockId::CoalOre), Some(BlockId::CoalOre),
                Some(BlockId::CoalOre), Some(BlockId::CoalOre), Some(BlockId::CoalOre),
                Some(BlockId::CoalOre), Some(BlockId::CoalOre), Some(BlockId::CoalOre),
            ],
            result: ItemStack { block: BlockId::CoalBlock, count: 1 },
        });

        // 1x Coal Block (1x1) -> 9x Coal Ore
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::CoalBlock)],
            result: ItemStack { block: BlockId::CoalOre, count: 9 },
        });

        // 9x Diamond (3x3) -> 1x Diamond Block
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::Diamond), Some(BlockId::Diamond), Some(BlockId::Diamond),
                Some(BlockId::Diamond), Some(BlockId::Diamond), Some(BlockId::Diamond),
                Some(BlockId::Diamond), Some(BlockId::Diamond), Some(BlockId::Diamond),
            ],
            result: ItemStack { block: BlockId::DiamondBlock, count: 1 },
        });

        // 1x Diamond Block (1x1) -> 9x Diamond
        self.recipes.push(Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some(BlockId::DiamondBlock)],
            result: ItemStack { block: BlockId::Diamond, count: 9 },
        });

        // Shovels (1x3: Material over 2 Sticks)
        let shovel_materials = [
            (BlockId::Planks, BlockId::WoodenShovel),
            (BlockId::Stone, BlockId::StoneShovel),
            (BlockId::IronIngot, BlockId::IronShovel),
            (BlockId::GoldIngot, BlockId::GoldShovel),
            (BlockId::Diamond, BlockId::DiamondShovel),
        ];
        for (mat, tool) in shovel_materials {
            self.recipes.push(Recipe {
                width: 1,
                height: 3,
                pattern: vec![
                    Some(mat),
                    Some(BlockId::Stick),
                    Some(BlockId::Stick),
                ],
                result: ItemStack { block: tool, count: 1 },
            });
        }

        // Swords (1x3: 2 Materials over Stick)
        let sword_materials = [
            (BlockId::Planks, BlockId::WoodenSword),
            (BlockId::Stone, BlockId::StoneSword),
            (BlockId::IronIngot, BlockId::IronSword),
            (BlockId::GoldIngot, BlockId::GoldSword),
            (BlockId::Diamond, BlockId::DiamondSword),
        ];
        for (mat, tool) in sword_materials {
            self.recipes.push(Recipe {
                width: 1,
                height: 3,
                pattern: vec![
                    Some(mat),
                    Some(mat),
                    Some(BlockId::Stick),
                ],
                result: ItemStack { block: tool, count: 1 },
            });
        }

        // Pickaxes (3x3: 3 Materials on top, 2 Sticks below center)
        let pick_materials = [
            (BlockId::Planks, BlockId::WoodenPickaxe),
            (BlockId::Stone, BlockId::StonePickaxe),
            (BlockId::IronIngot, BlockId::IronPickaxe),
            (BlockId::GoldIngot, BlockId::GoldPickaxe),
            (BlockId::Diamond, BlockId::DiamondPickaxe),
        ];
        for (mat, tool) in pick_materials {
            self.recipes.push(Recipe {
                width: 3,
                height: 3,
                pattern: vec![
                    Some(mat), Some(mat), Some(mat),
                    None, Some(BlockId::Stick), None,
                    None, Some(BlockId::Stick), None,
                ],
                result: ItemStack { block: tool, count: 1 },
            });
        }

        // Axes (2x3: 3 Materials corner, 2 Sticks) - right and left variants
        let axe_materials = [
            (BlockId::Planks, BlockId::WoodenAxe),
            (BlockId::Stone, BlockId::StoneAxe),
            (BlockId::IronIngot, BlockId::IronAxe),
            (BlockId::GoldIngot, BlockId::GoldAxe),
            (BlockId::Diamond, BlockId::DiamondAxe),
        ];
        for (mat, tool) in axe_materials {
            // Right-facing axe
            self.recipes.push(Recipe {
                width: 2,
                height: 3,
                pattern: vec![
                    Some(mat), Some(mat),
                    Some(mat), Some(BlockId::Stick),
                    None, Some(BlockId::Stick),
                ],
                result: ItemStack { block: tool, count: 1 },
            });
            // Left-facing axe
            self.recipes.push(Recipe {
                width: 2,
                height: 3,
                pattern: vec![
                    Some(mat), Some(mat),
                    Some(BlockId::Stick), Some(mat),
                    Some(BlockId::Stick), None,
                ],
                result: ItemStack { block: tool, count: 1 },
            });
        }

        // Bed (3x2: 3 Feathers over 3 Planks)
        self.recipes.push(Recipe {
            width: 3,
            height: 2,
            pattern: vec![
                Some(BlockId::Feather), Some(BlockId::Feather), Some(BlockId::Feather),
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::Bed, count: 1 },
        });

        // Bed alternate (3x2: 3 Leaves over 3 Planks)
        self.recipes.push(Recipe {
            width: 3,
            height: 2,
            pattern: vec![
                Some(BlockId::Leaves), Some(BlockId::Leaves), Some(BlockId::Leaves),
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::Bed, count: 1 },
        });

        // Wooden Slabs (3x1: 3 Planks horizontal -> 6 WoodenSlab)
        self.recipes.push(Recipe {
            width: 3,
            height: 1,
            pattern: vec![
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::WoodenSlab, count: 6 },
        });

        // Stone Slabs (3x1: 3 Stone horizontal -> 6 StoneSlab)
        self.recipes.push(Recipe {
            width: 3,
            height: 1,
            pattern: vec![
                Some(BlockId::Stone), Some(BlockId::Stone), Some(BlockId::Stone),
            ],
            result: ItemStack { block: BlockId::StoneSlab, count: 6 },
        });

        // Wooden Stairs (3x3 stair shape -> 4 WoodenStairs)
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::Planks), None, None,
                Some(BlockId::Planks), Some(BlockId::Planks), None,
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::WoodenStairs, count: 4 },
        });
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                None, None, Some(BlockId::Planks),
                None, Some(BlockId::Planks), Some(BlockId::Planks),
                Some(BlockId::Planks), Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::WoodenStairs, count: 4 },
        });

        // Stone Stairs (3x3 stair shape -> 4 StoneStairs)
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::Stone), None, None,
                Some(BlockId::Stone), Some(BlockId::Stone), None,
                Some(BlockId::Stone), Some(BlockId::Stone), Some(BlockId::Stone),
            ],
            result: ItemStack { block: BlockId::StoneStairs, count: 4 },
        });
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                None, None, Some(BlockId::Stone),
                None, Some(BlockId::Stone), Some(BlockId::Stone),
                Some(BlockId::Stone), Some(BlockId::Stone), Some(BlockId::Stone),
            ],
            result: ItemStack { block: BlockId::StoneStairs, count: 4 },
        });

        // Ladder (3x3: H-shape of Sticks -> 3 Ladder)
        self.recipes.push(Recipe {
            width: 3,
            height: 3,
            pattern: vec![
                Some(BlockId::Stick), None, Some(BlockId::Stick),
                Some(BlockId::Stick), Some(BlockId::Stick), Some(BlockId::Stick),
                Some(BlockId::Stick), None, Some(BlockId::Stick),
            ],
            result: ItemStack { block: BlockId::Ladder, count: 3 },
        });

        // Doors (2x3: 6 Planks)
        self.recipes.push(Recipe {
            width: 2,
            height: 3,
            pattern: vec![
                Some(BlockId::Planks), Some(BlockId::Planks),
                Some(BlockId::Planks), Some(BlockId::Planks),
                Some(BlockId::Planks), Some(BlockId::Planks),
            ],
            result: ItemStack { block: BlockId::Door, count: 1 },
        });

        // TNT (2x2: Sand + Dirt)
        self.recipes.push(Recipe {
            width: 2,
            height: 2,
            pattern: vec![
                Some(BlockId::Sand), Some(BlockId::Dirt),
                Some(BlockId::Dirt), Some(BlockId::Sand),
            ],
            result: ItemStack { block: BlockId::Tnt, count: 1 },
        });
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
}
