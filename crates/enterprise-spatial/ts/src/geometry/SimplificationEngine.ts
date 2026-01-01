/**
 * Simplification Engine
 * Simplifies geometries while preserving shape
 */

import {
  Geometry,
  LineString,
  Polygon,
  Position,
  SimplificationOptions,
  GeometryError,
} from '../types';
import { GeometryFactory } from './GeometryFactory';

export class SimplificationEngine {
  /**
   * Simplify geometry using specified algorithm
   */
  static simplify(
    geometry: Geometry,
    options: SimplificationOptions
  ): Geometry {
    switch (geometry.type) {
      case 'LineString':
        return this.simplifyLineString(geometry as LineString, options);
      case 'Polygon':
        return this.simplifyPolygon(geometry as Polygon, options);
      case 'MultiLineString':
      case 'MultiPolygon':
        return this.simplifyMultiGeometry(geometry, options);
      default:
        return geometry; // Points don't need simplification
    }
  }

  /**
   * Simplify line string using Douglas-Peucker algorithm
   */
  private static simplifyLineString(
    line: LineString,
    options: SimplificationOptions
  ): LineString {
    const simplified = options.highQuality
      ? this.douglasPeucker(line.coordinates, options.tolerance)
      : this.radialDistance(line.coordinates, options.tolerance);

    return GeometryFactory.createLineString(simplified);
  }

  /**
   * Simplify polygon
   */
  private static simplifyPolygon(
    polygon: Polygon,
    options: SimplificationOptions
  ): Polygon {
    const simplifiedRings = polygon.coordinates.map((ring) => {
      const simplified = options.highQuality
        ? this.douglasPeucker(ring, options.tolerance)
        : this.radialDistance(ring, options.tolerance);

      // Ensure ring is closed
      return GeometryFactory.closeRing(simplified);
    });

    return GeometryFactory.createPolygon(simplifiedRings);
  }

  /**
   * Simplify multi-geometry
   */
  private static simplifyMultiGeometry(
    geometry: Geometry,
    options: SimplificationOptions
  ): Geometry {
    // Would need to handle each component
    return geometry;
  }

  /**
   * Douglas-Peucker algorithm for line simplification
   */
  static douglasPeucker(points: Position[], tolerance: number): Position[] {
    if (points.length <= 2) {
      return points;
    }

    const first = points[0];
    const last = points[points.length - 1];

    // Find point with maximum distance from line
    let maxDist = 0;
    let maxIndex = 0;

    for (let i = 1; i < points.length - 1; i++) {
      const dist = this.perpendicularDistance(points[i], first, last);
      if (dist > maxDist) {
        maxDist = dist;
        maxIndex = i;
      }
    }

    // If max distance is greater than tolerance, recursively simplify
    if (maxDist > tolerance) {
      const left = this.douglasPeucker(points.slice(0, maxIndex + 1), tolerance);
      const right = this.douglasPeucker(points.slice(maxIndex), tolerance);

      // Combine results (remove duplicate middle point)
      return [...left.slice(0, -1), ...right];
    }

    // Return just endpoints
    return [first, last];
  }

  /**
   * Radial distance simplification (faster but less accurate)
   */
  static radialDistance(points: Position[], tolerance: number): Position[] {
    if (points.length <= 2) {
      return points;
    }

    const simplified: Position[] = [points[0]];
    let prev = points[0];

    for (let i = 1; i < points.length; i++) {
      const point = points[i];
      const dist = GeometryFactory.distance(prev, point);

      if (dist >= tolerance) {
        simplified.push(point);
        prev = point;
      }
    }

    // Always include last point
    if (simplified[simplified.length - 1] !== points[points.length - 1]) {
      simplified.push(points[points.length - 1]);
    }

    return simplified;
  }

  /**
   * Visvalingam-Whyatt algorithm (area-based simplification)
   */
  static visvalingamWhyatt(
    points: Position[],
    threshold: number
  ): Position[] {
    if (points.length <= 2) {
      return points;
    }

    // Calculate effective areas for each point
    const areas = this.calculateEffectiveAreas(points);

    // Remove points with smallest areas until threshold
    const keep = new Set<number>();
    keep.add(0); // Always keep first
    keep.add(points.length - 1); // Always keep last

    for (let i = 1; i < points.length - 1; i++) {
      if (areas[i] >= threshold) {
        keep.add(i);
      }
    }

    return points.filter((_, i) => keep.has(i));
  }

