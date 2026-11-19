//! Synteza odgłosów owcy, osadnika, żelaznego golema, ryb, delfina, wędkowania i pająka.

use std::f32::consts::{PI, TAU};

use super::blocks_ui::{synth_sweep, synth_water_splash};
use super::dsp::{BiquadFilter, FastRng, IirFilter, render};

pub(super) fn synth_sheep_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut lp, mut phase) = (
        BiquadFilter::bandpass(540.0, 2.2, sr),
        BiquadFilter::bandpass(1180.0, 2.0, sr),
        BiquadFilter::lowpass(420.0, 1.4, sr),
        0.0f32,
    );
    render(sr, dur.max(0.65), |_, t, nt| {
        let base_f = if nt < 0.25 {
            205.0 + (nt / 0.25) * 30.0
        } else {
            235.0 - ((nt - 0.25) / 0.75) * 50.0
        };
        phase += TAU * (base_f + (t * 6.4 * TAU).sin() * 7.5) / sr;
        let cords = phase.sin() * 0.55
            + (phase * 2.0).sin() * 0.25
            + (phase * 3.0).sin() * 0.12
            + (phase * 4.0).sin() * 0.06;
        let blend = ((t - 0.08) / 0.15).clamp(0.0, 1.0);
        let vocal = lp.process(cords) * (1.0 - blend)
            + (bp1.process(cords) * 0.65 + bp2.process(cords) * 0.35) * blend * 1.4;
        let env = if t < 0.08 {
            (t / 0.08).powf(0.8)
        } else if nt > 0.65 {
            ((1.0 - nt) / 0.35).powf(1.6)
        } else {
            1.0
        };
        (vocal * 1.5 + rng.next_f32() * 0.04 * blend) * env * vol
    })
}

pub(super) fn synth_sheep_hurt(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut lp, mut phase) = (
        BiquadFilter::bandpass(680.0, 2.0, sr),
        IirFilter::lowpass(800.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.25), |_, t, nt| {
        phase += TAU * (290.0 - nt * 80.0 + (t * 8.5 * TAU).sin() * 10.0) / sr;
        let tone = (phase.sin() * 1.5).clamp(-1.0, 1.0) * 0.6 + (phase * 2.0).sin() * 0.4;
        let env = if t < 0.015 {
            t / 0.015
        } else {
            (-nt * 8.0).exp()
        };
        (bp.process(tone) * 1.6 + lp.process(rng.next_f32()) * 0.2) * env * vol
    })
}

pub(super) fn synth_sheep_death(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(520.0, 1.8, sr), 0.0f32);
    render(sr, dur.max(0.65), |_, t, nt| {
        phase +=
            TAU * ((220.0 - nt.powf(0.7) * 110.0).max(80.0) + (t * 5.0 * TAU).sin() * 6.0) / sr;
        let env = if t < 0.04 {
            t / 0.04
        } else {
            (1.0 - nt).max(0.0).powf(1.4)
        };
        bp.process(phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3) * env * vol * 1.5
    })
}

pub(super) fn synth_shears(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut hp, mut bp1, mut bp2) = (
        IirFilter::highpass(2200.0, sr),
        BiquadFilter::bandpass(3400.0, 6.0, sr),
        BiquadFilter::bandpass(4900.0, 5.0, sr),
    );
    render(sr, dur.max(0.12), |_, t, nt| {
        let noise = rng.next_f32() * 2.0 - 1.0;
        let env_f = if t < 0.02 {
            t / 0.02
        } else {
            (-nt * 18.0).exp()
        };
        (hp.process(noise) * 0.5 * env_f
            + (bp1.process(noise) + bp2.process(noise)) * 0.8 * (-nt * 12.0).exp())
            * vol
            * 1.7
    })
}

pub(super) fn synth_fish_flop(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = IirFilter::lowpass(1800.0, sr);
    render(sr, dur.max(0.12), |_, _, nt| {
        let noise = rng.next_f32() * 2.0 - 1.0;
        (lp.process(noise) * 0.6 + noise * (nt * 36.0 * PI).sin().abs() * 0.4)
            * (1.0 - nt).max(0.0).powf(1.5)
            * vol
            * 1.4
    })
}

