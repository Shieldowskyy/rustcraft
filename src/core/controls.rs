//! Modular JSON Controls System for RustCraft
//! Loads key, gamepad, and mouse bindings from `controls.json`.

use crate::app::InputState;

pub const ACTION_COUNT: usize = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    MoveForward = 0,
    MoveBackward = 1,
    StrafeLeft = 2,
    StrafeRight = 3,
    Jump = 4,
    Sneak = 5,
    Sprint = 6,
    Inventory = 7,
    DropItem = 8,
    ToggleFly = 9,
    ToggleGamemode = 10,
    Pause = 11,
    Save = 12,
    AttackBreak = 13,
    UsePlace = 14,
    HotbarPrev = 15,
    HotbarNext = 16,
    OpenChat = 17,
    OpenCommand = 18,
}

impl Action {
    pub const ALL: [Action; ACTION_COUNT] = [
        Action::MoveForward,
        Action::MoveBackward,
        Action::StrafeLeft,
        Action::StrafeRight,
        Action::Jump,
        Action::Sneak,
        Action::Sprint,
        Action::Inventory,
        Action::DropItem,
        Action::ToggleFly,
        Action::ToggleGamemode,
        Action::Pause,
        Action::Save,
        Action::AttackBreak,
        Action::UsePlace,
        Action::HotbarPrev,
        Action::HotbarNext,
        Action::OpenChat,
        Action::OpenCommand,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Action::MoveForward => "move_forward",
            Action::MoveBackward => "move_backward",
            Action::StrafeLeft => "strafe_left",
            Action::StrafeRight => "strafe_right",
            Action::Jump => "jump",
            Action::Sneak => "sneak",
            Action::Sprint => "sprint",
            Action::Inventory => "inventory",
            Action::DropItem => "drop_item",
            Action::ToggleFly => "toggle_fly",
            Action::ToggleGamemode => "toggle_gamemode",
            Action::Pause => "pause",
            Action::Save => "save",
            Action::AttackBreak => "attack_break",
            Action::UsePlace => "use_place",
            Action::HotbarPrev => "hotbar_prev",
            Action::HotbarNext => "hotbar_next",
            Action::OpenChat => "open_chat",
            Action::OpenCommand => "open_command",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "move_forward" | "forward" => Some(Action::MoveForward),
            "move_backward" | "backward" => Some(Action::MoveBackward),
            "strafe_left" | "left" => Some(Action::StrafeLeft),
            "strafe_right" | "right" => Some(Action::StrafeRight),
            "jump" => Some(Action::Jump),
            "sneak" | "crouch" => Some(Action::Sneak),
            "sprint" | "run" => Some(Action::Sprint),
            "inventory" => Some(Action::Inventory),
            "drop_item" | "drop" => Some(Action::DropItem),
            "toggle_fly" | "fly" => Some(Action::ToggleFly),
            "toggle_gamemode" | "gamemode" => Some(Action::ToggleGamemode),
            "pause" | "menu" => Some(Action::Pause),
            "save" | "quick_save" => Some(Action::Save),
            "attack_break" | "attack" | "break" => Some(Action::AttackBreak),
            "use_place" | "use" | "place" => Some(Action::UsePlace),
            "hotbar_prev" | "prev_slot" => Some(Action::HotbarPrev),
            "hotbar_next" | "next_slot" => Some(Action::HotbarNext),
            "open_chat" | "chat" => Some(Action::OpenChat),
            "open_command" | "command" | "slash" => Some(Action::OpenCommand),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    W, S, A, D,
    Space, Shift, Ctrl, C, V, X,
    Escape, P, F, E, Q, G, M, F5,
    Enter, Up, Down, Left, Right,
    N, Delete, Backspace, Tab,
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    T, Slash,
}

impl KeyCode {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "w" => Some(KeyCode::W),
            "s" => Some(KeyCode::S),
            "a" => Some(KeyCode::A),
            "d" => Some(KeyCode::D),
            "space" => Some(KeyCode::Space),
            "shift" | "leftshift" | "lshift" => Some(KeyCode::Shift),
            "control" | "ctrl" | "leftcontrol" | "lctrl" => Some(KeyCode::Ctrl),
            "c" => Some(KeyCode::C),
            "v" => Some(KeyCode::V),
            "x" => Some(KeyCode::X),
            "escape" | "esc" => Some(KeyCode::Escape),
            "p" => Some(KeyCode::P),
            "f" => Some(KeyCode::F),
            "e" => Some(KeyCode::E),
            "q" => Some(KeyCode::Q),
            "g" => Some(KeyCode::G),
            "m" => Some(KeyCode::M),
            "f5" => Some(KeyCode::F5),
            "enter" | "return" => Some(KeyCode::Enter),
            "up" => Some(KeyCode::Up),
            "down" => Some(KeyCode::Down),
            "left" => Some(KeyCode::Left),
            "right" => Some(KeyCode::Right),
            "n" => Some(KeyCode::N),
            "delete" | "del" => Some(KeyCode::Delete),
            "backspace" => Some(KeyCode::Backspace),
            "tab" => Some(KeyCode::Tab),
            "1" | "key1" => Some(KeyCode::Key1),
            "2" | "key2" => Some(KeyCode::Key2),
            "3" | "key3" => Some(KeyCode::Key3),
            "4" | "key4" => Some(KeyCode::Key4),
            "5" | "key5" => Some(KeyCode::Key5),
            "6" | "key6" => Some(KeyCode::Key6),
            "7" | "key7" => Some(KeyCode::Key7),
            "8" | "key8" => Some(KeyCode::Key8),
            "9" | "key9" => Some(KeyCode::Key9),
            "0" | "key0" => Some(KeyCode::Key0),
            "t" => Some(KeyCode::T),
            "slash" | "/" => Some(KeyCode::Slash),
            _ => None,
        }
    }

    #[inline]
    pub fn is_down(&self, input: &InputState) -> bool {
        match self {
            KeyCode::W => input.key_w,
            KeyCode::S => input.key_s,
            KeyCode::A => input.key_a,
            KeyCode::D => input.key_d,
            KeyCode::Space => input.key_space,
            KeyCode::Shift => input.key_shift,
            KeyCode::Ctrl => input.key_ctrl,
            KeyCode::C => input.key_c,
            KeyCode::V => input.key_v,
            KeyCode::X => input.key_x,
            KeyCode::Escape => input.key_escape,
            KeyCode::P => input.key_p,
            KeyCode::F => input.key_f,
            KeyCode::E => input.key_e,
            KeyCode::Q => input.key_q,
            KeyCode::G => input.key_g,
            KeyCode::M => input.key_m,
            KeyCode::F5 => input.key_f5,
            KeyCode::Enter => input.key_enter,
            KeyCode::Up => input.key_up,
            KeyCode::Down => input.key_down,
            KeyCode::Left => input.key_left,
            KeyCode::Right => input.key_right,
            KeyCode::N => input.key_n,
            KeyCode::Delete => input.key_del,
            KeyCode::Backspace => input.key_backspace,
            KeyCode::Tab => input.key_tab,
            KeyCode::Key1 => input.key_digits[1],
            KeyCode::Key2 => input.key_digits[2],
            KeyCode::Key3 => input.key_digits[3],
            KeyCode::Key4 => input.key_digits[4],
            KeyCode::Key5 => input.key_digits[5],
            KeyCode::Key6 => input.key_digits[6],
            KeyCode::Key7 => input.key_digits[7],
            KeyCode::Key8 => input.key_digits[8],
            KeyCode::Key9 => input.key_digits[9],
            KeyCode::Key0 => input.key_digits[0],
            KeyCode::T => input.key_t,
            KeyCode::Slash => input.key_slash,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadInput {
    South,
    East,
    West,
    North,
    LB,
    RB,
    L3,
    R3,
    Start,
    Select,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    LeftTrigger,
    RightTrigger,
}

impl GamepadInput {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "south" | "a" | "cross" => Some(GamepadInput::South),
            "east" | "b" | "circle" => Some(GamepadInput::East),
            "west" | "x" | "square" => Some(GamepadInput::West),
            "north" | "y" | "triangle" => Some(GamepadInput::North),
            "lb" | "l1" | "leftbumper" => Some(GamepadInput::LB),
            "rb" | "r1" | "rightbumper" => Some(GamepadInput::RB),
            "l3" | "leftstick" => Some(GamepadInput::L3),
            "r3" | "rightstick" => Some(GamepadInput::R3),
            "start" => Some(GamepadInput::Start),
            "select" => Some(GamepadInput::Select),
            "dpadup" | "dpad_up" => Some(GamepadInput::DpadUp),
            "dpaddown" | "dpad_down" => Some(GamepadInput::DpadDown),
            "dpadleft" | "dpad_left" => Some(GamepadInput::DpadLeft),
            "dpadright" | "dpad_right" => Some(GamepadInput::DpadRight),
            "lefttrigger" | "lt" | "l2" => Some(GamepadInput::LeftTrigger),
            "righttrigger" | "rt" | "r2" => Some(GamepadInput::RightTrigger),
            _ => None,
        }
    }

    #[inline]
    pub fn is_down(&self, input: &InputState) -> bool {
        match self {
            GamepadInput::South => input.gp_btn_south,
            GamepadInput::East => input.gp_btn_east,
            GamepadInput::West => input.gp_btn_west,
            GamepadInput::North => input.gp_btn_north,
            GamepadInput::LB => input.gp_btn_lb,
            GamepadInput::RB => input.gp_btn_rb,
            GamepadInput::L3 => input.gp_btn_l3,
            GamepadInput::R3 => input.gp_btn_r3,
            GamepadInput::Start => input.gp_btn_start,
            GamepadInput::Select => input.gp_btn_select,
            GamepadInput::DpadUp => input.gp_dpad_up,
            GamepadInput::DpadDown => input.gp_dpad_down,
            GamepadInput::DpadLeft => input.gp_dpad_left,
            GamepadInput::DpadRight => input.gp_dpad_right,
            GamepadInput::LeftTrigger => input.gp_left_trigger > 0.15,
            GamepadInput::RightTrigger => input.gp_right_trigger > 0.15,
        }
    }

    #[inline]
    pub fn is_just_pressed(&self, input: &InputState) -> bool {
        match self {
            GamepadInput::South => input.gp_south_just_pressed,
            GamepadInput::East => input.gp_east_just_pressed,
            GamepadInput::West => input.gp_west_just_pressed,
            GamepadInput::North => input.gp_north_just_pressed,
            GamepadInput::LB => input.gp_lb_just_pressed,
            GamepadInput::RB => input.gp_rb_just_pressed,
            GamepadInput::L3 => false,
            GamepadInput::R3 => input.gp_r3_just_pressed,
            GamepadInput::Start => input.gp_start_just_pressed,
            GamepadInput::Select => input.gp_select_just_pressed,
            GamepadInput::DpadUp => input.gp_dpad_up_just_pressed,
            GamepadInput::DpadDown => input.gp_dpad_down_just_pressed,
            GamepadInput::DpadLeft => input.gp_dpad_left_just_pressed,
            GamepadInput::DpadRight => input.gp_dpad_right_just_pressed,
            GamepadInput::LeftTrigger => input.gp_left_trigger_just_pressed,
            GamepadInput::RightTrigger => input.gp_right_trigger_just_pressed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseInput {
    Left,
    Right,
    WheelUp,
    WheelDown,
}

impl MouseInput {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "left" | "mouse1" | "lmb" => Some(MouseInput::Left),
            "right" | "mouse2" | "rmb" => Some(MouseInput::Right),
            "wheelup" | "scrollup" => Some(MouseInput::WheelUp),
            "wheeldown" | "scrolldown" => Some(MouseInput::WheelDown),
            _ => None,
        }
    }

    #[inline]
    pub fn is_down(&self, input: &InputState) -> bool {
        match self {
            MouseInput::Left => input.mouse_left,
            MouseInput::Right => input.mouse_right,
            MouseInput::WheelUp => input.scroll_delta > 0.0,
            MouseInput::WheelDown => input.scroll_delta < 0.0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActionBinding {
    pub keys: Vec<KeyCode>,
    pub gamepad: Vec<GamepadInput>,
    pub mouse: Vec<MouseInput>,
}

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

        bindings[Action::MoveForward.index()] = ActionBinding {
            keys: vec![KeyCode::W],
            gamepad: vec![],
            mouse: vec![],
        };
        bindings[Action::MoveBackward.index()] = ActionBinding {
            keys: vec![KeyCode::S],
            gamepad: vec![],
            mouse: vec![],
        };
        bindings[Action::StrafeLeft.index()] = ActionBinding {
            keys: vec![KeyCode::A],
            gamepad: vec![],
            mouse: vec![],
        };
        bindings[Action::StrafeRight.index()] = ActionBinding {
            keys: vec![KeyCode::D],
            gamepad: vec![],
            mouse: vec![],
        };
        bindings[Action::Jump.index()] = ActionBinding {
            keys: vec![KeyCode::Space],
            gamepad: vec![GamepadInput::South],
            mouse: vec![],
        };
        bindings[Action::Sneak.index()] = ActionBinding {
            keys: vec![KeyCode::Shift, KeyCode::C],
            gamepad: vec![GamepadInput::East],
            mouse: vec![],
        };
        bindings[Action::Sprint.index()] = ActionBinding {
            keys: vec![KeyCode::Ctrl],
            gamepad: vec![GamepadInput::L3],
            mouse: vec![],
        };
        bindings[Action::Inventory.index()] = ActionBinding {
            keys: vec![KeyCode::E],
            gamepad: vec![GamepadInput::North],
            mouse: vec![],
        };
        bindings[Action::DropItem.index()] = ActionBinding {
            keys: vec![KeyCode::Q],
            gamepad: vec![GamepadInput::West],
            mouse: vec![],
        };
        bindings[Action::ToggleFly.index()] = ActionBinding {
            keys: vec![KeyCode::F],
            gamepad: vec![GamepadInput::R3],
            mouse: vec![],
        };
        bindings[Action::ToggleGamemode.index()] = ActionBinding {
            keys: vec![KeyCode::G],
            gamepad: vec![GamepadInput::Select],
            mouse: vec![],
        };
        bindings[Action::Pause.index()] = ActionBinding {
            keys: vec![KeyCode::Escape, KeyCode::P],
            gamepad: vec![GamepadInput::Start],
            mouse: vec![],
        };
        bindings[Action::Save.index()] = ActionBinding {
            keys: vec![KeyCode::F5],
            gamepad: vec![],
            mouse: vec![],
        };
        bindings[Action::AttackBreak.index()] = ActionBinding {
            keys: vec![],
            gamepad: vec![GamepadInput::RightTrigger],
            mouse: vec![MouseInput::Left],
        };
        bindings[Action::UsePlace.index()] = ActionBinding {
            keys: vec![],
            gamepad: vec![GamepadInput::LeftTrigger],
            mouse: vec![MouseInput::Right],
        };
        bindings[Action::HotbarPrev.index()] = ActionBinding {
            keys: vec![],
            gamepad: vec![GamepadInput::LB],
            mouse: vec![MouseInput::WheelDown],
        };
        bindings[Action::HotbarNext.index()] = ActionBinding {
            keys: vec![],
            gamepad: vec![GamepadInput::RB],
            mouse: vec![MouseInput::WheelUp],
        };
        bindings[Action::OpenChat.index()] = ActionBinding {
            keys: vec![KeyCode::T],
            gamepad: vec![],
            mouse: vec![],
        };
        bindings[Action::OpenCommand.index()] = ActionBinding {
            keys: vec![KeyCode::Slash],
            gamepad: vec![],
            mouse: vec![],
        };

        Self { bindings }
    }

    /// Loads bindings from `controls.json` on disk if available,
    /// falling back to embedded default `controls.json`.
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("controls.json") {
            if let Ok(cfg) = Self::from_json(&content) {
                return cfg;
            }
        }
        Self::from_json(include_str!("../../controls.json"))
            .unwrap_or_else(|_| Self::default_bindings())
    }

    /// Parses JSON string into `ControlsConfig`.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        let mut config = Self::default_bindings();

        // Minimal robust zero-dependency JSON parser for controls config
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
}

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

            let mut active = false;

            // 1. Gamepad inputs (independent of mouse capture)
            for gp in &binding.gamepad {
                if gp.is_down(input) {
                    active = true;
                    break;
                }
            }

            // 2. Mouse inputs
            if !active {
                // UI actions can work regardless; gameplay actions require mouse capture
                let mouse_allowed = match action {
                    Action::HotbarPrev | Action::HotbarNext => true,
                    Action::Pause | Action::Inventory => true,
                    _ => input.is_mouse_captured,
                };

                if mouse_allowed {
                    for m in &binding.mouse {
                        if m.is_down(input) {
                            active = true;
                            break;
                        }
                    }
                }
            }

            // 3. Keyboard inputs
            if !active {
                let key_allowed = match action {
                    Action::Pause | Action::Inventory | Action::Save | Action::ToggleFly | Action::ToggleGamemode => true,
                    _ => input.is_mouse_captured,
                };

                if key_allowed {
                    for k in &binding.keys {
                        if k.is_down(input) {
                            active = true;
                            break;
                        }
                    }
                }
            }

            self.active_now[idx] = active;

            // Edge detection
            let edge = active && !self.active_prev[idx];

            // Explicit gamepad one-shot triggers
            let mut gp_edge = false;
            for gp in &binding.gamepad {
                if gp.is_just_pressed(input) {
                    gp_edge = true;
                    break;
                }
            }

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

// ============================================================================
// Zero-Dependency JSON Tokenizer & Parser for Controls
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
enum JsonToken {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Colon,
    Comma,
    StringVal(String),
}

#[derive(Debug, Clone)]
enum JsonValue {
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

fn tokenize_json(json: &str) -> Result<Vec<JsonToken>, String> {
    let mut tokens = Vec::new();
    let mut chars = json.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '{' => {
                chars.next();
                tokens.push(JsonToken::BraceOpen);
            }
            '}' => {
                chars.next();
                tokens.push(JsonToken::BraceClose);
            }
            '[' => {
                chars.next();
                tokens.push(JsonToken::BracketOpen);
            }
            ']' => {
                chars.next();
                tokens.push(JsonToken::BracketClose);
            }
            ':' => {
                chars.next();
                tokens.push(JsonToken::Colon);
            }
            ',' => {
                chars.next();
                tokens.push(JsonToken::Comma);
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                let mut escaped = false;
                while let Some(ch) = chars.next() {
                    if escaped {
                        s.push(ch);
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        break;
                    } else {
                        s.push(ch);
                    }
                }
                tokens.push(JsonToken::StringVal(s));
            }
            _ if c.is_whitespace() => {
                chars.next();
            }
            _ => {
                // Skip comments or non-relevant characters
                chars.next();
            }
        }
    }

    Ok(tokens)
}

