//! Constraint solver using Newton-Raphson iteration
//!
//! This module implements a non-linear constraint solver that uses Newton-Raphson
//! iteration to satisfy geometric constraints in the CAD system.

use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::constraints::{Constraint, ConstraintState, ConstraintSystem};
use crate::primitives::{Line, Point};
use crate::{CadError, CadResult};

/// Constraint solver configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,

    /// Convergence tolerance
    pub tolerance: f64,

    /// Step size for numerical differentiation
    pub epsilon: f64,

    /// Damping factor for Newton-Raphson
    pub damping: f64,

    /// Minimum step size before giving up
    pub min_step: f64,

    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-6,
            epsilon: 1e-8,
            damping: 0.8,
            min_step: 1e-10,
            verbose: false,
        }
    }
}

impl SolverConfig {
    /// Create a fast solver configuration (fewer iterations)
    pub fn fast() -> Self {
        Self {
            max_iterations: 50,
            tolerance: 1e-4,
            ..Default::default()
        }
    }

    /// Create a precise solver configuration (more iterations, tighter tolerance)
    pub fn precise() -> Self {
        Self {
            max_iterations: 200,
            tolerance: 1e-9,
            epsilon: 1e-10,
            damping: 0.9,
            ..Default::default()
        }
    }

    /// Create a robust solver configuration (handles difficult cases)
    pub fn robust() -> Self {
        Self {
            max_iterations: 500,
            tolerance: 1e-6,
            damping: 0.5,
            ..Default::default()
        }
    }
}

/// Constraint solver using Newton-Raphson iteration
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    system: ConstraintSystem,
    config: SolverConfig,
    variables: Vec<Variable>,
    variable_map: HashMap<VariableKey, usize>,
}

impl ConstraintSolver {
    /// Create a new constraint solver
    pub fn new() -> Self {
        Self::with_config(SolverConfig::default())
    }

    /// Create a solver with custom configuration
    pub fn with_config(config: SolverConfig) -> Self {
        Self {
            system: ConstraintSystem::new(),
            config,
            variables: Vec::new(),
            variable_map: HashMap::new(),
        }
    }

    /// Add a constraint to the solver
    pub fn add_constraint(&mut self, constraint: Constraint) -> CadResult<Uuid> {
        let id = self.system.add_constraint(constraint);
        Ok(id)
    }

    /// Remove a constraint
    pub fn remove_constraint(&mut self, id: Uuid) -> CadResult<()> {
        self.system.remove_constraint(id)
    }

    /// Set point position
    pub fn set_point(&mut self, id: Uuid, point: Point) {
        self.system.state_mut().set_point(id, point);
        self.ensure_variable(VariableKey::PointX(id));
        self.ensure_variable(VariableKey::PointY(id));
    }

    /// Set line
    pub fn set_line(&mut self, id: Uuid, line: Line) {
        self.system.state_mut().set_line(id, line);
        // Lines are defined by their endpoints, so we don't need separate variables
    }

    /// Get current state
    pub fn state(&self) -> &ConstraintState {
        self.system.state()
    }

    /// Solve constraints using Newton-Raphson iteration
    pub fn solve(&mut self, max_iterations: Option<usize>) -> CadResult<SolverResult> {
        let max_iter = max_iterations.unwrap_or(self.config.max_iterations);

        // Build variable vector from current state
        self.rebuild_variables();

        let mut iteration = 0;
        let mut converged = false;
        let mut final_error = 0.0;

        if self.config.verbose {
            tracing::info!("Starting constraint solver with {} variables", self.variables.len());
            tracing::info!("Initial constraints: {}", self.system.constraints().len());
        }

        while iteration < max_iter {
            // Evaluate constraint errors
            let errors = self.evaluate_constraints()?;
            let error_norm = errors.norm();

            final_error = error_norm;

            if self.config.verbose && iteration % 10 == 0 {
                tracing::debug!("Iteration {}: error = {}", iteration, error_norm);
            }

            // Check convergence
            if error_norm < self.config.tolerance {
                converged = true;
                break;
            }

            // Calculate Jacobian matrix
            let jacobian = self.calculate_jacobian()?;

            // Solve for delta: J * delta = -errors
            let delta = self.solve_linear_system(&jacobian, &errors)?;

            // Check if step is too small
            if delta.norm() < self.config.min_step {
                if self.config.verbose {
                    tracing::warn!("Step size too small, stopping");
                }
                break;
            }

            // Apply update with damping
            self.apply_update(&delta)?;

            iteration += 1;
        }

        // Update state from variables
        self.update_state_from_variables()?;

        if self.config.verbose {
            tracing::info!(
                "Solver finished: {} iterations, error = {}, converged = {}",
                iteration,
                final_error,
                converged
            );
        }

        Ok(SolverResult {
            converged,
            iterations: iteration,
            final_error,
            tolerance: self.config.tolerance,
        })
    }

