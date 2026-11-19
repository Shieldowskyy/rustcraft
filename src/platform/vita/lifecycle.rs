//! PS Vita application startup, PRX loading, overclocking, and main game loop (`run_vita_app`).
//! Coordinates `RustCraftApp`, `GxmRenderer`, `VitaInputTracker`, and `VitaImeManager`.

#![allow(dead_code)]

use crate::app::RustCraftApp;

use super::ffi::*;
use super::ime::VitaImeManager;
use super::input::VitaInputTracker;
use super::panic_log::{install_vita_panic_hook, write_crash_bytes};

// ============================================================================
// 4. PS Vita Main Loop Entry Point
// ============================================================================

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

/// Main application entry point for PlayStation Vita – launches GXM hardware renderer.
pub fn run_vita_app() {
    // 0. Install panic hook immediately at startup before any allocations or PRX calls
    install_vita_panic_hook();

    #[cfg(debug_assertions)]
    write_crash_bytes(
        b"ux0:data/rustcraft_boot.log\0",
        b"=== RustCraft PS Vita Debug Boot Log ===\n",
        false,
    );

    // 1. Load required Vita system modules
    //    NOTE: Do NOT load `libc.suprx` on `armv7-sony-vita-newlibeabihf` because Newlib
    //    already provides `malloc`/`free` statically and `libc.suprx` wastes RAM on a second heap.
    load_module(b"vs0:sys/external/libSceGxm.suprx\0");
    load_module(b"vs0:sys/external/libSceIme.suprx\0");

    crate::vita_log!("Modules loaded. Setting clock frequencies...");

    // 2. Overclock PS Vita clocks to maximum performance
    //    444MHz ARM, 222MHz BUS, 222MHz GPU, 166MHz XBAR
    unsafe {
        scePowerSetArmClockFrequency(444);
        scePowerSetBusClockFrequency(222);
        scePowerSetGpuClockFrequency(222);
        scePowerSetGpuXbarClockFrequency(166);
    }

    crate::vita_log!("Initializing RustCraftApp on heap...");

    // 3. Initialize game engine (boxed on heap to prevent stack frame overflow).
    //    CRITICAL: Use `"gxm"` (not `"software"`) so `app.update_and_render` skips
    //    CPU 3D software rasterization and only draws the 2D HUD/UI overlay!
    let seed = unsafe { sceKernelGetProcessTimeWide() } as u32;
    let mut app = Box::new(RustCraftApp::new(960, 544, "gxm", seed));
    #[cfg(feature = "profiler")]
    {
        app.profiler_enabled = true;
        app.settings.debug_overlay = true;
    }

    crate::vita_log!("Initializing GxmRenderer...");

    // 4. Initialize GXM Hardware Renderer
    //    Allocates CDRAM triple-buffering, ShaderPatcher, GPU texture atlas, render targets.
    let mut gxm = Box::new(crate::platform::vita_gxm::GxmRenderer::new(&app.renderer.textures));

    crate::vita_log!("Initializing controller...");

    // 5. Initialize gamepad controller and touchscreen tracker
    let mut input_tracker = VitaInputTracker::new();
    let mut ime_mgr = VitaImeManager::new();
    let mut last_time = unsafe { sceKernelGetProcessTimeWide() };

    crate::vita_log!("Entering main game loop...");

    // Main game loop
    loop {
        let current_time = unsafe { sceKernelGetProcessTimeWide() };
        let dt_us = current_time.saturating_sub(last_time);
        last_time = current_time;
        let dt = (dt_us as f32 / 1_000_000.0).clamp(0.001, 0.1);

        // Check if app requested opening Vita on-screen keyboard (IME)
        if let Some((ref title, ref initial)) = app.ime_requested.take() {
            ime_mgr.start(title, initial);
        }

        // Update IME dialog if running
        if let Some(ime_result) = ime_mgr.update() {
            match ime_result {
                Some(text) => app.apply_ime_text(&text),
                None => app.cancel_ime(),
            }
        }

        // Poll Vita controller (in menus free cursor is active, in-game camera is locked to analog stick)
        let is_captured = matches!(app.state, crate::app::GameState::Playing);
        let input = input_tracker.poll(is_captured, dt);

        // Update game logic (physics, camera, world streaming, AI, 2D UI)
        let output = app.update_and_render(dt, &input);

        if output.quit_requested {
            crate::vita_log!("Quit requested from menu. Saving world state...");
            let _ = crate::world::storage::save_current_world(
                &mut app.world,
                &app.camera,
                &app.player,
                app.save_slot,
                None,
            );
            break;
        }

        if matches!(app.state, crate::app::GameState::Playing) {
            let hand_block: Option<u8> = app.player.inventory.hotbar
                [app.player.selected_hotbar_slot as usize]
                .map(|s| s.block.to_u8());
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

            let tri_count = gxm.render_frame(
                &app.camera,
                &mut app.world,
                Some(app.renderer.pixel_buffer()),
                app.break_target,
                app.break_progress,
                &app.items,
                &app.mob_manager.mobs,
                hand_block,
                app.walk_phase,
                hand_speed,
                app.hand_animation,
                app.settings.render_distance,
                app.settings.fog,
                app.settings.clouds,
                app.settings.chunk_anim,
                app.total_time,
                if app.settings.particles {
                    &app.particles.particles
                } else {
                    &[]
                },
                if app.settings.damage_tilt {
                    app.damage_tilt_timer
                } else {
                    0.0
                },
                is_underwater,
                is_in_lava,
                low_health,
            );
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