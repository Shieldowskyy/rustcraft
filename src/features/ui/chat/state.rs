//! Stan czatu, historia wiadomości, obsługa wejścia klawiatury oraz autouzupełnianie (`Tab`).

use crate::core::controls::KeyCode;
use crate::core::math::Vec3;
use crate::features::entities::{ItemEntity, MobManager, ParticleSystem};
use crate::features::player::{Camera, Player};
use crate::features::ui::draw::pack_color;
use crate::features::world::World;
use crate::platform::clipboard::{get_clipboard, set_clipboard};

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub text: String,
    pub color: u32,
    pub age: f32,
}

pub enum ChatAction {
    Execute(String),
    Close,
}

pub struct CommandContext<'a> {
    pub player: &'a mut Player,
    pub camera: &'a mut Camera,
    pub world: &'a mut World,
    pub mob_manager: &'a mut MobManager,
    pub items: &'a mut Vec<ItemEntity>,
    pub particles: &'a mut ParticleSystem,
    pub is_flying: &'a mut bool,
    pub fly_velocity: &'a mut Vec3,
    pub horiz_velocity: &'a mut Vec3,
    pub reload_chunks_requested: &'a mut bool,
}

#[derive(Clone, Copy)]
pub(super) struct ChatContextMenu {
    pub x: i32,
    pub y: i32,
    pub target: ChatContextTarget,
}

#[derive(Clone, Copy)]
pub(super) enum ChatContextTarget {
    Input,
    Message(usize),
}

