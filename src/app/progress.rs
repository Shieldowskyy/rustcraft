//! Zarządzanie postępem gracza (osiągnięcia, questy, zapis świata), tło menu oraz implementacja `EngineApp`.

use crate::core::controls::Action;
use crate::features::blocks::BlockId;
use crate::features::player::{AchievementId, AchievementProgress, QuestProgress};
use crate::features::world::{
    load_achievement_progress, load_quest_progress, save_achievement_progress, save_current_world,
    save_quest_progress,
};

use super::{AppOutput, InputState, RustCraftApp};

impl RustCraftApp {
    pub(crate) fn render_menu_world_backdrop(&mut self) {
        if !crate::platform::capabilities().uses_hardware_gxm {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        } else {
            self.renderer.color_buffer.fill(0);
        }
    }

    pub(crate) fn process_achievement_unlocks(&mut self, unlocked: Vec<AchievementId>) {
        if unlocked.is_empty() {
            return;
        }
        self.pending_achievement_toasts.extend(unlocked);
        if self.achievement_toast.is_none() {
            self.achievement_toast = self
                .pending_achievement_toasts
                .pop_front()
                .map(|id| (id, 6.0));
        }
        self.persist_achievement_progress();
    }

    pub(crate) fn record_discovered_block(&mut self, block: BlockId) {
        if self.achievements.record_block_discovered(block) {
            self.persist_achievement_progress();
        }
    }

    pub(crate) fn record_collected_item(&mut self, block: BlockId) {
        if self.achievements.record_item_collected(block) {
            self.persist_achievement_progress();
        }
        let q_inv = self.quests.sync_inventory(&self.player.inventory);
        self.process_quest_completions(q_inv);
    }

    pub(crate) fn record_inventory_items_collected(&mut self) {
        let inventory = &self.player.inventory;
        let blocks: Vec<BlockId> = inventory
            .hotbar
            .iter()
            .chain(inventory.main.iter())
            .chain(inventory.crafting_2x2.iter())
            .chain(inventory.crafting_3x3.iter())
            .chain(inventory.armor.iter())
            .chain(std::iter::once(&inventory.offhand))
            .chain(std::iter::once(&inventory.held_item))
            .filter_map(|slot| slot.map(|stack| stack.block))
            .collect();
        let mut changed = false;
        for block in blocks {
            changed |= self.achievements.record_item_collected(block);
        }
        if changed {
            self.persist_achievement_progress();
        }
    }

    pub(crate) fn persist_achievement_progress(&self) {
        if let Err(error) = save_achievement_progress(self.save_slot, &self.achievements) {
            crate::log_error!("ACHIEVEMENT", "{}", error);
        }
    }

    pub(crate) fn reset_achievements_for_slot(&mut self, slot: u8) {
        self.achievements = AchievementProgress::default();
        self.achievement_toast = None;
        self.pending_achievement_toasts.clear();
        if let Err(error) = save_achievement_progress(slot, &self.achievements) {
            crate::log_error!("ACHIEVEMENT", "{}", error);
        }
    }

    pub(crate) fn load_achievements_for_slot(&mut self, slot: u8) {
        self.achievement_toast = None;
        self.pending_achievement_toasts.clear();
        match load_achievement_progress(slot) {
            Ok(progress) => self.achievements = progress,
            Err(error) => {
                crate::log_error!("ACHIEVEMENT", "{}", error);
                self.achievements = AchievementProgress::default();
            }
        }
    }

    pub(crate) fn questbook_key_hint(&self) -> &'static str {
        let binding = &self.controls.config.bindings[Action::Questbook.index()];
        binding
            .keys
            .first()
            .map(|k| k.display_name())
            .or_else(|| binding.mouse.first().map(|m| m.display_name()))
            .or_else(|| binding.gamepad.first().map(|g| g.display_name()))
            .unwrap_or("Esc")
    }

    pub(crate) fn process_quest_completions(&mut self, completed: Vec<usize>) {
        if !self.quests.enabled || completed.is_empty() {
            return;
        }
        self.pending_quest_toasts.extend(completed);
        if self.quest_toast.is_none() {
            self.quest_toast = self.pending_quest_toasts.pop_front().map(|id| (id, 5.5));
        }
        self.audio
            .play_sound(crate::audio::SoundId::UiClick, 0.75, 1.35);
        if let Err(error) = save_quest_progress(self.save_slot, &self.quests) {
            crate::log_error!("QUEST", "{}", error);
        }
    }

    pub(crate) fn reset_quests_for_slot(&mut self, slot: u8, enabled: bool) {
        self.quests = QuestProgress::default();
        self.quests.enabled = enabled;
        self.quest_toast = None;
        self.pending_quest_toasts.clear();
        if let Err(error) = save_quest_progress(slot, &self.quests) {
            crate::log_error!("QUEST", "{}", error);
        }
    }

    pub(crate) fn load_quests_for_slot(&mut self, slot: u8) {
        self.quest_toast = None;
        self.pending_quest_toasts.clear();
        match load_quest_progress(slot) {
            Ok(progress) => self.quests = progress,
            Err(error) => {
                crate::log_error!("QUEST", "{}", error);
                self.quests = QuestProgress::default();
            }
        }
    }

    pub(crate) fn save_world_progress(&mut self) -> Result<(), String> {
        let world_result = save_current_world(
            &mut self.world,
            &self.camera,
            &self.player,
            self.save_slot,
            self.last_clean_thumbnail.as_deref(),
        );
        let achievement_result = save_achievement_progress(self.save_slot, &self.achievements);
        let quest_result = save_quest_progress(self.save_slot, &self.quests);
        let result = world_result.and(achievement_result).and(quest_result);
        if let Err(error) = &result {
            crate::log_error!("SAVE", "Failed to save world progress: {}", error);
        }
        result
    }
}

impl crate::engine::EngineApp for RustCraftApp {
    fn update(&mut self, dt: f32, input: &InputState) -> AppOutput {
        self.update_and_render(dt, input)
    }

    fn render(&mut self) {}

    fn resize(&mut self, width: usize, height: usize) {
        self.resize(width, height);
    }

    fn pixel_buffer(&self) -> &[u32] {
        self.pixel_buffer()
    }
}
