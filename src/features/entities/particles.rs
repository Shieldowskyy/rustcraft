//! Client-side cosmetic particle system for block breaking, explosions, combat, drowning, and breeding.
//! Mutates transient visual `Particle` entries; does not alter authoritative gameplay simulation.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::core::math::Vec3;

/// An individual particle instance with position, velocity, RGB tint, lifetime, and size.
#[derive(Clone, Copy)]
pub struct Particle {
    /// World position of the particle.
    pub pos: Vec3,
    /// Velocity vector in units/sec.
    pub vel: Vec3,
    /// RGB color modulation.
    pub color: [f32; 3],
    /// Remaining lifetime in seconds.
    pub life: f32,
    /// Render size in world units.
    pub size: f32,
}

struct ParticleRng(u64);

impl ParticleRng {
    fn from_block_pos(pos: Vec3) -> Self {
        let mut h = DefaultHasher::new();
        (pos.x as i32, pos.y as i32, pos.z as i32).hash(&mut h);
        Self(h.finish())
    }

    fn from_world_pos(pos: Vec3) -> Self {
        let mut h = DefaultHasher::new();
        (pos.x as i32, (pos.y * 100.0) as i32, pos.z as i32).hash(&mut h);
        Self(h.finish())
    }

    #[inline]
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as f32) / (u32::MAX as f32)
    }
}

