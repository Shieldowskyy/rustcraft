// System cząsteczek: efekty rozbicia bloku, eksplozji, trafienia moba
// i "poofu" po jego śmierci. Czysto kosmetyczne - nie wpływa na logikę gry.

use crate::math::Vec3;

pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub color: [f32; 3],
    pub life: f32, // pozostały czas życia w sekundach
    pub size: f32,
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
}

impl ParticleSystem {
    pub fn new() -> Self {
        ParticleSystem { particles: Vec::new() }
    }

    /// Spawn a burst of particles at `pos` with the block's face color.
    pub fn spawn_block_break(&mut self, pos: Vec3, color: [f32; 3]) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut rng = {
            let mut h = DefaultHasher::new();
            (pos.x as i32, pos.y as i32, pos.z as i32).hash(&mut h);
            h.finish()
        };
        let mut next = || -> f32 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng >> 33) as f32) / (u32::MAX as f32)
        };
        for _ in 0..12 {
            let vx = (next() - 0.5) * 5.0;
            let vy = next() * 5.0 + 1.0;
            let vz = (next() - 0.5) * 5.0;
            let brightness = 0.75 + next() * 0.5;
            self.particles.push(Particle {
                pos: Vec3::new(pos.x + 0.5, pos.y + 0.5, pos.z + 0.5),
                vel: Vec3::new(vx, vy, vz),
                color: [color[0] * brightness, color[1] * brightness, color[2] * brightness],
                life: 0.4 + next() * 0.3,
                size: 0.05 + next() * 0.06,
            });
        }
    }

    /// Efektowna eksplozja ogniowo-dymna
    pub fn spawn_explosion(&mut self, pos: Vec3) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut rng = {
            let mut h = DefaultHasher::new();
            (pos.x as i32, pos.y as i32, pos.z as i32).hash(&mut h);
            h.finish()
        };
        let mut next = || -> f32 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng >> 33) as f32) / (u32::MAX as f32)
        };

        for _ in 0..60 {
            let vx = (next() - 0.5) * 16.0;
            let vy = (next() - 0.2) * 16.0;
            let vz = (next() - 0.5) * 16.0;
            let is_fire = next() > 0.3;
            let color = if is_fire {
                [1.0, 0.3 + next() * 0.5, 0.05]
            } else {
                [0.25, 0.25, 0.25]
            };
            self.particles.push(Particle {
                pos: Vec3::new(pos.x + 0.5, pos.y + 0.5, pos.z + 0.5),
                vel: Vec3::new(vx, vy, vz),
                color,
                life: 0.5 + next() * 0.6,
                size: 0.12 + next() * 0.15,
            });
        }
    }

    /// Efekt uderzenia moba (krytyczne/czerwone iskry)
    pub fn spawn_mob_hit(&mut self, pos: Vec3) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut rng = {
            let mut h = DefaultHasher::new();
            (pos.x as i32, (pos.y * 100.0) as i32, pos.z as i32).hash(&mut h);
            h.finish()
        };
        let mut next = || -> f32 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng >> 33) as f32) / (u32::MAX as f32)
        };
        for _ in 0..10 {
            let vx = (next() - 0.5) * 4.0;
            let vy = next() * 3.5 + 1.0;
            let vz = (next() - 0.5) * 4.0;
            let is_crit = next() > 0.4;
            let color = if is_crit {
                [0.95, 0.20, 0.15]
            } else {
                [0.95, 0.85, 0.20]
            };
            self.particles.push(Particle {
                pos,
                vel: Vec3::new(vx, vy, vz),
                color,
                life: 0.3 + next() * 0.25,
                size: 0.06 + next() * 0.05,
            });
        }
    }

    /// Minecraftowy obłok dymu („poof”) po wyparowaniu moba
    pub fn spawn_mob_death_poof(&mut self, pos: Vec3) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut rng = {
            let mut h = DefaultHasher::new();
            (pos.x as i32, (pos.y * 100.0) as i32, pos.z as i32).hash(&mut h);
            h.finish()
        };
        let mut next = || -> f32 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((rng >> 33) as f32) / (u32::MAX as f32)
        };
        for _ in 0..30 {
            let vx = (next() - 0.5) * 3.2;
            let vy = next() * 2.2 + 0.4;
            let vz = (next() - 0.5) * 3.2;
            let shade = 0.75 + next() * 0.24;
            self.particles.push(Particle {
                pos: Vec3::new(
                    pos.x + (next() - 0.5) * 0.4,
                    pos.y + (next() - 0.5) * 0.4,
                    pos.z + (next() - 0.5) * 0.4,
                ),
                vel: Vec3::new(vx, vy, vz),
                color: [shade, shade, shade],
                life: 0.6 + next() * 0.5,
                size: 0.14 + next() * 0.12,
            });
        }
    }

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
