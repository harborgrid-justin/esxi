//! Sun/directional light with time-of-day simulation

use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Timelike, Utc};

/// Sun light parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunParameters {
    /// Geographic latitude (degrees)
    pub latitude: f32,

    /// Geographic longitude (degrees)
    pub longitude: f32,

    /// Date and time (UTC)
    pub datetime: Option<DateTime<Utc>>,

    /// Sun azimuth override (degrees, 0 = North)
    pub azimuth_override: Option<f32>,

    /// Sun elevation override (degrees above horizon)
    pub elevation_override: Option<f32>,

    /// Sun intensity multiplier
    pub intensity: f32,

    /// Sun color temperature (Kelvin)
    pub temperature: f32,
}

impl Default for SunParameters {
    fn default() -> Self {
        Self {
            latitude: 37.7749,    // San Francisco
            longitude: -122.4194,
            datetime: None,
            azimuth_override: None,
            elevation_override: None,
            intensity: 1.0,
            temperature: 5500.0, // Daylight white
        }
    }
}

/// Sun/directional light
pub struct SunLight {
    /// Sun parameters
    params: SunParameters,

    /// Current sun direction (world space)
    direction: Vec3,

    /// Current sun color
    color: Vec3,

    /// Current intensity
    intensity: f32,

    /// Cast shadows
    cast_shadows: bool,
}

impl SunLight {
    /// Create a new sun light
    pub fn new(params: SunParameters) -> Self {
        let mut sun = Self {
            params,
            direction: Vec3::new(0.0, -1.0, 0.0),
            color: Vec3::ONE,
            intensity: 1.0,
            cast_shadows: true,
        };

        sun.calculate_sun_position();
        sun.calculate_sun_color();

        sun
    }

    /// Create sun from time and location
    pub fn from_time(
        datetime: DateTime<Utc>,
        latitude: f32,
        longitude: f32,
    ) -> Self {
        let params = SunParameters {
            latitude,
            longitude,
            datetime: Some(datetime),
            ..Default::default()
        };

        Self::new(params)
    }

    /// Update sun position based on time
    pub fn update(&mut self, time: f32) {
        // If using time-of-day animation
        if self.params.datetime.is_none() {
            // Simple day/night cycle
            let hour = (time % 24.0) * 15.0; // 15 degrees per hour
            let elevation = Self::simple_elevation_curve(hour);
            let azimuth = hour * 15.0;

            self.direction = Self::direction_from_angles(azimuth, elevation);
        } else {
            self.calculate_sun_position();
        }

        self.calculate_sun_color();
    }

    /// Calculate sun position from parameters
    fn calculate_sun_position(&mut self) {
        if let Some(azimuth) = self.params.azimuth_override {
            let elevation = self.params.elevation_override.unwrap_or(45.0);
            self.direction = Self::direction_from_angles(azimuth, elevation);
            return;
        }

        if let Some(datetime) = self.params.datetime {
            // Solar position calculation
            let (azimuth, elevation) = Self::calculate_solar_position(
                datetime,
                self.params.latitude,
                self.params.longitude,
            );

            self.direction = Self::direction_from_angles(azimuth, elevation);
        } else {
            // Default noon position
            self.direction = Self::direction_from_angles(180.0, 45.0);
        }
    }

    /// Calculate solar position (simplified)
    fn calculate_solar_position(
        datetime: DateTime<Utc>,
        latitude: f32,
        longitude: f32,
    ) -> (f32, f32) {
        // Simplified solar position algorithm
        // In a real implementation, use a proper astronomical algorithm

        let hour = datetime.hour() as f32 + datetime.minute() as f32 / 60.0;

        // Solar noon is at 12:00 + longitude offset
        let solar_noon = 12.0 - longitude / 15.0;
        let hour_angle = (hour - solar_noon) * 15.0;

        // Simplified elevation calculation
        let elevation = 90.0 - latitude.abs() + 23.5 * (hour / 24.0 * std::f32::consts::TAU).sin();

        // Azimuth (0 = North, 90 = East, 180 = South, 270 = West)
        let azimuth = 180.0 + hour_angle;

        (azimuth, elevation.max(0.0))
    }

    /// Simple elevation curve for day/night cycle
    fn simple_elevation_curve(time_of_day: f32) -> f32 {
        // Sunrise at 6:00 (90°), noon at 12:00 (180°), sunset at 18:00 (270°)
        let angle = (time_of_day - 6.0) * 15.0;
        let elevation = 90.0 * (angle.to_radians()).sin();

        elevation.max(-10.0) // Clamp to prevent going too far below horizon
    }

    /// Convert azimuth/elevation angles to direction vector
    fn direction_from_angles(azimuth_deg: f32, elevation_deg: f32) -> Vec3 {
        let azimuth = azimuth_deg.to_radians();
        let elevation = elevation_deg.to_radians();

        let x = elevation.cos() * azimuth.sin();
        let y = -elevation.sin(); // Negative because light direction points down
        let z = -elevation.cos() * azimuth.cos();

        Vec3::new(x, y, z).normalize()
    }

