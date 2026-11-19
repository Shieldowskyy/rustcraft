//! Synteza kroków, niszczenia i kopania bloków, kliknięć UI, skrzyń, drzwi oraz eksplozji.

use std::f32::consts::{PI, TAU};

use super::dsp::{BiquadFilter, FastRng, IirFilter, add_bp_grains, render};

pub(super) fn synth_noise_burst(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    hp_cut: f32,
    lp_cut: f32,
    decay: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut hp, mut lp) = (
        IirFilter::highpass(hp_cut, sr),
        IirFilter::lowpass(lp_cut, sr),
    );
    render(sr, dur, |_, _, nt| {
        lp.process(hp.process(rng.next_f32())) * (1.0 - nt).powf(decay) * vol
    })
}

pub(super) fn synth_impact(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    tone_freq: f32,
    noise_lp: f32,
    noise_dur: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut lp, mut phase) = (IirFilter::lowpass(noise_lp, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        phase += TAU * tone_freq / sr;
        let sine_val = phase.sin() * (-t * 30.0).exp();
        let noise_env = if t < noise_dur {
            (1.0 - (t / noise_dur)).max(0.0)
        } else {
            0.0
        };
        (sine_val + lp.process(rng.next_f32()) * noise_env * 0.7) * vol
    })
}

pub(super) fn synth_stone_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.32)) as usize];
    let (mut bp_crack, mut bp_rumble, mut phase) = (
        BiquadFilter::bandpass(2400.0, 2.2, sr),
        BiquadFilter::bandpass(140.0, 1.5, sr),
        0.0f32,
    );
    for i in 0..((sr * 0.08) as usize).min(out.len()) {
        let t = i as f32 / sr;
        phase += TAU * (180.0 - t * 1400.0).max(50.0) / sr;
        let raw = rng.next_f32();
        out[i] += (phase.sin() * (-t * 45.0).exp() * 0.85
            + bp_crack.process(raw) * (-t * 60.0).exp() * 1.4
            + bp_rumble.process(raw) * (-t * 30.0).exp() * 0.6)
            * vol;
    }
    let grains = [
        (0.035, 2800.0, 0.85),
        (0.070, 2100.0, 0.70),
        (0.115, 3200.0, 0.55),
        (0.165, 1800.0, 0.42),
        (0.220, 2500.0, 0.30),
        (0.280, 1500.0, 0.18),
    ];
    add_bp_grains(&mut out, rng, sr, 0.035, 3.0, vol, false, &grains);
    out
}

pub(super) fn synth_wood_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.30)) as usize];
    let (mut bp_splinter, mut p1, mut p2) =
        (BiquadFilter::bandpass(1650.0, 1.8, sr), 0.0f32, 0.0f32);
    for i in 0..((sr * 0.10) as usize).min(out.len()) {
        let t = i as f32 / sr;
        let freq = (210.0 - t * 800.0).max(90.0);
        p1 += TAU * freq / sr;
        p2 += TAU * (freq * 2.1) / sr;
        out[i] += ((p1.sin() * 0.6 + p2.sin() * 0.4) * (-t * 35.0).exp()
            + bp_splinter.process(rng.next_f32()) * (-t * 40.0).exp() * 1.2)
            * vol;
    }
    let grains = [
        (0.03, 1900.0, 0.75),
        (0.065, 1400.0, 0.60),
        (0.11, 2200.0, 0.45),
        (0.16, 1700.0, 0.32),
        (0.22, 1200.0, 0.20),
    ];
    add_bp_grains(&mut out, rng, sr, 0.03, 2.5, vol, false, &grains);
    out
}

pub(super) fn synth_dirt_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.28)) as usize];
    let (mut lp, mut hp, mut phase) = (
        IirFilter::lowpass(850.0, sr),
        IirFilter::highpass(180.0, sr),
        0.0f32,
    );
    for i in 0..((sr * 0.09) as usize).min(out.len()) {
        let t = i as f32 / sr;
        phase += TAU * 85.0 / sr;
        out[i] += (phase.sin() * (-t * 40.0).exp() * 0.9
            + lp.process(hp.process(rng.next_f32())) * (-t * 35.0).exp() * 1.1)
            * vol;
    }
    let c_len = (sr * 0.04) as usize;
    for &(time, amp) in &[
        (0.035, 0.65),
        (0.075, 0.50),
        (0.12, 0.38),
        (0.17, 0.26),
        (0.22, 0.15),
    ] {
        let start = (time * sr) as usize;
        let (mut c_lp, mut c_hp) = (
            IirFilter::lowpass(650.0, sr),
            IirFilter::highpass(220.0, sr),
        );
        for i in 0..c_len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / c_len as f32;
            out[start + i] +=
                c_lp.process(c_hp.process(rng.next_f32())) * (1.0 - t).powi(2) * amp * vol;
        }
    }
    out
}

