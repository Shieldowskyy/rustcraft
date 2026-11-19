//! Player inventory domain (`ItemStack`, `Inventory`, slot interactions, stacking, auto-equip).
//! Strictly isolated from UI layout and platform `#[cfg]`.

mod ops;

#[cfg(test)]
mod tests;

use crate::features::blocks::BlockId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemStack {
    pub block: BlockId,
    pub count: u8,
    pub durability: u16,
}

impl ItemStack {
    #[inline(always)]
    pub const fn new(block: BlockId, count: u8) -> Self {
        Self {
            block,
            count,
            durability: block.max_durability(),
        }
    }

    #[inline(always)]
    pub const fn with_durability(block: BlockId, count: u8, durability: u16) -> Self {
        Self { block, count, durability }
    }

    /// Reduces durability by `amount`. Returns `true` if destroyed (reached 0).
    pub fn damage(&mut self, amount: u16) -> bool {
        if self.durability == 0 {
            return false;
        }
        if amount >= self.durability {
            self.durability = 0;
            true
        } else {
            self.durability -= amount;
            false
        }
    }

    #[inline(always)]
    pub fn durability_ratio(&self) -> f32 {
        let max = self.block.max_durability();
        if max == 0 {
            1.0
        } else {
            (self.durability as f32 / max as f32).clamp(0.0, 1.0)
        }
    }
}

pub struct Inventory {
    pub hotbar: [Option<ItemStack>; 10],
    pub main: [Option<ItemStack>; 27],
    pub crafting_2x2: [Option<ItemStack>; 4],
    pub crafting_3x3: [Option<ItemStack>; 9],
    pub held_item: Option<ItemStack>,
    pub armor: [Option<ItemStack>; 4],
    pub offhand: Option<ItemStack>,
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            hotbar: [None; 10],
            main: [None; 27],
            crafting_2x2: [None; 4],
            crafting_3x3: [None; 9],
            held_item: None,
            armor: [None; 4],
            offhand: None,
        }
    }
}
