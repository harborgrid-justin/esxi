//! Weather visualization system

use crate::{Camera, Error, Result};
use glam::{Vec2, Vec3};
use wgpu::RenderPass;
use serde::{Deserialize, Serialize};

/// Weather type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherType {
    /// Clear weather
    Clear,
    /// Rain
    Rain,
    /// Snow
    Snow,
    /// Fog
    Fog,
    /// Clouds only
    Cloudy,
}

/// Weather parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherParameters {
    /// Current weather type
    pub weather_type: WeatherType,

    /// Intensity (0.0 - 1.0)
    pub intensity: f32,

    /// Wind direction
    pub wind_direction: Vec2,

    /// Wind speed
    pub wind_speed: f32,

    /// Cloud coverage (0.0 - 1.0)
    pub cloud_coverage: f32,
}

impl Default for WeatherParameters {
    fn default() -> Self {
        Self {
            weather_type: WeatherType::Clear,
            intensity: 0.0,
            wind_direction: Vec2::new(1.0, 0.0),
            wind_speed: 5.0,
            cloud_coverage: 0.3,
        }
    }
}

/// Weather system
pub struct WeatherSystem {
    /// Weather parameters
    params: WeatherParameters,

    /// Particle systems for rain/snow
    particles: Vec<WeatherParticle>,

    /// Time accumulator
    time: f32,
}

impl WeatherSystem {
    /// Create a new weather system
    pub fn new() -> Self {
        Self {
            params: WeatherParameters::default(),
            particles: Vec::new(),
            time: 0.0,
        }
    }

    /// Create with specific weather
    pub fn with_weather(weather_type: WeatherType, intensity: f32) -> Self {
        let mut system = Self::new();
        system.set_weather(weather_type, intensity);
        system
    }

    /// Set weather type and intensity
    pub fn set_weather(&mut self, weather_type: WeatherType, intensity: f32) {
        self.params.weather_type = weather_type;
        self.params.intensity = intensity.clamp(0.0, 1.0);
        self.regenerate_particles();
    }

    /// Update weather system
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        // Update particles
        for particle in &mut self.particles {
            particle.update(delta_time, &self.params);
        }

        // Remove dead particles and spawn new ones
        self.particles.retain(|p| p.is_alive());

        while self.particles.len() < self.target_particle_count() {
            self.spawn_particle();
        }
    }

    /// Render weather effects
    pub fn render<'a>(&'a self, render_pass: &mut RenderPass<'a>, camera: &Camera) -> Result<()> {
        // Render particles (rain drops, snowflakes)
        // This would typically use a particle rendering system
        Ok(())
    }

    /// Regenerate particles based on current weather
    fn regenerate_particles(&mut self) {
        self.particles.clear();

        let count = self.target_particle_count();
        for _ in 0..count {
            self.spawn_particle();
        }
    }

    /// Calculate target particle count
    fn target_particle_count(&self) -> usize {
        match self.params.weather_type {
            WeatherType::Clear => 0,
            WeatherType::Rain => (5000.0 * self.params.intensity) as usize,
            WeatherType::Snow => (2000.0 * self.params.intensity) as usize,
            WeatherType::Fog => 0, // Fog doesn't use particles
            WeatherType::Cloudy => 0,
        }
    }

    /// Spawn a new particle
    fn spawn_particle(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let position = Vec3::new(
            rng.gen_range(-100.0..100.0),
            rng.gen_range(0.0..50.0),
            rng.gen_range(-100.0..100.0),
        );

        let particle = match self.params.weather_type {
            WeatherType::Rain => WeatherParticle::rain(position),
            WeatherType::Snow => WeatherParticle::snow(position),
            _ => return,
        };

        self.particles.push(particle);
    }

    /// Get parameters
    pub fn params(&self) -> &WeatherParameters {
        &self.params
    }

    /// Set parameters
    pub fn set_params(&mut self, params: WeatherParameters) {
        self.params = params;
        self.regenerate_particles();
    }

    /// Get particle count
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Weather particle (rain drop, snowflake)
struct WeatherParticle {
    /// Position
    position: Vec3,

    /// Velocity
    velocity: Vec3,

    /// Lifetime remaining
    lifetime: f32,

    /// Particle type
    particle_type: WeatherType,
}

impl WeatherParticle {
    /// Create a rain drop
    fn rain(position: Vec3) -> Self {
        Self {
            position,
            velocity: Vec3::new(0.0, -20.0, 0.0), // Falling fast
            lifetime: 5.0,
            particle_type: WeatherType::Rain,
        }
    }

    /// Create a snowflake
    fn snow(position: Vec3) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            position,
            velocity: Vec3::new(
                rng.gen_range(-1.0..1.0),
                -2.0, // Falling slowly
                rng.gen_range(-1.0..1.0),
            ),
            lifetime: 10.0,
            particle_type: WeatherType::Snow,
        }
    }

    /// Update particle
    fn update(&mut self, delta_time: f32, params: &WeatherParameters) {
        // Apply wind
        let wind = Vec3::new(
            params.wind_direction.x * params.wind_speed,
            0.0,
            params.wind_direction.y * params.wind_speed,
        );

        self.velocity += wind * delta_time * 0.1;

        // Update position
        self.position += self.velocity * delta_time;

        // Decrease lifetime
        self.lifetime -= delta_time;

        // Reset if below ground
        if self.position.y < 0.0 {
            self.lifetime = 0.0;
        }
    }

    /// Check if particle is alive
    fn is_alive(&self) -> bool {
        self.lifetime > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_system() {
        let weather = WeatherSystem::new();
        assert_eq!(weather.params().weather_type, WeatherType::Clear);
        assert_eq!(weather.particle_count(), 0);
    }

    #[test]
    fn test_rain_weather() {
        let mut weather = WeatherSystem::with_weather(WeatherType::Rain, 0.5);
        assert_eq!(weather.params().weather_type, WeatherType::Rain);
        assert!(weather.particle_count() > 0);
    }

    #[test]
    fn test_snow_weather() {
        let mut weather = WeatherSystem::with_weather(WeatherType::Snow, 0.8);
        assert_eq!(weather.params().weather_type, WeatherType::Snow);
        assert!(weather.particle_count() > 0);
    }

    #[test]
    fn test_weather_update() {
        let mut weather = WeatherSystem::with_weather(WeatherType::Rain, 0.5);
        let initial_count = weather.particle_count();

        weather.update(1.0);

        // Particles should still exist
        assert!(weather.particle_count() > 0);
    }

    #[test]
    fn test_particle_lifecycle() {
        let mut particle = WeatherParticle::rain(Vec3::new(0.0, 10.0, 0.0));
        assert!(particle.is_alive());

        // Update until particle dies
        for _ in 0..100 {
            particle.update(0.1, &WeatherParameters::default());
        }

        assert!(!particle.is_alive());
    }
}
