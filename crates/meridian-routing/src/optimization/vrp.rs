//! Vehicle Routing Problem solver

use crate::error::Result;
use crate::graph::Graph;
use geo_types::Point;
use serde::{Deserialize, Serialize};

/// VRP problem definition
#[derive(Debug, Clone)]
pub struct VrpProblem {
    /// Depot location
    pub depot: Point,

    /// Customer locations
    pub customers: Vec<Customer>,

    /// Vehicle capacity
    pub vehicle_capacity: f64,

    /// Number of vehicles
    pub num_vehicles: usize,

    /// Maximum route duration
    pub max_route_duration: Option<f64>,
}

/// Customer with demand
#[derive(Debug, Clone)]
pub struct Customer {
    pub location: Point,
    pub demand: f64,
    pub time_window: Option<TimeWindow>,
    pub service_time: f64,
}

/// Time window constraint
#[derive(Debug, Clone, Copy)]
pub struct TimeWindow {
    pub start: f64,
    pub end: f64,
}

/// VRP solution
#[derive(Debug, Clone)]
pub struct VrpSolution {
    /// Routes for each vehicle
    pub routes: Vec<Route>,

    /// Total cost (distance/time)
    pub total_cost: f64,

    /// Total demand served
    pub total_demand: f64,

    /// Computation time
    pub computation_time_ms: u64,
}

/// Single vehicle route
#[derive(Debug, Clone)]
pub struct Route {
    pub vehicle_id: usize,
    pub stops: Vec<usize>,
    pub load: f64,
    pub cost: f64,
}

/// VRP solver
pub struct VrpSolver {
    strategy: VrpStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum VrpStrategy {
    /// Clarke-Wright savings algorithm
    ClarkeWright,

    /// Sweep algorithm
    Sweep,

    /// Genetic algorithm
    Genetic,
}

impl VrpSolver {
    pub fn new(strategy: VrpStrategy) -> Self {
        Self { strategy }
    }

    /// Solve VRP
    pub fn solve(&self, problem: &VrpProblem, distance_matrix: &[Vec<f64>]) -> Result<VrpSolution> {
        let start = std::time::Instant::now();

        let routes = match self.strategy {
            VrpStrategy::ClarkeWright => self.clarke_wright(problem, distance_matrix),
            VrpStrategy::Sweep => self.sweep(problem, distance_matrix),
            VrpStrategy::Genetic => self.genetic(problem, distance_matrix),
        };

        let total_cost: f64 = routes.iter().map(|r| r.cost).sum();
        let total_demand: f64 = routes.iter().map(|r| r.load).sum();
        let computation_time_ms = start.elapsed().as_millis() as u64;

        Ok(VrpSolution {
            routes,
            total_cost,
            total_demand,
            computation_time_ms,
        })
    }

    /// Clarke-Wright savings algorithm
    fn clarke_wright(&self, problem: &VrpProblem, matrix: &[Vec<f64>]) -> Vec<Route> {
        let n = problem.customers.len();
        let mut routes = Vec::new();

        // Start with individual routes for each customer
        let mut customer_routes: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        let mut route_loads: Vec<f64> = problem.customers.iter().map(|c| c.demand).collect();

        // Calculate savings for merging routes
        let mut savings = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                let saving = matrix[0][i + 1] + matrix[0][j + 1] - matrix[i + 1][j + 1];
                savings.push((saving, i, j));
            }
        }

        // Sort by savings (descending)
        savings.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Merge routes based on savings
        for (_, i, j) in savings {
            if customer_routes[i].is_empty() || customer_routes[j].is_empty() {
                continue;
            }

            let combined_load = route_loads[i] + route_loads[j];

            if combined_load <= problem.vehicle_capacity {
                // Merge routes
                let mut merged = customer_routes[i].clone();
                merged.extend_from_slice(&customer_routes[j]);
                customer_routes[i] = merged;
                route_loads[i] = combined_load;

                customer_routes[j].clear();
                route_loads[j] = 0.0;
            }
        }

