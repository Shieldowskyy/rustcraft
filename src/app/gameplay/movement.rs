//! Player camera look, stance (fly/crouch/sprint/FOV), locomotion physics, and hotbar selection.
//! Mutates camera orientation/position, player velocities, collision state, and active hotbar slot.
//! Delegates biome/ambient probing and survival vitals to `vitals.rs`.

use crate::core::controls::Action;
use crate::core::math::Vec3;
use crate::gameplay::physics::{
    move_player_with_collision, player_touches_ladder, CROUCH_EYE_OFFSET, CROUCH_HEIGHT,
    FLY_SPEED, GRAVITY, JUMP_SPEED, PLAYER_HALF_WIDTH, SNEAK_SPEED, SPRINT_JUMP_BOOST,
    SPRINT_SPEED, STANDING_EYE_OFFSET, STANDING_HEIGHT, SWIM_SPEED, TERMINAL_VELOCITY, WALK_SPEED,
};
use crate::gameplay::player::GameMode;
use crate::world::block::BlockId;

use super::super::{AppOutput, CameraPerspective, InputState, RustCraftApp};

impl RustCraftApp {
    pub(super) fn update_camera_and_stance(
        &mut self,
        dt: f32,
        input: &InputState,
        out: &mut AppOutput,
    ) -> (Vec3, bool, bool) {
        let any_click = input.mouse_left || input.mouse_right;
        if any_click && !input.is_mouse_captured && !self.chat.is_open {
            out.request_capture_mouse = true;
        }

        let look_x_dir = if self.settings.invert_z { -1.0 } else { 1.0 };
        let look_y_dir = if self.settings.invert_y { -1.0 } else { 1.0 };

        if input.is_mouse_captured && !self.chat.is_open {
            self.camera.rotate(
                input.mouse_dx * 0.0025 * look_x_dir * self.settings.mouse_sensitivity,
                -input.mouse_dy * 0.0025 * look_y_dir * self.settings.mouse_sensitivity,
            );
        }

        let gp_look_speed_x = 3.2 * self.settings.gamepad_sensitivity;
        let gp_look_speed_y = 2.8 * self.settings.gamepad_sensitivity;
        if (input.gp_right_stick_x.abs() > 0.0 || input.gp_right_stick_y.abs() > 0.0)
            && !self.chat.is_open
        {
            self.camera.rotate(
                input.gp_right_stick_x * gp_look_speed_x * dt * look_x_dir,
                input.gp_right_stick_y * gp_look_speed_y * dt * look_y_dir,
            );
        }

        let forward = self.camera.forward_flat();
        let right = self.camera.right_flat();
        let mut wish_dir = Vec3::zero();

        if !self.chat.is_open {
            if self.controls.is_active(Action::MoveForward) {
                wish_dir = wish_dir + forward;
            }
            if self.controls.is_active(Action::MoveBackward) {
                wish_dir = wish_dir - forward;
            }
            if self.controls.is_active(Action::StrafeLeft) {
                wish_dir = wish_dir - right;
            }
            if self.controls.is_active(Action::StrafeRight) {
                wish_dir = wish_dir + right;
            }
            if input.gp_left_stick_y.abs() > 0.0 || input.gp_left_stick_x.abs() > 0.0 {
                wish_dir =
                    wish_dir + forward * input.gp_left_stick_y + right * input.gp_left_stick_x;
            }
            if wish_dir.length_sq() > 0.0001 {
                wish_dir = wish_dir.normalized();
            }
        }

        if self.jump_double_tap_timer > 0.0 {
            self.jump_double_tap_timer = (self.jump_double_tap_timer - dt).max(0.0);
        }

        if !self.chat.is_open
            && self.controls.is_just_pressed(Action::Jump)
            && self.player.gamemode == GameMode::Creative
        {
            if self.jump_double_tap_timer > 0.0 {
                self.fly_mode = !self.fly_mode;
                self.jump_double_tap_timer = 0.0;
                self.horiz_velocity = Vec3::zero();
                self.fly_velocity = Vec3::zero();
                if self.fly_mode {
                    self.vertical_velocity = 0.0;
                }
                crate::log_info!(
                    "PLAYER",
                    "Flight mode toggled by double-tap jump: {}",
                    self.fly_mode
                );
            } else {
                self.jump_double_tap_timer = 0.35;
            }
        }

        if !self.chat.is_open && self.controls.is_just_pressed(Action::ToggleFly) {
            self.fly_mode = !self.fly_mode;
            self.horiz_velocity = Vec3::zero();
            self.fly_velocity = Vec3::zero();
            crate::log_debug!("PLAYER", "Flight mode: {}", self.fly_mode);
        }

        if !self.chat.is_open && self.controls.is_just_pressed(Action::ToggleGamemode) {
            self.player.gamemode = if self.player.gamemode == GameMode::Survival {
                GameMode::Creative
            } else {
                self.fly_mode = false;
                GameMode::Survival
            };
            crate::log_info!("PLAYER", "Gamemode changed to {:?}", self.player.gamemode);
        }

        // F5 Camera Perspective cycling (First Person -> Third Person Back -> Third Person Front)
        if !self.chat.is_open
            && ((input.key_f5 && !self.prev_f5_down) || input.gp_select_just_pressed)
        {
            self.camera_perspective = match self.camera_perspective {
                CameraPerspective::FirstPerson => CameraPerspective::ThirdPersonBack,
                CameraPerspective::ThirdPersonBack => CameraPerspective::ThirdPersonFront,
                CameraPerspective::ThirdPersonFront => CameraPerspective::FirstPerson,
            };
            let mode_str = match self.camera_perspective {
                CameraPerspective::FirstPerson => "Perspective: First Person",
                CameraPerspective::ThirdPersonBack => "Perspective: Third Person (Back)",
                CameraPerspective::ThirdPersonFront => "Perspective: Third Person (Front)",
            };
            self.chat
                .add_message(mode_str.to_string(), crate::ui::pack_color(0.4, 0.9, 1.0));
            crate::log_info!("CAMERA", "{}", mode_str);
        }

        let sneak_btn_down = !self.chat.is_open && self.controls.is_active(Action::Sneak);
        let sneak_just_pressed =
            !self.chat.is_open && self.controls.is_just_pressed(Action::Sneak);
        self.prev_sneak_btn = sneak_btn_down;

        if self.settings.sneak_mode == crate::core::settings::SneakMode::Toggle {
            if sneak_just_pressed {
                self.crouch_toggled = !self.crouch_toggled;
            }
        } else {
            self.crouch_toggled = sneak_btn_down;
        }

        let is_crouching = !self.fly_mode && self.crouch_toggled;

        let target_eye = if is_crouching {
            CROUCH_EYE_OFFSET
        } else {
            STANDING_EYE_OFFSET
        };
        let target_h = if is_crouching {
            CROUCH_HEIGHT
        } else {
            STANDING_HEIGHT
        };
        let t_eye = 1.0 - (-14.0 * dt).exp();
        let old_eye = self.current_eye_offset;
        self.current_eye_offset += (target_eye - self.current_eye_offset) * t_eye;
        self.current_player_height += (target_h - self.current_player_height) * t_eye;

        if !self.fly_mode {
            self.camera.position.y += self.current_eye_offset - old_eye;
        }

        let can_sprint =
            self.player.gamemode == GameMode::Creative || self.player.hunger > 6.0;
        let sprint_btn_down = !self.chat.is_open
            && (self.controls.is_active(Action::Sprint) || input.gp_left_stick_y > 0.85);
        let sprint_just_pressed = !self.chat.is_open
            && (self.controls.is_just_pressed(Action::Sprint)
                || (input.gp_left_stick_y > 0.85 && !self.prev_sprint_btn));
        self.prev_sprint_btn = sprint_btn_down;

        if self.settings.sprint_mode == crate::core::settings::SprintMode::Toggle {
            if sprint_just_pressed && can_sprint && !is_crouching {
                self.sprint_toggled = !self.sprint_toggled;
            }
            if !can_sprint || is_crouching || wish_dir.length_sq() <= 0.01 {
                self.sprint_toggled = false;
            }
        } else {
            self.sprint_toggled = sprint_btn_down;
        }

        let is_sprinting =
            !is_crouching && can_sprint && self.sprint_toggled && wish_dir.length_sq() > 0.01;

        let target_fov_deg = if is_sprinting {
            self.settings.fov_deg + 15.0
        } else if is_crouching {
            self.settings.fov_deg - 5.0
        } else {
            self.settings.fov_deg
        };
        let target_fov = target_fov_deg.to_radians();
        let t_fov = 1.0 - (-8.0 * dt).exp();
        self.camera.fov_y += (target_fov - self.camera.fov_y) * t_fov;

        (wish_dir, is_crouching, is_sprinting)
    }

