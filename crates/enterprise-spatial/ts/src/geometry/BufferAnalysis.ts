/**
 * Buffer Analysis
 * Creates buffer zones around geometries
 */

import {
  Geometry,
  Point,
  LineString,
  Polygon,
  Position,
  BufferOptions,
  GeometryError,
} from '../types';
import { GeometryFactory } from './GeometryFactory';

export class BufferAnalysis {
  /**
   * Create a buffer around a geometry
   */
  static buffer(geometry: Geometry, options: BufferOptions): Polygon {
    const distance = this.convertDistance(options.distance, options.units);
    const steps = options.steps || 16;

    switch (geometry.type) {
      case 'Point':
        return this.bufferPoint(geometry as Point, distance, steps);
      case 'LineString':
        return this.bufferLineString(
          geometry as LineString,
          distance,
          steps,
          options.cap || 'round'
        );
      case 'Polygon':
        return this.bufferPolygon(geometry as Polygon, distance, steps);
      default:
        throw new GeometryError(`Buffer not supported for ${geometry.type}`);
    }
  }

  /**
   * Create buffer around a point
   */
  private static bufferPoint(
    point: Point,
    distance: number,
    steps: number
  ): Polygon {
    return GeometryFactory.createCircle(point.coordinates, distance, steps);
  }

  /**
   * Create buffer around a line string
   */
  private static bufferLineString(
    line: LineString,
    distance: number,
    steps: number,
    cap: 'round' | 'flat' | 'square'
  ): Polygon {
    const coords = line.coordinates;
    const bufferedPoints: Position[] = [];

    // Create buffer on both sides of the line
    for (let i = 0; i < coords.length - 1; i++) {
      const p1 = coords[i];
      const p2 = coords[i + 1];

      // Calculate perpendicular offset
      const dx = p2[0] - p1[0];
      const dy = p2[1] - p1[1];
      const len = Math.sqrt(dx * dx + dy * dy);

      const offsetX = (-dy / len) * distance;
      const offsetY = (dx / len) * distance;

      // Add offset points
      bufferedPoints.push([p1[0] + offsetX, p1[1] + offsetY]);
    }

    // Add end cap
    const lastPoint = coords[coords.length - 1];
    if (cap === 'round') {
      const endCapPoints = this.createRoundCap(
        coords[coords.length - 2],
        lastPoint,
        distance,
        steps / 2
      );
      bufferedPoints.push(...endCapPoints);
    } else if (cap === 'square') {
      bufferedPoints.push(...this.createSquareCap(lastPoint, distance));
    }

    // Create buffer on other side (reverse direction)
    for (let i = coords.length - 1; i > 0; i--) {
      const p1 = coords[i];
      const p2 = coords[i - 1];

      const dx = p2[0] - p1[0];
      const dy = p2[1] - p1[1];
      const len = Math.sqrt(dx * dx + dy * dy);

      const offsetX = (-dy / len) * distance;
      const offsetY = (dx / len) * distance;

      bufferedPoints.push([p1[0] + offsetX, p1[1] + offsetY]);
    }

    // Add start cap
    const firstPoint = coords[0];
    if (cap === 'round') {
      const startCapPoints = this.createRoundCap(
        coords[1],
        firstPoint,
        distance,
        steps / 2
      );
      bufferedPoints.push(...startCapPoints);
    } else if (cap === 'square') {
      bufferedPoints.push(...this.createSquareCap(firstPoint, distance));
    }

    // Close the polygon
    bufferedPoints.push(bufferedPoints[0]);

    return GeometryFactory.createPolygon([bufferedPoints]);
  }

  /**
   * Create buffer around a polygon
   */
  private static bufferPolygon(
    polygon: Polygon,
    distance: number,
    steps: number
  ): Polygon {
    const outerRing = polygon.coordinates[0];
    const bufferedPoints: Position[] = [];

    for (let i = 0; i < outerRing.length - 1; i++) {
      const prev = outerRing[i === 0 ? outerRing.length - 2 : i - 1];
      const curr = outerRing[i];
      const next = outerRing[i + 1];

      // Calculate offset based on adjacent segments
      const offset = this.calculateOffset(prev, curr, next, distance);
      bufferedPoints.push(offset);
    }

    bufferedPoints.push(bufferedPoints[0]);

    return GeometryFactory.createPolygon([bufferedPoints]);
  }

