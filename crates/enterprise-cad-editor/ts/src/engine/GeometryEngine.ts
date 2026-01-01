/**
 * Geometry Engine - Core Geometric Calculations
 * Comprehensive geometric operations for CAD systems
 */

import { Point, Vector2D, BoundingBox, Circle, Line, Path, BezierCurve } from '../types';

export class GeometryEngine {
  /**
   * Calculate distance between two points
   */
  static distance(p1: Point, p2: Point): number {
    const dx = p2.x - p1.x;
    const dy = p2.y - p1.y;
    return Math.sqrt(dx * dx + dy * dy);
  }

  /**
   * Calculate angle between two points (in radians)
   */
  static angle(p1: Point, p2: Point): number {
    return Math.atan2(p2.y - p1.y, p2.x - p1.x);
  }

  /**
   * Calculate angle between three points (vertex at p2)
   */
  static angleBetween(p1: Point, p2: Point, p3: Point): number {
    const angle1 = this.angle(p2, p1);
    const angle2 = this.angle(p2, p3);
    let diff = angle2 - angle1;

    // Normalize to [-π, π]
    while (diff > Math.PI) diff -= 2 * Math.PI;
    while (diff < -Math.PI) diff += 2 * Math.PI;

    return diff;
  }

  /**
   * Linear interpolation between two points
   */
  static lerp(p1: Point, p2: Point, t: number): Point {
    return {
      x: p1.x + (p2.x - p1.x) * t,
      y: p1.y + (p2.y - p1.y) * t
    };
  }

  /**
   * Calculate midpoint between two points
   */
  static midpoint(p1: Point, p2: Point): Point {
    return {
      x: (p1.x + p2.x) / 2,
      y: (p1.y + p2.y) / 2
    };
  }

  /**
   * Rotate point around origin
   */
  static rotate(point: Point, angle: number, origin: Point = { x: 0, y: 0 }): Point {
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    const dx = point.x - origin.x;
    const dy = point.y - origin.y;

    return {
      x: origin.x + dx * cos - dy * sin,
      y: origin.y + dx * sin + dy * cos
    };
  }

  /**
   * Project point onto line segment
   */
  static projectPointOnLine(point: Point, lineStart: Point, lineEnd: Point): {
    point: Point;
    t: number;
    distance: number;
  } {
    const dx = lineEnd.x - lineStart.x;
    const dy = lineEnd.y - lineStart.y;
    const lengthSq = dx * dx + dy * dy;

    if (lengthSq === 0) {
      // Line has zero length
      return {
        point: lineStart,
        t: 0,
        distance: this.distance(point, lineStart)
      };
    }

    const px = point.x - lineStart.x;
    const py = point.y - lineStart.y;
    let t = (px * dx + py * dy) / lengthSq;

    // Clamp to line segment
    t = Math.max(0, Math.min(1, t));

    const projected = {
      x: lineStart.x + t * dx,
      y: lineStart.y + t * dy
    };

    return {
      point: projected,
      t,
      distance: this.distance(point, projected)
    };
  }

  /**
   * Calculate perpendicular point on line
   */
  static perpendicular(point: Point, lineStart: Point, lineEnd: Point): Point {
    const projection = this.projectPointOnLine(point, lineStart, lineEnd);
    return projection.point;
  }

  /**
   * Check if point is on line segment
   */
  static isPointOnLine(point: Point, lineStart: Point, lineEnd: Point, tolerance: number = 0.01): boolean {
    const projection = this.projectPointOnLine(point, lineStart, lineEnd);
    return projection.distance <= tolerance && projection.t >= 0 && projection.t <= 1;
  }