    /// Rebuild variable vector from current state
    fn rebuild_variables(&mut self) {
        self.variables.clear();
        self.variable_map.clear();

        // Collect point data first to avoid borrow checker issues
        let points: Vec<_> = self.system.state().points()
            .iter()
            .map(|(id, point)| (*id, *point))
            .collect();

        // Add all point coordinates as variables
        for (id, point) in points {
            self.add_variable(VariableKey::PointX(id), point.x);
            self.add_variable(VariableKey::PointY(id), point.y);
        }
    }

    /// Ensure a variable exists
    fn ensure_variable(&mut self, key: VariableKey) {
        if !self.variable_map.contains_key(&key) {
            let value = match key {
                VariableKey::PointX(id) => {
                    self.system.state().get_point(id).map(|p| p.x).unwrap_or(0.0)
                }
                VariableKey::PointY(id) => {
                    self.system.state().get_point(id).map(|p| p.y).unwrap_or(0.0)
                }
            };
            self.add_variable(key, value);
        }
    }

    /// Add a variable
    fn add_variable(&mut self, key: VariableKey, value: f64) {
        let index = self.variables.len();
        self.variables.push(Variable { key, value });
        self.variable_map.insert(key, index);
    }

    /// Evaluate all constraint errors
    fn evaluate_constraints(&self) -> CadResult<DVector<f64>> {
        let constraints = self.system.constraints();
        let n = constraints.len();
        let mut errors = DVector::zeros(n);

        for (i, constraint) in constraints.iter().enumerate() {
            if constraint.enabled {
                let error = constraint.error(self.system.state())?;
                errors[i] = error * constraint.weight().sqrt();
            }
        }

        Ok(errors)
    }

    /// Calculate Jacobian matrix using numerical differentiation
    fn calculate_jacobian(&self) -> CadResult<DMatrix<f64>> {
        let n_constraints = self.system.constraints().len();
        let n_variables = self.variables.len();

        let mut jacobian = DMatrix::zeros(n_constraints, n_variables);

        // For each variable
        for (var_idx, variable) in self.variables.iter().enumerate() {
            // Create perturbed state
            let mut perturbed_solver = self.clone();
            perturbed_solver.variables[var_idx].value += self.config.epsilon;
            perturbed_solver.update_state_from_variables()?;

            // Evaluate constraints with perturbed state
            let errors_perturbed = perturbed_solver.evaluate_constraints()?;
            let errors_original = self.evaluate_constraints()?;

            // Numerical derivative
            for (constraint_idx, _) in self.system.constraints().iter().enumerate() {
                let derivative = (errors_perturbed[constraint_idx] - errors_original[constraint_idx])
                    / self.config.epsilon;
                jacobian[(constraint_idx, var_idx)] = derivative;
            }
        }

        Ok(jacobian)
    }

    /// Solve linear system J * delta = -errors
    fn solve_linear_system(&self, jacobian: &DMatrix<f64>, errors: &DVector<f64>) -> CadResult<DVector<f64>> {
        let n = self.variables.len();

        // Use least squares if overdetermined: (J^T * J) * delta = -J^T * errors
        let jt = jacobian.transpose();
        let jtj = &jt * jacobian;
        let jte = &jt * errors;

        // Add regularization for numerical stability
        let regularization = 1e-6;
        let jtj_reg = jtj + DMatrix::identity(n, n) * regularization;

        // Solve using LU decomposition
        let lu = jtj_reg.lu();
        let delta = lu.solve(&(-jte))
            .ok_or_else(|| CadError::SolverConvergence("Failed to solve linear system".into()))?;

        Ok(delta)
    }

    /// Apply update to variables
    fn apply_update(&mut self, delta: &DVector<f64>) -> CadResult<()> {
        for (i, variable) in self.variables.iter_mut().enumerate() {
            variable.value += self.config.damping * delta[i];
        }

        self.update_state_from_variables()?;
        Ok(())
    }