    /// Calculate sun color based on elevation and temperature
    fn calculate_sun_color(&mut self) {
        let elevation = self.elevation_angle();

        // Color shifts based on sun elevation
        let base_color = Self::temperature_to_rgb(self.params.temperature);

        // Sunrise/sunset shift to orange/red
        let sunset_factor = (1.0 - (elevation / 30.0).min(1.0)).max(0.0);

        let color = Vec3::new(
            base_color.x + sunset_factor * 0.3,
            base_color.y - sunset_factor * 0.2,
            base_color.z - sunset_factor * 0.4,
        );

        self.color = color.clamp(Vec3::ZERO, Vec3::ONE);

        // Intensity based on elevation
        self.intensity = if elevation > 0.0 {
            self.params.intensity * (elevation / 90.0).min(1.0)
        } else {
            0.0
        };
    }

    /// Convert color temperature to RGB
    fn temperature_to_rgb(kelvin: f32) -> Vec3 {
        let temp = kelvin / 100.0;

        let red = if temp <= 66.0 {
            1.0
        } else {
            let r = temp - 60.0;
            (329.698727446 * r.powf(-0.1332047592)) / 255.0
        };

        let green = if temp <= 66.0 {
            (99.4708025861 * (temp).ln() - 161.1195681661) / 255.0
        } else {
            (288.1221695283 * (temp - 60.0).powf(-0.0755148492)) / 255.0
        };

        let blue = if temp >= 66.0 {
            1.0
        } else if temp <= 19.0 {
            0.0
        } else {
            (138.5177312231 * (temp - 10.0).ln() - 305.0447927307) / 255.0
        };

        Vec3::new(
            red.clamp(0.0, 1.0),
            green.clamp(0.0, 1.0),
            blue.clamp(0.0, 1.0),
        )
    }

    /// Get sun direction
    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    /// Get sun color
    pub fn color(&self) -> Vec3 {
        self.color
    }

    /// Get sun intensity
    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    /// Check if sun casts shadows
    pub fn cast_shadows(&self) -> bool {
        self.cast_shadows
    }

    /// Set shadow casting
    pub fn set_cast_shadows(&mut self, cast: bool) {
        self.cast_shadows = cast;
    }

    /// Get elevation angle (degrees above horizon)
    pub fn elevation_angle(&self) -> f32 {
        (-self.direction.y).asin().to_degrees()
    }

    /// Get azimuth angle (degrees from North)
    pub fn azimuth_angle(&self) -> f32 {
        self.direction.x.atan2(-self.direction.z).to_degrees() + 180.0
    }

    /// Check if it's daytime
    pub fn is_daytime(&self) -> bool {
        self.elevation_angle() > 0.0
    }

    /// Check if it's nighttime
    pub fn is_nighttime(&self) -> bool {
        !self.is_daytime()
    }

    /// Get time of day as string
    pub fn time_of_day_string(&self) -> &str {
        let elevation = self.elevation_angle();

        if elevation < -6.0 {
            "Night"
        } else if elevation < 0.0 {
            "Dusk"
        } else if elevation < 10.0 {
            "Dawn"
        } else if elevation < 60.0 {
            "Morning/Afternoon"
        } else {
            "Noon"
        }
    }

    /// Get parameters
    pub fn params(&self) -> &SunParameters {
        &self.params
    }

    /// Update parameters
    pub fn set_params(&mut self, params: SunParameters) {
        self.params = params;
        self.calculate_sun_position();
        self.calculate_sun_color();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sun_creation() {
        let sun = SunLight::new(SunParameters::default());

        assert!(sun.direction().length() > 0.99);
        assert!(sun.direction().length() < 1.01);
    }

    #[test]
    fn test_sun_from_time() {
        let datetime = Utc::now();
        let sun = SunLight::from_time(datetime, 37.7749, -122.4194);

        assert!(sun.elevation_angle() >= -90.0 && sun.elevation_angle() <= 90.0);
        assert!(sun.azimuth_angle() >= 0.0 && sun.azimuth_angle() <= 360.0);
    }

    #[test]
    fn test_day_night() {
        let mut sun = SunLight::new(SunParameters {
            elevation_override: Some(45.0),
            ..Default::default()
        });

        assert!(sun.is_daytime());

        sun.params.elevation_override = Some(-10.0);
        sun.calculate_sun_position();

        assert!(sun.is_nighttime());
    }

    #[test]
    fn test_temperature_to_rgb() {
        let warm = SunLight::temperature_to_rgb(3000.0); // Warm/sunset
        let cool = SunLight::temperature_to_rgb(6500.0); // Cool/noon

        assert!(warm.x > warm.z); // More red than blue
        assert!(cool.z >= cool.x); // More blue than red
    }

    #[test]
    fn test_sun_update() {
        let mut sun = SunLight::new(SunParameters::default());

        let initial_dir = sun.direction();

        sun.update(12.0); // Noon
        let noon_dir = sun.direction();

        sun.update(18.0); // Sunset
        let sunset_dir = sun.direction();

        // Direction should change with time
        assert_ne!(initial_dir, noon_dir);
        assert_ne!(noon_dir, sunset_dir);
    }
}
