use crate::math::Vec3;
use crate::block::BlockId;
use crate::world::World;

pub struct ItemEntity {
    pub position: Vec3,
    pub velocity: Vec3,
    pub block: BlockId,
    pub count: u8,
    pub pickup_delay: f32, // Delay before it can be picked up
    pub age: f32,          // For despawning and floating animation
}

impl ItemEntity {
    pub fn new(position: Vec3, velocity: Vec3, block: BlockId, count: u8) -> Self {
        Self {
            position,
            velocity,
            block,
            count,
            pickup_delay: 1.0, // 1 second before it can be picked up
            age: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32, world: &World) {
        self.age += dt;
        if self.pickup_delay > 0.0 {
            self.pickup_delay -= dt;
        }

        self.velocity.y -= crate::physics::GRAVITY * dt;

        // Simple collision: just check block below/around
        // We can reuse AABB but items are small, let's treat it as a point or small box
        let item_half = 0.125;
        let mut p = self.position;
        
        p.y += self.velocity.y * dt;
        if world.box_collides(
            Vec3::new(p.x - item_half, p.y - item_half, p.z - item_half),
            Vec3::new(p.x + item_half, p.y + item_half, p.z + item_half),
        ) {
            p.y -= self.velocity.y * dt;
            self.velocity.y = 0.0;
            self.velocity.x *= 0.5; // Friction
            self.velocity.z *= 0.5;
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
