use crate::core::math::IVec3;
use crate::gameplay::inventory::ItemStack;
use crate::world::block::BlockId;

pub const SMELT_DURATION: f32 = 10.0;

/// Returns the fuel burn time in seconds for a given block or item.
pub fn fuel_burn_time(block: BlockId) -> Option<f32> {
    match block {
        BlockId::CoalBlock => Some(720.0),
        BlockId::CoalOre => Some(80.0),
        BlockId::Wood => Some(15.0),
        BlockId::Planks => Some(15.0),
        BlockId::CraftingTable => Some(15.0),
        BlockId::Chest => Some(15.0),
        BlockId::Door
        | BlockId::WoodenPickaxe
        | BlockId::WoodenAxe
        | BlockId::WoodenShovel
        | BlockId::WoodenSword
        | BlockId::WoodenHoe => Some(10.0),
        BlockId::Stick | BlockId::Sapling => Some(5.0),
        BlockId::Leaves
        | BlockId::SnowLeaves
        | BlockId::CherryLeaves
        | BlockId::AutumnLeaves => Some(2.0),
        _ => None,
    }
}

/// Returns the output block/item produced by smelting the input block.
pub fn smelting_result(input: BlockId) -> Option<BlockId> {
    if let Some(res) = crate::engine::modding::get_custom_smelting(input) {
        return Some(res);
    }
    match input {
        BlockId::RawPorkchop => Some(BlockId::CookedPorkchop),
        BlockId::RawBeef => Some(BlockId::CookedBeef),
        BlockId::RawChicken => Some(BlockId::CookedChicken),
        BlockId::RawCod => Some(BlockId::CookedCod),
        BlockId::RawSalmon => Some(BlockId::CookedSalmon),
        BlockId::Kelp => Some(BlockId::DriedKelp),
        BlockId::Potato => Some(BlockId::BakedPotato),
        BlockId::Sand => Some(BlockId::Glass),
        BlockId::IronOre => Some(BlockId::IronOunce),
        BlockId::GoldOre => Some(BlockId::GoldOunce),
        BlockId::EmeraldOre => Some(BlockId::Emerald),
        BlockId::Wood => Some(BlockId::CoalOre),
        BlockId::Cobblestone => Some(BlockId::Stone),
        BlockId::Stone => Some(BlockId::SmoothStone),
        BlockId::ClayBall => Some(BlockId::Brick),
        _ => None,
    }
}

/// Data stored for an individual chest tile entity (27 inventory slots).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChestData {
    pub items: [Option<ItemStack>; 27],
}

impl ChestData {
    pub fn new() -> Self {
        Self {
            items: [None; 27],
        }
    }

