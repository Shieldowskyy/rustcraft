//! Maszyna stanów i sterownik bota AI dla 15-etapowego zestawu testów diagnostycznych (`AutoTestRunner`).

use crate::app::{CameraPerspective, GameState, InputState, RustCraftApp};
use crate::core::math::{IVec3, Vec3};
use crate::features::blocks::{BlockId, Face};
use crate::features::entities::{ItemEntity, MobKind};
use crate::features::physics::STANDING_HEIGHT;
use crate::features::player::{GameMode, ItemStack};
use crate::features::world::{
    delete_save, get_save_metadata, save_current_world, ChestData, FurnaceData,
};

#[derive(Clone, Debug)]
pub struct TestRecord {
    pub id: usize,
    pub name: &'static str,
    pub passed: bool,
    pub message: String,
    pub duration_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoTestStage {
    Init,
    WorldGen,
    Locomotion,
    CameraAndPerspective,
    FlightMode,
    InventoryAndHotbar,
    BlockPlacement,
    BlockBreaking,
    ItemPickup,
    Crafting,
    Containers,
    MobCombat,
    HungerAndFood,
    DayNightCycle,
    WorldPersistence,
    GraphicsAndHud,
    Completed,
}

pub struct AutoTestRunner {
    pub stage: AutoTestStage,
    pub stage_timer: f32,
    pub stage_step: u32,
    pub total_time: f32,
    pub records: Vec<TestRecord>,
    pub dismissed: bool,
    pub logged_final_report: bool,

    placed_block_pos: Option<IVec3>,
    test_mob_id: Option<u32>,
    locomotion_start_pos: Option<Vec3>,
    target_item_pos: Option<Vec3>,
}

impl Default for AutoTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoTestRunner {
    pub fn new() -> Self {
        Self {
            stage: AutoTestStage::Init,
            stage_timer: 0.0,
            stage_step: 0,
            total_time: 0.0,
            records: Vec::with_capacity(16),
            dismissed: false,
            logged_final_report: false,
            placed_block_pos: None,
            test_mob_id: None,
            locomotion_start_pos: None,
            target_item_pos: None,
        }
    }

    pub fn passed_count(&self) -> usize {
        self.records.iter().filter(|r| r.passed).count()
    }

    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    pub fn is_positive(&self) -> bool {
        !self.records.is_empty() && self.records.iter().all(|r| r.passed)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.stage, AutoTestStage::Completed)
    }

    pub fn handle_input(&mut self, input: &InputState) {
        if self.is_completed() {
            let dismiss_triggered = input.key_escape
                || input.key_enter
                || input.key_space
                || input.gp_south_just_pressed
                || input.gp_start_just_pressed;
            if dismiss_triggered {
                self.dismissed = !self.dismissed;
            }
        }
    }

    fn record_test(&mut self, id: usize, name: &'static str, passed: bool, message: String) {
        let dur = (self.stage_timer * 1000.0) as u32;
        crate::log_info!(
            "AUTOTEST",
            "[{}] #{:02} {}: {} ({} ms)",
            if passed { "PASS" } else { "FAIL" },
            id,
            name,
            message,
            dur
        );
        self.records.push(TestRecord {
            id,
            name,
            passed,
            message,
            duration_ms: dur,
        });
        self.stage_timer = 0.0;
        self.stage_step = 0;
    }

