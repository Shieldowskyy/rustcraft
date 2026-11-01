//! Minecraft-style in-game chat and command console.
//!
//! Opened via `T` (chat mode) or `/` (command mode with pre-filled slash).
//! Supports command history, standard commands (`/gamemode`, `/time`, `/give`, `/tp`, etc.),
//! and visual text fade-out.

use crate::core::math::Vec3;
use crate::gameplay::item::ItemEntity;
use crate::gameplay::mob::{MobKind, MobManager};
use crate::gameplay::particles::ParticleSystem;
use crate::gameplay::player::{GameMode, Player};
use crate::graphics::camera::Camera;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};
use crate::world::block::BlockId;
use crate::world::World;

/// Single chat line in history.
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
    input_selection: Option<(usize, usize)>,
    context_menu: Option<ChatContextMenu>,
    selecting_text: bool,
    was_mouse_left: bool,
    was_mouse_right: bool,
    was_copy: bool,
    was_paste: bool,
    was_select_all: bool,

    // Key edge trackers
    was_backspace: bool,
    was_delete: bool,
    was_left: bool,
    was_right: bool,
    was_up: bool,
    was_down: bool,
    was_enter: bool,
    was_escape: bool,
    was_tab: bool,
    tab_candidates: Vec<String>,
    tab_candidate_index: usize,
    last_tab_completion: Option<(usize, String)>,
    pub ignore_slash_until_release: bool,
}

#[derive(Clone, Copy)]
struct ChatContextMenu {
    x: i32,
    y: i32,
    target: ChatContextTarget,
}

