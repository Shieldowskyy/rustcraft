//! # Minecraft-Style Dynamic Weather Subsystem
//!
//! Simulates clear weather, rain, and thunderstorms with realistic transition intervals,
//! biome-specific precipitation (rain vs. snow vs. dry desert overcast), lightning strikes,
//! lighting attenuation, sky coloration, and sleeping/command integration.

use crate::core::math::Vec3;
use crate::world::Biome;

/// Types of weather conditions supported by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeatherType {
    /// Sunny or starry clear skies without precipitation.
    #[default]
    Clear,
    /// Overcast sky with active rainfall or snowfall in cold biomes.
    Rain,
    /// Dark stormy skies with heavy precipitation, lightning strikes, and thunder claps.
    Thunder,
}

impl WeatherType {
    /// Human-readable English name of the weather type.
    pub fn name(&self) -> &'static str {
        match self {
            WeatherType::Clear => "Clear",
            WeatherType::Rain => "Rain",
            WeatherType::Thunder => "Thunder",
        }
    }

    /// Whether this weather type involves active precipitation (rain or snow).
    #[inline]
    pub fn is_raining(&self) -> bool {
        matches!(self, WeatherType::Rain | WeatherType::Thunder)
    }

    /// Whether this weather type involves thunderstorm activity and lightning.
    #[inline]
    pub fn is_thundering(&self) -> bool {
        matches!(self, WeatherType::Thunder)
    }
}

/// Visual and physical category of precipitation at a specific world coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecipitationKind {
    /// Liquid rainfall (temperate, jungle, swamp, forest, plains, etc.).
    Rain,
    /// Fluffy falling snowflakes (snowy plains, snowy taiga, frozen peaks, etc.).
    Snow,
    /// Overcast or clear skies without active particles (deserts, badlands, savannas).
    None,
}

/// Active visual lightning strike instance with ground impact coordinates and lifetime.
#[derive(Debug, Clone, Copy)]
pub struct LightningStrike {
    /// World coordinates where the bolt strikes the surface.
    pub pos: Vec3,
    /// Remaining visual flash duration in seconds.
    pub timer: f32,
    /// Initial maximum duration for fading.
    pub max_duration: f32,
    /// Procedural branch variation seed.
    pub seed: u32,
}

impl LightningStrike {
    pub fn new(pos: Vec3, duration: f32, seed: u32) -> Self {
        Self {
            pos,
            timer: duration,
            max_duration: duration,
            seed,
        }
    }

    #[inline]
    pub fn alpha(&self) -> f32 {
        (self.timer / self.max_duration).clamp(0.0, 1.0)
    }

    /// Procedurally generates 3D zig-zagging line segments for visual rendering of the lightning bolt.
    pub fn generate_segments(&self) -> Vec<(Vec3, Vec3)> {
        let mut segments = Vec::with_capacity(16);
        let sky_y = (self.pos.y + 45.0).min(90.0);
        let start = Vec3::new(self.pos.x, sky_y, self.pos.z);
        let steps = 10;
        let mut prev = start;
        let mut rng = self.seed;

        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let target_y = sky_y + (self.pos.y - sky_y) * t;

            if i == steps {
                segments.push((prev, self.pos));
                break;
            }

            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let jx = (((rng >> 16) & 0xFF) as f32 / 255.0 - 0.5) * 4.5;
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let jz = (((rng >> 16) & 0xFF) as f32 / 255.0 - 0.5) * 4.5;

            let cur = Vec3::new(
                start.x + (self.pos.x - start.x) * t + jx,
                target_y,
                start.z + (self.pos.z - start.z) * t + jz,
            );
            segments.push((prev, cur));

            // Forked branch for visual realism
            if i == 4 || i == 7 {
                let branch_end = cur + Vec3::new(jx * 1.8, -3.5, jz * 1.8);
                segments.push((cur, branch_end));
            }

            prev = cur;
        }

        segments
    }
}

/// Complete weather simulation state machine.
#[derive(Debug, Clone)]
pub struct WeatherState {
    /// Active macro weather type.
    pub current_type: WeatherType,
    /// Remaining duration in seconds for the current weather type.
    pub timer: f32,
    /// Smooth transition factor [0.0, 1.0] for rain visuals, sound, and darkening.
    pub rain_strength: f32,
    /// Smooth transition factor [0.0, 1.0] for thunderstorm darkness and lightning.
    pub thunder_strength: f32,
    /// Countdown in seconds until the next lightning strike during a storm.
    pub lightning_timer: f32,
    /// Brightness multiplier spike [0.0, 1.0] that illuminates the world during lightning flashes.
    pub lightning_flash: f32,
    /// Active visual lightning bolts.
    pub active_lightning: Vec<LightningStrike>,
    /// Internal PRNG state for weather cycling and random strikes.
    rng_state: u64,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self::new(1337)
    }
}

