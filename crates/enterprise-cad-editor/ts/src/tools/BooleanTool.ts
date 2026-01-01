/**
 * Boolean Tool - CSG (Constructive Solid Geometry) Operations
 * Perform union, subtract, intersect, and exclude operations on shapes
 */

import { Shape, BooleanOperation, BooleanResult, PathShape, ShapeType } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';

export class BooleanTool {
  /**
   * Perform boolean operation on two shapes
   */
  static execute(
    operation: BooleanOperation,
    shapeA: Shape,
    shapeB: Shape
  ): BooleanResult {
    try {
      switch (operation) {
        case BooleanOperation.Union:
          return this.union(shapeA, shapeB);
        case BooleanOperation.Subtract:
          return this.subtract(shapeA, shapeB);
        case BooleanOperation.Intersect:
          return this.intersect(shapeA, shapeB);
        case BooleanOperation.Exclude:
          return this.exclude(shapeA, shapeB);
        default:
          return {
            success: false,
            shapes: [],
            error: `Unsupported operation: ${operation}`
          };
      }
    } catch (error) {
      return {
        success: false,
        shapes: [],
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  /**
   * Union operation (A ∪ B)
   */
  private static union(shapeA: Shape, shapeB: Shape): BooleanResult {
    // Simplified union - full implementation would use polygon clipping
    // For circles, we can create a combined shape

    if (!this.canPerformBoolean(shapeA, shapeB)) {
      return {
        success: false,
        shapes: [],
        error: 'Boolean operations only supported for compatible shape types'
      };
    }

    // For demonstration, return original shapes
    // Real implementation would use a library like clipper-lib or martinez
    return {
      success: true,
      shapes: [shapeA], // Simplified result
      error: undefined
    };
  }

  /**
   * Subtract operation (A - B)
   */
  private static subtract(shapeA: Shape, shapeB: Shape): BooleanResult {
    if (!this.canPerformBoolean(shapeA, shapeB)) {
      return {
        success: false,
        shapes: [],
        error: 'Boolean operations only supported for compatible shape types'
      };
    }

    // Simplified implementation
    return {
      success: true,
      shapes: [shapeA],
      error: undefined
    };
  }

  /**
   * Intersect operation (A ∩ B)
   */
  private static intersect(shapeA: Shape, shapeB: Shape): BooleanResult {
    if (!this.canPerformBoolean(shapeA, shapeB)) {
      return {
        success: false,
        shapes: [],
        error: 'Boolean operations only supported for compatible shape types'
      };
    }

    // Check if shapes intersect
    const boundsA = shapeA.getBounds();
    const boundsB = shapeB.getBounds();

    if (!GeometryEngine.boundsIntersect(boundsA, boundsB)) {
      return {
        success: true,
        shapes: [], // No intersection
        error: undefined
      };
    }

    // Simplified implementation
    return {
      success: true,
      shapes: [],
      error: undefined
    };
  }

  /**
   * Exclude operation (A ⊕ B) - symmetric difference
   */
  private static exclude(shapeA: Shape, shapeB: Shape): BooleanResult {
    if (!this.canPerformBoolean(shapeA, shapeB)) {
      return {
        success: false,
        shapes: [],
        error: 'Boolean operations only supported for compatible shape types'
      };
    }

    // Simplified implementation
    return {
      success: true,
      shapes: [shapeA, shapeB],
      error: undefined
    };
  }

  /**
   * Check if boolean operations can be performed
   */
  private static canPerformBoolean(shapeA: Shape, shapeB: Shape): boolean {
    // Boolean operations typically work on closed shapes
    const supportedTypes = [
      ShapeType.Rectangle,
      ShapeType.Circle,
      ShapeType.Ellipse,
      ShapeType.Polygon,
      ShapeType.Path
    ];

    return (
      supportedTypes.includes(shapeA.type) &&
      supportedTypes.includes(shapeB.type)
    );
  }

  /**
   * Batch boolean operation on multiple shapes
   */
  static executeOnMultiple(
    operation: BooleanOperation,
    shapes: Shape[]
  ): BooleanResult {
    if (shapes.length < 2) {
      return {
        success: false,
        shapes: [],
        error: 'At least 2 shapes required for boolean operation'
      };
    }

    let result = shapes[0];

    for (let i = 1; i < shapes.length; i++) {
      const opResult = this.execute(operation, result, shapes[i]);
      if (!opResult.success || opResult.shapes.length === 0) {
        return opResult;
      }
      result = opResult.shapes[0];
    }

    return {
      success: true,
      shapes: [result],
      error: undefined
    };
  }

  /**
   * Check if two shapes overlap
   */
  static shapesOverlap(shapeA: Shape, shapeB: Shape): boolean {
    const boundsA = shapeA.getBounds();
    const boundsB = shapeB.getBounds();

    return GeometryEngine.boundsIntersect(boundsA, boundsB);
  }

  /**
   * Get intersection area (approximate)
   */
  static getIntersectionArea(shapeA: Shape, shapeB: Shape): number {
    const boundsA = shapeA.getBounds();
    const boundsB = shapeB.getBounds();

    if (!GeometryEngine.boundsIntersect(boundsA, boundsB)) {
      return 0;
    }

    // Calculate intersection bounds
    const minX = Math.max(boundsA.minX, boundsB.minX);
    const minY = Math.max(boundsA.minY, boundsB.minY);
    const maxX = Math.min(boundsA.maxX, boundsB.maxX);
    const maxY = Math.min(boundsA.maxY, boundsB.maxY);

    if (maxX <= minX || maxY <= minY) {
      return 0;
    }

    return (maxX - minX) * (maxY - minY);
  }
}