#[derive(Clone, Copy)]
enum ChatContextTarget {
    Input,
    Message(usize),
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
        self.add_message(text, pack_color(1.0, 1.0, 0.4)); // Minecraft yellow/gold
    }

    pub fn add_success(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(0.4, 1.0, 0.4)); // Minecraft green
    }

    pub fn add_error(&mut self, text: impl Into<String>) {
        self.add_message(text, pack_color(1.0, 0.35, 0.35)); // Minecraft red
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

    /// Handles keyboard input when chat panel is open.
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

        // 1. Enter key -> submit message / command
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

        // 2. Escape key -> cancel and close
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
            || input.is_key_down(crate::core::controls::KeyCode::Ctrl)
            || input.is_key_down(crate::core::controls::KeyCode::LeftCtrl)
            || input.is_key_down(crate::core::controls::KeyCode::RightCtrl);
        let copy_down = input.key_c || input.is_key_down(crate::core::controls::KeyCode::C);
        let copy_pressed = control_down && copy_down && !self.was_copy;
        self.was_copy = control_down && copy_down;
        if copy_pressed {
            if let Some(selected) = self.selected_text().or_else(|| self.selected_input_text()) {
                crate::ui::clipboard::set_clipboard(&selected);
            }
        }
        let select_all_down =
            control_down && (input.key_a || input.is_key_down(crate::core::controls::KeyCode::A));
        if select_all_down && !self.was_select_all {
            self.input_selection = Some((0, self.input_text.len()));
            self.selection_anchor = None;
            self.selection_focus = None;
        }
        self.was_select_all = select_all_down;

        let paste_down =
            control_down && (input.key_v || input.is_key_down(crate::core::controls::KeyCode::V));
        let paste_pressed = paste_down && !self.was_paste;
        self.was_paste = paste_down;
        if paste_pressed {
            self.paste_into_input(&crate::ui::clipboard::get_clipboard());
        }

        // 3. Arrow keys: Up/Down for command history
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

        // 4. Cursor Navigation (Left / Right)
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

        // 5. Backspace & Delete
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

        // 6. Character typing
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

    fn complete_player_name(&mut self, player_names: &[String]) {
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

    pub fn update_text_selection(
        &mut self,
        input: &crate::app::InputState,
        width: usize,
        height: usize,
    ) {
        let left_pressed = input.mouse_left && !self.was_mouse_left;
        self.was_mouse_left = input.mouse_left;
        let right_pressed = input.mouse_right && !self.was_mouse_right;
        self.was_mouse_right = input.mouse_right;

        if !self.is_open {
            self.selecting_text = false;
            self.context_menu = None;
            return;
        }

        if right_pressed {
            if let Some((x, y)) = input.mouse_pos.map(|(x, y)| (x as i32, y as i32)) {
                if self.input_rect_contains(x, y, width, height) {
                    let clicked_cursor = self.input_cursor_at(x, width);
                    if !self.input_selection.is_some_and(|(start, end)| {
                        clicked_cursor >= start && clicked_cursor <= end
                    }) {
                        self.cursor_pos = clicked_cursor;
                        self.input_selection = None;
                    }
                    self.context_menu = Some(ChatContextMenu {
                        x: x.clamp(0, (width as i32 - 116).max(0)),
                        y: y.clamp(0, (height as i32 - 57).max(0)),
                        target: ChatContextTarget::Input,
                    });
                    return;
                }
                if let Some((message_index, _)) = self.message_position_at(x, y, width, height) {
                    self.context_menu = Some(ChatContextMenu {
                        x: x.clamp(0, (width as i32 - 116).max(0)),
                        y: y.clamp(0, (height as i32 - 57).max(0)),
                        target: ChatContextTarget::Message(message_index),
                    });
                    return;
                }
            }
            self.context_menu = None;
        }

        if let Some(menu) = self.context_menu {
            if left_pressed {
                if let Some((x, y)) = input.mouse_pos.map(|(x, y)| (x as i32, y as i32)) {
                    let item = if x >= menu.x && x < menu.x + 116 && y >= menu.y && y < menu.y + 57
                    {
                        Some(((y - menu.y) / 19) as usize)
                    } else {
                        None
                    };
                    self.context_menu = None;
                    if let Some(item) = item {
                        self.apply_context_action(item, menu.target, height);
                        return;
                    }
                } else {
                    self.context_menu = None;
                }
            }
        }

        let cursor = input
            .mouse_pos
            .and_then(|(x, y)| self.message_position_at(x as i32, y as i32, width, height));
        if left_pressed {
            if input
                .mouse_pos
                .is_some_and(|(x, y)| self.input_rect_contains(x as i32, y as i32, width, height))
            {
                if let Some((x, _)) = input.mouse_pos {
                    self.cursor_pos = self.input_cursor_at(x as i32, width);
                }
                self.input_selection = None;
                self.selection_anchor = None;
                self.selection_focus = None;
                self.selecting_text = false;
                return;
            }
            self.selection_anchor = cursor;
            self.selection_focus = cursor;
            self.input_selection = None;
            self.selecting_text = cursor.is_some();
        } else if self.selecting_text && input.mouse_left {
            if let Some(cursor) = cursor {
                self.selection_focus = Some(cursor);
            }
        } else if !input.mouse_left {
            self.selecting_text = false;
        }
    }

    fn apply_context_action(&mut self, action: usize, target: ChatContextTarget, height: usize) {
        match action {
            0 => {
                let selected = match target {
                    ChatContextTarget::Input => self.selected_input_text(),
                    ChatContextTarget::Message(index) => self
                        .selected_text()
                        .or_else(|| self.messages.get(index).map(|message| message.text.clone())),
                };
                if let Some(text) = selected {
                    crate::ui::clipboard::set_clipboard(&text);
                }
            }
            1 => self.paste_into_input(&crate::ui::clipboard::get_clipboard()),
            2 => match target {
                ChatContextTarget::Input => {
                    self.input_selection = Some((0, self.input_text.len()));
                    self.selection_anchor = None;
                    self.selection_focus = None;
                }
                ChatContextTarget::Message(_) => {
                    let (rows, count) = self.visible_message_rows(height);
                    let oldest = rows.iter().take(count).map(|(index, _)| *index).min();
                    if let Some(oldest) = oldest {
                        let newest = rows
                            .iter()
                            .take(count)
                            .map(|(index, _)| *index)
                            .max()
                            .unwrap_or(oldest);
                        self.selection_anchor = Some((oldest, 0));
                        self.selection_focus =
                            Some((newest, self.messages[newest].text.chars().count()));
                        self.input_selection = None;
                    }
                }
            },
            _ => {}
        }
    }

    fn input_rect_contains(&self, x: i32, y: i32, width: usize, height: usize) -> bool {
        let left = 5;
        let top = height as i32 - 20;
        x >= left && x < width as i32 - left && y >= top && y < top + 14
    }

    fn input_cursor_at(&self, x: i32, width: usize) -> usize {
        let start_x = 8 + crate::graphics::font::measure_text("> ", 1, 1);
        let rel_x = (x - start_x).clamp(0, width as i32);
        let mut prefix = String::new();
        let mut cursor = 0;
        for (byte_index, ch) in self.input_text.char_indices() {
            let char_x = crate::graphics::font::measure_text(&prefix, 1, 1);
            let next_prefix = format!("{prefix}{ch}");
            let next_x = crate::graphics::font::measure_text(&next_prefix, 1, 1);
            cursor = if rel_x < (char_x + next_x) / 2 {
                byte_index
            } else {
                byte_index + ch.len_utf8()
            };
            prefix = next_prefix;
            if rel_x < next_x {
                break;
            }
        }
        cursor.min(self.input_text.len())
    }

    fn selected_input_text(&self) -> Option<String> {
        let (start, end) = self.input_selection?;
        (start <= end)
            .then(|| self.input_text.get(start..end).map(str::to_string))
            .flatten()
    }

    pub fn selected_copy_text(&self) -> Option<String> {
        self.selected_text().or_else(|| self.selected_input_text())
    }

    pub fn paste_clipboard_text(&mut self, text: &str) {
        if self.is_open {
            self.paste_into_input(text);
        }
    }

    fn delete_input_selection(&mut self) {
        if let Some((start, end)) = self.input_selection.take() {
            if start <= end && end <= self.input_text.len() {
                self.input_text.replace_range(start..end, "");
                self.cursor_pos = start;
            }
        }
    }

    fn paste_into_input(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.delete_input_selection();
        let insert_at = self.cursor_pos.min(self.input_text.len());
        if self.input_text.is_char_boundary(insert_at) {
            let sanitized: String = text.chars().filter(|ch| !ch.is_control()).collect();
            self.input_text.insert_str(insert_at, &sanitized);
            self.cursor_pos += sanitized.len();
            self.cursor_blink = 0.0;
        }
    }

    fn visible_message_rows(&self, height: usize) -> ([(usize, i32); 12], usize) {
        let mut rows = [(0, 0); 12];
        let mut row_count = 0;
        let mut y = height as i32 - 18 - 3;
        for index in (0..self.messages.len()).rev() {
            if row_count >= rows.len() {
                break;
            }
            if !self.is_open && self.messages[index].age > 10.0 {
                continue;
            }
            let alpha = if self.is_open {
                0.70f32
            } else {
                (1.0 - (self.messages[index].age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0) * 0.70
            };
            if alpha <= 0.02 {
                continue;
            }
            y -= 11;
            rows[row_count] = (index, y);
            row_count += 1;
        }
        (rows, row_count)
    }

    fn message_position_at(
        &self,
        px: i32,
        py: i32,
        width: usize,
        height: usize,
    ) -> Option<(usize, usize)> {
        let base_x = 8;
        let (rows, row_count) = self.visible_message_rows(height);
        for (index, y) in rows.into_iter().take(row_count) {
            if py < y - 2 || py >= y + 9 {
                continue;
            }
            let text = &self.messages[index].text;
            let text_width = crate::graphics::font::measure_text(text, 1, 1);
            if px < base_x || px > (base_x + text_width).min(width as i32 - base_x) {
                return None;
            }
            let relative_x = px - base_x;
            let mut closest = (0, i32::MAX);
            let mut prefix = String::new();
            for (char_index, ch) in text.chars().enumerate() {
                let char_x = crate::graphics::font::measure_text(&prefix, 1, 1);
                let distance = (relative_x - char_x).abs();
                if distance < closest.1 {
                    closest = (char_index, distance);
                }
                prefix.push(ch);
            }
            let end_index = text.chars().count();
            let end_x = crate::graphics::font::measure_text(text, 1, 1);
            if (relative_x - end_x).abs() < closest.1 {
                closest.0 = end_index;
            }
            return Some((index, closest.0));
        }
        None
    }

    fn selected_text(&self) -> Option<String> {
        let (Some(anchor), Some(focus)) = (self.selection_anchor, self.selection_focus) else {
            return None;
        };
        let (start, end) = if anchor <= focus {
            (anchor, focus)
        } else {
            (focus, anchor)
        };
        if start == end {
            return None;
        }

        let mut selected_lines = Vec::new();
        for index in start.0..=end.0 {
            let chars: Vec<char> = self.messages.get(index)?.text.chars().collect();
            let from = if index == start.0 { start.1 } else { 0 };
            let to = if index == end.0 { end.1 } else { chars.len() };
            selected_lines.push(chars.get(from..to)?.iter().collect::<String>());
        }
        Some(selected_lines.join("\n"))
    }

    /// Renders chat messages and active command prompt overlay.
    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize) {
        let base_x = 8i32;
        let line_height = 11i32;
        let max_visible_lines = 12;
        let input_y = height as i32 - 18;

        // 1. Draw message history (from bottom to top)
        let (rows, row_count) = self.visible_message_rows(height);
        for (index, y) in rows.into_iter().take(row_count.min(max_visible_lines)) {
            let msg = &self.messages[index];
            let alpha = if self.is_open {
                0.70f32
            } else {
                (1.0 - (msg.age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0) * 0.70
            };
            let text_w = crate::graphics::font::measure_text(&msg.text, 1, 1);
            let bar_w = (text_w + 6).min(width as i32 - base_x * 2);

            fill_rect_alpha(
                buffer,
                width,
                height,
                base_x - 3,
                y - 2,
                bar_w,
                line_height,
                pack_color(0.0, 0.0, 0.0),
                alpha,
            );

            if let (Some(anchor), Some(focus)) = (self.selection_anchor, self.selection_focus) {
                let (start, end) = if anchor <= focus {
                    (anchor, focus)
                } else {
                    (focus, anchor)
                };
                if index >= start.0 && index <= end.0 {
                    let chars: Vec<char> = msg.text.chars().collect();
                    let from = if index == start.0 { start.1 } else { 0 };
                    let to = if index == end.0 { end.1 } else { chars.len() };
                    if from < to && to <= chars.len() {
                        let prefix: String = chars[..from].iter().collect();
                        let selected: String = chars[from..to].iter().collect();
                        let selection_x =
                            base_x + crate::graphics::font::measure_text(&prefix, 1, 1);
                        let selection_w = crate::graphics::font::measure_text(&selected, 1, 1);
                        fill_rect_alpha(
                            buffer,
                            width,
                            height,
                            selection_x,
                            y - 1,
                            selection_w.max(1),
                            9,
                            pack_color(0.18, 0.40, 0.78),
                            0.90,
                        );
                    }
                }
            }

            let text_alpha = if self.is_open {
                1.0
            } else {
                (1.0 - (msg.age - 8.0).max(0.0) / 2.0).clamp(0.0, 1.0)
            };

            if text_alpha > 0.05 {
                draw_text_colored_shadowed(
                    buffer, width, height, &msg.text, base_x, y, 1, 1, msg.color,
                );
            }
        }

        // 2. Draw chat input bar when open
        if self.is_open {
            let bar_w = (width as i32 - base_x * 2).max(100);
            let bar_h = 14i32;

            fill_rect_alpha(
                buffer,
                width,
                height,
                base_x - 3,
                input_y - 2,
                bar_w + 6,
                bar_h,
                pack_color(0.0, 0.0, 0.0),
                0.80,
            );
            fill_rect_alpha(
                buffer,
                width,
                height,
                base_x - 3,
                input_y + bar_h - 2,
                bar_w + 6,
                1,
                pack_color(0.45, 0.45, 0.45),
                0.90,
            );

            let prefix = "> ";
            let display_str = format!("{}{}", prefix, self.input_text);
            let white = pack_color(1.0, 1.0, 1.0);
            if let Some((start, end)) = self.input_selection {
                if start < end && end <= self.input_text.len() {
                    if let (Some(selected), Some(before)) = (
                        self.input_text.get(start..end),
                        self.input_text.get(..start),
                    ) {
                        let sx = base_x
                            + crate::graphics::font::measure_text(
                                &format!("{prefix}{before}"),
                                1,
                                1,
                            );
                        let sw = crate::graphics::font::measure_text(selected, 1, 1);
                        fill_rect_alpha(
                            buffer,
                            width,
                            height,
                            sx,
                            input_y,
                            sw,
                            9,
                            pack_color(0.18, 0.40, 0.78),
                            0.90,
                        );
                    }
                }
            }
            draw_text_colored_shadowed(
                buffer,
                width,
                height,
                &display_str,
                base_x,
                input_y + 1,
                1,
                1,
                white,
            );

            // Blinking cursor
            let show_cursor = (self.cursor_blink % 0.8) < 0.4;
            if show_cursor {
                let cursor_sub = format!("{}{}", prefix, &self.input_text[..self.cursor_pos]);
                let cursor_x = base_x + crate::graphics::font::measure_text(&cursor_sub, 1, 1);
                fill_rect(buffer, width, height, cursor_x, input_y + 1, 2, 7, white);
            }
        }

        if let Some(menu) = self.context_menu {
            let menu_width = 116;
            let menu_height = 57;
            fill_rect_alpha(
                buffer,
                width,
                height,
                menu.x,
                menu.y,
                menu_width,
                menu_height,
                pack_color(0.08, 0.09, 0.12),
                0.98,
            );
            fill_rect(
                buffer,
                width,
                height,
                menu.x,
                menu.y,
                menu_width,
                1,
                pack_color(0.55, 0.58, 0.64),
            );
            fill_rect(
                buffer,
                width,
                height,
                menu.x,
                menu.y,
                1,
                menu_height,
                pack_color(0.55, 0.58, 0.64),
            );
            fill_rect(
                buffer,
                width,
                height,
                menu.x + menu_width - 1,
                menu.y,
                1,
                menu_height,
                pack_color(0.22, 0.24, 0.28),
            );
            fill_rect(
                buffer,
                width,
                height,
                menu.x,
                menu.y + menu_height - 1,
                menu_width,
                1,
                pack_color(0.22, 0.24, 0.28),
            );
            for (index, label) in ["Copy", "Paste", "Select all"].iter().enumerate() {
                draw_text_colored_shadowed(
                    buffer,
                    width,
                    height,
                    label,
                    menu.x + 8,
                    menu.y + 5 + index as i32 * 19,
                    1,
                    1,
                    pack_color(0.95, 0.95, 0.97),
                );
            }
        }
    }

    /// Dispatches and executes chat command or regular text message.
    pub fn dispatch(&mut self, text: &str, ctx: &mut CommandContext) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }

        if text.starts_with('/') {
            self.execute_command(text, ctx);
        } else {
            self.add_player_chat("Player", text);
        }
    }

    /// Parses and executes slash commands like `/gamemode`, `/time`, `/give`, `/tp`, etc.
    pub fn execute_command(&mut self, full_cmd: &str, ctx: &mut CommandContext) {
        crate::log_info!("Command", "Executing: {}", full_cmd);
        let trimmed = full_cmd.trim_start_matches('/');
        let mut parts = trimmed.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c.to_ascii_lowercase(),
            None => return,
        };

        match cmd.as_str() {
            "help" | "?" => {
                self.add_info("--- Showing help page (Minecraft Commands) ---");
                self.add_info("/gamemode <survival|creative> - Change game mode");
                self.add_info("/time set <day|noon|night|midnight|number> - Set time");
                self.add_info("/tps - Show multiplayer server tick rate and tick time");
                self.add_info("/give <item> [count] - Add items to inventory");
                self.add_info("/tp <x> <y> <z> or /tp <player> - Teleport");
                self.add_info(
                    "/spawn <pig|cow|chicken|zombie|skeleton|reaper> [count] - Spawn mob",
                );
                self.add_info("/mobs - List all currently spawned mobs in the world");
                self.add_info("/heal - Restore health and hunger");
                self.add_info("/kill - Kill the player");
                self.add_info("/clear [chat] - Clear inventory or chat");
                self.add_info("/seed - Display world generation seed");
                self.add_info("/reloadchunks - Reload chunks and rebuild mesh cache");
                self.add_info("/fly - Toggle flight mode");
                self.add_info("/weather <clear|rain|thunder|query> [duration] - Control weather");
                self.add_info("/difficulty <peaceful|easy|normal|hard>");

                let mod_cmds = crate::engine::modding::get_all_mod_commands();
                if !mod_cmds.is_empty() {
                    self.add_info("--- Mod Commands ---");
                    for (c_name, c_desc) in mod_cmds {
                        if c_desc.is_empty() {
                            self.add_info(format!("/{}", c_name));
                        } else {
                            self.add_info(format!("/{} - {}", c_name, c_desc));
                        }
                    }
                }
            }

            "gamemode" | "gm" => {
                let mode_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                match mode_arg.as_str() {
                    "creative" | "c" | "1" => {
                        ctx.player.gamemode = GameMode::Creative;
                        self.add_success("Set own game mode to Creative Mode");
                        crate::log_debug!("Command", "Gamemode changed to Creative");
                    }
                    "survival" | "s" | "0" => {
                        ctx.player.gamemode = GameMode::Survival;
                        *ctx.is_flying = false;
                        self.add_success("Set own game mode to Survival Mode");
                        crate::log_debug!("Command", "Gamemode changed to Survival");
                    }
                    _ => {
                        self.add_error("Usage: /gamemode <survival|creative>");
                    }
                }
            }

            "time" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "set" => {
                        let val = parts.next().unwrap_or("").to_ascii_lowercase();
                        match val.as_str() {
                            "day" | "1000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.25;
                                self.add_success("Set the time to 1000 (Day)");
                            }
                            "noon" | "6000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.50;
                                self.add_success("Set the time to 6000 (Noon)");
                            }
                            "night" | "13000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.75;
                                self.add_success("Set the time to 13000 (Night)");
                            }
                            "midnight" | "18000" => {
                                ctx.world.time_of_day = 0.0;
                                self.add_success("Set the time to 18000 (Midnight)");
                            }
                            other => {
                                if let Ok(ticks) = other.parse::<f32>() {
                                    let frac = (ticks % 24000.0) / 24000.0;
                                    ctx.world.time_of_day = World::DAY_DURATION * frac;
                                    self.add_success(format!("Set the time to {:.0}", ticks));
                                } else {
                                    self.add_error(
                                        "Usage: /time set <day|noon|night|midnight|number>",
                                    );
                                }
                            }
                        }
                    }
                    "query" => {
                        let ticks = (ctx.world.day_fraction() * 24000.0) as u32;
                        self.add_info(format!("The time is {} ticks", ticks));
                    }
                    _ => {
                        self.add_error("Usage: /time <set|query> <value>");
                    }
                }
            }

            "tps" => {
                self.add_info("TPS is available when connected to a multiplayer server or host.");
            }

            "weather" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                let duration_arg = parts.next().and_then(|s| s.parse::<f32>().ok());
                match sub.as_str() {
                    "clear" => {
                        let duration = duration_arg.unwrap_or(600.0);
                        ctx.world
                            .weather
                            .set_weather(crate::world::weather::WeatherType::Clear, Some(duration));
                        self.add_success(format!(
                            "Set the weather to clear (duration: {:.0}s)",
                            duration
                        ));
                    }
                    "rain" => {
                        let duration = duration_arg.unwrap_or(360.0);
                        ctx.world
                            .weather
                            .set_weather(crate::world::weather::WeatherType::Rain, Some(duration));
                        self.add_success(format!(
                            "Set the weather to rain (duration: {:.0}s)",
                            duration
                        ));
                    }
                    "thunder" | "thunderstorm" => {
                        let duration = duration_arg.unwrap_or(240.0);
                        ctx.world.weather.set_weather(
                            crate::world::weather::WeatherType::Thunder,
                            Some(duration),
                        );
                        self.add_success(format!(
                            "Set the weather to thunder and rain (duration: {:.0}s)",
                            duration
                        ));
                    }
                    "query" => {
                        let state_str = match ctx.world.weather.current_type {
                            crate::world::weather::WeatherType::Clear => "clear",
                            crate::world::weather::WeatherType::Rain => "rain",
                            crate::world::weather::WeatherType::Thunder => "thunder",
                        };
                        self.add_info(format!(
                            "The weather is currently {} (remaining: {:.0}s, rain intensity: {:.0}%)",
                            state_str,
                            ctx.world.weather.timer,
                            ctx.world.weather.rain_strength * 100.0
                        ));
                    }
                    _ => {
                        self.add_error("Usage: /weather <clear|rain|thunder|query> [duration]");
                    }
                }
            }

            "give" => {
                let item_arg = match parts.next() {
                    Some(a) => a,
                    None => {
                        self.add_error("Usage: /give <item> [count]");
                        return;
                    }
                };
                let count: u8 = parts
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(1)
                    .clamp(1, 64);

                if let Some(block) = parse_block_id(item_arg) {
                    let leftover = ctx.player.inventory.add_item(block, count);
                    let given = count - leftover;
                    if given > 0 {
                        self.add_success(format!("Gave {} [{}] to Player", given, block.name()));
                        crate::log_debug!("Command", "Gave {} {:?} to Player", given, block);
                    }
                    if leftover > 0 {
                        self.add_error(format!(
                            "Inventory full! Could not give {} items.",
                            leftover
                        ));
                    }
                } else {
                    self.add_error(format!("Unknown item: '{}'", item_arg));
                }
            }

            "tp" | "teleport" => {
                let x_str = parts.next();
                let y_str = parts.next();
                let z_str = parts.next();

                if let (Some(xs), Some(ys), Some(zs)) = (x_str, y_str, z_str) {
                    let parse_coord = |s: &str, current: f32| -> Option<f32> {
                        if let Some(rel) = s.strip_prefix('~') {
                            if rel.is_empty() {
                                Some(current)
                            } else {
                                rel.parse::<f32>().ok().map(|d| current + d)
                            }
                        } else {
                            s.parse::<f32>().ok()
                        }
                    };

                    let px = parse_coord(xs, ctx.camera.position.x);
                    let py = parse_coord(ys, ctx.camera.position.y);
                    let pz = parse_coord(zs, ctx.camera.position.z);

                    if let (Some(x), Some(y), Some(z)) = (px, py, pz) {
                        ctx.camera.position = Vec3::new(x, y, z);
                        *ctx.horiz_velocity = Vec3::zero();
                        *ctx.fly_velocity = Vec3::zero();
                        self.add_success(format!(
                            "Teleported Player to {:.1}, {:.1}, {:.1}",
                            x, y, z
                        ));
                        crate::log_debug!(
                            "Command",
                            "Teleported Player to ({:.1}, {:.1}, {:.1})",
                            x,
                            y,
                            z
                        );
                    } else {
                        self.add_error("Invalid coordinates. Example: /tp 0 70 0 or /tp ~ ~10 ~");
                    }
                } else {
                    self.add_error("Usage: /tp <x> <y> <z> or /tp <player>");
                }
            }

            "spawn" => {
                let mut kind_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                if kind_arg == "mob" {
                    kind_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                }
                let count: usize = parts
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(1)
                    .clamp(1, 20);

                let mob_kind = match kind_arg.as_str() {
                    "pig" | "swinia" => Some(MobKind::Pig),
                    "cow" | "krowa" => Some(MobKind::Cow),
                    "chicken" | "kurczak" => Some(MobKind::Chicken),
                    "sheep" | "owca" => Some(MobKind::Sheep),
                    "zombie" | "zombi" => Some(MobKind::Zombie),
                    "skeleton" | "szkielet" => Some(MobKind::Skeleton),
                    "reaper" => Some(MobKind::Reaper),
                    "dolphin" | "delfin" => Some(MobKind::Dolphin),
                    "cod" | "dorsz" => Some(MobKind::Cod),
                    "salmon" | "losos" => Some(MobKind::Salmon),
                    "pufferfish" | "rozdymka" => Some(MobKind::Pufferfish),
                    "tropical_fish" | "tropikalna_ryba" => Some(MobKind::TropicalFish),
                    "villager" | "osadnik" | "wiesniak" => Some(MobKind::Villager),
                    "spider" | "pajak" => Some(MobKind::Spider),
                    _ => None,
                };

                if let Some(kind) = mob_kind {
                    let forward = ctx.camera.forward_flat();
                    for i in 0..count {
                        let offset = forward * (2.0 + (i as f32 * 0.8));
                        let spawn_pos = ctx.camera.position + offset;
                        ctx.mob_manager.spawn_mob(kind, spawn_pos);
                        ctx.particles.spawn_mob_hit(spawn_pos);
                    }
                    self.add_success(format!("Spawned {} {:?}(s)", count, kind));
                    crate::log_debug!("Command", "Spawned {} {:?} mobs", count, kind);
                } else {
                    self.add_error("Usage: /spawn <pig|cow|chicken|sheep|zombie|skeleton|reaper|dolphin|cod|salmon|pufferfish|tropical_fish|villager> [count]");
                }
            }

            "mobs" | "moblist" | "listmobs" => {
                self.list_mobs_debug(ctx);
            }

            "debug" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "mobs" | "moblist" => {
                        self.list_mobs_debug(ctx);
                    }
                    _ => {
                        self.add_info("Debug commands: /mobs, /debug mobs");
                    }
                }
            }

            "heal" => {
                ctx.player.health = ctx.player.max_health;
                ctx.player.hunger = ctx.player.max_hunger;
                ctx.player.saturation = 20.0;
                ctx.player.air = ctx.player.max_air;
                self.add_success("Healed Player to full health and hunger");
                crate::log_debug!("Command", "Healed Player to full health and hunger");
            }

            "kill" => {
                ctx.player.take_damage(999.0);
                self.add_error("Ouch! That looks like it hurt.");
                crate::log_debug!("Command", "Killed Player via command");
            }

            "clear" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                if sub == "chat" {
                    self.messages.clear();
                } else {
                    ctx.player.inventory.hotbar = [None; 10];
                    ctx.player.inventory.main = [None; 27];
                    ctx.player.inventory.held_item = None;
                    self.add_success("Cleared the inventory of Player");
                    crate::log_debug!("Command", "Cleared player inventory");
                }
            }

            "seed" => {
                let seed = ctx.world.seed;
                self.add_info(format!("Seed: [{}]", seed));
            }

            "fly" => {
                *ctx.is_flying = !*ctx.is_flying;
                if *ctx.is_flying {
                    self.add_success("Flight mode: ENABLED");
                } else {
                    self.add_info("Flight mode: DISABLED");
                }
                crate::log_debug!("Command", "Flight mode set to {}", *ctx.is_flying);
            }

            "reload" | "reloadchunks" => {
                ctx.world.mark_all_chunks_dirty();
                *ctx.reload_chunks_requested = true;
                self.add_success("Reloading all chunks and rebuilding mesh cache...");
                crate::log_info!("WORLD", "Chunk reload triggered via chat command");
            }

            "difficulty" => {
                let d = parts.next().unwrap_or("normal");
                self.add_info(format!("Set difficulty to {}", d));
            }

            unknown => {
                if let Some(cmd_def) = crate::engine::modding::find_mod_command(&cmd) {
                    crate::engine::modding::execute_mod_command(&cmd_def, self, ctx);
                } else {
                    self.add_error(format!(
                        "Unknown command: '/{}'. Type /help for help.",
                        unknown
                    ));
                }
            }
        }
    }

    /// Debug helper to display all currently spawned mobs in the world.
    fn list_mobs_debug(&mut self, ctx: &CommandContext) {
        let total = ctx.mob_manager.mobs.len();
        if total == 0 {
            self.add_info("No mobs currently spawned in the world.");
            return;
        }

        let player_pos = ctx.camera.position;
        self.add_info(format!("=== Spawned Mobs (Total: {}) ===", total));

        let mut pigs = 0;
        let mut cows = 0;
        let mut chickens = 0;
        let mut sheep = 0;
        let mut zombies = 0;
        let mut skeletons = 0;
        let mut reapers = 0;
        let mut dolphins = 0;
        let mut cod = 0;
        let mut salmon = 0;
        let mut pufferfish = 0;
        let mut tropical_fish = 0;
        let mut villagers = 0;
        let mut iron_golems = 0;
        let mut spiders = 0;

        for mob in &ctx.mob_manager.mobs {
            match mob.kind {
                MobKind::Pig => pigs += 1,
                MobKind::Cow => cows += 1,
                MobKind::Chicken => chickens += 1,
                MobKind::Sheep => sheep += 1,
                MobKind::Zombie => zombies += 1,
                MobKind::Skeleton => skeletons += 1,
                MobKind::Reaper => reapers += 1,
                MobKind::Dolphin => dolphins += 1,
                MobKind::Cod => cod += 1,
                MobKind::Salmon => salmon += 1,
                MobKind::Pufferfish => pufferfish += 1,
                MobKind::TropicalFish => tropical_fish += 1,
                MobKind::Villager => villagers += 1,
                MobKind::IronGolem => iron_golems += 1,
                MobKind::Spider => spiders += 1,
            }
            let dist = (mob.pos - player_pos).length();
            let hostile_tag = if mob.kind.is_hostile() {
                " [HOSTILE]"
            } else {
                ""
            };
            self.add_message(
                format!(
                    "#{:02} {} at ({:.1}, {:.1}, {:.1}) | Dist: {:.1}m | HP: {:.0}/{:.0}{}",
                    mob.id,
                    mob.kind.name(),
                    mob.pos.x,
                    mob.pos.y,
                    mob.pos.z,
                    dist,
                    mob.health.max(0.0),
                    mob.kind.max_health(),
                    hostile_tag,
                ),
                if mob.kind.is_hostile() {
                    pack_color(1.0, 0.45, 0.45)
                } else {
                    pack_color(0.55, 1.0, 0.55)
                },
            );
        }

        self.add_info(format!(
            "Summary: Pigs: {}, Cows: {}, Chickens: {}, Sheep: {}, Zombies: {}, Skeletons: {}, Reapers: {}, Spiders: {}, Dolphins: {}, Fish: {}, Villagers: {}, Iron Golems: {}",
            pigs, cows, chickens, sheep, zombies, skeletons, reapers, spiders, dolphins, cod + salmon + pufferfish + tropical_fish, villagers, iron_golems
        ));
    }
}

