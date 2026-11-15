//! Create World menu screen and interactive UI element positioning.

use crate::gameplay::player::GameMode;
use crate::ui::draw::pack_color;
use crate::ui::widgets::{Button, ButtonVariant, Rect, TextInput, UiCanvas};

/// Calculates layout bounding boxes for interactive create-world form elements.
///
/// # Returns
/// An array of 7 rectangles `[(x, y, width, height); 7]`:
/// - `0`: Name input field
/// - `1`: Seed input field
/// - `2`: Random seed generator button
/// - `3`: Game mode toggle button
/// - `4`: Quests enabled/disabled toggle button
/// - `5`: Create World action button
/// - `6`: Cancel action button
pub fn create_world_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 7] {
    let cx = width as i32 / 2;
    let _ = height;

    let input_w = 460;
    let input_h = 36;
    let x_left = cx - input_w / 2;

    let y_name = 112;
    let y_seed = 180;
    let y_mode = 252;
    let y_quests = 326;
    let y_bottom = 400;

    let seed_input_w = 320;
    let seed_btn_w = 130;
    let seed_btn_x = x_left + seed_input_w + 10;

    let btn_w = 220;
    let btn_h = 42;
    let btn_gap = 20;

    [
        // 0: Name input box
        (x_left, y_name, input_w, input_h),
        // 1: Seed input box
        (x_left, y_seed, seed_input_w, input_h),
        // 2: Randomize seed button
        (seed_btn_x, y_seed, seed_btn_w, input_h),
        // 3: GameMode toggle button
        (x_left, y_mode, input_w, 40),
        // 4: Quests toggle button
        (x_left, y_quests, input_w, 36),
        // 5: Create World button
        (cx - btn_w - btn_gap / 2, y_bottom, btn_w, btn_h),
        // 6: Cancel button
        (cx + btn_gap / 2, y_bottom, btn_w, btn_h),
    ]
}

