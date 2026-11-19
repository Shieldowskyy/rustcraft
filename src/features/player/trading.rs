//! Villager trading professions, localized offer catalogs, affordability checks, and trade execution.
//! Reads/consumes items from `Player::inventory` and updates trade use counters.

use crate::core::i18n::{current_language, Language};
use crate::features::blocks::BlockId;

use super::inventory::ItemStack;
use super::state::Player;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VillagerProfession {
    Farmer,
    Librarian,
    Blacksmith,
    Cleric,
    Butcher,
    Nitwit,
}

impl VillagerProfession {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Farmer,
            1 => Self::Librarian,
            2 => Self::Blacksmith,
            3 => Self::Cleric,
            4 => Self::Butcher,
            _ => Self::Nitwit,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            Self::Farmer => 0,
            Self::Librarian => 1,
            Self::Blacksmith => 2,
            Self::Cleric => 3,
            Self::Butcher => 4,
            Self::Nitwit => 5,
        }
    }

    pub fn name_pl(self) -> &'static str {
        match self {
            Self::Farmer => "Rolnik",
            Self::Librarian => "Bibliotekarz",
            Self::Blacksmith => "Kowal",
            Self::Cleric => "Kaplan",
            Self::Butcher => "Rzeznik",
            Self::Nitwit => "Mieszkaniec",
        }
    }

    pub fn name_en(self) -> &'static str {
        match self {
            Self::Farmer => "Farmer",
            Self::Librarian => "Librarian",
            Self::Blacksmith => "Weaponsmith",
            Self::Cleric => "Cleric",
            Self::Butcher => "Butcher",
            Self::Nitwit => "Nitwit",
        }
    }

    pub fn display_name(self) -> &'static str {
        match current_language() {
            Language::English => self.name_en(),
            Language::Polish => self.name_pl(),
        }
    }

    pub fn default_trades(self) -> Vec<TradeOffer> {
        match self {
            Self::Farmer => vec![
                TradeOffer::new(BlockId::Wheat, 20, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Carrot, 16, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Potato, 15, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::Bread, 4),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::Apple, 3),
                TradeOffer::new(BlockId::Emerald, 2, BlockId::CookedPorkchop, 3),
                TradeOffer::new(BlockId::Emerald, 3, BlockId::BakedPotato, 6),
            ],
            Self::Librarian => vec![
                TradeOffer::new(BlockId::Wheat, 16, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::Stick, 16),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::Glass, 4),
                TradeOffer::new(BlockId::Emerald, 3, BlockId::Torch, 16),
                TradeOffer::new(BlockId::Emerald, 8, BlockId::Diamond, 1),
            ],
            Self::Blacksmith => vec![
                TradeOffer::new(BlockId::CoalOre, 15, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::IronIngot, 4, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::GoldIngot, 3, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Emerald, 5, BlockId::IronSword, 1),
                TradeOffer::new(BlockId::Emerald, 7, BlockId::IronPickaxe, 1),
                TradeOffer::new(BlockId::Emerald, 5, BlockId::IronAxe, 1),
                TradeOffer::new(BlockId::Emerald, 12, BlockId::DiamondSword, 1),
                TradeOffer::new(BlockId::Emerald, 15, BlockId::DiamondPickaxe, 1),
            ],
            Self::Cleric => vec![
                TradeOffer::new(BlockId::Bone, 16, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::RustStone, 10, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::BoneMeal, 6),
                TradeOffer::new(BlockId::Emerald, 2, BlockId::Apple, 4),
                TradeOffer::new(BlockId::Emerald, 4, BlockId::RustStoneBlock, 1),
            ],
            Self::Butcher => vec![
                TradeOffer::new(BlockId::RawBeef, 14, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::RawPorkchop, 14, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::RawChicken, 14, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::RawCod, 12, BlockId::Emerald, 1),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::CookedBeef, 4),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::CookedPorkchop, 4),
                TradeOffer::new(BlockId::Emerald, 1, BlockId::CookedChicken, 4),
            ],
            Self::Nitwit => vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TradeOffer {
    pub cost_a: ItemStack,
    pub cost_b: Option<ItemStack>,
    pub result: ItemStack,
    pub uses: u32,
    pub max_uses: u32,
}

impl TradeOffer {
    pub fn new(cost_block: BlockId, cost_count: u8, result_block: BlockId, result_count: u8) -> Self {
        Self {
            cost_a: ItemStack::new(cost_block, cost_count),
            cost_b: None,
            result: ItemStack::new(result_block, result_count),
            uses: 0,
            max_uses: 16,
        }
    }

    pub fn with_secondary_cost(mut self, cost_block: BlockId, cost_count: u8) -> Self {
        self.cost_b = Some(ItemStack::new(cost_block, cost_count));
        self
    }

    pub fn is_maxed_out(&self) -> bool {
        self.uses >= self.max_uses
    }

