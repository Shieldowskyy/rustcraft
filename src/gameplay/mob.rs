use std::f32::consts::PI;
use crate::core::math::{IVec3, Vec3};
use crate::world::block::BlockId;
use crate::world::World;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MobKind {
    Pig,
    Cow,
    Chicken,
    Sheep,
    Zombie,
    Skeleton,
    Reaper,
    Dolphin,
    Cod,
    Salmon,
    Pufferfish,
    TropicalFish,
    Villager,
    IronGolem,
    Spider,
}

impl MobKind {
    pub fn is_aquatic(&self) -> bool {
        matches!(
            self,
            MobKind::Dolphin
                | MobKind::Cod
                | MobKind::Salmon
                | MobKind::Pufferfish
                | MobKind::TropicalFish
        )
    }

    pub fn max_health(&self) -> f32 {
        match self {
            MobKind::Pig => 10.0,
            MobKind::Cow => 10.0,
            MobKind::Chicken => 4.0,
            MobKind::Sheep => 8.0,
            MobKind::Zombie => 20.0,
            MobKind::Skeleton => 20.0,
            MobKind::Reaper => 20.0,
            MobKind::Dolphin => 10.0,
            MobKind::Cod => 3.0,
            MobKind::Salmon => 3.0,
            MobKind::Pufferfish => 3.0,
            MobKind::TropicalFish => 3.0,
            MobKind::Villager => 20.0,
            MobKind::IronGolem => 100.0,
            MobKind::Spider => 16.0,
        }
    }

    pub fn half_width(&self) -> f32 {
        match self {
            MobKind::Pig => 0.4,
            MobKind::Cow => 0.45,
            MobKind::Chicken => 0.25,
            MobKind::Sheep => 0.45,
            MobKind::Zombie => 0.35,
            MobKind::Skeleton => 0.30,
            MobKind::Reaper => 0.30,
            MobKind::Dolphin => 0.38,
            MobKind::Cod => 0.20,
            MobKind::Salmon => 0.24,
            MobKind::Pufferfish => 0.25,
            MobKind::TropicalFish => 0.18,
            MobKind::Villager => 0.32,
            MobKind::IronGolem => 0.65,
            MobKind::Spider => 0.65,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            MobKind::Pig => 0.85,
            MobKind::Cow => 1.35,
            MobKind::Chicken => 0.70,
            MobKind::Sheep => 1.30,
            MobKind::Zombie => 1.85,
            MobKind::Skeleton => 1.85,
            MobKind::Reaper => 1.70,
            MobKind::Dolphin => 0.60,
            MobKind::Cod => 0.30,
            MobKind::Salmon => 0.35,
            MobKind::Pufferfish => 0.35,
            MobKind::TropicalFish => 0.28,
            MobKind::Villager => 1.95,
            MobKind::IronGolem => 2.70,
            MobKind::Spider => 0.50,
        }
    }

    #[allow(dead_code)]
    pub fn is_hostile(&self) -> bool {
        match self {
            MobKind::Pig | MobKind::Cow | MobKind::Chicken | MobKind::Sheep | MobKind::Villager | MobKind::IronGolem => false,
            MobKind::Dolphin | MobKind::Cod | MobKind::Salmon | MobKind::Pufferfish | MobKind::TropicalFish => false,
            MobKind::Zombie | MobKind::Skeleton | MobKind::Reaper | MobKind::Spider => true,
        }
    }

    pub fn is_tempting_item(&self, item: BlockId) -> bool {
        match self {
            MobKind::Cow | MobKind::Sheep => item == BlockId::Wheat,
            MobKind::Pig => matches!(item, BlockId::Carrot | BlockId::Potato | BlockId::BakedPotato),
            MobKind::Chicken => item == BlockId::WheatSeeds,
            MobKind::Dolphin => matches!(item, BlockId::RawCod | BlockId::RawSalmon),
            MobKind::Cod | MobKind::Salmon | MobKind::TropicalFish | MobKind::Pufferfish => {
                matches!(item, BlockId::WheatSeeds | BlockId::Kelp | BlockId::Seagrass)
            }
            MobKind::Villager => matches!(item, BlockId::Emerald | BlockId::EmeraldBlock | BlockId::Bread),
            MobKind::IronGolem => matches!(item, BlockId::IronIngot | BlockId::IronBlock),
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            MobKind::Pig => "Pig",
            MobKind::Cow => "Cow",
            MobKind::Chicken => "Chicken",
            MobKind::Sheep => "Sheep",
            MobKind::Zombie => "Zombie",
            MobKind::Skeleton => "Skeleton",
            MobKind::Reaper => "Reaper",
            MobKind::Dolphin => "Dolphin",
            MobKind::Cod => "Cod",
            MobKind::Salmon => "Salmon",
            MobKind::Pufferfish => "Pufferfish",
            MobKind::TropicalFish => "Tropical Fish",
            MobKind::Villager => "Villager",
            MobKind::IronGolem => "Iron Golem",
            MobKind::Spider => "Spider",
        }
    }

    pub fn idle_sound(&self) -> Option<crate::audio::SoundId> {
        match self {
            MobKind::Pig => Some(crate::audio::SoundId::PigIdle),
            MobKind::Cow => Some(crate::audio::SoundId::CowIdle),
            MobKind::Chicken => Some(crate::audio::SoundId::ChickenIdle),
            MobKind::Sheep => Some(crate::audio::SoundId::SheepIdle),
            MobKind::Zombie => Some(crate::audio::SoundId::ZombieIdle),
            MobKind::Skeleton => Some(crate::audio::SoundId::SkeletonIdle),
            MobKind::Reaper => None,
            MobKind::Dolphin => Some(crate::audio::SoundId::DolphinIdle),
            MobKind::Villager => Some(crate::audio::SoundId::VillagerIdle),
            MobKind::IronGolem => Some(crate::audio::SoundId::IronGolemIdle),
            MobKind::Spider => Some(crate::audio::SoundId::SpiderIdle),
            MobKind::Cod | MobKind::Salmon | MobKind::Pufferfish | MobKind::TropicalFish => None,
        }
    }

    pub fn hurt_sound(&self) -> crate::audio::SoundId {
        match self {
            MobKind::Pig => crate::audio::SoundId::PigHurt,
            MobKind::Cow => crate::audio::SoundId::CowHurt,
            MobKind::Chicken => crate::audio::SoundId::ChickenHurt,
            MobKind::Sheep => crate::audio::SoundId::SheepHurt,
            MobKind::Zombie => crate::audio::SoundId::ZombieHurt,
            MobKind::Skeleton => crate::audio::SoundId::SkeletonHurt,
            MobKind::Reaper => crate::audio::SoundId::ReaperHurt,
            MobKind::Dolphin => crate::audio::SoundId::DolphinHurt,
            MobKind::Villager => crate::audio::SoundId::VillagerHurt,
            MobKind::IronGolem => crate::audio::SoundId::IronGolemHurt,
            MobKind::Spider => crate::audio::SoundId::SpiderHurt,
            MobKind::Cod | MobKind::Salmon | MobKind::Pufferfish | MobKind::TropicalFish => crate::audio::SoundId::FishFlop,
        }
    }

