//! Minecraft-style title screen splash texts ("toasty").
//!
//! Provides an authentic collection of over 50 splash texts, pulsing animation,
//! angled rotation, drop shadowing, and click-to-cycle easter egg functionality.

use crate::graphics::font;
use crate::ui::draw::pack_color;

/// Collection of 60 curated Minecraft & RustCraft splash texts.
pub const SPLASH_TEXTS: &[&str] = &[
    "100% Pure Rust!",
    "Przestaj kopac prosto w dol!",
    "Pamietaj o pochodniach!",
    "Nie patrz w oczy Endermanowi!",
    "Zero-cost abstractions!",
    "Bezpieczenstwo pamieci gwarantowane!",
    "Lepiej niz Bedrock!",
    "Szybciej niz Java!",
    "Herobrine zostal usuniety!",
    "Kop diamenty, nie zelazo!",
    "Zbuduj schronienie przed zmrokiem!",
    "SssSSSS... Bum!",
    "Creeper? Aww man!",
    "Nie jedz surowego kurczaka!",
    "Gdzie jest moj kilof?!",
    "Teraz takze na PS Vita i Androidzie!",
    "60 FPS nawet na tosterze!",
    "Hardware OpenGL acceleration!",
    "Voxel magic!",
    "Szmaragdy waluta przyszlosci!",
    "Kto zgasil swiatlo w kopalni?",
    "Nie ma to jak zapach drewna!",
    "Chleb to podstawa przetrwania!",
    "Zrob se zbroje z diamentow!",
    "Pilnuj paska glodu!",
    "Wiecej FPS niz pikseli!",
    "Czysty kod, czyste voxele!",
    "Zawsze nos wiaderko wody!",
    "Drzwi chronia przed zombie!",
    "Patrz pod nogi w Netherze!",
    "Craftuj poki dzien jasny!",
    "Wielkie gory sniezne czekaja!",
    "Ciemnosc kryje skarby... i potwory!",
    "Zrob zapasowa skrzynie!",
    "Redstone to czysta magia!",
    "Zloty kilof za szybko sie psuje!",
    "Nie ufaj podejrzanym dzwiekom!",
    "Kup pan szmaragd od wiesniaka!",
    "Spij w lozku, aby zresetowac noc!",
    "Nie kladz lozka w Netherze!",
    "Krowy daja mleko, owce welne!",
    "Zbuduj zamek marzen!",
    "Klon Minecrafta, ale w Rust!",
    "Thread-safe bez wyscigow danych!",
    "Borrow checker zatwierdzil ten swiat!",
    "Cargo run --release i lecimy!",
    "Plywanie w lawie nie jest zdrowe!",
    "Zjedz pieczone ziemniaczki!",
    "Odglosy jaskin o 3 nad ranem!",
    "Uwaga na sypiacy sie zwir!",
    "Kop glebiej, diamenty sa na dole!",
    "Wioska tuz za wzgorzem!",
    "Wracaj do bazy przed switem!",
    "Wcisnij F3 po statystyki!",
    "Nie zapomnij oswoic wilka!",
    "Optymalizacja ponad wszystko!",
    "Crafting table to twoje drugie ja!",
    "Kopiesz wegiel na pochodnie?",
    "Zelazo na start to podstawa!",
    "Nie ma bugow, sa niespodzianki!",
];

/// Renders a dynamic, pulsing, angled yellow splash text next to the main title.
pub fn draw_splash_text(
    buffer: &mut [u32],
    buf_w: usize,
    buf_h: usize,
    text: &str,
    center_x: i32,
    center_y: i32,
    time_sec: f32,
) {
    // Minecraft-style splash pulsation: oscillates size smoothly
    let pulse = (time_sec * 6.0).sin();
    let scale = 1.45 + 0.15 * pulse.abs();

    // Signature -18° angle with subtle organic sway
    let angle = -0.32 + 0.03 * (time_sec * 3.2).sin();
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    let text_w = font::measure_text(text, 1, 1);
    let text_h = 7;
    let half_w = text_w as f32 / 2.0;
    let half_h = text_h as f32 / 2.0;

    // Collect base glyph pixel coordinates relative to center
    let mut points: Vec<(f32, f32)> = Vec::with_capacity(text_w as usize * 8);
    font::draw_text(text, 0, 0, 1, 1, |px, py| {
        points.push((px as f32 - half_w, py as f32 - half_h));
    });

    let shadow_color = pack_color(0.24, 0.22, 0.04); // Dark golden-brown shadow
    let text_color = pack_color(1.0, 1.0, 0.28);     // Signature Minecraft vibrant golden yellow

    let draw_scaled_pts = |buf: &mut [u32], ox: f32, oy: f32, col: u32| {
        for &(lx, ly) in &points {
            let sx = lx * scale;
            let sy = ly * scale;
            let rx = sx * cos_a - sy * sin_a + ox;
            let ry = sx * sin_a + sy * cos_a + oy;
            let x0 = rx.round() as i32;
            let y0 = ry.round() as i32;

            // Plot 2x2 retro chunky pixels
            for dy in 0..2 {
                for dx in 0..2 {
                    let px = x0 + dx;
                    let py = y0 + dy;
                    if px >= 0 && py >= 0 && px < buf_w as i32 && py < buf_h as i32 {
                        buf[py as usize * buf_w + px as usize] = col;
                    }
                }
            }
        }
    };

    // Draw drop shadow offset by +2.5px down-right
    draw_scaled_pts(buffer, center_x as f32 + 2.5, center_y as f32 + 2.5, shadow_color);
    // Draw bright foreground text
    draw_scaled_pts(buffer, center_x as f32, center_y as f32, text_color);
}
