//! Android multi-touch state tracker, virtual analog joystick, and button/gamepad accumulators.
//! Maps raw pointer down/move/up events to movement vectors and virtual button states.

#![allow(dead_code)]

use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// 2. Android Multi-Touch & Virtual Controls System
// ============================================================================

pub const AMOTION_EVENT_ACTION_DOWN: i32 = 0;
pub const AMOTION_EVENT_ACTION_UP: i32 = 1;
pub const AMOTION_EVENT_ACTION_MOVE: i32 = 2;
pub const AMOTION_EVENT_ACTION_CANCEL: i32 = 3;
pub const AMOTION_EVENT_ACTION_POINTER_DOWN: i32 = 5;
pub const AMOTION_EVENT_ACTION_POINTER_UP: i32 = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum TouchRole {
    MovementJoystick { origin_x: f32, origin_y: f32 },
    CameraLook {
        start_x: f32,
        start_y: f32,
        last_x: f32,
        last_y: f32,
        start_time: Instant,
        is_dragging: bool,
    },
    JumpButton,
    SneakButton,
    SprintButton,
    MineButton,
    PlaceButton,
    MenuButton,
    InventoryButton,
    ChatButton,
    FlyButton,
    CamButton,
    HotbarSlot(u8),
    OffhandSlot,
    UiTouch,
    CloseMenuButton,
}

pub struct TouchTracker {
    pub(super) touches: HashMap<i32, (f32, f32)>,
    pub(super) roles: HashMap<i32, TouchRole>,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub primary_touch_pos: Option<(f32, f32)>,
    pub primary_touch_down: bool,

    // Button states
    pub btn_jump: bool,
    pub btn_sneak: bool,
    pub btn_sprint: bool,
    pub btn_mine: bool,
    pub btn_place_held: bool,
    pub tap_place_pulse: bool,
    pub offhand_swap_just: bool,
    pub menu_right_click_held: bool,
    pub btn_menu_just: bool,
    pub btn_inv_just: bool,
    pub btn_chat_just: bool,
    pub btn_fly_just: bool,
    pub btn_cam_just: bool,
    pub selected_hotbar: Option<u8>,

    pub key_jump: bool,
    pub key_sneak: bool,
    pub key_mine: bool,
    pub key_place_held: bool,
    pub key_move_forward: bool,
    pub key_move_backward: bool,
    pub key_move_left: bool,
    pub key_move_right: bool,

    // Movement vector from virtual joystick
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub stick_x: f32,
    pub stick_y: f32,
    pub stick_sprinting: bool,

    // Physical gamepad analog axes
    pub gp_axis_lx: f32,
    pub gp_axis_ly: f32,
    pub gp_axis_rx: f32,
    pub gp_axis_ry: f32,
    pub gp_axis_lt: f32,
    pub gp_axis_rt: f32,
    pub gp_axis_hat_x: f32,
    pub gp_axis_hat_y: f32,

    // Text input & keys from Android IME / physical keyboard
    pub pending_typed_chars: Vec<char>,
    pub key_backspace_just: bool,
    pub key_enter_just: bool,
}

impl Default for TouchTracker {
    fn default() -> Self {
        Self {
            touches: HashMap::new(),
            roles: HashMap::new(),
            mouse_dx: 0.0,
            mouse_dy: 0.0,
            primary_touch_pos: None,
            primary_touch_down: false,
            btn_jump: false,
            btn_sneak: false,
            btn_sprint: false,
            btn_mine: false,
            btn_place_held: false,
            tap_place_pulse: false,
            offhand_swap_just: false,
            menu_right_click_held: false,
            btn_menu_just: false,
            btn_inv_just: false,
            btn_chat_just: false,
            btn_fly_just: false,
            btn_cam_just: false,
            selected_hotbar: None,
            key_jump: false,
            key_sneak: false,
            key_mine: false,
            key_place_held: false,
            key_move_forward: false,
            key_move_backward: false,
            key_move_left: false,
            key_move_right: false,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            stick_x: 0.0,
            stick_y: 0.0,
            stick_sprinting: false,
            gp_axis_lx: 0.0,
            gp_axis_ly: 0.0,
            gp_axis_rx: 0.0,
            gp_axis_ry: 0.0,
            gp_axis_lt: 0.0,
            gp_axis_rt: 0.0,
            gp_axis_hat_x: 0.0,
            gp_axis_hat_y: 0.0,
            pending_typed_chars: Vec::new(),
            key_backspace_just: false,
            key_enter_just: false,
        }
    }
}

