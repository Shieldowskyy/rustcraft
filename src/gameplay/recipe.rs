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

        // Shovels (1x3: Material over 2 Sticks)
        let shovel_materials = [
            (BlockId::Planks, BlockId::WoodenShovel),
            (BlockId::Stone, BlockId::StoneShovel),
            (BlockId::IronOre, BlockId::IronShovel),
            (BlockId::IronIngot, BlockId::IronShovel),
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
            (BlockId::IronOre, BlockId::IronSword),
            (BlockId::IronIngot, BlockId::IronSword),
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
            (BlockId::IronOre, BlockId::IronPickaxe),
            (BlockId::IronIngot, BlockId::IronPickaxe),
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
            (BlockId::IronOre, BlockId::IronAxe),
            (BlockId::IronIngot, BlockId::IronAxe),
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
            None, Some(ItemStack { block: BlockId::IronOre, count: 1 }), None,
            None, Some(ItemStack { block: BlockId::IronOre, count: 1 }), None,
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
}
