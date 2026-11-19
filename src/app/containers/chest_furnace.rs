//! Interakcja z kontenerami skrzyni (`Chest`) i pieca (`Furnace`) w `RustCraftApp`.

use crate::core::math::IVec3;
use crate::features::blocks::BlockId;
use crate::features::player::{GameMode, Inventory, ItemStack};
use crate::features::ui::{
    draw_chest_ui, draw_furnace_ui, ChestLayout, ChestSlotTarget, FurnaceLayout, FurnaceSlotTarget,
};
use crate::features::world::{fuel_burn_time, ChestData, FurnaceData};

use crate::app::{AppOutput, GameState, InputState, RustCraftApp};

impl RustCraftApp {
    fn handle_chest_player_slot_click(
        slot: &mut Option<ItemStack>,
        held_item: &mut Option<ItemStack>,
        chest: &mut ChestData,
        click_left: bool,
        click_right: bool,
        shift_held: bool,
    ) {
        if click_left {
            if shift_held {
                if let Some(st) = slot.take() {
                    let rem = chest.add_item(st.block, st.count);
                    if rem > 0 {
                        *slot = Some(ItemStack::new(st.block, rem));
                    }
                }
            } else {
                Inventory::handle_slot_left_click(slot, held_item);
            }
        } else if click_right {
            Inventory::handle_slot_right_click(slot, held_item);
        }
    }

