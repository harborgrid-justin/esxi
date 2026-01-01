/**
 * Bezier Math - Bezier Curve Calculations
 * Comprehensive Bezier curve mathematics
 */

import { Point, BezierCurve, BoundingBox } from '../types';
import { GeometryEngine } from '../engine/GeometryEngine';

export class BezierMath {
  /**
   * Evaluate quadratic Bezier at parameter t
   */
  static evaluateQuadratic(p0: Point, p1: Point, p2: Point, t: number): Point {
    const mt = 1 - t;
    return {
      x: mt * mt * p0.x + 2 * mt * t * p1.x + t * t * p2.x,
      y: mt * mt * p0.y + 2 * mt * t * p1.y + t * t * p2.y
    };
  }

  /**
   * Evaluate cubic Bezier at parameter t
   */
  static evaluateCubic(p0: Point, p1: Point, p2: Point, p3: Point, t: number): Point {
    const mt = 1 - t;
    const mt2 = mt * mt;
    const mt3 = mt2 * mt;
    const t2 = t * t;
    const t3 = t2 * t;

    return {
      x: mt3 * p0.x + 3 * mt2 * t * p1.x + 3 * mt * t2 * p2.x + t3 * p3.x,
      y: mt3 * p0.y + 3 * mt2 * t * p1.y + 3 * mt * t2 * p2.y + t3 * p3.y
    };
  }

  /**
   * Get derivative of quadratic Bezier at parameter t
   */
  static derivativeQuadratic(p0: Point, p1: Point, p2: Point, t: number): Point {
    const mt = 1 - t;
    return {
      x: 2 * mt * (p1.x - p0.x) + 2 * t * (p2.x - p1.x),
      y: 2 * mt * (p1.y - p0.y) + 2 * t * (p2.y - p1.y)
    };
  }

  /**
   * Get derivative of cubic Bezier at parameter t
   */
  static derivativeCubic(p0: Point, p1: Point, p2: Point, p3: Point, t: number): Point {
    const mt = 1 - t;
    const mt2 = mt * mt;
    const t2 = t * t;

    return {
      x: 3 * mt2 * (p1.x - p0.x) + 6 * mt * t * (p2.x - p1.x) + 3 * t2 * (p3.x - p2.x),
      y: 3 * mt2 * (p1.y - p0.y) + 6 * mt * t * (p2.y - p1.y) + 3 * t2 * (p3.y - p2.y)
    };
  }

  /**
   * Split cubic Bezier at parameter t
   */
  static splitCubic(
    p0: Point,
    p1: Point,
    p2: Point,
    p3: Point,
    t: number
  ): [BezierCurve, BezierCurve] {
    // De Casteljau's algorithm
    const p01 = GeometryEngine.lerp(p0, p1, t);
    const p12 = GeometryEngine.lerp(p1, p2, t);
    const p23 = GeometryEngine.lerp(p2, p3, t);

    const p012 = GeometryEngine.lerp(p01, p12, t);
    const p123 = GeometryEngine.lerp(p12, p23, t);

    const p0123 = GeometryEngine.lerp(p012, p123, t);

    const left: BezierCurve = {
      p0,
      p1: p01,
      p2: p012,
      p3: p0123,
      order: 3,
      evaluate: (t: number) => BezierMath.evaluateCubic(p0, p01, p012, p0123, t),
      derivative: (t: number) => BezierMath.derivativeCubic(p0, p01, p012, p0123, t),
      split: (t: number) => BezierMath.splitCubic(p0, p01, p012, p0123, t),
      getBounds: () => BezierMath.getBoundsCubic(p0, p01, p012, p0123),
      getLength: () => BezierMath.getLengthCubic(p0, p01, p012, p0123)
    };

    const right: BezierCurve = {
      p0: p0123,
      p1: p123,
      p2: p23,
      p3,
      order: 3,
      evaluate: (t: number) => BezierMath.evaluateCubic(p0123, p123, p23, p3, t),
      derivative: (t: number) => BezierMath.derivativeCubic(p0123, p123, p23, p3, t),
      split: (t: number) => BezierMath.splitCubic(p0123, p123, p23, p3, t),
      getBounds: () => BezierMath.getBoundsCubic(p0123, p123, p23, p3),
      getLength: () => BezierMath.getLengthCubic(p0123, p123, p23, p3)
    };

    return [left, right];
  }

