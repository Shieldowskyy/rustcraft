use crate::gameplay::inventory::Inventory;
use crate::world::block::BlockId;

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub creative_tab: usize, // 0: Budulec, 1: Narzedzia, 2: Wszystkie, 3: Ekwipunek
    pub invulnerability_timer: f32,
    pub has_landed: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            gamemode: GameMode::Creative,
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
            invulnerability_timer: 3.0,
            has_landed: false,
        }
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
            return true;
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
    }
}

