use crate::gameplay::inventory::Inventory;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
}

pub struct Player {
    pub gamemode: GameMode,
    pub health: f32,
    pub max_health: f32,
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

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }
}