  /**
   * Calculate intersection of two line segments
   */
  static lineIntersection(
    a1: Point,
    a2: Point,
    b1: Point,
    b2: Point
  ): Point | null {
    const dx1 = a2.x - a1.x;
    const dy1 = a2.y - a1.y;
    const dx2 = b2.x - b1.x;
    const dy2 = b2.y - b1.y;

    const denominator = dx1 * dy2 - dy1 * dx2;

    if (Math.abs(denominator) < 1e-10) {
      // Lines are parallel
      return null;
    }

    const dx3 = a1.x - b1.x;
    const dy3 = a1.y - b1.y;

    const t1 = (dx3 * dy2 - dy3 * dx2) / denominator;
    const t2 = (dx3 * dy1 - dy3 * dx1) / denominator;

    // Check if intersection is within both segments
    if (t1 >= 0 && t1 <= 1 && t2 >= 0 && t2 <= 1) {
      return {
        x: a1.x + t1 * dx1,
        y: a1.y + t1 * dy1
      };
    }

    return null;
  }

  /**
   * Calculate intersection of infinite lines
   */
  static lineIntersectionInfinite(
    a1: Point,
    a2: Point,
    b1: Point,
    b2: Point
  ): Point | null {
    const dx1 = a2.x - a1.x;
    const dy1 = a2.y - a1.y;
    const dx2 = b2.x - b1.x;
    const dy2 = b2.y - b1.y;

    const denominator = dx1 * dy2 - dy1 * dx2;

    if (Math.abs(denominator) < 1e-10) {
      return null;
    }

    const dx3 = a1.x - b1.x;
    const dy3 = a1.y - b1.y;

    const t1 = (dx3 * dy2 - dy3 * dx2) / denominator;

    return {
      x: a1.x + t1 * dx1,
      y: a1.y + t1 * dy1
    };
  }

  /**
   * Calculate circle-line intersection
   */
  static circleLineIntersection(
    circle: Circle,
    lineStart: Point,
    lineEnd: Point
  ): Point[] {
    const dx = lineEnd.x - lineStart.x;
    const dy = lineEnd.y - lineStart.y;
    const fx = lineStart.x - circle.center.x;
    const fy = lineStart.y - circle.center.y;

    const a = dx * dx + dy * dy;
    const b = 2 * (fx * dx + fy * dy);
    const c = fx * fx + fy * fy - circle.radius * circle.radius;

    const discriminant = b * b - 4 * a * c;

    if (discriminant < 0) {
      return []; // No intersection
    }

    const sqrtDisc = Math.sqrt(discriminant);
    const t1 = (-b - sqrtDisc) / (2 * a);
    const t2 = (-b + sqrtDisc) / (2 * a);

    const intersections: Point[] = [];

    if (t1 >= 0 && t1 <= 1) {
      intersections.push({
        x: lineStart.x + t1 * dx,
        y: lineStart.y + t1 * dy
      });
    }

    if (t2 >= 0 && t2 <= 1 && Math.abs(t1 - t2) > 1e-10) {
      intersections.push({
        x: lineStart.x + t2 * dx,
        y: lineStart.y + t2 * dy
      });
    }

    return intersections;
  }

  /**
   * Calculate circle-circle intersection
   */
  static circleCircleIntersection(c1: Circle, c2: Circle): Point[] {
    const d = this.distance(c1.center, c2.center);

    if (d > c1.radius + c2.radius || d < Math.abs(c1.radius - c2.radius) || d === 0) {
      return []; // No intersection
    }

    const a = (c1.radius * c1.radius - c2.radius * c2.radius + d * d) / (2 * d);
    const h = Math.sqrt(c1.radius * c1.radius - a * a);

    const px = c1.center.x + (a / d) * (c2.center.x - c1.center.x);
    const py = c1.center.y + (a / d) * (c2.center.y - c1.center.y);

    const dx = (h / d) * (c2.center.y - c1.center.y);
    const dy = (h / d) * (c2.center.x - c1.center.x);

    return [
      { x: px + dx, y: py - dy },
      { x: px - dx, y: py + dy }
    ];
  }

  /**
   * Calculate bounding box for points
   */
  static getBounds(points: Point[]): BoundingBox {
    if (points.length === 0) {
      return { minX: 0, minY: 0, maxX: 0, maxY: 0, width: 0, height: 0, center: { x: 0, y: 0 } };
    }

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    for (const point of points) {
      minX = Math.min(minX, point.x);
      minY = Math.min(minY, point.y);
      maxX = Math.max(maxX, point.x);
      maxY = Math.max(maxY, point.y);
    }

    return {
      minX,
      minY,
      maxX,
      maxY,
      width: maxX - minX,
      height: maxY - minY,
      center: {
        x: (minX + maxX) / 2,
        y: (minY + maxY) / 2
      }
    };
  }