impl WeatherState {
    /// Creates a new weather simulation initialized to clear skies.
    pub fn new(seed: u32) -> Self {
        let mut state = Self {
            current_type: WeatherType::Clear,
            timer: 900.0, // 15 minutes of initial clear weather
            rain_strength: 0.0,
            thunder_strength: 0.0,
            lightning_timer: 8.0,
            lightning_flash: 0.0,
            active_lightning: Vec::new(),
            rng_state: seed as u64 ^ 0x5DEECE66D,
        };
        state.reset_timer_for(WeatherType::Clear);
        state
    }

    pub fn network_rng_state(&self) -> u64 {
        self.rng_state
    }

    pub fn set_network_state(
        &mut self,
        current_type: WeatherType,
        timer: f32,
        rain_strength: f32,
        thunder_strength: f32,
        lightning_timer: f32,
        lightning_flash: f32,
        rng_state: u64,
        active_lightning: Vec<LightningStrike>,
    ) {
        self.current_type = current_type;
        self.timer = timer;
        self.rain_strength = rain_strength;
        self.thunder_strength = thunder_strength;
        self.lightning_timer = lightning_timer;
        self.lightning_flash = lightning_flash;
        self.rng_state = rng_state;
        self.active_lightning = active_lightning;
    }

    /// Pseudo-random float [0.0, 1.0) using linear congruential generator.
    fn next_f32(&mut self) -> f32 {
        self.rng_state = self.rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.rng_state >> 33) as f32) / (u32::MAX as f32)
    }

    /// Random float in range `[min, max]`.
    fn rand_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Resets the cycle duration for `weather` according to authentic Minecraft timings.
    fn reset_timer_for(&mut self, weather: WeatherType) {
        self.timer = match weather {
            // Clear weather lasts 10 to 25 minutes
            WeatherType::Clear => self.rand_range(600.0, 1500.0),
            // Rain lasts 5 to 12 minutes
            WeatherType::Rain => self.rand_range(300.0, 720.0),
            // Thunderstorms last 3 to 7 minutes
            WeatherType::Thunder => self.rand_range(180.0, 420.0),
        };
    }

    /// Manually sets the weather to `weather` with an optional duration in seconds.
    pub fn set_weather(&mut self, weather: WeatherType, duration_secs: Option<f32>) {
        self.current_type = weather;
        if let Some(dur) = duration_secs {
            self.timer = dur.max(1.0);
        } else {
            self.reset_timer_for(weather);
        }
        if weather == WeatherType::Clear {
            self.lightning_flash = 0.0;
            self.active_lightning.clear();
        } else if weather == WeatherType::Thunder {
            self.lightning_timer = self.rand_range(3.0, 7.0);
        }
    }

    /// Immediately clears weather and resets active lightning (e.g. when sleeping in bed).
    pub fn clear(&mut self) {
        self.set_weather(WeatherType::Clear, None);
    }

    /// Updates the weather simulation over `dt` seconds.
    ///
    /// If a lightning strike occurs, returns the [`LightningStrike`].
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        surface_finder: impl Fn(f32, f32) -> f32,
    ) -> Option<LightningStrike> {
        self.timer -= dt;
        if self.timer <= 0.0 {
            // Transition to next weather state
            let next_type = match self.current_type {
                WeatherType::Clear => {
                    // 75% Rain, 25% Thunder
                    if self.next_f32() < 0.25 {
                        WeatherType::Thunder
                    } else {
                        WeatherType::Rain
                    }
                }
                WeatherType::Rain | WeatherType::Thunder => WeatherType::Clear,
            };
            self.set_weather(next_type, None);
        }

        // Smooth transition for rain strength (3.5s fade in / out)
        let target_rain = if self.current_type.is_raining() { 1.0 } else { 0.0 };
        if self.rain_strength < target_rain {
            self.rain_strength = (self.rain_strength + dt / 3.5).min(target_rain);
        } else if self.rain_strength > target_rain {
            self.rain_strength = (self.rain_strength - dt / 3.5).max(target_rain);
        }

        // Smooth transition for thunder strength (3.5s fade in / out)
        let target_thunder = if self.current_type.is_thundering() { 1.0 } else { 0.0 };
        if self.thunder_strength < target_thunder {
            self.thunder_strength = (self.thunder_strength + dt / 3.5).min(target_thunder);
        } else if self.thunder_strength > target_thunder {
            self.thunder_strength = (self.thunder_strength - dt / 3.5).max(target_thunder);
        }

        // Rapid decay for sudden lightning flashes
        if self.lightning_flash > 0.0 {
            self.lightning_flash = (self.lightning_flash - dt * 4.2).max(0.0);
        }

        // Update active lightning visual instances
        self.active_lightning.retain_mut(|bolt| {
            bolt.timer -= dt;
            bolt.timer > 0.0
        });

        // Thunderstorm lightning strikes
        let mut new_strike = None;
        if self.current_type == WeatherType::Thunder && self.thunder_strength > 0.1 {
            self.lightning_timer -= dt;
            if self.lightning_timer <= 0.0 {
                // Next strike in 4.0 to 11.0 seconds
                self.lightning_timer = self.rand_range(4.0, 11.0);

                // Pick random strike location around player (radius 18m to 64m)
                let angle = self.next_f32() * std::f32::consts::TAU;
                let dist = self.rand_range(18.0, 64.0);
                let strike_x = player_pos.x + angle.cos() * dist;
                let strike_z = player_pos.z + angle.sin() * dist;
                let strike_y = surface_finder(strike_x, strike_z);

                let bolt_pos = Vec3::new(strike_x, strike_y, strike_z);
                let seed = (self.rng_state & 0xFFFF_FFFF) as u32;
                let strike = LightningStrike::new(bolt_pos, 0.28, seed);
                self.active_lightning.push(strike);
                self.lightning_flash = 1.0;
                new_strike = Some(strike);
            }
        }

        new_strike
    }

    /// Whether rain is actively falling and perceptible.
    #[inline]
    pub fn is_raining(&self) -> bool {
        self.rain_strength > 0.02
    }

    /// Whether thunderstorm effects (dark skies, thunder) are active.
    #[inline]
    pub fn is_thundering(&self) -> bool {
        self.thunder_strength > 0.02
    }

    /// Determines the type of precipitation falling in a specific biome.
    #[inline]
    pub fn precipitation_at(&self, biome: Biome) -> PrecipitationKind {
        if !self.is_raining() || !biome.has_precipitation() {
            PrecipitationKind::None
        } else if biome.has_snow() {
            PrecipitationKind::Snow
        } else {
            PrecipitationKind::Rain
        }
    }

    /// Attenuates daylight intensity based on overcast skies and sudden lightning flashes.
    #[inline]
    pub fn modulate_daylight(&self, base_daylight: f32) -> f32 {
        let rain_dim = 1.0 - 0.38 * self.rain_strength;
        let thunder_dim = 1.0 - 0.32 * self.thunder_strength;
        let dimmed = base_daylight * rain_dim * thunder_dim;
        (dimmed + self.lightning_flash * 0.95).clamp(0.04, 1.85)
    }

    /// Modulates cloud color tint towards stormy gray during precipitation.
    pub fn modulate_cloud_color(&self, base_color: [f32; 3]) -> [f32; 3] {
        if self.rain_strength <= 0.01 {
            return base_color;
        }
        let storm_tint = [0.36, 0.38, 0.44];
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        let t = self.rain_strength * 0.75 + self.thunder_strength * 0.20;
        [
            lerp(base_color[0], storm_tint[0], t),
            lerp(base_color[1], storm_tint[1], t),
            lerp(base_color[2], storm_tint[2], t),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_state_transitions() {
        let mut weather = WeatherState::new(42);
        assert_eq!(weather.current_type, WeatherType::Clear);
        assert!(!weather.is_raining());
        assert!(!weather.is_thundering());

        // Set rain
        weather.set_weather(WeatherType::Rain, Some(300.0));
        assert_eq!(weather.current_type, WeatherType::Rain);
        assert_eq!(weather.timer, 300.0);

        // Advance 4 seconds: rain_strength should ramp to 1.0
        weather.update(4.0, Vec3::zero(), |_x, _z| 60.0);
        assert!(weather.is_raining());
        assert!((weather.rain_strength - 1.0).abs() < 0.05);

        // Biome precipitation checks
        assert_eq!(weather.precipitation_at(Biome::Plains), PrecipitationKind::Rain);
        assert_eq!(weather.precipitation_at(Biome::SnowyTundra), PrecipitationKind::Snow);
        assert_eq!(weather.precipitation_at(Biome::Desert), PrecipitationKind::None);

        // Bed sleep clear
        weather.clear();
        assert_eq!(weather.current_type, WeatherType::Clear);
        weather.update(4.0, Vec3::zero(), |_x, _z| 60.0);
        assert!(!weather.is_raining());
        assert_eq!(weather.precipitation_at(Biome::Plains), PrecipitationKind::None);
    }

    #[test]
    fn test_thunderstorm_and_lightning_flash() {
        let mut weather = WeatherState::new(100);
        weather.set_weather(WeatherType::Thunder, Some(200.0));
        weather.thunder_strength = 1.0;
        weather.rain_strength = 1.0;
        weather.lightning_timer = 0.5;

        // Advance 1 second: lightning timer should trigger a strike
        let strike = weather.update(1.0, Vec3::new(10.0, 60.0, 10.0), |_x, _z| 64.0);
        assert!(strike.is_some());
        assert_eq!(weather.active_lightning.len(), 1);
        assert!(weather.lightning_flash > 0.8);
        assert!(weather.modulate_daylight(0.5) > 1.0); // Flash spikes daylight
    }
}
