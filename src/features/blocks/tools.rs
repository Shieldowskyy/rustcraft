//! Tool, weapon, armor, food nutrition, and durability properties for `BlockId`.
//! Unifies pickaxe/axe/shovel/hoe/sword tiers using structured descriptors and `BlockSpec::mining_category`.

use super::constants::DEFAULT_BARE_HAND_DAMAGE;
use super::id::{ArmorSlot, BlockId};
use super::spec::MiningCategory;

#[derive(Copy, Clone)]
enum ToolKind {
    Pickaxe { speed: f32, can_obsidian: bool, tier2_speed: bool },
    Axe(f32),
    Shovel(f32),
    Hoe,
    Sword,
    Shears,
}

impl BlockId {
    const fn tool_kind(self) -> Option<ToolKind> {
        match self {
            BlockId::DiamondPickaxe => Some(ToolKind::Pickaxe { speed: 10.0, can_obsidian: true, tier2_speed: true }),
            BlockId::GoldPickaxe => Some(ToolKind::Pickaxe { speed: 8.0, can_obsidian: false, tier2_speed: true }),
            BlockId::IronPickaxe => Some(ToolKind::Pickaxe { speed: 7.0, can_obsidian: false, tier2_speed: true }),
            BlockId::StonePickaxe => Some(ToolKind::Pickaxe { speed: 4.5, can_obsidian: false, tier2_speed: false }),
            BlockId::WoodenPickaxe => Some(ToolKind::Pickaxe { speed: 2.5, can_obsidian: false, tier2_speed: false }),
            BlockId::DiamondAxe => Some(ToolKind::Axe(10.0)),
            BlockId::GoldAxe => Some(ToolKind::Axe(8.0)),
            BlockId::IronAxe => Some(ToolKind::Axe(7.0)),
            BlockId::StoneAxe => Some(ToolKind::Axe(4.5)),
            BlockId::WoodenAxe => Some(ToolKind::Axe(2.5)),
            BlockId::DiamondShovel => Some(ToolKind::Shovel(10.0)),
            BlockId::GoldShovel => Some(ToolKind::Shovel(8.0)),
            BlockId::IronShovel => Some(ToolKind::Shovel(7.0)),
            BlockId::StoneShovel => Some(ToolKind::Shovel(4.5)),
            BlockId::WoodenShovel => Some(ToolKind::Shovel(2.5)),
            BlockId::WoodenHoe | BlockId::StoneHoe | BlockId::IronHoe | BlockId::DiamondHoe | BlockId::GoldHoe => {
                Some(ToolKind::Hoe)
            }
            BlockId::WoodenSword | BlockId::StoneSword | BlockId::IronSword | BlockId::DiamondSword | BlockId::GoldSword => {
                Some(ToolKind::Sword)
            }
            BlockId::Shears => Some(ToolKind::Shears),
            _ => None,
        }
    }

    pub fn is_food(self) -> bool {
        match self {
            BlockId::Custom(id) => crate::engine::modding::is_custom_food(id),
            _ => self.food_nutrition().is_some(),
        }
    }

    /// Returns `(hunger_points, saturation_points)`.
    pub fn food_nutrition(self) -> Option<(f32, f32)> {
        match self {
            BlockId::Custom(id) => crate::engine::modding::get_custom_nutrition(id),
            BlockId::RawPorkchop => Some((3.0, 1.8)),
            BlockId::CookedPorkchop => Some((8.0, 12.8)),
            BlockId::RawBeef => Some((3.0, 1.8)),
            BlockId::CookedBeef => Some((8.0, 12.8)),
            BlockId::Apple => Some((4.0, 2.4)),
            BlockId::RawChicken => Some((2.0, 1.2)),
            BlockId::CookedChicken => Some((6.0, 7.2)),
            BlockId::Bread => Some((5.0, 6.0)),
            BlockId::Carrot => Some((3.0, 3.6)),
            BlockId::Potato => Some((1.0, 0.6)),
            BlockId::BakedPotato => Some((5.0, 6.0)),
            BlockId::RawCod => Some((2.0, 0.4)),
            BlockId::CookedCod => Some((5.0, 6.0)),
            BlockId::RawSalmon => Some((2.0, 0.4)),
            BlockId::CookedSalmon => Some((6.0, 9.6)),
            BlockId::Pufferfish => Some((1.0, 0.2)),
            BlockId::TropicalFish => Some((1.0, 0.2)),
            BlockId::DriedKelp => Some((1.0, 0.6)),
            BlockId::SpiderEye => Some((2.0, 3.2)),
            _ => None,
        }
    }