    /// Update constraint state from variable values
    fn update_state_from_variables(&mut self) -> CadResult<()> {
        // Group variables by point ID
        let mut point_updates: HashMap<Uuid, Point> = HashMap::new();

        for variable in &self.variables {
            match variable.key {
                VariableKey::PointX(id) => {
                    let point = point_updates
                        .entry(id)
                        .or_insert_with(|| self.system.state().get_point(id).copied().unwrap_or(Point::origin()));
                    point.x = variable.value;
                }
                VariableKey::PointY(id) => {
                    let point = point_updates
                        .entry(id)
                        .or_insert_with(|| self.system.state().get_point(id).copied().unwrap_or(Point::origin()));
                    point.y = variable.value;
                }
            }
        }

        // Apply updates
        for (id, point) in point_updates {
            self.system.state_mut().set_point(id, point);
        }

        Ok(())
    }

    /// Get solver statistics
    pub fn stats(&self) -> SolverStats {
        SolverStats {
            num_constraints: self.system.constraints().len(),
            num_variables: self.variables.len(),
            current_error: self.system.total_error().unwrap_or(f64::INFINITY),
        }
    }

    /// Validate constraint system
    pub fn validate(&self) -> CadResult<Vec<String>> {
        self.system.validate()
    }

    /// Reset solver
    pub fn reset(&mut self) {
        self.system.clear();
        self.variables.clear();
        self.variable_map.clear();
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Variable in the solver
#[derive(Debug, Clone)]
struct Variable {
    key: VariableKey,
    value: f64,
}

/// Variable identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum VariableKey {
    PointX(Uuid),
    PointY(Uuid),
}

/// Result of constraint solving
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverResult {
    /// Whether the solver converged
    pub converged: bool,

    /// Number of iterations performed
    pub iterations: usize,

    /// Final constraint error
    pub final_error: f64,

    /// Convergence tolerance used
    pub tolerance: f64,
}

impl SolverResult {
    /// Check if solution is acceptable
    pub fn is_acceptable(&self) -> bool {
        self.converged || self.final_error < self.tolerance * 10.0
    }

    /// Get quality score (0.0 = poor, 1.0 = perfect)
    pub fn quality(&self) -> f64 {
        if self.converged {
            1.0
        } else {
            (self.tolerance / (self.final_error + self.tolerance)).min(1.0)
        }
    }
}

/// Solver statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverStats {
    pub num_constraints: usize,
    pub num_variables: usize,
    pub current_error: f64,
}

/// Incremental solver for real-time constraint solving
pub struct IncrementalSolver {
    solver: ConstraintSolver,
    last_result: Option<SolverResult>,
}

impl IncrementalSolver {
    /// Create a new incremental solver
    pub fn new() -> Self {
        Self {
            solver: ConstraintSolver::with_config(SolverConfig::fast()),
            last_result: None,
        }
    }

    /// Update and solve incrementally
    pub fn update(&mut self) -> CadResult<&SolverResult> {
        // Use fewer iterations for incremental solving
        let result = self.solver.solve(Some(10))?;
        self.last_result = Some(result);
        Ok(self.last_result.as_ref().unwrap())
    }

    /// Get the underlying solver
    pub fn solver(&self) -> &ConstraintSolver {
        &self.solver
    }

    /// Get mutable solver
    pub fn solver_mut(&mut self) -> &mut ConstraintSolver {
        &mut self.solver
    }

    /// Get last result
    pub fn last_result(&self) -> Option<&SolverResult> {
        self.last_result.as_ref()
    }
}

impl Default for IncrementalSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::Constraint;

    #[test]
    fn test_solver_creation() {
        let solver = ConstraintSolver::new();
        assert_eq!(solver.stats().num_constraints, 0);
    }

    #[test]
    fn test_solver_config() {
        let config = SolverConfig::fast();
        assert_eq!(config.max_iterations, 50);

        let config = SolverConfig::precise();
        assert_eq!(config.max_iterations, 200);
    }

    #[test]
    fn test_fixed_point_constraint() {
        let mut solver = ConstraintSolver::new();

        let point_id = Uuid::new_v4();
        let fixed_pos = Point::new(10.0, 20.0);

        solver.set_point(point_id, Point::new(5.0, 5.0));
        solver.add_constraint(Constraint::fixed(point_id, fixed_pos)).unwrap();

        let result = solver.solve(None).unwrap();

        assert!(result.converged);
        let final_point = solver.state().get_point(point_id).unwrap();
        assert!((final_point.x - 10.0).abs() < 0.001);
        assert!((final_point.y - 20.0).abs() < 0.001);
    }
}
