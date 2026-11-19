//! Unit tests for `GameSettings` and quality/control enums.

use super::*;

#[test]
fn test_leaf_quality_cycling_and_distances() {
    let mut quality = LeafQuality::Fancy;
    assert_eq!(quality.max_distance_sq(), f32::INFINITY);
    assert!(quality.is_fancy());

    quality.cycle(true);
    assert_eq!(quality, LeafQuality::Fast);
    assert_eq!(quality.max_distance_sq(), 0.0);
    assert!(!quality.is_fancy());

    quality.cycle(true);
    assert_eq!(quality, LeafQuality::Near);
    assert_eq!(quality.max_distance_sq(), 256.0);

    quality.cycle(true);
    assert_eq!(quality, LeafQuality::Medium);
    assert_eq!(quality.max_distance_sq(), 1024.0);

    quality.cycle(true);
    assert_eq!(quality, LeafQuality::Far);
    assert_eq!(quality.max_distance_sq(), 4096.0);

    quality.cycle(false);
    assert_eq!(quality, LeafQuality::Medium);
}

#[test]
fn test_control_modes_cycling() {
    let mut sprint = SprintMode::Hold;
    sprint.cycle();
    assert_eq!(sprint, SprintMode::Toggle);
    sprint.cycle();
    assert_eq!(sprint, SprintMode::Hold);

    let mut sneak = SneakMode::Hold;
    sneak.cycle();
    assert_eq!(sneak, SneakMode::Toggle);

    let mut jump = JumpMode::Normal;
    jump.cycle();
    assert_eq!(jump, JumpMode::AutoJump);
    jump.cycle();
    assert_eq!(jump, JumpMode::Normal);
}

#[test]
fn test_invert_axes_toggle() {
    let mut settings = GameSettings::default();
    assert!(!settings.invert_y);
    assert!(!settings.invert_z);

    settings.toggle_invert_y();
    assert!(settings.invert_y);

    settings.toggle_invert_z();
    assert!(settings.invert_z);
}

#[test]
fn test_profiler_server_settings() {
    let mut settings = GameSettings::default();
    assert!(!settings.profiler_server);
    assert_eq!(settings.profiler_server_port, 8080);

    settings.toggle_profiler_server();
    assert!(settings.profiler_server);

    settings.cycle_profiler_server_port(true);
    assert_eq!(settings.profiler_server_port, 8088);

    settings.cycle_profiler_server_port(false);
    assert_eq!(settings.profiler_server_port, 8080);

    settings.cycle_profiler_server_port(false);
    assert_eq!(settings.profiler_server_port, 3000);
}

#[test]
fn test_volume_control_5_percent_and_rounding() {
    let mut settings = GameSettings::default();
    settings.master_volume = 0.83;
    settings.cycle_master_volume(true);
    assert_eq!((settings.master_volume * 100.0).round() as i32, 90);

    settings.master_volume = 0.82;
    settings.cycle_master_volume(true);
    assert_eq!((settings.master_volume * 100.0).round() as i32, 85);

    settings.cycle_master_volume(true);
    assert_eq!((settings.master_volume * 100.0).round() as i32, 90);

    settings.cycle_master_volume(false);
    assert_eq!((settings.master_volume * 100.0).round() as i32, 85);

    settings.master_volume = 1.0;
    settings.cycle_master_volume(true);
    assert_eq!((settings.master_volume * 100.0).round() as i32, 0);

    settings.cycle_master_volume(false);
    assert_eq!((settings.master_volume * 100.0).round() as i32, 100);
}

#[test]
fn test_default_master_volume_is_50_percent() {
    assert_eq!(GameSettings::default().master_volume, 0.5);
}

#[test]
fn test_cycle_brightness() {
    let mut settings = GameSettings::default();
    assert_eq!(settings.brightness, 0.5);

    settings.cycle_brightness(true);
    assert_eq!(settings.brightness, 0.75);

    settings.cycle_brightness(true);
    assert_eq!(settings.brightness, 1.0);

    settings.cycle_brightness(true);
    assert_eq!(settings.brightness, 0.0);

    settings.cycle_brightness(false);
    assert_eq!(settings.brightness, 1.0);
}

