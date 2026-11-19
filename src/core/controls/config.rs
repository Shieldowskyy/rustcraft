//! Control bindings configuration: defaults, rebinding, and `controls.json` serialization.

use super::json::{parse_json_object, tokenize_json, JsonValue};
use super::{Action, ActionBinding, GamepadInput, KeyCode, MouseInput, ACTION_COUNT};

#[derive(Debug, Clone)]
pub struct ControlsConfig {
    pub bindings: [ActionBinding; ACTION_COUNT],
}

impl Default for ControlsConfig {
    fn default() -> Self {
        Self::default_bindings()
    }
}

impl ControlsConfig {
    pub fn default_bindings() -> Self {
        let mut bindings: [ActionBinding; ACTION_COUNT] = Default::default();

        bindings[Action::MoveForward.index()] = ActionBinding { keys: vec![KeyCode::W], gamepad: vec![], mouse: vec![] };
        bindings[Action::MoveBackward.index()] = ActionBinding { keys: vec![KeyCode::S], gamepad: vec![], mouse: vec![] };
        bindings[Action::StrafeLeft.index()] = ActionBinding { keys: vec![KeyCode::A], gamepad: vec![], mouse: vec![] };
        bindings[Action::StrafeRight.index()] = ActionBinding { keys: vec![KeyCode::D], gamepad: vec![], mouse: vec![] };
        bindings[Action::Jump.index()] = ActionBinding { keys: vec![KeyCode::Space], gamepad: vec![GamepadInput::South], mouse: vec![] };
        bindings[Action::Sneak.index()] = ActionBinding { keys: vec![KeyCode::Shift, KeyCode::C], gamepad: vec![GamepadInput::East], mouse: vec![] };
        bindings[Action::Sprint.index()] = ActionBinding { keys: vec![KeyCode::Ctrl], gamepad: vec![GamepadInput::L3], mouse: vec![] };
        bindings[Action::Inventory.index()] = ActionBinding { keys: vec![KeyCode::E], gamepad: vec![GamepadInput::North], mouse: vec![] };
        bindings[Action::DropItem.index()] = ActionBinding { keys: vec![KeyCode::Q], gamepad: vec![GamepadInput::West], mouse: vec![] };
        bindings[Action::ToggleFly.index()] = ActionBinding { keys: vec![KeyCode::F], gamepad: vec![GamepadInput::R3], mouse: vec![] };
        bindings[Action::ToggleGamemode.index()] = ActionBinding { keys: vec![KeyCode::G], gamepad: vec![GamepadInput::Select], mouse: vec![] };
        bindings[Action::Pause.index()] = ActionBinding { keys: vec![KeyCode::Escape, KeyCode::P], gamepad: vec![GamepadInput::Start], mouse: vec![] };
        bindings[Action::Save.index()] = ActionBinding { keys: vec![KeyCode::F5], gamepad: vec![], mouse: vec![] };
        bindings[Action::AttackBreak.index()] = ActionBinding { keys: vec![], gamepad: vec![GamepadInput::RightTrigger], mouse: vec![MouseInput::Left] };
        bindings[Action::UsePlace.index()] = ActionBinding { keys: vec![], gamepad: vec![GamepadInput::LeftTrigger], mouse: vec![MouseInput::Right] };
        bindings[Action::HotbarPrev.index()] = ActionBinding { keys: vec![], gamepad: vec![GamepadInput::LB], mouse: vec![MouseInput::WheelDown] };
        bindings[Action::HotbarNext.index()] = ActionBinding { keys: vec![], gamepad: vec![GamepadInput::RB], mouse: vec![MouseInput::WheelUp] };
        bindings[Action::OpenChat.index()] = ActionBinding { keys: vec![KeyCode::T], gamepad: vec![], mouse: vec![] };
        bindings[Action::OpenCommand.index()] = ActionBinding { keys: vec![KeyCode::Slash], gamepad: vec![], mouse: vec![] };
        bindings[Action::PickBlock.index()] = ActionBinding { keys: vec![], gamepad: vec![], mouse: vec![MouseInput::Middle] };
        bindings[Action::Questbook.index()] = ActionBinding { keys: vec![KeyCode::J], gamepad: vec![], mouse: vec![] };

        Self { bindings }
    }