impl TouchTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates continuous hold state (e.g. hold-to-mine when touching the screen without dragging)
    pub fn update_step(&mut self) {
        self.refresh_held_buttons();
    }

    pub fn handle_touch_event(
        &mut self,
        action: i32,
        pointer_id: i32,
        x: f32,
        y: f32,
        screen_w: f32,
        screen_h: f32,
        frame_w: f32,
        frame_h: f32,
        in_menu: bool,
    ) {
        // Map native screen coordinates to RustCraft virtual render frame
        let norm_x = if screen_w > 0.0 { (x / screen_w) * frame_w } else { x };
        let norm_y = if screen_h > 0.0 { (y / screen_h) * frame_h } else { y };

        let masked_action = action & 0xFF;

        match masked_action {
            AMOTION_EVENT_ACTION_DOWN | AMOTION_EVENT_ACTION_POINTER_DOWN => {
                self.touches.insert(pointer_id, (norm_x, norm_y));
                self.primary_touch_pos = Some((norm_x, norm_y));
                self.primary_touch_down = true;

                if in_menu {
                    // Check top-right close / back button in menus
                    if norm_x >= frame_w - 85.0 && norm_y <= 42.0 {
                        self.btn_menu_just = true;
                        self.btn_inv_just = true;
                        self.roles.insert(pointer_id, TouchRole::CloseMenuButton);
                    } else {
                        // Two-finger touch in inventory/menus triggers right-click (split stack, quick-equip)
                        if self.touches.len() > 1 {
                            self.menu_right_click_held = true;
                        }
                        self.roles.insert(pointer_id, TouchRole::UiTouch);
                    }
                    return;
                }

                // Check UI / Virtual buttons
                let role = self.classify_touch_down(norm_x, norm_y, frame_w, frame_h);
                if let TouchRole::MovementJoystick { origin_x, origin_y } = role {
                    self.update_joystick(norm_x - origin_x, norm_y - origin_y);
                }
                self.apply_role_press(role);
                self.roles.insert(pointer_id, role);
            }
            AMOTION_EVENT_ACTION_MOVE => {
                self.touches.insert(pointer_id, (norm_x, norm_y));
                if in_menu {
                    self.primary_touch_pos = Some((norm_x, norm_y));
                    return;
                }

                if let Some(role) = self.roles.get_mut(&pointer_id) {
                    match role {
                        TouchRole::MovementJoystick { origin_x, origin_y } => {
                            let dx = norm_x - *origin_x;
                            let dy = norm_y - *origin_y;
                            self.update_joystick(dx, dy);
                        }
                        TouchRole::CameraLook {
                            start_x,
                            start_y,
                            last_x,
                            last_y,
                            is_dragging,
                            ..
                        } => {
                            let total_dist = (norm_x - *start_x).hypot(norm_y - *start_y);
                            if total_dist > 6.0 {
                                *is_dragging = true;
                            }
                            let dx = norm_x - *last_x;
                            let dy = norm_y - *last_y;
                            self.mouse_dx += dx * 1.20;
                            self.mouse_dy += dy * 1.20;
                            *last_x = norm_x;
                            *last_y = norm_y;
                        }
                        _ => {}
                    }
                }
            }
            AMOTION_EVENT_ACTION_CANCEL => {
                self.cancel_touches();
            }
            AMOTION_EVENT_ACTION_UP | AMOTION_EVENT_ACTION_POINTER_UP => {
                self.touches.remove(&pointer_id);
                if in_menu && self.touches.len() <= 1 {
                    self.menu_right_click_held = false;
                }
                if let Some(role) = self.roles.remove(&pointer_id) {
                    // Tap vs Hold detection for CameraLook
                    if let TouchRole::CameraLook { start_time, is_dragging, .. } = role {
                        if !is_dragging && start_time.elapsed().as_secs_f32() < 0.22 {
                            // Quick tap on world view without drag = Place / Use block or interact!
                            self.tap_place_pulse = true;
                        }
                        if start_time.elapsed().as_secs_f32() >= 0.22 {
                            // Stopped holding mining
                            self.btn_mine = false;
                        }
                    }
                }
                self.refresh_held_buttons();

                if self.touches.is_empty() {
                    self.primary_touch_pos = None;
                    self.primary_touch_down = false;
                    self.reset_movement();
                } else if let Some((_, &(fx, fy))) = self.touches.iter().next() {
                    self.primary_touch_pos = Some((fx, fy));
                }
                self.refresh_joystick();
            }
            _ => {}
        }
    }

    pub(super) fn update_joystick(&mut self, dx: f32, dy: f32) {
        let deadzone = 8.0;
        let max_dist = 52.0;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > deadzone {
            let norm_dx = (dx / max_dist).clamp(-1.0, 1.0);
            let norm_dy = (dy / max_dist).clamp(-1.0, 1.0);
            self.stick_x = norm_dx;
            self.stick_y = norm_dy;
            self.move_forward = norm_dy < -0.25;
            self.move_backward = norm_dy > 0.25;
            self.move_left = norm_dx < -0.25;
            self.move_right = norm_dx > 0.25;
            self.stick_sprinting = norm_dy < -0.85;
        } else {
            self.reset_movement();
        }
    }
}