  /**
   * Get bounding box for cubic Bezier
   */
  static getBoundsCubic(p0: Point, p1: Point, p2: Point, p3: Point): BoundingBox {
    const points = [p0, p3];

    // Find extrema in x
    const ax = 3 * (-p0.x + 3 * p1.x - 3 * p2.x + p3.x);
    const bx = 6 * (p0.x - 2 * p1.x + p2.x);
    const cx = 3 * (p1.x - p0.x);

    const tx = this.solveQuadratic(ax, bx, cx);
    for (const t of tx) {
      if (t >= 0 && t <= 1) {
        points.push(this.evaluateCubic(p0, p1, p2, p3, t));
      }
    }

    // Find extrema in y
    const ay = 3 * (-p0.y + 3 * p1.y - 3 * p2.y + p3.y);
    const by = 6 * (p0.y - 2 * p1.y + p2.y);
    const cy = 3 * (p1.y - p0.y);

    const ty = this.solveQuadratic(ay, by, cy);
    for (const t of ty) {
      if (t >= 0 && t <= 1) {
        points.push(this.evaluateCubic(p0, p1, p2, p3, t));
      }
    }

    return GeometryEngine.getBounds(points);
  }

  /**
   * Solve quadratic equation ax^2 + bx + c = 0
   */
  private static solveQuadratic(a: number, b: number, c: number): number[] {
    if (Math.abs(a) < 1e-10) {
      // Linear equation
      if (Math.abs(b) < 1e-10) return [];
      return [-c / b];
    }

    const discriminant = b * b - 4 * a * c;
    if (discriminant < 0) return [];

    const sqrtDisc = Math.sqrt(discriminant);
    return [(-b - sqrtDisc) / (2 * a), (-b + sqrtDisc) / (2 * a)];
  }

  /**
   * Calculate length of cubic Bezier (numerical integration)
   */
  static getLengthCubic(p0: Point, p1: Point, p2: Point, p3: Point, segments: number = 20): number {
    let length = 0;
    let prevPoint = p0;

    for (let i = 1; i <= segments; i++) {
      const t = i / segments;
      const point = this.evaluateCubic(p0, p1, p2, p3, t);
      length += GeometryEngine.distance(prevPoint, point);
      prevPoint = point;
    }

    return length;
  }

  /**
   * Find point at specific length along curve
   */
  static pointAtLength(p0: Point, p1: Point, p2: Point, p3: Point, targetLength: number): Point {
    const totalLength = this.getLengthCubic(p0, p1, p2, p3);
    if (targetLength >= totalLength) return p3;
    if (targetLength <= 0) return p0;

    // Binary search for parameter t
    let tMin = 0;
    let tMax = 1;
    let t = targetLength / totalLength; // Initial guess

    for (let i = 0; i < 20; i++) {
      const length = this.getLengthCubic(p0, p1, p2, p3);
      const ratio = targetLength / length;

      if (Math.abs(ratio - 1) < 0.001) break;

      if (ratio < 1) {
        tMax = t;
      } else {
        tMin = t;
      }

      t = (tMin + tMax) / 2;
    }

    return this.evaluateCubic(p0, p1, p2, p3, t);
  }

  /**
   * Convert cubic Bezier to polyline
   */
  static toPolyline(p0: Point, p1: Point, p2: Point, p3: Point, segments: number = 20): Point[] {
    const points: Point[] = [];

    for (let i = 0; i <= segments; i++) {
      const t = i / segments;
      points.push(this.evaluateCubic(p0, p1, p2, p3, t));
    }

    return points;
  }

  /**
   * Calculate curvature at parameter t
   */
  static curvature(p0: Point, p1: Point, p2: Point, p3: Point, t: number): number {
    const d1 = this.derivativeCubic(p0, p1, p2, p3, t);

    // Second derivative
    const mt = 1 - t;
    const d2 = {
      x: 6 * mt * (p2.x - 2 * p1.x + p0.x) + 6 * t * (p3.x - 2 * p2.x + p1.x),
      y: 6 * mt * (p2.y - 2 * p1.y + p0.y) + 6 * t * (p3.y - 2 * p2.y + p1.y)
    };

    const cross = d1.x * d2.y - d1.y * d2.x;
    const speed = Math.sqrt(d1.x * d1.x + d1.y * d1.y);

    return Math.abs(cross) / Math.pow(speed, 3);
  }

  /**
   * Elevate quadratic Bezier to cubic
   */
  static elevateToCubic(p0: Point, p1: Point, p2: Point): [Point, Point, Point, Point] {
    const cp1 = {
      x: p0.x + (2 / 3) * (p1.x - p0.x),
      y: p0.y + (2 / 3) * (p1.y - p0.y)
    };

    const cp2 = {
      x: p2.x + (2 / 3) * (p1.x - p2.x),
      y: p2.y + (2 / 3) * (p1.y - p2.y)
    };

    return [p0, cp1, cp2, p2];
  }
}