    /// Generates randomized starter/dungeon loot for cabin chests.
    pub fn generate_house_loot(seed: u32, pos: IVec3) -> Self {
        let mut chest = Self::new();
        let mut rng = (seed as u64)
            ^ ((pos.x as u64) << 32)
            ^ ((pos.y as u64) << 16)
            ^ (pos.z as u64);
        
        let mut next_rand = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng >> 32) as u32
        };

        // Always add some food, fuel, materials and tools
        chest.items[0] = Some(ItemStack::new(BlockId::Apple, 3 + (next_rand() % 5) as u8));
        chest.items[1] = Some(ItemStack::new(BlockId::CookedPorkchop, 2 + (next_rand() % 4) as u8));
        chest.items[4] = Some(ItemStack::new(BlockId::IronIngot, 2 + (next_rand() % 4) as u8));
        chest.items[9] = Some(ItemStack::new(BlockId::CoalOre, 4 + (next_rand() % 8) as u8));
        chest.items[10] = Some(ItemStack::new(BlockId::Torch, 4 + (next_rand() % 6) as u8));
        chest.items[13] = Some(ItemStack::new(BlockId::Stick, 6 + (next_rand() % 10) as u8));
        if next_rand() % 2 == 0 {
            chest.items[14] = Some(ItemStack::new(BlockId::Emerald, 1 + (next_rand() % 3) as u8));
        }
        chest.items[18] = Some(ItemStack::new(BlockId::Planks, 8 + (next_rand() % 16) as u8));
        if next_rand() % 2 == 0 {
            chest.items[22] = Some(ItemStack::new(BlockId::StonePickaxe, 1));
        } else {
            chest.items[22] = Some(ItemStack::new(BlockId::StoneAxe, 1));
        }

        chest
    }

    /// Generates village blacksmith forge loot (Emeralds, Iron, Obsidian/Stone, Bread, Tools).
    pub fn generate_blacksmith_loot(seed: u32, pos: IVec3) -> Self {
        let mut chest = Self::new();
        let mut rng = (seed as u64)
            ^ ((pos.x as u64) << 32)
            ^ ((pos.y as u64) << 16)
            ^ (pos.z as u64);

        let mut next_rand = || {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (rng >> 32) as u32
        };

        // Emeralds (1 - 6)
        chest.items[2] = Some(ItemStack::new(BlockId::Emerald, 1 + (next_rand() % 6) as u8));
        // Iron Ingots (2 - 7)
        chest.items[4] = Some(ItemStack::new(BlockId::IronIngot, 2 + (next_rand() % 6) as u8));
        // Bread (1 - 4)
        chest.items[7] = Some(ItemStack::new(BlockId::Bread, 1 + (next_rand() % 4) as u8));
        // Gold Ingot or Apple
        if next_rand() % 2 == 0 {
            chest.items[11] = Some(ItemStack::new(BlockId::GoldIngot, 1 + (next_rand() % 3) as u8));
        } else {
            chest.items[11] = Some(ItemStack::new(BlockId::Apple, 2 + (next_rand() % 3) as u8));
        }
        // Blacksmith Tool
        match next_rand() % 3 {
            0 => {
                chest.items[20] = Some(ItemStack::new(BlockId::IronPickaxe, 1));
            }
            1 => {
                chest.items[20] = Some(ItemStack::new(BlockId::IronSword, 1));
            }
            _ => {
                chest.items[20] = Some(ItemStack::new(BlockId::IronAxe, 1));
            }
        }

        chest
    }

    /// Adds items to the chest, stacking where possible. Returns remaining unplaced count.
    pub fn add_item(&mut self, block: BlockId, mut count: u8) -> u8 {
        for slot in &mut self.items {
            if let Some(stack) = slot {
                if stack.block == block && stack.count < 64 {
                    let to_add = count.min(64 - stack.count);
                    stack.count += to_add;
                    count -= to_add;
                    if count == 0 {
                        return 0;
                    }
                }
            }
        }
        for slot in &mut self.items {
            if slot.is_none() {
                let to_add = count.min(64);
                *slot = Some(ItemStack::new(block, to_add));
                count -= to_add;
                if count == 0 {
                    return 0;
                }
            }
        }
        count
    }
}

/// Data stored for an individual furnace tile entity (smelting logic, fuel and cook progress).
#[derive(Clone, Debug, PartialEq)]
pub struct FurnaceData {
    pub input: Option<ItemStack>,
    pub fuel: Option<ItemStack>,
    pub output: Option<ItemStack>,
    pub burn_time_remaining: f32,
    pub burn_time_total: f32,
    pub cook_progress: f32,
}

impl FurnaceData {
    pub fn new() -> Self {
        Self {
            input: None,
            fuel: None,
            output: None,
            burn_time_remaining: 0.0,
            burn_time_total: 0.0,
            cook_progress: 0.0,
        }
    }

    pub fn is_burning(&self) -> bool {
        self.burn_time_remaining > 0.0
    }