/// Renders the Create World configuration dialogue.
pub fn draw_create_world_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    slot: u8,
    name: &str,
    seed_str: &str,
    gamemode: GameMode,
    quests_enabled: bool,
    focused_element: usize, // 0: Name, 1: Seed, 2: RandSeedBtn, 3: GameMode, 4: Quests, 5: CreateBtn, 6: CancelBtn
    hovered_element: Option<usize>,
    blink_timer: f32,
    is_selected: bool,
    is_gamepad_active: bool,
) {
    let mut ui = UiCanvas::new(buffer, width, height);
    let cx = ui.cx();
    let rects = create_world_rects(width, height);

    // Dark backdrop overlay
    ui.backdrop(
        Some(pack_color(0.06, 0.07, 0.09)),
        pack_color(0.02, 0.03, 0.05),
        0.65,
    );

    // Main modal dialogue panel
    let panel = Rect::centered_x(width, 20, 540, 465);
    ui.modal_panel(
        panel,
        pack_color(0.10, 0.12, 0.16),
        0.90,
        pack_color(0.35, 0.40, 0.50),
        pack_color(0.05, 0.06, 0.08),
    );

    // Header title & subtitle
    let title = crate::core::i18n::t("create_world.title");
    ui.text_centered_x(title, cx, panel.y + 16, 3, 2, pack_color(0.95, 0.85, 0.35));

    let subtitle = format!("Configure world parameters for slot {:02}", slot);
    ui.text_centered_x(&subtitle, cx, panel.y + 44, 1, 1, pack_color(0.7, 0.75, 0.85));

    let cursor_visible = (blink_timer % 0.8) < 0.4;

    // --- 1. WORLD NAME ---
    let default_name = format!("New World {:02}", slot);
    ui.text_input(
        rects[0],
        TextInput::new(name)
            .placeholder(&default_name)
            .label(crate::core::i18n::t("create_world.name_label"))
            .focused(focused_element == 0)
            .hovered(hovered_element == Some(0))
            .selected(is_selected)
            .cursor(cursor_visible)
            .outer_frame(),
    );

    // --- 2. WORLD SEED & RANDOM BUTTON ---
    ui.text_input(
        rects[1],
        TextInput::new(seed_str)
            .placeholder("(Random seed)")
            .label(crate::core::i18n::t("create_world.seed_label"))
            .focused(focused_element == 1)
            .hovered(hovered_element == Some(1))
            .selected(is_selected)
            .cursor(cursor_visible)
            .outer_frame(),
    );

    ui.button(
        rects[2],
        Button::new("RANDOM")
            .scale(1)
            .focused(focused_element == 2)
            .hovered(hovered_element == Some(2))
            .text_color(pack_color(0.95, 0.95, 0.95))
            .outer_frame(),
    );

    // --- 3. DEFAULT GAME MODE ---
    let mode_rect = Rect::from_tuple(rects[3]);
    ui.text(
        crate::core::i18n::t("create_world.gamemode_label"),
        mode_rect.x,
        mode_rect.y - 16,
        1,
        1,
        pack_color(0.85, 0.90, 0.95),
    );

    let (mode_btn_text, mode_text_color, mode_desc) = match gamemode {
        GameMode::Survival => (
            format!(
                "{}: {}",
                crate::core::i18n::t("create_world.gamemode_label"),
                crate::core::i18n::t("create_world.mode_survival")
            ),
            pack_color(0.4, 0.95, 0.45),
            "Health & hunger meters, mobs, block destruction takes time.",
        ),
        GameMode::Creative => (
            format!(
                "{}: {}",
                crate::core::i18n::t("create_world.gamemode_label"),
                crate::core::i18n::t("create_world.mode_creative")
            ),
            pack_color(0.4, 0.85, 1.0),
            "Flight, invulnerability, instant block breaking, full item inventory.",
        ),
    };

    let toggle_variant = ButtonVariant::Flat {
        bg: pack_color(0.16, 0.19, 0.26),
        hover_bg: pack_color(0.25, 0.32, 0.45),
        border: pack_color(0.25, 0.28, 0.35),
        hover_border: pack_color(0.5, 0.7, 1.0),
    };

    ui.button(
        mode_rect,
        Button::new(&mode_btn_text)
            .variant(toggle_variant)
            .focused(focused_element == 3)
            .hovered(hovered_element == Some(3))
            .text_color(mode_text_color)
            .outer_frame(),
    );
    ui.text_centered_x(
        mode_desc,
        cx,
        mode_rect.bottom() + 6,
        1,
        1,
        pack_color(0.65, 0.72, 0.82),
    );

    // --- 4. QUESTS TOGGLE BUTTON ---
    let quests_rect = Rect::from_tuple(rects[4]);
    let (q_state_lbl, q_state_col, q_desc) = if quests_enabled {
        (
            crate::core::i18n::t("create_world.quests_enabled"),
            pack_color(0.45, 0.95, 0.50),
            crate::core::i18n::t("create_world.quests_enabled_desc"),
        )
    } else {
        (
            crate::core::i18n::t("create_world.quests_disabled"),
            pack_color(0.95, 0.48, 0.48),
            crate::core::i18n::t("create_world.quests_disabled_desc"),
        )
    };
    let q_btn_text = format!(
        "{}: {}",
        crate::core::i18n::t("create_world.quests_label"),
        q_state_lbl
    );

    ui.button(
        quests_rect,
        Button::new(&q_btn_text)
            .variant(toggle_variant)
            .focused(focused_element == 4)
            .hovered(hovered_element == Some(4))
            .text_color(q_state_col)
            .outer_frame(),
    );
    ui.text_centered_x(
        q_desc,
        cx,
        quests_rect.bottom() + 6,
        1,
        1,
        pack_color(0.65, 0.72, 0.82),
    );

    // --- 5. CREATE WORLD BUTTON ---
    ui.button(
        rects[5],
        Button::new(crate::core::i18n::t("create_world.create_btn"))
            .primary()
            .scale(1)
            .focused(focused_element == 5)
            .hovered(hovered_element == Some(5))
            .outer_frame(),
    );

    // --- 6. CANCEL BUTTON ---
    ui.button(
        rects[6],
        Button::new(crate::core::i18n::t("create_world.cancel_btn"))
            .danger()
            .scale(1)
            .focused(focused_element == 6)
            .hovered(hovered_element == Some(6))
            .outer_frame(),
    );

    // Bottom help/navigation bar
    let footer = if is_gamepad_active {
        "[D-PAD / L-STICK] NAVIGATE | [A] SELECT/TOGGLE | [Y] RANDOM SEED | [B] CANCEL"
    } else {
        "[TAB / ARROWS] NAVIGATE | [KEYBOARD] TYPE | [ENTER] CREATE | [ESC] CANCEL"
    };
    ui.footer(footer, false);
}
