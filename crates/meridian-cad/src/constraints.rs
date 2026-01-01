//! Geometric constraints for parametric CAD design
//!
//! This module implements geometric constraints that define relationships between
//! CAD entities. Constraints are resolved by the constraint solver to maintain
//! geometric properties during editing.

use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::primitives::{Line, Point};
use crate::{CadError, CadResult};

/// Geometric constraint defining relationships between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: Uuid,
    pub constraint_type: ConstraintType,
    pub priority: i32,
    pub enabled: bool,
}

impl Constraint {
    /// Create a new constraint
    pub fn new(constraint_type: ConstraintType) -> Self {
        Self {
            id: Uuid::new_v4(),
            constraint_type,
            priority: 0,
            enabled: true,
        }
    }

    /// Create a parallel constraint between two lines
    pub fn parallel(line1_id: Uuid, line2_id: Uuid) -> Self {
        Self::new(ConstraintType::Parallel {
            line1: line1_id,
            line2: line2_id,
        })
    }

    /// Create a perpendicular constraint between two lines
    pub fn perpendicular(line1_id: Uuid, line2_id: Uuid) -> Self {
        Self::new(ConstraintType::Perpendicular {
            line1: line1_id,
            line2: line2_id,
        })
    }

    /// Create a tangent constraint between two entities
    pub fn tangent(entity1_id: Uuid, entity2_id: Uuid) -> Self {
        Self::new(ConstraintType::Tangent {
            entity1: entity1_id,
            entity2: entity2_id,
        })
    }

    /// Create a coincident constraint between two points
    pub fn coincident(point1_id: Uuid, point2_id: Uuid) -> Self {
        Self::new(ConstraintType::Coincident {
            point1: point1_id,
            point2: point2_id,
        })
    }

    /// Create a fixed constraint for a point
    pub fn fixed(point_id: Uuid, position: Point) -> Self {
        Self::new(ConstraintType::Fixed {
            point: point_id,
            position,
        })
    }

    /// Create an angle constraint between two lines
    pub fn angle(line1_id: Uuid, line2_id: Uuid, angle_degrees: f64) -> Self {
        Self::new(ConstraintType::Angle {
            line1: line1_id,
            line2: line2_id,
            angle: angle_degrees.to_radians(),
        })
    }

    /// Create a distance constraint between two points
    pub fn distance(point1_id: Uuid, point2_id: Uuid, distance: f64) -> Self {
        Self::new(ConstraintType::Distance {
            point1: point1_id,
            point2: point2_id,
            distance,
        })
    }

    /// Create a horizontal constraint for a line
    pub fn horizontal(line_id: Uuid) -> Self {
        Self::new(ConstraintType::Horizontal { line: line_id })
    }

    /// Create a vertical constraint for a line
    pub fn vertical(line_id: Uuid) -> Self {
        Self::new(ConstraintType::Vertical { line: line_id })
    }

    /// Create an equal length constraint between two lines
    pub fn equal_length(line1_id: Uuid, line2_id: Uuid) -> Self {
        Self::new(ConstraintType::EqualLength {
            line1: line1_id,
            line2: line2_id,
        })
    }

    /// Create a midpoint constraint
    pub fn midpoint(point_id: Uuid, line_id: Uuid) -> Self {
        Self::new(ConstraintType::Midpoint {
            point: point_id,
            line: line_id,
        })
    }

    /// Create a symmetry constraint
    pub fn symmetric(point1_id: Uuid, point2_id: Uuid, axis_id: Uuid) -> Self {
        Self::new(ConstraintType::Symmetric {
            point1: point1_id,
            point2: point2_id,
            axis: axis_id,
        })
    }