    /// Ticks the furnace forward in time. Returns `true` if the burning state (lit/unlit) changed.
    pub fn tick(&mut self, dt: f32) -> bool {
        let was_burning = self.is_burning();

        let can_smelt = if let Some(input_stack) = self.input {
            if let Some(res) = smelting_result(input_stack.block) {
                match self.output {
                    None => true,
                    Some(out) if out.block == res && out.count < 64 => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        };

        // If not burning but we have smeltable items, attempt to consume fuel
        if self.burn_time_remaining <= 0.0 && can_smelt {
            if let Some(fuel_stack) = &mut self.fuel {
                if let Some(burn_dur) = fuel_burn_time(fuel_stack.block) {
                    self.burn_time_remaining = burn_dur;
                    self.burn_time_total = burn_dur;
                    fuel_stack.count -= 1;
                    if fuel_stack.count == 0 {
                        self.fuel = None;
                    }
                }
            }
        }

        // Advance or extinguish burning fuel
        if self.burn_time_remaining > 0.0 {
            self.burn_time_remaining = (self.burn_time_remaining - dt).max(0.0);
            if self.burn_time_remaining == 0.0 {
                self.burn_time_total = 0.0;
            }
        }

        // Advance cooking progress if actively burning and smelting is valid
        if self.burn_time_remaining > 0.0 && can_smelt {
            self.cook_progress += dt;
            if self.cook_progress >= SMELT_DURATION {
                self.cook_progress = 0.0;
                // Produce output item
                if let Some(input_stack) = &mut self.input {
                    if let Some(res) = smelting_result(input_stack.block) {
                        match &mut self.output {
                            None => {
                                self.output = Some(ItemStack::new(res, 1));
                            }
                            Some(out) => {
                                out.count += 1;
                            }
                        }
                        input_stack.count -= 1;
                        if input_stack.count == 0 {
                            self.input = None;
                        }
                    }
                }
            }
        } else if !can_smelt {
            self.cook_progress = 0.0;
        }

        let is_burning_now = self.is_burning();
        was_burning != is_burning_now
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_furnace_smelting_food_and_ores() {
        let mut furnace = FurnaceData::new();
        furnace.input = Some(ItemStack::new(BlockId::RawPorkchop, 2));
        furnace.fuel = Some(ItemStack::new(BlockId::CoalOre, 1));

        // Tick to consume fuel and cook first porkchop (SMELT_DURATION = 10s)
        let state_changed = furnace.tick(5.0);
        assert!(state_changed); // lit
        assert!(furnace.is_burning());
        assert_eq!(furnace.fuel, None); // 1 coal consumed

        furnace.tick(5.0); // complete cooking
        assert_eq!(furnace.output, Some(ItemStack::new(BlockId::CookedPorkchop, 1)));
        assert_eq!(furnace.input, Some(ItemStack::new(BlockId::RawPorkchop, 1)));
        assert!(furnace.is_burning()); // Coal lasts 80s, so still burning

        furnace.tick(10.0); // cook second porkchop
        assert_eq!(furnace.output, Some(ItemStack::new(BlockId::CookedPorkchop, 2)));
        assert_eq!(furnace.input, None);

        // Fuel continues to burn down even when input is empty (nothing being smelted)
        assert!(furnace.is_burning());
        assert_eq!(furnace.burn_time_remaining, 60.0); // 80 - 5 - 5 - 10 = 60

        furnace.tick(30.0);
        assert!(furnace.is_burning());
        assert_eq!(furnace.burn_time_remaining, 30.0);

        let went_out = furnace.tick(30.0);
        assert!(went_out);
        assert!(!furnace.is_burning());
        assert_eq!(furnace.burn_time_remaining, 0.0);
    }

    #[test]
    fn test_smelting_recipes() {
        assert_eq!(smelting_result(BlockId::RawPorkchop), Some(BlockId::CookedPorkchop));
        assert_eq!(smelting_result(BlockId::RawBeef), Some(BlockId::CookedBeef));
        assert_eq!(smelting_result(BlockId::RawChicken), Some(BlockId::CookedChicken));
        assert_eq!(smelting_result(BlockId::RawCod), Some(BlockId::CookedCod));
        assert_eq!(smelting_result(BlockId::RawSalmon), Some(BlockId::CookedSalmon));
        assert_eq!(smelting_result(BlockId::Kelp), Some(BlockId::DriedKelp));
        assert_eq!(smelting_result(BlockId::Sand), Some(BlockId::Glass));
        assert_eq!(smelting_result(BlockId::IronOre), Some(BlockId::IronOunce));
        assert_eq!(smelting_result(BlockId::GoldOre), Some(BlockId::GoldOunce));
        assert_eq!(smelting_result(BlockId::Wood), Some(BlockId::CoalOre));
        assert_eq!(smelting_result(BlockId::Cobblestone), Some(BlockId::Stone));
        assert_eq!(smelting_result(BlockId::Stone), Some(BlockId::SmoothStone));
        assert_eq!(smelting_result(BlockId::ClayBall), Some(BlockId::Brick));
        assert_eq!(smelting_result(BlockId::Air), None);
    }

    #[test]
    fn test_chest_loot_generation_and_stacking() {
        let mut chest = ChestData::generate_house_loot(12345, IVec3::new(10, 64, 10));
        let occupied = chest.items.iter().filter(|s| s.is_some()).count();
        assert!(occupied >= 6);

        let rem = chest.add_item(BlockId::Torch, 10);
        assert_eq!(rem, 0); // Stacked or placed into free slot
    }

    #[test]
    fn test_fuel_burn_times() {
        assert_eq!(fuel_burn_time(BlockId::CoalOre), Some(80.0));
        assert_eq!(fuel_burn_time(BlockId::CoalBlock), Some(720.0));
        assert_eq!(fuel_burn_time(BlockId::Wood), Some(15.0));
        assert_eq!(fuel_burn_time(BlockId::Planks), Some(15.0));
        assert_eq!(fuel_burn_time(BlockId::Chest), Some(15.0));
        assert_eq!(fuel_burn_time(BlockId::Door), Some(10.0));
        assert_eq!(fuel_burn_time(BlockId::WoodenPickaxe), Some(10.0));
        assert_eq!(fuel_burn_time(BlockId::Stick), Some(5.0));
        assert_eq!(fuel_burn_time(BlockId::Leaves), Some(2.0));
        assert_eq!(fuel_burn_time(BlockId::Stone), None);
        assert_eq!(fuel_burn_time(BlockId::Dirt), None);
    }
}
