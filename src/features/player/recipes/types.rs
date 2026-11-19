//! Core recipe types (`Recipe`, `RecipeBook`) and declarative `recipe!`/`recipe_slot!` macros.

use crate::features::blocks::BlockId;

use super::super::achievements::AchievementProgress;
use super::super::inventory::ItemStack;
use super::{catalog_blocks, catalog_items};

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
        Some($crate::features::blocks::BlockId::$item)
    };
}

/// Declarative macro for defining crafting recipes concisely.
#[macro_export]
macro_rules! recipe {
    ($input:ident => $result:ident * $count:expr) => {
        $crate::features::player::recipes::Recipe {
            width: 1,
            height: 1,
            pattern: vec![Some($crate::features::blocks::BlockId::$input)],
            result: $crate::features::player::inventory::ItemStack::new(
                $crate::features::blocks::BlockId::$result,
                $count,
            ),
        }
    };

    ($w:expr, $h:expr, [ $($item:tt),* $(,)? ] => $result:ident * $count:expr) => {
        $crate::features::player::recipes::Recipe {
            width: $w,
            height: $h,
            pattern: vec![ $( $crate::recipe_slot!($item) ),* ],
            result: $crate::features::player::inventory::ItemStack::new(
                $crate::features::blocks::BlockId::$result,
                $count,
            ),
        }
    };
}

impl Recipe {
    pub fn new(
        width: usize,
        height: usize,
        pattern: Vec<Option<BlockId>>,
        result: BlockId,
        count: u8,
    ) -> Self {
        Self { width, height, pattern, result: ItemStack::new(result, count) }
    }

    pub fn is_unlocked_by(&self, mut has_collected_item: impl FnMut(BlockId) -> bool) -> bool {
        self.pattern.iter().flatten().all(|&ingredient| has_collected_item(ingredient))
    }
}

pub struct RecipeBook {
    pub(super) recipes: Vec<Recipe>,
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn unlocked_recipe_indices(&self, progress: &AchievementProgress) -> Vec<usize> {
        self.recipes
            .iter()
            .enumerate()
            .filter_map(|(index, recipe)| {
                recipe.is_unlocked_by(|item| progress.has_collected_item(item)).then_some(index)
            })
            .collect()
    }

    fn register_default_recipes(&mut self) {
        catalog_items::register_item_and_gear_recipes(self);
        catalog_blocks::register_building_and_mechanic_recipes(self);
    }
}
