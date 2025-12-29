//! Pickup and Delivery Problem solver

use geo_types::Point;
use serde::{Deserialize, Serialize};

/// Pickup and Delivery problem
#[derive(Debug, Clone)]
pub struct PickupDeliveryProblem {
    /// Depot location
    pub depot: Point,

    /// Pickup-delivery pairs
    pub pairs: Vec<PickupDeliveryPair>,

    /// Vehicle capacity
    pub vehicle_capacity: f64,

    /// Number of vehicles
    pub num_vehicles: usize,
}

/// Pickup-delivery pair
#[derive(Debug, Clone)]
pub struct PickupDeliveryPair {
    pub id: usize,
    pub pickup_location: Point,
    pub delivery_location: Point,
    pub load: f64,
    pub pickup_time_window: Option<TimeWindow>,
    pub delivery_time_window: Option<TimeWindow>,
}

/// Time window
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TimeWindow {
    pub earliest: f64,
    pub latest: f64,
}

/// Solution to pickup-delivery problem
#[derive(Debug, Clone)]
pub struct PickupDeliverySolution {
    pub routes: Vec<PdRoute>,
    pub total_cost: f64,
    pub unserved_pairs: Vec<usize>,
}

/// Route for pickup-delivery
#[derive(Debug, Clone)]
pub struct PdRoute {
    pub vehicle_id: usize,
    pub sequence: Vec<PdStop>,
    pub total_cost: f64,
}

/// Stop in pickup-delivery route
#[derive(Debug, Clone)]
pub struct PdStop {
    pub pair_id: usize,
    pub stop_type: StopType,
    pub location: Point,
    pub arrival_time: f64,
}

/// Type of stop
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopType {
    Pickup,
    Delivery,
}

/// Solve pickup-delivery problem using insertion heuristic
pub fn solve_pdp(problem: &PickupDeliveryProblem) -> PickupDeliverySolution {
    let mut routes = Vec::new();
    let mut unserved = Vec::new();

    // Simple greedy insertion
    for pair in &problem.pairs {
        let inserted = try_insert_pair(pair, &mut routes, problem.vehicle_capacity);

        if !inserted {
            // Create new route
            if routes.len() < problem.num_vehicles {
                let route = create_route_for_pair(pair, routes.len());
                routes.push(route);
            } else {
                unserved.push(pair.id);
            }
        }
    }

    let total_cost: f64 = routes.iter().map(|r| r.total_cost).sum();

    PickupDeliverySolution {
        routes,
        total_cost,
        unserved_pairs: unserved,
    }
}

/// Try to insert pair into existing routes
fn try_insert_pair(
    pair: &PickupDeliveryPair,
    routes: &mut [PdRoute],
    capacity: f64,
) -> bool {
    for route in routes.iter_mut() {
        // Check capacity
        let current_load: f64 = route
            .sequence
            .iter()
            .map(|s| if s.stop_type == StopType::Pickup { pair.load } else { -pair.load })
            .sum();

        if current_load + pair.load <= capacity {
            // Insert pickup at best position
            route.sequence.push(PdStop {
                pair_id: pair.id,
                stop_type: StopType::Pickup,
                location: pair.pickup_location,
                arrival_time: 0.0,
            });

            // Insert delivery after pickup
            route.sequence.push(PdStop {
                pair_id: pair.id,
                stop_type: StopType::Delivery,
                location: pair.delivery_location,
                arrival_time: 0.0,
            });

            return true;
        }
    }

    false
}

/// Create new route for a pair
fn create_route_for_pair(pair: &PickupDeliveryPair, vehicle_id: usize) -> PdRoute {
    PdRoute {
        vehicle_id,
        sequence: vec![
            PdStop {
                pair_id: pair.id,
                stop_type: StopType::Pickup,
                location: pair.pickup_location,
                arrival_time: 0.0,
            },
            PdStop {
                pair_id: pair.id,
                stop_type: StopType::Delivery,
                location: pair.delivery_location,
                arrival_time: 0.0,
            },
        ],
        total_cost: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdp_basic() {
        let problem = PickupDeliveryProblem {
            depot: Point::new(0.0, 0.0),
            pairs: vec![PickupDeliveryPair {
                id: 0,
                pickup_location: Point::new(1.0, 1.0),
                delivery_location: Point::new(2.0, 2.0),
                load: 10.0,
                pickup_time_window: None,
                delivery_time_window: None,
            }],
            vehicle_capacity: 20.0,
            num_vehicles: 1,
        };

        let solution = solve_pdp(&problem);
        assert_eq!(solution.routes.len(), 1);
        assert_eq!(solution.unserved_pairs.len(), 0);
    }
}