    /// Evaluate constraint error
    pub fn error(&self, state: &ConstraintState) -> CadResult<f64> {
        match &self.constraint_type {
            ConstraintType::Parallel { line1, line2 } => {
                let l1 = state.get_line(*line1)?;
                let l2 = state.get_line(*line2)?;

                let dir1 = l1.direction();
                let dir2 = l2.direction();

                // Error is the absolute value of cross product (sin of angle)
                let cross = dir1.x * dir2.y - dir1.y * dir2.x;
                Ok(cross.abs())
            }

            ConstraintType::Perpendicular { line1, line2 } => {
                let l1 = state.get_line(*line1)?;
                let l2 = state.get_line(*line2)?;

                let dir1 = l1.direction();
                let dir2 = l2.direction();

                // Error is the absolute value of dot product (cos of angle)
                let dot = dir1.x * dir2.x + dir1.y * dir2.y;
                Ok(dot.abs())
            }

            ConstraintType::Coincident { point1, point2 } => {
                let p1 = state.get_point(*point1)?;
                let p2 = state.get_point(*point2)?;

                // Error is the distance between points
                Ok(p1.distance(p2))
            }

            ConstraintType::Fixed { point, position } => {
                let p = state.get_point(*point)?;

                // Error is the distance from fixed position
                Ok(p.distance(position))
            }

            ConstraintType::Angle { line1, line2, angle } => {
                let l1 = state.get_line(*line1)?;
                let l2 = state.get_line(*line2)?;

                let angle1 = l1.angle();
                let angle2 = l2.angle();
                let mut diff = (angle1 - angle2).abs();

                // Normalize angle difference
                if diff > std::f64::consts::PI {
                    diff = 2.0 * std::f64::consts::PI - diff;
                }

                Ok((diff - angle).abs())
            }

            ConstraintType::Distance { point1, point2, distance } => {
                let p1 = state.get_point(*point1)?;
                let p2 = state.get_point(*point2)?;

                let actual_distance = p1.distance(p2);
                Ok((actual_distance - distance).abs())
            }

            ConstraintType::Horizontal { line } => {
                let l = state.get_line(*line)?;

                // Error is the y-component of the direction vector
                Ok((l.end.y - l.start.y).abs())
            }

            ConstraintType::Vertical { line } => {
                let l = state.get_line(*line)?;

                // Error is the x-component of the direction vector
                Ok((l.end.x - l.start.x).abs())
            }

            ConstraintType::EqualLength { line1, line2 } => {
                let l1 = state.get_line(*line1)?;
                let l2 = state.get_line(*line2)?;

                let len1 = l1.length();
                let len2 = l2.length();

                Ok((len1 - len2).abs())
            }

            ConstraintType::Midpoint { point, line } => {
                let p = state.get_point(*point)?;
                let l = state.get_line(*line)?;

                let midpoint = l.midpoint();
                Ok(p.distance(&midpoint))
            }

            ConstraintType::Symmetric { point1, point2, axis } => {
                let p1 = state.get_point(*point1)?;
                let p2 = state.get_point(*point2)?;
                let axis_line = state.get_line(*axis)?;

                // Calculate reflection of p1 across axis
                let axis_dir = axis_line.direction();
                let axis_perp = Vector2::new(-axis_dir.y, axis_dir.x);

                let to_p1 = Vector2::new(p1.x - axis_line.start.x, p1.y - axis_line.start.y);
                let dist_to_axis = to_p1.dot(&axis_perp);

                let reflected = Point::new(
                    p1.x - 2.0 * dist_to_axis * axis_perp.x,
                    p1.y - 2.0 * dist_to_axis * axis_perp.y,
                );

                Ok(reflected.distance(p2))
            }

            ConstraintType::Tangent { entity1, entity2 } => {
                // Simplified tangency check - in real implementation would handle different entity types
                Ok(0.0)
            }

            ConstraintType::Concentric { entity1, entity2 } => {
                // Simplified - would check if two circular entities share the same center
                Ok(0.0)
            }

            ConstraintType::Collinear { line1, line2 } => {
                let l1 = state.get_line(*line1)?;
                let l2 = state.get_line(*line2)?;

                // Check if lines are on the same infinite line
                let dist = l1.distance_to_point(&l2.start);
                Ok(dist)
            }

            ConstraintType::EqualRadius { entity1, entity2 } => {
                // Simplified - would compare radii of circular entities
                Ok(0.0)
            }
        }
    }

    /// Get constraint weight for solver
    pub fn weight(&self) -> f64 {
        match &self.constraint_type {
            ConstraintType::Fixed { .. } => 1000.0,      // Highest priority
            ConstraintType::Coincident { .. } => 100.0,
            ConstraintType::Distance { .. } => 50.0,
            ConstraintType::Angle { .. } => 50.0,
            ConstraintType::Horizontal { .. } => 75.0,
            ConstraintType::Vertical { .. } => 75.0,
            ConstraintType::Parallel { .. } => 25.0,
            ConstraintType::Perpendicular { .. } => 25.0,
            _ => 10.0,
        }
    }
}

