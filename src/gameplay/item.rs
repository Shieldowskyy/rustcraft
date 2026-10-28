use crate::core::math::Vec3;
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