  /**
   * Check if point is inside bounding box
   */
  static pointInBounds(point: Point, bounds: BoundingBox): boolean {
    return (
      point.x >= bounds.minX &&
      point.x <= bounds.maxX &&
      point.y >= bounds.minY &&
      point.y <= bounds.maxY
    );
  }

  /**
   * Check if bounding boxes intersect
   */
  static boundsIntersect(a: BoundingBox, b: BoundingBox): boolean {
    return !(
      a.maxX < b.minX ||
      a.minX > b.maxX ||
      a.maxY < b.minY ||
      a.minY > b.maxY
    );
  }

  /**
   * Check if point is inside polygon (ray casting algorithm)
   */
  static pointInPolygon(point: Point, polygon: Point[]): boolean {
    let inside = false;
    const n = polygon.length;

    for (let i = 0, j = n - 1; i < n; j = i++) {
      const xi = polygon[i].x;
      const yi = polygon[i].y;
      const xj = polygon[j].x;
      const yj = polygon[j].y;

      const intersect =
        yi > point.y !== yj > point.y &&
        point.x < ((xj - xi) * (point.y - yi)) / (yj - yi) + xi;

      if (intersect) inside = !inside;
    }

    return inside;
  }

  /**
   * Calculate polygon area (signed)
   */
  static polygonArea(points: Point[]): number {
    let area = 0;
    const n = points.length;

    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      area += points[i].x * points[j].y;
      area -= points[j].x * points[i].y;
    }

    return area / 2;
  }

  /**
   * Calculate polygon centroid
   */
  static polygonCentroid(points: Point[]): Point {
    let cx = 0;
    let cy = 0;
    let area = 0;
    const n = points.length;

    for (let i = 0; i < n; i++) {
      const j = (i + 1) % n;
      const cross = points[i].x * points[j].y - points[j].x * points[i].y;
      cx += (points[i].x + points[j].x) * cross;
      cy += (points[i].y + points[j].y) * cross;
      area += cross;
    }

    area /= 2;
    const factor = 1 / (6 * area);

    return {
      x: cx * factor,
      y: cy * factor
    };
  }

  /**
   * Offset polygon (positive = outward, negative = inward)
   */
  static offsetPolygon(points: Point[], distance: number): Point[] {
    const n = points.length;
    const offset: Point[] = [];

    for (let i = 0; i < n; i++) {
      const prev = points[(i - 1 + n) % n];
      const curr = points[i];
      const next = points[(i + 1) % n];

      // Calculate normals
      const n1x = -(curr.y - prev.y);
      const n1y = curr.x - prev.x;
      const len1 = Math.sqrt(n1x * n1x + n1y * n1y);

      const n2x = -(next.y - curr.y);
      const n2y = next.x - curr.x;
      const len2 = Math.sqrt(n2x * n2x + n2y * n2y);

      // Average normal
      const nx = (n1x / len1 + n2x / len2) / 2;
      const ny = (n1y / len1 + n2y / len2) / 2;
      const len = Math.sqrt(nx * nx + ny * ny);

      offset.push({
        x: curr.x + (nx / len) * distance,
        y: curr.y + (ny / len) * distance
      });
    }

    return offset;
  }

  /**
   * Create arc points
   */
  static createArc(
    center: Point,
    radius: number,
    startAngle: number,
    endAngle: number,
    segments: number = 32
  ): Point[] {
    const points: Point[] = [];
    const angleDiff = endAngle - startAngle;

    for (let i = 0; i <= segments; i++) {
      const t = i / segments;
      const angle = startAngle + angleDiff * t;
      points.push({
        x: center.x + Math.cos(angle) * radius,
        y: center.y + Math.sin(angle) * radius
      });
    }

    return points;
  }
}