    pub fn death_sound(&self) -> crate::audio::SoundId {
        match self {
            MobKind::Pig => crate::audio::SoundId::PigDeath,
            MobKind::Cow => crate::audio::SoundId::CowDeath,
            MobKind::Chicken => crate::audio::SoundId::ChickenDeath,
            MobKind::Sheep => crate::audio::SoundId::SheepDeath,
            MobKind::Zombie => crate::audio::SoundId::ZombieDeath,
            MobKind::Skeleton => crate::audio::SoundId::SkeletonDeath,
            MobKind::Reaper => crate::audio::SoundId::ReaperDeath,
            MobKind::Dolphin => crate::audio::SoundId::DolphinDeath,
            MobKind::Villager => crate::audio::SoundId::VillagerDeath,
            MobKind::IronGolem => crate::audio::SoundId::IronGolemDeath,
            MobKind::Spider => crate::audio::SoundId::SpiderDeath,
            MobKind::Cod | MobKind::Salmon | MobKind::Pufferfish | MobKind::TropicalFish => crate::audio::SoundId::FishFlop,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mob {
    #[allow(dead_code)]
    pub id: u32,
    pub kind: MobKind,
    pub pos: Vec3,          // Feet position
    pub velocity: Vec3,
    pub yaw: f32,           // Facing angle in radians
    pub head_yaw_offset: f32, // Head relative look angle
    pub health: f32,
    pub hurt_timer: f32,    // Flashes red when > 0.0
    pub death_timer: f32,   // Ticks up when dying (0.0 to 0.8s)
    pub is_dead: bool,
    pub walk_phase: f32,
    pub on_ground: bool,
    pub ai_timer: f32,
    pub ai_state_is_moving: bool,
    pub target_yaw: f32,
    pub panic_timer: f32,
    pub despawn_pending: bool,
    pub ambient_sound_timer: f32,
    pub pending_sound: Option<crate::audio::SoundId>,
    pub fuse: f32,
    pub is_ignited: bool,
    pub in_love: f32,
    pub breed_cooldown: f32,
    pub is_baby: bool,
    pub age: f32,
    pub love_particle_timer: f32,
    pub wool_color: u8,
    pub is_sheared: bool,
    pub eat_grass_timer: f32,
    pub is_puffed: bool,
    pub puff_timer: f32,
    pub swim_pitch: f32,
    pub jump_timer: f32,
    pub out_of_water_timer: f32,
    pub fish_variant: u8,
    pub villager_profession: u8,
    pub trades: Vec<crate::gameplay::trading::TradeOffer>,
    pub attack_cooldown: f32,
    pub home_pos: Option<Vec3>,
}

impl Mob {
    pub fn new(id: u32, kind: MobKind, pos: Vec3) -> Self {
        let max_hp = kind.max_health();
        let initial_ambient = 3.0 + ((id.wrapping_mul(2654435761) % 1000) as f32 / 1000.0) * 8.0;
        let wool_color = if kind == MobKind::Sheep {
            let roll = (id.wrapping_mul(1664525) % 100) as u8;
            if roll < 80 {
                0 // White (80%)
            } else if roll < 85 {
                8 // Light Gray (5%)
            } else if roll < 90 {
                7 // Gray (5%)
            } else if roll < 95 {
                15 // Black (5%)
            } else if roll < 98 {
                12 // Brown (3%)
            } else {
                6 // Pink (2% - rare!)
            }
        } else {
            0
        };
        let villager_profession = if kind == MobKind::Villager {
            (id % 5) as u8 // Farmer, Librarian, Blacksmith, Cleric, Butcher
        } else {
            0
        };
        let trades = if kind == MobKind::Villager {
            crate::gameplay::trading::VillagerProfession::from_u8(villager_profession).default_trades()
        } else {
            Vec::new()
        };
        Mob {
            id,
            kind,
            pos,
            velocity: Vec3::zero(),
            yaw: 0.0,
            head_yaw_offset: 0.0,
            health: max_hp,
            hurt_timer: 0.0,
            death_timer: 0.0,
            is_dead: false,
            walk_phase: 0.0,
            on_ground: false,
            ai_timer: 1.0,
            ai_state_is_moving: false,
            target_yaw: 0.0,
            panic_timer: 0.0,
            despawn_pending: false,
            ambient_sound_timer: initial_ambient,
            pending_sound: None,
            fuse: 0.0,
            is_ignited: false,
            in_love: 0.0,
            breed_cooldown: 0.0,
            is_baby: false,
            age: 0.0,
            love_particle_timer: 0.0,
            wool_color,
            is_sheared: false,
            eat_grass_timer: 0.0,
            is_puffed: false,
            puff_timer: 0.0,
            swim_pitch: 0.0,
            jump_timer: 3.0 + ((id % 7) as f32),
            out_of_water_timer: 0.0,
            fish_variant: (id % 4) as u8,
            villager_profession,
            trades,
            attack_cooldown: 0.0,
            home_pos: (kind == MobKind::Villager).then_some(pos),
        }
    }

    pub fn aabb(&self) -> (Vec3, Vec3) {
        let scale = if self.is_baby { 0.5 } else { 1.0 };
        let (hw, h) = if self.kind == MobKind::Pufferfish && self.is_puffed {
            (0.46 * scale, 0.65 * scale)
        } else {
            (self.kind.half_width() * scale, self.kind.height() * scale)
        };
        (
            Vec3::new(self.pos.x - hw, self.pos.y, self.pos.z - hw),
            Vec3::new(self.pos.x + hw, self.pos.y + h, self.pos.z + hw),
        )
    }

    /// Center of mass / torso for particle effects and targeting
    pub fn center_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.pos.y + self.kind.height() * 0.5, self.pos.z)
    }

    pub fn take_damage(&mut self, damage: f32, knockback_dir: Vec3) {
        if self.is_dead {
            return;
        }
        self.health -= damage;
        self.hurt_timer = 0.35;
        self.panic_timer = 5.0;

        let kb = if knockback_dir.length_sq() > 1e-4 {
            knockback_dir.normalized()
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let (kb_horiz, kb_vert) = if self.kind == MobKind::IronGolem {
            (0.4, 0.4)
        } else {
            (2.2, 1.6)
        };
        self.velocity.x += kb.x * kb_horiz;
        self.velocity.z += kb.z * kb_horiz;
        self.velocity.y = kb_vert;
        self.on_ground = false;

        if self.health <= 0.0 {
            self.health = 0.0;
            self.is_dead = true;
            self.death_timer = 0.0;
            self.pending_sound = Some(self.kind.death_sound());
        } else {
            self.pending_sound = Some(self.kind.hurt_sound());
        }
    }

    pub fn update(&mut self, dt: f32, world: &World, rng: &mut u32, player_pos: Option<Vec3>, player_held: Option<BlockId>) {
        let rand_f32 = |state: &mut u32| -> f32 {
            *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (*state as f32) / (u32::MAX as f32)
        };

        if self.in_love > 0.0 {
            self.in_love = (self.in_love - dt).max(0.0);
        }
        if self.breed_cooldown > 0.0 {
            self.breed_cooldown = (self.breed_cooldown - dt).max(0.0);
        }
        if self.is_baby {
            self.age += dt;
            if self.age >= 0.0 {
                self.is_baby = false;
            }
        }
        if self.eat_grass_timer > 0.0 {
            self.eat_grass_timer = (self.eat_grass_timer - dt).max(0.0);
            if self.eat_grass_timer == 0.0 {
                self.is_sheared = false;
            }
        }

        if !self.is_dead {
            self.ambient_sound_timer -= dt;
            if self.ambient_sound_timer <= 0.0 {
                self.ambient_sound_timer = 7.0 + rand_f32(rng) * 9.0;
                if self.pending_sound.is_none() {
                    self.pending_sound = self.kind.idle_sound();
                }
            }
        }

        if self.hurt_timer > 0.0 {
            self.hurt_timer = (self.hurt_timer - dt).max(0.0);
        }

        if self.is_dead {
            self.death_timer += dt;
            self.velocity.y -= 22.0 * dt;
            self.apply_movement(dt, world);
            if self.death_timer >= 0.8 {
                self.despawn_pending = true;
            }
            return;
        }

        let (min, max) = self.aabb();
        let min_x = min.x.floor() as i32;
        let max_x = max.x.floor() as i32;
        let min_y = min.y.floor() as i32;
        let max_y = max.y.floor() as i32;
        let min_z = min.z.floor() as i32;
        let max_z = max.z.floor() as i32;

        let mut in_lava = false;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    if world.get_block(IVec3::new(x, y, z)) == BlockId::Lava {
                        in_lava = true;
                        break;
                    }
                }
                if in_lava { break; }
            }
            if in_lava { break; }
        }

        if in_lava {
            self.velocity.x *= 0.8;
            self.velocity.z *= 0.8;
            if self.velocity.y < -3.0 {
                self.velocity.y = -3.0;
            }

            if self.hurt_timer <= 0.0 {
                let rx = (rand_f32(rng) - 0.5) * 0.4;
                let rz = (rand_f32(rng) - 0.5) * 0.4;
                self.take_damage(4.0, Vec3::new(rx, 0.8, rz));
            }
        }

        // Sunlight burning for undead hostile mobs (Zombie, Skeleton)
        if (self.kind == MobKind::Zombie || self.kind == MobKind::Skeleton) && world.sunlight_factor() > 0.65 {
            let ix = self.pos.x.floor() as i32;
            let iy = (self.pos.y + self.kind.height()).ceil() as i32;
            let iz = self.pos.z.floor() as i32;
            let mut covered = false;
            for y in iy..=64 {
                let blk = world.get_block(IVec3::new(ix, y, iz));
                if blk.is_solid() || blk == BlockId::Leaves || blk == BlockId::SnowLeaves {
                    covered = true;
                    break;
                }
            }
            if !covered && self.hurt_timer <= 0.0 {
                self.take_damage(2.0, Vec3::new(0.0, 0.5, 0.0));
            }
        }

