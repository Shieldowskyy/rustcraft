//! Synteza odgłosów zwierząt hodowlanych i potworów (świnia, krowa, kurczak, zombie, szkielet, reaper).

use std::f32::consts::{PI, TAU};

use super::dsp::{BiquadFilter, FastRng, IirFilter, add_bp_grains, render};

pub(super) fn synth_pig_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2) = (
        BiquadFilter::bandpass(480.0, 2.5, sr),
        BiquadFilter::bandpass(1100.0, 2.0, sr),
    );
    let (mut lp, mut hp, mut phase) = (
        IirFilter::lowpass(750.0, sr),
        IirFilter::highpass(220.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.25), |_, t, nt| {
        phase += TAU * (135.0 - nt * 25.0) / sr;
        let rough = 0.45 + 0.55 * (t * 28.0 * TAU).sin();
        let src =
            (phase.sin() * 0.5 + (phase * 2.0).sin() * 0.3 + (phase * 3.0).sin() * 0.2) * rough;
        let formant = bp1.process(src) * 0.6 + bp2.process(src) * 0.4;
        let breath = hp.process(lp.process(rng.next_f32())) * rough * 0.25;
        let env = if t < 0.02 {
            t / 0.02
        } else {
            (-nt * 5.0).exp()
        };
        (formant * 1.8 + breath) * env * vol
    })
}

pub(super) fn synth_pig_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut hp, mut phase) = (
        BiquadFilter::bandpass(900.0, 2.8, sr),
        IirFilter::highpass(1600.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.24), |_, t, nt| {
        let base_f = if nt < 0.25 {
            780.0 + (nt / 0.25) * 140.0
        } else {
            920.0 - ((nt - 0.25) / 0.75) * 280.0
        };
        phase += TAU * (base_f + (t * 38.0 * TAU).sin() * 40.0).max(200.0) / sr;
        let tone =
            bp.process((phase.sin() * 2.0).clamp(-1.0, 1.0) * 0.65 + (phase * 2.0).sin() * 0.35);
        let env = if t < 0.008 {
            t / 0.008
        } else {
            (-nt * 9.5).exp()
        };
        (tone * 1.6 + hp.process(rng.next_f32()) * 0.18) * env * vol
    })
}

pub(super) fn synth_pig_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut lp, mut phase) = (
        BiquadFilter::bandpass(950.0, 2.4, sr),
        BiquadFilter::bandpass(450.0, 2.0, sr),
        IirFilter::lowpass(650.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.60), |_, t, nt| {
        let base_f = if t < 0.15 {
            920.0 + (t / 0.15) * 60.0
        } else {
            980.0 - ((t - 0.15) / (dur - 0.15)).powf(0.85) * 740.0
        };
        phase += TAU * (base_f + (t * 26.0 * TAU).sin() * 55.0).max(120.0) / sr;
        let tone = (phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let formant = bp1.process(tone) * (1.0 - nt * 0.5) + bp2.process(tone) * (nt * 0.7);
        let env = if t < 0.01 {
            t / 0.01
        } else {
            (1.0 - nt).powf(1.4)
        };
        (formant * 1.5 + lp.process(rng.next_f32()) * (nt * 0.35)) * env * vol
    })
}

pub(super) fn synth_cow_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut lp, mut phase) = (
        BiquadFilter::bandpass(380.0, 2.2, sr),
        BiquadFilter::bandpass(780.0, 2.0, sr),
        BiquadFilter::lowpass(320.0, 1.4, sr),
        0.0f32,
    );
    render(sr, dur.max(0.95), |_, t, nt| {
        let freq = if nt < 0.20 {
            118.0 + (nt / 0.20) * 14.0
        } else {
            132.0 - ((nt - 0.20) / 0.80) * 27.0
        };
        phase += TAU * (freq + (t * 4.2 * TAU).sin() * 3.5) / sr;
        let cords = phase.sin() * 0.55
            + (phase * 2.0).sin() * 0.28
            + (phase * 3.0).sin() * 0.12
            + (phase * 4.0).sin() * 0.05;
        let blend = ((t - 0.12) / 0.18).clamp(0.0, 1.0);
        let vocal = lp.process(cords) * (1.0 - blend)
            + (bp1.process(cords) * 0.65 + bp2.process(cords) * 0.35) * blend * 1.5;
        let env = if t < 0.15 {
            (t / 0.15).powf(0.8)
        } else if nt > 0.70 {
            ((1.0 - nt) / 0.30).powf(1.5)
        } else {
            1.0
        };
        (vocal * 1.6 + rng.next_f32() * 0.05 * blend) * env * vol
    })
}

