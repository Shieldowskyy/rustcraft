use crate::core::math::Vec3;
use crate::graphics::atlas::TextureId;
use crate::world::block::BlockId;
use crate::world::World;

pub struct ItemEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub block: BlockId,
    pub count: u8,
    pub pickup_delay: f32,
    pub age: f32,
}

impl ItemEntity {
    pub fn new(position: Vec3, velocity: Vec3, block: BlockId, count: u8) -> Self {
        Self {
            position,
            velocity,
            block,
            count,
            pickup_delay: 1.0,
            age: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, player_pos: Option<Vec3>) {
        self.age += dt;
        if self.pickup_delay > 0.0 {
            self.pickup_delay -= dt;
        }

        // Magnet attraction towards player if within range and pickup delay is over
        if let Some(target) = player_pos {
            if self.pickup_delay <= 0.0 {
                let to_player = target - self.position;
                let dist_sq = to_player.length_sq();
                if dist_sq < 6.25 && dist_sq > 0.01 { // ~2.5m attraction radius
                    let dist = dist_sq.sqrt();
                    let factor = (1.0 - dist / 2.5).clamp(0.0, 1.0);
                    let pull = to_player.normalized() * (9.0 * factor);
                    self.velocity.x += pull.x * dt;
                    self.velocity.y += (pull.y + 2.0 * factor) * dt;
                    self.velocity.z += pull.z * dt;
                }
            }
        }

        self.velocity.y -= crate::gameplay::physics::GRAVITY * dt;

        let item_half = 0.14;
        let mut p = self.position;
        
        p.y += self.velocity.y * dt;
        if world.box_collides(
            Vec3::new(p.x - item_half, p.y - item_half, p.z - item_half),
            Vec3::new(p.x + item_half, p.y + item_half, p.z + item_half),
        ) {
            p.y -= self.velocity.y * dt;
            self.velocity.y = 0.0;
            self.velocity.x *= 0.6; // Friction
            self.velocity.z *= 0.6;
        } else {
            self.velocity.x *= 0.98; // Air resistance
            self.velocity.z *= 0.98;
        }
        
        p.x += self.velocity.x * dt;
        if world.box_collides(
            Vec3::new(p.x - item_half, p.y - item_half, p.z - item_half),
            Vec3::new(p.x + item_half, p.y + item_half, p.z + item_half),
        ) {
            p.x -= self.velocity.x * dt;
            self.velocity.x = 0.0;
        }

        p.z += self.velocity.z * dt;
        if world.box_collides(
            Vec3::new(p.x - item_half, p.y - item_half, p.z - item_half),
            Vec3::new(p.x + item_half, p.y + item_half, p.z + item_half),
        ) {
            p.z -= self.velocity.z * dt;
            self.velocity.z = 0.0;
        }

        self.position = p;
    }
}

/// Dynamic physical projectile entity for arrows shot by the player or skeleton archers.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub shooter_is_player: bool,
    pub in_ground: bool,
    pub lifetime: f32,
    pub damage: f32,
}

impl ArrowEntity {
    pub fn new(position: Vec3, velocity: Vec3, shooter_is_player: bool, damage: f32) -> Self {
        Self {
            position,
            velocity,
            shooter_is_player,
            in_ground: false,
            lifetime: 0.0,
            damage,
        }
    }

    /// Ticks arrow ballistics, gravity, and voxel collision.
    /// Returns true if the arrow struck a voxel block this tick (transitioned into ground).
    pub fn update(&mut self, dt: f32, world: &World) -> bool {
        self.lifetime += dt;
        if self.in_ground {
            return false;
        }

        // Apply realistic ballistic gravity and air drag
        self.velocity.y -= 18.0 * dt;
        self.velocity.x *= 0.99;
        self.velocity.z *= 0.99;

        let next_pos = self.position + self.velocity * dt;
        let arrow_radius = 0.08;

        if world.box_collides(
            Vec3::new(next_pos.x - arrow_radius, next_pos.y - arrow_radius, next_pos.z - arrow_radius),
            Vec3::new(next_pos.x + arrow_radius, next_pos.y + arrow_radius, next_pos.z + arrow_radius),
        ) {
            self.in_ground = true;
            self.velocity = Vec3::zero();
            return true;
        }

        self.position = next_pos;
        false
    }
}

