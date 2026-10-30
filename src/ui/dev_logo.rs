//! Console-style developer logo splash screen (shown once at startup).
//!
//! Loads `graphics/dev_logo.png` (any size / aspect ratio: square or rectangular) and draws it
//! centered, aspect-preserving, with a fade-in / hold / fade-out sequence.
//! A built-in copy is embedded in the binary; a file on disk overrides it (non-wasm platforms).

use std::io::Read;

static EMBEDDED_LOGO: &[u8] = include_bytes!("../../graphics/dev_logo.png");

/// Total splash length in seconds.
pub const DEV_LOGO_DURATION: f32 = 4.4;
const FADE_IN: f32 = 0.8;
const FADE_OUT: f32 = 1.4;
/// Duration of the menu fade-in from black after the logo ends.
pub const MENU_FADE_IN: f32 = 0.9;

/// Decoded RGBA logo (straight alpha, 0xAARRGGBB).
pub struct DevLogo {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

impl DevLogo {
    /// Loads the logo from disk (if present) or falls back to the embedded image.
    pub fn load() -> Option<Self> {
        #[cfg(not(any(target_arch = "wasm32", target_os = "vita", target_os = "xbox360", target_os = "xenon")))]
        {
            let mut candidates = vec![std::path::PathBuf::from("graphics/dev_logo.png")];
            if let Ok(exe) = std::env::current_exe() {
                if let Some(p) = exe.parent() {
                    candidates.push(p.join("graphics/dev_logo.png"));
                }
            }
            for path in candidates {
                if let Ok(bytes) = std::fs::read(&path) {
                    if let Ok(logo) = Self::from_png_bytes(&bytes) {
                        return Some(logo);
                    }
                }
            }
        }
        Self::from_png_bytes(EMBEDDED_LOGO).ok()
    }

    /// Minimal PNG decoder: non-interlaced, 8/16-bit gray / gray+alpha / RGB / RGBA and 8-bit palette.
    pub fn from_png_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 8 || data[..8] != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
            return Err("not a PNG".into());
        }
        let (mut w, mut h, mut depth, mut ctype, mut interlace) = (0usize, 0usize, 0u8, 0u8, 0u8);
        let mut palette: Vec<[u8; 3]> = Vec::new();
        let mut trns: Vec<u8> = Vec::new();
        let mut idat: Vec<u8> = Vec::new();

        let mut pos = 8;
        while pos + 8 <= data.len() {
            let len = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
            let kind = &data[pos + 4..pos + 8];
            let start = pos + 8;
            let end = start.checked_add(len).ok_or("bad chunk")?;
            if end + 4 > data.len() {
                return Err("truncated PNG".into());
            }
            let body = &data[start..end];
            match kind {
                b"IHDR" if len >= 13 => {
                    w = u32::from_be_bytes([body[0], body[1], body[2], body[3]]) as usize;
                    h = u32::from_be_bytes([body[4], body[5], body[6], body[7]]) as usize;
                    depth = body[8];
                    ctype = body[9];
                    interlace = body[12];
                }
                b"PLTE" => palette = body.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
                b"tRNS" => trns = body.to_vec(),
                b"IDAT" => idat.extend_from_slice(body),
                b"IEND" => break,
                _ => {}
            }
            pos = end + 4;
        }

        if w == 0 || h == 0 || w > 8192 || h > 8192 {
            return Err("invalid PNG size".into());
        }
        if interlace != 0 {
            return Err("interlaced PNG not supported".into());
        }
        let channels = match ctype {
            0 => 1,
            2 => 3,
            3 => 1,
            4 => 2,
            6 => 4,
            _ => return Err("unsupported PNG color type".into()),
        };
        if !(depth == 8 || (depth == 16 && ctype != 3)) {
            return Err("unsupported PNG bit depth".into());
        }
        let bpp = channels * (depth as usize / 8);
        let stride = w * bpp;

        let mut raw = Vec::with_capacity((stride + 1) * h);
        flate2::read::ZlibDecoder::new(&idat[..])
            .read_to_end(&mut raw)
            .map_err(|e| format!("PNG inflate error: {e}"))?;
        if raw.len() < (stride + 1) * h {
            return Err("PNG data too short".into());
        }