pub(super) fn synth_grass_break(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp_l, mut lp_l, mut lp_e) = (
        IirFilter::highpass(1400.0, sr),
        IirFilter::lowpass(4200.0, sr),
        IirFilter::lowpass(400.0, sr),
    );
    render(sr, dur.max(0.26), |_, _, nt| {
        let raw = rng.next_f32();
        (lp_l.process(hp_l.process(raw)) * (1.0 - nt).powf(1.8) * 0.85 * 1.1
            + lp_e.process(raw) * (-nt * 12.0).exp() * 0.45)
            * vol
    })
}

pub(super) fn synth_glass_shatter(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.38)) as usize];
    let (mut hp, mut lp) = (
        IirFilter::highpass(2200.0, sr),
        IirFilter::lowpass(9500.0, sr),
    );
    let snap_len = (sr * 0.015) as usize;
    for i in 0..snap_len.min(out.len()) {
        out[i] += lp.process(hp.process(rng.next_f32()))
            * (1.0 - i as f32 / snap_len as f32).powi(3)
            * 1.8
            * vol;
    }
    let (mut p1, mut p2, mut p3) = (0.0f32, 0.0f32, 0.0f32);
    for i in 0..((sr * 0.32) as usize).min(out.len()) {
        let t = i as f32 / sr;
        p1 += TAU * 3150.0 / sr;
        p2 += TAU * 5200.0 / sr;
        p3 += TAU * 7800.0 / sr;
        out[i] +=
            (p1.sin() * 0.4 + p2.sin() * 0.35 + p3.sin() * 0.25) * (-t * 12.0).exp() * 0.7 * vol;
    }
    let tinkles = [
        (0.03, 4200.0, 0.7),
        (0.07, 6800.0, 0.6),
        (0.12, 3900.0, 0.5),
        (0.17, 8100.0, 0.4),
        (0.23, 5400.0, 0.3),
        (0.29, 7200.0, 0.22),
        (0.34, 4800.0, 0.15),
    ];
    add_bp_grains(&mut out, rng, sr, 0.02, 4.0, vol, false, &tinkles);
    out
}

pub(super) fn synth_bp_dig(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    tone_f: f32,
    tone_dec: f32,
    tone_amp: f32,
    bp_f: f32,
    bp_q: f32,
    bp_dec: f32,
    bp_amp: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(bp_f, bp_q, sr), 0.0f32);
    render(sr, dur, |_, t, _| {
        phase += TAU * tone_f / sr;
        (phase.sin() * (-t * tone_dec).exp() * tone_amp
            + bp.process(rng.next_f32()) * (-t * bp_dec).exp() * bp_amp)
            * vol
    })
}

pub(super) fn synth_dirt_dig(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut hp, mut phase) = (
        IirFilter::lowpass(900.0, sr),
        IirFilter::highpass(180.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.07), |_, t, _| {
        phase += TAU * 95.0 / sr;
        let env = (-t * 50.0).exp();
        (phase.sin() * env * 0.75 + lp.process(hp.process(rng.next_f32())) * env) * vol
    })
}

pub(super) fn synth_dig_scrape(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    hp_f: f32,
    lp_f: f32,
    dec: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut lp, mut hp) = (IirFilter::lowpass(lp_f, sr), IirFilter::highpass(hp_f, sr));
    render(sr, dur, |_, t, _| {
        lp.process(hp.process(rng.next_f32())) * (-t * dec).exp() * 1.1 * vol
    })
}

pub(super) fn synth_water_splash(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut lp, mut phase) = (
        IirFilter::highpass(500.0, sr),
        IirFilter::lowpass(2600.0, sr),
        0.0f32,
    );
    render(sr, dur, |_, t, nt| {
        phase += TAU * (500.0 - t * 1800.0).max(120.0) / sr;
        let plop = phase.sin() * (-t * 35.0).exp() * 0.7;
        let spray =
            lp.process(hp.process(rng.next_f32())) * (nt * (1.0 - nt) * 4.0).powf(1.5) * 0.55;
        (plop + spray) * vol
    })
}

pub(super) fn synth_sweep(sr: f32, dur: f32, start_f: f32, end_f: f32, vol: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    render(sr, dur, |_, _, nt| {
        phase += TAU * (start_f + (end_f - start_f) * nt) / sr;
        phase.sin() * (1.0 - nt).powf(1.1) * vol
    })
}

