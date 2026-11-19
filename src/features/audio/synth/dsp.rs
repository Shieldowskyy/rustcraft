//! Prymitywy DSP do proceduralnej syntezy dźwięku (`FastRng`, `IirFilter`, `BiquadFilter`, `render`, `add_bp_grains`).

use std::f32::consts::TAU;

#[derive(Clone, Copy)]
pub struct FastRng {
    state: u32,
}

impl FastRng {
    #[inline(always)]
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x12345678 } else { seed },
        }
    }

    #[inline(always)]
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    #[inline(always)]
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32 / 2147483648.0) - 1.0
    }
}

#[derive(Clone, Copy)]
pub struct IirFilter {
    alpha: f32,
    prev_out: f32,
    prev_in: f32,
    is_highpass: bool,
}

impl IirFilter {
    pub fn lowpass(cutoff_hz: f32, sr: f32) -> Self {
        let rc = 1.0 / (TAU * cutoff_hz.max(10.0));
        let dt = 1.0 / sr;
        Self {
            alpha: (dt / (rc + dt)).clamp(0.001, 0.999),
            prev_out: 0.0,
            prev_in: 0.0,
            is_highpass: false,
        }
    }

    pub fn highpass(cutoff_hz: f32, sr: f32) -> Self {
        let rc = 1.0 / (TAU * cutoff_hz.max(10.0));
        let dt = 1.0 / sr;
        Self {
            alpha: (rc / (rc + dt)).clamp(0.001, 0.999),
            prev_out: 0.0,
            prev_in: 0.0,
            is_highpass: true,
        }
    }

    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let out = if self.is_highpass {
            self.alpha * (self.prev_out + input - self.prev_in)
        } else {
            self.prev_out + self.alpha * (input - self.prev_out)
        };
        self.prev_in = input;
        self.prev_out = out;
        out
    }
}

#[derive(Clone, Copy)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl BiquadFilter {
    fn new(b0: f32, b1: f32, b2: f32, a0: f32, a1: f32, a2: f32) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn params(hz: f32, q: f32, sr: f32) -> (f32, f32) {
        let w0 = TAU * hz.clamp(20.0, sr * 0.48) / sr;
        (w0.cos(), w0.sin() / (2.0 * q.max(0.1)))
    }

    pub fn bandpass(center_hz: f32, q: f32, sr: f32) -> Self {
        let (cos_w0, alpha) = Self::params(center_hz, q, sr);
        Self::new(alpha, 0.0, -alpha, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
    }

    pub fn lowpass(cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let (cos_w0, alpha) = Self::params(cutoff_hz, q, sr);
        let b1 = 1.0 - cos_w0;
        Self::new(b1 * 0.5, b1, b1 * 0.5, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
    }

    pub fn highpass(cutoff_hz: f32, q: f32, sr: f32) -> Self {
        let (cos_w0, alpha) = Self::params(cutoff_hz, q, sr);
        let b0 = (1.0 + cos_w0) * 0.5;
        Self::new(b0, -(1.0 + cos_w0), b0, 1.0 + alpha, -2.0 * cos_w0, 1.0 - alpha)
    }

    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

#[inline]
pub(super) fn render(
    sr: f32,
    duration: f32,
    mut f: impl FnMut(usize, f32, f32) -> f32,
) -> Vec<f32> {
    let count = (sr * duration) as usize;
    let inv_count = 1.0 / count.max(1) as f32;
    (0..count)
        .map(|i| f(i, i as f32 / sr, i as f32 * inv_count))
        .collect()
}

pub(super) fn add_bp_grains(
    out: &mut [f32],
    rng: &mut FastRng,
    sr: f32,
    g_dur: f32,
    q: f32,
    vol: f32,
    impulse: bool,
    grains: &[(f32, f32, f32)],
) {
    let g_len = (sr * g_dur) as usize;
    for &(time, freq, amp) in grains {
        let start = (time * sr) as usize;
        let mut bp = BiquadFilter::bandpass(freq, q, sr);
        for i in 0..g_len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / g_len as f32;
            let raw = if impulse && i < 3 {
                1.0
            } else if impulse {
                rng.next_f32() * 0.4
            } else {
                rng.next_f32()
            };
            out[start + i] += bp.process(raw) * (1.0 - t).powi(2) * amp * vol;
        }
    }
}

#[inline]
pub(super) fn normalize_and_scale(samples: &mut [f32], target_vol: f32) {
    let peak = samples.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
    if peak > 1e-4 {
        let mult = target_vol / peak;
        for s in samples.iter_mut() {
            *s = (*s * mult).clamp(-1.0, 1.0);
        }
    }
}