        // Unfilter scanlines.
        let mut img = vec![0u8; stride * h];
        for y in 0..h {
            let filter = raw[y * (stride + 1)];
            let line = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
            for x in 0..stride {
                let a = if x >= bpp { img[y * stride + x - bpp] as i32 } else { 0 };
                let b = if y > 0 { img[(y - 1) * stride + x] as i32 } else { 0 };
                let c = if x >= bpp && y > 0 { img[(y - 1) * stride + x - bpp] as i32 } else { 0 };
                let pred = match filter {
                    0 => 0,
                    1 => a,
                    2 => b,
                    3 => (a + b) / 2,
                    4 => {
                        let p = a + b - c;
                        let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
                        if pa <= pb && pa <= pc { a } else if pb <= pc { b } else { c }
                    }
                    _ => return Err("bad PNG filter".into()),
                };
                img[y * stride + x] = line[x].wrapping_add(pred as u8);
            }
        }

        // Convert to ARGB.
        let step = depth as usize / 8;
        let mut pixels = Vec::with_capacity(w * h);
        for y in 0..h {
            for x in 0..w {
                let o = y * stride + x * bpp;
                let ch = |i: usize| img[o + i * step]; // high byte for 16-bit
                let (r, g, b, a) = match ctype {
                    0 => (ch(0), ch(0), ch(0), 255),
                    2 => (ch(0), ch(1), ch(2), 255),
                    3 => {
                        let idx = img[o] as usize;
                        let p = palette.get(idx).copied().unwrap_or([0, 0, 0]);
                        (p[0], p[1], p[2], trns.get(idx).copied().unwrap_or(255))
                    }
                    4 => (ch(0), ch(0), ch(0), ch(1)),
                    _ => (ch(0), ch(1), ch(2), ch(3)),
                };
                pixels.push(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32);
            }
        }
        Ok(Self { width: w, height: h, pixels })
    }

    /// Fade multiplier (0..1) for the elapsed time.
    pub fn fade_at(t: f32) -> f32 {
        if t < FADE_IN {
            (t / FADE_IN).clamp(0.0, 1.0)
        } else if t > DEV_LOGO_DURATION - FADE_OUT {
            ((DEV_LOGO_DURATION - t) / FADE_OUT).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Draws the logo centered on a black screen, scaled to fit while preserving aspect ratio
    /// (works for both square and rectangular images).
    pub fn draw(&self, buffer: &mut [u32], buf_w: usize, buf_h: usize, fade: f32) {
        buffer.iter_mut().for_each(|p| *p = 0);
        if self.width == 0 || self.height == 0 || buf_w == 0 || buf_h == 0 {
            return;
        }
        let max_w = buf_w as f32 * 0.6;
        let max_h = buf_h as f32 * 0.5;
        let scale = (max_w / self.width as f32).min(max_h / self.height as f32);
        let dw = ((self.width as f32 * scale) as usize).max(1);
        let dh = ((self.height as f32 * scale) as usize).max(1);
        let x0 = (buf_w.saturating_sub(dw)) / 2;
        let y0 = (buf_h.saturating_sub(dh)) / 2;

        for dy in 0..dh {
            let sy = (dy * self.height / dh).min(self.height - 1);
            let row = (y0 + dy) * buf_w;
            if y0 + dy >= buf_h {
                break;
            }
            for dx in 0..dw {
                if x0 + dx >= buf_w {
                    break;
                }
                let sx = (dx * self.width / dw).min(self.width - 1);
                let px = self.pixels[sy * self.width + sx];
                let a = ((px >> 24) & 0xFF) as f32 / 255.0 * fade;
                if a <= 0.0 {
                    continue;
                }
                let r = (((px >> 16) & 0xFF) as f32 * a) as u32;
                let g = (((px >> 8) & 0xFF) as f32 * a) as u32;
                let b = ((px & 0xFF) as f32 * a) as u32;
                buffer[row + x0 + dx] = (r << 16) | (g << 8) | b;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_logo_decodes() {
        let logo = DevLogo::from_png_bytes(EMBEDDED_LOGO).expect("decode");
        assert!(logo.width > 0 && logo.height > 0);
        assert_eq!(logo.pixels.len(), logo.width * logo.height);
        let mut buf = vec![0u32; 320 * 240];
        logo.draw(&mut buf, 320, 240, 1.0);
        assert!(buf.iter().any(|&p| p != 0));
    }
}