pub(super) fn synth_click(sr: f32, dur: f32, freq: f32, vol: f32) -> Vec<f32> {
    let (mut phase, mut click_rng) = (0.0f32, 0x2A3B4C5Du32);
    render(sr, dur, |_, t, nt| {
        phase += TAU * (freq * (1.18 - 0.35 * nt)) / sr;
        click_rng = click_rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((click_rng >> 16) as f32 / 32768.0) - 1.0;
        ((phase.sin() * 0.65 + noise * (-t * 500.0).exp() * 0.75) * (-t * 85.0).exp() * vol)
            .clamp(-1.0, 1.0)
    })
}

pub(super) fn synth_hurt_grunt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut phase) = (IirFilter::lowpass(600.0, sr), 0.0f32);
    render(sr, dur, |_, _, nt| {
        phase += TAU * (160.0 - nt * 60.0) / sr;
        (phase.sin() * 0.6 + lp.process(rng.next_f32()) * 0.4) * (-nt * 12.0).exp() * vol
    })
}

pub(super) fn synth_chest_creak(sr: f32, dur: f32, open: bool, vol: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    render(sr, dur, |_, _, nt| {
        phase += TAU * (if open { 300.0 + nt * 450.0 } else { 750.0 - nt * 450.0 }) / sr;
        let tri = (phase % TAU / PI - 1.0).abs() * 2.0 - 1.0;
        tri * (nt * (1.0 - nt) * 4.0).min(1.0) * vol
    })
}

pub(super) fn synth_door_swing(sr: f32, dur: f32, open: bool, vol: f32) -> Vec<f32> {
    let mut phase = 0.0f32;
    render(sr, dur, |_, _, nt| {
        phase += TAU * (if open { 180.0 + nt * 140.0 } else { 320.0 - nt * 140.0 }) / sr;
        phase.sin() * (-nt * 4.0).exp() * vol
    })
}

pub(super) fn synth_explosion(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut phase) = (IirFilter::lowpass(400.0, sr), 0.0f32);
    render(sr, dur, |_, t, nt| {
        let crack = if t < 0.08 {
            rng.next_f32() * (1.0 - t / 0.08).max(0.0) * 0.9
        } else {
            0.0
        };
        phase += TAU * (55.0 - nt * 25.0) / sr;
        let rumble = (phase.sin() * 0.4 + lp.process(rng.next_f32()) * 0.6) * (-t * 2.5).exp();
        (crack + rumble) * vol
    })
}

pub(super) fn synth_wood_step(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp_c, mut bp_b) = (
        BiquadFilter::bandpass(320.0, 3.2, sr),
        BiquadFilter::bandpass(680.0, 3.5, sr),
    );
    let (mut hp_t, mut lp_t, mut phase) = (
        IirFilter::highpass(1200.0, sr),
        IirFilter::lowpass(3800.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.12), |i, t, nt| {
        phase += TAU * (180.0 - nt * 95.0).max(70.0) / sr;
        let thump = (phase.sin() * 0.7 + (phase * 2.1).sin() * 0.3) * (-t * 38.0).exp();
        let raw_imp = if i < 3 { 1.5 } else { rng.next_f32() * 0.3 };
        let cavity = bp_c.process(raw_imp) * (-t * 32.0).exp();
        let board = bp_b.process(raw_imp) * (-t * 42.0).exp();
        let sole = lp_t.process(hp_t.process(rng.next_f32())) * (-t * 65.0).exp();
        (thump * 0.65 + cavity * 0.75 + board * 0.45 + sole * 0.40) * vol
    })
}

pub(super) fn synth_wood_creak(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut bp_h) = (
        BiquadFilter::bandpass(380.0, 4.2, sr),
        BiquadFilter::bandpass(940.0, 3.6, sr),
        BiquadFilter::bandpass(1750.0, 4.0, sr),
    );
    let (mut car_p, mut mod_p) = (0.0f32, 0.0f32);
    render(sr, dur.max(0.20), |_, t, nt| {
        let base_pitch = 320.0 - (nt * TAU).sin() * 35.0 - nt * 45.0;
        let friction = if (t * 55.0 * TAU).sin() > 0.85 {
            rng.next_f32() * 0.35
        } else {
            0.0
        };
        let mod_idx = (1.8 * (1.0 - nt).powf(0.8) + friction).max(0.1);
        mod_p += TAU * (base_pitch * 2.82) / sr;
        car_p += TAU * base_pitch / sr;
        let raw_fm = (car_p + mod_p.sin() * mod_idx).sin();
        let body = bp1.process(raw_fm) * 0.70
            + bp2.process(raw_fm) * 0.45
            + bp_h.process(raw_fm + friction) * 0.25;
        let env = (nt * PI).sin().powf(0.7) * (1.0 - nt * 0.3);
        (body / (1.0 + body.abs() * 0.4)) * env * vol * 1.3
    })
}
