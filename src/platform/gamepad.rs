use gilrs::{Axis, Button, Event, EventType, Gilrs};

/// Stan kontrolera / gamepada z obsługą martwej strefy (deadzone) oraz detekcją
/// pojedynczych kliknięć (just-pressed) do nawigacji po menu i ekwipunku.
pub struct GamepadManager {
    gilrs: Option<Gilrs>,
    pub state: GamepadState,
    prev_state: GamepadState,
    deadzone: f32,
    stick_repeat_timer: f32,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct GamepadState {
    pub is_connected: bool,

    // Osie analogowe (-1.0 .. 1.0 lub 0.0 .. 1.0)
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,

    // Przyciski cyfrowe (wciskanie ciągłe)
    pub btn_south: bool,   // A / Cross (Skok / Wybór w UI)
    pub btn_east: bool,    // B / Circle (Kucanie / Wstecz w UI)
    pub btn_west: bool,    // X / Square (Wyrzucanie przedmiotu)
    pub btn_north: bool,   // Y / Triangle (Ekwipunek)
    pub btn_lb: bool,      // L1 / Left Bumper (Poprzedni slot hotbara)
    pub btn_rb: bool,      // R1 / Right Bumper (Następny slot hotbara)
    pub btn_l3: bool,      // Left Stick Click (Bieg / Sprint)
    pub btn_r3: bool,      // Right Stick Click (Tryb lotu)
    pub btn_start: bool,   // Start / Menu (Pauza)
    pub btn_select: bool,  // Select / Back (Zmiana trybu gry / Zapis)
    pub dpad_up: bool,     // D-Pad Góra
    pub dpad_down: bool,   // D-Pad Dół
    pub dpad_left: bool,   // D-Pad Lewo
    pub dpad_right: bool,  // D-Pad Prawo
}

#[derive(Clone, Copy, Default, Debug)]
pub struct GamepadEvents {
    pub south_just_pressed: bool,
    pub east_just_pressed: bool,
    pub west_just_pressed: bool,
    pub north_just_pressed: bool,
    pub lb_just_pressed: bool,
    pub rb_just_pressed: bool,
    pub l3_just_pressed: bool,
    pub r3_just_pressed: bool,
    pub start_just_pressed: bool,
    pub select_just_pressed: bool,
    pub dpad_up_just_pressed: bool,
    pub dpad_down_just_pressed: bool,
    pub dpad_left_just_pressed: bool,
    pub dpad_right_just_pressed: bool,
    pub right_trigger_just_pressed: bool,
    pub left_trigger_just_pressed: bool,
}

impl GamepadManager {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("Brak obsługi lub błąd inicjalizacji gamepadów: {e}");
                None
            }
        };

        Self {
            gilrs,
            state: GamepadState::default(),
            prev_state: GamepadState::default(),
            deadzone: 0.16,
            stick_repeat_timer: 0.0,
        }
    }

    /// Aktualizuje stan pada na podstawie zdarzeń systemowych z biblioteki gilrs.
    pub fn update(&mut self, dt: f32) -> GamepadEvents {
        self.prev_state = self.state;

        if let Some(gilrs) = &mut self.gilrs {
            while let Some(Event { id: _, event, .. }) = gilrs.next_event() {
                match event {
                    EventType::Connected => {
                        self.state.is_connected = true;
                    }
                    EventType::Disconnected => {
                        self.state = GamepadState::default();
                    }
                    EventType::ButtonPressed(btn, _) => match btn {
                        Button::South => self.state.btn_south = true,
                        Button::East => self.state.btn_east = true,
                        Button::West => self.state.btn_west = true,
                        Button::North => self.state.btn_north = true,
                        Button::LeftTrigger => self.state.btn_lb = true,
                        Button::RightTrigger => self.state.btn_rb = true,
                        Button::LeftThumb => self.state.btn_l3 = true,
                        Button::RightThumb => self.state.btn_r3 = true,
                        Button::Start => self.state.btn_start = true,
                        Button::Select | Button::Mode => self.state.btn_select = true,
                        Button::DPadUp => self.state.dpad_up = true,
                        Button::DPadDown => self.state.dpad_down = true,
                        Button::DPadLeft => self.state.dpad_left = true,
                        Button::DPadRight => self.state.dpad_right = true,
                        _ => {}
                    },
                    EventType::ButtonReleased(btn, _) => match btn {
                        Button::South => self.state.btn_south = false,
                        Button::East => self.state.btn_east = false,
                        Button::West => self.state.btn_west = false,
                        Button::North => self.state.btn_north = false,
                        Button::LeftTrigger => self.state.btn_lb = false,
                        Button::RightTrigger => self.state.btn_rb = false,
                        Button::LeftThumb => self.state.btn_l3 = false,
                        Button::RightThumb => self.state.btn_r3 = false,
                        Button::Start => self.state.btn_start = false,
                        Button::Select | Button::Mode => self.state.btn_select = false,
                        Button::DPadUp => self.state.dpad_up = false,
                        Button::DPadDown => self.state.dpad_down = false,
                        Button::DPadLeft => self.state.dpad_left = false,
                        Button::DPadRight => self.state.dpad_right = false,
                        _ => {}
                    },
                    EventType::ButtonChanged(btn, val, _) => match btn {
                        Button::LeftTrigger2 => self.state.left_trigger = val,
                        Button::RightTrigger2 => self.state.right_trigger = val,
                        _ => {}
                    },
                    EventType::AxisChanged(axis, val, _) => {
                        let filtered_val = if val.abs() < self.deadzone { 0.0 } else { val };
                        match axis {
                            Axis::LeftStickX => self.state.left_stick_x = filtered_val,
                            Axis::LeftStickY => self.state.left_stick_y = filtered_val,
                            Axis::RightStickX => self.state.right_stick_x = filtered_val,
                            Axis::RightStickY => self.state.right_stick_y = filtered_val,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Jeśli jest podłączony jakikolwiek gamepad w gilrs
            if gilrs.gamepads().next().is_some() {
                self.state.is_connected = true;
            }
        }

        // Obsługa powtarzania nawigacji analogowej w menu (stick repeat)
        self.stick_repeat_timer += dt;
        let mut stick_nav_up = false;
        let mut stick_nav_down = false;
        let mut stick_nav_left = false;
        let mut stick_nav_right = false;

        if self.stick_repeat_timer >= 0.22 {
            if self.state.left_stick_y > 0.55 {
                stick_nav_up = true;
                self.stick_repeat_timer = 0.0;
            } else if self.state.left_stick_y < -0.55 {
                stick_nav_down = true;
                self.stick_repeat_timer = 0.0;
            }

            if self.state.left_stick_x < -0.55 {
                stick_nav_left = true;
                self.stick_repeat_timer = 0.0;
            } else if self.state.left_stick_x > 0.55 {
                stick_nav_right = true;
                self.stick_repeat_timer = 0.0;
            }
        }

        GamepadEvents {
            south_just_pressed: self.state.btn_south && !self.prev_state.btn_south,
            east_just_pressed: self.state.btn_east && !self.prev_state.btn_east,
            west_just_pressed: self.state.btn_west && !self.prev_state.btn_west,
            north_just_pressed: self.state.btn_north && !self.prev_state.btn_north,
            lb_just_pressed: self.state.btn_lb && !self.prev_state.btn_lb,
            rb_just_pressed: self.state.btn_rb && !self.prev_state.btn_rb,
            l3_just_pressed: self.state.btn_l3 && !self.prev_state.btn_l3,
            r3_just_pressed: self.state.btn_r3 && !self.prev_state.btn_r3,
            start_just_pressed: self.state.btn_start && !self.prev_state.btn_start,
            select_just_pressed: self.state.btn_select && !self.prev_state.btn_select,
            dpad_up_just_pressed: (self.state.dpad_up && !self.prev_state.dpad_up) || stick_nav_up,
            dpad_down_just_pressed: (self.state.dpad_down && !self.prev_state.dpad_down) || stick_nav_down,
            dpad_left_just_pressed: (self.state.dpad_left && !self.prev_state.dpad_left) || stick_nav_left,
            dpad_right_just_pressed: (self.state.dpad_right && !self.prev_state.dpad_right) || stick_nav_right,
            right_trigger_just_pressed: self.state.right_trigger > 0.45 && self.prev_state.right_trigger <= 0.45,
            left_trigger_just_pressed: self.state.left_trigger > 0.45 && self.prev_state.left_trigger <= 0.45,
        }
    }
}
