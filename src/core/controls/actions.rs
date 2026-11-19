//! Action identifiers and per-action binding structs for keyboard, mouse, and gamepad.
//! Defines the 21 logical input actions and localized Polish/English labels.

pub const ACTION_COUNT: usize = 21;

macro_rules! define_actions {
    ($(($idx:literal, $var:ident, $str:literal, $($pat:literal)|+, $pl:literal, $en:literal)),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Action {
            $($var = $idx,)*
        }

        impl Action {
            pub const ALL: [Action; ACTION_COUNT] = [$(Action::$var),*];

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Action::$var => $str,)*
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                let s = s.trim().to_ascii_lowercase();
                match s.as_str() {
                    $($($pat)|+ => Some(Action::$var),)*
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
                        $(Action::$var => $pl,)*
                    },
                    crate::core::i18n::Language::English => match self {
                        $(Action::$var => $en,)*
                    },
                }
            }
        }
    };
}

define_actions! {
    (0, MoveForward, "move_forward", "move_forward" | "forward", "Ruch do przodu", "Move Forward"),
    (1, MoveBackward, "move_backward", "move_backward" | "backward", "Ruch do tyłu", "Move Backward"),
    (2, StrafeLeft, "strafe_left", "strafe_left" | "left", "Ruch w lewo", "Strafe Left"),
    (3, StrafeRight, "strafe_right", "strafe_right" | "right", "Ruch w prawo", "Strafe Right"),
    (4, Jump, "jump", "jump", "Skok", "Jump"),
    (5, Sneak, "sneak", "sneak" | "crouch", "Skradanie / Kucanie", "Sneak / Crouch"),
    (6, Sprint, "sprint", "sprint" | "run", "Sprint / Bieg", "Sprint"),
    (7, Inventory, "inventory", "inventory", "Otwórz ekwipunek", "Open Inventory"),
    (8, DropItem, "drop_item", "drop_item" | "drop", "Wyrzuć trzymany przedmiot", "Drop Item"),
    (9, ToggleFly, "toggle_fly", "toggle_fly" | "fly", "Przełącz tryb latania", "Toggle Fly Mode"),
    (10, ToggleGamemode, "toggle_gamemode", "toggle_gamemode" | "gamemode", "Przełącz tryb gry", "Toggle Game Mode"),
    (11, Pause, "pause", "pause" | "menu", "Menu gry / Pauza", "Pause / Game Menu"),
    (12, Save, "save", "save" | "quick_save", "Szybki zapis świata", "Quick Save World"),
    (13, AttackBreak, "attack_break", "attack_break" | "attack" | "break", "Atak / Niszczenie bloku", "Attack / Break Block"),
    (14, UsePlace, "use_place", "use_place" | "use" | "place", "Użycie / Postawienie bloku", "Use / Place Block"),
    (15, HotbarPrev, "hotbar_prev", "hotbar_prev" | "prev_slot", "Poprzedni slot paska", "Previous Hotbar Slot"),
    (16, HotbarNext, "hotbar_next", "hotbar_next" | "next_slot", "Następny slot paska", "Next Hotbar Slot"),
    (17, OpenChat, "open_chat", "open_chat" | "chat", "Otwórz czat", "Open Chat"),
    (18, OpenCommand, "open_command", "open_command" | "command" | "slash", "Otwórz wiersz poleceń", "Open Command"),
    (19, PickBlock, "pick_block", "pick_block" | "pick", "Wybierz blok (Pick Block)", "Pick Block"),
    (20, Questbook, "questbook", "questbook" | "quests" | "journal", "Księga zadań (Questbook)", "Questbook"),
}

impl Action {
    pub fn category(&self, lang: crate::core::i18n::Language) -> &'static str {
        match lang {
            crate::core::i18n::Language::Polish => match self {
                Action::MoveForward | Action::MoveBackward | Action::StrafeLeft | Action::StrafeRight
                | Action::Jump | Action::Sneak | Action::Sprint | Action::ToggleFly => "Ruch",
                Action::AttackBreak | Action::UsePlace | Action::PickBlock | Action::Inventory | Action::DropItem | Action::ToggleGamemode => "Rozgrywka",
                Action::HotbarPrev | Action::HotbarNext | Action::Pause | Action::Save | Action::OpenChat | Action::OpenCommand | Action::Questbook => "Interfejs",
            },
            crate::core::i18n::Language::English => match self {
                Action::MoveForward | Action::MoveBackward | Action::StrafeLeft | Action::StrafeRight
                | Action::Jump | Action::Sneak | Action::Sprint | Action::ToggleFly => "Movement",
                Action::AttackBreak | Action::UsePlace | Action::PickBlock | Action::Inventory | Action::DropItem | Action::ToggleGamemode => "Gameplay",
                Action::HotbarPrev | Action::HotbarNext | Action::Pause | Action::Save | Action::OpenChat | Action::OpenCommand | Action::Questbook => "Interface",
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ActionBinding {
    pub keys: Vec<super::KeyCode>,
    pub gamepad: Vec<super::GamepadInput>,
    pub mouse: Vec<super::MouseInput>,
}

