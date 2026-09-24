use crate::core::settings::GameSettings;
use crate::graphics::font;
use crate::ui::draw::{draw_text_colored_shadowed, draw_text_shadowed, fill_rect, fill_rect_alpha, pack_color};

pub fn settings_option_rects(width: usize, height: usize) -> [(i32, i32, i32, i32); 10] {
    let opt_w = 460;
    let opt_h = 26;
    let spacing = 4;
    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    let start_y = cy - 162;

    [
        (cx - opt_w / 2, start_y, opt_w, opt_h),
        (cx - opt_w / 2, start_y + opt_h + spacing, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 2, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 3, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 4, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 5, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 6, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 7, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 8, opt_w, opt_h),
        (cx - opt_w / 2, start_y + (opt_h + spacing) * 9 + 4, opt_w, opt_h),
    ]
}

pub fn draw_settings_menu(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    settings: &GameSettings,
    hovered_idx: Option<usize>,
) {
    fill_rect_alpha(buffer, width, height, 0, 0, width as i32, height as i32, pack_color(0.0, 0.0, 0.0), 0.75);

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;

    let title = "USTAWIENIA GRAFICZNE";
    let t_w = font::measure_text(title, 3, 2);
    draw_text_colored_shadowed(buffer, width, height, title, cx - t_w / 2, cy - 192, 3, 2, pack_color(0.95, 0.85, 0.35));

    let options = [
        format!("ZASIEG WIDZENIA: < {} CHUNKOW >", settings.render_distance),
        format!("LISCIE: {}", if settings.fancy_leaves { "PRZESWITUJACE (FANCY)" } else { "ZWARTE (FAST)" }),
        format!("MGLA: {}", if settings.fog { "WLACZONA" } else { "WYLACZONA" }),
        format!("CHMURY: {}", if settings.clouds { "WLACZONE" } else { "WYLACZONE" }),
        format!("CZASTECZKI (PARTICLES): {}", if settings.particles { "WLACZONE" } else { "WYLACZONE" }),
        format!("ANIMACJA CHUNKOW: {}", if settings.chunk_anim { "WLACZONA" } else { "WYLACZONA" }),
        format!("POLE WIDZENIA (FOV): < {:.0} DEG >", settings.fov_deg),
        format!("EFEKTY PODWODNE: {}", if settings.underwater_fx { "WLACZONE" } else { "WYLACZONE" }),
        format!("NAKLADKA DEBUGOWANIA: {}", if settings.debug_overlay { "WLACZONA" } else { "WYLACZONA" }),
        #[cfg(target_os = "vita")]
        "ZAPISZ I WSTECZ [O]".to_string(),
        #[cfg(not(target_os = "vita"))]
        "ZAPISZ I WSTECZ (ESC)".to_string(),
    ];

    let rects = settings_option_rects(width, height);

    for (i, (label, &(bx, by, bw, bh))) in options.iter().zip(rects.iter()).enumerate() {
        let is_hover = hovered_idx == Some(i);

        let body_color = if is_hover {
            if i == 9 { pack_color(0.25, 0.45, 0.35) } else { pack_color(0.32, 0.42, 0.60) }
        } else {
            if i == 9 { pack_color(0.18, 0.30, 0.22) } else { pack_color(0.20, 0.22, 0.26) }
        };

        let top_left = if is_hover { pack_color(0.75, 0.85, 1.0) } else { pack_color(0.55, 0.55, 0.60) };
        let bottom_right = if is_hover { pack_color(0.15, 0.20, 0.35) } else { pack_color(0.10, 0.10, 0.12) };

        fill_rect(buffer, width, height, bx, by, bw, bh, body_color);
        fill_rect(buffer, width, height, bx, by, bw, 2, top_left);
        fill_rect(buffer, width, height, bx, by, 2, bh, top_left);
        fill_rect(buffer, width, height, bx, by + bh - 2, bw, 2, bottom_right);
        fill_rect(buffer, width, height, bx + bw - 2, by, 2, bh, bottom_right);

        let scale = 2;
        let tw = font::measure_text(label, scale, 1);
        let tx = bx + (bw - tw) / 2;
        let ty = by + (bh - (font::GLYPH_H as i32 * scale)) / 2;

        if is_hover {
            draw_text_colored_shadowed(buffer, width, height, label, tx, ty, scale, 1, pack_color(1.0, 1.0, 0.3));
        } else {
            draw_text_shadowed(buffer, width, height, label, tx, ty, scale, 1);
        }
    }

    #[cfg(target_os = "vita")]
    let hint = "STEROWANIE: [D-PAD] WYBOR/ZMIANA | [X / DOTYK] PRZELACZ | [O] WSTECZ";
    #[cfg(not(target_os = "vita"))]
    let hint = "STEROWANIE: [D-PAD / STRZALKI] WYBOR/ZMIANA | [A / ENTER / LPM] PRZELACZ | [B / ESC] WSTECZ";
    let hw = font::measure_text(hint, 1, 1);
    draw_text_colored_shadowed(buffer, width, height, hint, cx - hw / 2, height as i32 - 20, 1, 1, pack_color(0.7, 0.75, 0.85));
}
