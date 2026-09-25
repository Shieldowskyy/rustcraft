use crate::world::block::BlockId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemStack {
    pub block: BlockId,
    pub count: u8,
}

pub struct Inventory {
    pub hotbar: [Option<ItemStack>; 10],
    pub main: [Option<ItemStack>; 27],
    pub crafting_2x2: [Option<ItemStack>; 4],
    pub crafting_3x3: [Option<ItemStack>; 9],
    pub held_item: Option<ItemStack>,
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            hotbar: [None; 10],
            main: [None; 27],
            crafting_2x2: [None; 4],
            crafting_3x3: [None; 9],
            held_item: None,
        }
    }

    pub fn return_crafting_items(&mut self) {
        let mut to_return = Vec::new();
        // Return 2x2 crafting items
        for slot in &mut self.crafting_2x2 {
            if let Some(stack) = slot.take() {
                to_return.push(stack);
            }
        }
        // Return 3x3 crafting items
        for slot in &mut self.crafting_3x3 {
            if let Some(stack) = slot.take() {
                to_return.push(stack);
            }
        }
        // Return held item
        if let Some(stack) = self.held_item.take() {
            to_return.push(stack);
        }

        for stack in to_return {
            let rem = self.add_item(stack.block, stack.count);
            if rem > 0 {
                // If inventory is completely full, put remaining back in held_item
                self.held_item = Some(ItemStack { block: stack.block, count: rem });
            }
        }
    }

    pub fn handle_slot_left_click(slot: &mut Option<ItemStack>, held: &mut Option<ItemStack>) {
        match (slot.as_mut(), held.as_mut()) {
            (None, None) => {}
            (Some(_), None) => {
                *held = slot.take();
            }
            (None, Some(_)) => {
                *slot = held.take();
            }
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
                // Split half into hand
                let take = (s.count + 1) / 2;
                let keep = s.count - take;
                *held = Some(ItemStack { block: s.block, count: take });
                if keep == 0 {
                    *slot = None;
                } else {
                    s.count = keep;
                }
            }
            (None, Some(h)) => {
                // Place 1 item
                *slot = Some(ItemStack { block: h.block, count: 1 });
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
                        *slot = Some(ItemStack { block: h.block, count: 1 });
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
        // Try to stack in hotbar first
        for slot in &mut self.hotbar {
            if let Some(stack) = slot {
                if stack.block == block && stack.count < 64 {
                    let space = 64 - stack.count;
                    if count <= space {
                        stack.count += count;
                        return 0;
                    } else {
                        stack.count = 64;
                        count -= space;
                    }
                }
            }
        }
        // Try to stack in main inventory
        for slot in &mut self.main {
            if let Some(stack) = slot {
                if stack.block == block && stack.count < 64 {
                    let space = 64 - stack.count;
                    if count <= space {
                        stack.count += count;
                        return 0;
                    } else {
                        stack.count = 64;
                        count -= space;
                    }
                }
            }
        }
        // Find empty slot in hotbar
        for slot in &mut self.hotbar {
            if slot.is_none() {
                *slot = Some(ItemStack { block, count });
                return 0;
            }
        }
        // Find empty slot in main inventory
        for slot in &mut self.main {
            if slot.is_none() {
                *slot = Some(ItemStack { block, count });
                return 0;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_add_and_stack() {
        let mut inv = Inventory::new();
        assert_eq!(inv.add_item(BlockId::Dirt, 30), 0);
        assert_eq!(inv.hotbar[0], Some(ItemStack { block: BlockId::Dirt, count: 30 }));

        assert_eq!(inv.add_item(BlockId::Dirt, 40), 0);
        assert_eq!(inv.hotbar[0], Some(ItemStack { block: BlockId::Dirt, count: 64 }));
        assert_eq!(inv.hotbar[1], Some(ItemStack { block: BlockId::Dirt, count: 6 }));
    }

    #[test]
    fn test_slot_left_click_swap_and_stack() {
        let mut slot = Some(ItemStack { block: BlockId::Stone, count: 10 });
        let mut held = Some(ItemStack { block: BlockId::Stone, count: 20 });
        
        Inventory::handle_slot_left_click(&mut slot, &mut held);
        assert_eq!(slot, Some(ItemStack { block: BlockId::Stone, count: 30 }));
        assert_eq!(held, None);

        // Pick up
        Inventory::handle_slot_left_click(&mut slot, &mut held);
        assert_eq!(slot, None);
        assert_eq!(held, Some(ItemStack { block: BlockId::Stone, count: 30 }));
    }

    #[test]
    fn test_slot_right_click_split_and_drop_one() {
        let mut slot = Some(ItemStack { block: BlockId::Wood, count: 10 });
        let mut held = None;

        // Split in half
        Inventory::handle_slot_right_click(&mut slot, &mut held);
        assert_eq!(slot, Some(ItemStack { block: BlockId::Wood, count: 5 }));
        assert_eq!(held, Some(ItemStack { block: BlockId::Wood, count: 5 }));

        // Place one into empty slot
        let mut empty_slot = None;
        Inventory::handle_slot_right_click(&mut empty_slot, &mut held);
        assert_eq!(empty_slot, Some(ItemStack { block: BlockId::Wood, count: 1 }));
        assert_eq!(held, Some(ItemStack { block: BlockId::Wood, count: 4 }));
    }

    #[test]
    fn test_return_crafting_items() {
        let mut inv = Inventory::new();
        inv.crafting_2x2[0] = Some(ItemStack { block: BlockId::Wood, count: 2 });
        inv.crafting_3x3[4] = Some(ItemStack { block: BlockId::Stick, count: 5 });
        inv.held_item = Some(ItemStack { block: BlockId::IronPickaxe, count: 1 });

        inv.return_crafting_items();
        assert!(inv.crafting_2x2.iter().all(|s| s.is_none()));
        assert!(inv.crafting_3x3.iter().all(|s| s.is_none()));
        assert!(inv.held_item.is_none());
        assert_eq!(inv.hotbar[0], Some(ItemStack { block: BlockId::Wood, count: 2 }));
        assert_eq!(inv.hotbar[1], Some(ItemStack { block: BlockId::Stick, count: 5 }));
        assert_eq!(inv.hotbar[2], Some(ItemStack { block: BlockId::IronPickaxe, count: 1 }));
    }

    #[test]
    fn test_slot_right_drag_distribution() {
        let mut held = Some(ItemStack { block: BlockId::Planks, count: 4 });
        let mut slot1 = None;
        let mut slot2 = None;
        let mut slot3 = Some(ItemStack { block: BlockId::Planks, count: 2 });
        let mut slot4 = Some(ItemStack { block: BlockId::Stone, count: 5 }); // Different item

        Inventory::handle_slot_right_drag(&mut slot1, &mut held);
        assert_eq!(slot1, Some(ItemStack { block: BlockId::Planks, count: 1 }));
        assert_eq!(held, Some(ItemStack { block: BlockId::Planks, count: 3 }));

        Inventory::handle_slot_right_drag(&mut slot2, &mut held);
        assert_eq!(slot2, Some(ItemStack { block: BlockId::Planks, count: 1 }));
        assert_eq!(held, Some(ItemStack { block: BlockId::Planks, count: 2 }));

        Inventory::handle_slot_right_drag(&mut slot3, &mut held);
        assert_eq!(slot3, Some(ItemStack { block: BlockId::Planks, count: 3 }));
        assert_eq!(held, Some(ItemStack { block: BlockId::Planks, count: 1 }));

        // Incompatible slot should not take items or modify slot
        Inventory::handle_slot_right_drag(&mut slot4, &mut held);
        assert_eq!(slot4, Some(ItemStack { block: BlockId::Stone, count: 5 }));
        assert_eq!(held, Some(ItemStack { block: BlockId::Planks, count: 1 }));
    }
}
