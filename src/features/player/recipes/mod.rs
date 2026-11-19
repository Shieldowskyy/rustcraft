//! Crafting recipes domain facade (`Recipe`, `RecipeBook`, `recipe!`, `recipe_slot!`).
//! Strictly isolated from UI and platform `#[cfg]`.

#[macro_use]
mod types;
mod catalog_blocks;
mod catalog_items;
mod crafting;

#[cfg(test)]
mod tests_catalog;
#[cfg(test)]
mod tests_core;

pub use types::{Recipe, RecipeBook};
