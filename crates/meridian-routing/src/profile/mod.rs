//! Routing profiles for different transportation modes

pub mod vehicle;

pub use vehicle::VehicleProfile;

use serde::{Deserialize, Serialize};

/// Routing profile defining preferences and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingProfile {
    /// Profile name
    name: String,

    /// Vehicle type
    vehicle: VehicleType,

    /// Average speed (km/h)
    average_speed: f64,

    /// Maximum speed (km/h)
    max_speed: Option<f64>,

    /// Avoid highways
    avoid_highways: bool,

    /// Avoid tolls
    avoid_tolls: bool,

    /// Avoid ferries
    avoid_ferries: bool,

    /// Weight restrictions (kg)
    max_weight: Option<f64>,

    /// Height restrictions (meters)
    max_height: Option<f64>,

    /// Width restrictions (meters)
    max_width: Option<f64>,

    /// Length restrictions (meters)
    max_length: Option<f64>,

    /// Hazardous materials
    hazmat: bool,
}

impl RoutingProfile {
    /// Create a custom profile
    pub fn new(name: impl Into<String>, vehicle: VehicleType) -> Self {
        Self {
            name: name.into(),
            vehicle,
            average_speed: vehicle.default_speed(),
            max_speed: None,
            avoid_highways: false,
            avoid_tolls: false,
            avoid_ferries: false,
            max_weight: None,
            max_height: None,
            max_width: None,
            max_length: None,
            hazmat: false,
        }
    }

    /// Driving profile (car)
    pub fn driving() -> Self {
        Self::new("driving", VehicleType::Car)
    }

    /// Truck profile
    pub fn truck() -> Self {
        let mut profile = Self::new("truck", VehicleType::Truck);
        profile.max_weight = Some(40000.0); // 40 tons
        profile.max_height = Some(4.0); // 4 meters
        profile.max_width = Some(2.5); // 2.5 meters
        profile.max_length = Some(16.5); // 16.5 meters
        profile
    }

    /// Walking profile
    pub fn walking() -> Self {
        Self::new("walking", VehicleType::Pedestrian)
    }

    /// Cycling profile
    pub fn cycling() -> Self {
        Self::new("cycling", VehicleType::Bicycle)
    }

    /// Motorcycle profile
    pub fn motorcycle() -> Self {
        Self::new("motorcycle", VehicleType::Motorcycle)
    }

    /// Get profile name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get vehicle type
    pub fn vehicle_type(&self) -> VehicleType {
        self.vehicle
    }

    /// Get average speed
    pub fn average_speed_kmh(&self) -> f64 {
        self.average_speed
    }

    /// Set average speed
    pub fn with_average_speed(mut self, speed_kmh: f64) -> Self {
        self.average_speed = speed_kmh;
        self
    }

    /// Set avoid highways
    pub fn with_avoid_highways(mut self, avoid: bool) -> Self {
        self.avoid_highways = avoid;
        self
    }

    /// Set avoid tolls
    pub fn with_avoid_tolls(mut self, avoid: bool) -> Self {
        self.avoid_tolls = avoid;
        self
    }

    /// Set maximum weight
    pub fn with_max_weight(mut self, weight_kg: f64) -> Self {
        self.max_weight = Some(weight_kg);
        self
    }

    /// Set maximum height
    pub fn with_max_height(mut self, height_m: f64) -> Self {
        self.max_height = Some(height_m);
        self
    }

    /// Set hazmat flag
    pub fn with_hazmat(mut self, hazmat: bool) -> Self {
        self.hazmat = hazmat;
        self
    }

    /// Check if profile should avoid highways
    pub fn avoids_highways(&self) -> bool {
        self.avoid_highways
    }

    /// Check if profile should avoid tolls
    pub fn avoids_tolls(&self) -> bool {
        self.avoid_tolls
    }

    /// Check if profile should avoid ferries
    pub fn avoids_ferries(&self) -> bool {
        self.avoid_ferries
    }

    /// Get weight restriction
    pub fn max_weight(&self) -> Option<f64> {
        self.max_weight
    }

    /// Get height restriction
    pub fn max_height(&self) -> Option<f64> {
        self.max_height
    }

    /// Check if carrying hazmat
    pub fn is_hazmat(&self) -> bool {
        self.hazmat
    }
}

/// Vehicle type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleType {
    Car,
    Truck,
    Bus,
    Bicycle,
    Pedestrian,
    Motorcycle,
}

impl VehicleType {
    pub fn default_speed(&self) -> f64 {
        match self {
            VehicleType::Car => 50.0,
            VehicleType::Truck => 40.0,
            VehicleType::Bus => 45.0,
            VehicleType::Bicycle => 20.0,
            VehicleType::Pedestrian => 5.0,
            VehicleType::Motorcycle => 55.0,
        }
    }

    pub fn can_use_motorway(&self) -> bool {
        matches!(self, VehicleType::Car | VehicleType::Truck | VehicleType::Bus | VehicleType::Motorcycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driving_profile() {
        let profile = RoutingProfile::driving();
        assert_eq!(profile.name(), "driving");
        assert_eq!(profile.vehicle_type(), VehicleType::Car);
        assert!(profile.average_speed_kmh() > 0.0);
    }

    #[test]
    fn test_truck_profile() {
        let profile = RoutingProfile::truck();
        assert!(profile.max_weight().is_some());
        assert!(profile.max_height().is_some());
    }

    #[test]
    fn test_profile_builder() {
        let profile = RoutingProfile::driving()
            .with_avoid_highways(true)
            .with_avoid_tolls(true)
            .with_average_speed(60.0);

        assert!(profile.avoids_highways());
        assert!(profile.avoids_tolls());
        assert_eq!(profile.average_speed_kmh(), 60.0);
    }
}