/// Types of geometric constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Two lines must be parallel
    Parallel {
        line1: Uuid,
        line2: Uuid,
    },

    /// Two lines must be perpendicular
    Perpendicular {
        line1: Uuid,
        line2: Uuid,
    },

    /// Two entities must be tangent
    Tangent {
        entity1: Uuid,
        entity2: Uuid,
    },

    /// Two points must occupy the same location
    Coincident {
        point1: Uuid,
        point2: Uuid,
    },

    /// Point is fixed at a specific location
    Fixed {
        point: Uuid,
        position: Point,
    },

    /// Two lines must form a specific angle
    Angle {
        line1: Uuid,
        line2: Uuid,
        angle: f64, // radians
    },

    /// Two points must maintain a specific distance
    Distance {
        point1: Uuid,
        point2: Uuid,
        distance: f64,
    },

    /// Line must be horizontal
    Horizontal {
        line: Uuid,
    },

    /// Line must be vertical
    Vertical {
        line: Uuid,
    },

    /// Two lines must have equal length
    EqualLength {
        line1: Uuid,
        line2: Uuid,
    },

    /// Point must be at the midpoint of a line
    Midpoint {
        point: Uuid,
        line: Uuid,
    },

    /// Two points must be symmetric about an axis
    Symmetric {
        point1: Uuid,
        point2: Uuid,
        axis: Uuid,
    },

    /// Two circular entities must be concentric
    Concentric {
        entity1: Uuid,
        entity2: Uuid,
    },

    /// Two lines must be collinear
    Collinear {
        line1: Uuid,
        line2: Uuid,
    },

    /// Two circular entities must have equal radius
    EqualRadius {
        entity1: Uuid,
        entity2: Uuid,
    },
}

impl ConstraintType {
    /// Get human-readable name of constraint type
    pub fn name(&self) -> &str {
        match self {
            ConstraintType::Parallel { .. } => "Parallel",
            ConstraintType::Perpendicular { .. } => "Perpendicular",
            ConstraintType::Tangent { .. } => "Tangent",
            ConstraintType::Coincident { .. } => "Coincident",
            ConstraintType::Fixed { .. } => "Fixed",
            ConstraintType::Angle { .. } => "Angle",
            ConstraintType::Distance { .. } => "Distance",
            ConstraintType::Horizontal { .. } => "Horizontal",
            ConstraintType::Vertical { .. } => "Vertical",
            ConstraintType::EqualLength { .. } => "Equal Length",
            ConstraintType::Midpoint { .. } => "Midpoint",
            ConstraintType::Symmetric { .. } => "Symmetric",
            ConstraintType::Concentric { .. } => "Concentric",
            ConstraintType::Collinear { .. } => "Collinear",
            ConstraintType::EqualRadius { .. } => "Equal Radius",
        }
    }

    /// Get all entity IDs referenced by this constraint
    pub fn referenced_entities(&self) -> Vec<Uuid> {
        match self {
            ConstraintType::Parallel { line1, line2 }
            | ConstraintType::Perpendicular { line1, line2 }
            | ConstraintType::Angle { line1, line2, .. }
            | ConstraintType::EqualLength { line1, line2 }
            | ConstraintType::Collinear { line1, line2 } => vec![*line1, *line2],

            ConstraintType::Tangent { entity1, entity2 }
            | ConstraintType::Concentric { entity1, entity2 }
            | ConstraintType::EqualRadius { entity1, entity2 } => vec![*entity1, *entity2],

            ConstraintType::Coincident { point1, point2 }
            | ConstraintType::Distance { point1, point2, .. } => vec![*point1, *point2],

            ConstraintType::Fixed { point, .. } => vec![*point],

            ConstraintType::Horizontal { line } | ConstraintType::Vertical { line } => vec![*line],

            ConstraintType::Midpoint { point, line } => vec![*point, *line],

            ConstraintType::Symmetric { point1, point2, axis } => {
                vec![*point1, *point2, *axis]
            }
        }
    }
}

/// Constraint state holding current geometry values
#[derive(Debug, Clone)]
pub struct ConstraintState {
    points: HashMap<Uuid, Point>,
    lines: HashMap<Uuid, Line>,
}

impl ConstraintState {
    /// Create a new constraint state
    pub fn new() -> Self {
        Self {
            points: HashMap::new(),
            lines: HashMap::new(),
        }
    }

    /// Add or update a point
    pub fn set_point(&mut self, id: Uuid, point: Point) {
        self.points.insert(id, point);
    }

    /// Get a point by ID
    pub fn get_point(&self, id: Uuid) -> CadResult<&Point> {
        self.points
            .get(&id)
            .ok_or_else(|| CadError::ConstraintError(format!("Point {} not found", id)))
    }

