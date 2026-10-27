use crate::gameplay::inventory::Inventory;
use crate::world::block::BlockId;

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
    pub selected_hotbar_slot: u8, // 0 to 9
    pub creative_tab: usize, // 0: Budulec, 1: Narzedzia, 2: Mechanizmy, 3: Rolnictwo, 4: Wszystkie, 5: Ekwipunek
    pub creative_scroll: usize,
    pub invulnerability_timer: f32,
    pub has_landed: bool,
    pub spawn_point: Option<crate::core::math::IVec3>,
    pub fire_timer: f32,
    pub fire_damage_timer: f32,
    pub held_item_name_timer: f32,
    pub last_held_block: Option<crate::world::block::BlockId>,
    pub last_selected_slot: u8,
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
    pub hotbar: [Option<crate::gameplay::inventory::ItemStack>; 10],
    pub main: [Option<crate::gameplay::inventory::ItemStack>; 27],
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
        }
    }

    /// Ticks the held item change detection and display timer.
    pub fn update_held_item_tracker(&mut self, dt: f32) {
        let current_held = self.inventory.hotbar[self.selected_hotbar_slot as usize].map(|s| s.block);
        if self.selected_hotbar_slot != self.last_selected_slot || current_held != self.last_held_block {
            self.last_selected_slot = self.selected_hotbar_slot;
            self.last_held_block = current_held;
            if current_held.is_some() {
                self.held_item_name_timer = 2.5;
            } else {
                self.held_item_name_timer = 0.0;
            }
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
        self.starvation_timer = 0.0;
        self.heal_timer = 0.0;
        self.invulnerability_timer = 1.0;
        self.fall_distance = 0.0;
        self.has_landed = false;
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
        self.health += amount;
        if self.health > self.max_health {
            self.health = self.max_health;
        }
    }

    pub fn can_eat(&self, food: BlockId) -> bool {
        if !food.is_food() {
            return false;
        }
        if self.gamemode == GameMode::Creative {
            return false;
        }
        self.hunger < self.max_hunger
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_air_initialization() {
        let player = Player::new();
        assert_eq!(player.air, 20.0);
        assert_eq!(player.max_air, 20.0);
        assert!(!player.head_underwater);
    }

    #[test]
    fn test_player_hunger_and_eating() {
        let mut player = Player::new();
        player.gamemode = GameMode::Survival;
        player.hunger = 10.0;
        player.saturation = 0.0;

        assert!(player.can_eat(BlockId::CookedBeef));
        assert!(!player.can_eat(BlockId::Stone));

        let eaten = player.eat(BlockId::CookedBeef);
        assert!(eaten);
        assert_eq!(player.hunger, 18.0);
        assert!(player.saturation > 0.0);

        player.eat(BlockId::Apple);
        assert_eq!(player.hunger, 20.0);
        assert!(!player.can_eat(BlockId::Apple));

        let mut creative_player = Player::new();
        creative_player.gamemode = GameMode::Creative;
        creative_player.hunger = 10.0;
        assert!(!creative_player.can_eat(BlockId::CookedBeef));
        assert!(!creative_player.can_eat(BlockId::Apple));
    }

    #[test]
    fn test_player_save_and_apply_state() {
        let mut player = Player::new();
        player.health = 14.0;
        player.hunger = 12.0;
        player.saturation = 3.5;
        player.exhaustion = 1.2;
        player.air = 15.0;
        player.selected_hotbar_slot = 3;
        player.inventory.hotbar[3] = Some(crate::gameplay::inventory::ItemStack {
            block: BlockId::IronPickaxe,
            count: 1,
        });

        let saved = player.save_state();
        assert_eq!(saved.health, 14.0);
        assert_eq!(saved.hunger, 12.0);
        assert_eq!(saved.selected_hotbar_slot, 3);
        assert_eq!(saved.hotbar[3].unwrap().block, BlockId::IronPickaxe);

        // Reset player to another world
        player.reset(GameMode::Creative);
        assert_eq!(player.health, 20.0);
        assert_eq!(player.hunger, 20.0);
        assert_eq!(player.selected_hotbar_slot, 0);
        assert!(player.inventory.hotbar[3].is_none());

        // Restore saved player state
        player.apply_saved_state(&saved);
        assert_eq!(player.health, 14.0);
        assert_eq!(player.hunger, 12.0);
        assert_eq!(player.selected_hotbar_slot, 3);
        assert_eq!(player.inventory.hotbar[3].unwrap().block, BlockId::IronPickaxe);
    }

    #[test]
    fn test_player_fire_state() {
        let mut player = Player::new();
        assert_eq!(player.fire_timer, 0.0);
        assert_eq!(player.fire_damage_timer, 0.0);

        player.fire_timer = 5.0;
        player.reset(GameMode::Survival);
        assert_eq!(player.fire_timer, 0.0);
    }
}