fn parse_json_object(tokens: &[JsonToken]) -> Result<Vec<(String, JsonValue)>, String> {
    let mut idx = 0;
    if idx < tokens.len() && tokens[idx] == JsonToken::BraceOpen {
        idx += 1;
    }

    let mut result = Vec::new();

    while idx < tokens.len() {
        if tokens[idx] == JsonToken::BraceClose {
            break;
        }

        if let JsonToken::StringVal(ref key) = tokens[idx] {
            let key = key.clone();
            idx += 1;

            if idx >= tokens.len() || tokens[idx] != JsonToken::Colon {
                return Err("Expected colon after key".to_string());
            }
            idx += 1;

            let (val, next_idx) = parse_json_value(tokens, idx)?;
            result.push((key, val));
            idx = next_idx;

            if idx < tokens.len() && tokens[idx] == JsonToken::Comma {
                idx += 1;
            }
        } else {
            idx += 1;
        }
    }

    Ok(result)
}

fn parse_json_value(tokens: &[JsonToken], start: usize) -> Result<(JsonValue, usize), String> {
    if start >= tokens.len() {
        return Err("Unexpected end of tokens".to_string());
    }

    match &tokens[start] {
        JsonToken::StringVal(s) => Ok((JsonValue::Str(s.clone()), start + 1)),
        JsonToken::BracketOpen => {
            let mut arr = Vec::new();
            let mut idx = start + 1;
            while idx < tokens.len() {
                if tokens[idx] == JsonToken::BracketClose {
                    idx += 1;
                    break;
                }
                let (val, next_idx) = parse_json_value(tokens, idx)?;
                arr.push(val);
                idx = next_idx;
                if idx < tokens.len() && tokens[idx] == JsonToken::Comma {
                    idx += 1;
                }
            }
            Ok((JsonValue::Array(arr), idx))
        }
        JsonToken::BraceOpen => {
            let mut obj = Vec::new();
            let mut idx = start + 1;
            while idx < tokens.len() {
                if tokens[idx] == JsonToken::BraceClose {
                    idx += 1;
                    break;
                }
                if let JsonToken::StringVal(ref k) = tokens[idx] {
                    let k = k.clone();
                    idx += 1;
                    if idx >= tokens.len() || tokens[idx] != JsonToken::Colon {
                        return Err("Expected colon in object".to_string());
                    }
                    idx += 1;
                    let (val, next_idx) = parse_json_value(tokens, idx)?;
                    obj.push((k, val));
                    idx = next_idx;
                    if idx < tokens.len() && tokens[idx] == JsonToken::Comma {
                        idx += 1;
                    }
                } else {
                    idx += 1;
                }
            }
            Ok((JsonValue::Object(obj), idx))
        }
        _ => Err(format!("Unexpected token at {}", start)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controls_parsing() {
        let json = include_str!("../../controls.json");
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
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            mouse_pos: None,
            mouse_left: false,
            mouse_right: false,
            scroll_delta: 0.0,
            key_w: false,
            key_s: false,
            key_a: false,
            key_d: false,
            key_space: false,
            key_shift: false,
            key_ctrl: false,
            key_c: false,
            key_v: false,
            key_x: false,
            key_escape: false,
            key_p: false,
            key_f: false,
            key_e: false,
            key_q: false,
            key_g: false,
            key_m: false,
            key_f3: false,
            key_f5: false,
            key_enter: false,
            key_up: false,
            key_down: false,
            key_left: false,
            key_right: false,
            key_n: false,
            key_b: false,
            key_u: false,
            key_del: false,
            key_backspace: false,
            key_tab: false,
            key_t: false,
            key_slash: false,
            key_digits: [false; 10],
            typed_chars: Vec::new(),
            gp_left_stick_x: -0.95, // Player strafing hard left
            gp_left_stick_y: 0.0,
            gp_right_stick_x: 0.0,
            gp_right_stick_y: 0.0,
            gp_left_trigger: 0.0,
            gp_right_trigger: 0.0,
            gp_btn_south: false,
            gp_btn_east: false,
            gp_btn_west: false,
            gp_btn_north: false,
            gp_btn_lb: false,
            gp_btn_rb: false,
            gp_btn_l3: false,
            gp_btn_r3: false,
            gp_btn_start: false,
            gp_btn_select: false,
            gp_dpad_up: false,
            gp_dpad_down: false,
            gp_dpad_left: false,
            gp_dpad_right: false,
            gp_south_just_pressed: false,
            gp_east_just_pressed: false,
            gp_west_just_pressed: false,
            gp_north_just_pressed: false,
            gp_lb_just_pressed: false,
            gp_rb_just_pressed: false,
            gp_r3_just_pressed: false,
            gp_start_just_pressed: false,
            gp_select_just_pressed: false,
            gp_dpad_up_just_pressed: false,
            gp_dpad_down_just_pressed: false,
            gp_dpad_left_just_pressed: false,
            gp_dpad_right_just_pressed: false,
            gp_menu_up: false,
            gp_menu_down: false,
            gp_menu_left: false,
            gp_menu_right: false,
            gp_right_trigger_just_pressed: false,
            gp_left_trigger_just_pressed: false,
            is_mouse_captured: true,
            gp_connected: false,
        };

        manager.update(&input);
        // Left stick strafing should NOT trigger Hotbar Prev or Next
        assert!(!manager.is_active(Action::HotbarPrev));
        assert!(!manager.is_just_pressed(Action::HotbarPrev));
        assert!(!manager.is_active(Action::HotbarNext));
        assert!(!manager.is_just_pressed(Action::HotbarNext));

        // Now press LB (bumper)
        input.gp_btn_lb = true;
        input.gp_lb_just_pressed = true;
        manager.update(&input);
        assert!(manager.is_active(Action::HotbarPrev));
        assert!(manager.is_just_pressed(Action::HotbarPrev));
        assert!(!manager.is_active(Action::HotbarNext));

        // Now release LB and press RB
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
}
