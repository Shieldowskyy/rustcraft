//! Komendy świata i istot (`/time`, `/weather`, `/spawn`, `/mobs`, `/debug`, `/seed`, `/reloadchunks`, `/tps`).

use std::str::SplitWhitespace;

use crate::features::entities::MobKind;
use crate::features::ui::draw::pack_color;
use crate::features::world::{WeatherType, World};

use super::state::{ChatState, CommandContext};

impl ChatState {
    pub(super) fn execute_world_or_mob_command(
        &mut self,
        cmd: &str,
        parts: &mut SplitWhitespace<'_>,
        ctx: &mut CommandContext,
    ) -> bool {
        match cmd {
            "time" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "set" => {
                        let val = parts.next().unwrap_or("").to_ascii_lowercase();
                        match val.as_str() {
                            "day" | "1000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.25;
                                self.add_success("Set the time to 1000 (Day)");
                            }
                            "noon" | "6000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.50;
                                self.add_success("Set the time to 6000 (Noon)");
                            }
                            "night" | "13000" => {
                                ctx.world.time_of_day = World::DAY_DURATION * 0.75;
                                self.add_success("Set the time to 13000 (Night)");
                            }
                            "midnight" | "18000" => {
                                ctx.world.time_of_day = 0.0;
                                self.add_success("Set the time to 18000 (Midnight)");
                            }
                            other => {
                                if let Ok(ticks) = other.parse::<f32>() {
                                    let frac = (ticks % 24000.0) / 24000.0;
                                    ctx.world.time_of_day = World::DAY_DURATION * frac;
                                    self.add_success(format!("Set the time to {:.0}", ticks));
                                } else {
                                    self.add_error("Usage: /time set <day|noon|night|midnight|number>");
                                }
                            }
                        }
                    }
                    "query" => {
                        let ticks = (ctx.world.day_fraction() * 24000.0) as u32;
                        self.add_info(format!("The time is {} ticks", ticks));
                    }
                    _ => self.add_error("Usage: /time <set|query> <value>"),
                }
                true
            }
            "tps" => {
                self.add_info("TPS is available when connected to a multiplayer server or host.");
                true
            }
            "weather" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                let duration_arg = parts.next().and_then(|s| s.parse::<f32>().ok());
                match sub.as_str() {
                    "clear" => {
                        let duration = duration_arg.unwrap_or(600.0);
                        ctx.world.weather.set_weather(WeatherType::Clear, Some(duration));
                        self.add_success(format!("Set the weather to clear (duration: {:.0}s)", duration));
                    }
                    "rain" => {
                        let duration = duration_arg.unwrap_or(360.0);
                        ctx.world.weather.set_weather(WeatherType::Rain, Some(duration));
                        self.add_success(format!("Set the weather to rain (duration: {:.0}s)", duration));
                    }
                    "thunder" | "thunderstorm" => {
                        let duration = duration_arg.unwrap_or(240.0);
                        ctx.world.weather.set_weather(WeatherType::Thunder, Some(duration));
                        self.add_success(format!(
                            "Set the weather to thunder and rain (duration: {:.0}s)",
                            duration
                        ));
                    }
                    "query" => {
                        let state_str = match ctx.world.weather.current_type {
                            WeatherType::Clear => "clear",
                            WeatherType::Rain => "rain",
                            WeatherType::Thunder => "thunder",
                        };
                        self.add_info(format!(
                            "The weather is currently {} (remaining: {:.0}s, rain intensity: {:.0}%)",
                            state_str,
                            ctx.world.weather.timer,
                            ctx.world.weather.rain_strength * 100.0
                        ));
                    }
                    _ => self.add_error("Usage: /weather <clear|rain|thunder|query> [duration]"),
                }
                true
            }
            "spawn" => {
                let mut kind_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                if kind_arg == "mob" {
                    kind_arg = parts.next().unwrap_or("").to_ascii_lowercase();
                }
                let count: usize = parts
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(1)
                    .clamp(1, 20);

                let mob_kind = match kind_arg.as_str() {
                    "pig" | "swinia" => Some(MobKind::Pig),
                    "cow" | "krowa" => Some(MobKind::Cow),
                    "chicken" | "kurczak" => Some(MobKind::Chicken),
                    "sheep" | "owca" => Some(MobKind::Sheep),
                    "zombie" | "zombi" => Some(MobKind::Zombie),
                    "skeleton" | "szkielet" => Some(MobKind::Skeleton),
                    "reaper" => Some(MobKind::Reaper),
                    "dolphin" | "delfin" => Some(MobKind::Dolphin),
                    "cod" | "dorsz" => Some(MobKind::Cod),
                    "salmon" | "losos" => Some(MobKind::Salmon),
                    "pufferfish" | "rozdymka" => Some(MobKind::Pufferfish),
                    "tropical_fish" | "tropikalna_ryba" => Some(MobKind::TropicalFish),
                    "villager" | "osadnik" | "wiesniak" => Some(MobKind::Villager),
                    "spider" | "pajak" => Some(MobKind::Spider),
                    _ => None,
                };

                if let Some(kind) = mob_kind {
                    let forward = ctx.camera.forward_flat();
                    for i in 0..count {
                        let offset = forward * (2.0 + (i as f32 * 0.8));
                        let spawn_pos = ctx.camera.position + offset;
                        ctx.mob_manager.spawn_mob(kind, spawn_pos);
                        ctx.particles.spawn_mob_hit(spawn_pos);
                    }
                    self.add_success(format!("Spawned {} {:?}(s)", count, kind));
                    crate::log_debug!("Command", "Spawned {} {:?} mobs", count, kind);
                } else {
                    self.add_error("Usage: /spawn <pig|cow|chicken|sheep|zombie|skeleton|reaper|dolphin|cod|salmon|pufferfish|tropical_fish|villager> [count]");
                }
                true
            }
            "mobs" | "moblist" | "listmobs" => {
                self.list_mobs_debug(ctx);
                true
            }
            "debug" => {
                let sub = parts.next().unwrap_or("").to_ascii_lowercase();
                match sub.as_str() {
                    "mobs" | "moblist" => self.list_mobs_debug(ctx),
                    _ => self.add_info("Debug commands: /mobs, /debug mobs"),
                }
                true
            }
            "seed" => {
                self.add_info(format!("Seed: [{}]", ctx.world.seed));
                true
            }
            "reload" | "reloadchunks" => {
                ctx.world.mark_all_chunks_dirty();
                *ctx.reload_chunks_requested = true;
                self.add_success("Reloading all chunks and rebuilding mesh cache...");
                crate::log_info!("WORLD", "Chunk reload triggered via chat command");
                true
            }
            _ => false,
        }
    }

    pub(super) fn list_mobs_debug(&mut self, ctx: &CommandContext) {
        let total = ctx.mob_manager.mobs.len();
        if total == 0 {
            self.add_info("No mobs currently spawned in the world.");
            return;
        }

        let player_pos = ctx.camera.position;
        self.add_info(format!("=== Spawned Mobs (Total: {}) ===", total));

        let mut pigs = 0;
        let mut cows = 0;
        let mut chickens = 0;
        let mut sheep = 0;
        let mut zombies = 0;
        let mut skeletons = 0;
        let mut reapers = 0;
        let mut dolphins = 0;
        let mut cod = 0;
        let mut salmon = 0;
        let mut pufferfish = 0;
        let mut tropical_fish = 0;
        let mut villagers = 0;
        let mut iron_golems = 0;
        let mut spiders = 0;

        for mob in &ctx.mob_manager.mobs {
            match mob.kind {
                MobKind::Pig => pigs += 1,
                MobKind::Cow => cows += 1,
                MobKind::Chicken => chickens += 1,
                MobKind::Sheep => sheep += 1,
                MobKind::Zombie => zombies += 1,
                MobKind::Skeleton => skeletons += 1,
                MobKind::Reaper => reapers += 1,
                MobKind::Dolphin => dolphins += 1,
                MobKind::Cod => cod += 1,
                MobKind::Salmon => salmon += 1,
                MobKind::Pufferfish => pufferfish += 1,
                MobKind::TropicalFish => tropical_fish += 1,
                MobKind::Villager => villagers += 1,
                MobKind::IronGolem => iron_golems += 1,
                MobKind::Spider => spiders += 1,
            }
            let dist = (mob.pos - player_pos).length();
            let hostile_tag = if mob.kind.is_hostile() { " [HOSTILE]" } else { "" };
            self.add_message(
                format!(
                    "#{:02} {} at ({:.1}, {:.1}, {:.1}) | Dist: {:.1}m | HP: {:.0}/{:.0}{}",
                    mob.id,
                    mob.kind.name(),
                    mob.pos.x,
                    mob.pos.y,
                    mob.pos.z,
                    dist,
                    mob.health.max(0.0),
                    mob.kind.max_health(),
                    hostile_tag,
                ),
                if mob.kind.is_hostile() {
                    pack_color(1.0, 0.45, 0.45)
                } else {
                    pack_color(0.55, 1.0, 0.55)
                },
            );
        }

        self.add_info(format!(
            "Summary: Pigs: {}, Cows: {}, Chickens: {}, Sheep: {}, Zombies: {}, Skeletons: {}, Reapers: {}, Spiders: {}, Dolphins: {}, Fish: {}, Villagers: {}, Iron Golems: {}",
            pigs,
            cows,
            chickens,
            sheep,
            zombies,
            skeletons,
            reapers,
            spiders,
            dolphins,
            cod + salmon + pufferfish + tropical_fish,
            villagers,
            iron_golems
        ));
    }
}