        let mut in_water = false;
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    let b = world.get_block(IVec3::new(x, y, z));
                    if matches!(b, BlockId::Water | BlockId::Kelp | BlockId::Seagrass) {
                        in_water = true;
                        break;
                    }
                }
                if in_water { break; }
            }
            if in_water { break; }
        }

        // Aquatic mobs 3D swimming & flopping AI
        if self.kind.is_aquatic() {
            if in_water {
                self.out_of_water_timer = 0.0;
                self.ai_timer -= dt;
                if self.ai_timer <= 0.0 {
                    let roll = rand_f32(rng);
                    if roll < 0.22 {
                        self.ai_state_is_moving = false;
                        self.ai_timer = 1.0 + rand_f32(rng) * 2.0;
                    } else {
                        self.ai_state_is_moving = true;
                        self.target_yaw = rand_f32(rng) * PI * 2.0;
                        self.ai_timer = 2.0 + rand_f32(rng) * 4.0;
                    }
                }

                if let Some(target) = player_pos {
                    let dx = target.x - self.pos.x;
                    let dy = target.y - self.pos.y;
                    let dz = target.z - self.pos.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    if self.kind == MobKind::Pufferfish {
                        if dist_sq < 3.2 * 3.2 {
                            self.is_puffed = true;
                            self.puff_timer = 2.5;
                        }
                    }

                    if self.kind == MobKind::Dolphin {
                        if dist_sq < 14.0 * 14.0 && dist_sq > 2.5 * 2.5 {
                            self.target_yaw = dx.atan2(dz) + (rand_f32(rng) - 0.5) * 0.4;
                            self.ai_state_is_moving = true;
                        }
                    }
                }

                if self.puff_timer > 0.0 {
                    self.puff_timer -= dt;
                    if self.puff_timer <= 0.0 {
                        self.is_puffed = false;
                    }
                }

                if self.kind == MobKind::Dolphin {
                    self.jump_timer -= dt;
                    let above_pos = (self.pos + Vec3::new(0.0, 1.2, 0.0)).floor_to_ivec();
                    if self.jump_timer <= 0.0 && world.get_block(above_pos) == BlockId::Air && self.ai_state_is_moving {
                        self.jump_timer = 4.5 + rand_f32(rng) * 5.0;
                        self.velocity.y = 5.2;
                        self.pending_sound = Some(crate::audio::SoundId::DolphinJump);
                    }
                }

                let mut diff = self.target_yaw - self.yaw;
                while diff > PI { diff -= PI * 2.0; }
                while diff < -PI { diff += PI * 2.0; }
                self.yaw += diff * (4.5 * dt).min(1.0);

                let swim_speed = match self.kind {
                    MobKind::Dolphin => if self.ai_state_is_moving { 4.2 } else { 1.5 },
                    MobKind::Salmon => if self.ai_state_is_moving { 2.4 } else { 0.9 },
                    _ => if self.ai_state_is_moving { 1.8 } else { 0.6 },
                };

                let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
                let target_vx = forward.x * swim_speed;
                let target_vz = forward.z * swim_speed;

                let surface_above = world.get_block((self.pos + Vec3::new(0.0, 0.6, 0.0)).floor_to_ivec());
                let floor_below = world.get_block((self.pos - Vec3::new(0.0, 0.4, 0.0)).floor_to_ivec());
                let target_vy = if floor_below.is_solid() {
                    1.2
                } else if surface_above == BlockId::Air && self.velocity.y <= 0.5 {
                    -0.8
                } else {
                    (self.id.wrapping_mul(101) % 5) as f32 * 0.3 - 0.6
                };

                self.velocity.x += (target_vx - self.velocity.x) * (6.0 * dt).min(1.0);
                self.velocity.z += (target_vz - self.velocity.z) * (6.0 * dt).min(1.0);
                if self.velocity.y < 3.0 {
                    self.velocity.y += (target_vy - self.velocity.y) * (4.0 * dt).min(1.0);
                } else {
                    self.velocity.y -= 18.0 * dt;
                }

                let current_pitch = (-self.velocity.y * 0.22).clamp(-0.55, 0.55);
                self.swim_pitch += (current_pitch - self.swim_pitch) * (6.0 * dt).min(1.0);

                let _ = self.apply_movement(dt, world);

                let h_speed = self.velocity.x.hypot(self.velocity.z);
                self.walk_phase += dt * (h_speed * 5.0 + 3.0);
                return;
            } else {
                // Out of water (flopping on land)
                self.out_of_water_timer += dt;
                self.velocity.y -= 22.0 * dt;
                self.velocity.x *= (1.0 - 5.0 * dt).max(0.0);
                self.velocity.z *= (1.0 - 5.0 * dt).max(0.0);

                self.walk_phase += dt * 16.0;
                if self.on_ground {
                    self.ai_timer -= dt;
                    if self.ai_timer <= 0.0 {
                        self.ai_timer = 0.5 + rand_f32(rng) * 0.8;
                        self.velocity.y = 3.0 + rand_f32(rng) * 1.5;
                        self.velocity.x = (rand_f32(rng) - 0.5) * 2.8;
                        self.velocity.z = (rand_f32(rng) - 0.5) * 2.8;
                        self.on_ground = false;
                        self.pending_sound = Some(crate::audio::SoundId::FishFlop);
                    }
                }

                if self.out_of_water_timer >= 12.0 && self.hurt_timer <= 0.0 {
                    self.take_damage(2.0, Vec3::new(0.0, 1.0, 0.0));
                }

                let _ = self.apply_movement(dt, world);
                return;
            }
        }

        if self.panic_timer > 0.0 {
            self.panic_timer = (self.panic_timer - dt).max(0.0);
        }

        let mut pursuing_player = false;
        if self.kind.is_hostile() {
            if let Some(target) = player_pos {
                let dx = target.x - self.pos.x;
                let dz = target.z - self.pos.z;
                let dy = (target.y - self.pos.y).abs();
                let dist_sq = dx * dx + dz * dz;

                if self.kind == MobKind::Reaper {
                    let total_dist_sq = dist_sq + (target.y - self.pos.y) * (target.y - self.pos.y);
                    if total_dist_sq < 3.2 * 3.2 {
                        if !self.is_ignited {
                            self.is_ignited = true;
                            self.pending_sound = Some(crate::audio::SoundId::ReaperHiss);
                        }
                        self.fuse += dt;
                        if self.fuse >= 1.5 {
                            self.is_dead = true;
                            self.despawn_pending = true;
                        }
                    } else if total_dist_sq > 5.5 * 5.5 {
                        self.is_ignited = false;
                        self.fuse = (self.fuse - dt * 1.5).max(0.0);
                    } else if self.is_ignited {
                        self.fuse += dt;
                        if self.fuse >= 1.5 {
                            self.is_dead = true;
                            self.despawn_pending = true;
                        }
                    }

                    if dist_sq < 24.0 * 24.0 && dy < 12.0 && !self.is_dead {
                        pursuing_player = true;
                        self.target_yaw = dx.atan2(dz);
                        self.ai_state_is_moving = self.fuse <= 0.0 || total_dist_sq > 2.8 * 2.8;
                        self.head_yaw_offset = 0.0;
                    }
                } else if self.kind == MobKind::Skeleton && dist_sq < 24.0 * 24.0 && dy < 12.0 {
                    pursuing_player = true;
                    self.target_yaw = dx.atan2(dz);
                    self.head_yaw_offset = 0.0;
                    // Skeleton keeps combat distance (moves closer if > 9m, stops to aim if <= 9m)
                    self.ai_state_is_moving = dist_sq > 9.0 * 9.0;
                } else if self.kind == MobKind::Spider && dist_sq < 24.0 * 24.0 && dy < 12.0 {
                    pursuing_player = true;
                    self.target_yaw = dx.atan2(dz);
                    self.ai_state_is_moving = true;
                    self.head_yaw_offset = 0.0;
                    if dist_sq < 6.0 * 6.0 && dist_sq > 1.8 * 1.8 && self.on_ground {
                        let to_p = Vec3::new(dx, 0.0, dz).normalized();
                        self.velocity.y = 4.8;
                        self.velocity.x = to_p.x * 5.5;
                        self.velocity.z = to_p.z * 5.5;
                        self.on_ground = false;
                    }
                } else if dist_sq < 24.0 * 24.0 && dy < 12.0 {
                    pursuing_player = true;
                    self.target_yaw = dx.atan2(dz);
                    self.ai_state_is_moving = true;
                    self.head_yaw_offset = 0.0;
                }
            }
        }

        let mut following_player = false;
        if !self.kind.is_hostile() && self.panic_timer <= 0.0 {
            if let (Some(target), Some(held)) = (player_pos, player_held) {
                if self.kind.is_tempting_item(held) {
                    let dx = target.x - self.pos.x;
                    let dz = target.z - self.pos.z;
                    let dy = (target.y - self.pos.y).abs();
                    let dist_sq = dx * dx + dz * dz;
                    if dist_sq < 10.0 * 10.0 && dy < 6.0 {
                        self.target_yaw = dx.atan2(dz);
                        self.head_yaw_offset = 0.0;
                        if dist_sq > 2.2 * 2.2 {
                            self.ai_state_is_moving = true;
                            following_player = true;
                        } else {
                            self.ai_state_is_moving = false;
                            following_player = true;
                        }
                    }
                }
            }
        }

        let mut returning_home = false;
        if self.kind == MobKind::Villager {
            if let Some(home) = self.home_pos {
                let dx = home.x - self.pos.x;
                let dz = home.z - self.pos.z;
                if dx * dx + dz * dz > 14.0 * 14.0 {
                    self.target_yaw = dx.atan2(dz);
                    self.ai_state_is_moving = true;
                    self.ai_timer = 0.5;
                    following_player = false;
                    returning_home = true;
                }
            }
        }

        // Animals get skittish and step away if player approaches closely without food
        let mut skittish_avoiding = false;
        if matches!(self.kind, MobKind::Pig | MobKind::Cow | MobKind::Chicken | MobKind::Sheep)
            && !following_player
            && self.panic_timer <= 0.0
        {
            if let Some(target) = player_pos {
                let dx = self.pos.x - target.x;
                let dz = self.pos.z - target.z;
                let dy = (target.y - self.pos.y).abs();
                let dist_sq = dx * dx + dz * dz;
                if dist_sq < 3.8 * 3.8 && dy < 2.5 {
                    self.target_yaw = dx.atan2(dz);
                    self.head_yaw_offset = 0.0;
                    self.ai_state_is_moving = true;
                    skittish_avoiding = true;
                }
            }
        }

        // Villagers turn to look at nearby players
        if self.kind == MobKind::Villager && !following_player && !returning_home && self.panic_timer <= 0.0 {
            if let Some(target) = player_pos {
                let dx = target.x - self.pos.x;
                let dz = target.z - self.pos.z;
                let dy = (target.y - self.pos.y).abs();
                if dx * dx + dz * dz < 3.5 * 3.5 && dy < 2.5 {
                    self.target_yaw = dx.atan2(dz);
                    self.head_yaw_offset = 0.0;
                    self.ai_state_is_moving = false;
                }
            }
        }

        if !pursuing_player && !following_player && !skittish_avoiding && !returning_home {
            self.ai_timer -= dt;
            if self.ai_timer <= 0.0 {
                if self.panic_timer > 0.0 {
                    self.ai_state_is_moving = true;
                    if let Some(target) = player_pos {
                        let dx = self.pos.x - target.x;
                        let dz = self.pos.z - target.z;
                        let base_yaw = dx.atan2(dz);
                        let zigzag = (rand_f32(rng) - 0.5) * 0.75;
                        self.target_yaw = base_yaw + zigzag;
                    } else {
                        self.target_yaw += (rand_f32(rng) - 0.5) * 1.6;
                    }
                    self.ai_timer = 0.3 + rand_f32(rng) * 0.35;
                } else {
                    let roll = rand_f32(rng);
                    if roll < 0.45 {
                        self.ai_state_is_moving = false;
                        self.ai_timer = 2.0 + rand_f32(rng) * 3.5;
                        self.head_yaw_offset = (rand_f32(rng) - 0.5) * 0.8;
                    } else {
                        self.ai_state_is_moving = true;
                        self.target_yaw = rand_f32(rng) * PI * 2.0;
                        self.ai_timer = 1.5 + rand_f32(rng) * 3.0;
                        self.head_yaw_offset = (rand_f32(rng) - 0.5) * 0.2;
                    }
                }
            }
        }

        let mut diff = self.target_yaw - self.yaw;
        while diff > PI { diff -= PI * 2.0; }
        while diff < -PI { diff += PI * 2.0; }
        let turn_speed = if self.panic_timer > 0.0 || pursuing_player { 10.0 } else { 4.0 };
        self.yaw += diff * (turn_speed * dt).min(1.0);

        let move_speed = if self.eat_grass_timer > 0.0 {
            0.0
        } else if self.panic_timer > 0.0 {
            5.4 // Rapid sprint away from player when panicked!
        } else if pursuing_player {
            2.6
        } else if skittish_avoiding {
            2.4 // Quick evasive step away when approached without food
        } else if self.ai_state_is_moving {
            1.4
        } else {
            0.0
        };

        if self.on_ground {
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let target_vel_x = forward.x * move_speed;
            let target_vel_z = forward.z * move_speed;

            let accel = if self.ai_state_is_moving || pursuing_player { 12.0 } else { 16.0 };
            self.velocity.x += (target_vel_x - self.velocity.x) * (accel * dt).min(1.0);
            self.velocity.z += (target_vel_z - self.velocity.z) * (accel * dt).min(1.0);
        } else {
            self.velocity.x *= (1.0 - 2.0 * dt).max(0.0);
            self.velocity.z *= (1.0 - 2.0 * dt).max(0.0);
        }

        if in_water {
            self.velocity.y = (self.velocity.y + 12.0 * dt).min(1.8);
            self.velocity.x *= (1.0 - 4.0 * dt).max(0.0);
            self.velocity.z *= (1.0 - 4.0 * dt).max(0.0);
        } else {
            self.velocity.y -= 22.0 * dt;
            let max_fall_speed = if self.kind == MobKind::Chicken { -3.2 } else { -30.0 };
            if self.velocity.y < max_fall_speed {
                self.velocity.y = max_fall_speed;
            }
        }

        if self.on_ground && (self.ai_state_is_moving || self.panic_timer > 0.0 || pursuing_player) {
            let forward = Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
            let hw = self.kind.half_width();
            let check_dist = hw + 0.35;
            let check_pos = Vec3::new(self.pos.x + forward.x * check_dist, self.pos.y + 0.4, self.pos.z + forward.z * check_dist);
            let block_ahead = world.get_block(check_pos.floor_to_ivec());
            if block_ahead.is_solid() {
                let above_pos = Vec3::new(check_pos.x, self.pos.y + self.kind.height() + 0.8, check_pos.z);
                let block_above = world.get_block(above_pos.floor_to_ivec());
                if !block_above.is_solid() {
                    self.velocity.y = 6.8;
                    self.on_ground = false;
                }
            }
        }

        let hit_wall = self.apply_movement(dt, world);
        if self.kind == MobKind::Spider && hit_wall && !self.is_dead {
            self.velocity.y = 3.8;
            self.on_ground = false;
        }

        let horizontal_speed = (self.velocity.x * self.velocity.x + self.velocity.z * self.velocity.z).sqrt();

        if self.on_ground && horizontal_speed > 0.05 {
            self.walk_phase += dt * (horizontal_speed * 4.5);
        } else if !self.on_ground {
            if self.kind == MobKind::Chicken {
                self.walk_phase += dt * 16.0; // Rapid wing flapping when in air
            } else {
                self.walk_phase += dt * 3.0;
            }
        } else {
            self.walk_phase *= (1.0 - 6.0 * dt).max(0.0);
        }
    }

    fn apply_movement(&mut self, dt: f32, world: &World) -> bool {
        let hw = self.kind.half_width();
        let h = self.kind.height();

        let aabb_at = |p: Vec3| -> (Vec3, Vec3) {
            (
                Vec3::new(p.x - hw, p.y, p.z - hw),
                Vec3::new(p.x + hw, p.y + h, p.z + hw),
            )
        };

        let mut hit_wall = false;
        let step_height = 1.05;

        // X movement
        let dx = self.velocity.x * dt;
        if dx.abs() > 1e-6 {
            let test_x = Vec3::new(self.pos.x + dx, self.pos.y, self.pos.z);
            let (min_x, max_x) = aabb_at(test_x);
            if world.box_collides(min_x, max_x) {
                let (head_min, head_max) = aabb_at(Vec3::new(self.pos.x, self.pos.y + step_height, self.pos.z));
                let (step_min, step_max) = aabb_at(Vec3::new(self.pos.x + dx, self.pos.y + step_height, self.pos.z));
                if self.on_ground && !world.box_collides(head_min, head_max) && !world.box_collides(step_min, step_max) {
                    self.pos.x += dx;
                    self.pos.y += 1.0;
                } else {
                    self.velocity.x = 0.0;
                    hit_wall = true;
                }
            } else {
                self.pos.x = test_x.x;
            }
        }

        // Z movement
        let dz = self.velocity.z * dt;
        if dz.abs() > 1e-6 {
            let test_z = Vec3::new(self.pos.x, self.pos.y, self.pos.z + dz);
            let (min_z, max_z) = aabb_at(test_z);
            if world.box_collides(min_z, max_z) {
                let (head_min, head_max) = aabb_at(Vec3::new(self.pos.x, self.pos.y + step_height, self.pos.z));
                let (step_min, step_max) = aabb_at(Vec3::new(self.pos.x, self.pos.y + step_height, self.pos.z + dz));
                if self.on_ground && !world.box_collides(head_min, head_max) && !world.box_collides(step_min, step_max) {
                    self.pos.z += dz;
                    self.pos.y += 1.0;
                } else {
                    self.velocity.z = 0.0;
                    hit_wall = true;
                }
            } else {
                self.pos.z = test_z.z;
            }
        }

        // Y movement
        self.on_ground = false;
        let dy = self.velocity.y * dt;
        let test_y = Vec3::new(self.pos.x, self.pos.y + dy, self.pos.z);
        let (min_y, max_y) = aabb_at(test_y);
        if world.box_collides(min_y, max_y) {
            if self.velocity.y <= 0.0 {
                self.on_ground = true;
            }
            self.velocity.y = 0.0;
        } else {
            self.pos.y = test_y.y;
        }

        hit_wall
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MobDrop {
    pub pos: Vec3,
    pub block: BlockId,
    pub count: u8,
}

pub struct MobManager {
    pub mobs: Vec<Mob>,
    next_mob_id: u32,
    spawn_timer: f32,
    pub rng_state: u32,
    pub dying_poofs: Vec<Vec3>,
    pub dying_drops: Vec<MobDrop>,
    pub pending_explosions: Vec<Vec3>,
    pub breeding_hearts: Vec<Vec3>,
    pub love_hearts: Vec<Vec3>,
    pub pending_block_changes: Vec<(IVec3, BlockId)>,
    pub pending_arrows: Vec<crate::gameplay::ArrowEntity>,
}

impl MobManager {
    pub fn new() -> Self {
        MobManager {
            mobs: Vec::new(),
            next_mob_id: 1,
            spawn_timer: 1.0,
            rng_state: 987654321,
            dying_poofs: Vec::new(),
            dying_drops: Vec::new(),
            pending_explosions: Vec::new(),
            breeding_hearts: Vec::new(),
            love_hearts: Vec::new(),
            pending_block_changes: Vec::new(),
            pending_arrows: Vec::new(),
        }
    }

    pub fn spawn_mob(&mut self, kind: MobKind, pos: Vec3) -> u32 {
        let id = self.next_mob_id;
        self.next_mob_id = self.next_mob_id.wrapping_add(1);
        self.mobs.push(Mob::new(id, kind, pos));
        id
    }

    pub fn update(&mut self, dt: f32, world: &World, player_pos: Vec3, player_held: Option<BlockId>, is_creative: bool) {
        let mut rng = self.rng_state;
        self.breeding_hearts.clear();
        self.love_hearts.clear();

        for mob in &mut self.mobs {
            let was_exploding = mob.kind == MobKind::Reaper && mob.fuse >= 1.5;
            let target_pos = if is_creative {
                if mob.kind.is_hostile() {
                    None
                } else if player_held.map(|h| mob.kind.is_tempting_item(h)) == Some(true) {
                    Some(player_pos)
                } else {
                    None
                }
            } else {
                Some(player_pos)
            };

            if is_creative && mob.kind == MobKind::Reaper && mob.is_ignited {
                mob.is_ignited = false;
                mob.fuse = 0.0;
            }

            // Sheep grazing / eating grass to regrow wool
            if mob.kind == MobKind::Sheep && !mob.is_dead && mob.on_ground && mob.eat_grass_timer <= 0.0 {
                let graze_roll = (rng.wrapping_mul(1664525) % 1000) as f32 / 1000.0;
                let threshold = if mob.is_sheared { 0.008 } else { 0.002 };
                if graze_roll < threshold {
                    let foot_x = mob.pos.x.floor() as i32;
                    let foot_y = (mob.pos.y - 0.2).floor() as i32;
                    let foot_z = mob.pos.z.floor() as i32;
                    let under_pos = IVec3::new(foot_x, foot_y, foot_z);
                    if world.get_block(under_pos) == BlockId::Grass {
                        mob.eat_grass_timer = 2.0;
                        mob.ai_state_is_moving = false;
                        self.pending_block_changes.push((under_pos, BlockId::Dirt));
                    }
                }
            }

            mob.update(dt, world, &mut rng, target_pos, player_held);
            if mob.kind == MobKind::Reaper && mob.fuse >= 1.5 && !was_exploding {
                self.pending_explosions.push(mob.center_pos());
            }

            if mob.in_love > 0.0 && !mob.is_dead {
                mob.love_particle_timer -= dt;
                if mob.love_particle_timer <= 0.0 {
                    mob.love_particle_timer = 0.45;
                    self.love_hearts.push(mob.center_pos() + Vec3::new(0.0, mob.kind.height() * 0.45, 0.0));
                }
            }

            // Skeleton ranged archery attack against player
            if !is_creative && mob.kind == MobKind::Skeleton && !mob.is_dead {
                let dx = player_pos.x - mob.pos.x;
                let dz = player_pos.z - mob.pos.z;
                let dy = (player_pos.y - mob.pos.y).abs();
                let dist_sq = dx * dx + dz * dz;
                if dist_sq < 18.0 * 18.0 && dy < 10.0 {
                    let start_pos = mob.pos + Vec3::new(0.0, 1.4, 0.0);
                    let aim_pos = player_pos + Vec3::new(0.0, 0.8, 0.0);
                    let shot_delta = aim_pos - start_pos;
                    let line_of_sight = world.raycast_camera_collision(
                        start_pos,
                        shot_delta,
                        shot_delta.length(),
                    ) >= shot_delta.length();
                    if line_of_sight {
                        mob.attack_cooldown += dt;
                    }
                    if line_of_sight && mob.attack_cooldown >= 2.2 {
                        mob.attack_cooldown = 0.0;
                        let dir = (aim_pos - start_pos).normalized();
                        let arrow = crate::gameplay::ArrowEntity::new(
                            start_pos,
                            (dir + Vec3::new(0.0, 0.08, 0.0)) * 18.0,
                            false,
                            3.0,
                        );
                        self.pending_arrows.push(arrow);
                        mob.pending_sound = Some(crate::audio::SoundId::BowShoot);
                    }
                }
            }

            let dist_sq = (mob.pos.x - player_pos.x) * (mob.pos.x - player_pos.x)
                + (mob.pos.z - player_pos.z) * (mob.pos.z - player_pos.z);
            if dist_sq > 75.0 * 75.0 && !mob.is_dead {
                mob.despawn_pending = true;
            }
        }

        // Iron Golem AI: defend villagers and player by attacking nearby hostile mobs
        let mut golem_attacks: Vec<(usize, usize, f32, f32)> = Vec::new();
        let mob_count = self.mobs.len();
        for i in 0..mob_count {
            if self.mobs[i].kind != MobKind::IronGolem || self.mobs[i].is_dead {
                continue;
            }
            let golem_pos = self.mobs[i].pos;
            let mut closest_enemy: Option<(usize, f32, f32, f32)> = None;
            for j in 0..mob_count {
                if !self.mobs[j].kind.is_hostile() || self.mobs[j].is_dead {
                    continue;
                }
                let dx = self.mobs[j].pos.x - golem_pos.x;
                let dz = self.mobs[j].pos.z - golem_pos.z;
                let dy = (self.mobs[j].pos.y - golem_pos.y).abs();
                let dist_sq = dx * dx + dz * dz;
                if dist_sq < 16.0 * 16.0 && dy < 6.0 {
                    if closest_enemy.as_ref().map_or(true, |&(_, min_d, _, _)| dist_sq < min_d) {
                        closest_enemy = Some((j, dist_sq, dx, dz));
                    }
                }
            }

            if let Some((target_idx, dist_sq, dx, dz)) = closest_enemy {
                self.mobs[i].target_yaw = dx.atan2(dz);
                self.mobs[i].ai_state_is_moving = true;
                if dist_sq <= 2.2 * 2.2 {
                    golem_attacks.push((i, target_idx, dx, dz));
                }
            }
        }

        for (golem_idx, target_idx, dx, dz) in golem_attacks {
            let kb_dir = Vec3::new(dx.signum() * 1.5, 4.2, dz.signum() * 1.5);
            self.mobs[target_idx].take_damage(14.0, kb_dir);
            self.mobs[golem_idx].pending_sound = Some(crate::audio::SoundId::IronGolemAttack);
            self.mobs[golem_idx].walk_phase += 2.0;
        }

        // Animal breeding pairs check
        let mut babies = Vec::new();
        let len = self.mobs.len();
        for i in 0..len {
            if self.mobs[i].in_love <= 0.0 || self.mobs[i].is_baby || self.mobs[i].is_dead {
                continue;
            }
            for j in (i + 1)..len {
                if self.mobs[j].in_love <= 0.0 || self.mobs[j].is_baby || self.mobs[j].is_dead {
                    continue;
                }
                if self.mobs[i].kind == self.mobs[j].kind {
                    let dx = self.mobs[i].pos.x - self.mobs[j].pos.x;
                    let dy = self.mobs[i].pos.y - self.mobs[j].pos.y;
                    let dz = self.mobs[i].pos.z - self.mobs[j].pos.z;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq < 2.25 {
                        self.mobs[i].in_love = 0.0;
                        self.mobs[j].in_love = 0.0;
                        self.mobs[i].breed_cooldown = 300.0;
                        self.mobs[j].breed_cooldown = 300.0;
                        let mid = (self.mobs[i].pos + self.mobs[j].pos) * 0.5;
                        self.breeding_hearts.push(mid + Vec3::new(0.0, 0.8, 0.0));
                        let mut baby = Mob::new(self.next_mob_id, self.mobs[i].kind, mid);
                        self.next_mob_id = self.next_mob_id.wrapping_add(1);
                        baby.is_baby = true;
                        baby.age = -120.0;
                        baby.health = self.mobs[i].kind.max_health() * 0.5;
                        if self.mobs[i].kind == MobKind::Sheep {
                            let pick_parent = (rng.wrapping_mul(22695477) % 2) == 0;
                            baby.wool_color = if pick_parent {
                                self.mobs[i].wool_color
                            } else {
                                self.mobs[j].wool_color
                            };
                        }
                        babies.push(baby);
                    } else if dist_sq < 64.0 {
                        self.mobs[i].target_yaw = (-dx).atan2(-dz);
                        self.mobs[i].ai_state_is_moving = true;
                        self.mobs[j].target_yaw = dx.atan2(dz);
                        self.mobs[j].ai_state_is_moving = true;
                    }
                }
            }
        }
        for b in babies {
            self.mobs.push(b);
        }

        self.dying_poofs.clear();
        self.dying_drops.clear();
        self.mobs.retain(|m| {
            if m.despawn_pending {
                if m.is_dead {
                    self.dying_poofs.push(m.center_pos());
                    if !m.is_baby {
                        match m.kind {
                        MobKind::Pig => {
                            let count = 1 + (m.id % 3) as u8; // 1-3 meat drops
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawPorkchop,
                                count,
                            });
                        }
                        MobKind::Cow => {
                            let count = 1 + (m.id % 3) as u8; // 1-3 meat drops
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawBeef,
                                count,
                            });
                            let leather_count = (m.id % 3) as u8; // 0-2 leather drops
                            if leather_count > 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.1, 0.05),
                                    block: BlockId::Leather,
                                    count: leather_count,
                                });
                            }
                        }
                        MobKind::Sheep => {
                            let count = 1 + (m.id % 2) as u8; // 1-2 meat drops
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawBeef,
                                count,
                            });
                            if !m.is_sheared {
                                let wool_block = BlockId::wool_block_from_color(m.wool_color);
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.1, 0.05),
                                    block: wool_block,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::Chicken => {
                            let meat_count = 1 + (m.id % 2) as u8; // 1-2 raw chicken
                            let feather_count = 1 + ((m.id >> 2) % 3) as u8; // 1-3 feathers
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawChicken,
                                count: meat_count,
                            });
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos() + Vec3::new(0.1, 0.1, 0.1),
                                block: BlockId::Feather,
                                count: feather_count,
                            });
                        }
                        MobKind::Zombie => {
                            let meat_count = 1 + (m.id % 2) as u8; // 1-2 raw beef
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawBeef,
                                count: meat_count,
                            });
                            if m.id % 6 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.1, 0.05),
                                    block: BlockId::IronIngot,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::Skeleton => {
                            let bone_count = 1 + (m.id % 3) as u8; // 1-3 bones
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::Bone,
                                count: bone_count,
                            });
                            let arrow_count = ((m.id >> 1) % 3) as u8; // 0-2 arrows
                            if arrow_count > 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.1, 0.05),
                                    block: BlockId::Arrow,
                                    count: arrow_count,
                                });
                            }
                        }
                        MobKind::Reaper => {
                            if m.fuse < 1.45 {
                                let gunpowder_count = 1 + (m.id % 2) as u8; // 1-2 gunpowder
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos(),
                                    block: BlockId::Gunpowder,
                                    count: gunpowder_count,
                                });
                            }
                        }
                        MobKind::Spider => {
                            let count = 1 + (m.id % 2) as u8; // 1-2 string
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::StringItem,
                                count,
                            });
                            if m.id % 3 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.1, 0.05),
                                    block: BlockId::SpiderEye,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::Dolphin => {
                            if m.id % 2 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos(),
                                    block: BlockId::RawCod,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::Cod => {
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawCod,
                                count: 1,
                            });
                            if m.id % 5 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.05, 0.05),
                                    block: BlockId::Bone,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::Salmon => {
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::RawSalmon,
                                count: 1,
                            });
                            if m.id % 5 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.05, 0.05),
                                    block: BlockId::Bone,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::Pufferfish => {
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::Pufferfish,
                                count: 1,
                            });
                        }
                        MobKind::TropicalFish => {
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::TropicalFish,
                                count: 1,
                            });
                        }
                        MobKind::Villager => {
                            if m.id % 3 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos(),
                                    block: BlockId::Emerald,
                                    count: 1,
                                });
                            }
                        }
                        MobKind::IronGolem => {
                            let ingots = 3 + (m.id % 3) as u8; // 3-5 iron ingots
                            self.dying_drops.push(MobDrop {
                                pos: m.center_pos(),
                                block: BlockId::IronIngot,
                                count: ingots,
                            });
                            if m.id % 2 == 0 {
                                self.dying_drops.push(MobDrop {
                                    pos: m.center_pos() + Vec3::new(0.05, 0.1, 0.05),
                                    block: BlockId::RedWool, // Poppy
                                    count: 1,
                                });
                            }
                        }
                    }
                    }

                    // Mod-defined mob death drops
                    for (drop_block, drop_count) in crate::engine::modding::get_custom_mob_drops(m.kind.name()) {
                        self.dying_drops.push(MobDrop {
                            pos: m.center_pos() + crate::core::math::Vec3::new(0.0, 0.2, 0.0),
                            block: drop_block,
                            count: drop_count,
                        });
                    }
                }
                false
            } else {
                true
            }
        });

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            let is_night = world.sunlight_factor() < 0.50;
            self.spawn_timer = if is_night { 2.0 } else { 4.0 };
            self.try_natural_spawn(world, player_pos, &mut rng);
        }

        self.rng_state = rng;
    }

    pub fn apply_explosion(&mut self, explosion_pos: Vec3, radius: f32, max_damage: f32) {
        for mob in &mut self.mobs {
            if mob.is_dead {
                continue;
            }
            let center = mob.center_pos();
            let to_mob = center - explosion_pos;
            let dist = to_mob.length();
            if dist <= radius {
                let factor = (1.0 - dist / radius).clamp(0.0, 1.0);
                let damage = (factor * max_damage).max(4.0);
                let kb_dir = if dist > 0.01 {
                    to_mob.normalized()
                } else {
                    Vec3::new(0.0, 1.0, 0.0)
                };
                mob.take_damage(damage, kb_dir);
                let kb_strength = factor * 12.0;
                mob.velocity.x += kb_dir.x * kb_strength;
                mob.velocity.y = (mob.velocity.y + kb_dir.y * kb_strength + 4.0).min(14.0);
                mob.velocity.z += kb_dir.z * kb_strength;
                mob.on_ground = false;
            }
        }
    }

    fn try_natural_spawn(&mut self, world: &World, player_pos: Vec3, rng: &mut u32) {
        let rand_f32 = |state: &mut u32| -> f32 {
            *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            (*state as f32) / (u32::MAX as f32)
        };

        let is_night = world.sunlight_factor() < 0.50;
        let hostile_count = self
            .mobs
            .iter()
            .filter(|m| m.kind.is_hostile() && !m.is_dead)
            .count();

        if hostile_count >= 14 {
            return;
        }
        if !is_night && rand_f32(rng) > 0.40 {
            return;
        }

        // Try several attempts to find a valid location near the player
        let attempts = if is_night { 5 } else { 2 };
        for _ in 0..attempts {
            let angle = rand_f32(rng) * PI * 2.0;
            let dist = 14.0 + rand_f32(rng) * 24.0;
            let target_x = (player_pos.x + angle.cos() * dist).floor() as i32;
            let target_z = (player_pos.z + angle.sin() * dist).floor() as i32;

            if is_night {
                // In night mode, prioritize surface spawns first so players actually see hostile mobs!
                let mut surface_y = None;
                for y in (1..63).rev() {
                    let block = world.get_block(IVec3::new(target_x, y, target_z));
                    if block.is_solid() && block != BlockId::Leaves && block != BlockId::SnowLeaves && block != BlockId::Lava && block != BlockId::Water {
                        let above1 = world.get_block(IVec3::new(target_x, y + 1, target_z));
                        let above2 = world.get_block(IVec3::new(target_x, y + 2, target_z));
                        if above1 == BlockId::Air && above2 == BlockId::Air {
                            surface_y = Some(y + 1);
                            break;
                        }
                    }
                }

                if let Some(sy) = surface_y {
                    let roll = rand_f32(rng);
                    let kind = if roll < 0.35 {
                        MobKind::Zombie
                    } else if roll < 0.60 {
                        MobKind::Skeleton
                    } else if roll < 0.80 {
                        MobKind::Reaper
                    } else {
                        MobKind::Spider
                    };
                    let spawn_pos = Vec3::new(target_x as f32 + 0.5, sy as f32, target_z as f32 + 0.5);
                    self.spawn_mob(kind, spawn_pos);
                    crate::log_info!("MOB", "Night surface spawn: {:?} at ({:.1}, {:.1}, {:.1})", kind, spawn_pos.x, spawn_pos.y, spawn_pos.z);
                    return;
                }
            }

            // Cave spawn (underground): Hostile mobs
            let mut cave_spawn = None;
            let py = player_pos.y.floor() as i32;
            let y_min = (py - 12).clamp(2, 58);
            let y_max = (py + 8).clamp(3, 60);

            for y in y_min..=y_max {
                let floor_block = world.get_block(IVec3::new(target_x, y, target_z));
                if floor_block.is_solid() && floor_block != BlockId::Lava && floor_block != BlockId::Water {
                    let above1 = world.get_block(IVec3::new(target_x, y + 1, target_z));
                    let above2 = world.get_block(IVec3::new(target_x, y + 2, target_z));
                    if above1 == BlockId::Air && above2 == BlockId::Air {
                        let mut has_roof = false;
                        for cy in (y + 3)..=63 {
                            if world.get_block(IVec3::new(target_x, cy, target_z)).is_solid() {
                                has_roof = true;
                                break;
                            }
                        }
                        if has_roof {
                            cave_spawn = Some(y + 1);
                            break;
                        }
                    }
                }
            }

            if let Some(cy) = cave_spawn {
                let roll = rand_f32(rng);
                let kind = if roll < 0.35 {
                    MobKind::Zombie
                } else if roll < 0.60 {
                    MobKind::Skeleton
                } else if roll < 0.80 {
                    MobKind::Reaper
                } else {
                    MobKind::Spider
                };
                let spawn_pos = Vec3::new(target_x as f32 + 0.5, cy as f32, target_z as f32 + 0.5);
                self.spawn_mob(kind, spawn_pos);
                return;
            }

            // Aquatic water spawn check
            let mut water_column = None;
            for y in (2..=21).rev() {
                if world.get_block(IVec3::new(target_x, y, target_z)) == BlockId::Water {
                    let mut bot_y = y;
                    for by in (1..y).rev() {
                        let b = world.get_block(IVec3::new(target_x, by, target_z));
                        if b != BlockId::Water && b != BlockId::Kelp && b != BlockId::Seagrass {
                            bot_y = by + 1;
                            break;
                        }
                    }
                    if y - bot_y >= 2 {
                        water_column = Some((bot_y, y));
                    }
                    break;
                }
            }

            if let Some((bot, top)) = water_column {
                let aquatic_count = self.mobs.iter().filter(|m| m.kind.is_aquatic() && !m.is_dead).count();
                if aquatic_count < 14 {
                    let spawn_y = bot as f32 + 0.5 + rand_f32(rng) * ((top - bot) as f32 - 0.5);
                    let roll = rand_f32(rng);
                    let kind = if roll < 0.35 {
                        MobKind::Cod
                    } else if roll < 0.60 {
                        MobKind::Salmon
                    } else if roll < 0.75 {
                        MobKind::TropicalFish
                    } else if roll < 0.88 {
                        MobKind::Pufferfish
                    } else {
                        MobKind::Dolphin
                    };
                    let spawn_pos = Vec3::new(target_x as f32 + 0.5, spawn_y, target_z as f32 + 0.5);
                    self.spawn_mob(kind, spawn_pos);
                    crate::log_info!("MOB", "Aquatic spawn: {:?} at ({:.1}, {:.1}, {:.1})", kind, spawn_pos.x, spawn_pos.y, spawn_pos.z);
                    return;
                }
            }
        }
    }

    pub fn raycast_mob(&self, origin: Vec3, dir: Vec3, max_reach: f32) -> Option<(usize, f32)> {
        let dir = dir.normalized();
        let mut closest_hit: Option<(usize, f32)> = None;

        for (idx, mob) in self.mobs.iter().enumerate() {
            if mob.is_dead {
                continue;
            }

            let (min, max) = mob.aabb();
            let padding = 0.15;
            let min = Vec3::new(min.x - padding, min.y, min.z - padding);
            let max = Vec3::new(max.x + padding, max.y + padding, max.z + padding);

            if let Some(t) = ray_aabb_intersect(origin, dir, min, max) {
                if t >= 0.0 && t <= max_reach {
                    match closest_hit {
                        Some((_, closest_t)) => {
                            if t < closest_t {
                                closest_hit = Some((idx, t));
                            }
                        }
                        None => {
                            closest_hit = Some((idx, t));
                        }
                    }
                }
            }
        }

        closest_hit
    }
}