    pub fn is_meat(self) -> bool {
        matches!(
            self,
            BlockId::RawPorkchop
                | BlockId::CookedPorkchop
                | BlockId::RawBeef
                | BlockId::CookedBeef
                | BlockId::RawChicken
                | BlockId::CookedChicken
                | BlockId::RawCod
                | BlockId::CookedCod
                | BlockId::RawSalmon
                | BlockId::CookedSalmon
        )
    }

    pub fn is_fish(self) -> bool {
        matches!(
            self,
            BlockId::RawCod | BlockId::CookedCod | BlockId::RawSalmon | BlockId::CookedSalmon | BlockId::Pufferfish | BlockId::TropicalFish
        )
    }

    pub fn is_hoe(self) -> bool {
        matches!(self.tool_kind(), Some(ToolKind::Hoe))
    }

    pub fn is_sword(self) -> bool {
        matches!(self.tool_kind(), Some(ToolKind::Sword))
    }

    pub fn is_tool(self) -> bool {
        self.tool_kind().is_some() || matches!(self, BlockId::Stick | BlockId::FishingRod)
    }

    pub fn attack_damage(self) -> f32 {
        match self {
            BlockId::DiamondSword => 13.0,
            BlockId::DiamondAxe => 11.0,
            BlockId::IronSword => 10.0,
            BlockId::IronAxe => 9.0,
            BlockId::StoneSword | BlockId::StoneAxe | BlockId::DiamondPickaxe => 8.0,
            BlockId::GoldSword | BlockId::WoodenAxe => 7.0,
            BlockId::IronPickaxe | BlockId::WoodenSword | BlockId::GoldAxe | BlockId::DiamondShovel => 6.0,
            BlockId::StonePickaxe => 5.0,
            BlockId::GoldPickaxe => 4.5,
            BlockId::DiamondHoe | BlockId::IronShovel | BlockId::WoodenPickaxe => 4.0,
            BlockId::StoneShovel => 3.5,
            BlockId::IronHoe | BlockId::WoodenShovel | BlockId::GoldShovel | BlockId::Shears => 3.0,
            BlockId::StoneHoe | BlockId::Stick => 2.5,
            BlockId::WoodenHoe | BlockId::GoldHoe => 2.0,
            _ => DEFAULT_BARE_HAND_DAMAGE,
        }
    }

    pub fn mining_speed_multiplier(tool: Option<BlockId>, target: BlockId) -> f32 {
        let Some(t) = tool else { return 1.0 };
        let Some(kind) = t.tool_kind() else { return 1.0 };
        let spec = target.spec();
        match kind {
            ToolKind::Pickaxe { speed, can_obsidian, tier2_speed } => match spec.mining_category {
                MiningCategory::PickaxeTier0 => speed,
                MiningCategory::PickaxeTier2 if tier2_speed => speed,
                MiningCategory::PickaxeObsidian if can_obsidian => speed,
                _ => 1.0,
            },
            ToolKind::Axe(speed) => {
                if spec.mining_category == MiningCategory::AxeTarget { speed } else { 1.0 }
            }
            ToolKind::Shovel(speed) => {
                if spec.mining_category == MiningCategory::ShovelTarget { speed } else { 1.0 }
            }
            ToolKind::Hoe => {
                if spec.leaves || spec.mining_category == MiningCategory::HoeCropTarget
                    || matches!(target, BlockId::Farmland | BlockId::FarmlandWet)
                {
                    6.0
                } else {
                    1.0
                }
            }
            ToolKind::Sword => {
                if spec.leaves { 3.0 } else if target.is_wool() { 2.0 } else { 1.0 }
            }
            ToolKind::Shears => {
                if target.is_wool() || spec.leaves { 10.0 } else { 1.0 }
            }
        }
    }

    pub fn can_harvest_with_tool(self, tool: Option<BlockId>) -> bool {
        let Some(required_tier) = self.spec().required_pickaxe_tier else {
            return true;
        };
        let Some(tool) = tool else { return false };
        let tool_tier = match tool {
            BlockId::WoodenPickaxe | BlockId::GoldPickaxe => 0,
            BlockId::StonePickaxe => 1,
            BlockId::IronPickaxe => 2,
            BlockId::DiamondPickaxe => 3,
            _ => return false,
        };
        tool_tier >= required_tier
    }