    /// Get a mutable point by ID
    pub fn get_point_mut(&mut self, id: Uuid) -> CadResult<&mut Point> {
        self.points
            .get_mut(&id)
            .ok_or_else(|| CadError::ConstraintError(format!("Point {} not found", id)))
    }

    /// Add or update a line
    pub fn set_line(&mut self, id: Uuid, line: Line) {
        self.lines.insert(id, line);
    }

    /// Get a line by ID
    pub fn get_line(&self, id: Uuid) -> CadResult<&Line> {
        self.lines
            .get(&id)
            .ok_or_else(|| CadError::ConstraintError(format!("Line {} not found", id)))
    }

    /// Get a mutable line by ID
    pub fn get_line_mut(&mut self, id: Uuid) -> CadResult<&mut Line> {
        self.lines
            .get_mut(&id)
            .ok_or_else(|| CadError::ConstraintError(format!("Line {} not found", id)))
    }

    /// Get all points
    pub fn points(&self) -> &HashMap<Uuid, Point> {
        &self.points
    }

    /// Get all lines
    pub fn lines(&self) -> &HashMap<Uuid, Line> {
        &self.lines
    }
}

impl Default for ConstraintState {
    fn default() -> Self {
        Self::new()
    }
}

/// Constraint system managing multiple constraints
#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    constraints: Vec<Constraint>,
    state: ConstraintState,
}

impl ConstraintSystem {
    /// Create a new constraint system
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
            state: ConstraintState::new(),
        }
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) -> Uuid {
        let id = constraint.id;
        self.constraints.push(constraint);
        id
    }

    /// Remove a constraint
    pub fn remove_constraint(&mut self, id: Uuid) -> CadResult<()> {
        let index = self
            .constraints
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| CadError::ConstraintError(format!("Constraint {} not found", id)))?;

        self.constraints.remove(index);
        Ok(())
    }

    /// Get all constraints
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Get constraint state
    pub fn state(&self) -> &ConstraintState {
        &self.state
    }

    /// Get mutable constraint state
    pub fn state_mut(&mut self) -> &mut ConstraintState {
        &mut self.state
    }

    /// Calculate total constraint error
    pub fn total_error(&self) -> CadResult<f64> {
        let mut total = 0.0;
        for constraint in &self.constraints {
            if constraint.enabled {
                let error = constraint.error(&self.state)?;
                total += error * error * constraint.weight();
            }
        }
        Ok(total)
    }

    /// Check if constraints are satisfied (within tolerance)
    pub fn is_satisfied(&self, tolerance: f64) -> CadResult<bool> {
        for constraint in &self.constraints {
            if constraint.enabled {
                let error = constraint.error(&self.state)?;
                if error > tolerance {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Get all constraints affecting an entity
    pub fn constraints_for_entity(&self, entity_id: Uuid) -> Vec<&Constraint> {
        self.constraints
            .iter()
            .filter(|c| c.constraint_type.referenced_entities().contains(&entity_id))
            .collect()
    }

    /// Validate constraint system for conflicts
    pub fn validate(&self) -> CadResult<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for redundant constraints
        for (i, c1) in self.constraints.iter().enumerate() {
            for c2 in self.constraints.iter().skip(i + 1) {
                if std::mem::discriminant(&c1.constraint_type) == std::mem::discriminant(&c2.constraint_type) {
                    let refs1 = c1.constraint_type.referenced_entities();
                    let refs2 = c2.constraint_type.referenced_entities();

                    if refs1 == refs2 {
                        warnings.push(format!(
                            "Redundant constraint: {} appears multiple times",
                            c1.constraint_type.name()
                        ));
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Clear all constraints
    pub fn clear(&mut self) {
        self.constraints.clear();
    }
}

impl Default for ConstraintSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_creation() {
        let constraint = Constraint::parallel(Uuid::new_v4(), Uuid::new_v4());
        assert!(matches!(constraint.constraint_type, ConstraintType::Parallel { .. }));
    }

    #[test]
    fn test_constraint_state() {
        let mut state = ConstraintState::new();
        let id = Uuid::new_v4();
        let point = Point::new(10.0, 20.0);

        state.set_point(id, point);
        let retrieved = state.get_point(id).unwrap();

        assert_eq!(retrieved.x, 10.0);
        assert_eq!(retrieved.y, 20.0);
    }

    #[test]
    fn test_constraint_system() {
        let mut system = ConstraintSystem::new();
        let constraint = Constraint::horizontal(Uuid::new_v4());

        system.add_constraint(constraint);
        assert_eq!(system.constraints().len(), 1);
    }
}