fn ray_aabb_intersect(origin: Vec3, dir: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let mut t_min = (min.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    let mut t_max = (max.x - origin.x) / if dir.x != 0.0 { dir.x } else { 1e-8 };
    if t_min > t_max { std::mem::swap(&mut t_min, &mut t_max); }

    let mut ty_min = (min.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    let mut ty_max = (max.y - origin.y) / if dir.y != 0.0 { dir.y } else { 1e-8 };
    if ty_min > ty_max { std::mem::swap(&mut ty_min, &mut ty_max); }

    if (t_min > ty_max) || (ty_min > t_max) {
        return None;
    }
    if ty_min > t_min { t_min = ty_min; }
    if ty_max < t_max { t_max = ty_max; }

    let mut tz_min = (min.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    let mut tz_max = (max.z - origin.z) / if dir.z != 0.0 { dir.z } else { 1e-8 };
    if tz_min > tz_max { std::mem::swap(&mut tz_min, &mut tz_max); }

    if (t_min > tz_max) || (tz_min > t_max) {
        return None;
    }
    if tz_min > t_min { t_min = tz_min; }
    if tz_max < t_max { t_max = tz_max; }

    if t_max < 0.0 {
        return None;
    }

    Some(if t_min >= 0.0 { t_min } else { t_max })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SoundId;

    #[test]
    fn test_mob_sounds_damage_and_death() {
        let mut pig = Mob::new(1, MobKind::Pig, Vec3::new(0.0, 10.0, 0.0));
        assert_eq!(pig.pending_sound, None);

        // Non-fatal hit
        pig.take_damage(3.0, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(pig.pending_sound, Some(SoundId::PigHurt));
        assert!(!pig.is_dead);
        let _ = pig.pending_sound.take();

        // Fatal hit
        pig.take_damage(100.0, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(pig.pending_sound, Some(SoundId::PigDeath));
        assert!(pig.is_dead);
    }

    #[test]
    fn test_all_mob_kinds_sound_mapping() {
        let kinds = [
            MobKind::Pig,
            MobKind::Cow,
            MobKind::Chicken,
            MobKind::Sheep,
            MobKind::Zombie,
            MobKind::Skeleton,
            MobKind::Reaper,
            MobKind::Spider,
        ];
        for kind in kinds {
            let mut mob = Mob::new(42, kind, Vec3::zero());
            if let Some(snd) = kind.idle_sound() {
                assert_eq!(snd.category(), crate::audio::SoundCategory::Sfx);
            }
            assert_eq!(kind.hurt_sound().category(), crate::audio::SoundCategory::Sfx);
            assert_eq!(kind.death_sound().category(), crate::audio::SoundCategory::Sfx);

            mob.take_damage(1.0, Vec3::zero());
            assert_eq!(mob.pending_sound, Some(kind.hurt_sound()));

            mob.take_damage(999.0, Vec3::zero());
            assert_eq!(mob.pending_sound, Some(kind.death_sound()));
        }
    }

    #[test]
    fn test_husbandry_tempting_items_and_baby_scaling() {
        use crate::world::block::BlockId;

        assert!(MobKind::Cow.is_tempting_item(BlockId::Wheat));
        assert!(MobKind::Sheep.is_tempting_item(BlockId::Wheat));
        assert!(!MobKind::Cow.is_tempting_item(BlockId::Carrot));

        assert!(MobKind::Pig.is_tempting_item(BlockId::Carrot));
        assert!(MobKind::Pig.is_tempting_item(BlockId::Potato));
        assert!(MobKind::Pig.is_tempting_item(BlockId::BakedPotato));
        assert!(!MobKind::Pig.is_tempting_item(BlockId::Wheat));

        assert!(MobKind::Chicken.is_tempting_item(BlockId::WheatSeeds));
        assert!(!MobKind::Chicken.is_tempting_item(BlockId::Bread));

        let adult = Mob::new(1, MobKind::Cow, Vec3::new(0.0, 0.0, 0.0));
        let mut baby = Mob::new(2, MobKind::Cow, Vec3::new(0.0, 0.0, 0.0));
        baby.is_baby = true;
        let adult_aabb = adult.aabb();
        let baby_aabb = baby.aabb();
        assert!(baby_aabb.1.x - baby_aabb.0.x < adult_aabb.1.x - adult_aabb.0.x);
    }

    #[test]
    fn test_sheep_shearing_and_grazing_regrowth() {
        let mut sheep = Mob::new(100, MobKind::Sheep, Vec3::zero());
        assert!(!sheep.is_sheared);
        assert!(sheep.wool_color <= 15);

        // Shearing
        sheep.is_sheared = true;
        assert!(sheep.is_sheared);

        // Grazing
        sheep.eat_grass_timer = 1.0;
        let world = crate::world::World::new_with_seed(42);
        let mut rng = 12345;
        sheep.update(0.5, &world, &mut rng, None, None);
        assert!(sheep.is_sheared);
        assert!(sheep.eat_grass_timer > 0.0);

        // Regrow wool
        sheep.update(0.6, &world, &mut rng, None, None);
        assert_eq!(sheep.eat_grass_timer, 0.0);
        assert!(!sheep.is_sheared, "Wool should regrow after grazing");
    }

    #[test]
    fn test_mobs_do_not_see_player_in_creative() {
        let mut mob_manager = MobManager::new();
        let world = crate::world::World::new_with_seed(12345);
        let player_pos = Vec3::new(5.0, 60.0, 5.0);

        // Spawn a Reaper and a Zombie within targeting distance (2 blocks away)
        let reaper_id = mob_manager.spawn_mob(MobKind::Reaper, Vec3::new(5.0, 60.0, 7.0));
        let zombie_id = mob_manager.spawn_mob(MobKind::Zombie, Vec3::new(5.0, 60.0, 8.0));

        // 1. Creative mode: update mobs
        mob_manager.update(0.05, &world, player_pos, None, true);

        let reaper = mob_manager.mobs.iter().find(|m| m.id == reaper_id).unwrap();
        assert!(!reaper.is_ignited, "Reaper must not ignite or see player in Creative mode");
        assert_eq!(reaper.fuse, 0.0, "Reaper fuse must stay 0 in Creative mode");

        let zombie = mob_manager.mobs.iter().find(|m| m.id == zombie_id).unwrap();
        let angle_to_player = (player_pos.x - zombie.pos.x).atan2(player_pos.z - zombie.pos.z);
        assert_ne!(zombie.target_yaw, angle_to_player, "Zombie must not target player in Creative mode");

        // 2. Survival mode: update mobs
        mob_manager.update(0.05, &world, player_pos, None, false);

        let reaper = mob_manager.mobs.iter().find(|m| m.id == reaper_id).unwrap();
        assert!(reaper.is_ignited, "Reaper should ignite when seeing player in Survival mode");

        let zombie = mob_manager.mobs.iter().find(|m| m.id == zombie_id).unwrap();
        let angle_to_player = (player_pos.x - zombie.pos.x).atan2(player_pos.z - zombie.pos.z);
        assert!((zombie.target_yaw - angle_to_player).abs() < 1e-4, "Zombie should target player in Survival mode");
    }

    #[test]
    fn test_skeleton_does_not_shoot_through_walls() {
        use crate::world::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

        let mut world = World::new_with_seed(1);
        world.chunks.clear();
        world.chunks.insert(
            (0, 0),
            Chunk {
                cx: 0,
                cz: 0,
                blocks: vec![
                    BlockId::Air.to_u8();
                    (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize
                ],
                dirty: false,
            },
        );
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                world.set_block(IVec3::new(x, 0, z), BlockId::Stone);
            }
        }
        for y in 1..=3 {
            world.set_block(IVec3::new(4, y, 2), BlockId::Stone);
        }

        let mut manager = MobManager::new();
        manager.spawn_mob(MobKind::Skeleton, Vec3::new(2.5, 1.0, 2.5));
        manager.update(0.05, &world, Vec3::new(8.5, 1.0, 2.5), None, false);

        assert!(manager.pending_arrows.is_empty());
        assert_eq!(manager.mobs[0].attack_cooldown, 0.0);
    }

    #[test]
    fn test_villagers_return_to_their_home_after_wandering() {
        use crate::world::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

        let mut world = World::new_with_seed(1);
        world.chunks.clear();
        world.chunks.insert(
            (0, 0),
            Chunk {
                cx: 0,
                cz: 0,
                blocks: vec![
                    BlockId::Air.to_u8();
                    (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize
                ],
                dirty: false,
            },
        );
        for x in 0..CHUNK_SIZE_X {
            for z in 0..CHUNK_SIZE_Z {
                world.set_block(IVec3::new(x, 10, z), BlockId::Grass);
            }
        }

        let home = Vec3::new(8.5, 11.0, 8.5);
        let mut villager = Mob::new(1, MobKind::Villager, home);
        let mut rng = 12345;
        let mut max_distance_sq: f32 = 0.0;
        for _ in 0..600 {
            villager.update(0.05, &world, &mut rng, None, None);
            let dx = villager.pos.x - home.x;
            let dz = villager.pos.z - home.z;
            max_distance_sq = max_distance_sq.max(dx * dx + dz * dz);
        }

        assert!(max_distance_sq < 17.0 * 17.0);
    }

    #[test]
    fn test_daytime_natural_spawns_stay_underground() {
        use crate::world::{Chunk, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z};

        let mut world = World::new_with_seed(1);
        world.chunks.clear();
        world.time_of_day = 120.0;
        for cx in -3..=2 {
            for cz in -3..=2 {
                let mut chunk = Chunk {
                    cx,
                    cz,
                    blocks: vec![0; (CHUNK_SIZE_X * CHUNK_SIZE_Y * CHUNK_SIZE_Z) as usize],
                    dirty: false,
                };
                for x in 0..CHUNK_SIZE_X {
                    for z in 0..CHUNK_SIZE_Z {
                        chunk.set_local(x, 20, z, BlockId::Grass);
                    }
                }
                world.chunks.insert((cx, cz), chunk);
            }
        }

        let mut manager = MobManager::new();
        let mut rng = 12345;
        for _ in 0..100 {
            manager.try_natural_spawn(&world, Vec3::new(0.0, 30.0, 0.0), &mut rng);
        }

        assert!(
            manager.mobs.is_empty(),
            "Natural daytime spawns must not appear on the surface"
        );
    }
}