    /// Loads bindings from `controls.json` on disk, falling back to embedded defaults.
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("controls.json") {
            if let Ok(cfg) = Self::from_json(&content) {
                return cfg;
            }
        }
        Self::from_json(include_str!("../../../controls.json"))
            .unwrap_or_else(|_| Self::default_bindings())
    }

    /// Parses a JSON string into `ControlsConfig`.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let mut config = Self::default_bindings();
        let tokens = tokenize_json(json_str)?;
        let parsed = parse_json_object(&tokens)?;

        for (action_key, action_val) in parsed {
            if let Some(action) = Action::from_str(&action_key) {
                let mut binding = ActionBinding::default();

                if let JsonValue::Object(inner_map) = action_val {
                    for (field_name, field_val) in inner_map {
                        match field_name.to_ascii_lowercase().as_str() {
                            "keys" => {
                                if let JsonValue::Array(arr) = field_val {
                                    for item in arr {
                                        if let JsonValue::Str(k_str) = item {
                                            if let Some(k) = KeyCode::from_str(&k_str) {
                                                binding.keys.push(k);
                                            }
                                        }
                                    }
                                }
                            }
                            "gamepad" => {
                                if let JsonValue::Array(arr) = field_val {
                                    for item in arr {
                                        if let JsonValue::Str(gp_str) = item {
                                            if let Some(gp) = GamepadInput::from_str(&gp_str) {
                                                binding.gamepad.push(gp);
                                            }
                                        }
                                    }
                                }
                            }
                            "mouse" => {
                                if let JsonValue::Array(arr) = field_val {
                                    for item in arr {
                                        if let JsonValue::Str(m_str) = item {
                                            if let Some(m) = MouseInput::from_str(&m_str) {
                                                binding.mouse.push(m);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }

                config.bindings[action.index()] = binding;
            }
        }

        Ok(config)
    }

    pub fn rebind_key(&mut self, action: Action, key: KeyCode) {
        self.bindings[action.index()].keys = vec![key];
    }

    pub fn rebind_mouse(&mut self, action: Action, mouse: MouseInput) {
        self.bindings[action.index()].mouse = vec![mouse];
    }

    pub fn rebind_gamepad(&mut self, action: Action, gp: GamepadInput) {
        self.bindings[action.index()].gamepad = vec![gp];
    }

    pub fn clear_key_binding(&mut self, action: Action) {
        self.bindings[action.index()].keys.clear();
        self.bindings[action.index()].mouse.clear();
    }

    pub fn clear_gamepad_binding(&mut self, action: Action) {
        self.bindings[action.index()].gamepad.clear();
    }

    pub fn reset_action_to_default(&mut self, action: Action) {
        let def = Self::default_bindings();
        self.bindings[action.index()] = def.bindings[action.index()].clone();
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default_bindings();
    }

    pub fn to_json(&self) -> String {
        let mut out = String::from("{\n");
        for (i, action) in Action::ALL.iter().enumerate() {
            let b = &self.bindings[action.index()];
            out.push_str(&format!("  \"{}\": {{\n", action.as_str()));
            push_string_array(&mut out, "    \"keys\": [", b.keys.iter().map(KeyCode::as_str), true);
            push_string_array(&mut out, "    \"gamepad\": [", b.gamepad.iter().map(GamepadInput::as_str), true);
            push_string_array(&mut out, "    \"mouse\": [", b.mouse.iter().map(MouseInput::as_str), false);
            out.push_str(if i + 1 < Action::ALL.len() { "  },\n" } else { "  }\n" });
        }
        out.push_str("}\n");
        out
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        std::fs::write("controls.json", self.to_json())
    }
}

fn push_string_array<'a>(
    out: &mut String,
    prefix: &str,
    items: impl Iterator<Item = &'a str>,
    trailing_comma: bool,
) {
    out.push_str(prefix);
    for (idx, val) in items.enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(val);
        out.push('"');
    }
    out.push_str(if trailing_comma { "],\n" } else { "]\n" });
}

