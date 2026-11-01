use crate::core::controls::Action;
use crate::core::math::IVec3;
use crate::gameplay::inventory::{Inventory, ItemStack};
use crate::gameplay::player::GameMode;
use crate::gameplay::recipe::RecipeBook;
use crate::ui::{
    draw_chest_ui, draw_furnace_ui, draw_villager_trade_ui, ChestLayout, ChestSlotTarget, FurnaceLayout, FurnaceSlotTarget,
    InventoryLayout, InventorySlotTarget, VillagerTradeLayout, VillagerTradeSlotTarget,
};
use crate::gameplay::trading::VillagerProfession;
use crate::world::block::BlockId;
use super::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
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
        } else if self.player.gamemode == GameMode::Creative && self.player.creative_tab < crate::ui::CREATIVE_TABS.len() - 1 {
            let blocks = crate::ui::inventory_ui::creative_tab_blocks(self.player.creative_tab);
            let total_rows = (blocks.len() + 8) / 9;
            let max_scroll = total_rows.saturating_sub(4);
            if input.scroll_delta < 0.0 {
                self.player.creative_scroll = (self.player.creative_scroll + 1).min(max_scroll);
            } else if input.scroll_delta > 0.0 {
                self.player.creative_scroll = self.player.creative_scroll.saturating_sub(1);
            }
        } else {
            if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
                self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
            }
            if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
                self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
            }
        }

        if self.controls.is_just_pressed(Action::Inventory)
            || self.controls.is_just_pressed(Action::Pause)
            || input.gp_east_just_pressed
        {
            self.player.inventory.return_crafting_items();
            if self.player.gamemode == GameMode::Creative {
                self.player.inventory.held_item = None;
            }
            self.crafting_table_open = false;
            self.recipe_book_open = false;
            self.recipe_book_status = None;
            self.state = GameState::Playing;
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
            out.request_capture_mouse = true;
            crate::log_info!("MENU", "Closed inventory screen, returning to Playing");
        }

        let layout = InventoryLayout::new(self.renderer.width, self.renderer.height, self.crafting_table_open);
        let mut inv_hovered_target = if self.is_gamepad_active && !mouse_moved {
            None
        } else {
            mouse_pos.and_then(|(mx, my)| {
                layout.find_slot_at(mx as i32, my as i32, &self.player)
            })
        };
        if self.crafting_table_open && self.player.gamemode != GameMode::Creative
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

        // Number keys 1-9 (indices 1..9) and 0 (index 0) to assign/swap into hotbar slots
        for i in 0..10 {
            if input.key_digits[i] && !self.prev_key_digits[i] {
                if let Some(target) = inv_hovered_target {
                    match target {
                        InventorySlotTarget::Creative(id) => {
                            self.player.inventory.hotbar[i] = Some(ItemStack::new(BlockId::from_u8(id), 64));
                        }
                        InventorySlotTarget::Hotbar(from_idx) => {
                            self.player.inventory.hotbar.swap(i, from_idx);
                        }
                        InventorySlotTarget::Main(from_idx) => {
                            std::mem::swap(&mut self.player.inventory.hotbar[i], &mut self.player.inventory.main[from_idx]);
                        }
                        InventorySlotTarget::Armor(armor_idx) => {
                            std::mem::swap(&mut self.player.inventory.hotbar[i], &mut self.player.inventory.armor[armor_idx]);
                        }
                        InventorySlotTarget::Offhand => {
                            std::mem::swap(&mut self.player.inventory.hotbar[i], &mut self.player.inventory.offhand);
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

        // Handle right-click dragging across slots to distribute 1 item per slot (like Minecraft)
        if is_right_active {
            if let Some(target) = inv_hovered_target {
                if matches!(target, InventorySlotTarget::Hotbar(_) | InventorySlotTarget::Main(_) | InventorySlotTarget::Crafting2x2(_) | InventorySlotTarget::Crafting3x3(_) | InventorySlotTarget::Armor(_) | InventorySlotTarget::Offhand) {
                    if !self.right_drag_visited.contains(&target) {
                        self.right_drag_visited.push(target);
                        match target {
                            InventorySlotTarget::Hotbar(idx) => {
                                if click_right && self.player.inventory.held_item.is_none() {
                                    if self.player.inventory.try_auto_equip_hotbar(idx) {
                                        self.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                                    } else {
                                        let slot = &mut self.player.inventory.hotbar[idx];
                                        Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                    }
                                } else {
                                    let slot = &mut self.player.inventory.hotbar[idx];
                                    Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Main(idx) => {
                                if click_right && self.player.inventory.held_item.is_none() {
                                    if self.player.inventory.try_auto_equip_main(idx) {
                                        self.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                                    } else {
                                        let slot = &mut self.player.inventory.main[idx];
                                        Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                    }
                                } else {
                                    let slot = &mut self.player.inventory.main[idx];
                                    Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Crafting2x2(idx) => {
                                let slot = &mut self.player.inventory.crafting_2x2[idx];
                                if click_right && self.player.inventory.held_item.is_none() {
                                    Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                } else {
                                    Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Crafting3x3(idx) => {
                                let slot = &mut self.player.inventory.crafting_3x3[idx];
                                if click_right && self.player.inventory.held_item.is_none() {
                                    Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                } else {
                                    Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            InventorySlotTarget::Armor(armor_idx) => {
                                let slot = &mut self.player.inventory.armor[armor_idx];
                                if click_right && self.player.inventory.held_item.is_none() && slot.is_some() {
                                    self.player.inventory.held_item = slot.take();
                                    self.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                                }
                            }
                            InventorySlotTarget::Offhand => {
                                let slot = &mut self.player.inventory.offhand;
                                if click_right && self.player.inventory.held_item.is_none() {
                                    Inventory::handle_slot_right_click(slot, &mut self.player.inventory.held_item);
                                } else {
                                    Inventory::handle_slot_right_drag(slot, &mut self.player.inventory.held_item);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if click_left || (click_right && inv_hovered_target.map_or(false, |t| matches!(t, InventorySlotTarget::CreativeTab(_) | InventorySlotTarget::CreativeScrollUp | InventorySlotTarget::CreativeScrollDown | InventorySlotTarget::Creative(_) | InventorySlotTarget::Trash | InventorySlotTarget::CraftingOutput))) {
            if let Some(target) = inv_hovered_target {
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                match target {
                    InventorySlotTarget::CreativeTab(tab_idx) => {
                        self.player.creative_tab = tab_idx;
                        self.player.creative_scroll = 0;
                    }
                    InventorySlotTarget::CreativeScrollUp => {
                        self.player.creative_scroll = self.player.creative_scroll.saturating_sub(1);
                    }
                    InventorySlotTarget::CreativeScrollDown => {
                        let blocks = crate::ui::inventory_ui::creative_tab_blocks(self.player.creative_tab);
                        let total_rows = (blocks.len() + 8) / 9;
                        let max_scroll = total_rows.saturating_sub(4);
                        self.player.creative_scroll = (self.player.creative_scroll + 1).min(max_scroll);
                    }
                    InventorySlotTarget::RecipeBookToggle => {
                        self.recipe_book_open = !self.recipe_book_open;
                        self.recipe_book_status = None;
                        self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                    }
                    InventorySlotTarget::RecipeEntry(recipe_index) => {
                        self.recipe_book_status = None;
                        if let Some(recipe) = self.recipe_book.recipes().get(recipe_index).cloned() {
                            let grid_is_empty = self.player.inventory.crafting_3x3.iter().all(Option::is_none)
                                && self.player.inventory.held_item.is_none();
                            if !grid_is_empty {
                                self.recipe_book_status = Some("Clear grid and hand first");
                            } else if recipe.width == 0
                                || recipe.height == 0
                                || recipe.width > 3
                                || recipe.height > 3
                                || recipe.pattern.len() != recipe.width * recipe.height
                            {
                                crate::log_error!("CRAFTING", "Recipe {} does not fit the 3x3 table", recipe_index);
                                self.recipe_book_status = Some("Recipe does not fit");
                            } else {
                                let ingredients_available = recipe.pattern.iter().flatten().all(|&ingredient| {
                                    let required = recipe.pattern.iter().filter(|slot| **slot == Some(ingredient)).count() as u16;
                                    self.player.inventory.count_item(ingredient) >= required
                                });
                                if !ingredients_available {
                                    self.recipe_book_status = Some("Missing ingredients");
                                } else {
                                    let mut consumed_all = true;
                                    for ingredient in recipe.pattern.iter().flatten() {
                                        if !self.player.inventory.consume_item(*ingredient) {
                                            crate::log_error!("CRAFTING", "Could not consume verified recipe ingredient {:?}", ingredient);
                                            consumed_all = false;
                                            break;
                                        }
                                    }
                                    if consumed_all {
                                        for (index, ingredient) in recipe.pattern.iter().enumerate() {
                                            let row = index / recipe.width;
                                            let col = index % recipe.width;
                                            self.player.inventory.crafting_3x3[row * 3 + col] =
                                                ingredient.map(|block| ItemStack::new(block, 1));
                                        }
                                        self.recipe_book_open = false;
                                    } else {
                                        self.recipe_book_status = Some("Could not load recipe");
                                    }
                                }
                            }
                        } else {
                            crate::log_error!("CRAFTING", "Recipe book referenced missing recipe {}", recipe_index);
                            self.recipe_book_status = Some("Recipe unavailable");
                        }
                    }
                    InventorySlotTarget::Creative(id) => {
                        let block = BlockId::from_u8(id);
                        if input.key_shift {
                            // Shift-click places 64-stack into first empty hotbar slot
                            let placed = if let Some(slot) = self.player.inventory.hotbar.iter_mut().find(|s| s.is_none()) {
                                *slot = Some(ItemStack::new(block, 64));
                                true
                            } else {
                                false
                            };
                            if !placed {
                                self.player.inventory.hotbar[self.player.selected_hotbar_slot as usize] = Some(ItemStack::new(block, 64));
                            }
                        } else {
                            // Normal click: pick up full 64-stack into hand
                            self.player.inventory.held_item = Some(ItemStack::new(block, 64));
                        }
                    }
                    InventorySlotTarget::Trash => {
                        if input.key_shift {
                            // Shift-click on trash clears all hotbar slots
                            self.player.inventory.hotbar.fill(None);
                            self.player.inventory.held_item = None;
                        } else {
                            self.player.inventory.held_item = None;
                        }
                    }
                    InventorySlotTarget::Hotbar(idx) => {
                        let slot = &mut self.player.inventory.hotbar[idx];
                        if click_left {
                            Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                        }
                    }
                    InventorySlotTarget::Main(idx) => {
                        let slot = &mut self.player.inventory.main[idx];
                        if click_left {
                            Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                        }
                    }
                    InventorySlotTarget::Crafting2x2(idx) => {
                        let slot = &mut self.player.inventory.crafting_2x2[idx];
                        if click_left {
                            Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                        }
                    }
                    InventorySlotTarget::Crafting3x3(idx) => {
                        let slot = &mut self.player.inventory.crafting_3x3[idx];
                        if click_left {
                            Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                        }
                    }
                    InventorySlotTarget::Armor(armor_idx) => {
                        let slot = &mut self.player.inventory.armor[armor_idx];
                        if let Some(held) = self.player.inventory.held_item {
                            let expected_slot = crate::world::block::ArmorSlot::from_index(armor_idx);
                            if held.block.armor_slot() == Some(expected_slot) {
                                std::mem::swap(slot, &mut self.player.inventory.held_item);
                                self.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                            }
                        } else if slot.is_some() {
                            self.player.inventory.held_item = slot.take();
                            self.audio.play_sound(crate::audio::SoundId::ArmorEquip, 0.85, 1.0);
                        }
                    }
                    InventorySlotTarget::Offhand => {
                        let slot = &mut self.player.inventory.offhand;
                        if click_left {
                            Inventory::handle_slot_left_click(slot, &mut self.player.inventory.held_item);
                        }
                    }
                    InventorySlotTarget::CraftingOutput => {
                        let result = if self.crafting_table_open {
                            self.recipe_book.match_grid(&self.player.inventory.crafting_3x3, 3, 3)
                        } else {
                            self.recipe_book.match_grid(&self.player.inventory.crafting_2x2, 2, 2)
                        };
                        if let Some(res) = result {
                            let can_take = match self.player.inventory.held_item {
                                None => true,
                                Some(h) => h.block == res.block && h.count + res.count <= 64,
                            };
                            if can_take {
                                crate::log_info!("CRAFTING", "Crafted {}x {:?}", res.count, res.block);
                                let unlocked = self.achievements.record_crafted(res.count);
                                self.process_achievement_unlocks(unlocked);
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
                    }
                }
            }
        }

        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
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

    pub(crate) fn update_and_render_chest(
        &mut self,
        dt: f32,
        input: &InputState,
        pos: IVec3,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let right_down = input.mouse_right;
        let chest_pos = pos;

        self.world.update_furnaces(dt);

        if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
            self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
        }
        if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
            self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
        }

        if self.controls.is_just_pressed(Action::Inventory)
            || self.controls.is_just_pressed(Action::Pause)
            || input.gp_east_just_pressed
        {
            if self.player.gamemode == GameMode::Creative {
                self.player.inventory.held_item = None;
            }
            crate::log_info!("MENU", "Closed chest container, returning to Playing");
            self.state = GameState::Playing;
            self.audio.play_sound(crate::audio::SoundId::ChestClose, 0.8, 1.0);
            out.request_capture_mouse = true;
        }

        let layout = ChestLayout::new(self.renderer.width, self.renderer.height);
        let hovered_target = mouse_pos.and_then(|(mx, my)| {
            layout.find_slot_at(mx as i32, my as i32)
        });

        let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        let click_right = (!self.prev_right_down && right_down) || input.gp_west_just_pressed;

        for i in 0..10 {
            if input.key_digits[i] && !self.prev_key_digits[i] {
                if let Some(target) = hovered_target {
                    let chest = self.world.get_or_create_chest(chest_pos);
                    match target {
                        ChestSlotTarget::Chest(idx) => {
                            std::mem::swap(&mut chest.items[idx], &mut self.player.inventory.hotbar[i]);
                        }
                        ChestSlotTarget::Main(idx) => {
                            std::mem::swap(&mut self.player.inventory.main[idx], &mut self.player.inventory.hotbar[i]);
                        }
                        ChestSlotTarget::Hotbar(idx) => {
                            self.player.inventory.hotbar.swap(i, idx);
                        }
                    }
                }
            }
        }

        if click_left || click_right {
            let chest = self.world.get_or_create_chest(chest_pos);
            if let Some(target) = hovered_target {
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                match target {
                    ChestSlotTarget::Chest(idx) => {
                        if click_left {
                            if input.key_shift {
                                if let Some(st) = chest.items[idx].take() {
                                    let rem = self.player.inventory.add_item(st.block, st.count);
                                    if rem > 0 {
                                        chest.items[idx] = Some(ItemStack::new(st.block, rem));
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut chest.items[idx], &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut chest.items[idx], &mut self.player.inventory.held_item);
                        }
                    }
                    ChestSlotTarget::Main(idx) => {
                        if click_left {
                            if input.key_shift {
                                if let Some(st) = self.player.inventory.main[idx].take() {
                                    let rem = chest.add_item(st.block, st.count);
                                    if rem > 0 {
                                        self.player.inventory.main[idx] = Some(ItemStack::new(st.block, rem));
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut self.player.inventory.main[idx], &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut self.player.inventory.main[idx], &mut self.player.inventory.held_item);
                        }
                    }
                    ChestSlotTarget::Hotbar(idx) => {
                        if click_left {
                            if input.key_shift {
                                if let Some(st) = self.player.inventory.hotbar[idx].take() {
                                    let rem = chest.add_item(st.block, st.count);
                                    if rem > 0 {
                                        self.player.inventory.hotbar[idx] = Some(ItemStack::new(st.block, rem));
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut self.player.inventory.hotbar[idx], &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut self.player.inventory.hotbar[idx], &mut self.player.inventory.held_item);
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
        let chest = self.world.get_or_create_chest(chest_pos);
        draw_chest_ui(
            &mut self.renderer.color_buffer,
            self.renderer.width,
            self.renderer.height,
            chest,
            &self.player,
            mouse_pos,
        );
    }

    pub(crate) fn update_and_render_furnace(
        &mut self,
        dt: f32,
        input: &InputState,
        pos: IVec3,
        out: &mut AppOutput,
    ) {
        let mouse_pos = input.mouse_pos;
        let left_down = input.mouse_left;
        let right_down = input.mouse_right;
        let furnace_pos = pos;

        self.world.update_furnaces(dt);

        if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
            self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot <= 0 { 9 } else { self.player.selected_hotbar_slot - 1 };
        }
        if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
            self.player.selected_hotbar_slot = if self.player.selected_hotbar_slot >= 9 { 0 } else { self.player.selected_hotbar_slot + 1 };
        }

        if self.controls.is_just_pressed(Action::Inventory)
            || self.controls.is_just_pressed(Action::Pause)
            || input.gp_east_just_pressed
        {
            if self.player.gamemode == GameMode::Creative {
                self.player.inventory.held_item = None;
            }
            crate::log_info!("MENU", "Closed furnace container, returning to Playing");
            self.state = GameState::Playing;
            out.request_capture_mouse = true;
        }

        let layout = FurnaceLayout::new(self.renderer.width, self.renderer.height);
        let hovered_target = mouse_pos.and_then(|(mx, my)| {
            layout.find_slot_at(mx as i32, my as i32)
        });

        let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        let click_right = (!self.prev_right_down && right_down) || input.gp_west_just_pressed;

        for i in 0..10 {
            if input.key_digits[i] && !self.prev_key_digits[i] {
                if let Some(target) = hovered_target {
                    let furnace = self.world.get_or_create_furnace(furnace_pos);
                    match target {
                        FurnaceSlotTarget::Input => {
                            std::mem::swap(&mut furnace.input, &mut self.player.inventory.hotbar[i]);
                        }
                        FurnaceSlotTarget::Fuel => {
                            std::mem::swap(&mut furnace.fuel, &mut self.player.inventory.hotbar[i]);
                        }
                        FurnaceSlotTarget::Output => {
                            if let Some(out_stack) = furnace.output.take() {
                                let rem = self.player.inventory.add_item(out_stack.block, out_stack.count);
                                if rem > 0 {
                                    furnace.output = Some(ItemStack::new(out_stack.block, rem));
                                }
                            }
                        }
                        FurnaceSlotTarget::Main(idx) => {
                            std::mem::swap(&mut self.player.inventory.main[idx], &mut self.player.inventory.hotbar[i]);
                        }
                        FurnaceSlotTarget::Hotbar(idx) => {
                            self.player.inventory.hotbar.swap(i, idx);
                        }
                    }
                }
            }
        }

        if click_left || click_right {
            let furnace = self.world.get_or_create_furnace(furnace_pos);
            if let Some(target) = hovered_target {
                self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
                match target {
                    FurnaceSlotTarget::Input => {
                        if click_left {
                            if input.key_shift {
                                if let Some(st) = furnace.input.take() {
                                    let rem = self.player.inventory.add_item(st.block, st.count);
                                    if rem > 0 {
                                        furnace.input = Some(ItemStack::new(st.block, rem));
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut furnace.input, &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut furnace.input, &mut self.player.inventory.held_item);
                        }
                    }
                    FurnaceSlotTarget::Fuel => {
                        if click_left {
                            if input.key_shift {
                                if let Some(st) = furnace.fuel.take() {
                                    let rem = self.player.inventory.add_item(st.block, st.count);
                                    if rem > 0 {
                                        furnace.fuel = Some(ItemStack::new(st.block, rem));
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut furnace.fuel, &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut furnace.fuel, &mut self.player.inventory.held_item);
                        }
                    }
                    FurnaceSlotTarget::Output => {
                        if click_left {
                            if input.key_shift {
                                if let Some(out_stack) = furnace.output.take() {
                                    let rem = self.player.inventory.add_item(out_stack.block, out_stack.count);
                                    if rem > 0 {
                                        furnace.output = Some(ItemStack::new(out_stack.block, rem));
                                    }
                                }
                            } else if let Some(out_stack) = &mut furnace.output {
                                if let Some(held) = &mut self.player.inventory.held_item {
                                    if held.block == out_stack.block && held.count + out_stack.count <= 64 {
                                        held.count += out_stack.count;
                                        furnace.output = None;
                                    }
                                } else {
                                    self.player.inventory.held_item = furnace.output.take();
                                }
                            }
                        }
                    }
                    FurnaceSlotTarget::Main(idx) => {
                        if click_left {
                            if input.key_shift {
                                if let Some(mut item) = self.player.inventory.main[idx].take() {
                                    if crate::world::tile_entity::fuel_burn_time(item.block).is_some() {
                                        if let Some(fuel) = &mut furnace.fuel {
                                            if fuel.block == item.block && fuel.count < 64 {
                                                let to_add = item.count.min(64 - fuel.count);
                                                fuel.count += to_add;
                                                item.count -= to_add;
                                            }
                                        } else {
                                            furnace.fuel = Some(item);
                                            item.count = 0;
                                        }
                                    }
                                    if item.count > 0 {
                                        if let Some(inp) = &mut furnace.input {
                                            if inp.block == item.block && inp.count < 64 {
                                                let to_add = item.count.min(64 - inp.count);
                                                inp.count += to_add;
                                                item.count -= to_add;
                                            }
                                        } else if furnace.input.is_none() {
                                            furnace.input = Some(item);
                                            item.count = 0;
                                        }
                                    }
                                    if item.count > 0 {
                                        self.player.inventory.main[idx] = Some(item);
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut self.player.inventory.main[idx], &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut self.player.inventory.main[idx], &mut self.player.inventory.held_item);
                        }
                    }
                    FurnaceSlotTarget::Hotbar(idx) => {
                        if click_left {
                            if input.key_shift {
                                if let Some(mut item) = self.player.inventory.hotbar[idx].take() {
                                    if crate::world::tile_entity::fuel_burn_time(item.block).is_some() {
                                        if let Some(fuel) = &mut furnace.fuel {
                                            if fuel.block == item.block && fuel.count < 64 {
                                                let to_add = item.count.min(64 - fuel.count);
                                                fuel.count += to_add;
                                                item.count -= to_add;
                                            }
                                        } else {
                                            furnace.fuel = Some(item);
                                            item.count = 0;
                                        }
                                    }
                                    if item.count > 0 {
                                        if let Some(inp) = &mut furnace.input {
                                            if inp.block == item.block && inp.count < 64 {
                                                let to_add = item.count.min(64 - inp.count);
                                                inp.count += to_add;
                                                item.count -= to_add;
                                            }
                                        } else if furnace.input.is_none() {
                                            furnace.input = Some(item);
                                            item.count = 0;
                                        }
                                    }
                                    if item.count > 0 {
                                        self.player.inventory.hotbar[idx] = Some(item);
                                    }
                                }
                            } else {
                                Inventory::handle_slot_left_click(&mut self.player.inventory.hotbar[idx], &mut self.player.inventory.held_item);
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(&mut self.player.inventory.hotbar[idx], &mut self.player.inventory.held_item);
                        }
                    }
                }
            }
        }

        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }
        let furnace = self.world.get_or_create_furnace(furnace_pos);
        draw_furnace_ui(
            &mut self.renderer.color_buffer,
            self.renderer.width,
            self.renderer.height,
            furnace,
            &self.player,
            mouse_pos,
        );
    }

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

        // Close trading window
        if self.controls.is_just_pressed(Action::Inventory)
            || self.controls.is_just_pressed(Action::Pause)
            || input.gp_east_just_pressed
        {
            self.state = GameState::Playing;
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
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

        // Trade selection navigation (mouse wheel or hotbar prev/next)
        if input.scroll_delta < 0.0 || self.controls.is_just_pressed(Action::HotbarNext) {
            selected_trade = (selected_trade + 1) % trade_count;
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.5, 1.2);
        } else if input.scroll_delta > 0.0 || self.controls.is_just_pressed(Action::HotbarPrev) {
            selected_trade = if selected_trade == 0 { trade_count - 1 } else { selected_trade - 1 };
            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.5, 1.2);
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
                            self.audio.play_sound(crate::audio::SoundId::UiClick, 0.6, 1.1);
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
                        self.audio.play_sound(crate::audio::SoundId::VillagerTrade, 1.0, 1.0);
                        self.particles.spawn_growth(mob_pos);
                    } else {
                        self.audio.play_sound(crate::audio::SoundId::VillagerNo, 1.0, 1.0);
                    }
                }
            }
        }

        self.state = GameState::VillagerTrade { mob_id, selected_trade };
        self.prev_left_down = left_down;

        #[cfg(not(target_os = "vita"))]
        {
            self.renderer.render_frame(&mut self.world, &self.camera);
            self.renderer.apply_biome_color_filter(self.biome_filter_tint);
        }
        #[cfg(target_os = "vita")]
        {
            self.renderer.color_buffer.fill(0x0C0C10);
        }

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