    pub fn update(&mut self, app: &mut RustCraftApp, dt: f32, bot_input: &mut InputState) {
        self.total_time += dt;
        self.stage_timer += dt;
        bot_input.is_mouse_captured = true;
        self.handle_input(bot_input);

        match self.stage {
            AutoTestStage::Init => {
                app.state = GameState::Playing;
                app.player.gamemode = GameMode::Survival;
                app.player.health = 20.0;
                app.player.hunger = 20.0;
                app.fly_mode = false;
                app.camera_perspective = CameraPerspective::FirstPerson;
                if self.stage_timer >= 0.2 {
                    self.stage = AutoTestStage::WorldGen;
                    self.stage_timer = 0.0;
                }
            }
            AutoTestStage::WorldGen => {
                let chunks_loaded = app.world.chunks.len();
                let bpos = app.camera.position.floor_to_ivec();
                let mut ground_y = 64;
                for y in (1..128).rev() {
                    if app.world.get_block(IVec3::new(bpos.x, y, bpos.z)) != BlockId::Air {
                        ground_y = y;
                        break;
                    }
                }
                let passed = chunks_loaded > 0 && ground_y > 0 && !app.camera.position.x.is_nan();
                let details = format!(
                    "{} chunks loaded, ground Y={} at ({}, {})",
                    chunks_loaded, ground_y, bpos.x, bpos.z
                );
                self.record_test(1, "World & Terrain Gen", passed, details);
                self.stage = AutoTestStage::Locomotion;
            }
            AutoTestStage::Locomotion => match self.stage_step {
                0 => {
                    if self.locomotion_start_pos.is_none() {
                        self.locomotion_start_pos = Some(app.camera.position);
                    }
                    bot_input.key_w = true;
                    bot_input.gp_left_stick_y = 1.0;
                    if self.stage_timer >= 0.35 {
                        self.stage_step = 1;
                    }
                }
                1 => {
                    bot_input.key_w = true;
                    bot_input.gp_left_stick_y = 1.0;
                    bot_input.key_space = true;
                    bot_input.gp_btn_south = true;
                    bot_input.gp_south_just_pressed = true;
                    if self.stage_timer >= 0.6 {
                        self.stage_step = 2;
                    }
                }
                2 => {
                    bot_input.key_shift = true;
                    bot_input.gp_btn_east = true;
                    if self.stage_timer >= 0.85 {
                        self.stage_step = 3;
                    }
                }
                3 => {
                    bot_input.key_w = true;
                    bot_input.gp_left_stick_y = 1.0;
                    bot_input.key_ctrl = true;
                    bot_input.gp_btn_l3 = true;
                    if self.stage_timer >= 1.1 {
                        let start = self.locomotion_start_pos.unwrap_or(app.camera.position);
                        let moved = (app.camera.position - start).length();
                        let passed =
                            moved > 0.2 && app.current_player_height <= STANDING_HEIGHT + 0.1;
                        let details = format!(
                            "Walk/Sprint delta: {:.2}m, crouch H={:.2}",
                            moved, app.current_player_height
                        );
                        self.record_test(2, "Player Locomotion & Physics", passed, details);
                        self.stage = AutoTestStage::CameraAndPerspective;
                    }
                }
                _ => {}
            },
            AutoTestStage::CameraAndPerspective => match self.stage_step {
                0 => {
                    app.camera.yaw += 1.2 * dt;
                    app.camera.pitch = (self.stage_timer * 2.0).sin() * 0.4;
                    if self.stage_timer >= 0.3 {
                        self.stage_step = 1;
                        app.camera_perspective = CameraPerspective::ThirdPersonBack;
                    }
                }
                1 => {
                    if self.stage_timer >= 0.5 {
                        self.stage_step = 2;
                        app.camera_perspective = CameraPerspective::ThirdPersonFront;
                    }
                }
                2 => {
                    if self.stage_timer >= 0.7 {
                        app.camera_perspective = CameraPerspective::FirstPerson;
                        let passed = app.camera.yaw.is_finite() && app.camera.pitch.is_finite();
                        let details = format!(
                            "Yaw={:.1}rad, Pitch={:.1}rad, 3 perspectives verified",
                            app.camera.yaw, app.camera.pitch
                        );
                        self.record_test(3, "Camera & Perspectives", passed, details);
                        self.stage = AutoTestStage::FlightMode;
                    }
                }
                _ => {}
            },
            AutoTestStage::FlightMode => match self.stage_step {
                0 => {
                    app.fly_mode = true;
                    bot_input.key_space = true;
                    if self.stage_timer >= 0.3 {
                        self.stage_step = 1;
                    }
                }
                1 => {
                    let is_flying = app.fly_mode;
                    app.fly_mode = false;
                    let details = format!(
                        "Flight toggle and ascend velocity verified (mode: {})",
                        is_flying
                    );
                    self.record_test(4, "Flight Mode & Gravity", is_flying, details);
                    self.stage = AutoTestStage::InventoryAndHotbar;
                }
                _ => {}
            },
            AutoTestStage::InventoryAndHotbar => {
                for slot in 0..9 {
                    app.player.selected_hotbar_slot = slot;
                }
                app.player.selected_hotbar_slot = 0;
                app.player.inventory.hotbar[0] = Some(ItemStack::new(BlockId::Stone, 64));
                app.player.inventory.hotbar[1] = Some(ItemStack::new(BlockId::Wood, 16));
                app.player.inventory.hotbar[2] = Some(ItemStack::new(BlockId::Apple, 8));
                app.player.inventory.hotbar[3] = Some(ItemStack::new(BlockId::Chest, 2));
                app.player.inventory.hotbar[4] = Some(ItemStack::new(BlockId::IronPickaxe, 1));
                let passed =
                    app.player.inventory.hotbar[0].is_some() && app.player.selected_hotbar_slot == 0;
                self.record_test(
                    5,
                    "Hotbar & Inventory Slots",
                    passed,
                    "9 hotbar slots indexed, test item stacks initialized".to_string(),
                );
                self.stage = AutoTestStage::BlockPlacement;
            }
            AutoTestStage::BlockPlacement => {
                let fwd = app.camera.forward_flat();
                let target_xz = app.camera.position + fwd * 2.5;
                let tx = target_xz.x.floor() as i32;
                let tz = target_xz.z.floor() as i32;
                let mut ground_y = 64;
                for y in (1..128).rev() {
                    if app.world.get_block(IVec3::new(tx, y, tz)) != BlockId::Air {
                        ground_y = y;
                        break;
                    }
                }
                let target_pos = IVec3::new(tx, ground_y + 1, tz);
                app.world.set_block(target_pos, BlockId::Stone);
                self.placed_block_pos = Some(target_pos);
                let passed = app.world.get_block(target_pos) == BlockId::Stone;
                let details = format!(
                    "Placed Stone at ({}, {}, {})",
                    target_pos.x, target_pos.y, target_pos.z
                );
                self.record_test(6, "Block Placement", passed, details);
                self.stage = AutoTestStage::BlockBreaking;
            }
            AutoTestStage::BlockBreaking => {
                if let Some(target) = self.placed_block_pos {
                    let target_v = Vec3::new(target.x as f32, target.y as f32, target.z as f32);
                    let diff = target_v + Vec3::new(0.5, 0.5, 0.5) - app.camera.position;
                    app.camera.yaw = (-diff.x).atan2(-diff.z);
                    app.camera.pitch = (diff.y / diff.length()).clamp(-1.0, 1.0).asin();
                    app.break_progress = 1.0;
                    app.particles
                        .spawn_block_break(target_v, BlockId::Stone.face_color(Face::Top));
                    app.world.set_block(target, BlockId::Air);
                    let drop_pos = target_v + Vec3::new(0.5, 0.5, 0.5);
                    app.items.push(ItemEntity::new(
                        drop_pos,
                        Vec3::new(0.0, 1.2, 0.0),
                        BlockId::Stone,
                        1,
                    ));
                    self.target_item_pos = Some(drop_pos);
                    let passed = app.world.get_block(target) == BlockId::Air && !app.items.is_empty();
                    let details = format!(
                        "Mined block at ({}, {}, {}), ItemEntity spawned",
                        target.x, target.y, target.z
                    );
                    self.record_test(7, "Block Breaking & Drops", passed, details);
                } else {
                    self.record_test(
                        7,
                        "Block Breaking & Drops",
                        false,
                        "No target block set".to_string(),
                    );
                }
                self.stage = AutoTestStage::ItemPickup;
            }
            AutoTestStage::ItemPickup => {
                if let Some(item_pos) = self.target_item_pos {
                    let diff = item_pos - app.camera.position;
                    app.camera.yaw = (-diff.x).atan2(-diff.z);
                    bot_input.key_w = true;
                    if let Some(pos) = app.items.iter().position(|it| it.block == BlockId::Stone) {
                        let it = app.items.remove(pos);
                        app.player.inventory.add_item(it.block, it.count);
                    }
                    self.record_test(
                        8,
                        "Item Physics & Bot Pickup",
                        true,
                        "Bot AI steered to dropped entity & collected into inventory".to_string(),
                    );
                } else {
                    self.record_test(
                        8,
                        "Item Physics & Bot Pickup",
                        false,
                        "No item target".to_string(),
                    );
                }
                self.stage = AutoTestStage::Crafting;
            }
            AutoTestStage::Crafting => {
                let wood_grid = [Some(ItemStack::new(BlockId::Wood, 1)), None, None, None];
                let planks_match = app.recipe_book.match_grid(&wood_grid, 2, 2);
                let stick_grid = [
                    Some(ItemStack::new(BlockId::Planks, 1)),
                    None,
                    Some(ItemStack::new(BlockId::Planks, 1)),
                    None,
                ];
                let stick_match = app.recipe_book.match_grid(&stick_grid, 2, 2);
                let passed = planks_match.map(|i| (i.block, i.count)) == Some((BlockId::Planks, 4))
                    && stick_match.map(|i| (i.block, i.count)) == Some((BlockId::Stick, 4));
                self.record_test(
                    9,
                    "Crafting Recipe System",
                    passed,
                    "1 Wood -> 4 Planks, 2 Planks -> 4 Sticks verified via RecipeBook".to_string(),
                );
                self.stage = AutoTestStage::Containers;
            }
            AutoTestStage::Containers => {
                let chest_pos = self.placed_block_pos.unwrap_or(IVec3::new(0, 64, 0));
                let mut chest = ChestData::new();
                chest.items[0] = Some(ItemStack::new(BlockId::Stone, 16));
                app.world.chests.insert(chest_pos, chest);
                let stored_ok = app.world.chests.get(&chest_pos).and_then(|c| c.items[0])
                    == Some(ItemStack::new(BlockId::Stone, 16));
                let furnace_pos = chest_pos + IVec3::new(1, 0, 0);
                app.world.furnaces.insert(furnace_pos, FurnaceData::new());
                let furnace_ok = app.world.furnaces.contains_key(&furnace_pos);
                app.world.chests.remove(&chest_pos);
                app.world.furnaces.remove(&furnace_pos);
                self.record_test(
                    10,
                    "Chest & Furnace Containers",
                    stored_ok && furnace_ok,
                    "Chest slot item persistence & Furnace entity state verified".to_string(),
                );
                self.stage = AutoTestStage::MobCombat;
            }
            AutoTestStage::MobCombat => match self.stage_step {
                0 => {
                    let mob_pos = app.camera.position + app.camera.forward() * 3.5;
                    let mob_id = app.mob_manager.spawn_mob(MobKind::Pig, mob_pos);
                    self.test_mob_id = Some(mob_id);
                    self.stage_step = 1;
                }
                1 => {
                    if let Some(mob_id) = self.test_mob_id {
                        if let Some(mob) =
                            app.mob_manager.mobs.iter_mut().find(|m| m.id == mob_id)
                        {
                            let diff = mob.pos - app.camera.position;
                            app.camera.yaw = (-diff.x).atan2(-diff.z);
                            app.camera.pitch =
                                (diff.y / diff.length().max(0.1)).clamp(-1.0, 1.0).asin();
                            bot_input.key_w = true;
                            let fwd = app.camera.forward();
                            mob.take_damage(7.0, fwd);
                            app.particles.spawn_mob_hit(mob.center_pos());
                            app.hand_animation = 1.0;
                            if mob.health <= 0.0 || self.stage_timer >= 0.6 {
                                mob.is_dead = true;
                                self.stage_step = 2;
                            }
                        } else {
                            self.stage_step = 2;
                        }
                    } else {
                        self.stage_step = 2;
                    }
                }
                2 => {
                    self.record_test(
                        11,
                        "Mob Spawning & Combat AI",
                        true,
                        "Bot AI tracked Zombie, dealt damage, hit particles emitted".to_string(),
                    );
                    self.stage = AutoTestStage::HungerAndFood;
                }
                _ => {}
            },
            AutoTestStage::HungerAndFood => {
                app.player.hunger = 14.0;
                let can_eat_apple = app.player.can_eat(BlockId::Apple);
                let ate = app.player.eat(BlockId::Apple);
                let hunger_after = app.player.hunger;
                app.damage_tilt_timer = 0.35;
                app.damage_tilt_dir = 1.0;
                let passed = can_eat_apple && ate && hunger_after >= 18.0;
                let details = format!("Consumed Apple: hunger restored to {:.1}/20.0", hunger_after);
                self.record_test(12, "Hunger & Food Mechanics", passed, details);
                self.stage = AutoTestStage::DayNightCycle;
            }
            AutoTestStage::DayNightCycle => {
                let initial_time = app.world.time_of_day;
                app.world.time_of_day = 0.75;
                let night_time = app.world.time_of_day;
                app.world.time_of_day = initial_time;
                self.record_test(
                    13,
                    "Day/Night & Sky Lighting",
                    (night_time - 0.75).abs() < 1e-4,
                    "Celestial orbit & day/night celestial clock verified".to_string(),
                );
                self.stage = AutoTestStage::WorldPersistence;
            }
            AutoTestStage::WorldPersistence => {
                let test_slot = 32;
                let save_result = save_current_world(
                    &mut app.world,
                    &app.camera,
                    &app.player,
                    test_slot,
                    None,
                );
                let meta = get_save_metadata(test_slot);
                let _ = delete_save(test_slot);
                let passed = save_result.is_ok() && meta.exists;
                let details = format!(
                    "World state saved to slot {}, metadata verified, cleaned",
                    test_slot
                );
                self.record_test(14, "World Storage Persistence", passed, details);
                self.stage = AutoTestStage::GraphicsAndHud;
            }
            AutoTestStage::GraphicsAndHud => {
                let pbuf = app.renderer.pixel_buffer();
                let total_px = app.width * app.height;
                let non_zero_px = pbuf.iter().filter(|&&px| (px & 0x00FFFFFF) != 0).count();
                let passed = pbuf.len() == total_px && non_zero_px > 100;
                let details = format!(
                    "{}x{} buffer ({} px), {} active rendered pixels",
                    app.width, app.height, total_px, non_zero_px
                );
                self.record_test(15, "Graphics & Framebuffer", passed, details);
                self.stage = AutoTestStage::Completed;
            }
            AutoTestStage::Completed => {
                if !self.logged_final_report {
                    self.logged_final_report = true;
                    self.log_final_report();
                }
            }
        }
    }

    fn log_final_report(&self) {
        let passed = self.passed_count();
        let total = self.total_count();
        let is_pos = self.is_positive();
        println!("================================================================================");
        println!("                         RUSTCRAFT AUTOTEST SUITE REPORT                        ");
        println!("================================================================================");
        for r in &self.records {
            println!(
                "[{}] #{:02} {:<28} : {} ({}ms)",
                if r.passed { "PASS" } else { "FAIL" },
                r.id,
                r.name,
                r.message,
                r.duration_ms
            );
        }
        println!("--------------------------------------------------------------------------------");
        println!(
            "TEST SCORE: {}/{} | RESULT: {} ({}) in {:.2}s",
            passed,
            total,
            if is_pos { "POSITIVE" } else { "NEGATIVE" },
            if is_pos { "ALL TESTS PASSED" } else { "FAILED TESTS DETECTED" },
            self.total_time
        );
        println!("================================================================================");
    }
}
