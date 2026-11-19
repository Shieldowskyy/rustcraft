//! Interakcja z ekranem handlu z osadnikiem (`VillagerTrade`) w `RustCraftApp`.

use crate::core::controls::Action;
use crate::features::player::VillagerProfession;
use crate::features::ui::{draw_villager_trade_ui, VillagerTradeLayout, VillagerTradeSlotTarget};

use crate::app::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
    pub(crate) fn update_and_render_villager_trade(
        &mut self,
        dt: f32,
        input: &InputState,
        mob_id: u32,
        mut selected_trade: usize,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;

        self.world.update_furnaces(dt);

        if self.is_close_container_pressed(input) {
            self.state = GameState::Playing;
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            out.request_capture_mouse = true;
            return;
        }

        out.cursor_visible = Some(true);
        out.request_release_mouse = true;

        let mut trade_count = 0;
        let mut profession = VillagerProfession::Farmer;
        let mut mob_pos = self.camera.position;

        if let Some(mob) = self.mob_manager.mobs.iter().find(|m| m.id == mob_id) {
            trade_count = mob.trades.len();
            profession = VillagerProfession::from_u8(mob.villager_profession);
            mob_pos = mob.center_pos();
        }

        if trade_count == 0 {
            self.state = GameState::Playing;
            out.request_capture_mouse = true;
            return;
        }

        if selected_trade >= trade_count {
            selected_trade = 0;
        }

        if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
            selected_trade = (selected_trade + 1) % trade_count;
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.5, 1.2);
        } else if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
            selected_trade = if selected_trade == 0 {
                trade_count - 1
            } else {
                selected_trade - 1
            };
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.5, 1.2);
        }

        let layout = VillagerTradeLayout::new(self.renderer.width, self.renderer.height);
        let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        let mut executed_trade = false;

        if click_left {
            if let Some((mx, my)) = mouse_pos {
                if let Some(target) = layout.find_target_at(mx as i32, my as i32, trade_count) {
                    match target {
                        VillagerTradeSlotTarget::TradeOption(idx) => {
                            selected_trade = idx;
                            self.audio
                                .play_sound(crate::audio::SoundId::UiClick, 0.6, 1.1);
                        }
                        VillagerTradeSlotTarget::TradeButton => {
                            executed_trade = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if executed_trade {
            if let Some(mob) = self.mob_manager.mobs.iter_mut().find(|m| m.id == mob_id) {
                if let Some(trade) = mob.trades.get_mut(selected_trade) {
                    if trade.execute(&mut self.player) {
                        self.audio
                            .play_sound(crate::audio::SoundId::VillagerTrade, 1.0, 1.0);
                        let q_trade = self.quests.record_trade();
                        self.process_quest_completions(q_trade);
                        self.particles.spawn_growth(mob_pos);
                    } else {
                        self.audio
                            .play_sound(crate::audio::SoundId::VillagerNo, 1.0, 1.0);
                    }
                }
            }
        }

        self.state = GameState::VillagerTrade {
            mob_id,
            selected_trade,
        };
        self.prev_left_down = left_down;

        self.render_menu_world_backdrop();

        let trades_clone = if let Some(mob) = self.mob_manager.mobs.iter().find(|m| m.id == mob_id) {
            mob.trades.clone()
        } else {
            Vec::new()
        };

        draw_villager_trade_ui(
            &mut self.renderer.color_buffer,
            self.renderer.width,
            self.renderer.height,
            profession,
            &trades_clone,
            selected_trade,
            &self.player,
            mouse_pos,
        );
    }
}
