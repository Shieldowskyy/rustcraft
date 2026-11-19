//! Player vitals, survival/creative game mode, armor damage mitigation, food, and save snapshots.
//! Owns `Player::inventory`, combat timers, and state serialization (`SavedPlayerData`).

use crate::core::math::IVec3;
use crate::features::blocks::BlockId;

use super::inventory::{Inventory, ItemStack};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
}

pub struct Player {
    pub gamemode: GameMode,
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub max_hunger: f32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub starvation_timer: f32,
    pub heal_timer: f32,
    pub air: f32,
    pub max_air: f32,
    pub head_underwater: bool,
    pub fall_distance: f32,
    pub inventory: Inventory,
    pub selected_hotbar_slot: u8,
    pub creative_tab: usize,
    pub creative_scroll: usize,
    pub invulnerability_timer: f32,
    pub has_landed: bool,
    pub spawn_point: Option<IVec3>,
    pub fire_timer: f32,
    pub fire_damage_timer: f32,
    pub held_item_name_timer: f32,
    pub last_held_block: Option<BlockId>,
    pub last_selected_slot: u8,
    pub is_blocking: bool,
    pub bow_draw_time: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedPlayerData {
    pub health: f32,
    pub max_health: f32,
    pub hunger: f32,
    pub max_hunger: f32,
    pub saturation: f32,
    pub exhaustion: f32,
    pub air: f32,
    pub selected_hotbar_slot: u8,
    pub hotbar: [Option<ItemStack>; 10],
    pub main: [Option<ItemStack>; 27],
    pub armor: [Option<ItemStack>; 4],
    pub offhand: Option<ItemStack>,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        Self::new_with_gamemode(GameMode::Survival)
    }

    pub fn new_with_gamemode(gamemode: GameMode) -> Self {
        Self {
            gamemode,
            health: 20.0,
            max_health: 20.0,
            hunger: 20.0,
            max_hunger: 20.0,
            saturation: 5.0,
            exhaustion: 0.0,
            starvation_timer: 0.0,
            heal_timer: 0.0,
            air: 20.0,
            max_air: 20.0,
            head_underwater: false,
            fall_distance: 0.0,
            inventory: Inventory::new(),
            selected_hotbar_slot: 0,
            creative_tab: 0,
            creative_scroll: 0,
            invulnerability_timer: 3.0,
            has_landed: false,
            spawn_point: None,
            fire_timer: 0.0,
            fire_damage_timer: 0.0,
            held_item_name_timer: 0.0,
            last_held_block: None,
            last_selected_slot: 0,
            is_blocking: false,
            bow_draw_time: 0.0,
        }
    }

    /// Returns total defense points granted by equipped armor pieces (0.0 to 20.0).
    pub fn armor_defense_points(&self) -> f32 {
        let mut total = 0.0f32;
        for item in &self.inventory.armor {
            if let Some(stack) = item {
                total += stack.block.armor_defense();
            }
        }
        total
    }

    /// Computes damage multiplier after armor reduction (e.g. 20 points = 80% reduction -> 0.20 multiplier).
    pub fn damage_reduction_multiplier(&self) -> f32 {
        let defense = self.armor_defense_points();
        (1.0 - (defense * 0.04).min(0.80)).max(0.20)
    }

    /// Takes physical damage, returning `(damaged, blocked_by_shield, item_broke)`.
    pub fn take_physical_damage(&mut self, amount: f32, is_front_attack: bool) -> (bool, bool, bool) {
        if self.gamemode == GameMode::Creative || self.invulnerability_timer > 0.0 || self.is_dead() {
            return (false, false, false);
        }
        let mut item_broke = false;
        if self.is_blocking && is_front_attack {
            self.invulnerability_timer = 0.25;
            if let Some(shield) = &mut self.inventory.offhand {
                if shield.block.is_damageable() && shield.damage(1) {
                    self.inventory.offhand = None;
                    item_broke = true;
                }
            }
            return (false, true, item_broke);
        }
        let reduced = (amount * self.damage_reduction_multiplier()).max(1.0);
        let damaged = self.take_damage(reduced);
        if damaged {
            let armor_damage = (amount as u16 / 4).max(1);
            for slot in &mut self.inventory.armor {
                if let Some(armor_piece) = slot {
                    if armor_piece.block.is_damageable() && armor_piece.damage(armor_damage) {
                        *slot = None;
                        item_broke = true;
                    }
                }
            }
        }
        (damaged, false, item_broke)
    }

    /// Ticks the held item change detection and display timer.
    pub fn update_held_item_tracker(&mut self, dt: f32) {
        let current_held = self.inventory.hotbar[self.selected_hotbar_slot as usize].map(|s| s.block);
        if self.selected_hotbar_slot != self.last_selected_slot || current_held != self.last_held_block {
            self.last_selected_slot = self.selected_hotbar_slot;
            self.last_held_block = current_held;
            self.held_item_name_timer = if current_held.is_some() { 2.5 } else { 0.0 };
        }
        if self.held_item_name_timer > 0.0 {
            self.held_item_name_timer = (self.held_item_name_timer - dt).max(0.0);
        }
    }

    pub fn reset(&mut self, gamemode: GameMode) {
        *self = Self::new_with_gamemode(gamemode);
    }

    pub fn save_state(&self) -> SavedPlayerData {
        let (health, hunger, saturation, air) = if self.health <= 0.0 {
            (self.max_health, self.max_hunger, 5.0, self.max_air)
        } else {
            (self.health, self.hunger, self.saturation, self.air)
        };
        SavedPlayerData {
            health,
            max_health: self.max_health,
            hunger,
            max_hunger: self.max_hunger,
            saturation,
            exhaustion: self.exhaustion,
            air,
            selected_hotbar_slot: self.selected_hotbar_slot.min(9),
            hotbar: self.inventory.hotbar,
            main: self.inventory.main,
            armor: self.inventory.armor,
            offhand: self.inventory.offhand,
        }
    }

    pub fn apply_saved_state(&mut self, data: &SavedPlayerData) {
        self.health = if data.health <= 0.0 { data.max_health } else { data.health.min(data.max_health) };
        self.max_health = data.max_health;
        self.hunger = data.hunger.min(data.max_hunger);
        self.max_hunger = data.max_hunger;
        self.saturation = data.saturation;
        self.exhaustion = data.exhaustion;
        self.air = data.air.min(self.max_air);
        self.selected_hotbar_slot = data.selected_hotbar_slot.min(9);
        self.inventory.hotbar = data.hotbar;
        self.inventory.main = data.main;
        self.inventory.armor = data.armor;
        self.inventory.offhand = data.offhand;
        self.starvation_timer = 0.0;
        self.heal_timer = 0.0;
        self.invulnerability_timer = 1.0;
        self.fall_distance = 0.0;
        self.has_landed = false;
        self.is_blocking = false;
        self.bow_draw_time = 0.0;
    }

    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.gamemode == GameMode::Creative || self.invulnerability_timer > 0.0 || self.is_dead() {
            return false;
        }
        self.health -= amount;
        self.invulnerability_timer = 0.5;
        if self.health < 0.0 {
            self.health = 0.0;
        }
        true
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn can_eat(&self, food: BlockId) -> bool {
        food.is_food() && self.gamemode != GameMode::Creative && self.hunger < self.max_hunger
    }

    pub fn eat(&mut self, food: BlockId) -> bool {
        if let Some((points, sat)) = food.food_nutrition() {
            self.hunger = (self.hunger + points).min(self.max_hunger);
            self.saturation = (self.saturation + sat).min(self.hunger);
            true
        } else {
            false
        }
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }
}
