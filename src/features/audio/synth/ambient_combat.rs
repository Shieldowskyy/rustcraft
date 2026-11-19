//! Synteza dźwięków atmosferycznych (wiatr, jaskinia, woda, liście, piasek, deszcz, grzmot) oraz walki (łuk, tarcza, zbroja).

use std::f32::consts::{PI, TAU};

use super::dsp::{BiquadFilter, FastRng, IirFilter, normalize_and_scale, render};

pub(super) fn synth_wind_ambient(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp_sub, mut hp_sub) = (IirFilter::lowpass(180.0, sr), IirFilter::highpass(32.0, sr));
    let (mut bp_mid, mut bp_high, mut lp_hiss) = (
        BiquadFilter::bandpass(380.0, 1.2, sr),
        BiquadFilter::bandpass(680.0, 1.4, sr),
        IirFilter::lowpass(1100.0, sr),
    );
    let mut brown = 0.0f32;
    render(sr, dur.max(3.0), |_, _, nt| {
        let env = ((nt * PI).sin().powf(1.3)
            * (1.0 + 0.28 * (nt * PI * 3.5).sin() * (nt * PI).sin()))
        .clamp(0.0, 1.4);
        let raw = rng.next_f32();
        brown = (brown * 0.94 + raw * 0.06).clamp(-1.0, 1.0);
        let sub = hp_sub.process(lp_sub.process(raw + brown * 0.5)) * 1.6;
        let mid = bp_mid.process(raw) * 1.3;
        let high = bp_high.process(lp_hiss.process(raw)) * 0.75;
        ((sub * 0.65 + mid * 0.85 + high * 0.50) * env).clamp(-1.0, 1.0) * vol * 1.4
    })
}

pub(super) fn synth_cave_chord(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut p0, mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    let mut lp_cave = IirFilter::lowpass(480.0, sr);
    render(sr, dur.max(3.0), |_, t, nt| {
        let drift = (t * 0.3 * TAU).sin() * 0.8;
        p0 += TAU * (55.0 + drift) / sr;
        p1 += TAU * (65.41 - drift * 0.5) / sr;
        p2 += TAU * (77.78 + drift * 0.7) / sr;
        p3 += TAU * (98.0 + drift) / sr;
        let tone = p0.sin() * 0.35
            + p1.sin() * 0.30
            + p2.sin() * 0.22
            + p3.sin() * 0.18
            + lp_cave.process(rng.next_f32()) * 0.15;
        tone * (nt * PI).sin().powf(0.85) * vol * 1.3
    })
}

pub(super) fn synth_water_ambient(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp) = (
        BiquadFilter::bandpass(420.0, 1.4, sr),
        IirFilter::lowpass(750.0, sr),
    );
    let mut out = render(sr, dur.max(3.0), |_, t, nt| {
        let ripple = 0.75 + 0.25 * (t * 2.8 * TAU).sin() * (t * 0.7 * TAU).cos();
        lp.process(bp.process(rng.next_f32())) * ripple * (nt * PI).sin().powf(0.4) * 0.55
    });
    let bubbles = [
        (0.35, 620.0),
        (0.78, 840.0),
        (1.25, 520.0),
        (1.72, 950.0),
        (2.15, 710.0),
        (2.65, 880.0),
        (3.10, 600.0),
        (3.48, 760.0),
    ];
    let len = (sr * 0.045) as usize;
    for &(bt, base_f) in &bubbles {
        let (start, mut b_phase) = ((bt * sr) as usize, 0.0f32);
        for i in 0..len {
            if start + i >= out.len() {
                break;
            }
            let tau = i as f32 / sr;
            b_phase += TAU * (base_f + (base_f * 0.8) * (1.0 - (-tau * 85.0).exp())) / sr;
            out[start + i] += b_phase.sin() * (-tau * 55.0).exp() * 0.65;
        }
    }
    for s in &mut out {
        *s = (*s / (1.0 + s.abs() * 0.35)) * vol;
    }
    out
}

pub(super) fn synth_leaves_rustle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut bp_b) = (
        BiquadFilter::bandpass(2800.0, 1.8, sr),
        BiquadFilter::bandpass(5200.0, 2.2, sr),
        BiquadFilter::bandpass(480.0, 2.8, sr),
    );
    let mut hp = IirFilter::highpass(1600.0, sr);
    render(sr, dur.max(2.5), |_, t, nt| {
        let flutter = 0.55 + 0.30 * (t * 16.5 * TAU).sin() + 0.15 * (t * 23.0 * TAU).cos();
        let raw = rng.next_f32();
        let rustle = bp1.process(hp.process(raw)) * 0.65 + bp2.process(raw) * 0.40;
        (rustle * flutter + bp_b.process(raw) * 0.20) * (nt * PI).sin().powf(1.2) * vol * 1.35
    })
}

