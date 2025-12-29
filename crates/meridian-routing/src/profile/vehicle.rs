//! Vehicle profiles with detailed specifications

use serde::{Deserialize, Serialize};

/// Detailed vehicle profile with specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleProfile {
    /// Vehicle name
    pub name: String,

    /// Vehicle specifications
    pub specs: VehicleSpecs,

    /// Cost model
    pub cost_model: CostModel,

    /// Capabilities
    pub capabilities: VehicleCapabilities,
}

impl VehicleProfile {
    /// Create new vehicle profile
    pub fn new(name: impl Into<String>, specs: VehicleSpecs) -> Self {
        Self {
            name: name.into(),
            specs,
            cost_model: CostModel::default(),
            capabilities: VehicleCapabilities::default(),
        }
    }

    /// Create standard car profile
    pub fn standard_car() -> Self {
        Self::new(
            "standard_car",
            VehicleSpecs {
                weight_kg: 1500.0,
                height_m: 1.5,
                width_m: 1.8,
                length_m: 4.5,
                max_speed_kmh: 180.0,
                acceleration: 3.0,
                fuel_type: FuelType::Gasoline,
                fuel_capacity_l: 50.0,
                consumption_per_100km: 7.0,
            },
        )
    }

    /// Create delivery van profile
    pub fn delivery_van() -> Self {
        Self::new(
            "delivery_van",
            VehicleSpecs {
                weight_kg: 2500.0,
                height_m: 2.2,
                width_m: 2.0,
                length_m: 5.5,
                max_speed_kmh: 130.0,
                acceleration: 2.0,
                fuel_type: FuelType::Diesel,
                fuel_capacity_l: 70.0,
                consumption_per_100km: 9.0,
            },
        )
    }

    /// Create heavy truck profile
    pub fn heavy_truck() -> Self {
        let mut profile = Self::new(
            "heavy_truck",
            VehicleSpecs {
                weight_kg: 40000.0,
                height_m: 4.0,
                width_m: 2.5,
                length_m: 16.5,
                max_speed_kmh: 90.0,
                acceleration: 1.0,
                fuel_type: FuelType::Diesel,
                fuel_capacity_l: 400.0,
                consumption_per_100km: 30.0,
            },
        );

        profile.capabilities.requires_truck_routes = true;
        profile.capabilities.restricted_in_cities = true;

        profile
    }

    /// Create electric vehicle profile
    pub fn electric_vehicle() -> Self {
        Self::new(
            "electric_vehicle",
            VehicleSpecs {
                weight_kg: 1800.0,
                height_m: 1.5,
                width_m: 1.8,
                length_m: 4.5,
                max_speed_kmh: 150.0,
                acceleration: 5.0,
                fuel_type: FuelType::Electric,
                fuel_capacity_l: 60.0, // kWh equivalent
                consumption_per_100km: 15.0, // kWh/100km
            },
        )
    }

    /// Calculate route cost
    pub fn calculate_cost(&self, distance_km: f64, duration_hours: f64) -> f64 {
        let fuel_cost = (distance_km / 100.0) * self.specs.consumption_per_100km * self.cost_model.fuel_price_per_liter;
        let time_cost = duration_hours * self.cost_model.hourly_cost;
        let distance_cost = distance_km * self.cost_model.cost_per_km;

        fuel_cost + time_cost + distance_cost
    }

    /// Check if vehicle can fit through restriction
    pub fn fits_restriction(&self, max_height: Option<f64>, max_weight: Option<f64>) -> bool {
        if let Some(height_limit) = max_height {
            if self.specs.height_m > height_limit {
                return false;
            }
        }

        if let Some(weight_limit) = max_weight {
            if self.specs.weight_kg > weight_limit {
                return false;
            }
        }

        true
    }
}

/// Vehicle specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleSpecs {
    /// Total weight (kg)
    pub weight_kg: f64,

    /// Height (meters)
    pub height_m: f64,

    /// Width (meters)
    pub width_m: f64,

    /// Length (meters)
    pub length_m: f64,

    /// Maximum speed (km/h)
    pub max_speed_kmh: f64,

    /// Acceleration (m/s²)
    pub acceleration: f64,

    /// Fuel type
    pub fuel_type: FuelType,

    /// Fuel capacity (liters or kWh)
    pub fuel_capacity_l: f64,

    /// Fuel consumption (per 100km)
    pub consumption_per_100km: f64,
}

/// Fuel type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FuelType {
    Gasoline,
    Diesel,
    Electric,
    Hybrid,
    CNG,
    LPG,
    Hydrogen,
}

impl FuelType {
    pub fn emissions_factor(&self) -> f64 {
        match self {
            FuelType::Gasoline => 2.31,  // kg CO2 per liter
            FuelType::Diesel => 2.68,
            FuelType::Electric => 0.0,   // Simplified
            FuelType::Hybrid => 1.5,
            FuelType::CNG => 1.9,
            FuelType::LPG => 1.5,
            FuelType::Hydrogen => 0.0,
        }
    }
}

/// Cost model for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModel {
    /// Cost per kilometer
    pub cost_per_km: f64,

    /// Hourly cost (driver, etc.)
    pub hourly_cost: f64,

    /// Fuel price per liter
    pub fuel_price_per_liter: f64,

    /// Toll cost multiplier
    pub toll_multiplier: f64,

    /// Ferry cost multiplier
    pub ferry_multiplier: f64,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            cost_per_km: 0.5,
            hourly_cost: 25.0,
            fuel_price_per_liter: 1.5,
            toll_multiplier: 1.0,
            ferry_multiplier: 1.0,
        }
    }
}

/// Vehicle capabilities and restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCapabilities {
    /// Can use highways
    pub can_use_highways: bool,

    /// Requires truck routes
    pub requires_truck_routes: bool,

    /// Restricted in city centers
    pub restricted_in_cities: bool,

    /// Can use HOV lanes
    pub can_use_hov: bool,

    /// Can use bus lanes
    pub can_use_bus_lanes: bool,

    /// Requires charging/refueling stops
    pub requires_charging: bool,

    /// Range (km)
    pub range_km: Option<f64>,
}

impl Default for VehicleCapabilities {
    fn default() -> Self {
        Self {
            can_use_highways: true,
            requires_truck_routes: false,
            restricted_in_cities: false,
            can_use_hov: false,
            can_use_bus_lanes: false,
            requires_charging: false,
            range_km: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vehicle_profiles() {
        let car = VehicleProfile::standard_car();
        assert_eq!(car.specs.fuel_type, FuelType::Gasoline);

        let truck = VehicleProfile::heavy_truck();
        assert!(truck.capabilities.requires_truck_routes);

        let ev = VehicleProfile::electric_vehicle();
        assert_eq!(ev.specs.fuel_type, FuelType::Electric);
    }

    #[test]
    fn test_cost_calculation() {
        let car = VehicleProfile::standard_car();
        let cost = car.calculate_cost(100.0, 1.5);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_restrictions() {
        let truck = VehicleProfile::heavy_truck();
        assert!(!truck.fits_restriction(Some(3.0), None)); // Too tall
        assert!(truck.fits_restriction(Some(5.0), None)); // OK
    }
}
