//! Change nickname modal dialogue UI layout and rendering.

use crate::core::i18n::t;
use crate::graphics::font;
use crate::ui::draw::pack_color;
use crate::ui::widgets::{Button, HStack, Rect, TextInput, UiCanvas};

/// Computes bounding boxes `(x, y, width, height)` for the 4 elements in the nickname dialog:
/// - 0: Nickname input field box
/// - 1: [ DONE ] button
/// - 2: [ RANDOM ] button
/// - 3: [ CANCEL ] button
pub fn nickname_dialog_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 4] {
    let modal = Rect::centered_in(width, height, 460, 220);
    let input = Rect::centered_x(width, modal.y + 72, 330, 32).to_tuple();
    let btns: [(i32, i32, i32, i32); 3] =
        HStack::centered_row(width, modal.y + 144, 102, 30, 3, 12).array(102);

    [input, btns[0], btns[1], btns[2]]
}

/// Renders the Change Nickname modal dialogue with text input, selection, buttons, and gamepad hints.
pub fn draw_nickname_dialog(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    current_nick: &str,
    is_selected: bool,
    hovered_elem: Option<usize>,
    cursor_visible: bool,
    is_gamepad_active: bool,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();

    // 1. Full-screen dimmed backdrop
    ui.backdrop(None, pack_color(0.0, 0.0, 0.0), 0.70);

    // 2. Modal panel background and double beveled borders
    let modal = Rect::centered_in(width, height, 460.min(width as i32 - 40), 220);
    ui.modal_panel(
        modal,
        pack_color(0.09, 0.11, 0.15),
        1.0,
        pack_color(0.40, 0.50, 0.70),
        pack_color(0.04, 0.05, 0.08),
    );
    ui.bevel_border(
        modal.inset(3),
        1,
        pack_color(0.18, 0.22, 0.32),
        pack_color(0.04, 0.05, 0.08),
    );

    // 3. Title & prompt
    ui.text_centered_x(
        t("nickname.title"),
        cx,
        modal.y + 18,
        2,
        2,
        pack_color(0.95, 0.85, 0.40),
    );
    ui.text_centered_x(
        t("nickname.prompt"),
        cx,
        modal.y + 52,
        1,
        1,
        pack_color(0.80, 0.85, 0.95),
    );

    let rects = nickname_dialog_rects(width, height);
    let input_rect = Rect::from_tuple(rects[0]);

    // 4. Nickname Input Field
    let is_input_focused = hovered_elem == Some(0) || hovered_elem.is_none();
    ui.text_input(
        input_rect,
        TextInput::new(current_nick)
            .placeholder("Player")
            .focused(is_input_focused)
            .selected(is_selected)
            .cursor(cursor_visible),
    );

    // Character counter "12/24"
    let counter_str = format!("{}/24", current_nick.len());
    ui.text_right(
        &counter_str,
        input_rect.right() - 8,
        input_rect.y + (input_rect.h - font::GLYPH_H as i32) / 2,
        1,
        1,
        pack_color(0.50, 0.55, 0.65),
    );

    // Help text below input
    ui.text_centered_x(
        t("nickname.help"),
        cx,
        input_rect.bottom() + 8,
        1,
        1,
        pack_color(0.55, 0.60, 0.70),
    );

    // 5. Action Buttons (Done, Random, Cancel)
    let button_labels = [
        t("nickname.done"),
        t("nickname.random"),
        t("nickname.cancel"),
    ];

    for (i, (&label, &rect)) in button_labels.iter().zip(rects[1..].iter()).enumerate() {
        ui.button(
            rect,
            Button::new(label)
                .scale(1)
                .hovered(hovered_elem == Some(i + 1)),
        );
    }

    // 6. Gamepad navigation hint
    let footer_hint = if is_gamepad_active {
        t("nickname.controls_hint")
    } else {
        "[ENTER] Save   [R] Randomize   [ESC] Cancel"
    };
    ui.text_centered_x(
        footer_hint,
        cx,
        modal.bottom() - 22,
        1,
        1,
        pack_color(0.60, 0.65, 0.75),
    );
}