/// Parses a vanilla item/block string or numeric ID into a `BlockId`.
pub fn parse_vanilla_block_id(s: &str) -> Option<BlockId> {
    let s = s.trim().to_ascii_lowercase().replace('-', "_");
    match s.as_str() {
        "grass" | "grass_block" | "trawa" => Some(BlockId::Grass),
        "dirt" | "ziemia" => Some(BlockId::Dirt),
        "stone" | "kamien" | "rock" | "cobblestone" => Some(BlockId::Stone),
        "sand" | "piasek" => Some(BlockId::Sand),
        "wood" | "log" | "drewno" | "oak_log" => Some(BlockId::Wood),
        "planks" | "deski" | "wood_planks" => Some(BlockId::Planks),
        "leaves" | "liscie" => Some(BlockId::Leaves),
        "glass" | "szyba" => Some(BlockId::Glass),
        "ice" | "lod" => Some(BlockId::Ice),
        "tnt" => Some(BlockId::Tnt),
        "crafting_table" | "workbench" => Some(BlockId::CraftingTable),
        "chest" | "skrzynia" => Some(BlockId::Chest),
        "furnace" | "piec" => Some(BlockId::Furnace),
        "torch" | "pochodnia" => Some(BlockId::Torch),
        "bedrock" => Some(BlockId::Bedrock),
        "water" | "woda" => Some(BlockId::Water),
        "lava" | "lawa" => Some(BlockId::Lava),
        "mud" | "bloto" => Some(BlockId::Mud),
        "coal_ore" | "coal" | "wegiel" => Some(BlockId::CoalOre),
        "iron_ore" | "zelazo" => Some(BlockId::IronOre),
        "gold_ore" | "zloto" => Some(BlockId::GoldOre),
        "iron_ingot" | "ingot" => Some(BlockId::IronIngot),
        "gold_ingot" => Some(BlockId::GoldIngot),
        "iron_block" => Some(BlockId::IronBlock),
        "gold_block" => Some(BlockId::GoldBlock),
        "coal_block" => Some(BlockId::CoalBlock),
        "stick" | "patyk" => Some(BlockId::Stick),
        "apple" | "jablko" => Some(BlockId::Apple),
        "raw_porkchop" | "porkchop" => Some(BlockId::RawPorkchop),
        "cooked_porkchop" => Some(BlockId::CookedPorkchop),
        "raw_beef" | "beef" => Some(BlockId::RawBeef),
        "cooked_beef" | "steak" => Some(BlockId::CookedBeef),
        "raw_chicken" | "chicken" => Some(BlockId::RawChicken),
        "cooked_chicken" => Some(BlockId::CookedChicken),
        "feather" | "pioro" => Some(BlockId::Feather),
        "wooden_sword" => Some(BlockId::WoodenSword),
        "stone_sword" => Some(BlockId::StoneSword),
        "iron_sword" | "sword" | "diamond_sword" => Some(BlockId::IronSword),
        "wooden_pickaxe" => Some(BlockId::WoodenPickaxe),
        "stone_pickaxe" => Some(BlockId::StonePickaxe),
        "iron_pickaxe" | "pickaxe" | "diamond_pickaxe" => Some(BlockId::IronPickaxe),
        "wooden_axe" => Some(BlockId::WoodenAxe),
        "stone_axe" => Some(BlockId::StoneAxe),
        "iron_axe" | "axe" => Some(BlockId::IronAxe),
        "wooden_shovel" => Some(BlockId::WoodenShovel),
        "stone_shovel" => Some(BlockId::StoneShovel),
        "iron_shovel" | "shovel" => Some(BlockId::IronShovel),
        "pig_spawn_egg" | "pig_egg" => Some(BlockId::PigSpawnEgg),
        "cow_spawn_egg" | "cow_egg" => Some(BlockId::CowSpawnEgg),
        "chicken_spawn_egg" | "chicken_egg" => Some(BlockId::ChickenSpawnEgg),
        "bone" | "kosc" => Some(BlockId::Bone),
        "gunpowder" | "proch" => Some(BlockId::Gunpowder),
        "ruststone" | "rdzawnik" | "redstone" => Some(BlockId::RustStone),
        "ruststone_ore" | "ruda_rdzawnika" | "redstone_ore" => Some(BlockId::RustStoneOre),
        "ruststone_block" | "blok_rdzawnika" | "redstone_block" => Some(BlockId::RustStoneBlock),
        "ruststone_lamp" | "lampa_rdzawnikowa" | "lampa" | "redstone_lamp" => {
            Some(BlockId::RustStoneLamp)
        }
        "lever" | "dzwignia" => Some(BlockId::Lever),
        "button" | "przycisk" => Some(BlockId::Button),
        "pressure_plate" | "plytka" | "plytka_naciskowa" => Some(BlockId::PressurePlate),
        "shears" | "nozyce" => Some(BlockId::Shears),
        "sheep_spawn_egg" | "sheep_egg" | "owca_jajko" => Some(BlockId::SheepSpawnEgg),
        "wool" | "welna" | "white_wool" | "biala_welna" => Some(BlockId::WhiteWool),
        "orange_wool" | "pomaranczowa_welna" => Some(BlockId::OrangeWool),
        "magenta_wool" | "jasnofioletowa_welna" => Some(BlockId::MagentaWool),
        "light_blue_wool" | "jasnoniebieska_welna" => Some(BlockId::LightBlueWool),
        "yellow_wool" | "zolta_welna" => Some(BlockId::YellowWool),
        "lime_wool" | "jasnozielona_welna" => Some(BlockId::LimeWool),
        "pink_wool" | "rozowa_welna" => Some(BlockId::PinkWool),
        "gray_wool" | "szara_welna" => Some(BlockId::GrayWool),
        "light_gray_wool" | "jasnoszara_welna" => Some(BlockId::LightGrayWool),
        "cyan_wool" | "turkusowa_welna" => Some(BlockId::CyanWool),
        "purple_wool" | "fioletowa_welna" => Some(BlockId::PurpleWool),
        "blue_wool" | "niebieska_welna" => Some(BlockId::BlueWool),
        "brown_wool" | "brazowa_welna" => Some(BlockId::BrownWool),
        "green_wool" | "zielona_welna" => Some(BlockId::GreenWool),
        "red_wool" | "czerwona_welna" => Some(BlockId::RedWool),
        "black_wool" | "czarna_welna" => Some(BlockId::BlackWool),
        "kelp" | "wodorosty" => Some(BlockId::Kelp),
        "seagrass" | "trawa_morska" => Some(BlockId::Seagrass),
        "raw_cod" | "cod" | "dorsz" => Some(BlockId::RawCod),
        "cooked_cod" | "pieczony_dorsz" => Some(BlockId::CookedCod),
        "raw_salmon" | "salmon" | "losos" => Some(BlockId::RawSalmon),
        "cooked_salmon" | "pieczony_losos" => Some(BlockId::CookedSalmon),
        "pufferfish" | "rozdymka" => Some(BlockId::Pufferfish),
        "tropical_fish" | "tropikalna_ryba" => Some(BlockId::TropicalFish),
        "dried_kelp" | "suszone_wodorosty" => Some(BlockId::DriedKelp),
        "dolphin_spawn_egg" | "dolphin_egg" | "jajko_delfina" => Some(BlockId::DolphinSpawnEgg),
        "fish_spawn_egg" | "fish_egg" | "jajko_ryby" => Some(BlockId::FishSpawnEgg),
        "villager_spawn_egg" | "villager_egg" | "jajko_osadnika" | "jajko_wiesniaka" => {
            Some(BlockId::VillagerSpawnEgg)
        }
        "emerald" | "szmaragd" => Some(BlockId::Emerald),
        "emerald_ore" | "ruda_szmaragdu" => Some(BlockId::EmeraldOre),
        "emerald_block" | "blok_szmaragdu" => Some(BlockId::EmeraldBlock),
        "fishing_rod" | "wedka" => Some(BlockId::FishingRod),
        "string" | "nici" | "nic" => Some(BlockId::StringItem),
        "spider_eye" | "oko_pajaka" => Some(BlockId::SpiderEye),
        "spider_spawn_egg" | "spider_egg" | "jajko_pajaka" => Some(BlockId::SpiderSpawnEgg),
        _ => {
            if let Ok(id) = s.parse::<u8>() {
                if id > 0 {
                    return Some(BlockId::from_u8(id));
                }
            }
            None
        }
    }
}