    pub fn break_time_with_tool(self, tool: Option<BlockId>) -> f32 {
        let base = self.break_time();
        if base == 0.0 || base.is_infinite() {
            return base;
        }
        let mult = if !self.can_harvest_with_tool(tool) {
            1.0
        } else {
            Self::mining_speed_multiplier(tool, self)
        };
        base / mult
    }

    #[inline(always)]
    pub fn armor_slot(self) -> Option<ArmorSlot> {
        match self {
            BlockId::LeatherHelmet | BlockId::IronHelmet | BlockId::GoldHelmet | BlockId::DiamondHelmet => {
                Some(ArmorSlot::Helmet)
            }
            BlockId::LeatherChestplate | BlockId::IronChestplate | BlockId::GoldChestplate | BlockId::DiamondChestplate => {
                Some(ArmorSlot::Chestplate)
            }
            BlockId::LeatherLeggings | BlockId::IronLeggings | BlockId::GoldLeggings | BlockId::DiamondLeggings => {
                Some(ArmorSlot::Leggings)
            }
            BlockId::LeatherBoots | BlockId::IronBoots | BlockId::GoldBoots | BlockId::DiamondBoots => {
                Some(ArmorSlot::Boots)
            }
            _ => None,
        }
    }

    #[inline(always)]
    pub fn armor_defense(self) -> f32 {
        match self {
            BlockId::LeatherHelmet => 1.0,
            BlockId::LeatherChestplate => 3.0,
            BlockId::LeatherLeggings => 2.0,
            BlockId::LeatherBoots => 1.0,
            BlockId::GoldHelmet => 2.0,
            BlockId::GoldChestplate => 5.0,
            BlockId::GoldLeggings => 3.0,
            BlockId::GoldBoots => 1.0,
            BlockId::IronHelmet => 2.0,
            BlockId::IronChestplate => 6.0,
            BlockId::IronLeggings => 5.0,
            BlockId::IronBoots => 2.0,
            BlockId::DiamondHelmet => 3.0,
            BlockId::DiamondChestplate => 8.0,
            BlockId::DiamondLeggings => 6.0,
            BlockId::DiamondBoots => 3.0,
            _ => 0.0,
        }
    }

    #[inline(always)]
    pub fn is_armor(self) -> bool { self.armor_slot().is_some() }

    #[inline(always)]
    pub fn is_shield(self) -> bool { matches!(self, BlockId::Shield) }

    #[inline(always)]
    pub fn is_bow(self) -> bool { matches!(self, BlockId::Bow) }

    #[inline(always)]
    pub fn is_arrow(self) -> bool { matches!(self, BlockId::Arrow) }

    /// Maximum durability of tools, weapons, armor, bows, and shields; `0` if non-damageable.
    pub const fn max_durability(self) -> u16 {
        match self {
            BlockId::WoodenPickaxe | BlockId::WoodenAxe | BlockId::WoodenShovel | BlockId::WoodenSword | BlockId::WoodenHoe => 59,
            BlockId::StonePickaxe | BlockId::StoneAxe | BlockId::StoneShovel | BlockId::StoneSword | BlockId::StoneHoe => 131,
            BlockId::IronPickaxe | BlockId::IronAxe | BlockId::IronShovel | BlockId::IronSword | BlockId::IronHoe => 250,
            BlockId::GoldPickaxe | BlockId::GoldAxe | BlockId::GoldShovel | BlockId::GoldSword | BlockId::GoldHoe => 32,
            BlockId::DiamondPickaxe | BlockId::DiamondAxe | BlockId::DiamondShovel | BlockId::DiamondSword | BlockId::DiamondHoe => 1561,
            BlockId::Shears => 238,
            BlockId::FishingRod => 64,
            BlockId::Bow => 384,
            BlockId::Shield => 336,
            BlockId::LeatherHelmet => 55,
            BlockId::LeatherChestplate => 80,
            BlockId::LeatherLeggings => 75,
            BlockId::LeatherBoots => 65,
            BlockId::IronHelmet => 165,
            BlockId::IronChestplate => 240,
            BlockId::IronLeggings => 225,
            BlockId::IronBoots => 195,
            BlockId::GoldHelmet => 77,
            BlockId::GoldChestplate => 112,
            BlockId::GoldLeggings => 105,
            BlockId::GoldBoots => 91,
            BlockId::DiamondHelmet => 363,
            BlockId::DiamondChestplate => 528,
            BlockId::DiamondLeggings => 495,
            BlockId::DiamondBoots => 429,
            _ => 0,
        }
    }

    #[inline(always)]
    pub const fn is_damageable(self) -> bool { self.max_durability() > 0 }
}