  /**
   * Create round cap for line end
   */
  private static createRoundCap(
    prev: Position,
    point: Position,
    distance: number,
    steps: number
  ): Position[] {
    const points: Position[] = [];
    const [x, y] = point;

    // Calculate angle from previous point
    const dx = point[0] - prev[0];
    const dy = point[1] - prev[1];
    const startAngle = Math.atan2(dy, dx);

    for (let i = 0; i <= steps; i++) {
      const angle = startAngle + (i / steps) * Math.PI;
      const px = x + distance * Math.cos(angle);
      const py = y + distance * Math.sin(angle);
      points.push([px, py]);
    }

    return points;
  }

  /**
   * Create square cap for line end
   */
  private static createSquareCap(point: Position, distance: number): Position[] {
    const [x, y] = point;
    return [
      [x - distance, y + distance],
      [x + distance, y + distance],
      [x + distance, y - distance],
      [x - distance, y - distance],
    ];
  }

  /**
   * Calculate offset point for polygon buffering
   */
  private static calculateOffset(
    prev: Position,
    curr: Position,
    next: Position,
    distance: number
  ): Position {
    // Calculate perpendicular vectors
    const dx1 = curr[0] - prev[0];
    const dy1 = curr[1] - prev[1];
    const len1 = Math.sqrt(dx1 * dx1 + dy1 * dy1);

    const dx2 = next[0] - curr[0];
    const dy2 = next[1] - curr[1];
    const len2 = Math.sqrt(dx2 * dx2 + dy2 * dy2);

    // Normalize and get perpendiculars
    const n1x = -dy1 / len1;
    const n1y = dx1 / len1;
    const n2x = -dy2 / len2;
    const n2y = dx2 / len2;

    // Average the normals
    const nx = (n1x + n2x) / 2;
    const ny = (n1y + n2y) / 2;
    const nlen = Math.sqrt(nx * nx + ny * ny);

    // Scale to distance
    const scale = distance / nlen;

    return [curr[0] + nx * scale, curr[1] + ny * scale];
  }

  /**
   * Create multiple buffer rings
   */
  static multiBuffer(
    geometry: Geometry,
    distances: number[],
    options: Omit<BufferOptions, 'distance'>
  ): Polygon[] {
    return distances.map((distance) =>
      this.buffer(geometry, { ...options, distance })
    );
  }

  /**
   * Create variable width buffer along a line
   */
  static variableBuffer(
    line: LineString,
    distances: number[],
    options: Omit<BufferOptions, 'distance'>
  ): Polygon {
    if (distances.length !== line.coordinates.length) {
      throw new GeometryError(
        'Variable buffer requires one distance per coordinate'
      );
    }

    const coords = line.coordinates;
    const leftSide: Position[] = [];
    const rightSide: Position[] = [];

    for (let i = 0; i < coords.length - 1; i++) {
      const p1 = coords[i];
      const p2 = coords[i + 1];
      const dist = distances[i];

      const dx = p2[0] - p1[0];
      const dy = p2[1] - p1[1];
      const len = Math.sqrt(dx * dx + dy * dy);

      const offsetX = (-dy / len) * dist;
      const offsetY = (dx / len) * dist;

      leftSide.push([p1[0] + offsetX, p1[1] + offsetY]);
      rightSide.push([p1[0] - offsetX, p1[1] - offsetY]);
    }

    // Add last point
    const lastIdx = coords.length - 1;
    const lastDist = distances[lastIdx];
    const dx = coords[lastIdx][0] - coords[lastIdx - 1][0];
    const dy = coords[lastIdx][1] - coords[lastIdx - 1][1];
    const len = Math.sqrt(dx * dx + dy * dy);
    const offsetX = (-dy / len) * lastDist;
    const offsetY = (dx / len) * lastDist;

    leftSide.push([
      coords[lastIdx][0] + offsetX,
      coords[lastIdx][1] + offsetY,
    ]);
    rightSide.push([
      coords[lastIdx][0] - offsetX,
      coords[lastIdx][1] - offsetY,
    ]);

    // Combine sides
    const buffered = [...leftSide, ...rightSide.reverse(), leftSide[0]];

    return GeometryFactory.createPolygon([buffered]);
  }

  /**
   * Convert distance to meters based on units
   */
  private static convertDistance(distance: number, units: string): number {
    const conversions: Record<string, number> = {
      meters: 1,
      kilometers: 1000,
      feet: 0.3048,
      miles: 1609.34,
      degrees: 111320, // Approximate at equator
    };

    return distance * (conversions[units] || 1);
  }

  /**
   * Calculate optimal buffer steps based on distance and quality
   */
  static calculateOptimalSteps(distance: number, quality: 'low' | 'medium' | 'high' = 'medium'): number {
    const baseSteps = {
      low: 8,
      medium: 16,
      high: 32,
    };

    return baseSteps[quality];
  }
}
