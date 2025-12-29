//! Traveling Salesman Problem solver

use crate::error::Result;
use crate::graph::Graph;
use crate::profile::RoutingProfile;
use geo_types::Point;

/// TSP solution
#[derive(Debug, Clone)]
pub struct TspSolution {
    /// Ordered waypoints (tour)
    pub tour: Vec<usize>,

    /// Total tour distance/cost
    pub total_cost: f64,

    /// Computation time
    pub computation_time_ms: u64,
}

/// TSP solver using various heuristics
pub struct TspSolver {
    strategy: TspStrategy,
}

/// TSP solving strategy
#[derive(Debug, Clone, Copy)]
pub enum TspStrategy {
    /// Nearest neighbor heuristic (fast, approximate)
    NearestNeighbor,

    /// 2-opt local search
    TwoOpt,

    /// Simulated annealing
    SimulatedAnnealing,

    /// Genetic algorithm
    GeneticAlgorithm,
}

impl TspSolver {
    pub fn new(strategy: TspStrategy) -> Self {
        Self { strategy }
    }

    /// Solve TSP for given waypoints
    pub fn solve(
        &self,
        distance_matrix: &[Vec<f64>],
    ) -> Result<TspSolution> {
        let start = std::time::Instant::now();

        let tour = match self.strategy {
            TspStrategy::NearestNeighbor => self.nearest_neighbor(distance_matrix),
            TspStrategy::TwoOpt => {
                let mut tour = self.nearest_neighbor(distance_matrix);
                self.two_opt(&mut tour, distance_matrix);
                tour
            }
            TspStrategy::SimulatedAnnealing => self.simulated_annealing(distance_matrix),
            TspStrategy::GeneticAlgorithm => self.genetic_algorithm(distance_matrix),
        };

        let total_cost = self.compute_tour_cost(&tour, distance_matrix);
        let computation_time_ms = start.elapsed().as_millis() as u64;

        Ok(TspSolution {
            tour,
            total_cost,
            computation_time_ms,
        })
    }

    /// Nearest neighbor heuristic
    fn nearest_neighbor(&self, matrix: &[Vec<f64>]) -> Vec<usize> {
        let n = matrix.len();
        if n == 0 {
            return vec![];
        }

        let mut tour = Vec::with_capacity(n);
        let mut visited = vec![false; n];

        // Start at node 0
        let mut current = 0;
        tour.push(current);
        visited[current] = true;

        for _ in 1..n {
            let mut nearest = None;
            let mut min_dist = f64::INFINITY;

            for next in 0..n {
                if !visited[next] && matrix[current][next] < min_dist {
                    min_dist = matrix[current][next];
                    nearest = Some(next);
                }
            }

            if let Some(next) = nearest {
                tour.push(next);
                visited[next] = true;
                current = next;
            }
        }

        tour
    }

    /// 2-opt local search improvement
    fn two_opt(&self, tour: &mut Vec<usize>, matrix: &[Vec<f64>]) {
        let n = tour.len();
        let mut improved = true;

        while improved {
            improved = false;

            for i in 0..n - 1 {
                for j in i + 2..n {
                    let delta = self.two_opt_delta(tour, matrix, i, j);
                    if delta < -1e-6 {
                        // Reverse segment [i+1, j]
                        tour[i + 1..=j].reverse();
                        improved = true;
                    }
                }
            }
        }
    }

    /// Calculate delta for 2-opt swap
    fn two_opt_delta(&self, tour: &[usize], matrix: &[Vec<f64>], i: usize, j: usize) -> f64 {
        let n = tour.len();

        let a = tour[i];
        let b = tour[(i + 1) % n];
        let c = tour[j];
        let d = tour[(j + 1) % n];

        let old_cost = matrix[a][b] + matrix[c][d];
        let new_cost = matrix[a][c] + matrix[b][d];

        new_cost - old_cost
    }

    /// Simulated annealing
    fn simulated_annealing(&self, matrix: &[Vec<f64>]) -> Vec<usize> {
        let mut tour = self.nearest_neighbor(matrix);
        let mut best_tour = tour.clone();
        let mut best_cost = self.compute_tour_cost(&tour, matrix);

        let mut temperature = 1000.0;
        let cooling_rate = 0.995;
        let min_temperature = 0.1;

        while temperature > min_temperature {
            // Random 2-opt move
            let i = rand::random::<usize>() % (tour.len() - 1);
            let j = (i + 2 + rand::random::<usize>() % (tour.len() - i - 2)) % tour.len();

            let delta = self.two_opt_delta(&tour, matrix, i, j);

            if delta < 0.0 || rand::random::<f64>() < (-delta / temperature).exp() {
                tour[i + 1..=j].reverse();

                let current_cost = self.compute_tour_cost(&tour, matrix);
                if current_cost < best_cost {
                    best_cost = current_cost;
                    best_tour = tour.clone();
                }
            }

            temperature *= cooling_rate;
        }

        best_tour
    }

    /// Genetic algorithm (simplified)
    fn genetic_algorithm(&self, matrix: &[Vec<f64>]) -> Vec<usize> {
        // For now, fall back to 2-opt
        let mut tour = self.nearest_neighbor(matrix);
        self.two_opt(&mut tour, matrix);
        tour
    }

    /// Compute total tour cost
    fn compute_tour_cost(&self, tour: &[usize], matrix: &[Vec<f64>]) -> f64 {
        if tour.is_empty() {
            return 0.0;
        }

        let mut cost = 0.0;
        for i in 0..tour.len() {
            let from = tour[i];
            let to = tour[(i + 1) % tour.len()];
            cost += matrix[from][to];
        }
        cost
    }
}

/// Solve TSP for waypoints
pub fn solve(
    graph: &Graph,
    waypoints: &[Point],
    profile: &RoutingProfile,
) -> Result<TspSolution> {
    // Build distance matrix
    let matrix = crate::algorithms::many_to_many::calculate_symmetric_matrix(
        graph,
        waypoints,
        profile,
    )?;

    // Solve with 2-opt
    let solver = TspSolver::new(TspStrategy::TwoOpt);
    solver.solve(&matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_neighbor() {
        let matrix = vec![
            vec![0.0, 10.0, 15.0, 20.0],
            vec![10.0, 0.0, 35.0, 25.0],
            vec![15.0, 35.0, 0.0, 30.0],
            vec![20.0, 25.0, 30.0, 0.0],
        ];

        let solver = TspSolver::new(TspStrategy::NearestNeighbor);
        let solution = solver.solve(&matrix).unwrap();

        assert_eq!(solution.tour.len(), 4);
        assert!(solution.total_cost > 0.0);
    }

    #[test]
    fn test_two_opt() {
        let matrix = vec![
            vec![0.0, 10.0, 15.0, 20.0],
            vec![10.0, 0.0, 35.0, 25.0],
            vec![15.0, 35.0, 0.0, 30.0],
            vec![20.0, 25.0, 30.0, 0.0],
        ];

        let solver = TspSolver::new(TspStrategy::TwoOpt);
        let solution = solver.solve(&matrix).unwrap();

        assert_eq!(solution.tour.len(), 4);
        assert!(solution.total_cost > 0.0);
    }
}