pub(super) fn synth_cave_rocks(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let count = (sr * dur.max(1.8)) as usize;
    let mut out = vec![0.0f32; count];
    let impacts = [
        (0.04, 1800.0, 0.90),
        (0.16, 2400.0, 0.75),
        (0.31, 1500.0, 0.65),
        (0.44, 2900.0, 0.55),
        (0.55, 2100.0, 0.45),
        (0.63, 2600.0, 0.35),
        (0.70, 3300.0, 0.25),
    ];
    let len = (sr * 0.035) as usize;
    for &(it, freq, amp) in &impacts {
        let (start, mut bp, mut phase) = (
            (it * sr) as usize,
            BiquadFilter::bandpass(freq, 3.5, sr),
            0.0f32,
        );
        for i in 0..len {
            if start + i >= count {
                break;
            }
            let t = i as f32 / len as f32;
            let imp = if i < 3 { 1.6 } else { rng.next_f32() * 0.3 };
            phase += TAU * (140.0 - t * 40.0) / sr;
            out[start + i] +=
                (bp.process(imp) * (1.0 - t).powi(2) + phase.sin() * (1.0 - t).powi(3) * 0.4) * amp;
        }
    }
    for &(delay_s, gain) in &[(0.065, 0.35), (0.135, 0.22), (0.220, 0.14)] {
        let (delay, mut lp) = ((delay_s * sr) as usize, IirFilter::lowpass(1400.0, sr));
        for i in delay..count {
            out[i] += lp.process(out[i - delay]) * gain;
        }
    }
    for s in &mut out {
        *s *= vol * 1.2;
    }
    out
}

pub(super) fn synth_desert_sand(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut bp1, mut bp2) = (
        IirFilter::highpass(780.0, sr),
        BiquadFilter::bandpass(2400.0, 1.8, sr),
        BiquadFilter::bandpass(4600.0, 2.4, sr),
    );
    let mut phase = 0.0f32;
    render(sr, dur.max(2.8), |_, t, nt| {
        let shimmer = 0.65 + 0.35 * (t * 42.0 * TAU).sin();
        let raw = rng.next_f32();
        phase += TAU * 110.0 / sr;
        let comp = hp.process(raw) * 0.35
            + (bp1.process(raw) * 0.60 + bp2.process(raw) * 0.40) * shimmer
            + phase.sin() * 0.08 * (1.0 - nt * 0.5);
        comp * (nt * PI).sin().powf(1.6) * vol * 1.3
    })
}

pub(super) fn synth_rain_ambient(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut hp) = (
        BiquadFilter::bandpass(1400.0, 1.2, sr),
        IirFilter::lowpass(2800.0, sr),
        IirFilter::highpass(300.0, sr),
    );
    let mut out = render(sr, dur.max(3.0), |_, t, nt| {
        let gust = 0.80 + 0.20 * (t * 1.5 * TAU).sin() * (t * 0.4 * TAU).cos();
        lp.process(hp.process(bp.process(rng.next_f32()))) * gust * (nt * PI).sin().powf(0.3) * 0.55
    });
    let len = (sr * 0.015) as usize;
    for _ in 0..35 {
        let start = (rng.next_f32() * 0.9 * dur * sr) as usize;
        let mut d_bp = BiquadFilter::bandpass(1800.0 + rng.next_f32() * 1400.0, 2.5, sr);
        for i in 0..len {
            if start + i >= out.len() {
                break;
            }
            let tau = i as f32 / len as f32;
            out[start + i] += d_bp.process(rng.next_f32()) * (1.0 - tau).powi(2) * 0.22;
        }
    }
    normalize_and_scale(&mut out, vol);
    out
}

