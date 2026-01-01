/**
 * Constraint Solver - Parametric Constraint Resolution
 * Solves geometric constraints using iterative optimization
 */

import { Constraint, ConstraintType, Point, Shape } from '../types';
import { GeometryEngine } from './GeometryEngine';

interface ConstraintVariable {
  id: string;
  value: number;
  min?: number;
  max?: number;
  locked: boolean;
}

export class ConstraintSolver {
  private constraints: Map<string, Constraint>;
  private variables: Map<string, ConstraintVariable>;
  private shapeMap: Map<string, Shape>;

  private maxIterations: number = 100;
  private convergenceThreshold: number = 1e-6;

  constructor() {
    this.constraints = new Map();
    this.variables = new Map();
    this.shapeMap = new Map();
  }

  /**
   * Add a constraint
   */
  addConstraint(constraint: Constraint): void {
    this.constraints.set(constraint.id, constraint);
  }

  /**
   * Remove a constraint
   */
  removeConstraint(constraintId: string): void {
    this.constraints.delete(constraintId);
  }

  /**
   * Get constraint by ID
   */
  getConstraint(id: string): Constraint | undefined {
    return this.constraints.get(id);
  }

  /**
   * Get all constraints affecting an entity
   */
  getDependencies(entityId: string): Constraint[] {
    const dependencies: Constraint[] = [];

    for (const constraint of this.constraints.values()) {
      if (constraint.entities.includes(entityId)) {
        dependencies.push(constraint);
      }
    }

    return dependencies;
  }

  /**
   * Register a shape for constraint solving
   */
  registerShape(shape: Shape): void {
    this.shapeMap.set(shape.id, shape);
  }

  /**
   * Unregister a shape
   */
  unregisterShape(shapeId: string): void {
    this.shapeMap.delete(shapeId);
  }

  /**
   * Solve all constraints
   */
  solve(iterations?: number, tolerance?: number): boolean {
    const maxIter = iterations ?? this.maxIterations;
    const tol = tolerance ?? this.convergenceThreshold;

    let converged = false;

    for (let iter = 0; iter < maxIter; iter++) {
      let maxError = 0;

      // Evaluate all constraints
      for (const constraint of this.constraints.values()) {
        const error = this.evaluateConstraint(constraint);
        maxError = Math.max(maxError, Math.abs(error));

        if (Math.abs(error) > tol) {
          this.resolveConstraint(constraint, error);
        }
      }

      if (maxError < tol) {
        converged = true;
        break;
      }
    }

    return converged;
  }

  /**
   * Evaluate a constraint and return error
   */
  private evaluateConstraint(constraint: Constraint): number {
    switch (constraint.type) {
      case ConstraintType.Distance:
        return this.evaluateDistanceConstraint(constraint);
      case ConstraintType.Angle:
        return this.evaluateAngleConstraint(constraint);
      case ConstraintType.Parallel:
        return this.evaluateParallelConstraint(constraint);
      case ConstraintType.Perpendicular:
        return this.evaluatePerpendicularConstraint(constraint);
      case ConstraintType.Horizontal:
        return this.evaluateHorizontalConstraint(constraint);
      case ConstraintType.Vertical:
        return this.evaluateVerticalConstraint(constraint);
      case ConstraintType.Coincident:
        return this.evaluateCoincidentConstraint(constraint);
      default:
        return 0;
    }
  }

  /**
   * Resolve a constraint by adjusting entities
   */
  private resolveConstraint(constraint: Constraint, error: number): void {
    const dampingFactor = 0.5; // Damping for stability

    switch (constraint.type) {
      case ConstraintType.Distance:
        this.resolveDistanceConstraint(constraint, error * dampingFactor);
        break;
      case ConstraintType.Angle:
        this.resolveAngleConstraint(constraint, error * dampingFactor);
        break;
      case ConstraintType.Parallel:
        this.resolveParallelConstraint(constraint, error * dampingFactor);
        break;
      case ConstraintType.Perpendicular:
        this.resolvePerpendicularConstraint(constraint, error * dampingFactor);
        break;
      case ConstraintType.Horizontal:
        this.resolveHorizontalConstraint(constraint, error * dampingFactor);
        break;
      case ConstraintType.Vertical:
        this.resolveVerticalConstraint(constraint, error * dampingFactor);
        break;
      case ConstraintType.Coincident:
        this.resolveCoincidentConstraint(constraint, error * dampingFactor);
        break;
    }
  }