pub(super) fn synth_dolphin_idle(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    render(sr, dur.max(0.3), |i, t, nt| {
        let freq = 1800.0 + (nt * 6.0 * PI).sin() * 500.0 + nt * 400.0;
        let click = if (i % (sr as usize / 55)) < 4 {
            0.45
        } else {
            0.0
        };
        ((t * freq * TAU).sin() * 0.75 + click * 0.25) * (1.0 - nt).max(0.0).powf(1.2) * vol * 1.3
    })
}

pub(super) fn synth_dolphin_whistle(
    sr: f32,
    dur: f32,
    start_f: f32,
    df: f32,
    exp: f32,
    vol: f32,
) -> Vec<f32> {
    render(sr, dur, |_, t, nt| {
        (t * (start_f + nt * df) * TAU).sin() * (1.0 - nt).max(0.0).powf(exp) * vol
    })
}

pub(super) fn synth_fishing_reel(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    render(sr, dur.max(0.15), |i, _, nt| {
        let click = if (i % 280) < 35 {
            (rng.next_f32() * 2.0 - 1.0) * 0.8
        } else {
            0.0
        };
        click * (1.0 - nt).powf(1.2) * vol
    })
}

pub(super) fn synth_fish_catch(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut splash = synth_water_splash(rng, sr, dur, vol * 0.8);
    for (s, p) in splash
        .iter_mut()
        .zip(synth_sweep(sr, 0.12, 380.0, 880.0, vol * 0.9))
    {
        *s = (*s + p).clamp(-1.0, 1.0);
    }
    splash
}

pub(super) fn synth_villager_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(420.0, 2.5, sr),
        BiquadFilter::bandpass(1120.0, 3.0, sr),
        0.0f32,
    );
    render(sr, dur.max(0.40), |_, _, nt| {
        phase += TAU * (152.0 - nt * 14.0 + (nt * 8.0 * PI).sin() * 2.0) / sr;
        let raw = phase.sin() * 0.65
            + (phase * 2.0).sin() * 0.35
            + (phase * 3.0).sin() * 0.20
            + (phase * 4.0).sin() * 0.12;
        let env = if nt < 0.12 {
            (nt / 0.12).powf(0.8)
        } else if nt > 0.70 {
            ((1.0 - nt) / 0.30).powf(1.4)
        } else {
            1.0
        };
        ((bp1.process(raw) * 0.70 + bp2.process(raw) * 0.30) * env * 1.5
            + rng.next_f32() * 0.03 * env)
            * vol
    })
}

pub(super) fn synth_villager_vocal(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    bp_f: f32,
    f0: f32,
    df: f32,
    h1: f32,
    h2: f32,
    n_amp: f32,
    exp: f32,
    vol: f32,
) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(bp_f, 2.0, sr), 0.0f32);
    render(sr, dur, |_, _, nt| {
        phase += TAU * (f0 + nt * df) / sr;
        let raw =
            phase.sin() * h1 + (phase * 2.0).sin() * h2 + (rng.next_f32() * n_amp - n_amp * 0.5);
        bp.process(raw) * (1.0 - nt).powf(exp) * vol
    })
}

pub(super) fn synth_villager_trade(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp1, mut bp2, mut phase) = (
        BiquadFilter::bandpass(520.0, 2.2, sr),
        BiquadFilter::bandpass(1280.0, 2.5, sr),
        0.0f32,
    );
    render(sr, dur.max(0.35), |_, t, nt| {
        phase += TAU * (140.0 + nt.powf(1.4) * 75.0) / sr;
        let raw = phase.sin() * 0.65 + (phase * 2.0).sin() * 0.35 + (phase * 3.0).sin() * 0.18;
        let env = if nt < 0.10 {
            nt / 0.10
        } else if nt > 0.75 {
            (1.0 - nt) / 0.25
        } else {
            1.0
        };
        let chime = (TAU * (660.0 + nt * 440.0) * t).sin() * 0.12 * env;
        ((bp1.process(raw) * 0.65 + bp2.process(raw) * 0.35) * env * 1.4 + chime) * vol
    })
}

