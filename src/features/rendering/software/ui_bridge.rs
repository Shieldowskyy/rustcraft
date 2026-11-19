//! 2D UI & HUD delegation methods on `Renderer` writing into `color_buffer`.
//! Zero platform `#[cfg]`.

use super::types::Renderer;

impl Renderer {
    #[inline(always)]
    pub fn main_menu_button_rects(&self) -> [(i32, i32, i32, i32); 4] {
        crate::ui::main_menu_button_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn main_menu_nickname_rect(&self) -> (i32, i32, i32, i32) {
        crate::ui::menu::main_menu_nickname_rect(self.width, self.height)
    }

    #[inline(always)]
    pub fn multiplayer_menu_rects(&self) -> [(i32, i32, i32, i32); crate::ui::multiplayer_menu::MULTIPLAYER_RECT_COUNT] {
        crate::ui::multiplayer_menu_rects(self.width, self.height)
    }

    pub fn draw_main_menu(
        &mut self,
        hovered: Option<usize>,
        nickname: &str,
        nick_hovered: bool,
        is_gamepad_active: bool,
        splash_text: &str,
        time_sec: f32,
    ) {
        crate::ui::draw_main_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            hovered,
            nickname,
            nick_hovered,
            is_gamepad_active,
            splash_text,
            time_sec,
        );
    }

    pub fn draw_dev_logo(&mut self, logo: &crate::ui::dev_logo::DevLogo, fade: f32) {
        logo.draw(&mut self.color_buffer, self.width, self.height, fade);
    }

    #[inline(always)]
    pub fn nickname_dialog_rects(&self) -> [(i32, i32, i32, i32); 4] {
        crate::ui::nickname_dialog_rects(self.width, self.height)
    }

