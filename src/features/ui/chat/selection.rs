//! Zaznaczanie tekstu myszą, schowek (kopiuj/wklej/zaznacz wszystko) oraz menu kontekstowe czatu.

use crate::features::rendering::font;
use crate::platform::clipboard::{get_clipboard, set_clipboard};

use super::state::{ChatContextMenu, ChatContextTarget, ChatState};

impl ChatState {
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
                    set_clipboard(&text);
                }
            }
            1 => self.paste_into_input(&get_clipboard()),
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
        let start_x = 8 + font::measure_text("> ", 1, 1);
        let rel_x = (x - start_x).clamp(0, width as i32);
        let mut prefix = String::new();
        let mut cursor = 0;
        for (byte_index, ch) in self.input_text.char_indices() {
            let char_x = font::measure_text(&prefix, 1, 1);
            let next_prefix = format!("{prefix}{ch}");
            let next_x = font::measure_text(&next_prefix, 1, 1);
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

    pub(super) fn selected_input_text(&self) -> Option<String> {
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

    pub(super) fn delete_input_selection(&mut self) {
        if let Some((start, end)) = self.input_selection.take() {
            if start <= end && end <= self.input_text.len() {
                self.input_text.replace_range(start..end, "");
                self.cursor_pos = start;
            }
        }
    }

    pub(super) fn paste_into_input(&mut self, text: &str) {
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

    pub(super) fn visible_message_rows(&self, height: usize) -> ([(usize, i32); 12], usize) {
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
            let text_width = font::measure_text(text, 1, 1);
            if px < base_x || px > (base_x + text_width).min(width as i32 - base_x) {
                return None;
            }
            let relative_x = px - base_x;
            let mut closest = (0, i32::MAX);
            let mut prefix = String::new();
            for (char_index, ch) in text.chars().enumerate() {
                let char_x = font::measure_text(&prefix, 1, 1);
                let distance = (relative_x - char_x).abs();
                if distance < closest.1 {
                    closest = (char_index, distance);
                }
                prefix.push(ch);
            }
            let end_index = text.chars().count();
            let end_x = font::measure_text(text, 1, 1);
            if (relative_x - end_x).abs() < closest.1 {
                closest.0 = end_index;
            }
            return Some((index, closest.0));
        }
        None
    }

    pub(super) fn selected_text(&self) -> Option<String> {
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
}