  /**
   * Evaluate distance constraint
   */
  private evaluateDistanceConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 2 || !constraint.value) {
      return 0;
    }

    const entity1 = this.shapeMap.get(constraint.entities[0]);
    const entity2 = this.shapeMap.get(constraint.entities[1]);

    if (!entity1 || !entity2) {
      return 0;
    }

    const p1 = this.getEntityPosition(entity1);
    const p2 = this.getEntityPosition(entity2);

    const actualDistance = GeometryEngine.distance(p1, p2);
    return actualDistance - constraint.value;
  }

  /**
   * Resolve distance constraint
   */
  private resolveDistanceConstraint(constraint: Constraint, error: number): void {
    if (constraint.entities.length < 2 || !constraint.value) {
      return;
    }

    const entity1 = this.shapeMap.get(constraint.entities[0]);
    const entity2 = this.shapeMap.get(constraint.entities[1]);

    if (!entity1 || !entity2 || entity1.locked || entity2.locked) {
      return;
    }

    const p1 = this.getEntityPosition(entity1);
    const p2 = this.getEntityPosition(entity2);

    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    const dist = Math.sqrt(dx * dx + dy * dy);

    if (dist < 1e-10) return;

    const correction = error / 2; // Split correction between both entities
    const cx = (dx / dist) * correction;
    const cy = (dy / dist) * correction;

    // Apply correction
    this.moveEntity(entity1, -cx, -cy);
    this.moveEntity(entity2, cx, cy);
  }

  /**
   * Evaluate angle constraint
   */
  private evaluateAngleConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 3 || !constraint.value) {
      return 0;
    }

    const entity1 = this.shapeMap.get(constraint.entities[0]);
    const entity2 = this.shapeMap.get(constraint.entities[1]); // Vertex
    const entity3 = this.shapeMap.get(constraint.entities[2]);

    if (!entity1 || !entity2 || !entity3) {
      return 0;
    }

    const p1 = this.getEntityPosition(entity1);
    const p2 = this.getEntityPosition(entity2);
    const p3 = this.getEntityPosition(entity3);

    const actualAngle = GeometryEngine.angleBetween(p1, p2, p3);
    return actualAngle - constraint.value;
  }

  /**
   * Resolve angle constraint
   */
  private resolveAngleConstraint(constraint: Constraint, error: number): void {
    // Simplified angle resolution
    // Full implementation would rotate entities around vertex
  }

  /**
   * Evaluate parallel constraint
   */
  private evaluateParallelConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 2) {
      return 0;
    }

    // Get direction vectors of both lines
    const angle1 = this.getEntityAngle(constraint.entities[0]);
    const angle2 = this.getEntityAngle(constraint.entities[1]);

    let diff = angle2 - angle1;
    while (diff > Math.PI) diff -= Math.PI;
    while (diff < -Math.PI) diff += Math.PI;

    return diff;
  }

  /**
   * Resolve parallel constraint
   */
  private resolveParallelConstraint(constraint: Constraint, error: number): void {
    // Rotate one entity to match the other
  }

  /**
   * Evaluate perpendicular constraint
   */
  private evaluatePerpendicularConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 2) {
      return 0;
    }

    const angle1 = this.getEntityAngle(constraint.entities[0]);
    const angle2 = this.getEntityAngle(constraint.entities[1]);

    let diff = angle2 - angle1;
    while (diff > Math.PI) diff -= Math.PI;
    while (diff < -Math.PI) diff += Math.PI;

    return Math.abs(diff) - Math.PI / 2;
  }

  /**
   * Resolve perpendicular constraint
   */
  private resolvePerpendicularConstraint(constraint: Constraint, error: number): void {
    // Rotate to achieve perpendicular angle
  }

  /**
   * Evaluate horizontal constraint
   */
  private evaluateHorizontalConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 1) {
      return 0;
    }

    const entity = this.shapeMap.get(constraint.entities[0]);
    if (!entity) return 0;

    // Check if entity is horizontal
    const angle = this.getEntityAngle(constraint.entities[0]);
    return angle % Math.PI; // Should be 0 or π
  }

  /**
   * Resolve horizontal constraint
   */
  private resolveHorizontalConstraint(constraint: Constraint, error: number): void {
    // Align entity horizontally
  }

  /**
   * Evaluate vertical constraint
   */
  private evaluateVerticalConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 1) {
      return 0;
    }

    const angle = this.getEntityAngle(constraint.entities[0]);
    return (angle - Math.PI / 2) % Math.PI;
  }

  /**
   * Resolve vertical constraint
   */
  private resolveVerticalConstraint(constraint: Constraint, error: number): void {
    // Align entity vertically
  }

  /**
   * Evaluate coincident constraint
   */
  private evaluateCoincidentConstraint(constraint: Constraint): number {
    if (constraint.entities.length < 2) {
      return 0;
    }

    const entity1 = this.shapeMap.get(constraint.entities[0]);
    const entity2 = this.shapeMap.get(constraint.entities[1]);

    if (!entity1 || !entity2) {
      return 0;
    }

    const p1 = this.getEntityPosition(entity1);
    const p2 = this.getEntityPosition(entity2);

    return GeometryEngine.distance(p1, p2);
  }

  /**
   * Resolve coincident constraint
   */
  private resolveCoincidentConstraint(constraint: Constraint, error: number): void {
    if (constraint.entities.length < 2) {
      return;
    }

    const entity1 = this.shapeMap.get(constraint.entities[0]);
    const entity2 = this.shapeMap.get(constraint.entities[1]);

    if (!entity1 || !entity2 || entity1.locked || entity2.locked) {
      return;
    }

    const p1 = this.getEntityPosition(entity1);
    const p2 = this.getEntityPosition(entity2);

    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;

    // Move entities to midpoint
    this.moveEntity(entity1, dx / 2, dy / 2);
    this.moveEntity(entity2, -dx / 2, -dy / 2);
  }

  /**
   * Get entity position (simplified)
   */
  private getEntityPosition(entity: Shape): Point {
    const bounds = entity.getBounds();
    return bounds.center;
  }

  /**
   * Get entity angle (simplified)
   */
  private getEntityAngle(entityId: string): number {
    const entity = this.shapeMap.get(entityId);
    if (!entity) return 0;

    // Extract rotation from transform
    return entity.transform.rotation;
  }

  /**
   * Move entity by delta
   */
  private moveEntity(entity: Shape, dx: number, dy: number): void {
    entity.transform.position[0] += dx;
    entity.transform.position[1] += dy;
  }

  /**
   * Clear all constraints
   */
  clear(): void {
    this.constraints.clear();
    this.variables.clear();
    this.shapeMap.clear();
  }

  /**
   * Get constraint statistics
   */
  getStatistics(): {
    totalConstraints: number;
    satisfiedConstraints: number;
    unsatisfiedConstraints: number;
    averageError: number;
  } {
    let satisfied = 0;
    let totalError = 0;

    for (const constraint of this.constraints.values()) {
      const error = Math.abs(this.evaluateConstraint(constraint));
      totalError += error;

      if (error < this.convergenceThreshold) {
        satisfied++;
      }
    }

    return {
      totalConstraints: this.constraints.size,
      satisfiedConstraints: satisfied,
      unsatisfiedConstraints: this.constraints.size - satisfied,
      averageError: this.constraints.size > 0 ? totalError / this.constraints.size : 0
    };
  }
}
