//! Playing-state gameplay loop orchestrator (`RustCraftApp::update_and_render_playing`).
//! Coordinates network sync, chat/shortcuts, movement, vitals, mining, interactions, simulation, and HUD/scene rendering.
//! Delegates each subsystem to focused sibling modules with zero platform `#[cfg]` gates.

mod chat_shortcuts;
mod hud_debug;
mod interact;
mod mining;
mod movement;
mod net_sync;
mod projectiles;
mod render_scene;
mod simulation;
mod vitals;
mod world_interact;

#[cfg(test)]
mod tests;

use crate::core::math::IVec3;
use crate::core::profiler::FrameProfiler;
use crate::gameplay::mob::Mob;
use crate::platform::capabilities;
use crate::world::block::BlockId;

use super::{AppOutput, InputState, RustCraftApp};

pub(super) fn can_place_block_in(existing: BlockId) -> bool {
    matches!(existing, BlockId::Air | BlockId::Water)
}

pub(super) fn block_intersects_live_mob(place_pos: IVec3, mobs: &[Mob]) -> bool {
    mobs.iter()
        .any(|mob| !mob.is_dead && mob.intersects_block(place_pos))
}

impl RustCraftApp {
    pub(crate) fn update_and_render_playing(
        &mut self,
        dt: f32,
        input: &InputState,
        out: &mut AppOutput,
    ) {
        self.profiler_frame_count = self.profiler_frame_count.wrapping_add(1);
        let sample_interval = capabilities().profiler_sample_interval.max(1) as u64;
        let should_sample_detail = self.profiler_frame_count % sample_interval == 0;
        let collect_profiling =
            ((self.profiler_enabled && self.settings.debug_overlay) || self.profiler_server.is_some())
                && should_sample_detail;
        let frame_start_time = collect_profiling.then(std::time::Instant::now);
        let mut cur_profiler = FrameProfiler::default();

        self.chat.update(dt);
        self.sync_playing_network(dt, out);
        self.log_playing_telemetry(dt);
        self.handle_chat_and_shortcuts(input, out);

        let phys_start = collect_profiling.then(std::time::Instant::now);
        let (wish_dir, is_crouching, is_sprinting) = self.update_camera_and_stance(dt, input, out);
        self.update_player_locomotion(dt, input, wish_dir, is_crouching, is_sprinting, out);
        self.update_post_movement_bounds_and_hotbar(dt, input);

        let selected_block_id = self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize]
            .map(|stack| stack.block.to_u8())
            .unwrap_or(0);
        self.renderer.set_selected_block(selected_block_id);

        self.handle_mining_and_combat(dt, input);
        self.handle_creative_pick_block(input);
        self.handle_right_click_interactions(dt, is_crouching, selected_block_id, out);
        if let Some(p_start) = phys_start {
            cur_profiler.tick_physics_ms = p_start.elapsed().as_secs_f32() * 1000.0;
        }

        let hand_speed = self.tick_world_and_entities(dt, collect_profiling, &mut cur_profiler, out);
        let effective_camera = self.render_playing_scene(collect_profiling, &mut cur_profiler);
        self.render_playing_hud_and_debug(
            dt,
            input,
            &effective_camera,
            hand_speed,
            is_crouching,
            is_sprinting,
            collect_profiling,
            frame_start_time,
            cur_profiler,
        );
    }
}
