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

    pub fn display_name(&self, lang: crate::core::i18n::Language) -> &'static str {
        match lang {
            crate::core::i18n::Language::Polish => match self {
                Action::MoveForward => "Ruch do przodu",
                Action::MoveBackward => "Ruch do tyłu",
                Action::StrafeLeft => "Ruch w lewo",
                Action::StrafeRight => "Ruch w prawo",
                Action::Jump => "Skok",
                Action::Sneak => "Skradanie / Kucanie",
                Action::Sprint => "Sprint / Bieg",
                Action::Inventory => "Otwórz ekwipunek",
                Action::DropItem => "Wyrzuć trzymany przedmiot",
                Action::ToggleFly => "Przełącz tryb latania",
                Action::ToggleGamemode => "Przełącz tryb gry",
                Action::Pause => "Menu gry / Pauza",
                Action::Save => "Szybki zapis świata",
                Action::AttackBreak => "Atak / Niszczenie bloku",
                Action::UsePlace => "Użycie / Postawienie bloku",
                Action::HotbarPrev => "Poprzedni slot paska",
                Action::HotbarNext => "Następny slot paska",
                Action::OpenChat => "Otwórz czat",
                Action::OpenCommand => "Otwórz wiersz poleceń",
            },
            crate::core::i18n::Language::English => match self {
                Action::MoveForward => "Move Forward",
                Action::MoveBackward => "Move Backward",
                Action::StrafeLeft => "Strafe Left",
                Action::StrafeRight => "Strafe Right",
                Action::Jump => "Jump",
                Action::Sneak => "Sneak / Crouch",
                Action::Sprint => "Sprint",
                Action::Inventory => "Open Inventory",
                Action::DropItem => "Drop Item",
                Action::ToggleFly => "Toggle Fly Mode",
                Action::ToggleGamemode => "Toggle Game Mode",
                Action::Pause => "Pause / Game Menu",
                Action::Save => "Quick Save World",
                Action::AttackBreak => "Attack / Break Block",
                Action::UsePlace => "Use / Place Block",
                Action::HotbarPrev => "Previous Hotbar Slot",
                Action::HotbarNext => "Next Hotbar Slot",
                Action::OpenChat => "Open Chat",
                Action::OpenCommand => "Open Command",
            },
        }
    }

    pub fn category(&self, lang: crate::core::i18n::Language) -> &'static str {
        match lang {
            crate::core::i18n::Language::Polish => match self {
                Action::MoveForward | Action::MoveBackward | Action::StrafeLeft | Action::StrafeRight
                | Action::Jump | Action::Sneak | Action::Sprint | Action::ToggleFly => "Ruch",
                Action::AttackBreak | Action::UsePlace | Action::Inventory | Action::DropItem | Action::ToggleGamemode => "Rozgrywka",
                Action::HotbarPrev | Action::HotbarNext | Action::Pause | Action::Save | Action::OpenChat | Action::OpenCommand => "Interfejs",
            },
            crate::core::i18n::Language::English => match self {
                Action::MoveForward | Action::MoveBackward | Action::StrafeLeft | Action::StrafeRight
                | Action::Jump | Action::Sneak | Action::Sprint | Action::ToggleFly => "Movement",
                Action::AttackBreak | Action::UsePlace | Action::Inventory | Action::DropItem | Action::ToggleGamemode => "Gameplay",
                Action::HotbarPrev | Action::HotbarNext | Action::Pause | Action::Save | Action::OpenChat | Action::OpenCommand => "Interface",
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Digits
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Function keys (F1 to F24)
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,
    // Modifiers & special keys
    Space, Shift, LeftShift, RightShift, Ctrl, LeftCtrl, RightCtrl, Alt, LeftAlt, RightAlt,
    Tab, Enter, Escape, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
    Up, Down, Left, Right,
    CapsLock, NumLock, ScrollLock,
    // Punctuation & symbols
    Minus, Equal, LeftBracket, RightBracket, Semicolon, Apostrophe, Grave, Backslash, Slash, Comma, Period,
    // Numpad
    NumPad0, NumPad1, NumPad2, NumPad3, NumPad4, NumPad5, NumPad6, NumPad7, NumPad8, NumPad9,
    NumPadDot, NumPadSlash, NumPadAsterisk, NumPadMinus, NumPadPlus, NumPadEnter,
}

impl KeyCode {
    pub const ALL: [KeyCode; 114] = [
        KeyCode::A, KeyCode::B, KeyCode::C, KeyCode::D, KeyCode::E, KeyCode::F, KeyCode::G,
        KeyCode::H, KeyCode::I, KeyCode::J, KeyCode::K, KeyCode::L, KeyCode::M, KeyCode::N,
        KeyCode::O, KeyCode::P, KeyCode::Q, KeyCode::R, KeyCode::S, KeyCode::T, KeyCode::U,
        KeyCode::V, KeyCode::W, KeyCode::X, KeyCode::Y, KeyCode::Z,
        KeyCode::Key0, KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4,
        KeyCode::Key5, KeyCode::Key6, KeyCode::Key7, KeyCode::Key8, KeyCode::Key9,
        KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4, KeyCode::F5, KeyCode::F6,
        KeyCode::F7, KeyCode::F8, KeyCode::F9, KeyCode::F10, KeyCode::F11, KeyCode::F12,
        KeyCode::F13, KeyCode::F14, KeyCode::F15, KeyCode::F16, KeyCode::F17, KeyCode::F18,
        KeyCode::F19, KeyCode::F20, KeyCode::F21, KeyCode::F22, KeyCode::F23, KeyCode::F24,
        KeyCode::Space, KeyCode::Shift, KeyCode::LeftShift, KeyCode::RightShift,
        KeyCode::Ctrl, KeyCode::LeftCtrl, KeyCode::RightCtrl,
        KeyCode::Alt, KeyCode::LeftAlt, KeyCode::RightAlt,
        KeyCode::Tab, KeyCode::Enter, KeyCode::Escape, KeyCode::Backspace, KeyCode::Delete,
        KeyCode::Insert, KeyCode::Home, KeyCode::End, KeyCode::PageUp, KeyCode::PageDown,
        KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right,
        KeyCode::CapsLock, KeyCode::NumLock, KeyCode::ScrollLock,
        KeyCode::Minus, KeyCode::Equal, KeyCode::LeftBracket, KeyCode::RightBracket,
        KeyCode::Semicolon, KeyCode::Apostrophe, KeyCode::Grave, KeyCode::Backslash, KeyCode::Slash,
        KeyCode::Comma, KeyCode::Period,
        KeyCode::NumPad0, KeyCode::NumPad1, KeyCode::NumPad2, KeyCode::NumPad3, KeyCode::NumPad4,
        KeyCode::NumPad5, KeyCode::NumPad6, KeyCode::NumPad7, KeyCode::NumPad8, KeyCode::NumPad9,
        KeyCode::NumPadDot, KeyCode::NumPadSlash, KeyCode::NumPadAsterisk, KeyCode::NumPadMinus,
        KeyCode::NumPadPlus, KeyCode::NumPadEnter,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            KeyCode::A => "A", KeyCode::B => "B", KeyCode::C => "C", KeyCode::D => "D",
            KeyCode::E => "E", KeyCode::F => "F", KeyCode::G => "G", KeyCode::H => "H",
            KeyCode::I => "I", KeyCode::J => "J", KeyCode::K => "K", KeyCode::L => "L",
            KeyCode::M => "M", KeyCode::N => "N", KeyCode::O => "O", KeyCode::P => "P",
            KeyCode::Q => "Q", KeyCode::R => "R", KeyCode::S => "S", KeyCode::T => "T",
            KeyCode::U => "U", KeyCode::V => "V", KeyCode::W => "W", KeyCode::X => "X",
            KeyCode::Y => "Y", KeyCode::Z => "Z",
            KeyCode::Key0 => "0", KeyCode::Key1 => "1", KeyCode::Key2 => "2", KeyCode::Key3 => "3",
            KeyCode::Key4 => "4", KeyCode::Key5 => "5", KeyCode::Key6 => "6", KeyCode::Key7 => "7",
            KeyCode::Key8 => "8", KeyCode::Key9 => "9",
            KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3", KeyCode::F4 => "F4",
            KeyCode::F5 => "F5", KeyCode::F6 => "F6", KeyCode::F7 => "F7", KeyCode::F8 => "F8",
            KeyCode::F9 => "F9", KeyCode::F10 => "F10", KeyCode::F11 => "F11", KeyCode::F12 => "F12",
            KeyCode::F13 => "F13", KeyCode::F14 => "F14", KeyCode::F15 => "F15", KeyCode::F16 => "F16",
            KeyCode::F17 => "F17", KeyCode::F18 => "F18", KeyCode::F19 => "F19", KeyCode::F20 => "F20",
            KeyCode::F21 => "F21", KeyCode::F22 => "F22", KeyCode::F23 => "F23", KeyCode::F24 => "F24",
            KeyCode::Space => "Spacja",
            KeyCode::Shift => "Shift",
            KeyCode::LeftShift => "LShift",
            KeyCode::RightShift => "RShift",
            KeyCode::Ctrl => "Ctrl",
            KeyCode::LeftCtrl => "LCtrl",
            KeyCode::RightCtrl => "RCtrl",
            KeyCode::Alt => "Alt",
            KeyCode::LeftAlt => "LAlt",
            KeyCode::RightAlt => "RAlt",
            KeyCode::Tab => "Tab",
            KeyCode::Enter => "Enter",
            KeyCode::Escape => "Esc",
            KeyCode::Backspace => "Backspace",
            KeyCode::Delete => "Delete",
            KeyCode::Insert => "Insert",
            KeyCode::Home => "Home",
            KeyCode::End => "End",
            KeyCode::PageUp => "PageUp",
            KeyCode::PageDown => "PageDown",
            KeyCode::Up => "Góra",
            KeyCode::Down => "Dół",
            KeyCode::Left => "Lewo",
            KeyCode::Right => "Prawo",
            KeyCode::CapsLock => "Caps Lock",
            KeyCode::NumLock => "Num Lock",
            KeyCode::ScrollLock => "Scroll Lock",
            KeyCode::Minus => "-",
            KeyCode::Equal => "=",
            KeyCode::LeftBracket => "[",
            KeyCode::RightBracket => "]",
            KeyCode::Semicolon => ";",
            KeyCode::Apostrophe => "'",
            KeyCode::Grave => "`",
            KeyCode::Backslash => "\\",
            KeyCode::Slash => "/",
            KeyCode::Comma => ",",
            KeyCode::Period => ".",
            KeyCode::NumPad0 => "Num 0", KeyCode::NumPad1 => "Num 1", KeyCode::NumPad2 => "Num 2",
            KeyCode::NumPad3 => "Num 3", KeyCode::NumPad4 => "Num 4", KeyCode::NumPad5 => "Num 5",
            KeyCode::NumPad6 => "Num 6", KeyCode::NumPad7 => "Num 7", KeyCode::NumPad8 => "Num 8",
            KeyCode::NumPad9 => "Num 9", KeyCode::NumPadDot => "Num .", KeyCode::NumPadSlash => "Num /",
            KeyCode::NumPadAsterisk => "Num *", KeyCode::NumPadMinus => "Num -", KeyCode::NumPadPlus => "Num +",
            KeyCode::NumPadEnter => "Num Enter",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            KeyCode::A => "A", KeyCode::B => "B", KeyCode::C => "C", KeyCode::D => "D",
            KeyCode::E => "E", KeyCode::F => "F", KeyCode::G => "G", KeyCode::H => "H",
            KeyCode::I => "I", KeyCode::J => "J", KeyCode::K => "K", KeyCode::L => "L",
            KeyCode::M => "M", KeyCode::N => "N", KeyCode::O => "O", KeyCode::P => "P",
            KeyCode::Q => "Q", KeyCode::R => "R", KeyCode::S => "S", KeyCode::T => "T",
            KeyCode::U => "U", KeyCode::V => "V", KeyCode::W => "W", KeyCode::X => "X",
            KeyCode::Y => "Y", KeyCode::Z => "Z",
            KeyCode::Key0 => "0", KeyCode::Key1 => "1", KeyCode::Key2 => "2", KeyCode::Key3 => "3",
            KeyCode::Key4 => "4", KeyCode::Key5 => "5", KeyCode::Key6 => "6", KeyCode::Key7 => "7",
            KeyCode::Key8 => "8", KeyCode::Key9 => "9",
            KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3", KeyCode::F4 => "F4",
            KeyCode::F5 => "F5", KeyCode::F6 => "F6", KeyCode::F7 => "F7", KeyCode::F8 => "F8",
            KeyCode::F9 => "F9", KeyCode::F10 => "F10", KeyCode::F11 => "F11", KeyCode::F12 => "F12",
            KeyCode::F13 => "F13", KeyCode::F14 => "F14", KeyCode::F15 => "F15", KeyCode::F16 => "F16",
            KeyCode::F17 => "F17", KeyCode::F18 => "F18", KeyCode::F19 => "F19", KeyCode::F20 => "F20",
            KeyCode::F21 => "F21", KeyCode::F22 => "F22", KeyCode::F23 => "F23", KeyCode::F24 => "F24",
            KeyCode::Space => "Space",
            KeyCode::Shift => "Shift",
            KeyCode::LeftShift => "LeftShift",
            KeyCode::RightShift => "RightShift",
            KeyCode::Ctrl => "Control",
            KeyCode::LeftCtrl => "LeftControl",
            KeyCode::RightCtrl => "RightControl",
            KeyCode::Alt => "Alt",
            KeyCode::LeftAlt => "LeftAlt",
            KeyCode::RightAlt => "RightAlt",
            KeyCode::Tab => "Tab",
            KeyCode::Enter => "Enter",
            KeyCode::Escape => "Escape",
            KeyCode::Backspace => "Backspace",
            KeyCode::Delete => "Delete",
            KeyCode::Insert => "Insert",
            KeyCode::Home => "Home",
            KeyCode::End => "End",
            KeyCode::PageUp => "PageUp",
            KeyCode::PageDown => "PageDown",
            KeyCode::Up => "Up",
            KeyCode::Down => "Down",
            KeyCode::Left => "Left",
            KeyCode::Right => "Right",
            KeyCode::CapsLock => "CapsLock",
            KeyCode::NumLock => "NumLock",
            KeyCode::ScrollLock => "ScrollLock",
            KeyCode::Minus => "Minus",
            KeyCode::Equal => "Equal",
            KeyCode::LeftBracket => "LeftBracket",
            KeyCode::RightBracket => "RightBracket",
            KeyCode::Semicolon => "Semicolon",
            KeyCode::Apostrophe => "Apostrophe",
            KeyCode::Grave => "Grave",
            KeyCode::Backslash => "Backslash",
            KeyCode::Slash => "Slash",
            KeyCode::Comma => "Comma",
            KeyCode::Period => "Period",
            KeyCode::NumPad0 => "NumPad0", KeyCode::NumPad1 => "NumPad1", KeyCode::NumPad2 => "NumPad2",
            KeyCode::NumPad3 => "NumPad3", KeyCode::NumPad4 => "NumPad4", KeyCode::NumPad5 => "NumPad5",
            KeyCode::NumPad6 => "NumPad6", KeyCode::NumPad7 => "NumPad7", KeyCode::NumPad8 => "NumPad8",
            KeyCode::NumPad9 => "NumPad9", KeyCode::NumPadDot => "NumPadDot", KeyCode::NumPadSlash => "NumPadSlash",
            KeyCode::NumPadAsterisk => "NumPadAsterisk", KeyCode::NumPadMinus => "NumPadMinus", KeyCode::NumPadPlus => "NumPadPlus",
            KeyCode::NumPadEnter => "NumPadEnter",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "a" => Some(KeyCode::A), "b" => Some(KeyCode::B), "c" => Some(KeyCode::C),
            "d" => Some(KeyCode::D), "e" => Some(KeyCode::E), "f" => Some(KeyCode::F),
            "g" => Some(KeyCode::G), "h" => Some(KeyCode::H), "i" => Some(KeyCode::I),
            "j" => Some(KeyCode::J), "k" => Some(KeyCode::K), "l" => Some(KeyCode::L),
            "m" => Some(KeyCode::M), "n" => Some(KeyCode::N), "o" => Some(KeyCode::O),
            "p" => Some(KeyCode::P), "q" => Some(KeyCode::Q), "r" => Some(KeyCode::R),
            "s" => Some(KeyCode::S), "t" => Some(KeyCode::T), "u" => Some(KeyCode::U),
            "v" => Some(KeyCode::V), "w" => Some(KeyCode::W), "x" => Some(KeyCode::X),
            "y" => Some(KeyCode::Y), "z" => Some(KeyCode::Z),

            "0" | "key0" => Some(KeyCode::Key0), "1" | "key1" => Some(KeyCode::Key1),
            "2" | "key2" => Some(KeyCode::Key2), "3" | "key3" => Some(KeyCode::Key3),
            "4" | "key4" => Some(KeyCode::Key4), "5" | "key5" => Some(KeyCode::Key5),
            "6" | "key6" => Some(KeyCode::Key6), "7" | "key7" => Some(KeyCode::Key7),
            "8" | "key8" => Some(KeyCode::Key8), "9" | "key9" => Some(KeyCode::Key9),

            "f1" => Some(KeyCode::F1), "f2" => Some(KeyCode::F2), "f3" => Some(KeyCode::F3),
            "f4" => Some(KeyCode::F4), "f5" => Some(KeyCode::F5), "f6" => Some(KeyCode::F6),
            "f7" => Some(KeyCode::F7), "f8" => Some(KeyCode::F8), "f9" => Some(KeyCode::F9),
            "f10" => Some(KeyCode::F10), "f11" => Some(KeyCode::F11), "f12" => Some(KeyCode::F12),
            "f13" => Some(KeyCode::F13), "f14" => Some(KeyCode::F14), "f15" => Some(KeyCode::F15),
            "f16" => Some(KeyCode::F16), "f17" => Some(KeyCode::F17), "f18" => Some(KeyCode::F18),
            "f19" => Some(KeyCode::F19), "f20" => Some(KeyCode::F20), "f21" => Some(KeyCode::F21),
            "f22" => Some(KeyCode::F22), "f23" => Some(KeyCode::F23), "f24" => Some(KeyCode::F24),

            "space" => Some(KeyCode::Space),
            "shift" => Some(KeyCode::Shift),
            "leftshift" | "lshift" => Some(KeyCode::LeftShift),
            "rightshift" | "rshift" => Some(KeyCode::RightShift),
            "control" | "ctrl" => Some(KeyCode::Ctrl),
            "leftcontrol" | "lctrl" => Some(KeyCode::LeftCtrl),
            "rightcontrol" | "rctrl" => Some(KeyCode::RightCtrl),
            "alt" => Some(KeyCode::Alt),
            "leftalt" | "lalt" => Some(KeyCode::LeftAlt),
            "rightalt" | "ralt" => Some(KeyCode::RightAlt),

            "escape" | "esc" => Some(KeyCode::Escape),
            "enter" | "return" => Some(KeyCode::Enter),
            "tab" => Some(KeyCode::Tab),
            "backspace" => Some(KeyCode::Backspace),
            "delete" | "del" => Some(KeyCode::Delete),
            "insert" | "ins" => Some(KeyCode::Insert),
            "home" => Some(KeyCode::Home),
            "end" => Some(KeyCode::End),
            "pageup" | "pgup" => Some(KeyCode::PageUp),
            "pagedown" | "pgdn" => Some(KeyCode::PageDown),

            "up" => Some(KeyCode::Up),
            "down" => Some(KeyCode::Down),
            "left" => Some(KeyCode::Left),
            "right" => Some(KeyCode::Right),

            "capslock" | "caps" => Some(KeyCode::CapsLock),
            "numlock" => Some(KeyCode::NumLock),
            "scrolllock" => Some(KeyCode::ScrollLock),

            "-" | "minus" => Some(KeyCode::Minus),
            "=" | "equal" | "equals" => Some(KeyCode::Equal),
            "[" | "leftbracket" => Some(KeyCode::LeftBracket),
            "]" | "rightbracket" => Some(KeyCode::RightBracket),
            ";" | "semicolon" => Some(KeyCode::Semicolon),
            "'" | "apostrophe" | "quote" => Some(KeyCode::Apostrophe),
            "`" | "grave" | "backquote" | "tilde" => Some(KeyCode::Grave),
            "\\" | "backslash" => Some(KeyCode::Backslash),
            "/" | "slash" => Some(KeyCode::Slash),
            "," | "comma" => Some(KeyCode::Comma),
            "." | "period" | "dot" => Some(KeyCode::Period),

            "numpad0" => Some(KeyCode::NumPad0), "numpad1" => Some(KeyCode::NumPad1),
            "numpad2" => Some(KeyCode::NumPad2), "numpad3" => Some(KeyCode::NumPad3),
            "numpad4" => Some(KeyCode::NumPad4), "numpad5" => Some(KeyCode::NumPad5),
            "numpad6" => Some(KeyCode::NumPad6), "numpad7" => Some(KeyCode::NumPad7),
            "numpad8" => Some(KeyCode::NumPad8), "numpad9" => Some(KeyCode::NumPad9),
            "numpaddot" => Some(KeyCode::NumPadDot), "numpadslash" => Some(KeyCode::NumPadSlash),
            "numpadasterisk" | "numpadmultiply" => Some(KeyCode::NumPadAsterisk),
            "numpadminus" => Some(KeyCode::NumPadMinus), "numpadplus" => Some(KeyCode::NumPadPlus),
            "numpadenter" => Some(KeyCode::NumPadEnter),
            _ => None,
        }
    }

    #[inline]
    pub fn is_down(&self, input: &InputState) -> bool {
        if input.is_key_down(*self) {
            return true;
        }
        // Aliases and legacy bool fallbacks
        match self {
            KeyCode::W => input.key_w,
            KeyCode::S => input.key_s,
            KeyCode::A => input.key_a,
            KeyCode::D => input.key_d,
            KeyCode::Space => input.key_space,
            KeyCode::Shift | KeyCode::LeftShift | KeyCode::RightShift => input.key_shift,
            KeyCode::Ctrl | KeyCode::LeftCtrl | KeyCode::RightCtrl => input.key_ctrl,
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
            KeyCode::F3 => input.key_f3,
            KeyCode::F5 => input.key_f5,
            KeyCode::Enter => input.key_enter,
            KeyCode::Up => input.key_up,
            KeyCode::Down => input.key_down,
            KeyCode::Left => input.key_left,
            KeyCode::Right => input.key_right,
            KeyCode::N => input.key_n,
            KeyCode::B => input.key_b,
            KeyCode::U => input.key_u,
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
            _ => false,
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

            out.push_str("    \"keys\": [");
            for (ki, k) in b.keys.iter().enumerate() {
                if ki > 0 { out.push_str(", "); }
                out.push_str(&format!("\"{}\"", k.as_str()));
            }
            out.push_str("],\n");

            out.push_str("    \"gamepad\": [");
            for (gi, g) in b.gamepad.iter().enumerate() {
                if gi > 0 { out.push_str(", "); }
                out.push_str(&format!("\"{}\"", g.as_str()));
            }
            out.push_str("],\n");

            out.push_str("    \"mouse\": [");
            for (mi, m) in b.mouse.iter().enumerate() {
                if mi > 0 { out.push_str(", "); }
                out.push_str(&format!("\"{}\"", m.as_str()));
            }
            out.push_str("]\n");

            if i + 1 < Action::ALL.len() {
                out.push_str("  },\n");
            } else {
                out.push_str("  }\n");
            }
        }
        out.push_str("}\n");
        out
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let json = self.to_json();
        std::fs::write("controls.json", json)
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
            mouse_middle: false,
            mouse_4: false,
            mouse_5: false,
            mouse_extra: [false; 4],
            scroll_delta: 0.0,
            keys_down: std::collections::HashSet::new(),
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
