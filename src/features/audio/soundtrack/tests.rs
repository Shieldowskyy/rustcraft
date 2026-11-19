//! Testy jednostkowe kategorii muzyki, manifestu `soundtracks/`, dekodowania Ogg Opus i powiadomień `MusicToast`.

use std::fs;

use super::*;
use crate::features::audio::sound::SoundSample;
use crate::platform::capabilities;

#[test]
fn test_soundtrack_category_parsing_and_properties() {
    assert_eq!(SoundtrackCategory::parse("creative"), SoundtrackCategory::Creative);
    assert_eq!(SoundtrackCategory::parse("CREATIVE"), SoundtrackCategory::Creative);
    assert_eq!(SoundtrackCategory::parse("survival"), SoundtrackCategory::Survival);
    assert_eq!(SoundtrackCategory::parse("menu"), SoundtrackCategory::Menu);
    assert_eq!(SoundtrackCategory::parse("main_menu"), SoundtrackCategory::Menu);
    assert_eq!(SoundtrackCategory::parse("uncategorized"), SoundtrackCategory::Uncategorized);
    assert_eq!(SoundtrackCategory::parse("anything_else"), SoundtrackCategory::Uncategorized);

    assert_eq!(SoundtrackCategory::Creative.name(), "creative");
    assert_eq!(SoundtrackCategory::Survival.name(), "survival");
    assert_eq!(SoundtrackCategory::Menu.name(), "menu");
    assert_eq!(SoundtrackCategory::Uncategorized.name(), "uncategorized");
}

#[test]
fn test_soundtrack_manifest_parsing() {
    let mgr = SoundtrackManager::new("soundtracks");
    assert!(!mgr.tracks.is_empty(), "Should have discovered soundtrack tracks");

    let creative_track = mgr.tracks.iter().find(|t| t.id == "creative_calm");
    assert!(creative_track.is_some(), "creative_calm track must exist");
    let ct = creative_track.unwrap();
    assert!(ct.matches_category(SoundtrackCategory::Creative));
    assert!(!ct.matches_category(SoundtrackCategory::Survival));

    let wanderer_track = mgr.tracks.iter().find(|t| t.id == "wanderer");
    assert!(wanderer_track.is_some(), "wanderer track must exist");
    let wt = wanderer_track.unwrap();
    assert!(wt.matches_category(SoundtrackCategory::Creative));
    assert!(wt.matches_category(SoundtrackCategory::Survival));
    assert!(wt.matches_category(SoundtrackCategory::Menu));
}

#[test]
fn test_embedded_soundtracks_load_without_filesystem_assets() {
    if !capabilities().is_desktop {
        return;
    }
    assert!(!crate::embedded_assets::SOUNDTRACKS.is_empty());
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!(
        "rustcraft-embedded-soundtracks-{}-{unique}",
        std::process::id()
    ));
    let mut manager = SoundtrackManager::new(&temp_dir);
    let embedded_track = manager
        .tracks
        .iter_mut()
        .find(|track| track.embedded_data.is_some())
        .expect("embedded track should be registered");

    let sample = embedded_track
        .load_sample()
        .expect("embedded audio should decode");
    assert!(!sample.data.is_empty());
    drop(manager);
    fs::remove_dir_all(temp_dir).expect("temporary soundtrack directory should be removed");
}

#[test]
fn test_embedded_soundtrack_preloads_for_startup_menu() {
    if !capabilities().is_desktop {
        return;
    }
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!(
        "rustcraft-embedded-intro-{}-{unique}",
        std::process::id()
    ));
    let mut manager = SoundtrackManager::new(&temp_dir);

    manager.preload_intro(SoundtrackCategory::Menu);
    let intro_idx = manager
        .pending_intro
        .expect("embedded music should preload for menu");
    assert!(manager.tracks[intro_idx].sample.is_some());

    drop(manager);
    fs::remove_dir_all(temp_dir).expect("temporary soundtrack directory should be removed");
}

