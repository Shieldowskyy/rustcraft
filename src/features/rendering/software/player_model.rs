//! Articulated 3rd-person player model, armor overlays, remote player, and floating nametag rendering.
//! Zero platform `#[cfg]`.

use crate::core::math::{Mat4, Vec3};
use crate::features::blocks::BlockId;
use crate::features::physics::STANDING_EYE_OFFSET;
use crate::features::player::{Camera, ItemStack};
use crate::features::rendering::{emit_player_model_parts, font};

use super::types::Renderer;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub fn draw_player_model(
        &mut self,
        pos: Vec3,
        yaw: f32,
        pitch: f32,
        walk_phase: f32,
        swing_progress: f32,
        is_sneaking: bool,
        hurt_tint: f32,
        nickname: Option<&str>,
        held_block: Option<BlockId>,
        armor: &[Option<ItemStack>; 4],
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let camera_pos = camera.position;
        let held_pose = emit_player_model_parts(
            pos,
            yaw,
            pitch,
            walk_phase,
            swing_progress,
            is_sneaking,
            held_block,
            armor,
            |part| {
                self.draw_oriented_box(
                    part.origin,
                    part.center,
                    part.size,
                    part.forward,
                    part.right,
                    part.up,
                    part.tex_top,
                    part.tex_bottom,
                    part.tex_front,
                    part.tex_back,
                    part.tex_left,
                    part.tex_right,
                    hurt_tint,
                    camera_pos,
                    view_proj,
                );
            },
        );

        if let Some(held) = held_pose {
            self.draw_item_3d_oriented(
                held.block,
                held.pos,
                held.right,
                held.up,
                held.forward,
                0.85,
                view_proj,
            );
        }

        if let Some(nick) = nickname {
            self.draw_floating_nametag(pos, nick, view_proj);
        }
    }

    /// Projects and draws a floating player nametag above the given feet position into `color_buffer`.
    pub fn draw_floating_nametag(&mut self, feet_pos: Vec3, nickname: &str, view_proj: &Mat4) {
        let tag_pos = feet_pos + Vec3::new(0.0, 2.22, 0.0);
        let clip = view_proj.transform_point(tag_pos);
        if clip[3] > 0.2 {
            let ndc_x = clip[0] / clip[3];
            let ndc_y = clip[1] / clip[3];
            if (-1.1..=1.1).contains(&ndc_x) && (-1.1..=1.1).contains(&ndc_y) {
                let sx = ((ndc_x * 0.5 + 0.5) * self.width as f32) as i32;
                let sy = ((1.0 - (ndc_y * 0.5 + 0.5)) * self.height as f32) as i32;
                let ntw = font::measure_text(nickname, 1, 1);
                crate::ui::draw::fill_rect_alpha(
                    &mut self.color_buffer,
                    self.width,
                    self.height,
                    sx - ntw / 2 - 4,
                    sy - 10,
                    ntw + 8,
                    14,
                    crate::ui::draw::pack_color(0.0, 0.0, 0.0),
                    0.6,
                );
                crate::ui::draw::draw_text_colored_shadowed(
                    &mut self.color_buffer,
                    self.width,
                    self.height,
                    nickname,
                    sx - ntw / 2,
                    sy - 8,
                    1,
                    1,
                    crate::ui::draw::pack_color(1.0, 1.0, 1.0),
                );
            }
        }
    }

    pub fn draw_remote_player(
        &mut self,
        player: &crate::net::RemotePlayer,
        camera: &Camera,
        view_proj: &Mat4,
    ) {
        let feet_pos = Vec3::new(
            player.pos.x,
            player.pos.y - STANDING_EYE_OFFSET,
            player.pos.z,
        );
        let hurt_tint = if player.hurt_timer > 0.0 { 0.65 } else { 0.0 };
        self.draw_player_model(
            feet_pos,
            player.yaw,
            player.pitch,
            player.walk_phase,
            0.0,
            player.is_sneaking,
            hurt_tint,
            Some(&player.nickname),
            None,
            &[None, None, None, None],
            camera,
            view_proj,
        );
    }
}
