//! Unit tests for dynamic weather transitions, biome precipitation, and lightning flashes.
//! Migrated from `src/world/weather.rs`.

use crate::core::math::Vec3;

use super::biome::Biome;
use super::weather::{PrecipitationKind, WeatherState, WeatherType};


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
