use crate::gameplay::player::GameMode;
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, fill_rect, fill_rect_alpha, pack_color};

pub fn create_world_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 6] {
    let cx = width as i32 / 2;
    let _ = height;

    let input_w = 460;
    let input_h = 36;
    let x_left = cx - input_w / 2;

    let y_name = 120;
    let y_seed = 195;
    let y_mode = 275;
    let y_bottom = 370;

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
        (x_left, y_mode, input_w, 42),
        // 4: Create World button
        (cx - btn_w - btn_gap / 2, y_bottom, btn_w, btn_h),
        // 5: Cancel button
        (cx + btn_gap / 2, y_bottom, btn_w, btn_h),
    ]
}

pub fn draw_create_world_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    slot: u8,
    name: &str,
    seed_str: &str,
    gamemode: GameMode,
    focused_element: usize, // 0: Name, 1: Seed, 2: RandSeedBtn, 3: GameMode, 4: CreateBtn, 5: CancelBtn
    hovered_element: Option<usize>,
    blink_timer: f32,
    is_selected: bool,
) {
    let cx = width as i32 / 2;
    let rects = create_world_rects(width, height);

    // Pełne, czyste tło menu
    fill_rect(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.06, 0.07, 0.09));
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.02, 0.03, 0.05), 0.65);

    // Główny kontener dialogowy
    let panel_w = 540;
    let panel_h = 450;
    let panel_x = cx - panel_w / 2;
    let panel_y = 25;

    fill_rect_alpha(buffer, width, height, panel_x, panel_y, panel_w, panel_h, pack_color(0.10, 0.12, 0.16), 0.90);
    fill_rect(buffer, width, height, panel_x, panel_y, panel_w, 2, pack_color(0.35, 0.40, 0.50));
    fill_rect(buffer, width, height, panel_x, panel_y, 2, panel_h, pack_color(0.35, 0.40, 0.50));
    fill_rect(buffer, width, height, panel_x, panel_y + panel_h - 2, panel_w, 2, pack_color(0.05, 0.06, 0.08));
    fill_rect(buffer, width, height, panel_x + panel_w - 2, panel_y, 2, panel_h, pack_color(0.05, 0.06, 0.08));

    // Nagłówek
    let title = "TWORZENIE NOWEGO SWIATA";
    let title_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - title_w / 2, panel_y + 16, 3, 2, pack_color(0.95, 0.85, 0.35));

    let subtitle = format!("Konfiguracja nowego swiata dla slotu {:02}", slot);
    let sub_w = font::measure_text(&subtitle, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, &subtitle, cx - sub_w / 2, panel_y + 44, 1, 1, pack_color(0.7, 0.75, 0.85));

    let cursor_visible = (blink_timer % 0.8) < 0.4;

    // --- 1. NAZWA ŚWIATA ---
    let (nx, ny, nw, nh) = rects[0];
    draw_text_colored_shadowed(buffer, width, height, "NAZWA SWIATA:", nx, ny - 16, 1, 1, pack_color(0.85, 0.90, 0.95));

    let name_is_active = focused_element == 0;
    let name_is_hover = hovered_element == Some(0);
    let name_border = if name_is_active {
        pack_color(0.98, 0.85, 0.25)
    } else if name_is_hover {
        pack_color(0.5, 0.7, 1.0)
    } else {
        pack_color(0.25, 0.28, 0.35)
    };

    fill_rect(buffer, width, height, nx - 2, ny - 2, nw + 4, nh + 4, name_border);
    fill_rect(buffer, width, height, nx, ny, nw, nh, pack_color(0.06, 0.07, 0.09));

    let display_name = if name.is_empty() && !name_is_active {
        format!("Nowy Swiat {:02}", slot)
    } else {
        name.to_string()
    };

    let name_is_selected = name_is_active && is_selected && !name.is_empty();
    if name_is_selected {
        let text_w = font::measure_text(&display_name, 2, 1);
        fill_rect(buffer, width, height, nx + 10, ny + 8, text_w + 4, 20, pack_color(0.2, 0.45, 0.85));
    }

    let name_text_color = if name.is_empty() && !name_is_active {
        pack_color(0.5, 0.55, 0.6)
    } else {
        pack_color(1.0, 1.0, 1.0)
    };
    draw_text_colored_shadowed(buffer, width, height, &display_name, nx + 12, ny + 10, 2, 1, name_text_color);

    if name_is_active && !name_is_selected && cursor_visible {
        let text_w = font::measure_text(&display_name, 2, 1);
        let cur_x = (nx + 12 + text_w + 2).min(nx + nw - 14);
        draw_text_colored_shadowed(buffer, width, height, "_", cur_x, ny + 10, 2, 1, pack_color(1.0, 0.9, 0.3));
    }

    // --- 2. SEED ŚWIATA ---
    let (sx, sy, sw, sh) = rects[1];
    draw_text_colored_shadowed(buffer, width, height, "SEED (POZOSTAW PUSTE DLA LOSOWEGO):", sx, sy - 16, 1, 1, pack_color(0.85, 0.90, 0.95));

    let seed_is_active = focused_element == 1;
    let seed_is_hover = hovered_element == Some(1);
    let seed_border = if seed_is_active {
        pack_color(0.98, 0.85, 0.25)
    } else if seed_is_hover {
        pack_color(0.5, 0.7, 1.0)
    } else {
        pack_color(0.25, 0.28, 0.35)
    };

    fill_rect(buffer, width, height, sx - 2, sy - 2, sw + 4, sh + 4, seed_border);
    fill_rect(buffer, width, height, sx, sy, sw, sh, pack_color(0.06, 0.07, 0.09));

    let display_seed = if seed_str.is_empty() && !seed_is_active {
        "(Losowy seed)".to_string()
    } else {
        seed_str.to_string()
    };

    let seed_is_selected = seed_is_active && is_selected && !seed_str.is_empty();
    if seed_is_selected {
        let text_w = font::measure_text(&display_seed, 2, 1);
        fill_rect(buffer, width, height, sx + 10, sy + 8, text_w + 4, 20, pack_color(0.2, 0.45, 0.85));
    }

    let seed_text_color = if seed_str.is_empty() && !seed_is_active {
        pack_color(0.5, 0.55, 0.6)
    } else {
        pack_color(1.0, 1.0, 1.0)
    };
    draw_text_colored_shadowed(buffer, width, height, &display_seed, sx + 12, sy + 10, 2, 1, seed_text_color);

    if seed_is_active && !seed_is_selected && cursor_visible {
        let text_w = font::measure_text(&display_seed, 2, 1);
        let cur_x = (sx + 12 + text_w + 2).min(sx + sw - 14);
        draw_text_colored_shadowed(buffer, width, height, "_", cur_x, sy + 10, 2, 1, pack_color(1.0, 0.9, 0.3));
    }

    // Przycisk LOSUJ SEED
    let (rx, ry, rw, rh) = rects[2];
    let rand_is_active = focused_element == 2;
    let rand_is_hover = hovered_element == Some(2);
    let rand_body = if rand_is_hover || rand_is_active { pack_color(0.35, 0.45, 0.65) } else { pack_color(0.22, 0.25, 0.32) };
    let rand_border = if rand_is_active { pack_color(0.98, 0.85, 0.25) } else { pack_color(0.5, 0.55, 0.65) };

    fill_rect(buffer, width, height, rx - 2, ry - 2, rw + 4, rh + 4, rand_border);
    fill_rect(buffer, width, height, rx, ry, rw, rh, rand_body);
    let r_label = "LOSUJ";
    let r_tw = font::measure_text(r_label, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, r_label, rx + (rw - r_tw) / 2, ry + (rh - 5) / 2, 1, 1, pack_color(0.95, 0.95, 0.95));

    // --- 3. DOMYŚLNY TRYB GRY ---
    let (gx, gy, gw, gh) = rects[3];
    draw_text_colored_shadowed(buffer, width, height, "TRYB GRY:", gx, gy - 16, 1, 1, pack_color(0.85, 0.90, 0.95));

    let mode_is_active = focused_element == 3;
    let mode_is_hover = hovered_element == Some(3);
    let mode_border = if mode_is_active {
        pack_color(0.98, 0.85, 0.25)
    } else if mode_is_hover {
        pack_color(0.5, 0.7, 1.0)
    } else {
        pack_color(0.25, 0.28, 0.35)
    };

    let mode_bg = if mode_is_hover || mode_is_active {
        pack_color(0.25, 0.32, 0.45)
    } else {
        pack_color(0.16, 0.19, 0.26)
    };

    fill_rect(buffer, width, height, gx - 2, gy - 2, gw + 4, gh + 4, mode_border);
    fill_rect(buffer, width, height, gx, gy, gw, gh, mode_bg);

    let (mode_btn_text, mode_text_color, mode_desc) = match gamemode {
        GameMode::Survival => (
            "TRYB GRY: SURVIVAL (PRZETRWANIE)",
            pack_color(0.4, 0.95, 0.45),
            "Paski zdrowia i glodu, potwory, niszczenie blokow wymaga czasu.",
        ),
        GameMode::Creative => (
            "TRYB GRY: KREATYWNY (BUDOWANIE)",
            pack_color(0.4, 0.85, 1.0),
            "Latanie, niesmiertelnosc, blyskawiczne niszczenie, pelny ekwipunek.",
        ),
    };

    let m_tw = font::measure_text(mode_btn_text, 2, 1);
    draw_text_colored_shadowed(buffer, width, height, mode_btn_text, gx + (gw - m_tw) / 2, gy + 12, 2, 1, mode_text_color);

    let desc_w = font::measure_text(mode_desc, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, mode_desc, cx - desc_w / 2, gy + gh + 8, 1, 1, pack_color(0.65, 0.72, 0.82));

    // --- 4. PRZYCISK STWÓRZ ŚWIAT ---
    let (bx, by, bw, bh) = rects[4];
    let create_is_active = focused_element == 4;
    let create_is_hover = hovered_element == Some(4);

    let create_bg = if create_is_hover || create_is_active { pack_color(0.25, 0.65, 0.35) } else { pack_color(0.18, 0.45, 0.25) };
    let create_border = if create_is_active { pack_color(1.0, 0.95, 0.3) } else if create_is_hover { pack_color(0.6, 1.0, 0.6) } else { pack_color(0.3, 0.7, 0.4) };

    fill_rect(buffer, width, height, bx - 2, by - 2, bw + 4, bh + 4, create_border);
    fill_rect(buffer, width, height, bx, by, bw, bh, create_bg);

    #[cfg(target_os = "vita")]
    let create_lbl = "STWORZ SWIAT [X]";
    #[cfg(not(target_os = "vita"))]
    let create_lbl = "STWORZ SWIAT (ENTER)";

    let c_tw = font::measure_text(create_lbl, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, create_lbl, bx + (bw - c_tw) / 2, by + (bh - 5) / 2, 1, 1, pack_color(1.0, 1.0, 1.0));

    // --- 5. PRZYCISK ANULUJ ---
    let (ax, ay, aw, ah) = rects[5];
    let cancel_is_active = focused_element == 5;
    let cancel_is_hover = hovered_element == Some(5);

    let cancel_bg = if cancel_is_hover || cancel_is_active { pack_color(0.45, 0.25, 0.25) } else { pack_color(0.30, 0.18, 0.18) };
    let cancel_border = if cancel_is_active { pack_color(1.0, 0.95, 0.3) } else if cancel_is_hover { pack_color(1.0, 0.6, 0.6) } else { pack_color(0.55, 0.35, 0.35) };

    fill_rect(buffer, width, height, ax - 2, ay - 2, aw + 4, ah + 4, cancel_border);
    fill_rect(buffer, width, height, ax, ay, aw, ah, cancel_bg);

    #[cfg(target_os = "vita")]
    let cancel_lbl = "ANULUJ [O]";
    #[cfg(not(target_os = "vita"))]
    let cancel_lbl = "ANULUJ (ESC)";

    let a_tw = font::measure_text(cancel_lbl, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, cancel_lbl, ax + (aw - a_tw) / 2, ay + (ah - 5) / 2, 1, 1, pack_color(1.0, 1.0, 1.0));

    // Pasek pomocy / nawigacji na dole ekranu
    #[cfg(target_os = "vita")]
    let footer = "[D-PAD] NAWIGACJA | [X] PRZELACZ/WYBIERZ | [TROJKAT] LOSUJ SEED | [O] WSTECZ";
    #[cfg(not(target_os = "vita"))]
    let footer = "[TAB / STRZALKI] NAWIGACJA | [KLAWIATURA] WPISYWANIE | [ENTER] STWORZ | [ESC] ANULUJ";

    let ft_w = font::measure_text(footer, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, footer, cx - ft_w / 2, height as i32 - 18, 1, 1, pack_color(0.65, 0.72, 0.85));
}