pub(super) fn synth_cow_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut phase) = (
        BiquadFilter::bandpass(420.0, 2.0, sr),
        IirFilter::lowpass(600.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.32), |_, t, nt| {
        phase += TAU * (195.0 - nt * 90.0) / sr;
        let tone = (phase.sin() * 1.6).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let env = if t < 0.015 {
            t / 0.015
        } else {
            (-nt * 8.5).exp()
        };
        (bp.process(tone) * 1.7 + lp.process(rng.next_f32()) * 0.25) * env * vol
    })
}

pub(super) fn synth_cow_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut phase) = (
        BiquadFilter::bandpass(360.0, 1.8, sr),
        IirFilter::lowpass(180.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.85), |_, t, nt| {
        phase += TAU * ((145.0 - nt.powf(0.8) * 85.0).max(50.0) + (t * 3.5 * TAU).sin() * 4.0) / sr;
        let tone = phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3 + (phase * 3.0).sin() * 0.1;
        let env = if t < 0.02 {
            t / 0.02
        } else {
            (1.0 - nt).powf(1.6)
        };
        (bp.process(tone) * 1.8 + lp.process(rng.next_f32()) * 0.3 * (1.0 - nt)) * env * vol
    })
}

pub(super) fn synth_chicken_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.26)) as usize];
    for &(start_t, c_dur, f_start, f_end) in
        &[(0.0, 0.07, 740.0, 480.0), (0.11, 0.09, 660.0, 390.0)]
    {
        let (start_s, dur_s) = ((start_t * sr) as usize, (c_dur * sr) as usize);
        let (mut bp, mut phase) = (BiquadFilter::bandpass(620.0, 3.2, sr), 0.0f32);
        for s in 0..dur_s {
            if start_s + s >= out.len() {
                break;
            }
            let (t, cur_t) = (s as f32 / dur_s as f32, s as f32 / sr);
            phase += TAU * (f_start - t * (f_start - f_end)) / sr;
            let click = if cur_t < 0.006 {
                rng.next_f32() * 0.4 * (1.0 - cur_t / 0.006)
            } else {
                0.0
            };
            let vocal = bp.process(phase.sin() * 0.8 + (phase * 2.0).sin() * 0.2);
            out[start_s + s] += (vocal * 1.5 + click) * (1.0 - t).powf(1.8) * vol;
        }
    }
    out
}

pub(super) fn synth_chicken_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut hp, mut phase) = (
        BiquadFilter::bandpass(1050.0, 2.5, sr),
        IirFilter::highpass(1800.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.20), |_, t, nt| {
        phase += TAU * (1200.0 - nt * 580.0 + (t * 48.0 * TAU).sin() * 50.0).max(250.0) / sr;
        let vocal =
            bp.process((phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.7 + (phase * 2.0).sin() * 0.3);
        let env = if t < 0.008 {
            t / 0.008
        } else {
            (-nt * 11.0).exp()
        };
        (vocal * 1.6 + hp.process(rng.next_f32()) * 0.22) * env * vol
    })
}

pub(super) fn synth_chicken_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.52)) as usize];
    let (cry_len, mut bp, mut phase) = (
        (sr * 0.32) as usize,
        BiquadFilter::bandpass(980.0, 2.2, sr),
        0.0f32,
    );
    for i in 0..cry_len.min(out.len()) {
        let (t, cur_t) = (i as f32 / cry_len as f32, i as f32 / sr);
        phase += TAU * (1300.0 - t.powf(0.8) * 920.0).max(200.0) / sr;
        let env = if cur_t < 0.01 {
            cur_t / 0.01
        } else {
            (1.0 - t).powf(1.8)
        };
        out[i] += bp.process((phase.sin() * 1.8).clamp(-1.0, 1.0) * 0.75) * env * vol * 1.4;
    }
    let f_len = (sr * 0.045) as usize;
    for &(ft, amp) in &[(0.06, 0.55), (0.16, 0.45), (0.26, 0.35), (0.36, 0.20)] {
        let (start, mut lp, mut hp) = (
            (ft * sr) as usize,
            IirFilter::lowpass(800.0, sr),
            IirFilter::highpass(220.0, sr),
        );
        for i in 0..f_len {
            if start + i >= out.len() {
                break;
            }
            let t = i as f32 / f_len as f32;
            out[start + i] +=
                lp.process(hp.process(rng.next_f32())) * (1.0 - t).powi(2) * amp * vol;
        }
    }
    out
}

