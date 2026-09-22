use crate::world::block::BlockId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemStack {
    pub block: BlockId,
    pub count: u8,
}

pub struct Inventory {
    pub hotbar: [Option<ItemStack>; 10],
    pub main: [Option<ItemStack>; 27],
}

impl Inventory {
    pub fn new() -> Self {
        Self {
            hotbar: [None; 10],
            main: [None; 27],
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
}
