/**
 * Path Simplification - Douglas-Peucker Algorithm
 * Simplify paths while preserving shape
 */

import { Point } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';

export class PathSimplification {
  /**
   * Simplify path using Douglas-Peucker algorithm
   */
  static douglasPeucker(points: Point[], tolerance: number): Point[] {
    if (points.length <= 2) {
      return points;
    }

    // Find point with maximum distance from line segment
    let maxDistance = 0;
    let maxIndex = 0;

    const start = points[0];
    const end = points[points.length - 1];

    for (let i = 1; i < points.length - 1; i++) {
      const projection = GeometryEngine.projectPointOnLine(points[i], start, end);
      if (projection.distance > maxDistance) {
        maxDistance = projection.distance;
        maxIndex = i;
      }
    }

    // If max distance is greater than tolerance, recursively simplify
    if (maxDistance > tolerance) {
      // Recursive call for both segments
      const left = this.douglasPeucker(points.slice(0, maxIndex + 1), tolerance);
      const right = this.douglasPeucker(points.slice(maxIndex), tolerance);

      // Combine results (remove duplicate point at junction)
      return [...left.slice(0, -1), ...right];
    } else {
      // All points can be removed except endpoints
      return [start, end];
    }
  }

  /**
   * Simplify using Ramer-Douglas-Peucker variant with area threshold
   */
  static simplifyByArea(points: Point[], areaThreshold: number): Point[] {
    if (points.length <= 2) {
      return points;
    }

    const simplified: Point[] = [points[0]];

    for (let i = 1; i < points.length - 1; i++) {
      const prev = simplified[simplified.length - 1];
      const curr = points[i];
      const next = points[i + 1];

      // Calculate triangle area
      const area = Math.abs(
        (prev.x * (curr.y - next.y) +
         curr.x * (next.y - prev.y) +
         next.x * (prev.y - curr.y)) / 2
      );

      if (area > areaThreshold) {
        simplified.push(curr);
      }
    }

    simplified.push(points[points.length - 1]);
    return simplified;
  }

  /**
   * Simplify using Visvalingam-Whyatt algorithm (effective area)
   */
  static visvalingamWhyatt(points: Point[], targetCount: number): Point[] {
    if (points.length <= targetCount) {
      return points;
    }

    interface PointWithArea {
      point: Point;
      index: number;
      area: number;
      removed: boolean;
    }

    // Calculate initial areas
    const pointsWithArea: PointWithArea[] = points.map((point, index) => ({
      point,
      index,
      area: Infinity,
      removed: false
    }));

    // Calculate area for each point
    const calculateArea = (i: number): number => {
      if (i === 0 || i === points.length - 1) return Infinity;

      let prevIndex = i - 1;
      while (prevIndex >= 0 && pointsWithArea[prevIndex].removed) {
        prevIndex--;
      }

      let nextIndex = i + 1;
      while (nextIndex < points.length && pointsWithArea[nextIndex].removed) {
        nextIndex++;
      }

      if (prevIndex < 0 || nextIndex >= points.length) return Infinity;

      const prev = pointsWithArea[prevIndex].point;
      const curr = pointsWithArea[i].point;
      const next = pointsWithArea[nextIndex].point;

      return Math.abs(
        (prev.x * (curr.y - next.y) +
         curr.x * (next.y - prev.y) +
         next.x * (prev.y - curr.y)) / 2
      );
    };

    // Initial area calculation
    for (let i = 1; i < points.length - 1; i++) {
      pointsWithArea[i].area = calculateArea(i);
    }

    // Remove points until we reach target count
    const removeCount = points.length - targetCount;

    for (let removed = 0; removed < removeCount; removed++) {
      // Find point with minimum area
      let minArea = Infinity;
      let minIndex = -1;

      for (let i = 1; i < points.length - 1; i++) {
        if (!pointsWithArea[i].removed && pointsWithArea[i].area < minArea) {
          minArea = pointsWithArea[i].area;
          minIndex = i;
        }
      }

      if (minIndex === -1) break;

      // Remove point
      pointsWithArea[minIndex].removed = true;

      // Recalculate areas for neighbors
      let prevIndex = minIndex - 1;
      while (prevIndex >= 0 && pointsWithArea[prevIndex].removed) {
        prevIndex--;
      }

      let nextIndex = minIndex + 1;
      while (nextIndex < points.length && pointsWithArea[nextIndex].removed) {
        nextIndex++;
      }

      if (prevIndex >= 0) {
        pointsWithArea[prevIndex].area = calculateArea(prevIndex);
      }

      if (nextIndex < points.length) {
        pointsWithArea[nextIndex].area = calculateArea(nextIndex);
      }
    }

    // Return remaining points
    return pointsWithArea
      .filter(p => !p.removed)
      .map(p => p.point);
  }

  /**
   * Simplify using angle threshold
   */
  static simplifyByAngle(points: Point[], angleThreshold: number): Point[] {
    if (points.length <= 2) {
      return points;
    }

    const simplified: Point[] = [points[0]];

    for (let i = 1; i < points.length - 1; i++) {
      const prev = simplified[simplified.length - 1];
      const curr = points[i];
      const next = points[i + 1];

      const angle = Math.abs(GeometryEngine.angleBetween(prev, curr, next));

      // Keep point if angle is significant
      if (angle > angleThreshold) {
        simplified.push(curr);
      }
    }

    simplified.push(points[points.length - 1]);
    return simplified;
  }

  /**
   * Adaptive simplification based on local curvature
   */
  static adaptiveSimplify(points: Point[], minTolerance: number, maxTolerance: number): Point[] {
    if (points.length <= 2) {
      return points;
    }

    const simplified: Point[] = [points[0]];

    for (let i = 1; i < points.length - 1; i++) {
      const prev = simplified[simplified.length - 1];
      const curr = points[i];
      const next = points[i + 1];

      // Calculate local curvature
      const angle = Math.abs(GeometryEngine.angleBetween(prev, curr, next));
      const curvature = angle / Math.PI;

      // Adaptive tolerance based on curvature
      const tolerance = minTolerance + (maxTolerance - minTolerance) * (1 - curvature);

      const projection = GeometryEngine.projectPointOnLine(curr, prev, next);

      if (projection.distance > tolerance) {
        simplified.push(curr);
      }
    }

    simplified.push(points[points.length - 1]);
    return simplified;
  }

  /**
   * Calculate simplification error
   */
  static calculateError(original: Point[], simplified: Point[]): number {
    let totalError = 0;

    for (const point of original) {
      let minDistance = Infinity;

      // Find closest point on simplified path
      for (let i = 0; i < simplified.length - 1; i++) {
        const projection = GeometryEngine.projectPointOnLine(
          point,
          simplified[i],
          simplified[i + 1]
        );

        minDistance = Math.min(minDistance, projection.distance);
      }

      totalError += minDistance * minDistance;
    }

    return Math.sqrt(totalError / original.length);
  }
}