pub(super) fn synth_zombie_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(380.0, 3.0, sr),
        BiquadFilter::bandpass(820.0, 2.5, sr),
        0.0f32,
    );
    render(sr, dur.max(0.6), |_, t, nt| {
        phase += TAU * (115.0 - nt * 25.0 + (t * 6.0 * PI).sin() * 12.0) / sr;
        let buzz = ((phase.sin() * 2.2).clamp(-1.0, 1.0) + (phase * 2.0).sin() * 0.4) * 0.7;
        let sound = bp1.process(buzz + rng.next_f32() * 0.25) * 1.5 + bp2.process(buzz) * 0.8;
        let env = if nt < 0.15 {
            nt / 0.15
        } else if nt > 0.75 {
            (1.0 - nt) / 0.25
        } else {
            1.0
        };
        sound * env * vol
    })
}

pub(super) fn synth_zombie_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(450.0, 2.2, sr), 0.0f32);
    render(sr, dur.max(0.28), |_, _, nt| {
        phase += TAU * (170.0 - nt * 60.0) / sr;
        bp.process((phase.sin() * 2.5).clamp(-1.0, 1.0) * 0.8 + rng.next_f32() * 0.3)
            * (-nt * 6.0).exp()
            * vol
            * 1.4
    })
}

pub(super) fn synth_zombie_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(320.0, 2.0, sr), 0.0f32);
    render(sr, dur.max(0.7), |_, _, nt| {
        phase += TAU * (145.0 - nt.powf(0.8) * 85.0).max(45.0) / sr;
        bp.process((phase.sin() * 2.0).clamp(-1.0, 1.0) * 0.7 + rng.next_f32() * 0.2)
            * (1.0 - nt).powf(1.4)
            * vol
            * 1.3
    })
}

pub(super) fn synth_skeleton_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.35)) as usize];
    let clicks = [
        (0.03, 2400.0, 1.0),
        (0.09, 2400.0, 1.0),
        (0.16, 2400.0, 1.0),
        (0.24, 2400.0, 1.0),
    ];
    add_bp_grains(&mut out, rng, sr, 0.022, 4.0, vol * 1.5, true, &clicks);
    out
}

pub(super) fn synth_skeleton_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2) = (
        BiquadFilter::bandpass(1900.0, 3.5, sr),
        BiquadFilter::bandpass(3400.0, 4.0, sr),
    );
    render(sr, dur.max(0.20), |i, _, nt| {
        let imp = if i < 4 { 1.2 } else { rng.next_f32() * 0.35 };
        (bp1.process(imp) * 0.7 + bp2.process(imp) * 0.6) * (-nt * 14.0).exp() * vol * 1.8
    })
}

pub(super) fn synth_skeleton_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut out = vec![0.0f32; (sr * dur.max(0.6)) as usize];
    let drops = [
        (0.02, 1600.0, 1.0),
        (0.08, 1920.0, 0.88),
        (0.15, 2240.0, 0.76),
        (0.22, 2560.0, 0.64),
        (0.31, 2880.0, 0.52),
        (0.42, 3200.0, 0.40),
    ];
    add_bp_grains(&mut out, rng, sr, 0.035, 3.2, vol * 1.4, true, &drops);
    out
}

pub(super) fn synth_reaper_hiss(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut lp) = (
        IirFilter::highpass(1600.0, sr),
        IirFilter::lowpass(6800.0, sr),
    );
    render(sr, dur, |_, _, nt| {
        let swell = nt.powf(0.5) * (0.65 + 0.35 * ((nt * 32.0).sin() * 0.5 + 0.5));
        lp.process(hp.process(rng.next_f32() * 2.0 - 1.0)) * swell * vol * 1.2
    })
}

pub(super) fn synth_reaper_impulse(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    freq: f32,
    q: f32,
    imp_len: usize,
    imp_v: f32,
    n_amp: f32,
    exp_dec: bool,
    vol: f32,
) -> Vec<f32> {
    let mut bp = BiquadFilter::bandpass(freq, q, sr);
    render(sr, dur, |i, _, nt| {
        let imp = if i < imp_len {
            imp_v
        } else {
            rng.next_f32() * n_amp
        };
        let env = if exp_dec {
            (-nt * 12.0).exp()
        } else {
            (1.0 - nt).powf(1.8)
        };
        bp.process(imp) * env * vol
    })
}