pub(super) fn synth_villager_no(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut bp, mut phase) = (BiquadFilter::bandpass(460.0, 2.0, sr), 0.0f32);
    render(sr, dur.max(0.32), |_, _, nt| {
        let (freq, sub_t) = if nt < 0.48 {
            (160.0, nt / 0.48)
        } else {
            (125.0, (nt - 0.48) / 0.52)
        };
        phase += TAU * freq / sr;
        bp.process(phase.sin() * 0.7 + (phase * 2.0).sin() * 0.35)
            * (sub_t * (1.0 - sub_t) * 4.0).clamp(0.0, 1.0)
            * vol
            * 1.5
    })
}

pub(super) fn synth_iron_golem_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = BiquadFilter::lowpass(320.0, 1.2, sr);
    render(sr, dur.max(0.35), |_, t, nt| {
        let metal = (TAU * 180.0 * t).sin() * 0.4 + (TAU * 340.0 * t).sin() * 0.25;
        lp.process((rng.next_f32() - 0.5) * 0.5 + metal) * (1.0 - nt).powf(2.0) * vol * 1.6
    })
}

pub(super) fn synth_iron_golem_hurt(sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    render(sr, dur.max(0.30), |_, t, nt| {
        let hit = (TAU * 420.0 * t).sin() * 0.5
            + (TAU * 840.0 * t).sin() * 0.35
            + (TAU * 1260.0 * t).sin() * 0.15;
        hit * (-nt * 9.0).exp() * vol * 1.5
    })
}

pub(super) fn synth_iron_golem_death(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = BiquadFilter::lowpass(280.0, 1.5, sr);
    render(sr, dur.max(0.75), |_, t, nt| {
        let rumble = (TAU * (90.0 + (1.0 - nt) * 110.0) * t).sin() * 0.6;
        lp.process(rumble + (rng.next_f32() - 0.5) * 0.5) * (1.0 - nt).powf(1.5) * vol * 1.8
    })
}

pub(super) fn synth_iron_golem_attack(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let mut lp = BiquadFilter::lowpass(550.0, 1.0, sr);
    render(sr, dur.max(0.25), |_, t, nt| {
        let env = if nt < 0.25 {
            nt / 0.25
        } else {
            (1.0 - nt) / 0.75
        };
        (lp.process((rng.next_f32() - 0.5) * 0.8)
            + (TAU * 110.0 * t).sin() * 0.5 * (-nt * 8.0).exp())
            * env
            * vol
            * 1.6
    })
}

pub(super) fn synth_spider_idle(rng: &mut FastRng, sr: f32, dur: f32, vol: f32) -> Vec<f32> {
    let (mut lp, mut hp) = (
        IirFilter::lowpass(2800.0, sr),
        IirFilter::highpass(900.0, sr),
    );
    render(sr, dur, |_, t, _| {
        let rattle = ((t * 85.0).sin() * 0.5 + 0.5).powi(2);
        hp.process(lp.process((rng.next_f32() * 2.0 - 1.0) * rattle))
            * (1.0 - t / dur).max(0.0)
            * vol
    })
}

pub(super) fn synth_spider_vocal(
    rng: &mut FastRng,
    sr: f32,
    dur: f32,
    is_hp: bool,
    cutoff: f32,
    f0: f32,
    df: f32,
    t_amp: f32,
    n_amp: f32,
    exp: f32,
    vol: f32,
) -> Vec<f32> {
    let mut filt = if is_hp {
        IirFilter::highpass(cutoff, sr)
    } else {
        IirFilter::lowpass(cutoff, sr)
    };
    render(sr, dur, |_, t, _| {
        let rem = 1.0 - t / dur;
        let tone = (t * (f0 + rem * df) * TAU).sin() * t_amp;
        filt.process(tone + (rng.next_f32() * 2.0 - 1.0) * n_amp) * rem.powf(exp) * vol
    })
}