    pub fn draw_nickname_dialog(
        &mut self,
        current_nick: &str,
        is_selected: bool,
        hovered_elem: Option<usize>,
        cursor_visible: bool,
        is_gamepad_active: bool,
    ) {
        crate::ui::draw_nickname_dialog(
            &mut self.color_buffer,
            self.width,
            self.height,
            current_nick,
            is_selected,
            hovered_elem,
            cursor_visible,
            is_gamepad_active,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_multiplayer_menu(
        &mut self,
        nickname: &str,
        server_addr: &str,
        focused_field: usize,
        hovered_idx: Option<usize>,
        status_msg: Option<&str>,
        is_connecting: bool,
        lan_servers: &[crate::net::DiscoveredServer],
        selected_lan: Option<usize>,
        is_refreshing: bool,
    ) {
        crate::ui::draw_multiplayer_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            nickname,
            server_addr,
            focused_field,
            hovered_idx,
            status_msg,
            is_connecting,
            lan_servers,
            selected_lan,
            is_refreshing,
        );
    }

    pub fn draw_player_list_tab(
        &mut self,
        players: &[crate::net::PlayerInfo],
        local_nickname: &str,
        is_multiplayer: bool,
    ) {
        crate::ui::draw_player_list_tab(
            &mut self.color_buffer,
            self.width,
            self.height,
            players,
            local_nickname,
            is_multiplayer,
        );
    }

    #[inline(always)]
    pub fn save_card_rects(&self) -> [(i32, i32, i32, i32); crate::ui::VISIBLE_SAVE_ROWS] {
        crate::ui::save_card_rects(self.width)
    }

    #[inline(always)]
    pub fn save_bottom_button_rects(&self) -> [(i32, i32, i32, i32); 4] {
        crate::ui::save_bottom_button_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_tab_rects(&self) -> [(i32, i32, i32, i32); 4] {
        crate::ui::settings_tab_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_audio_rects(&self) -> [(i32, i32, i32, i32); 8] {
        crate::ui::settings_audio_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_graphics_rects(&self) -> [(i32, i32, i32, i32); 14] {
        crate::ui::settings_graphics_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_controls_rects(&self) -> [(i32, i32, i32, i32); 11] {
        crate::ui::settings_controls_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_debug_rects(&self) -> [(i32, i32, i32, i32); 5] {
        crate::ui::settings_debug_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn settings_option_rects(&self) -> [(i32, i32, i32, i32); 14] {
        crate::ui::settings_graphics_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn create_world_rects(&self) -> [(i32, i32, i32, i32); 8] {
        crate::ui::create_world_rects(self.width, self.height)
    }

    #[inline(always)]
    pub fn pause_menu_button_rects(&self) -> [(i32, i32, i32, i32); 7] {
        crate::ui::pause_menu_button_rects(self.width, self.height)
    }

    pub fn draw_save_select_menu(
        &mut self,
        selected_slot: u8,
        scroll_offset: usize,
        metadata_cache: &[crate::features::world::SaveMetadata; crate::ui::MAX_SAVE_SLOTS],
        hovered_card: Option<usize>,
        hovered_btn: Option<usize>,
        is_gamepad_active: bool,
    ) {
        crate::ui::draw_save_select_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            selected_slot,
            scroll_offset,
            metadata_cache,
            hovered_card,
            hovered_btn,
            is_gamepad_active,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_create_world_menu(
        &mut self,
        slot: u8,
        name: &str,
        seed_str: &str,
        gamemode: crate::features::player::GameMode,
        generator_type: crate::features::world::WorldGeneratorType,
        quests_enabled: bool,
        focused_element: usize,
        hovered_element: Option<usize>,
        blink_timer: f32,
        is_selected: bool,
        is_gamepad_active: bool,
    ) {
        crate::ui::draw_create_world_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            slot,
            name,
            seed_str,
            gamemode,
            generator_type,
            quests_enabled,
            focused_element,
            hovered_element,
            blink_timer,
            is_selected,
            is_gamepad_active,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_settings_menu(
        &mut self,
        settings: &crate::core::settings::GameSettings,
        active_tab: usize,
        hovered_tab: Option<usize>,
        hovered: Option<usize>,
        server_ip: &str,
        profiler_copied: bool,
        mouse_pos: Option<(f32, f32)>,
        is_gamepad_active: bool,
    ) {
        crate::ui::draw_settings_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            settings,
            active_tab,
            hovered_tab,
            hovered,
            server_ip,
            profiler_copied,
            mouse_pos,
            is_gamepad_active,
        );
    }

    pub fn draw_crosshair(&mut self) {
        crate::ui::draw_crosshair(&mut self.color_buffer, self.width, self.height);
    }

    pub fn draw_break_progress(&mut self, progress: f32) {
        crate::ui::draw_break_progress(&mut self.color_buffer, self.width, self.height, progress);
    }

    pub fn draw_underwater_overlay(&mut self, time: f32) {
        crate::ui::draw_underwater_overlay(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_lava_overlay(&mut self, time: f32) {
        crate::ui::draw_lava_overlay(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_burning_fire_overlay(&mut self, time: f32) {
        crate::ui::draw_burning_fire_overlay(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_hotbar(&mut self, player: &crate::features::player::Player) {
        crate::ui::draw_hotbar(&mut self.color_buffer, self.width, self.height, player);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_inventory(
        &mut self,
        player: &crate::features::player::Player,
        hovered_slot: Option<crate::ui::InventorySlotTarget>,
        crafting_table_open: bool,
        recipes: &crate::features::player::RecipeBook,
        unlocked_recipe_indices: &[usize],
        recipe_book_open: bool,
        recipe_book_scroll: usize,
        recipe_book_status: Option<&str>,
        mouse_pos: Option<(f32, f32)>,
        is_gamepad_active: bool,
    ) {
        crate::ui::draw_inventory_screen(
            &mut self.color_buffer,
            self.width,
            self.height,
            player,
            hovered_slot,
            crafting_table_open,
            recipes,
            unlocked_recipe_indices,
            recipe_book_open,
            recipe_book_scroll,
            recipe_book_status,
            mouse_pos,
            is_gamepad_active,
        );
    }

    pub fn draw_pause_menu(
        &mut self,
        fly_mode: bool,
        hovered: Option<usize>,
        is_gamepad_active: bool,
        can_host: bool,
        is_hosting: bool,
        status: Option<&str>,
    ) {
        crate::ui::draw_pause_menu(
            &mut self.color_buffer,
            self.width,
            self.height,
            fly_mode,
            hovered,
            is_gamepad_active,
            can_host,
            is_hosting,
            status,
        );
    }

    pub fn draw_capture_prompt(&mut self, native: bool) {
        crate::ui::draw_capture_prompt(&mut self.color_buffer, self.width, self.height, native);
    }

    pub fn draw_debug_panel(&mut self, lines: &[String]) {
        crate::ui::draw_debug_panel(&mut self.color_buffer, self.width, self.height, lines);
    }

    pub fn draw_advanced_debug_overlay(
        &mut self,
        left_lines: &[String],
        right_lines: &[String],
        profiler: &crate::core::profiler::FrameProfiler,
    ) {
        crate::ui::draw_advanced_debug_overlay(
            &mut self.color_buffer,
            self.width,
            self.height,
            left_lines,
            right_lines,
            profiler,
        );
    }

    #[inline(always)]
    pub fn death_screen_button_rects(&self) -> [(i32, i32, i32, i32); 2] {
        crate::ui::death_screen_button_rects(self.width, self.height)
    }

    pub fn apply_biome_color_filter(&mut self, tint: [f32; 3]) {
        crate::ui::apply_biome_color_filter(&mut self.color_buffer, tint);
    }

    pub fn draw_damage_flash(&mut self, intensity: f32) {
        crate::ui::draw_damage_flash(&mut self.color_buffer, self.width, self.height, intensity);
    }

    pub fn draw_low_health_vignette(&mut self, time: f32) {
        crate::ui::draw_low_health_vignette(&mut self.color_buffer, self.width, self.height, time);
    }

    pub fn draw_death_screen(&mut self, hovered: Option<usize>) {
        crate::ui::draw_death_screen(&mut self.color_buffer, self.width, self.height, hovered);
    }
}