/// Parses an item/block string into a `BlockId` (supports both vanilla and registered mod items).
pub fn parse_block_id(s: &str) -> Option<BlockId> {
    if let Some(b) = parse_vanilla_block_id(s) {
        return Some(b);
    }
    let s = s.trim().to_ascii_lowercase().replace('-', "_");
    crate::engine::modding::find_custom_block_by_name(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_open_close_and_history() {
        let mut chat = ChatState::new();
        assert!(!chat.is_open);

        chat.open("hello");
        assert!(chat.is_open);
        assert_eq!(chat.input_text, "hello");
        assert_eq!(chat.cursor_pos, 5);

        chat.close();
        assert!(!chat.is_open);
        assert_eq!(chat.input_text, "");
    }

    #[test]
    fn tab_completes_and_cycles_matching_player_names() {
        let mut chat = ChatState::new();
        chat.open("/tp Al");
        let names = vec!["Alex".to_string(), "Alice".to_string(), "Bob".to_string()];

        chat.complete_player_name(&names);
        assert_eq!(chat.input_text, "/tp Alex");

        chat.complete_player_name(&names);
        assert_eq!(chat.input_text, "/tp Alice");
    }

    #[test]
    fn mouse_selection_copies_text_across_chat_lines() {
        let mut chat = ChatState::new();
        chat.add_message("first", pack_color(1.0, 1.0, 1.0));
        chat.add_message("second", pack_color(1.0, 1.0, 1.0));
        chat.open("");

        let first_line_y = 200 - 18 - 3 - 22;
        let latest_line_y = first_line_y + 11;
        let first_char_width = crate::graphics::font::measure_text("f", 1, 1);
        let third_char_width = crate::graphics::font::measure_text("sec", 1, 1);
        let mut input = crate::app::InputState {
            mouse_pos: Some((8.0 + first_char_width as f32, first_line_y as f32)),
            mouse_left: true,
            ..Default::default()
        };
        chat.update_text_selection(&input, 320, 200);

        input.mouse_pos = Some((8.0 + third_char_width as f32, latest_line_y as f32));
        chat.update_text_selection(&input, 320, 200);
        assert_eq!(chat.selected_text().as_deref(), Some("irst\nsec"));
    }

    #[test]
    fn ctrl_a_selects_input_and_selection_can_be_replaced() {
        let mut chat = ChatState::new();
        chat.open("draft");
        let input = crate::app::InputState {
            key_ctrl: true,
            key_a: true,
            ..Default::default()
        };

        chat.handle_input(&input, &[]);
        assert_eq!(chat.selected_copy_text().as_deref(), Some("draft"));

        chat.paste_clipboard_text("replacement");
        assert_eq!(chat.input_text, "replacement");
    }

    #[test]
    fn input_context_menu_select_all_then_paste_replaces_text() {
        let mut chat = ChatState::new();
        chat.open("draft");
        let mut input = crate::app::InputState {
            mouse_pos: Some((30.0, 182.0)),
            mouse_right: true,
            ..Default::default()
        };
        chat.update_text_selection(&input, 320, 200);
        input.mouse_right = false;
        chat.update_text_selection(&input, 320, 200);
        let menu = chat
            .context_menu
            .expect("right-clicking the input should open its context menu");

        input.mouse_left = true;
        input.mouse_pos = Some(((menu.x + 8) as f32, (menu.y + 43) as f32));
        chat.update_text_selection(&input, 320, 200);
        assert_eq!(chat.selected_copy_text().as_deref(), Some("draft"));

        chat.paste_clipboard_text("replacement");
        assert_eq!(chat.input_text, "replacement");
    }

    #[test]
    fn ctrl_c_copies_selected_chat_text() {
        let _clipboard_lock = crate::ui::clipboard::CLIPBOARD_TEST_LOCK
            .lock()
            .expect("clipboard test lock poisoned");
        let mut chat = ChatState::new();
        chat.add_message("selected", pack_color(1.0, 1.0, 1.0));
        chat.open("");
        chat.selection_anchor = Some((0, 0));
        chat.selection_focus = Some((0, 8));

        chat.handle_input(&crate::app::InputState {
            key_ctrl: true,
            key_c: true,
            ..Default::default()
        }, &[]);

        assert_eq!(crate::ui::clipboard::internal_clipboard_text(), "selected");
    }

    #[test]
    fn right_click_chat_context_menu_can_copy_and_select_all() {
        let _clipboard_lock = crate::ui::clipboard::CLIPBOARD_TEST_LOCK
            .lock()
            .expect("clipboard test lock poisoned");
        let mut chat = ChatState::new();
        chat.add_message("first", pack_color(1.0, 1.0, 1.0));
        chat.add_message("second", pack_color(1.0, 1.0, 1.0));
        chat.open("");

        let mut input = crate::app::InputState {
            mouse_pos: Some((12.0, 168.0)),
            mouse_right: true,
            ..Default::default()
        };
        chat.update_text_selection(&input, 320, 200);
        assert!(chat.context_menu.is_some());

        input.mouse_right = false;
        chat.update_text_selection(&input, 320, 200);
        let menu = chat
            .context_menu
            .expect("right-click menu should remain open");
        input.mouse_pos = Some(((menu.x + 8) as f32, (menu.y + 5) as f32));
        input.mouse_left = true;
        chat.update_text_selection(&input, 320, 200);
        assert_eq!(crate::ui::clipboard::internal_clipboard_text(), "second");
        assert!(chat.context_menu.is_none());

        input.mouse_left = false;
        input.mouse_right = true;
        input.mouse_pos = Some((12.0, 168.0));
        chat.update_text_selection(&input, 320, 200);
        input.mouse_right = false;
        chat.update_text_selection(&input, 320, 200);
        let menu = chat
            .context_menu
            .expect("right-click menu should remain open");
        input.mouse_left = true;
        input.mouse_pos = Some(((menu.x + 8) as f32, (menu.y + 43) as f32));
        chat.update_text_selection(&input, 320, 200);
        assert_eq!(chat.selected_text().as_deref(), Some("first\nsecond"));
    }

    #[test]
    fn test_parse_block_id_items() {
        assert_eq!(parse_block_id("diamond_sword"), Some(BlockId::IronSword));
        assert_eq!(parse_block_id("wood"), Some(BlockId::Wood));
        assert_eq!(parse_block_id("raw_porkchop"), Some(BlockId::RawPorkchop));
        assert_eq!(parse_block_id("cooked_beef"), Some(BlockId::CookedBeef));
        assert_eq!(parse_block_id("apple"), Some(BlockId::Apple));
        assert_eq!(parse_block_id("torch"), Some(BlockId::Torch));
        assert_eq!(parse_block_id("bone"), Some(BlockId::Bone));
        assert_eq!(parse_block_id("kosc"), Some(BlockId::Bone));
        assert_eq!(parse_block_id("gunpowder"), Some(BlockId::Gunpowder));
        assert_eq!(parse_block_id("proch"), Some(BlockId::Gunpowder));
    }

    #[test]
    fn test_chat_mobs_debug_command() {
        let mut chat = ChatState::new();
        let mut mob_manager = MobManager::new();
        mob_manager.spawn_mob(MobKind::Zombie, Vec3::new(10.0, 64.0, 10.0));
        mob_manager.spawn_mob(MobKind::Skeleton, Vec3::new(15.0, 64.0, 15.0));
        mob_manager.spawn_mob(MobKind::Reaper, Vec3::new(8.0, 64.0, 8.0));

        let mut player = Player::new();
        let mut camera = Camera::new(Vec3::new(0.0, 64.0, 0.0));
        let mut world = World::new();
        let mut items = Vec::new();
        let mut particles = ParticleSystem::new();
        let mut is_flying = false;
        let mut fly_vel = Vec3::zero();
        let mut horiz_vel = Vec3::zero();
        let mut reload_chunks = false;

        let mut ctx = CommandContext {
            player: &mut player,
            camera: &mut camera,
            world: &mut world,
            mob_manager: &mut mob_manager,
            items: &mut items,
            particles: &mut particles,
            is_flying: &mut is_flying,
            fly_velocity: &mut fly_vel,
            horiz_velocity: &mut horiz_vel,
            reload_chunks_requested: &mut reload_chunks,
        };

        chat.execute_command("/mobs", &mut ctx);
        assert!(!chat.messages.is_empty());
        assert!(chat
            .messages
            .iter()
            .any(|m| m.text.contains("Spawned Mobs")));
        assert!(chat.messages.iter().any(|m| m.text.contains("Zombie")));
        assert!(chat.messages.iter().any(|m| m.text.contains("Skeleton")));
    }

    #[test]
    fn test_chat_weather_commands() {
        let mut chat = ChatState::new();
        let mut mob_manager = MobManager::new();
        let mut player = Player::new();
        let mut camera = Camera::new(Vec3::new(0.0, 64.0, 0.0));
        let mut world = World::new();
        let mut items = Vec::new();
        let mut particles = ParticleSystem::new();
        let mut is_flying = false;
        let mut fly_vel = Vec3::zero();
        let mut horiz_vel = Vec3::zero();
        let mut reload_chunks = false;

        let mut ctx = CommandContext {
            player: &mut player,
            camera: &mut camera,
            world: &mut world,
            mob_manager: &mut mob_manager,
            items: &mut items,
            particles: &mut particles,
            is_flying: &mut is_flying,
            fly_velocity: &mut fly_vel,
            horiz_velocity: &mut horiz_vel,
            reload_chunks_requested: &mut reload_chunks,
        };

        chat.execute_command("/weather rain 120", &mut ctx);
        assert_eq!(
            ctx.world.weather.current_type,
            crate::world::weather::WeatherType::Rain
        );
        assert_eq!(ctx.world.weather.timer, 120.0);

        chat.execute_command("/weather thunder 60", &mut ctx);
        assert_eq!(
            ctx.world.weather.current_type,
            crate::world::weather::WeatherType::Thunder
        );
        assert_eq!(ctx.world.weather.timer, 60.0);

        chat.execute_command("/weather query", &mut ctx);
        assert!(chat
            .messages
            .iter()
            .any(|m| m.text.contains("weather is currently thunder")));

        chat.execute_command("/weather clear", &mut ctx);
        assert_eq!(
            ctx.world.weather.current_type,
            crate::world::weather::WeatherType::Clear
        );
    }
}
