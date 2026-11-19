//! PS Vita application startup, PRX loading, overclocking, and main game loop (`run_vita_app`).
//! Coordinates `RustCraftApp`, `GxmRenderer`, `VitaInputTracker`, and `VitaImeManager`.

#![allow(dead_code)]

use crate::app::{GameState, RustCraftApp};

use super::ffi::*;
use super::ime::VitaImeManager;
use super::input::VitaInputTracker;
use super::panic_log::{install_vita_panic_hook, write_crash_bytes};

/// Loads a PS Vita system PRX module if not already loaded.
fn load_module(path: &[u8]) {
    unsafe {
        let mut status: i32 = 0;
        sceKernelLoadStartModule(
            path.as_ptr(),
            0,
            core::ptr::null(),
            0,
            core::ptr::null(),
            &mut status,
        );
    }
}

/// Main application entry point for PlayStation Vita — launches GXM hardware renderer.
pub fn run_vita_app() {
    install_vita_panic_hook();

    #[cfg(debug_assertions)]
    write_crash_bytes(
        b"ux0:data/rustcraft_boot.log\0",
        b"=== RustCraft PS Vita Debug Boot Log ===\n",
        false,
    );

    // Do NOT load `libc.suprx` on `armv7-sony-vita-newlibeabihf`: Newlib provides `malloc`/`free`
    // statically and `libc.suprx` wastes RAM on a second heap.
    load_module(b"vs0:sys/external/libSceGxm.suprx\0");
    load_module(b"vs0:sys/external/libSceIme.suprx\0");

    crate::vita_log!("Modules loaded. Setting clock frequencies...");
    unsafe {
        scePowerSetArmClockFrequency(444);
        scePowerSetBusClockFrequency(222);
        scePowerSetGpuClockFrequency(222);
        scePowerSetGpuXbarClockFrequency(166);
    }

    crate::vita_log!("Initializing RustCraftApp on heap...");
    let seed = unsafe { sceKernelGetProcessTimeWide() } as u32;
    let mut app = Box::new(RustCraftApp::new(960, 544, "gxm", seed));
    #[cfg(feature = "profiler")]
    {
        app.profiler_enabled = true;
        app.settings.debug_overlay = true;
    }

    crate::vita_log!("Initializing GxmRenderer...");
    let mut gxm = Box::new(crate::platform::vita_gxm::GxmRenderer::new(&app.renderer.textures));

    let mut synced_textures_ver = app.renderer.textures_version;
    let mut synced_mesh_ver = app.renderer.mesh_cache_version;
    let mut capture_buf: Vec<u32> = Vec::with_capacity(960 * 544);

    let mut input_tracker = VitaInputTracker::new();
    let mut ime_mgr = VitaImeManager::new();
    let mut last_time = unsafe { sceKernelGetProcessTimeWide() };

    crate::vita_log!("Entering main game loop...");

    loop {
        let current_time = unsafe { sceKernelGetProcessTimeWide() };
        let dt_us = current_time.saturating_sub(last_time);
        last_time = current_time;
        let dt = (dt_us as f32 / 1_000_000.0).clamp(0.001, 0.1);

        if let Some((ref title, ref initial)) = app.ime_requested.take() {
            ime_mgr.start(title, initial);
        }

        if let Some(ime_result) = ime_mgr.update() {
            match ime_result {
                Some(text) => app.apply_ime_text(&text),
                None => app.cancel_ime(),
            }
        }

        let is_captured = matches!(app.state, GameState::Playing);
        let input = input_tracker.poll(is_captured, dt);
        let output = app.update_and_render(dt, &input);

        if output.quit_requested {
            crate::vita_log!("Quit requested from menu. Saving world state...");
            let _ = crate::world::storage::save_current_world(
                &mut app.world,
                &app.camera,
                &app.player,
                app.save_slot,
                app.last_clean_thumbnail.as_deref(),
            );
            break;
        }

        if app.renderer.textures_version != synced_textures_ver {
            gxm.update_textures(&app.renderer.textures);
            synced_textures_ver = app.renderer.textures_version;
        }
        if app.renderer.mesh_cache_version != synced_mesh_ver {
            gxm.clear_mesh_cache();
            synced_mesh_ver = app.renderer.mesh_cache_version;
        }

        if app.uses_live_3d_scene() {
            let is_menu_backdrop = !matches!(app.state, GameState::Playing);
            let active_cam = if is_menu_backdrop {
                &app.camera
            } else {
                &app.last_effective_camera
            };
            let feet_pos = crate::core::math::Vec3::new(
                app.camera.position.x,
                app.camera.position.y - app.current_eye_offset,
                app.camera.position.z,
            );
            let hand_block = app.player.inventory.hotbar
                [app.player.selected_hotbar_slot as usize]
                .map(|s| s.block.to_u8());
            let offhand_block = app.player.inventory.offhand.as_ref().map(|s| s.block.to_u8());
            let hand_speed = (app.horiz_velocity.length() + app.fly_velocity.length()).max(0.0);

            let block_at_camera = app.world.get_block(app.camera.position.floor_to_ivec());
            let is_underwater = app.settings.underwater_fx
                && matches!(
                    block_at_camera,
                    crate::world::block::BlockId::Water
                        | crate::world::block::BlockId::Kelp
                        | crate::world::block::BlockId::Seagrass
                );
            let is_in_lava = block_at_camera == crate::world::block::BlockId::Lava;
            let low_health = app.player.gamemode == crate::gameplay::player::GameMode::Survival
                && app.player.health <= 2.0
                && app.player.health > 0.0;

            let frame_params = crate::platform::vita_gxm::GxmFrameParams {
                camera: active_cam,
                player_feet_pos: feet_pos,
                player_yaw: app.camera.yaw,
                player_pitch: app.camera.pitch,
                player_crouching: app.crouch_toggled,
                player_armor: &app.player.inventory.armor,
                camera_perspective: app.camera_perspective,
                is_menu_backdrop,
                hud_pixels: Some(app.renderer.pixel_buffer()),
                hud_width: app.width,
                hud_height: app.height,
                break_target: app.break_target,
                break_progress: app.break_progress,
                items: &app.items,
                mobs: &app.mob_manager.mobs,
                particles: if app.settings.particles { &app.particles.particles } else { &[] },
                arrows: &app.arrows,
                fishing_bobber: app.fishing_bobber.as_ref(),
                remote_players: app.net_client.as_ref().map(|c| &c.remote_players),
                hand_block,
                offhand_block,
                is_blocking: app.player.is_blocking,
                walk_phase: app.walk_phase,
                speed: hand_speed,
                attack_animation: app.hand_animation,
                render_distance: app.settings.render_distance,
                fog_enabled: app.settings.fog,
                clouds_enabled: app.settings.clouds,
                chunk_anim: app.settings.chunk_anim,
                brightness: app.settings.brightness,
                total_time: app.total_time,
                biome_tint: app.biome_filter_tint,
                damage_tilt_timer: app.damage_tilt_timer,
                is_underwater,
                is_in_lava,
                is_burning: app.player.fire_timer > 0.0,
                low_health,
            };

            let tri_count = gxm.render_frame(&mut app.world, &frame_params);
            app.gpu_triangles = tri_count;
            let (slots_used, max_slots, cdram, ram) = gxm.get_perf_stats();
            app.chunk_slots_info = (slots_used, max_slots);
            app.cdram_mb = cdram as f32 / (1024.0 * 1024.0);
            app.ram_mb = ram as f32 / (1024.0 * 1024.0);

            let (draw_calls, tris_rendered, tris_culled, subchunks_rendered, subchunks_culled) =
                gxm.last_render_telemetry();
            app.renderer.last_draw_calls = draw_calls;
            app.renderer.last_triangles_rendered = tris_rendered;
            app.renderer.last_triangles_culled = tris_culled;
            app.renderer.last_subchunks_rendered = subchunks_rendered;
            app.renderer.last_subchunks_culled = subchunks_culled;

            let need_shot = app
                .profiler_server
                .as_ref()
                .is_some_and(|s| s.is_screenshot_requested());
            let mut need_thumb = false;
            if !is_menu_backdrop {
                app.thumbnail_timer += dt;
                if app.thumbnail_timer >= 0.5 {
                    app.thumbnail_timer = 0.0;
                    need_thumb = true;
                }
            }
            if (need_thumb || need_shot) && gxm.capture_completed_xrgb(&mut capture_buf, need_shot) {
                if need_thumb {
                    app.last_clean_thumbnail = Some(crate::world::storage::capture_thumbnail(
                        &capture_buf,
                        app.width,
                        app.height,
                    ));
                }
                if need_shot {
                    app.submit_profiler_screenshot(&capture_buf);
                }
            }
        } else {
            gxm.present_2d_buffer(app.renderer.pixel_buffer(), app.width, app.height);
        }

        if app.settings.fps_limit > 0 {
            let target_frame_us = 1_000_000u64 / app.settings.fps_limit as u64;
            let frame_end = unsafe { sceKernelGetProcessTimeWide() };
            let elapsed_us = frame_end.saturating_sub(current_time);
            if elapsed_us < target_frame_us {
                let sleep_us = target_frame_us - elapsed_us;
                if sleep_us >= 500 {
                    unsafe {
                        sceKernelDelayThread(sleep_us as u32);
                    }
                }
            }
        }
    }

    crate::vita_log!("Exiting RustCraft clean process shutdown...");
    drop(app);
    drop(gxm);

    unsafe {
        sceKernelExitProcess(0);
    }
}
