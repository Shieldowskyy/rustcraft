//! Per-frame action evaluation state (`ControlsManager`) with edge detection.
//! Reads `InputState` snapshots and updates active/just-pressed flags without allocations.

use crate::app::InputState;

use super::{Action, ControlsConfig, ACTION_COUNT};

pub struct ControlsManager {
    pub config: ControlsConfig,
    active_now: [bool; ACTION_COUNT],
    active_prev: [bool; ACTION_COUNT],
    just_pressed: [bool; ACTION_COUNT],
}

impl Default for ControlsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlsManager {
    pub fn new() -> Self {
        Self {
            config: ControlsConfig::load(),
            active_now: [false; ACTION_COUNT],
            active_prev: [false; ACTION_COUNT],
            just_pressed: [false; ACTION_COUNT],
        }
    }

    pub fn reload(&mut self) {
        self.config = ControlsConfig::load();
    }

    /// Evaluates all inputs for the current frame.
    pub fn update(&mut self, input: &InputState) {
        self.active_prev = self.active_now;

        for action in Action::ALL {
            let idx = action.index();
            let binding = &self.config.bindings[idx];
            let mut active = binding.gamepad.iter().any(|gp| gp.is_down(input));

            if !active {
                let mouse_allowed = match action {
                    Action::HotbarPrev | Action::HotbarNext | Action::Pause | Action::Inventory => true,
                    _ => input.is_mouse_captured,
                };
                if mouse_allowed {
                    active = binding.mouse.iter().any(|m| m.is_down(input));
                }
            }

            if !active {
                let key_allowed = match action {
                    Action::Pause | Action::Inventory | Action::Save | Action::ToggleFly | Action::ToggleGamemode => true,
                    _ => input.is_mouse_captured,
                };
                if key_allowed {
                    active = binding.keys.iter().any(|k| k.is_down(input));
                }
            }

            self.active_now[idx] = active;
            let edge = active && !self.active_prev[idx];
            let gp_edge = binding.gamepad.iter().any(|gp| gp.is_just_pressed(input));
            self.just_pressed[idx] = edge || gp_edge;
        }
    }

    #[inline(always)]
    pub fn is_active(&self, action: Action) -> bool {
        self.active_now[action.index()]
    }

    #[inline(always)]
    pub fn is_just_pressed(&self, action: Action) -> bool {
        self.just_pressed[action.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::super::{GamepadInput, KeyCode, MouseInput};
    use super::*;

    #[test]
    fn test_controls_parsing() {
        let json = include_str!("../../../controls.json");
        let cfg = ControlsConfig::from_json(json).expect("Should parse controls.json");

        let hotbar_prev = &cfg.bindings[Action::HotbarPrev.index()];
        assert!(hotbar_prev.gamepad.contains(&GamepadInput::LB));
        assert!(hotbar_prev.mouse.contains(&MouseInput::WheelDown));

        let hotbar_next = &cfg.bindings[Action::HotbarNext.index()];
        assert!(hotbar_next.gamepad.contains(&GamepadInput::RB));
        assert!(hotbar_next.mouse.contains(&MouseInput::WheelUp));

        let jump = &cfg.bindings[Action::Jump.index()];
        assert!(jump.keys.contains(&KeyCode::Space));
        assert!(jump.gamepad.contains(&GamepadInput::South));
    }

    #[test]
    fn test_controls_manager_lb_rb_and_left_stick_isolation() {
        let mut manager = ControlsManager::new();
        let mut input = InputState {
            gp_left_stick_x: -0.95,
            is_mouse_captured: true,
            ..Default::default()
        };

        manager.update(&input);
        assert!(!manager.is_active(Action::HotbarPrev));
        assert!(!manager.is_just_pressed(Action::HotbarPrev));
        assert!(!manager.is_active(Action::HotbarNext));
        assert!(!manager.is_just_pressed(Action::HotbarNext));

        input.gp_btn_lb = true;
        input.gp_lb_just_pressed = true;
        manager.update(&input);
        assert!(manager.is_active(Action::HotbarPrev));
        assert!(manager.is_just_pressed(Action::HotbarPrev));
        assert!(!manager.is_active(Action::HotbarNext));

        input.gp_btn_lb = false;
        input.gp_lb_just_pressed = false;
        input.gp_btn_rb = true;
        input.gp_rb_just_pressed = true;
        manager.update(&input);
        assert!(!manager.is_active(Action::HotbarPrev));
        assert!(manager.is_active(Action::HotbarNext));
        assert!(manager.is_just_pressed(Action::HotbarNext));
    }

    #[test]
    fn test_custom_controls_json() {
        let custom_json = r#"{
            "jump": { "keys": ["W"], "gamepad": ["North"] },
            "inventory": { "keys": ["I"], "gamepad": ["Select"] }
        }"#;

        let cfg = ControlsConfig::from_json(custom_json).expect("Custom controls should parse");
        let jump = &cfg.bindings[Action::Jump.index()];
        assert_eq!(jump.keys, vec![KeyCode::W]);
        assert_eq!(jump.gamepad, vec![GamepadInput::North]);

        let inv = &cfg.bindings[Action::Inventory.index()];
        assert_eq!(inv.gamepad, vec![GamepadInput::Select]);
    }

    #[test]
    fn test_rebind_and_to_json_roundtrip() {
        let mut cfg = ControlsConfig::default_bindings();
        cfg.rebind_key(Action::Jump, KeyCode::F13);
        cfg.rebind_mouse(Action::AttackBreak, MouseInput::Mouse4);
        cfg.rebind_gamepad(Action::Sprint, GamepadInput::RightTrigger);

        let json = cfg.to_json();
        let parsed = ControlsConfig::from_json(&json).expect("Serialized controls should parse");

        assert_eq!(parsed.bindings[Action::Jump.index()].keys, vec![KeyCode::F13]);
        assert_eq!(parsed.bindings[Action::AttackBreak.index()].mouse, vec![MouseInput::Mouse4]);
        assert_eq!(parsed.bindings[Action::Sprint.index()].gamepad, vec![GamepadInput::RightTrigger]);
    }

    #[test]
    fn test_f_keys_above_12_and_extra_mouse() {
        assert_eq!(KeyCode::from_str("f13"), Some(KeyCode::F13));
        assert_eq!(KeyCode::from_str("F24"), Some(KeyCode::F24));
        assert_eq!(KeyCode::F13.display_name(), "F13");
        assert_eq!(KeyCode::F24.display_name(), "F24");

        assert_eq!(MouseInput::from_str("mouse4"), Some(MouseInput::Mouse4));
        assert_eq!(MouseInput::from_str("mouse5"), Some(MouseInput::Mouse5));
        assert_eq!(MouseInput::Mouse4.display_name(), "Mysz 4 (Wstecz)");
        assert_eq!(MouseInput::Mouse5.display_name(), "Mysz 5 (Dalej)");

        let mut input = InputState::default();
        input.keys_down.insert(KeyCode::F15);
        input.mouse_4 = true;

        assert!(KeyCode::F15.is_down(&input));
        assert!(!KeyCode::F16.is_down(&input));
        assert!(MouseInput::Mouse4.is_down(&input));
        assert!(!MouseInput::Mouse5.is_down(&input));
    }
}