pub struct ChatState {
    pub is_open: bool,
    pub input_text: String,
    pub cursor_pos: usize,
    pub messages: Vec<ChatMessage>,
    pub history: Vec<String>,
    pub history_idx: Option<usize>,
    pub cursor_blink: f32,
    pub saved_draft: String,
    pub selection_anchor: Option<(usize, usize)>,
    pub selection_focus: Option<(usize, usize)>,
    pub(super) input_selection: Option<(usize, usize)>,
    pub(super) context_menu: Option<ChatContextMenu>,
    pub(super) selecting_text: bool,
    pub(super) was_mouse_left: bool,
    pub(super) was_mouse_right: bool,
    pub(super) was_copy: bool,
    pub(super) was_paste: bool,
    pub(super) was_select_all: bool,
    pub(super) was_backspace: bool,
    pub(super) was_delete: bool,
    pub(super) was_left: bool,
    pub(super) was_right: bool,
    pub(super) was_up: bool,
    pub(super) was_down: bool,
    pub(super) was_enter: bool,
    pub(super) was_escape: bool,
    pub(super) was_tab: bool,
    pub(super) tab_candidates: Vec<String>,
    pub(super) tab_candidate_index: usize,
    pub(super) last_tab_completion: Option<(usize, String)>,
    pub ignore_slash_until_release: bool,
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            is_open: false,
            input_text: String::new(),
            cursor_pos: 0,
            messages: Vec::new(),
            history: Vec::new(),
            history_idx: None,
            cursor_blink: 0.0,
            saved_draft: String::new(),
            selection_anchor: None,
            selection_focus: None,
            input_selection: None,
            context_menu: None,
            selecting_text: false,
            was_mouse_left: false,
            was_mouse_right: false,
            was_copy: false,
            was_paste: false,
            was_select_all: false,
            was_backspace: false,
            was_delete: false,
            was_left: false,
            was_right: false,
            was_up: false,
            was_down: false,
            was_enter: false,
            was_escape: false,
            was_tab: false,
            tab_candidates: Vec::new(),
            tab_candidate_index: 0,
            last_tab_completion: None,
            ignore_slash_until_release: false,
        }
    }

    pub fn open(&mut self, initial_text: &str) {
        self.is_open = true;
        self.input_text = initial_text.to_string();
        self.cursor_pos = self.input_text.len();
        self.history_idx = None;
        self.cursor_blink = 0.0;
        self.saved_draft.clear();
        self.ignore_slash_until_release = initial_text.starts_with('/');
        self.was_tab = false;
        self.tab_candidates.clear();
        self.last_tab_completion = None;
        self.selection_anchor = None;
        self.selection_focus = None;
        self.input_selection = None;
        self.context_menu = None;
        self.selecting_text = false;
        crate::log_debug!("Chat", "Chat opened with prefix: '{}'", initial_text);
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.input_text.clear();
        self.cursor_pos = 0;
        self.history_idx = None;
        self.ignore_slash_until_release = false;
        self.was_tab = false;
        self.tab_candidates.clear();
        self.last_tab_completion = None;
        self.selection_anchor = None;
        self.selection_focus = None;
        self.input_selection = None;
        self.context_menu = None;
        self.selecting_text = false;
        crate::log_debug!("Chat", "Chat closed");
    }

    pub fn add_message(&mut self, text: impl Into<String>, color: u32) {
        let text = text.into();
        crate::log_info!("Chat", "{}", text);
        self.messages.push(ChatMessage {
            text,
            color,
            age: 0.0,
        });
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }

    pub fn add_info(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(1.0, 1.0, 0.4));
    }

    pub fn add_success(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(0.4, 1.0, 0.4));
    }

    pub fn add_error(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(1.0, 0.35, 0.35));
    }

    pub fn add_player_chat(&mut self, player_name: &str, msg: &str) {
        let line = format!("<{}> {}", player_name, msg);
        self.add_message(line, pack_color(1.0, 1.0, 1.0));
    }

    pub fn update(&mut self, dt: f32) {
        self.cursor_blink += dt;
        for msg in &mut self.messages {
            msg.age += dt;
        }
    }

    pub fn handle_input(
        &mut self,
        input: &crate::app::InputState,
        player_names: &[String],
    ) -> Option<ChatAction> {
        if !self.is_open {
            return None;
        }

        let tab_pressed = input.key_tab && !self.was_tab;
        self.was_tab = input.key_tab;

        let enter_pressed = input.key_enter && !self.was_enter;
        self.was_enter = input.key_enter;
        if enter_pressed {
            let submitted = self.input_text.trim().to_string();
            if !submitted.is_empty() {
                self.history.push(submitted.clone());
                if self.history.len() > 50 {
                    self.history.remove(0);
                }
                self.close();
                return Some(ChatAction::Execute(submitted));
            } else {
                self.close();
                return Some(ChatAction::Close);
            }
        }

        let esc_pressed = input.key_escape && !self.was_escape;
        self.was_escape = input.key_escape;
        if esc_pressed {
            self.close();
            return Some(ChatAction::Close);
        }

        if tab_pressed {
            self.complete_player_name(player_names);
        }

        let control_down = input.key_ctrl
            || input.is_key_down(KeyCode::Ctrl)
            || input.is_key_down(KeyCode::LeftCtrl)
            || input.is_key_down(KeyCode::RightCtrl);
        let copy_down = input.key_c || input.is_key_down(KeyCode::C);
        let copy_pressed = control_down && copy_down && !self.was_copy;
        self.was_copy = control_down && copy_down;
        if copy_pressed {
            if let Some(selected) = self.selected_text().or_else(|| self.selected_input_text()) {
                set_clipboard(&selected);
            }
        }
        let select_all_down = control_down && (input.key_a || input.is_key_down(KeyCode::A));
        if select_all_down && !self.was_select_all {
            self.input_selection = Some((0, self.input_text.len()));
            self.selection_anchor = None;
            self.selection_focus = None;
        }
        self.was_select_all = select_all_down;

        let paste_down = control_down && (input.key_v || input.is_key_down(KeyCode::V));
        let paste_pressed = paste_down && !self.was_paste;
        self.was_paste = paste_down;
        if paste_pressed {
            self.paste_into_input(&get_clipboard());
        }

        let up_pressed = input.key_up && !self.was_up;
        self.was_up = input.key_up;
        let down_pressed = input.key_down && !self.was_down;
        self.was_down = input.key_down;

        if up_pressed && !self.history.is_empty() {
            self.input_selection = None;
            if self.history_idx.is_none() {
                self.saved_draft = self.input_text.clone();
                let new_idx = self.history.len() - 1;
                self.history_idx = Some(new_idx);
                self.input_text = self.history[new_idx].clone();
                self.cursor_pos = self.input_text.len();
            } else if let Some(idx) = self.history_idx {
                if idx > 0 {
                    let new_idx = idx - 1;
                    self.history_idx = Some(new_idx);
                    self.input_text = self.history[new_idx].clone();
                    self.cursor_pos = self.input_text.len();
                }
            }
            self.cursor_blink = 0.0;
        } else if down_pressed && self.history_idx.is_some() {
            self.input_selection = None;
            let idx = self.history_idx.unwrap();
            if idx + 1 < self.history.len() {
                let new_idx = idx + 1;
                self.history_idx = Some(new_idx);
                self.input_text = self.history[new_idx].clone();
                self.cursor_pos = self.input_text.len();
            } else {
                self.history_idx = None;
                self.input_text = self.saved_draft.clone();
                self.cursor_pos = self.input_text.len();
            }
            self.cursor_blink = 0.0;
        }

        let left_pressed = input.key_left && !self.was_left;
        self.was_left = input.key_left;
        if left_pressed {
            if let Some((start, _)) = self.input_selection.take() {
                self.cursor_pos = start;
            } else if self.cursor_pos > 0 {
                self.cursor_pos = self.input_text[..self.cursor_pos]
                    .char_indices()
                    .last()
                    .map(|(index, _)| index)
                    .unwrap_or(0);
            }
            self.cursor_blink = 0.0;
        }

        let right_pressed = input.key_right && !self.was_right;
        self.was_right = input.key_right;
        if right_pressed {
            if let Some((_, end)) = self.input_selection.take() {
                self.cursor_pos = end;
            } else if self.cursor_pos < self.input_text.len() {
                self.cursor_pos = self.input_text[self.cursor_pos..]
                    .char_indices()
                    .nth(1)
                    .map(|(offset, _)| self.cursor_pos + offset)
                    .unwrap_or(self.input_text.len());
            }
            self.cursor_blink = 0.0;
        }

        let bs_pressed = input.key_backspace && !self.was_backspace;
        self.was_backspace = input.key_backspace;
        if bs_pressed && self.input_selection.is_some() {
            self.delete_input_selection();
            self.cursor_blink = 0.0;
        } else if bs_pressed && self.cursor_pos > 0 {
            let previous = self.input_text[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(index, _)| index)
                .unwrap_or(0);
            self.input_text.replace_range(previous..self.cursor_pos, "");
            self.cursor_pos = previous;
            self.cursor_blink = 0.0;
        }

        let del_pressed = input.key_del && !self.was_delete;
        self.was_delete = input.key_del;
        if del_pressed && self.input_selection.is_some() {
            self.delete_input_selection();
            self.cursor_blink = 0.0;
        } else if del_pressed && self.cursor_pos < self.input_text.len() {
            let next = self.input_text[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(offset, _)| self.cursor_pos + offset)
                .unwrap_or(self.input_text.len());
            self.input_text.replace_range(self.cursor_pos..next, "");
            self.cursor_blink = 0.0;
        }

        if self.ignore_slash_until_release && !input.key_slash {
            self.ignore_slash_until_release = false;
        }

        for &ch in input
            .typed_chars
            .iter()
            .filter(|_| !copy_pressed && !paste_pressed && !select_all_down)
        {
            if self.ignore_slash_until_release && ch == '/' {
                continue;
            }
            if !ch.is_control() {
                self.delete_input_selection();
                self.input_text.insert(self.cursor_pos, ch);
                self.cursor_pos += ch.len_utf8();
                self.cursor_blink = 0.0;
            }
        }

        None
    }

    pub(super) fn complete_player_name(&mut self, player_names: &[String]) {
        let start = self.input_text[..self.cursor_pos]
            .char_indices()
            .rev()
            .find(|(_, ch)| ch.is_whitespace())
            .map_or(0, |(index, ch)| index + ch.len_utf8());
        let end = self.input_text[self.cursor_pos..]
            .char_indices()
            .find(|(_, ch)| ch.is_whitespace())
            .map_or(self.input_text.len(), |(index, _)| self.cursor_pos + index);
        let token = self.input_text[start..end].to_string();

        let candidate_index = if let Some((last_start, last_name)) = &self.last_tab_completion {
            if *last_start == start && token == *last_name && !self.tab_candidates.is_empty() {
                self.tab_candidate_index % self.tab_candidates.len()
            } else {
                self.tab_candidates = player_names
                    .iter()
                    .filter(|name| name.to_ascii_lowercase().starts_with(&token.to_ascii_lowercase()))
                    .cloned()
                    .collect();
                0
            }
        } else {
            self.tab_candidates = player_names
                .iter()
                .filter(|name| name.to_ascii_lowercase().starts_with(&token.to_ascii_lowercase()))
                .cloned()
                .collect();
            0
        };

        if let Some(name) = self.tab_candidates.get(candidate_index).cloned() {
            self.input_text.replace_range(start..end, &name);
            self.cursor_pos = start + name.len();
            self.last_tab_completion = Some((start, name));
            self.tab_candidate_index = (candidate_index + 1) % self.tab_candidates.len();
        } else {
            self.last_tab_completion = None;
            self.tab_candidates.clear();
            self.tab_candidate_index = 0;
        }
    }
}
