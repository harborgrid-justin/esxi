/**
 * Validation Engine
 * Validates geometries and detects topological errors
 */

import {
  Geometry,
  Point,
  LineString,
  Polygon,
  Position,
  GeometryError,
} from '../types';
import { GeometryFactory } from './GeometryFactory';

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  type: string;
  message: string;
  location?: Position;
  severity: 'error' | 'warning';
}

export interface ValidationWarning {
  type: string;
  message: string;
  location?: Position;
}

export class ValidationEngine {
  /**
   * Validate geometry
   */
  static validate(geometry: Geometry): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    try {
      switch (geometry.type) {
        case 'Point':
          this.validatePoint(geometry as Point, errors, warnings);
          break;
        case 'LineString':
          this.validateLineString(geometry as LineString, errors, warnings);
          break;
        case 'Polygon':
          this.validatePolygon(geometry as Polygon, errors, warnings);
          break;
        case 'MultiPoint':
        case 'MultiLineString':
        case 'MultiPolygon':
          this.validateMultiGeometry(geometry, errors, warnings);
          break;
      }
    } catch (error) {
      errors.push({
        type: 'VALIDATION_EXCEPTION',
        message: error instanceof Error ? error.message : 'Unknown error',
        severity: 'error',
      });
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate point
   */
  private static validatePoint(
    point: Point,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const [x, y, z] = point.coordinates;

    if (!this.isValidNumber(x)) {
      errors.push({
        type: 'INVALID_COORDINATE',
        message: `Invalid X coordinate: ${x}`,
        location: point.coordinates,
        severity: 'error',
      });
    }

    if (!this.isValidNumber(y)) {
      errors.push({
        type: 'INVALID_COORDINATE',
        message: `Invalid Y coordinate: ${y}`,
        location: point.coordinates,
        severity: 'error',
      });
    }

    if (z !== undefined && !this.isValidNumber(z)) {
      errors.push({
        type: 'INVALID_COORDINATE',
        message: `Invalid Z coordinate: ${z}`,
        location: point.coordinates,
        severity: 'error',
      });
    }

    // Check for reasonable coordinate ranges
    if (Math.abs(x) > 180) {
      warnings.push({
        type: 'COORDINATE_OUT_OF_RANGE',
        message: `X coordinate ${x} exceeds typical longitude range`,
        location: point.coordinates,
      });
    }

    if (Math.abs(y) > 90) {
      warnings.push({
        type: 'COORDINATE_OUT_OF_RANGE',
        message: `Y coordinate ${y} exceeds typical latitude range`,
        location: point.coordinates,
      });
    }
  }

  /**
   * Validate line string
   */
  private static validateLineString(
    line: LineString,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    if (line.coordinates.length < 2) {
      errors.push({
        type: 'INSUFFICIENT_POINTS',
        message: 'LineString must have at least 2 points',
        severity: 'error',
      });
      return;
    }

    // Validate each point
    line.coordinates.forEach((pos, i) => {
      if (!this.isValidPosition(pos)) {
        errors.push({
          type: 'INVALID_COORDINATE',
          message: `Invalid coordinate at index ${i}`,
          location: pos,
          severity: 'error',
        });
      }
    });

    // Check for duplicate consecutive points
    for (let i = 0; i < line.coordinates.length - 1; i++) {
      if (this.positionsEqual(line.coordinates[i], line.coordinates[i + 1])) {
        warnings.push({
          type: 'DUPLICATE_POINTS',
          message: `Duplicate consecutive points at index ${i}`,
          location: line.coordinates[i],
        });
      }
    }

    // Check for self-intersection
    if (this.hasSelfIntersection(line)) {
      errors.push({
        type: 'SELF_INTERSECTION',
        message: 'LineString has self-intersection',
        severity: 'error',
      });
    }

    // Check line length
    const length = GeometryFactory.getLength(line);
    if (length === 0) {
      warnings.push({
        type: 'ZERO_LENGTH',
        message: 'LineString has zero length',
      });
    }
  }

  /**
   * Validate polygon
   */
  private static validatePolygon(
    polygon: Polygon,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    if (polygon.coordinates.length === 0) {
      errors.push({
        type: 'NO_RINGS',
        message: 'Polygon has no rings',
        severity: 'error',
      });
      return;
    }

    // Validate each ring
    polygon.coordinates.forEach((ring, ringIndex) => {
      this.validateRing(ring, ringIndex, errors, warnings);
    });

    // Validate exterior ring orientation (should be counter-clockwise)
    if (polygon.coordinates.length > 0) {
      const exteriorRing = polygon.coordinates[0];
      if (!this.isCounterClockwise(exteriorRing)) {
        warnings.push({
          type: 'INCORRECT_RING_ORIENTATION',
          message: 'Exterior ring should be counter-clockwise',
        });
      }
    }

    // Validate hole orientations (should be clockwise)
    for (let i = 1; i < polygon.coordinates.length; i++) {
      if (this.isCounterClockwise(polygon.coordinates[i])) {
        warnings.push({
          type: 'INCORRECT_RING_ORIENTATION',
          message: `Hole ${i} should be clockwise`,
        });
      }
    }

    // Check for self-intersection in rings
    polygon.coordinates.forEach((ring, i) => {
      if (this.ringHasSelfIntersection(ring)) {
        errors.push({
          type: 'SELF_INTERSECTION',
          message: `Ring ${i} has self-intersection`,
          severity: 'error',
        });
      }
    });

    // Check polygon area
    const area = GeometryFactory.getArea(polygon);
    if (area === 0) {
      warnings.push({
        type: 'ZERO_AREA',
        message: 'Polygon has zero area',
      });
    }

    // Check for degenerate polygon
    if (this.isDegenerate(polygon)) {
      errors.push({
        type: 'DEGENERATE_GEOMETRY',
        message: 'Polygon is degenerate',
        severity: 'error',
      });
    }
  }

  /**
   * Validate a ring
   */
  private static validateRing(
    ring: Position[],
    ringIndex: number,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    if (ring.length < 4) {
      errors.push({
        type: 'INSUFFICIENT_POINTS',
        message: `Ring ${ringIndex} must have at least 4 points`,
        severity: 'error',
      });
      return;
    }

    // Validate each position
    ring.forEach((pos, i) => {
      if (!this.isValidPosition(pos)) {
        errors.push({
          type: 'INVALID_COORDINATE',
          message: `Invalid coordinate in ring ${ringIndex} at index ${i}`,
          location: pos,
          severity: 'error',
        });
      }
    });

    // Check if ring is closed
    if (!this.positionsEqual(ring[0], ring[ring.length - 1])) {
      errors.push({
        type: 'RING_NOT_CLOSED',
        message: `Ring ${ringIndex} is not closed`,
        severity: 'error',
      });
    }

    // Check for duplicate consecutive points
    for (let i = 0; i < ring.length - 1; i++) {
      if (this.positionsEqual(ring[i], ring[i + 1])) {
        warnings.push({
          type: 'DUPLICATE_POINTS',
          message: `Duplicate consecutive points in ring ${ringIndex} at index ${i}`,
          location: ring[i],
        });
      }
    }
  }

  /**
   * Validate multi-geometry
   */
  private static validateMultiGeometry(
    geometry: Geometry,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Would validate each component geometry
    if (!geometry.geometries && !(geometry as any).coordinates) {
      errors.push({
        type: 'INVALID_GEOMETRY',
        message: 'Multi-geometry has no components',
        severity: 'error',
      });
    }
  }

  /**
   * Check if position is valid
   */
  private static isValidPosition(pos: Position): boolean {
    return pos.length >= 2 && this.isValidNumber(pos[0]) && this.isValidNumber(pos[1]);
  }

  /**
   * Check if number is valid
   */
  private static isValidNumber(n: number): boolean {
    return typeof n === 'number' && isFinite(n);
  }

  /**
   * Check if two positions are equal
   */
  private static positionsEqual(p1: Position, p2: Position, tolerance = 1e-10): boolean {
    return (
      Math.abs(p1[0] - p2[0]) < tolerance &&
      Math.abs(p1[1] - p2[1]) < tolerance
    );
  }

  /**
   * Check if ring is counter-clockwise
   */
  private static isCounterClockwise(ring: Position[]): boolean {
    let sum = 0;
    for (let i = 0; i < ring.length - 1; i++) {
      sum += (ring[i + 1][0] - ring[i][0]) * (ring[i + 1][1] + ring[i][1]);
    }
    return sum < 0;
  }

  /**
   * Check for self-intersection in line string
   */
  private static hasSelfIntersection(line: LineString): boolean {
    const coords = line.coordinates;

    for (let i = 0; i < coords.length - 3; i++) {
      for (let j = i + 2; j < coords.length - 1; j++) {
        if (this.segmentsIntersect(coords[i], coords[i + 1], coords[j], coords[j + 1])) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Check for self-intersection in ring
   */
  private static ringHasSelfIntersection(ring: Position[]): boolean {
    for (let i = 0; i < ring.length - 2; i++) {
      for (let j = i + 2; j < ring.length - 1; j++) {
        // Skip adjacent segments
        if (j === ring.length - 1 && i === 0) continue;

        if (this.segmentsIntersect(ring[i], ring[i + 1], ring[j], ring[j + 1])) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Check if two segments intersect
   */
  private static segmentsIntersect(
    p1: Position,
    p2: Position,
    p3: Position,
    p4: Position
  ): boolean {
    const ccw = (a: Position, b: Position, c: Position): boolean => {
      return (c[1] - a[1]) * (b[0] - a[0]) > (b[1] - a[1]) * (c[0] - a[0]);
    };

    return ccw(p1, p3, p4) !== ccw(p2, p3, p4) && ccw(p1, p2, p3) !== ccw(p1, p2, p4);
  }

  /**
   * Check if polygon is degenerate
   */
  private static isDegenerate(polygon: Polygon): boolean {
    const area = GeometryFactory.getArea(polygon);
    return area < 1e-10;
  }

  /**
   * Fix common geometry issues
   */
  static fix(geometry: Geometry): Geometry {
    const validation = this.validate(geometry);

    if (validation.valid) {
      return geometry;
    }

    // Fix based on error types
    let fixed = GeometryFactory.clone(geometry);

    if (geometry.type === 'Polygon') {
      fixed = this.fixPolygon(fixed as Polygon);
    } else if (geometry.type === 'LineString') {
      fixed = this.fixLineString(fixed as LineString);
    }

    return fixed;
  }

  /**
   * Fix polygon issues
   */
  private static fixPolygon(polygon: Polygon): Polygon {
    const fixedRings = polygon.coordinates.map((ring, i) => {
      // Close ring if needed
      let fixed = GeometryFactory.closeRing(ring);

      // Fix orientation
      if (i === 0 && !this.isCounterClockwise(fixed)) {
        fixed = fixed.reverse();
      } else if (i > 0 && this.isCounterClockwise(fixed)) {
        fixed = fixed.reverse();
      }

      return fixed;
    });

    return GeometryFactory.createPolygon(fixedRings);
  }

  /**
   * Fix line string issues
   */
  private static fixLineString(line: LineString): LineString {
    let coords = [...line.coordinates];

    // Remove duplicate consecutive points
    coords = coords.filter(
      (pos, i) => i === 0 || !this.positionsEqual(pos, coords[i - 1])
    );

    return GeometryFactory.createLineString(coords);
  }
}
