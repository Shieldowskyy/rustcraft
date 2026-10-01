//! Cross-platform gamepad polling and event abstraction using `gilrs`.

use gilrs::{Axis, Button, Event, EventType, Gilrs};

/// Gamepad manager tracking analog axes, button states, deadzones,
/// and edge-triggered click events (`just_pressed`) for menus and gameplay.
pub struct GamepadManager {
    gilrs: Option<Gilrs>,
    pub state: GamepadState,
    prev_state: GamepadState,
    deadzone: f32,
    stick_repeat_timer: f32,
}

/// Instantaneous gamepad hardware state.
#[derive(Clone, Copy, Default, Debug)]
pub struct GamepadState {
    pub is_connected: bool,

    // Analog stick axes (-1.0 .. 1.0) and analog triggers (0.0 .. 1.0)
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,

    // Digital button continuous press states
    pub btn_south: bool,   // A / Cross (Jump / UI Confirm)
    pub btn_east: bool,    // B / Circle (Sneak / UI Back)
    pub btn_west: bool,    // X / Square (Drop item)
    pub btn_north: bool,   // Y / Triangle (Inventory)
    pub btn_lb: bool,      // L1 / Left Bumper (Previous hotbar slot)
    pub btn_rb: bool,      // R1 / Right Bumper (Next hotbar slot)
    pub btn_lt: bool,      // L2 / Left Trigger (Place block)
    pub btn_rt: bool,      // R2 / Right Trigger (Break block / Attack)
    pub btn_l3: bool,      // Left Stick Click (Sprint)
    pub btn_r3: bool,      // Right Stick Click (Toggle fly mode)
    pub btn_start: bool,   // Start / Menu (Pause)
    pub btn_select: bool,  // Select / Back (Toggle game mode / Save)
    pub dpad_up: bool,     // D-Pad Up
    pub dpad_down: bool,   // D-Pad Down
    pub dpad_left: bool,   // D-Pad Left
    pub dpad_right: bool,  // D-Pad Right
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
    pub menu_up: bool,
    pub menu_down: bool,
    pub menu_left: bool,
    pub menu_right: bool,
    pub right_trigger_just_pressed: bool,
    pub left_trigger_just_pressed: bool,
}

impl GamepadManager {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("Gamepad subsystem unavailable or failed to initialize: {e}");
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

    /// Updates the gamepad state based on system events from the gilrs library.
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
                        Button::LeftTrigger2 => {
                            self.state.btn_lt = true;
                            self.state.left_trigger = 1.0;
                        }
                        Button::RightTrigger2 => {
                            self.state.btn_rt = true;
                            self.state.right_trigger = 1.0;
                        }
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
                        Button::LeftTrigger2 => {
                            self.state.btn_lt = false;
                            self.state.left_trigger = 0.0;
                        }
                        Button::RightTrigger2 => {
                            self.state.btn_rt = false;
                            self.state.right_trigger = 0.0;
                        }
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
                        Button::LeftTrigger2 => {
                            self.state.left_trigger = val;
                            self.state.btn_lt = val > 0.15;
                        }
                        Button::RightTrigger2 => {
                            self.state.right_trigger = val;
                            self.state.btn_rt = val > 0.15;
                        }
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

            // Direct state polling from connected gamepads
            for (_id, gamepad) in gilrs.gamepads() {
                if gamepad.is_connected() {
                    self.state.is_connected = true;

                    let lt_pressed = gamepad.is_pressed(Button::LeftTrigger2);
                    let rt_pressed = gamepad.is_pressed(Button::RightTrigger2);
                    let lt_val = gamepad.button_data(Button::LeftTrigger2).map(|d| d.value()).unwrap_or(0.0);
                    let rt_val = gamepad.button_data(Button::RightTrigger2).map(|d| d.value()).unwrap_or(0.0);

                    if lt_pressed || lt_val > 0.15 {
                        self.state.btn_lt = true;
                        self.state.left_trigger = self.state.left_trigger.max(if lt_pressed { 1.0 } else { lt_val });
                    }
                    if rt_pressed || rt_val > 0.15 {
                        self.state.btn_rt = true;
                        self.state.right_trigger = self.state.right_trigger.max(if rt_pressed { 1.0 } else { rt_val });
                    }
                    break;
                }
            }
        }

        // Handle analog stick repeat for UI navigation
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

        let dpad_up_edge = self.state.dpad_up && !self.prev_state.dpad_up;
        let dpad_down_edge = self.state.dpad_down && !self.prev_state.dpad_down;
        let dpad_left_edge = self.state.dpad_left && !self.prev_state.dpad_left;
        let dpad_right_edge = self.state.dpad_right && !self.prev_state.dpad_right;

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
            dpad_up_just_pressed: dpad_up_edge,
            dpad_down_just_pressed: dpad_down_edge,
            dpad_left_just_pressed: dpad_left_edge,
            dpad_right_just_pressed: dpad_right_edge,
            menu_up: dpad_up_edge || stick_nav_up,
            menu_down: dpad_down_edge || stick_nav_down,
            menu_left: dpad_left_edge || stick_nav_left,
            menu_right: dpad_right_edge || stick_nav_right,
            right_trigger_just_pressed: (self.state.btn_rt && !self.prev_state.btn_rt)
                || (self.state.right_trigger > 0.15 && self.prev_state.right_trigger <= 0.15),
            left_trigger_just_pressed: (self.state.btn_lt && !self.prev_state.btn_lt)
                || (self.state.left_trigger > 0.15 && self.prev_state.left_trigger <= 0.15),
        }
    }
}