    pub fn player_can_afford(&self, player: &Player) -> bool {
        if self.is_maxed_out() {
            return false;
        }
        let has_a = count_item_in_player(player, self.cost_a.block) >= (self.cost_a.count as usize);
        let has_b = self
            .cost_b
            .map_or(true, |b| count_item_in_player(player, b.block) >= (b.count as usize));
        has_a && has_b
    }

    pub fn execute(&mut self, player: &mut Player) -> bool {
        if !self.player_can_afford(player) {
            return false;
        }
        consume_item_from_player(player, self.cost_a.block, self.cost_a.count);
        if let Some(cost_b) = self.cost_b {
            consume_item_from_player(player, cost_b.block, cost_b.count);
        }

        let remaining = player.inventory.add_item(self.result.block, self.result.count);
        if remaining > 0 {
            for slot in &mut player.inventory.hotbar {
                if slot.is_none() {
                    *slot = Some(ItemStack::new(self.result.block, remaining));
                    break;
                }
            }
        }

        self.uses = self.uses.saturating_add(1);
        true
    }
}

pub fn count_item_in_player(player: &Player, block: BlockId) -> usize {
    player
        .inventory
        .main
        .iter()
        .chain(player.inventory.hotbar.iter())
        .flatten()
        .filter(|stack| stack.block == block)
        .map(|stack| stack.count as usize)
        .sum()
}

pub fn consume_item_from_player(player: &mut Player, block: BlockId, mut needed: u8) {
    let consume_from = |slots: &mut [Option<ItemStack>], rem: &mut u8| {
        for stack_opt in slots {
            if let Some(stack) = stack_opt {
                if stack.block == block {
                    if stack.count <= *rem {
                        *rem -= stack.count;
                        *stack_opt = None;
                    } else {
                        stack.count -= *rem;
                        *rem = 0;
                        return;
                    }
                }
            }
            if *rem == 0 {
                return;
            }
        }
    };
    consume_from(&mut player.inventory.main, &mut needed);
    if needed > 0 {
        consume_from(&mut player.inventory.hotbar, &mut needed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_villager_professions_and_names() {
        assert_eq!(VillagerProfession::from_u8(0), VillagerProfession::Farmer);
        assert_eq!(VillagerProfession::from_u8(1), VillagerProfession::Librarian);
        assert_eq!(VillagerProfession::from_u8(2), VillagerProfession::Blacksmith);
        assert_eq!(VillagerProfession::from_u8(3), VillagerProfession::Cleric);
        assert_eq!(VillagerProfession::from_u8(4), VillagerProfession::Butcher);
        assert_eq!(VillagerProfession::from_u8(5), VillagerProfession::Nitwit);

        assert_eq!(VillagerProfession::Farmer.to_u8(), 0);
        assert_eq!(VillagerProfession::Farmer.name_pl(), "Rolnik");
        assert_eq!(VillagerProfession::Farmer.name_en(), "Farmer");
        assert_eq!(VillagerProfession::Blacksmith.name_pl(), "Kowal");
        assert_eq!(VillagerProfession::Blacksmith.name_en(), "Weaponsmith");
    }

    #[test]
    fn test_villager_default_trades() {
        let farmer_trades = VillagerProfession::Farmer.default_trades();
        assert!(!farmer_trades.is_empty());
        assert_eq!(farmer_trades[0].cost_a.block, BlockId::Wheat);
        assert_eq!(farmer_trades[0].result.block, BlockId::Emerald);

        let smith_trades = VillagerProfession::Blacksmith.default_trades();
        assert!(!smith_trades.is_empty());
        assert_eq!(smith_trades[0].cost_a.block, BlockId::CoalOre);
        assert_eq!(smith_trades[0].result.block, BlockId::Emerald);
    }

    #[test]
    fn test_trade_afford_and_execute() {
        let mut player = Player::new();
        let mut offer = TradeOffer::new(BlockId::Wheat, 20, BlockId::Emerald, 1);

        assert!(!offer.player_can_afford(&player));
        assert!(!offer.execute(&mut player));

        player.inventory.main[0] = Some(ItemStack::new(BlockId::Wheat, 20));
        assert!(offer.player_can_afford(&player));

        assert!(offer.execute(&mut player));
        assert_eq!(offer.uses, 1);
        assert_eq!(count_item_in_player(&player, BlockId::Wheat), 0);
        assert_eq!(count_item_in_player(&player, BlockId::Emerald), 1);
    }

    #[test]
    fn test_trade_uses_and_max_out() {
        let mut player = Player::new();
        let mut offer = TradeOffer::new(BlockId::Emerald, 1, BlockId::Bread, 6);
        offer.max_uses = 2;

        player.inventory.main[0] = Some(ItemStack::new(BlockId::Emerald, 10));

        assert!(offer.execute(&mut player));
        assert_eq!(offer.uses, 1);
        assert!(!offer.is_maxed_out());

        assert!(offer.execute(&mut player));
        assert_eq!(offer.uses, 2);
        assert!(offer.is_maxed_out());

        assert!(!offer.player_can_afford(&player));
        assert!(!offer.execute(&mut player));
    }
}
