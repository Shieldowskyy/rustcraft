//! Logical keyboard key codes and `InputState` polling helpers.
//! Pure value enum with string serialization for `controls.json`.

use crate::app::InputState;

macro_rules! define_keycodes {
    ($(($var:ident, $disp:literal, $str:literal, $($pat:literal)|+)),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum KeyCode {
            $($var,)*
        }

        impl KeyCode {
            pub const ALL: [KeyCode; 114] = [$(KeyCode::$var),*];

            pub fn display_name(&self) -> &'static str {
                match self {
                    $(KeyCode::$var => $disp,)*
                }
            }

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(KeyCode::$var => $str,)*
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                let s = s.trim().to_ascii_lowercase();
                match s.as_str() {
                    $($($pat)|+ => Some(KeyCode::$var),)*
                    _ => None,
                }
            }
        }
    };
}

define_keycodes! {
    (A, "A", "A", "a"),
    (B, "B", "B", "b"),
    (C, "C", "C", "c"),
    (D, "D", "D", "d"),
    (E, "E", "E", "e"),
    (F, "F", "F", "f"),
    (G, "G", "G", "g"),
    (H, "H", "H", "h"),
    (I, "I", "I", "i"),
    (J, "J", "J", "j"),
    (K, "K", "K", "k"),
    (L, "L", "L", "l"),
    (M, "M", "M", "m"),
    (N, "N", "N", "n"),
    (O, "O", "O", "o"),
    (P, "P", "P", "p"),
    (Q, "Q", "Q", "q"),
    (R, "R", "R", "r"),
    (S, "S", "S", "s"),
    (T, "T", "T", "t"),
    (U, "U", "U", "u"),
    (V, "V", "V", "v"),
    (W, "W", "W", "w"),
    (X, "X", "X", "x"),
    (Y, "Y", "Y", "y"),
    (Z, "Z", "Z", "z"),
    (Key0, "0", "0", "0" | "key0"),
    (Key1, "1", "1", "1" | "key1"),
    (Key2, "2", "2", "2" | "key2"),
    (Key3, "3", "3", "3" | "key3"),
    (Key4, "4", "4", "4" | "key4"),
    (Key5, "5", "5", "5" | "key5"),
    (Key6, "6", "6", "6" | "key6"),
    (Key7, "7", "7", "7" | "key7"),
    (Key8, "8", "8", "8" | "key8"),
    (Key9, "9", "9", "9" | "key9"),
    (F1, "F1", "F1", "f1"),
    (F2, "F2", "F2", "f2"),
    (F3, "F3", "F3", "f3"),
    (F4, "F4", "F4", "f4"),
    (F5, "F5", "F5", "f5"),
    (F6, "F6", "F6", "f6"),
    (F7, "F7", "F7", "f7"),
    (F8, "F8", "F8", "f8"),
    (F9, "F9", "F9", "f9"),
    (F10, "F10", "F10", "f10"),
    (F11, "F11", "F11", "f11"),
    (F12, "F12", "F12", "f12"),
    (F13, "F13", "F13", "f13"),
    (F14, "F14", "F14", "f14"),
    (F15, "F15", "F15", "f15"),
    (F16, "F16", "F16", "f16"),
    (F17, "F17", "F17", "f17"),
    (F18, "F18", "F18", "f18"),
    (F19, "F19", "F19", "f19"),
    (F20, "F20", "F20", "f20"),
    (F21, "F21", "F21", "f21"),
    (F22, "F22", "F22", "f22"),
    (F23, "F23", "F23", "f23"),
    (F24, "F24", "F24", "f24"),
    (Space, "Spacja", "Space", "space"),
    (Shift, "Shift", "Shift", "shift"),
    (LeftShift, "LShift", "LeftShift", "leftshift" | "lshift"),
    (RightShift, "RShift", "RightShift", "rightshift" | "rshift"),
    (Ctrl, "Ctrl", "Control", "control" | "ctrl"),
    (LeftCtrl, "LCtrl", "LeftControl", "leftcontrol" | "lctrl"),
    (RightCtrl, "RCtrl", "RightControl", "rightcontrol" | "rctrl"),
    (Alt, "Alt", "Alt", "alt"),
    (LeftAlt, "LAlt", "LeftAlt", "leftalt" | "lalt"),
    (RightAlt, "RAlt", "RightAlt", "rightalt" | "ralt"),
    (Tab, "Tab", "Tab", "tab"),
    (Enter, "Enter", "Enter", "enter" | "return"),
    (Escape, "Esc", "Escape", "escape" | "esc"),
    (Backspace, "Backspace", "Backspace", "backspace"),
    (Delete, "Delete", "Delete", "delete" | "del"),
    (Insert, "Insert", "Insert", "insert" | "ins"),
    (Home, "Home", "Home", "home"),
    (End, "End", "End", "end"),
    (PageUp, "PageUp", "PageUp", "pageup" | "pgup"),
    (PageDown, "PageDown", "PageDown", "pagedown" | "pgdn"),
    (Up, "Góra", "Up", "up"),
    (Down, "Dół", "Down", "down"),
    (Left, "Lewo", "Left", "left"),
    (Right, "Prawo", "Right", "right"),
    (CapsLock, "Caps Lock", "CapsLock", "capslock" | "caps"),
    (NumLock, "Num Lock", "NumLock", "numlock"),
    (ScrollLock, "Scroll Lock", "ScrollLock", "scrolllock"),
    (Minus, "-", "Minus", "-" | "minus"),
    (Equal, "=", "Equal", "=" | "equal" | "equals"),
    (LeftBracket, "[", "LeftBracket", "[" | "leftbracket"),
    (RightBracket, "]", "RightBracket", "]" | "rightbracket"),
    (Semicolon, ";", "Semicolon", ";" | "semicolon"),
    (Apostrophe, "'", "Apostrophe", "'" | "apostrophe" | "quote"),
    (Grave, "`", "Grave", "`" | "grave" | "backquote" | "tilde"),
    (Backslash, "\\", "Backslash", "\\" | "backslash"),
    (Slash, "/", "Slash", "/" | "slash"),
    (Comma, ",", "Comma", "," | "comma"),
    (Period, ".", "Period", "." | "period" | "dot"),
    (NumPad0, "Num 0", "NumPad0", "numpad0"),
    (NumPad1, "Num 1", "NumPad1", "numpad1"),
    (NumPad2, "Num 2", "NumPad2", "numpad2"),
    (NumPad3, "Num 3", "NumPad3", "numpad3"),
    (NumPad4, "Num 4", "NumPad4", "numpad4"),
    (NumPad5, "Num 5", "NumPad5", "numpad5"),
    (NumPad6, "Num 6", "NumPad6", "numpad6"),
    (NumPad7, "Num 7", "NumPad7", "numpad7"),
    (NumPad8, "Num 8", "NumPad8", "numpad8"),
    (NumPad9, "Num 9", "NumPad9", "numpad9"),
    (NumPadDot, "Num .", "NumPadDot", "numpaddot"),
    (NumPadSlash, "Num /", "NumPadSlash", "numpadslash"),
    (NumPadAsterisk, "Num *", "NumPadAsterisk", "numpadasterisk" | "numpadmultiply"),
    (NumPadMinus, "Num -", "NumPadMinus", "numpadminus"),
    (NumPadPlus, "Num +", "NumPadPlus", "numpadplus"),
    (NumPadEnter, "Num Enter", "NumPadEnter", "numpadenter"),
}

impl KeyCode {
    #[inline]
    pub fn is_down(&self, input: &InputState) -> bool {
        if input.is_key_down(*self) {
            return true;
        }
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

