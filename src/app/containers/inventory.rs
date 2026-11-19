//! Interakcja z ekranem ekwipunku (crafting 2×2/3×3, książka przepisów, zakładki Creative, zbroja i hotbar).

use crate::core::controls::Action;
use crate::features::blocks::{ArmorSlot, BlockId};
use crate::features::player::{GameMode, Inventory, ItemStack, RecipeBook};
use crate::features::ui::{InventoryLayout, InventorySlotTarget};

use crate::app::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
    pub(super) fn cycle_hotbar_scroll(&mut self, input: &InputState) {
        if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
            self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot == 0 {
                9
            } else {
                self.player.selected_hotbar_slot - 1
            };
        }
        if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
            self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 {
                0
            } else {
                self.player.selected_hotbar_slot + 1
            };
        }
    }

    pub(super) fn is_close_container_pressed(&self, input: &InputState) -> bool {
        self.controls.is_just_pressed(Action::Inventory)
            || self.controls.is_just_pressed(Action::Pause)
            || (input.key_escape && !self.prev_escape_down)
            || input.gp_east_just_pressed
    }

    pub(crate) fn update_and_render_inventory(
        &mut self,
        dt: f32,
        input: &InputState,
        mouse_moved: bool,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let right_down = input.mouse_right;

        self.world.update_furnaces(dt);

        let unlocked_recipe_indices = self.recipe_book.unlocked_recipe_indices(&self.achievements);
        if self.crafting_table_open
            && self.player.gamemode != GameMode::Creative
            && self.recipe_book_open
            && input.scroll_delta != 0.0
        {
            let max_scroll = unlocked_recipe_indices.len().saturating_sub(5);
            if input.scroll_delta < 0.0 {
                self.recipe_book_scroll = (self.recipe_book_scroll + 1).min(max_scroll);
            } else {
                self.recipe_book_scroll = self.recipe_book_scroll.saturating_sub(1);
            }
        } else if self.player.gamemode == GameMode::Creative
            && self.player.creative_tab < crate::ui::CREATIVE_TABS.len() - 1
        {
            let blocks = crate::ui::inventory_ui::creative_tab_blocks(self.player.creative_tab);
            let total_rows = (blocks.len() + 8) / 9;
            let max_scroll = total_rows.saturating_sub(4);
            if input.scroll_delta < 0.0 {
                self.player.creative_scroll = (self.player.creative_scroll + 1).min(max_scroll);
            } else if input.scroll_delta > 0.0 {
                self.player.creative_scroll = self.player.creative_scroll.saturating_sub(1);
            }
        } else {
            self.cycle_hotbar_scroll(input);
        }

        if self.is_close_container_pressed(input) {
            self.player.inventory.return_crafting_items();
            if self.player.gamemode == GameMode::Creative {
                self.player.inventory.held_item = None;
            }
            self.crafting_table_open = false;
            self.recipe_book_open = false;
            self.recipe_book_status = None;
            self.state = GameState::Playing;
            self.audio
                .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            out.request_capture_mouse = true;
            crate::log_info!("MENU", "Closed inventory screen, returning to Playing");
        }

        let layout = InventoryLayout::new(
            self.renderer.width,
            self.renderer.height,
            self.crafting_table_open,
        );
        let mut inv_hovered_target = if self.is_gamepad_active && !mouse_moved {
            None
        } else {
            mouse_pos.and_then(|(mx, my)| layout.find_slot_at(mx as i32, my as i32, &self.player))
        };
        if self.crafting_table_open
            && self.player.gamemode != GameMode::Creative
            && (!self.is_gamepad_active || mouse_moved)
        {
            if let Some((mx, my)) = mouse_pos {
                if let Some(target) = layout.find_recipe_book_target_at(
                    mx as i32,
                    my as i32,
                    self.recipe_book_open,
                    &unlocked_recipe_indices,
                    self.recipe_book_scroll,
                ) {
                    inv_hovered_target = Some(target);
                }
            }
        }

        for i in 0..10 {
            if input.key_digits[i] && !self.prev_key_digits[i] {
                if let Some(target) = inv_hovered_target {
                    match target {
                        InventorySlotTarget::Creative(id) => {
                            self.player.inventory.hotbar[i] =
                                Some(ItemStack::new(BlockId::from_u8(id), 64));
                        }
                        InventorySlotTarget::Hotbar(from_idx) => {
                            self.player.inventory.hotbar.swap(i, from_idx);
                        }
                        InventorySlotTarget::Main(from_idx) => {
                            std::mem::swap(
                                &mut self.player.inventory.hotbar[i],
                                &mut self.player.inventory.main[from_idx],
                            );
                        }
                        InventorySlotTarget::Armor(armor_idx) => {
                            std::mem::swap(
                                &mut self.player.inventory.hotbar[i],
                                &mut self.player.inventory.armor[armor_idx],
                            );
                        }
                        InventorySlotTarget::Offhand => {
                            std::mem::swap(
                                &mut self.player.inventory.hotbar[i],
                                &mut self.player.inventory.offhand,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        let is_right_active = right_down || input.gp_btn_west;
        let click_right = (!self.prev_right_down && right_down) || input.gp_west_just_pressed;

        if !is_right_active {
            self.right_drag_visited.clear();
        }

        if is_right_active {
            if let Some(target) = inv_hovered_target {
                if matches!(
                    target,
                    InventorySlotTarget::Hotbar(_)
                        | InventorySlotTarget::Main(_)
                        | InventorySlotTarget::Crafting2x2(_)
                        | InventorySlotTarget::Crafting3x3(_)
                        | InventorySlotTarget::Armor(_)
                        | InventorySlotTarget::Offhand
                ) && !self.right_drag_visited.contains(&target)
                {
                    self.right_drag_visited.push(target);
                    match target {
                        InventorySlotTarget::Hotbar(idx) => {
                            if click_right && self.player.inventory.held_item.is_none() {
                                if self.player.inventory.try_auto_equip_hotbar(idx) {
                                    self.audio.play_sound(
                                        crate::audio::SoundId::ArmorEquip,
                                        0.85,
                                        1.0,
                                    );
                                } else {
                                    let slot = &mut self.player.inventory.hotbar[idx];
                                    Inventory::handle_slot_right_click(
                                        slot,
                                        &mut self.player.inventory.held_item,
                                    );
                                }
                            } else {
                                let slot = &mut self.player.inventory.hotbar[idx];
                                Inventory::handle_slot_right_drag(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        }
                        InventorySlotTarget::Main(idx) => {
                            if click_right && self.player.inventory.held_item.is_none() {
                                if self.player.inventory.try_auto_equip_main(idx) {
                                    self.audio.play_sound(
                                        crate::audio::SoundId::ArmorEquip,
                                        0.85,
                                        1.0,
                                    );
                                } else {
                                    let slot = &mut self.player.inventory.main[idx];
                                    Inventory::handle_slot_right_click(
                                        slot,
                                        &mut self.player.inventory.held_item,
                                    );
                                }
                            } else {
                                let slot = &mut self.player.inventory.main[idx];
                                Inventory::handle_slot_right_drag(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        }
                        InventorySlotTarget::Crafting2x2(idx) => {
                            let slot = &mut self.player.inventory.crafting_2x2[idx];
                            if click_right && self.player.inventory.held_item.is_none() {
                                Inventory::handle_slot_right_click(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            } else {
                                Inventory::handle_slot_right_drag(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        }
                        InventorySlotTarget::Crafting3x3(idx) => {
                            let slot = &mut self.player.inventory.crafting_3x3[idx];
                            if click_right && self.player.inventory.held_item.is_none() {
                                Inventory::handle_slot_right_click(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            } else {
                                Inventory::handle_slot_right_drag(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        }
                        InventorySlotTarget::Armor(armor_idx) => {
                            let slot = &mut self.player.inventory.armor[armor_idx];
                            if click_right
                                && self.player.inventory.held_item.is_none()
                                && slot.is_some()
                            {
                                self.player.inventory.held_item = slot.take();
                                self.audio.play_sound(
                                    crate::audio::SoundId::ArmorEquip,
                                    0.85,
                                    1.0,
                                );
                            }
                        }
                        InventorySlotTarget::Offhand => {
                            let slot = &mut self.player.inventory.offhand;
                            if click_right && self.player.inventory.held_item.is_none() {
                                Inventory::handle_slot_right_click(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            } else {
                                Inventory::handle_slot_right_drag(
                                    slot,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if click_left
            || (click_right
                && inv_hovered_target.is_some_and(|t| {
                    matches!(
                        t,
                        InventorySlotTarget::CreativeTab(_)
                            | InventorySlotTarget::CreativeScrollUp
                            | InventorySlotTarget::CreativeScrollDown
                            | InventorySlotTarget::Creative(_)
                            | InventorySlotTarget::Trash
                            | InventorySlotTarget::CraftingOutput
                    )
                }))
        {
            if let Some(target) = inv_hovered_target {
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                self.handle_inventory_primary_click(target, click_left, input.key_shift);
            }
        }

        self.render_menu_world_backdrop();
        self.renderer.draw_inventory(
            &self.player,
            inv_hovered_target,
            self.crafting_table_open,
            &self.recipe_book,
            &unlocked_recipe_indices,
            self.recipe_book_open,
            self.recipe_book_scroll,
            self.recipe_book_status,
            mouse_pos,
            self.is_gamepad_active,
        );
    }

    fn handle_inventory_primary_click(
        &mut self,
        target: InventorySlotTarget,
        click_left: bool,
        shift_held: bool,
    ) {
        match target {
            InventorySlotTarget::CreativeTab(tab_idx) => {
                self.player.creative_tab = tab_idx;
                self.player.creative_scroll = 0;
            }
            InventorySlotTarget::CreativeScrollUp => {
                self.player.creative_scroll = self.player.creative_scroll.saturating_sub(1);
            }
            InventorySlotTarget::CreativeScrollDown => {
                let blocks =
                    crate::ui::inventory_ui::creative_tab_blocks(self.player.creative_tab);
                let total_rows = (blocks.len() + 8) / 9;
                let max_scroll = total_rows.saturating_sub(4);
                self.player.creative_scroll = (self.player.creative_scroll + 1).min(max_scroll);
            }
            InventorySlotTarget::RecipeBookToggle => {
                self.recipe_book_open = !self.recipe_book_open;
                self.recipe_book_status = None;
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            }
            InventorySlotTarget::RecipeEntry(recipe_index) => {
                self.load_recipe_into_3x3(recipe_index);
            }
            InventorySlotTarget::Creative(id) => {
                let block = BlockId::from_u8(id);
                if shift_held {
                    let placed = if let Some(slot) =
                        self.player.inventory.hotbar.iter_mut().find(|s| s.is_none())
                    {
                        *slot = Some(ItemStack::new(block, 64));
                        true
                    } else {
                        false
                    };
                    if !placed {
                        self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] =
                            Some(ItemStack::new(block, 64));
                    }
                } else {
                    self.player.inventory.held_item = Some(ItemStack::new(block, 64));
                }
            }
            InventorySlotTarget::Trash => {
                if shift_held {
                    self.player.inventory.hotbar.fill(None);
                }
                self.player.inventory.held_item = None;
            }
            InventorySlotTarget::Hotbar(idx) => {
                if click_left {
                    Inventory::handle_slot_left_click(
                        &mut self.player.inventory.hotbar[idx],
                        &mut self.player.inventory.held_item,
                    );
                }
            }
            InventorySlotTarget::Main(idx) => {
                if click_left {
                    Inventory::handle_slot_left_click(
                        &mut self.player.inventory.main[idx],
                        &mut self.player.inventory.held_item,
                    );
                }
            }
            InventorySlotTarget::Crafting2x2(idx) => {
                if click_left {
                    Inventory::handle_slot_left_click(
                        &mut self.player.inventory.crafting_2x2[idx],
                        &mut self.player.inventory.held_item,
                    );
                }
            }
            InventorySlotTarget::Crafting3x3(idx) => {
                if click_left {
                    Inventory::handle_slot_left_click(
                        &mut self.player.inventory.crafting_3x3[idx],
                        &mut self.player.inventory.held_item,
                    );
                }
            }
            InventorySlotTarget::Armor(armor_idx) => {
                let slot = &mut self.player.inventory.armor[armor_idx];
                if let Some(held) = self.player.inventory.held_item {
                    let expected_slot = ArmorSlot::from_index(armor_idx);
                    if held.block.armor_slot() == Some(expected_slot) {
                        std::mem::swap(slot, &mut self.player.inventory.held_item);
                        self.audio
                            .play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                    }
                } else if slot.is_some() {
                    self.player.inventory.held_item = slot.take();
                    self.audio
                        .play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                }
            }
            InventorySlotTarget::Offhand => {
                if click_left {
                    Inventory::handle_slot_left_click(
                        &mut self.player.inventory.offhand,
                        &mut self.player.inventory.held_item,
                    );
                }
            }
            InventorySlotTarget::CraftingOutput => {
                self.take_crafting_output();
            }
        }
    }

    fn load_recipe_into_3x3(&mut self, recipe_index: usize) {
        self.recipe_book_status = None;
        let Some(recipe) = self.recipe_book.recipes().get(recipe_index).cloned() else {
            crate::log_error!(
                "CRAFTING",
                "Recipe book referenced missing recipe {}",
                recipe_index
            );
            self.recipe_book_status = Some("Recipe unavailable");
            return;
        };
        let grid_is_empty = self.player.inventory.crafting_3x3.iter().all(Option::is_none)
            && self.player.inventory.held_item.is_none();
        if !grid_is_empty {
            self.recipe_book_status = Some("Clear grid and hand first");
            return;
        }
        if recipe.width == 0
            || recipe.height == 0
            || recipe.width > 3
            || recipe.height > 3
            || recipe.pattern.len() != recipe.width * recipe.height
        {
            crate::log_error!(
                "CRAFTING",
                "Recipe {} does not fit the 3x3 table",
                recipe_index
            );
            self.recipe_book_status = Some("Recipe does not fit");
            return;
        }
        let ingredients_available = recipe.pattern.iter().flatten().all(|&ingredient| {
            let required = recipe
                .pattern
                .iter()
                .filter(|slot| **slot == Some(ingredient))
                .count() as u16;
            self.player.inventory.count_item(ingredient) >= required
        });
        if !ingredients_available {
            self.recipe_book_status = Some("Missing ingredients");
            return;
        }
        for ingredient in recipe.pattern.iter().flatten() {
            if !self.player.inventory.consume_item(*ingredient) {
                crate::log_error!(
                    "CRAFTING",
                    "Could not consume verified recipe ingredient {:?}",
                    ingredient
                );
                self.recipe_book_status = Some("Could not load recipe");
                return;
            }
        }
        for (index, ingredient) in recipe.pattern.iter().enumerate() {
            let row = index / recipe.width;
            let col = index % recipe.width;
            self.player.inventory.crafting_3x3[row * 3 + col] =
                ingredient.map(|block| ItemStack::new(block, 1));
        }
        self.recipe_book_open = false;
    }

    fn take_crafting_output(&mut self) {
        let result = if self.crafting_table_open {
            self.recipe_book
                .match_grid(&self.player.inventory.crafting_3x3, 3, 3)
        } else {
            self.recipe_book
                .match_grid(&self.player.inventory.crafting_2x2, 2, 2)
        };
        let Some(res) = result else { return };
        let can_take = match self.player.inventory.held_item {
            None => true,
            Some(h) => h.block == res.block && h.count + res.count <= 64,
        };
        if !can_take {
            return;
        }
        crate::log_info!("CRAFTING", "Crafted {}x {:?}", res.count, res.block);
        let unlocked = self.achievements.record_crafted(res.count);
        self.process_achievement_unlocks(unlocked);
        let q_craft = self.quests.record_crafted(res.block, res.count);
        self.process_quest_completions(q_craft);
        self.record_collected_item(res.block);
        if let Some(h) = &mut self.player.inventory.held_item {
            h.count += res.count;
        } else {
            self.player.inventory.held_item = Some(res);
        }
        if self.crafting_table_open {
            RecipeBook::consume_grid(&mut self.player.inventory.crafting_3x3);
        } else {
            RecipeBook::consume_grid(&mut self.player.inventory.crafting_2x2);
        }
    }
}
