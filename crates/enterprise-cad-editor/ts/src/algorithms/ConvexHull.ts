/**
 * Convex Hull - Hull Generation Algorithms
 * Generate convex hulls using various algorithms
 */

import { Point } from '../types';

export class ConvexHull {
  /**
   * Graham scan algorithm
   */
  static grahamScan(points: Point[]): Point[] {
    if (points.length < 3) return points;

    // Find lowest point (break ties by x-coordinate)
    let lowest = points[0];
    let lowestIndex = 0;

    for (let i = 1; i < points.length; i++) {
      if (points[i].y < lowest.y || (points[i].y === lowest.y && points[i].x < lowest.x)) {
        lowest = points[i];
        lowestIndex = i;
      }
    }

    // Sort points by polar angle with respect to lowest point
    const sorted = [...points];
    const anchor = sorted.splice(lowestIndex, 1)[0];

    sorted.sort((a, b) => {
      const angleA = Math.atan2(a.y - anchor.y, a.x - anchor.x);
      const angleB = Math.atan2(b.y - anchor.y, b.x - anchor.x);

      if (angleA !== angleB) {
        return angleA - angleB;
      }

      // If angles are equal, sort by distance
      const distA = (a.x - anchor.x) ** 2 + (a.y - anchor.y) ** 2;
      const distB = (b.x - anchor.x) ** 2 + (b.y - anchor.y) ** 2;
      return distA - distB;
    });

    // Build hull
    const hull: Point[] = [anchor, sorted[0]];

    for (let i = 1; i < sorted.length; i++) {
      let top = hull[hull.length - 1];
      let second = hull[hull.length - 2];

      // Remove points that make clockwise turn
      while (
        hull.length > 1 &&
        this.crossProduct(second, top, sorted[i]) <= 0
      ) {
        hull.pop();
        top = hull[hull.length - 1];
        second = hull[hull.length - 2];
      }

      hull.push(sorted[i]);
    }

    return hull;
  }

  /**
   * Jarvis march (gift wrapping) algorithm
   */
  static jarvisMarch(points: Point[]): Point[] {
    if (points.length < 3) return points;

    const hull: Point[] = [];

    // Find leftmost point
    let leftmost = points[0];
    for (const point of points) {
      if (point.x < leftmost.x) {
        leftmost = point;
      }
    }

    let current = leftmost;

    do {
      hull.push(current);
      let next = points[0];

      for (const point of points) {
        if (
          point === current ||
          this.crossProduct(current, next, point) > 0
        ) {
          next = point;
        }
      }

      current = next;
    } while (current !== leftmost);

    return hull;
  }

  /**
   * Quick hull algorithm
   */
  static quickHull(points: Point[]): Point[] {
    if (points.length < 3) return points;

    // Find leftmost and rightmost points
    let minPoint = points[0];
    let maxPoint = points[0];

    for (const point of points) {
      if (point.x < minPoint.x) minPoint = point;
      if (point.x > maxPoint.x) maxPoint = point;
    }

    const hull: Point[] = [];

    // Divide points into two sets
    const upper: Point[] = [];
    const lower: Point[] = [];

    for (const point of points) {
      if (this.crossProduct(minPoint, maxPoint, point) > 0) {
        upper.push(point);
      } else if (this.crossProduct(minPoint, maxPoint, point) < 0) {
        lower.push(point);
      }
    }

    // Find hull points recursively
    this.findHullPoints(minPoint, maxPoint, upper, hull);
    hull.push(maxPoint);
    this.findHullPoints(maxPoint, minPoint, lower, hull);

    return hull;
  }

  /**
   * Recursive helper for quick hull
   */
  private static findHullPoints(p1: Point, p2: Point, points: Point[], hull: Point[]): void {
    if (points.length === 0) return;

    // Find farthest point
    let farthest = points[0];
    let maxDistance = this.distanceToLine(farthest, p1, p2);

    for (const point of points) {
      const distance = this.distanceToLine(point, p1, p2);
      if (distance > maxDistance) {
        maxDistance = distance;
        farthest = point;
      }
    }

    // Divide remaining points
    const left: Point[] = [];
    const right: Point[] = [];

    for (const point of points) {
      if (this.crossProduct(p1, farthest, point) > 0) {
        left.push(point);
      } else if (this.crossProduct(farthest, p2, point) > 0) {
        right.push(point);
      }
    }

    this.findHullPoints(p1, farthest, left, hull);
    hull.push(farthest);
    this.findHullPoints(farthest, p2, right, hull);
  }

  /**
   * Calculate cross product for orientation
   */
  private static crossProduct(a: Point, b: Point, c: Point): number {
    return (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
  }

  /**
   * Calculate distance from point to line
   */
  private static distanceToLine(point: Point, lineStart: Point, lineEnd: Point): number {
    const numerator = Math.abs(
      (lineEnd.y - lineStart.y) * point.x -
      (lineEnd.x - lineStart.x) * point.y +
      lineEnd.x * lineStart.y -
      lineEnd.y * lineStart.x
    );

    const denominator = Math.sqrt(
      (lineEnd.y - lineStart.y) ** 2 + (lineEnd.x - lineStart.x) ** 2
    );

    return numerator / denominator;
  }

  /**
   * Check if a point is inside the convex hull
   */
  static isPointInside(point: Point, hull: Point[]): boolean {
    if (hull.length < 3) return false;

    const firstCross = this.crossProduct(hull[0], hull[1], point);

    for (let i = 1; i < hull.length; i++) {
      const next = (i + 1) % hull.length;
      const cross = this.crossProduct(hull[i], hull[next], point);

      if (Math.sign(cross) !== Math.sign(firstCross)) {
        return false;
      }
    }

    return true;
  }

  /**
   * Calculate area of convex hull
   */
  static area(hull: Point[]): number {
    if (hull.length < 3) return 0;

    let area = 0;

    for (let i = 0; i < hull.length; i++) {
      const next = (i + 1) % hull.length;
      area += hull[i].x * hull[next].y - hull[next].x * hull[i].y;
    }

    return Math.abs(area) / 2;
  }

  /**
   * Calculate perimeter of convex hull
   */
  static perimeter(hull: Point[]): number {
    if (hull.length < 2) return 0;

    let perimeter = 0;

    for (let i = 0; i < hull.length; i++) {
      const next = (i + 1) % hull.length;
      const dx = hull[next].x - hull[i].x;
      const dy = hull[next].y - hull[i].y;
      perimeter += Math.sqrt(dx * dx + dy * dy);
    }

    return perimeter;
  }
}