    fn handle_furnace_player_slot_click(
        slot: &mut Option<ItemStack>,
        held_item: &mut Option<ItemStack>,
        furnace: &mut FurnaceData,
        click_left: bool,
        click_right: bool,
        shift_held: bool,
    ) {
        if click_left {
            if shift_held {
                if let Some(mut item) = slot.take() {
                    if fuel_burn_time(item.block).is_some() {
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
                        *slot = Some(item);
                    }
                }
            } else {
                Inventory::handle_slot_left_click(slot, held_item);
            }
        } else if click_right {
            Inventory::handle_slot_right_click(slot, held_item);
        }
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

        self.world.update_furnaces(dt);
        self.cycle_hotbar_scroll(input);

        if self.is_close_container_pressed(input) {
            if self.player.gamemode == GameMode::Creative {
                self.player.inventory.held_item = None;
            }
            crate::log_info!("MENU", "Closed chest container, returning to Playing");
            self.state = GameState::Playing;
            self.audio
                .play_sound(crate::audio::SoundId::ChestClose, 0.8, 1.0);
            out.request_capture_mouse = true;
        }

        let layout = ChestLayout::new(self.renderer.width, self.renderer.height);
        let hovered_target = mouse_pos.and_then(|(mx, my)| layout.find_slot_at(mx as i32, my as i32));

        let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        let click_right = (!self.prev_right_down && right_down) || input.gp_west_just_pressed;

        for i in 0..10 {
            if input.key_digits[i] && !self.prev_key_digits[i] {
                if let Some(target) = hovered_target {
                    let chest = self.world.get_or_create_chest(pos);
                    match target {
                        ChestSlotTarget::Chest(idx) => {
                            std::mem::swap(&mut chest.items[idx], &mut self.player.inventory.hotbar[i]);
                        }
                        ChestSlotTarget::Main(idx) => {
                            std::mem::swap(
                                &mut self.player.inventory.main[idx],
                                &mut self.player.inventory.hotbar[i],
                            );
                        }
                        ChestSlotTarget::Hotbar(idx) => {
                            self.player.inventory.hotbar.swap(i, idx);
                        }
                    }
                }
            }
        }

        if click_left || click_right {
            let chest = self.world.get_or_create_chest(pos);
            if let Some(target) = hovered_target {
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
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
                                Inventory::handle_slot_left_click(
                                    &mut chest.items[idx],
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(
                                &mut chest.items[idx],
                                &mut self.player.inventory.held_item,
                            );
                        }
                    }
                    ChestSlotTarget::Main(idx) => {
                        Self::handle_chest_player_slot_click(
                            &mut self.player.inventory.main[idx],
                            &mut self.player.inventory.held_item,
                            chest,
                            click_left,
                            click_right,
                            input.key_shift,
                        );
                    }
                    ChestSlotTarget::Hotbar(idx) => {
                        Self::handle_chest_player_slot_click(
                            &mut self.player.inventory.hotbar[idx],
                            &mut self.player.inventory.held_item,
                            chest,
                            click_left,
                            click_right,
                            input.key_shift,
                        );
                    }
                }
            }
        }

        self.render_menu_world_backdrop();
        let chest = self.world.get_or_create_chest(pos);
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

        self.world.update_furnaces(dt);
        self.cycle_hotbar_scroll(input);

        if self.is_close_container_pressed(input) {
            if self.player.gamemode == GameMode::Creative {
                self.player.inventory.held_item = None;
            }
            crate::log_info!("MENU", "Closed furnace container, returning to Playing");
            self.state = GameState::Playing;
            out.request_capture_mouse = true;
        }

        let layout = FurnaceLayout::new(self.renderer.width, self.renderer.height);
        let hovered_target = mouse_pos.and_then(|(mx, my)| layout.find_slot_at(mx as i32, my as i32));

        let click_left = (left_down && !self.prev_left_down) || input.gp_south_just_pressed;
        let click_right = (!self.prev_right_down && right_down) || input.gp_west_just_pressed;

        for i in 0..10 {
            if input.key_digits[i] && !self.prev_key_digits[i] {
                if let Some(target) = hovered_target {
                    let furnace = self.world.get_or_create_furnace(pos);
                    match target {
                        FurnaceSlotTarget::Input => {
                            std::mem::swap(&mut furnace.input, &mut self.player.inventory.hotbar[i]);
                        }
                        FurnaceSlotTarget::Fuel => {
                            std::mem::swap(&mut furnace.fuel, &mut self.player.inventory.hotbar[i]);
                        }
                        FurnaceSlotTarget::Output => {
                            if let Some(out_stack) = furnace.output.take() {
                                let rem = self
                                    .player
                                    .inventory
                                    .add_item(out_stack.block, out_stack.count);
                                if rem > 0 {
                                    furnace.output = Some(ItemStack::new(out_stack.block, rem));
                                }
                            }
                        }
                        FurnaceSlotTarget::Main(idx) => {
                            std::mem::swap(
                                &mut self.player.inventory.main[idx],
                                &mut self.player.inventory.hotbar[i],
                            );
                        }
                        FurnaceSlotTarget::Hotbar(idx) => {
                            self.player.inventory.hotbar.swap(i, idx);
                        }
                    }
                }
            }
        }

        if click_left || click_right {
            let mut furnace_smelted: Option<(BlockId, u8)> = None;
            let furnace = self.world.get_or_create_furnace(pos);
            if let Some(target) = hovered_target {
                self.audio
                    .play_sound(crate::audio::SoundId::UiClick, 0.7, 1.0);
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
                                Inventory::handle_slot_left_click(
                                    &mut furnace.input,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(
                                &mut furnace.input,
                                &mut self.player.inventory.held_item,
                            );
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
                                Inventory::handle_slot_left_click(
                                    &mut furnace.fuel,
                                    &mut self.player.inventory.held_item,
                                );
                            }
                        } else if click_right {
                            Inventory::handle_slot_right_click(
                                &mut furnace.fuel,
                                &mut self.player.inventory.held_item,
                            );
                        }
                    }
                    FurnaceSlotTarget::Output => {
                        if click_left {
                            if input.key_shift {
                                if let Some(out_stack) = furnace.output.take() {
                                    let rem = self
                                        .player
                                        .inventory
                                        .add_item(out_stack.block, out_stack.count);
                                    let taken = out_stack.count.saturating_sub(rem);
                                    if taken > 0 {
                                        furnace_smelted = Some((out_stack.block, taken));
                                    }
                                    if rem > 0 {
                                        furnace.output = Some(ItemStack::new(out_stack.block, rem));
                                    }
                                }
                            } else if let Some(out_stack) = &mut furnace.output {
                                let taken_block = out_stack.block;
                                let taken_count = out_stack.count;
                                let mut took = false;
                                if let Some(held) = &mut self.player.inventory.held_item {
                                    if held.block == out_stack.block
                                        && held.count + out_stack.count <= 64
                                    {
                                        held.count += out_stack.count;
                                        furnace.output = None;
                                        took = true;
                                    }
                                } else {
                                    self.player.inventory.held_item = furnace.output.take();
                                    took = true;
                                }
                                if took {
                                    furnace_smelted = Some((taken_block, taken_count));
                                }
                            }
                        }
                    }
                    FurnaceSlotTarget::Main(idx) => {
                        Self::handle_furnace_player_slot_click(
                            &mut self.player.inventory.main[idx],
                            &mut self.player.inventory.held_item,
                            furnace,
                            click_left,
                            click_right,
                            input.key_shift,
                        );
                    }
                    FurnaceSlotTarget::Hotbar(idx) => {
                        Self::handle_furnace_player_slot_click(
                            &mut self.player.inventory.hotbar[idx],
                            &mut self.player.inventory.held_item,
                            furnace,
                            click_left,
                            click_right,
                            input.key_shift,
                        );
                    }
                }
            }
            if let Some((block, count)) = furnace_smelted {
                let q_smelt = self.quests.record_smelted(block, count);
                self.process_quest_completions(q_smelt);
            }
        }

        self.render_menu_world_backdrop();
        let furnace = self.world.get_or_create_furnace(pos);
        draw_furnace_ui(
            &mut self.renderer.color_buffer,
            self.renderer.width,
            self.renderer.height,
            furnace,
            &self.player,
            mouse_pos,
        );
    }
}