    pub(super) fn update_player_locomotion(
        &mut self,
        dt: f32,
        _input: &InputState,
        wish_dir: Vec3,
        is_crouching: bool,
        is_sprinting: bool,
        out: &mut AppOutput,
    ) {
        let move_speed = if is_crouching {
            SNEAK_SPEED
        } else if is_sprinting {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        if self.fly_mode {
            let mut move_dir = wish_dir;
            if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                move_dir.y += 1.0;
            }
            if !self.chat.is_open && self.controls.is_active(Action::Sneak) {
                move_dir.y -= 1.0;
            }
            if move_dir.length_sq() > 0.0001 {
                move_dir = move_dir.normalized();
            }

            let fly_fast = !self.chat.is_open && self.controls.is_active(Action::Sprint);
            let fly_mult = if fly_fast { 2.0 } else { 1.0 };
            let target_fly = move_dir * (FLY_SPEED * fly_mult);
            let t_fly = 1.0 - (-14.0 * dt).exp();
            self.fly_velocity = self.fly_velocity.lerp(target_fly, t_fly);

            self.camera.position = self.camera.position + self.fly_velocity * dt;
            self.vertical_velocity = 0.0;
            self.horiz_velocity = Vec3::zero();
            self.player.head_underwater = matches!(
                self.world.get_block(self.camera.position.floor_to_ivec()),
                BlockId::Water | BlockId::Kelp | BlockId::Seagrass
            );
            self.player.air = self.player.max_air;
            self.drown_damage_timer = 0.0;
            return;
        }

        let feet_pos = Vec3::new(
            self.camera.position.x,
            self.camera.position.y - self.current_eye_offset - 0.08,
            self.camera.position.z,
        );
        let feet_block = self.world.get_block(feet_pos.floor_to_ivec());
        let eye_block = self.world.get_block(self.camera.position.floor_to_ivec());
        let in_water = matches!(feet_block, BlockId::Water | BlockId::Kelp | BlockId::Seagrass)
            || matches!(eye_block, BlockId::Water | BlockId::Kelp | BlockId::Seagrass);
        let in_lava = feet_block == BlockId::Lava || eye_block == BlockId::Lava;

        // Dolphin's Grace: swimming near a dolphin grants * 1.85 swim speed boost
        let near_dolphin = in_water
            && self.mob_manager.mobs.iter().any(|m| {
                m.kind == crate::gameplay::mob::MobKind::Dolphin
                    && m.health > 0.0
                    && (m.pos - self.camera.position).length_sq() < 196.0
            });
        let swim_speed = if near_dolphin {
            SWIM_SPEED * 1.85
        } else {
            SWIM_SPEED
        };

        let target_horiz = wish_dir
            * (if in_lava {
                SWIM_SPEED * 0.55
            } else if in_water {
                swim_speed
            } else {
                move_speed
            });
        let on_ice = feet_block == BlockId::Ice;
        let in_ladder = player_touches_ladder(
            &self.world,
            Vec3::new(
                self.camera.position.x,
                self.camera.position.y - self.current_eye_offset,
                self.camera.position.z,
            ),
            PLAYER_HALF_WIDTH,
            self.current_player_height,
        );
        let damping = if in_lava {
            12.0
        } else if in_water {
            8.0
        } else if self.on_ground {
            if on_ice {
                2.0
            } else if is_crouching {
                18.0
            } else {
                15.0
            }
        } else {
            2.0
        };
        let t_horiz = 1.0 - (-damping * dt).exp();
        self.horiz_velocity = self.horiz_velocity.lerp(target_horiz, t_horiz);

        if in_ladder {
            self.player.fall_distance = 0.0;
            if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                self.vertical_velocity = 3.5;
            } else if is_crouching {
                self.vertical_velocity = 0.0;
            } else if wish_dir.length_sq() > 0.01 {
                self.vertical_velocity = 2.4;
            } else {
                self.vertical_velocity = -1.2;
            }
        } else if in_lava {
            self.vertical_velocity = (self.vertical_velocity - 2.5 * dt).max(-1.8);
            if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                self.vertical_velocity = 2.2;
            }
        } else if in_water {
            self.vertical_velocity = (self.vertical_velocity - 4.0 * dt).max(-2.5);
            if !self.chat.is_open && self.controls.is_active(Action::Jump) {
                self.vertical_velocity = 3.2;
            }
        } else {
            self.vertical_velocity =
                (self.vertical_velocity - GRAVITY * dt).max(-TERMINAL_VELOCITY);

            let mut should_jump =
                !self.chat.is_open && self.controls.is_active(Action::Jump) && self.on_ground;

            // Auto-jump logic
            if !should_jump
                && self.settings.jump_mode == crate::core::settings::JumpMode::AutoJump
                && self.on_ground
                && wish_dir.length_sq() > 0.01
            {
                let check_dir = wish_dir.normalized();
                let feet_y = self.camera.position.y - self.current_eye_offset;
                let probe_origin = Vec3::new(self.camera.position.x, feet_y, self.camera.position.z);
                let step_ahead = check_dir * 0.65;

                let probe_feet = (probe_origin + step_ahead + Vec3::new(0.0, 0.2, 0.0)).floor_to_ivec();
                let probe_head = (probe_origin + step_ahead + Vec3::new(0.0, 1.2, 0.0)).floor_to_ivec();
                let probe_above = (probe_origin + Vec3::new(0.0, 2.0, 0.0)).floor_to_ivec();

                if self.world.get_block(probe_feet).is_solid()
                    && !self.world.get_block(probe_head).is_solid()
                    && !self.world.get_block(probe_above).is_solid()
                {
                    should_jump = true;
                }
            }

            if should_jump {
                self.vertical_velocity = JUMP_SPEED;
                self.audio
                    .play_sound(crate::audio::SoundId::PlayerJump, 0.75, 1.0);
                if is_sprinting && wish_dir.length_sq() > 0.01 {
                    self.horiz_velocity = self.horiz_velocity + wish_dir * SPRINT_JUMP_BOOST;
                    self.player.exhaustion += 0.2;
                } else {
                    self.player.exhaustion += 0.05;
                }
            }
        }

        let old_y = self.camera.position.y;
        let motion = Vec3::new(
            self.horiz_velocity.x,
            self.vertical_velocity,
            self.horiz_velocity.z,
        ) * dt;
        let prev_grounded = self.was_on_ground;
        self.on_ground = move_player_with_collision(
            &self.world,
            &mut self.camera.position,
            motion,
            &mut self.vertical_velocity,
            self.current_eye_offset,
            self.current_player_height,
            is_crouching,
        );
        self.was_on_ground = self.on_ground;
        if !prev_grounded && self.on_ground && self.player.fall_distance > 0.4 {
            self.audio
                .play_sound(crate::audio::SoundId::PlayerLand, 0.85, 1.0);
        }
        if self.vertical_velocity < 0.0 && !self.fly_mode && !in_water && !in_lava {
            self.player.fall_distance += old_y - self.camera.position.y;
        } else if self.vertical_velocity > 0.0 || self.fly_mode || in_water || in_lava {
            self.player.fall_distance = 0.0;
        }

        self.update_ambient_and_biome(dt, is_sprinting, in_water);
        self.update_player_vitals(dt, wish_dir, is_sprinting, feet_pos, eye_block, in_water, in_lava, out);
    }

    pub(super) fn update_post_movement_bounds_and_hotbar(&mut self, dt: f32, input: &InputState) {
        if self.player.invulnerability_timer > 0.0 {
            self.player.invulnerability_timer -= dt;
        }
        if self.damage_tilt_timer > 0.0 {
            self.damage_tilt_timer = (self.damage_tilt_timer - dt).max(0.0);
        }

        if self.camera.position.y < -32.0 {
            if self.player.take_damage(20.0) {
                self.damage_tilt_timer = 0.4;
                self.request_rumble(1.0, 1.0, 300);
                crate::log_info!(
                    "PLAYER",
                    "Player took 20.0 void damage at y={:.1} (health: {:.1}/{:.1})",
                    self.camera.position.y,
                    self.player.health,
                    self.player.max_health
                );
            }
            if !self.player.is_dead() {
                self.camera.position.y = 80.0;
                self.vertical_velocity = 0.0;
            }
        }
        if self.camera.position.y > 250.0 {
            self.camera.position.y = 250.0;
            self.vertical_velocity = self.vertical_velocity.min(0.0);
        }

        if !self.chat.is_open {
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

            for i in 0..10 {
                if input.key_digits[i] {
                    self.player.selected_hotbar_slot = i as u8;
                }
            }
        }

        self.player.update_held_item_tracker(dt);
    }
}
