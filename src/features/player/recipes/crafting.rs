//! Crafting grid pattern matching (trimmed bounding box) and ingredient consumption.

use super::super::inventory::ItemStack;
use super::types::RecipeBook;

impl RecipeBook {
    pub fn match_grid(&self, grid: &[Option<ItemStack>], grid_w: usize, grid_h: usize) -> Option<ItemStack> {
        let mut min_r = None;
        let mut max_r = 0;
        let mut min_c = None;
        let mut max_c = 0;

        for r in 0..grid_h {
            for c in 0..grid_w {
                let idx = r * grid_w + c;
                if grid[idx].is_some() {
                    if min_r.is_none() {
                        min_r = Some(r);
                    }
                    max_r = r;
                    if min_c.is_none() || c < min_c.unwrap() {
                        min_c = Some(c);
                    }
                    if c > max_c {
                        max_c = c;
                    }
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
                sub_pattern.push(grid[r * grid_w + c].map(|s| s.block));
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