/// Emits 3D box parts `(offset, half_extents, texture)` for procedural tools and sticks.
pub fn emit_procedural_tool_boxes<F>(
    block: BlockId,
    pos: Vec3,
    tool_right: Vec3,
    tool_up: Vec3,
    scale: f32,
    mut emit: F,
) where
    F: FnMut(Vec3, [f32; 3], TextureId),
{
    let mat_tex = match block {
        BlockId::WoodenPickaxe
        | BlockId::WoodenAxe
        | BlockId::WoodenShovel
        | BlockId::WoodenSword
        | BlockId::WoodenHoe => TextureId::Planks,
        BlockId::StonePickaxe
        | BlockId::StoneAxe
        | BlockId::StoneShovel
        | BlockId::StoneSword
        | BlockId::StoneHoe => TextureId::Stone,
        BlockId::IronPickaxe
        | BlockId::IronAxe
        | BlockId::IronShovel
        | BlockId::IronSword
        | BlockId::IronHoe => TextureId::IronOre,
        BlockId::GoldPickaxe
        | BlockId::GoldAxe
        | BlockId::GoldShovel
        | BlockId::GoldSword
        | BlockId::GoldHoe => TextureId::GoldBlock,
        BlockId::DiamondPickaxe
        | BlockId::DiamondAxe
        | BlockId::DiamondShovel
        | BlockId::DiamondSword
        | BlockId::DiamondHoe => TextureId::DiamondBlock,
        _ => TextureId::WoodSide,
    };
    let handle_tex = TextureId::WoodSide;

    match block {
        BlockId::Stick => {
            emit(pos, [0.016 * scale, 0.26 * scale, 0.016 * scale], handle_tex);
        }
        BlockId::WoodenSword
        | BlockId::StoneSword
        | BlockId::IronSword
        | BlockId::GoldSword
        | BlockId::DiamondSword => {
            emit(pos - tool_up * (0.16 * scale), [0.024 * scale, 0.018 * scale, 0.024 * scale], handle_tex);
            emit(pos - tool_up * (0.08 * scale), [0.016 * scale, 0.065 * scale, 0.016 * scale], handle_tex);
            emit(pos, [0.085 * scale, 0.016 * scale, 0.026 * scale], mat_tex);
            emit(pos + tool_up * (0.20 * scale), [0.038 * scale, 0.180 * scale, 0.010 * scale], mat_tex);
            emit(pos + tool_up * (0.41 * scale), [0.020 * scale, 0.040 * scale, 0.008 * scale], mat_tex);
        }
        BlockId::WoodenPickaxe
        | BlockId::StonePickaxe
        | BlockId::IronPickaxe
        | BlockId::GoldPickaxe
        | BlockId::DiamondPickaxe => {
            emit(pos, [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            emit(pos + tool_up * (0.26 * scale), [0.032 * scale, 0.032 * scale, 0.032 * scale], mat_tex);
            emit(pos + tool_up * (0.27 * scale), [0.130 * scale, 0.024 * scale, 0.024 * scale], mat_tex);
            emit(pos + tool_up * (0.23 * scale) - tool_right * (0.12 * scale), [0.018 * scale, 0.032 * scale, 0.018 * scale], mat_tex);
            emit(pos + tool_up * (0.23 * scale) + tool_right * (0.12 * scale), [0.018 * scale, 0.032 * scale, 0.018 * scale], mat_tex);
        }
        BlockId::WoodenAxe
        | BlockId::StoneAxe
        | BlockId::IronAxe
        | BlockId::GoldAxe
        | BlockId::DiamondAxe => {
            emit(pos, [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            emit(pos + tool_up * (0.22 * scale) + tool_right * (0.045 * scale), [0.048 * scale, 0.065 * scale, 0.022 * scale], mat_tex);
            emit(pos + tool_up * (0.21 * scale) + tool_right * (0.088 * scale), [0.014 * scale, 0.078 * scale, 0.012 * scale], mat_tex);
            emit(pos + tool_up * (0.25 * scale) - tool_right * (0.024 * scale), [0.020 * scale, 0.028 * scale, 0.020 * scale], mat_tex);
        }
        BlockId::WoodenShovel
        | BlockId::StoneShovel
        | BlockId::IronShovel
        | BlockId::GoldShovel
        | BlockId::DiamondShovel => {
            emit(pos, [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            emit(pos - tool_up * (0.27 * scale), [0.042 * scale, 0.014 * scale, 0.018 * scale], handle_tex);
            emit(pos + tool_up * (0.33 * scale), [0.065 * scale, 0.075 * scale, 0.012 * scale], mat_tex);
            emit(pos + tool_up * (0.42 * scale), [0.038 * scale, 0.018 * scale, 0.010 * scale], mat_tex);
        }
        BlockId::WoodenHoe
        | BlockId::StoneHoe
        | BlockId::IronHoe
        | BlockId::GoldHoe
        | BlockId::DiamondHoe => {
            emit(pos, [0.015 * scale, 0.28 * scale, 0.015 * scale], handle_tex);
            emit(pos + tool_up * (0.26 * scale) + tool_right * (0.04 * scale), [0.08 * scale, 0.024 * scale, 0.024 * scale], mat_tex);
            emit(pos + tool_up * (0.22 * scale) + tool_right * (0.07 * scale), [0.024 * scale, 0.06 * scale, 0.02 * scale], mat_tex);
        }
        _ => {}
    }
}

/// Emits 3D box parts `(center, half_extents, texture)` for raw/cooked meat items.
pub fn emit_meat_boxes<F>(
    block: BlockId,
    pos: Vec3,
    meat_right: Vec3,
    meat_up: Vec3,
    scale: f32,
    mut emit: F,
) where
    F: FnMut(Vec3, [f32; 3], TextureId),
{
    let meat_tex = match block {
        BlockId::RawPorkchop => TextureId::RawPorkchop,
        BlockId::CookedPorkchop => TextureId::CookedPorkchop,
        BlockId::RawBeef => TextureId::RawBeef,
        BlockId::CookedBeef => TextureId::CookedBeef,
        BlockId::RawChicken => TextureId::RawChicken,
        BlockId::CookedChicken => TextureId::CookedChicken,
        _ => TextureId::RawBeef,
    };
    let bone_tex = TextureId::Bone;

    match block {
        BlockId::RawPorkchop | BlockId::CookedPorkchop => {
            emit(pos, [0.065 * scale, 0.080 * scale, 0.024 * scale], meat_tex);
            emit(pos + meat_right * (0.045 * scale) - meat_up * (0.025 * scale), [0.035 * scale, 0.055 * scale, 0.022 * scale], meat_tex);
            emit(pos - meat_right * (0.055 * scale) + meat_up * (0.085 * scale), [0.020 * scale, 0.030 * scale, 0.016 * scale], bone_tex);
        }
        BlockId::RawBeef | BlockId::CookedBeef => {
            emit(pos, [0.085 * scale, 0.070 * scale, 0.026 * scale], meat_tex);
            emit(pos - meat_right * (0.065 * scale) + meat_up * (0.020 * scale), [0.020 * scale, 0.045 * scale, 0.022 * scale], bone_tex);
            emit(pos + meat_right * (0.050 * scale) - meat_up * (0.030 * scale), [0.045 * scale, 0.035 * scale, 0.024 * scale], meat_tex);
        }
        BlockId::RawChicken | BlockId::CookedChicken => {
            emit(pos + meat_up * (0.035 * scale), [0.055 * scale, 0.065 * scale, 0.048 * scale], meat_tex);
            emit(pos - meat_up * (0.055 * scale), [0.016 * scale, 0.040 * scale, 0.016 * scale], bone_tex);
            emit(pos - meat_up * (0.095 * scale), [0.028 * scale, 0.016 * scale, 0.024 * scale], bone_tex);
        }
        _ => {}
    }
}

/// Emits 3D box parts `(center, half_extents, texture)` for a feather item.
pub fn emit_feather_boxes<F>(
    pos: Vec3,
    feather_right: Vec3,
    feather_up: Vec3,
    scale: f32,
    mut emit: F,
) where
    F: FnMut(Vec3, [f32; 3], TextureId),
{
    let quill_tex = TextureId::Bone;
    let feather_tex = TextureId::ChickenFeathers;

    emit(pos + feather_up * (0.015 * scale), [0.008 * scale, 0.140 * scale, 0.008 * scale], quill_tex);
    emit(pos - feather_up * (0.135 * scale), [0.006 * scale, 0.035 * scale, 0.006 * scale], quill_tex);
    emit(pos - feather_up * (0.045 * scale) + feather_right * (0.012 * scale), [0.038 * scale, 0.045 * scale, 0.005 * scale], feather_tex);
    emit(pos + feather_up * (0.040 * scale) + feather_right * (0.016 * scale), [0.052 * scale, 0.055 * scale, 0.005 * scale], feather_tex);
    emit(pos + feather_up * (0.115 * scale) + feather_right * (0.008 * scale), [0.030 * scale, 0.038 * scale, 0.004 * scale], feather_tex);
    emit(pos + feather_up * (0.155 * scale), [0.012 * scale, 0.018 * scale, 0.004 * scale], feather_tex);
}

/// Emits 3D box parts `(center, half_extents, texture)` for an arrow item/projectile.
pub fn emit_arrow_boxes<F>(pos: Vec3, arrow_up: Vec3, scale: f32, mut emit: F)
where
    F: FnMut(Vec3, [f32; 3], TextureId),
{
    emit(pos, [0.008 * scale, 0.15 * scale, 0.008 * scale], TextureId::WoodSide);
    emit(pos + arrow_up * (0.17 * scale), [0.018 * scale, 0.025 * scale, 0.018 * scale], TextureId::IronOre);
    emit(pos - arrow_up * (0.12 * scale), [0.032 * scale, 0.025 * scale, 0.004 * scale], TextureId::ChickenFeathers);
}

/// Emits 3D box parts `(center, half_extents, texture)` for a held or dropped torch.
pub fn emit_torch_boxes<F>(pos: Vec3, up: Vec3, scale: f32, mut emit: F)
where
    F: FnMut(Vec3, [f32; 3], TextureId),
{
    emit(pos, [0.025 * scale, 0.20 * scale, 0.025 * scale], TextureId::Torch);
    emit(pos + up * (0.16 * scale), [0.038 * scale, 0.05 * scale, 0.038 * scale], TextureId::Torch);
}


