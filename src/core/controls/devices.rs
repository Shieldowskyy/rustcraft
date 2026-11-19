//! Gamepad and mouse button/wheel input definitions with `InputState` queries.

use crate::app::InputState;

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
    pub const ALL: [GamepadInput; 16] = [
        GamepadInput::South,
        GamepadInput::East,
        GamepadInput::West,
        GamepadInput::North,
        GamepadInput::LB,
        GamepadInput::RB,
        GamepadInput::LeftTrigger,
        GamepadInput::RightTrigger,
        GamepadInput::L3,
        GamepadInput::R3,
        GamepadInput::Start,
        GamepadInput::Select,
        GamepadInput::DpadUp,
        GamepadInput::DpadDown,
        GamepadInput::DpadLeft,
        GamepadInput::DpadRight,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            GamepadInput::South => "A / Krzyżyk",
            GamepadInput::East => "B / Kółko",
            GamepadInput::West => "X / Kwadrat",
            GamepadInput::North => "Y / Trójkąt",
            GamepadInput::LB => "LB / L1",
            GamepadInput::RB => "RB / R1",
            GamepadInput::LeftTrigger => "LT / L2",
            GamepadInput::RightTrigger => "RT / R2",
            GamepadInput::L3 => "L3 (Gałka)",
            GamepadInput::R3 => "R3 (Gałka)",
            GamepadInput::Start => "Start / Menu",
            GamepadInput::Select => "Select / Podgląd",
            GamepadInput::DpadUp => "D-Pad Góra",
            GamepadInput::DpadDown => "D-Pad Dół",
            GamepadInput::DpadLeft => "D-Pad Lewo",
            GamepadInput::DpadRight => "D-Pad Prawo",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GamepadInput::South => "South",
            GamepadInput::East => "East",
            GamepadInput::West => "West",
            GamepadInput::North => "North",
            GamepadInput::LB => "LB",
            GamepadInput::RB => "RB",
            GamepadInput::LeftTrigger => "LeftTrigger",
            GamepadInput::RightTrigger => "RightTrigger",
            GamepadInput::L3 => "L3",
            GamepadInput::R3 => "R3",
            GamepadInput::Start => "Start",
            GamepadInput::Select => "Select",
            GamepadInput::DpadUp => "DpadUp",
            GamepadInput::DpadDown => "DpadDown",
            GamepadInput::DpadLeft => "DpadLeft",
            GamepadInput::DpadRight => "DpadRight",
        }
    }

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
    Middle,
    WheelUp,
    WheelDown,
    Mouse4,
    Mouse5,
    Mouse6,
    Mouse7,
    Mouse8,
}

impl MouseInput {
    pub const ALL: [MouseInput; 10] = [
        MouseInput::Left,
        MouseInput::Right,
        MouseInput::Middle,
        MouseInput::WheelUp,
        MouseInput::WheelDown,
        MouseInput::Mouse4,
        MouseInput::Mouse5,
        MouseInput::Mouse6,
        MouseInput::Mouse7,
        MouseInput::Mouse8,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            MouseInput::Left => "LPM",
            MouseInput::Right => "PPM",
            MouseInput::Middle => "ŚPM (Kółko)",
            MouseInput::WheelUp => "Kółko w górę",
            MouseInput::WheelDown => "Kółko w dół",
            MouseInput::Mouse4 => "Mysz 4 (Wstecz)",
            MouseInput::Mouse5 => "Mysz 5 (Dalej)",
            MouseInput::Mouse6 => "Mysz 6",
            MouseInput::Mouse7 => "Mysz 7",
            MouseInput::Mouse8 => "Mysz 8",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            MouseInput::Left => "Left",
            MouseInput::Right => "Right",
            MouseInput::Middle => "Middle",
            MouseInput::WheelUp => "WheelUp",
            MouseInput::WheelDown => "WheelDown",
            MouseInput::Mouse4 => "Mouse4",
            MouseInput::Mouse5 => "Mouse5",
            MouseInput::Mouse6 => "Mouse6",
            MouseInput::Mouse7 => "Mouse7",
            MouseInput::Mouse8 => "Mouse8",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "left" | "mouse1" | "lmb" => Some(MouseInput::Left),
            "right" | "mouse2" | "rmb" => Some(MouseInput::Right),
            "middle" | "mouse3" | "mmb" => Some(MouseInput::Middle),
            "wheelup" | "scrollup" => Some(MouseInput::WheelUp),
            "wheeldown" | "scrolldown" => Some(MouseInput::WheelDown),
            "mouse4" | "thumb1" | "back" => Some(MouseInput::Mouse4),
            "mouse5" | "thumb2" | "forward" => Some(MouseInput::Mouse5),
            "mouse6" => Some(MouseInput::Mouse6),
            "mouse7" => Some(MouseInput::Mouse7),
            "mouse8" => Some(MouseInput::Mouse8),
            _ => None,
        }
    }

    #[inline]
    pub fn is_down(&self, input: &InputState) -> bool {
        match self {
            MouseInput::Left => input.mouse_left,
            MouseInput::Right => input.mouse_right,
            MouseInput::Middle => input.mouse_middle,
            MouseInput::WheelUp => input.scroll_delta > 0.0,
            MouseInput::WheelDown => input.scroll_delta < 0.0,
            MouseInput::Mouse4 => input.mouse_4,
            MouseInput::Mouse5 => input.mouse_5,
            MouseInput::Mouse6 => input.mouse_extra[0],
            MouseInput::Mouse7 => input.mouse_extra[1],
            MouseInput::Mouse8 => input.mouse_extra[2],
        }
    }
}