pub(super) fn synth_thunder(rng: &mut FastRng, sr: f32, is_close: bool, vol: f32) -> Vec<f32> {
    let count = (sr * if is_close { 4.2 } else { 3.8 }) as usize;
    let mut out = vec![0.0f32; count];
    if is_close {
        let (crack_len, mut c_hp, mut c_bp) = (
            (sr * 0.15) as usize,
            IirFilter::highpass(3500.0, sr),
            BiquadFilter::bandpass(2400.0, 1.8, sr),
        );
        for i in 0..crack_len {
            let noise = rng.next_f32();
            out[i] += (c_hp.process(noise) * 0.6 + c_bp.process(noise) * 0.4)
                * (1.0 - i as f32 / crack_len as f32).powf(1.5)
                * 0.85;
        }
        let (thump_len, thump_start, mut phase) =
            ((sr * 0.45) as usize, (sr * 0.02) as usize, 0.0f32);
        for i in 0..thump_len {
            if thump_start + i >= count {
                break;
            }
            phase += TAU * (130.0 * (-(i as f32 / sr) * 12.0).exp() + 45.0) / sr;
            out[thump_start + i] +=
                phase.sin() * (1.0 - i as f32 / thump_len as f32).powf(1.8) * 0.75;
        }
    }
    let r_start = if is_close { (sr * 0.1) as usize } else { 0 };
    let (mut lp1, mut lp2, mut bp_sub) = (
        IirFilter::lowpass(85.0, sr),
        IirFilter::lowpass(120.0, sr),
        BiquadFilter::bandpass(55.0, 1.5, sr),
    );
    for i in r_start..count {
        let (t, norm_t, noise) = (
            (i - r_start) as f32 / sr,
            (i - r_start) as f32 / (count - r_start) as f32,
            rng.next_f32(),
        );
        let sub = bp_sub.process(noise) * 0.5 + lp1.process(lp2.process(noise)) * 0.5;
        let roll = 0.65 + 0.25 * (t * 3.2 * TAU).sin() + 0.15 * (t * 1.4 * TAU + 1.2).sin();
        out[i] += sub * roll * (1.0 - norm_t).powf(1.4) * 0.80;
    }
    normalize_and_scale(&mut out, vol);
    out
}

pub(super) fn synth_bow_shoot(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut phase) = (IirFilter::highpass(1600.0, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        phase += TAU * (140.0 + 220.0 * (-t * 30.0).exp()) / sr;
        let whoosh = hp.process(rng.next_f32())
            * if t < 0.08 {
                (1.0 - t / 0.08) * 0.45
            } else {
                0.0
            };
        (phase.sin() * (-t * 22.0).exp() * 0.75 + whoosh) * vol
    })
}

pub(super) fn synth_bow_hit(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(2400.0, 3.0, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        let snap = bp.process(rng.next_f32()) * (-t * 45.0).exp() * 0.9;
        phase += TAU * (420.0 * (-t * 25.0).exp() + 180.0) / sr;
        (snap + phase.sin() * (-t * 35.0).exp() * 0.6) * vol
    })
}

pub(super) fn synth_shield_block(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(1450.0, 5.0, sr),
        BiquadFilter::bandpass(2900.0, 6.0, sr),
        0.0f32,
    );
    render(sr, dur, |_, t, _| {
        let raw = rng.next_f32();
        phase += TAU * 180.0 / sr;
        ((bp1.process(raw) * 0.6 + bp2.process(raw) * 0.4) * (-t * 16.0).exp() * 0.85
            + phase.sin() * (-t * 28.0).exp() * 0.6)
            * vol
    })
}

pub(super) fn synth_armor_equip(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur) as usize];
    let mut bp = BiquadFilter::bandpass(3200.0, 4.0, sr);
    let clink_len = (0.05 * sr) as usize;
    for &ct in &[0.015f32, 0.075, 0.14] {
        let start = (ct * sr) as usize;
        for j in 0..clink_len {
            if start + j >= out.len() {
                break;
            }
            out[start + j] += bp.process(rng.next_f32()) * (-(j as f32 / sr) * 60.0).exp() * 0.7;
        }
    }
    normalize_and_scale(&mut out, vol);
    out
}

pub(super) fn synth_tool_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut lp, snap_lim) = (
        IirFilter::highpass(400.0, sr),
        IirFilter::lowpass(4500.0, sr),
        (sr * 0.015) as usize,
    );
    render(sr, dur, |i, t, _| {
        let crack = (t * (1400.0 - t * 3000.0).max(100.0) * TAU).sin() * 0.4;
        let snap = (rng.next_f32() * 2.0 - 1.0) * if i < snap_lim { 0.9 } else { 0.3 };
        hp.process(lp.process(crack + snap)) * (-18.0 * t).exp() * vol
    })
}