  /**
   * Calculate effective areas for Visvalingam-Whyatt
   */
  private static calculateEffectiveAreas(points: Position[]): number[] {
    const areas: number[] = new Array(points.length).fill(Infinity);

    for (let i = 1; i < points.length - 1; i++) {
      areas[i] = this.triangleArea(points[i - 1], points[i], points[i + 1]);
    }

    return areas;
  }

  /**
   * Calculate triangle area
   */
  private static triangleArea(p1: Position, p2: Position, p3: Position): number {
    return Math.abs(
      (p1[0] * (p2[1] - p3[1]) +
        p2[0] * (p3[1] - p1[1]) +
        p3[0] * (p1[1] - p2[1])) /
        2
    );
  }

  /**
   * Calculate perpendicular distance from point to line
   */
  private static perpendicularDistance(
    point: Position,
    lineStart: Position,
    lineEnd: Position
  ): number {
    const [x, y] = point;
    const [x1, y1] = lineStart;
    const [x2, y2] = lineEnd;

    const dx = x2 - x1;
    const dy = y2 - y1;

    if (dx === 0 && dy === 0) {
      // Line is actually a point
      return GeometryFactory.distance(point, lineStart);
    }

    // Calculate perpendicular distance
    const num = Math.abs(dy * x - dx * y + x2 * y1 - y2 * x1);
    const den = Math.sqrt(dx * dx + dy * dy);

    return num / den;
  }

  /**
   * Smooth geometry using Chaikin's algorithm
   */
  static smooth(points: Position[], iterations = 1): Position[] {
    let smoothed = points;

    for (let iter = 0; iter < iterations; iter++) {
      const newPoints: Position[] = [];

      for (let i = 0; i < smoothed.length - 1; i++) {
        const p1 = smoothed[i];
        const p2 = smoothed[i + 1];

        // Add point 1/4 way along segment
        newPoints.push([
          p1[0] * 0.75 + p2[0] * 0.25,
          p1[1] * 0.75 + p2[1] * 0.25,
        ]);

        // Add point 3/4 way along segment
        newPoints.push([
          p1[0] * 0.25 + p2[0] * 0.75,
          p1[1] * 0.25 + p2[1] * 0.75,
        ]);
      }

      // Add last point
      newPoints.push(smoothed[smoothed.length - 1]);

      smoothed = newPoints;
    }

    return smoothed;
  }

  /**
   * Densify geometry by adding intermediate points
   */
  static densify(points: Position[], maxSegmentLength: number): Position[] {
    const densified: Position[] = [points[0]];

    for (let i = 0; i < points.length - 1; i++) {
      const p1 = points[i];
      const p2 = points[i + 1];
      const dist = GeometryFactory.distance(p1, p2);

      if (dist > maxSegmentLength) {
        // Add intermediate points
        const segments = Math.ceil(dist / maxSegmentLength);
        for (let j = 1; j < segments; j++) {
          const t = j / segments;
          densified.push([
            p1[0] + (p2[0] - p1[0]) * t,
            p1[1] + (p2[1] - p1[1]) * t,
          ]);
        }
      }

      densified.push(p2);
    }

    return densified;
  }

  /**
   * Remove spikes from geometry
   */
  static removeSpikes(
    points: Position[],
    angleThreshold = 160
  ): Position[] {
    if (points.length < 3) {
      return points;
    }

    const cleaned: Position[] = [points[0]];

    for (let i = 1; i < points.length - 1; i++) {
      const prev = points[i - 1];
      const curr = points[i];
      const next = points[i + 1];

      const angle = this.calculateAngle(prev, curr, next);

      // Keep point if angle is not too sharp
      if (angle < angleThreshold) {
        cleaned.push(curr);
      }
    }

    cleaned.push(points[points.length - 1]);

    return cleaned;
  }

  /**
   * Calculate angle at point between three consecutive points
   */
  private static calculateAngle(
    p1: Position,
    p2: Position,
    p3: Position
  ): number {
    const v1x = p1[0] - p2[0];
    const v1y = p1[1] - p2[1];
    const v2x = p3[0] - p2[0];
    const v2y = p3[1] - p2[1];

    const dot = v1x * v2x + v1y * v2y;
    const det = v1x * v2y - v1y * v2x;

    const angle = Math.atan2(det, dot);
    return Math.abs((angle * 180) / Math.PI);
  }
}