        // Convert to Route objects
        let mut vehicle_id = 0;
        for (i, stops) in customer_routes.iter().enumerate() {
            if !stops.is_empty() {
                let cost = self.calculate_route_cost(stops, matrix);
                routes.push(Route {
                    vehicle_id,
                    stops: stops.clone(),
                    load: route_loads[i],
                    cost,
                });
                vehicle_id += 1;
            }
        }

        routes
    }

    /// Sweep algorithm
    fn sweep(&self, problem: &VrpProblem, matrix: &[Vec<f64>]) -> Vec<Route> {
        // Simplified sweep: just create routes sequentially
        let mut routes = Vec::new();
        let mut vehicle_id = 0;
        let mut current_route = Vec::new();
        let mut current_load = 0.0;

        for (i, customer) in problem.customers.iter().enumerate() {
            if current_load + customer.demand <= problem.vehicle_capacity {
                current_route.push(i);
                current_load += customer.demand;
            } else {
                // Start new route
                if !current_route.is_empty() {
                    let cost = self.calculate_route_cost(&current_route, matrix);
                    routes.push(Route {
                        vehicle_id,
                        stops: current_route.clone(),
                        load: current_load,
                        cost,
                    });
                    vehicle_id += 1;
                }

                current_route = vec![i];
                current_load = customer.demand;
            }
        }

        // Add last route
        if !current_route.is_empty() {
            let cost = self.calculate_route_cost(&current_route, matrix);
            routes.push(Route {
                vehicle_id,
                stops: current_route,
                load: current_load,
                cost,
            });
        }

        routes
    }

    /// Genetic algorithm (placeholder)
    fn genetic(&self, problem: &VrpProblem, matrix: &[Vec<f64>]) -> Vec<Route> {
        // Fall back to Clarke-Wright
        self.clarke_wright(problem, matrix)
    }

    /// Calculate cost for a route
    fn calculate_route_cost(&self, stops: &[usize], matrix: &[Vec<f64>]) -> f64 {
        if stops.is_empty() {
            return 0.0;
        }

        let mut cost = matrix[0][stops[0] + 1]; // Depot to first customer

        for i in 0..stops.len() - 1 {
            cost += matrix[stops[i] + 1][stops[i + 1] + 1];
        }

        cost += matrix[stops[stops.len() - 1] + 1][0]; // Last customer to depot

        cost
    }
}

/// Solve VRP
pub fn solve(graph: &Graph, problem: &VrpProblem) -> Result<VrpSolution> {
    // Build distance matrix
    let mut all_points = vec![problem.depot];
    all_points.extend(problem.customers.iter().map(|c| c.location));

    let matrix = crate::algorithms::many_to_many::calculate_symmetric_matrix(
        graph,
        &all_points,
        &crate::profile::RoutingProfile::driving(),
    )?;

    let solver = VrpSolver::new(VrpStrategy::ClarkeWright);
    solver.solve(problem, &matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clarke_wright() {
        let problem = VrpProblem {
            depot: Point::new(0.0, 0.0),
            customers: vec![
                Customer {
                    location: Point::new(1.0, 1.0),
                    demand: 10.0,
                    time_window: None,
                    service_time: 5.0,
                },
                Customer {
                    location: Point::new(2.0, 2.0),
                    demand: 15.0,
                    time_window: None,
                    service_time: 5.0,
                },
            ],
            vehicle_capacity: 30.0,
            num_vehicles: 2,
            max_route_duration: None,
        };

        let matrix = vec![
            vec![0.0, 10.0, 20.0],
            vec![10.0, 0.0, 10.0],
            vec![20.0, 10.0, 0.0],
        ];

        let solver = VrpSolver::new(VrpStrategy::ClarkeWright);
        let solution = solver.solve(&problem, &matrix).unwrap();

        assert!(!solution.routes.is_empty());
        assert!(solution.total_cost > 0.0);
    }
}