/// Particle manager managing active particles, physics integration, and emitter helpers.
pub struct ParticleSystem {
    /// Active particle buffer.
    pub particles: Vec<Particle>,
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleSystem {
    /// Creates an empty particle system.
    pub fn new() -> Self {
        ParticleSystem { particles: Vec::new() }
    }

    /// Spawns a burst of particles at `pos` matching the block's face color.
    pub fn spawn_block_break(&mut self, pos: Vec3, color: [f32; 3]) {
        let mut rng = ParticleRng::from_block_pos(pos);
        for _ in 0..12 {
            let vx = (rng.next_f32() - 0.5) * 5.0;
            let vy = rng.next_f32() * 5.0 + 1.0;
            let vz = (rng.next_f32() - 0.5) * 5.0;
            let brightness = 0.75 + rng.next_f32() * 0.5;
            self.particles.push(Particle {
                pos: Vec3::new(pos.x + 0.5, pos.y + 0.5, pos.z + 0.5),
                vel: Vec3::new(vx, vy, vz),
                color: [color[0] * brightness, color[1] * brightness, color[2] * brightness],
                life: 0.4 + rng.next_f32() * 0.3,
                size: 0.05 + rng.next_f32() * 0.06,
            });
        }
    }

    /// Spawns an explosive fire-and-smoke effect (e.g. for TNT).
    pub fn spawn_explosion(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_block_pos(pos);
        for _ in 0..60 {
            let vx = (rng.next_f32() - 0.5) * 16.0;
            let vy = (rng.next_f32() - 0.2) * 16.0;
            let vz = (rng.next_f32() - 0.5) * 16.0;
            let is_fire = rng.next_f32() > 0.3;
            let color = if is_fire {
                [1.0, 0.3 + rng.next_f32() * 0.5, 0.05]
            } else {
                [0.25, 0.25, 0.25]
            };
            self.particles.push(Particle {
                pos: Vec3::new(pos.x + 0.5, pos.y + 0.5, pos.z + 0.5),
                vel: Vec3::new(vx, vy, vz),
                color,
                life: 0.5 + rng.next_f32() * 0.6,
                size: 0.12 + rng.next_f32() * 0.15,
            });
        }
    }

    /// Spawns fire and smoke particles when an item or entity burns in lava.
    pub fn spawn_lava_burn(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_world_pos(pos);
        for _ in 0..16 {
            let vx = (rng.next_f32() - 0.5) * 2.5;
            let vy = rng.next_f32() * 4.0 + 1.5;
            let vz = (rng.next_f32() - 0.5) * 2.5;
            let is_smoke = rng.next_f32() > 0.5;
            let color = if is_smoke {
                [0.2, 0.2, 0.2]
            } else {
                [1.0, 0.25 + rng.next_f32() * 0.45, 0.02]
            };
            self.particles.push(Particle {
                pos,
                vel: Vec3::new(vx, vy, vz),
                color,
                life: 0.35 + rng.next_f32() * 0.35,
                size: 0.08 + rng.next_f32() * 0.08,
            });
        }
    }

    /// Spawns hit impact sparks (critical / damage flashes).
    pub fn spawn_mob_hit(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_world_pos(pos);
        for _ in 0..10 {
            let vx = (rng.next_f32() - 0.5) * 4.0;
            let vy = rng.next_f32() * 3.5 + 1.0;
            let vz = (rng.next_f32() - 0.5) * 4.0;
            let is_crit = rng.next_f32() > 0.4;
            let color = if is_crit { [0.95, 0.20, 0.15] } else { [0.95, 0.85, 0.20] };
            self.particles.push(Particle {
                pos,
                vel: Vec3::new(vx, vy, vz),
                color,
                life: 0.3 + rng.next_f32() * 0.25,
                size: 0.06 + rng.next_f32() * 0.05,
            });
        }
    }

    /// Spawns a Minecraft-style white smoke cloud ("poof") upon mob death.
    pub fn spawn_mob_death_poof(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_world_pos(pos);
        for _ in 0..30 {
            let vx = (rng.next_f32() - 0.5) * 3.2;
            let vy = rng.next_f32() * 2.2 + 0.4;
            let vz = (rng.next_f32() - 0.5) * 3.2;
            let shade = 0.75 + rng.next_f32() * 0.24;
            self.particles.push(Particle {
                pos: Vec3::new(
                    pos.x + (rng.next_f32() - 0.5) * 0.4,
                    pos.y + (rng.next_f32() - 0.5) * 0.4,
                    pos.z + (rng.next_f32() - 0.5) * 0.4,
                ),
                vel: Vec3::new(vx, vy, vz),
                color: [shade, shade, shade],
                life: 0.6 + rng.next_f32() * 0.5,
                size: 0.14 + rng.next_f32() * 0.12,
            });
        }
    }

    /// Spawns water bubble particles when an entity is submerged and drowning.
    pub fn spawn_drown_bubbles(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_world_pos(pos);
        for _ in 0..12 {
            let vx = (rng.next_f32() - 0.5) * 1.6;
            let vy = rng.next_f32() * 2.2 + 0.8;
            let vz = (rng.next_f32() - 0.5) * 1.6;
            self.particles.push(Particle {
                pos: Vec3::new(
                    pos.x + (rng.next_f32() - 0.5) * 0.5,
                    pos.y + (rng.next_f32() - 0.5) * 0.3 - 0.2,
                    pos.z + (rng.next_f32() - 0.5) * 0.5,
                ),
                vel: Vec3::new(vx, vy, vz),
                color: [0.65 + rng.next_f32() * 0.3, 0.88 + rng.next_f32() * 0.12, 1.0],
                life: 0.5 + rng.next_f32() * 0.4,
                size: 0.06 + rng.next_f32() * 0.05,
            });
        }
    }

    /// Spawns floating heart particles for animal breeding/love mode.
    pub fn spawn_hearts(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_world_pos(pos);
        for _ in 0..7 {
            let vx = (rng.next_f32() - 0.5) * 0.8;
            let vy = rng.next_f32() * 1.5 + 0.8;
            let vz = (rng.next_f32() - 0.5) * 0.8;
            self.particles.push(Particle {
                pos: Vec3::new(
                    pos.x + (rng.next_f32() - 0.5) * 0.4,
                    pos.y + rng.next_f32() * 0.4,
                    pos.z + (rng.next_f32() - 0.5) * 0.4,
                ),
                vel: Vec3::new(vx, vy, vz),
                color: [0.95, 0.18 + rng.next_f32() * 0.1, 0.32 + rng.next_f32() * 0.1],
                life: 0.8 + rng.next_f32() * 0.5,
                size: 0.09 + rng.next_f32() * 0.05,
            });
        }
    }

    /// Spawns green growth sparkle particles for bonemeal or baby growth.
    pub fn spawn_growth(&mut self, pos: Vec3) {
        let mut rng = ParticleRng::from_world_pos(pos);
        for _ in 0..14 {
            let vx = (rng.next_f32() - 0.5) * 1.2;
            let vy = rng.next_f32() * 2.0 + 0.6;
            let vz = (rng.next_f32() - 0.5) * 1.2;
            let is_gold = rng.next_f32() > 0.6;
            let color = if is_gold { [0.92, 0.88, 0.25] } else { [0.22, 0.92, 0.32] };
            self.particles.push(Particle {
                pos: Vec3::new(
                    pos.x + (rng.next_f32() - 0.5) * 0.6,
                    pos.y + rng.next_f32() * 0.5,
                    pos.z + (rng.next_f32() - 0.5) * 0.6,
                ),
                vel: Vec3::new(vx, vy, vz),
                color,
                life: 0.6 + rng.next_f32() * 0.4,
                size: 0.07 + rng.next_f32() * 0.05,
            });
        }
    }

    /// Advances simulation of all active particles by `dt` seconds, applying gravity and drag.
    pub fn update(&mut self, dt: f32) {
        const GRAVITY: f32 = 8.0;
        self.particles.retain_mut(|p| {
            p.vel.y -= GRAVITY * dt;
            p.vel.x *= (1.0 - 1.5 * dt).max(0.0);
            p.vel.z *= (1.0 - 1.5 * dt).max(0.0);
            p.pos = p.pos + p.vel * dt;
            p.life -= dt;
            p.life > 0.0
        });
    }
}
