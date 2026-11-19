//! Unit tests for profiler telemetry serialization, BMP encoding, and server lifecycle.

use super::*;

#[test]
fn test_profiler_snapshot_json_and_csv_formatting() {
    let snapshot = ProfilerSnapshot {
        timestamp_ms: 1700000000,
        fps: 60,
        frame_time_ms: 16.67,
        tick_ms: 2.1,
        world_io_ms: 1.2,
        render_terrain_ms: 7.5,
        render_trans_ms: 2.0,
        render_clouds_ms: 0.5,
        render_entities_ms: 1.1,
        render_sky_ms: 0.4,
        render_hand_ms: 0.3,
        render_ui_ms: 0.6,
        tick_physics_ms: 0.8,
        tick_mobs_ms: 0.6,
        tick_entities_ms: 0.4,
        tick_world_ms: 0.3,
        world_gen_ms: 0.4,
        world_mesh_ms: 0.8,
        render_cull_ms: 0.2,
        total_ram_bytes: 32 * 1024 * 1024,
        chunk_voxels_bytes: 12 * 1024 * 1024,
        mesh_cache_bytes: 8 * 1024 * 1024,
        textures_and_fonts_bytes: 2 * 1024 * 1024,
        framebuffers_bytes: 4 * 1024 * 1024,
        entities_bytes: 64 * 1024,
        ui_and_misc_bytes: 128 * 1024,
        cdram_bytes: 0,
        pipeline_name: "vulkan".to_string(),
        renderer_name: "Vulkan 1.3".to_string(),
        gpu_device: "Test GPU".to_string(),
        driver_info: "Test Driver".to_string(),
        draw_calls: 42,
        triangles_rendered: 15400,
        triangles_culled: 3200,
        subchunks_rendered: 120,
        subchunks_culled_frustum: 40,
        raster_threads: 16,
        chunks_loaded: 144,
        player_pos: [100.0, 64.0, -200.0],
        biome: "PLAINS".to_string(),
        daylight_factor: 1.0,
        platform: "Linux x86_64".to_string(),
        is_paused: false,
        game_state: "Playing".to_string(),
        gamemode: "Survival".to_string(),
        render_distance: 6,
        fov_deg: 70.0,
        msaa: 1,
        fps_limit: 0,
        leaf_quality: "Fancy".to_string(),
        clouds: true,
        fog: true,
        particles: true,
        chunk_anim: true,
        underwater_fx: true,
        debug_overlay: true,
    };

    let json = snapshot.to_json();
    assert!(json.contains("\"fps\":60"));
    assert!(json.contains("\"pipeline\":\"vulkan\""));
    assert!(json.contains("\"total_mb\":32.00"));
    assert!(json.contains("\"chunks_loaded\":144"));
    assert!(json.contains("\"is_paused\":false"));
    assert!(json.contains("\"render_distance\":6"));

    let csv = snapshot.to_csv_row();
    assert!(csv.contains("60,16.67"));
    assert!(csv.contains("PLAINS"));
    assert!(csv.contains("144"));
    assert!(csv.contains("Playing"));
}

#[test]
fn test_encode_bmp_and_screenshot_metadata() {
    let width = 4;
    let height = 2;
    let color_buffer = vec![
        0x00FF0000, 0x0000FF00, 0x000000FF, 0x00FFFFFF,
        0x00123456, 0x00654321, 0x00ABCDEF, 0x00000000,
    ];

    let bmp = encode_bmp(&color_buffer, width, height);
    assert_eq!(&bmp[0..2], b"BM");
    assert_eq!(bmp.len(), 54 + (width * 3) * height);

    let data = ScreenshotData {
        bmp_bytes: bmp,
        width,
        height,
        timestamp_ms: 1700000000,
        fps: 60,
        player_pos: [10.0, 64.0, -20.0],
        biome: "FOREST".to_string(),
        game_state: "Playing".to_string(),
        gamemode: "Creative".to_string(),
    };

    let json = data.to_json();
    assert!(json.contains("\"width\":4"));
    assert!(json.contains("\"height\":2"));
    assert!(json.contains("\"fps\":60"));
    assert!(json.contains("\"biome\":\"FOREST\""));
}

#[test]
fn test_profiler_server_lifecycle() {
    let mut server = ProfilerServer::start(18080).expect("Failed to bind test profiler server");
    assert_eq!(server.port, 18080);
    assert!(!server.ip_address.is_empty());

    let mut snap = ProfilerSnapshot::default();
    snap.fps = 120;
    snap.platform = "Unit Test".to_string();
    server.update_snapshot(snap);

    server.request_screenshot();
    assert!(server.is_screenshot_requested());

    let shot = ScreenshotData {
        bmp_bytes: vec![1, 2, 3],
        width: 1,
        height: 1,
        timestamp_ms: 12345,
        fps: 120,
        player_pos: [0.0, 0.0, 0.0],
        biome: "PLAINS".to_string(),
        game_state: "Playing".to_string(),
        gamemode: "Survival".to_string(),
    };
    server.set_screenshot(shot);
    assert!(!server.is_screenshot_requested());
    assert!(server.get_screenshot().is_some());

    server.stop();
    assert!(!server.running.load(std::sync::atomic::Ordering::Relaxed));
}