#[test]
fn test_ogg_opus_decoding_roundtrip() {
    let mut encoder =
        opus_pure::OpusEncoder::new(48000, 2, opus_pure::Application::Audio).unwrap();
    let head = opus_pure::OpusHead::for_encoder(&encoder, 48000);
    let mut writer = opus_pure::OggOpusWriter::new(Vec::new(), head).unwrap();

    let frame_size = 960;
    let mut pcm_in = Vec::with_capacity(frame_size * 2);
    for i in 0..frame_size {
        let t = i as f32 / 48000.0;
        let val = (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5;
        pcm_in.push(val);
        pcm_in.push(val * 0.8);
    }

    let mut packet_buf = vec![0u8; 1000];
    let pkt_len = encoder.encode(&pcm_in, frame_size, &mut packet_buf).unwrap();
    writer.write_packet(&packet_buf[..pkt_len]).unwrap();
    let ogg_bytes = writer.finish().unwrap();

    let sample =
        SoundSample::from_file_bytes(&ogg_bytes).expect("Failed to decode Ogg Opus stream");
    assert_eq!(sample.channels, 2, "Decoded sample must be stereo");
    assert_eq!(sample.sample_rate, 48000);
    assert!(!sample.data.is_empty(), "Must contain decoded audio frames");
}

#[test]
fn test_music_toast_lifecycle_and_rendering() {
    let mut toast = MusicToast::default();
    assert!(!toast.visible);

    toast.show("Calm Horizon", "RustCraft", SoundtrackCategory::Creative);
    assert!(toast.visible);
    assert_eq!(toast.title, "Calm Horizon");
    assert_eq!(toast.category_name, "Creative");

    toast.update(2.0);
    let mut fb = vec![0u32; 800 * 600];
    toast.draw(&mut fb, 800, 600, false, Some(0.4), Some(180.0));

    let painted_pixels = fb.iter().filter(|&&p| p != 0).count();
    assert!(
        painted_pixels > 100,
        "Toast must paint visible pixels to framebuffer"
    );

    toast.update(2.0);
    assert!(toast.visible);
    assert!(toast.timer <= 3.1);

    toast.update(4.0);
    assert!(!toast.visible);

    let mut fb_pause = vec![0u32; 800 * 600];
    toast.draw(&mut fb_pause, 800, 600, true, Some(0.75), Some(200.0));
    let painted_pause = fb_pause.iter().filter(|&&p| p != 0).count();
    assert!(
        painted_pause > 100,
        "Forced toast on pause menu must render pixels"
    );
}

#[test]
fn test_decode_user_opus_if_present() {
    let path = "soundtracks/uncategorized/Buant Groove.opus";
    if std::path::Path::new(path).exists() {
        let bytes = std::fs::read(path).unwrap();
        let sample = SoundSample::from_file_bytes(&bytes).unwrap();
        assert_eq!(sample.channels, 2);
        assert_eq!(sample.sample_rate, 48000);
        assert!(!sample.data.is_empty());
    }
}

#[test]
fn test_uncategorized_random_shuffling_and_no_repeats() {
    let mut mgr = SoundtrackManager::new("soundtracks");
    let uncategorized_indices: Vec<usize> = mgr
        .tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| t.categories.contains(&SoundtrackCategory::Uncategorized))
        .map(|(i, _)| i)
        .collect();

    assert!(
        uncategorized_indices.len() >= 3,
        "Expected at least 3 uncategorized tracks"
    );

    mgr.set_seed(0x12345678);
    mgr.refill_uncategorized_queue(&uncategorized_indices);
    let order_1 = mgr.uncategorized_queue.clone();

    mgr.set_seed(0x87654321);
    mgr.refill_uncategorized_queue(&uncategorized_indices);
    let order_2 = mgr.uncategorized_queue.clone();

    assert_ne!(
        order_1, order_2,
        "Different PRNG seeds must yield different random playlist orders"
    );

    assert_eq!(order_1.len(), uncategorized_indices.len());
    for idx in &uncategorized_indices {
        assert!(
            order_1.contains(idx),
            "Shuffle queue must contain all tracks without dropping any"
        );
    }
}
