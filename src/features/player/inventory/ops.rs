//! Inventory slot manipulation: crafting return, auto-equip, stacking, mouse click/drag, consume/count.

use crate::features::blocks::BlockId;

use super::{Inventory, ItemStack};

impl Inventory {
    pub fn return_crafting_items(&mut self) {
        let mut to_return = Vec::new();
        for slot in &mut self.crafting_2x2 {
            if let Some(stack) = slot.take() {
                to_return.push(stack);
            }
        }
        for slot in &mut self.crafting_3x3 {
            if let Some(stack) = slot.take() {
                to_return.push(stack);
            }
        }
        if let Some(stack) = self.held_item.take() {
            to_return.push(stack);
        }

        for stack in to_return {
            let rem = self.add_item(stack.block, stack.count);
            if rem > 0 {
                self.held_item = Some(ItemStack::new(stack.block, rem));
            }
        }
    }

    fn try_auto_equip_slot(
        slot: &mut Option<ItemStack>,
        armor: &mut [Option<ItemStack>; 4],
        offhand: &mut Option<ItemStack>,
    ) -> bool {
        let Some(stack) = *slot else {
            return false;
        };
        if let Some(armor_slot) = stack.block.armor_slot() {
            let aidx = armor_slot as usize;
            let old = armor[aidx].take();
            armor[aidx] = slot.take();
            *slot = old;
            true
        } else if stack.block.is_shield() {
            let old = offhand.take();
            *offhand = slot.take();
            *slot = old;
            true
        } else {
            false
        }
    }

    pub fn try_auto_equip_hotbar(&mut self, idx: usize) -> bool {
        Self::try_auto_equip_slot(&mut self.hotbar[idx], &mut self.armor, &mut self.offhand)
    }

    pub fn try_auto_equip_main(&mut self, idx: usize) -> bool {
        Self::try_auto_equip_slot(&mut self.main[idx], &mut self.armor, &mut self.offhand)
    }

    /// Attempts to consume 1 item of the specified type from offhand, hotbar, or main inventory.
    pub fn consume_item(&mut self, block: BlockId) -> bool {
        for slot in std::iter::once(&mut self.offhand)
            .chain(self.hotbar.iter_mut())
            .chain(self.main.iter_mut())
        {
            if let Some(stack) = slot {
                if stack.block == block {
                    if stack.count > 1 {
                        stack.count -= 1;
                    } else {
                        *slot = None;
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn count_item(&self, block: BlockId) -> u16 {
        self.hotbar
            .iter()
            .chain(self.main.iter())
            .chain(std::iter::once(&self.offhand))
            .filter_map(|slot| *slot)
            .filter(|stack| stack.block == block)
            .map(|stack| stack.count as u16)
            .sum()
    }

    pub fn handle_slot_left_click(slot: &mut Option<ItemStack>, held: &mut Option<ItemStack>) {
        match (slot.as_mut(), held.as_mut()) {
            (None, None) => {}
            (Some(_), None) => *held = slot.take(),
            (None, Some(_)) => *slot = held.take(),
            (Some(s), Some(h)) => {
                if s.block == h.block {
                    let space = 64 - s.count;
                    if h.count <= space {
                        s.count += h.count;
                        *held = None;
                    } else {
                        s.count = 64;
                        h.count -= space;
                    }
                } else {
                    std::mem::swap(slot, held);
                }
            }
        }
    }

    pub fn handle_slot_right_click(slot: &mut Option<ItemStack>, held: &mut Option<ItemStack>) {
        match (slot.as_mut(), held.as_mut()) {
            (None, None) => {}
            (Some(s), None) => {
                let take = (s.count + 1) / 2;
                let keep = s.count - take;
                *held = Some(ItemStack::new(s.block, take));
                if keep == 0 {
                    *slot = None;
                } else {
                    s.count = keep;
                }
            }
            (None, Some(h)) => {
                *slot = Some(ItemStack::new(h.block, 1));
                if h.count > 1 {
                    h.count -= 1;
                } else {
                    *held = None;
                }
            }
            (Some(s), Some(h)) => {
                if s.block == h.block && s.count < 64 {
                    s.count += 1;
                    if h.count > 1 {
                        h.count -= 1;
                    } else {
                        *held = None;
                    }
                } else if s.block != h.block {
                    std::mem::swap(slot, held);
                }
            }
        }
    }

    pub fn handle_slot_right_drag(slot: &mut Option<ItemStack>, held: &mut Option<ItemStack>) {
        if let Some(h) = held.as_mut() {
            if h.count > 0 {
                match slot.as_mut() {
                    None => {
                        *slot = Some(ItemStack::new(h.block, 1));
                        if h.count > 1 {
                            h.count -= 1;
                        } else {
                            *held = None;
                        }
                    }
                    Some(s) if s.block == h.block && s.count < 64 => {
                        s.count += 1;
                        if h.count > 1 {
                            h.count -= 1;
                        } else {
                            *held = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn add_item(&mut self, block: BlockId, mut count: u8) -> u8 {
        for slot in self.hotbar.iter_mut().chain(self.main.iter_mut()) {
            if let Some(stack) = slot {
                if stack.block == block && stack.count < 64 {
                    let space = 64 - stack.count;
                    if count <= space {
                        stack.count += count;
                        return 0;
                    }
                    stack.count = 64;
                    count -= space;
                }
            }
        }
        for slot in self.hotbar.iter_mut().chain(self.main.iter_mut()) {
            if slot.is_none() {
                *slot = Some(ItemStack::new(block, count));
                return 0;
            }
        }
        count
    }
}
