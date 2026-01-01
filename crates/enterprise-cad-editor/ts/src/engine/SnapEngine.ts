/**
 * Snap Engine - Smart Snapping for CAD Operations
 * Provides intelligent snapping to grid, points, and geometric features
 */

import { Point, SnapPoint, SnapType, SnapSettings, Shape, ShapeType } from '../types';
import { GeometryEngine } from './GeometryEngine';

export class SnapEngine {
  private settings: SnapSettings;
  private shapes: Map<string, Shape>;

  constructor(settings?: Partial<SnapSettings>) {
    this.settings = {
      enabled: settings?.enabled ?? true,
      threshold: settings?.threshold ?? 10,
      types: settings?.types ?? new Set([
        SnapType.Grid,
        SnapType.Point,
        SnapType.Midpoint,
        SnapType.Center,
        SnapType.Endpoint,
        SnapType.Intersection
      ]),
      gridSize: settings?.gridSize ?? 10,
      angleSnap: settings?.angleSnap ?? 15
    };

    this.shapes = new Map();
  }

  /**
   * Update snap settings
   */
  updateSettings(settings: Partial<SnapSettings>): void {
    Object.assign(this.settings, settings);
  }

  /**
   * Register shapes for snapping
   */
  registerShapes(shapes: Shape[]): void {
    this.shapes.clear();
    for (const shape of shapes) {
      this.shapes.set(shape.id, shape);
    }
  }

  /**
   * Find snap points near a position
   */
  findSnapPoints(position: Point, viewport: { zoom: number }): SnapPoint[] {
    if (!this.settings.enabled) {
      return [];
    }

    const snapPoints: SnapPoint[] = [];
    const worldThreshold = this.settings.threshold / viewport.zoom;

    // Grid snapping
    if (this.settings.types.has(SnapType.Grid) && this.settings.gridSize) {
      const gridSnap = this.snapToGrid(position);
      if (GeometryEngine.distance(position, gridSnap.point) <= worldThreshold) {
        snapPoints.push(gridSnap);
      }
    }

    // Shape-based snapping
    for (const shape of this.shapes.values()) {
      if (!shape.visible) continue;

      // Endpoint snapping
      if (this.settings.types.has(SnapType.Endpoint)) {
        snapPoints.push(...this.findEndpoints(shape, position, worldThreshold));
      }

      // Midpoint snapping
      if (this.settings.types.has(SnapType.Midpoint)) {
        snapPoints.push(...this.findMidpoints(shape, position, worldThreshold));
      }

      // Center snapping
      if (this.settings.types.has(SnapType.Center)) {
        const centerSnap = this.findCenter(shape, position, worldThreshold);
        if (centerSnap) {
          snapPoints.push(centerSnap);
        }
      }

      // Quadrant snapping (for circles/ellipses)
      if (this.settings.types.has(SnapType.Quadrant)) {
        snapPoints.push(...this.findQuadrants(shape, position, worldThreshold));
      }
    }

    // Intersection snapping
    if (this.settings.types.has(SnapType.Intersection)) {
      snapPoints.push(...this.findIntersections(position, worldThreshold));
    }

    // Sort by distance
    snapPoints.sort((a, b) => a.distance - b.distance);

    return snapPoints;
  }

  /**
   * Get best snap point
   */
  snap(position: Point, viewport: { zoom: number }): SnapPoint | null {
    const snapPoints = this.findSnapPoints(position, viewport);
    return snapPoints.length > 0 ? snapPoints[0] : null;
  }

  /**
   * Snap to grid
   */
  private snapToGrid(position: Point): SnapPoint {
    const gridSize = this.settings.gridSize || 10;
    const snapped = {
      x: Math.round(position.x / gridSize) * gridSize,
      y: Math.round(position.y / gridSize) * gridSize
    };

    return {
      point: snapped,
      type: SnapType.Grid,
      distance: GeometryEngine.distance(position, snapped)
    };
  }

  /**
   * Snap to angle
   */
  snapAngle(basePoint: Point, currentPoint: Point): Point {
    if (!this.settings.angleSnap) {
      return currentPoint;
    }

    const angle = GeometryEngine.angle(basePoint, currentPoint);
    const distance = GeometryEngine.distance(basePoint, currentPoint);

    const snapAngleRad = (this.settings.angleSnap * Math.PI) / 180;
    const snappedAngle = Math.round(angle / snapAngleRad) * snapAngleRad;

    return {
      x: basePoint.x + Math.cos(snappedAngle) * distance,
      y: basePoint.y + Math.sin(snappedAngle) * distance
    };
  }

  /**
   * Find endpoints of a shape
   */
  private findEndpoints(shape: Shape, position: Point, threshold: number): SnapPoint[] {
    const points: SnapPoint[] = [];

    switch (shape.type) {
      case ShapeType.Line: {
        const lineShape = shape as any;
        const endpoints = [lineShape.start, lineShape.end];

        for (const endpoint of endpoints) {
          const distance = GeometryEngine.distance(position, endpoint);
          if (distance <= threshold) {
            points.push({
              point: endpoint,
              type: SnapType.Endpoint,
              targetId: shape.id,
              distance
            });
          }
        }
        break;
      }

      case ShapeType.Polygon:
      case ShapeType.Polyline: {
        const polyShape = shape as any;
        for (const point of polyShape.points) {
          const distance = GeometryEngine.distance(position, point);
          if (distance <= threshold) {
            points.push({
              point,
              type: SnapType.Endpoint,
              targetId: shape.id,
              distance
            });
          }
        }
        break;
      }
    }

    return points;
  }

  /**
   * Find midpoints of a shape
   */
  private findMidpoints(shape: Shape, position: Point, threshold: number): SnapPoint[] {
    const points: SnapPoint[] = [];

    switch (shape.type) {
      case ShapeType.Line: {
        const lineShape = shape as any;
        const midpoint = GeometryEngine.midpoint(lineShape.start, lineShape.end);
        const distance = GeometryEngine.distance(position, midpoint);

        if (distance <= threshold) {
          points.push({
            point: midpoint,
            type: SnapType.Midpoint,
            targetId: shape.id,
            distance
          });
        }
        break;
      }

      case ShapeType.Polygon:
      case ShapeType.Polyline: {
        const polyShape = shape as any;
        const n = polyShape.points.length;

        for (let i = 0; i < n - 1; i++) {
          const midpoint = GeometryEngine.midpoint(
            polyShape.points[i],
            polyShape.points[i + 1]
          );
          const distance = GeometryEngine.distance(position, midpoint);

          if (distance <= threshold) {
            points.push({
              point: midpoint,
              type: SnapType.Midpoint,
              targetId: shape.id,
              distance
            });
          }
        }
        break;
      }
    }

    return points;
  }

  /**
   * Find center of a shape
   */
  private findCenter(shape: Shape, position: Point, threshold: number): SnapPoint | null {
    let center: Point | null = null;

    switch (shape.type) {
      case ShapeType.Circle: {
        const circleShape = shape as any;
        center = circleShape.circle.center;
        break;
      }

      case ShapeType.Ellipse: {
        const ellipseShape = shape as any;
        center = ellipseShape.center;
        break;
      }

      case ShapeType.Rectangle: {
        const rectShape = shape as any;
        center = {
          x: rectShape.rect.x + rectShape.rect.width / 2,
          y: rectShape.rect.y + rectShape.rect.height / 2
        };
        break;
      }

      default: {
        const bounds = shape.getBounds();
        center = bounds.center;
      }
    }

    if (center) {
      const distance = GeometryEngine.distance(position, center);
      if (distance <= threshold) {
        return {
          point: center,
          type: SnapType.Center,
          targetId: shape.id,
          distance
        };
      }
    }

    return null;
  }

  /**
   * Find quadrant points (for circles/ellipses)
   */
  private findQuadrants(shape: Shape, position: Point, threshold: number): SnapPoint[] {
    const points: SnapPoint[] = [];

    if (shape.type === ShapeType.Circle) {
      const circleShape = shape as any;
      const { center, radius } = circleShape.circle;

      const quadrants = [
        { x: center.x + radius, y: center.y }, // Right
        { x: center.x, y: center.y + radius }, // Bottom
        { x: center.x - radius, y: center.y }, // Left
        { x: center.x, y: center.y - radius }  // Top
      ];

      for (const quadrant of quadrants) {
        const distance = GeometryEngine.distance(position, quadrant);
        if (distance <= threshold) {
          points.push({
            point: quadrant,
            type: SnapType.Quadrant,
            targetId: shape.id,
            distance
          });
        }
      }
    }

    return points;
  }

  /**
   * Find intersections between shapes
   */
  private findIntersections(position: Point, threshold: number): SnapPoint[] {
    const points: SnapPoint[] = [];
    const shapes = Array.from(this.shapes.values()).filter(s => s.visible);

    // Check all shape pairs for intersections
    for (let i = 0; i < shapes.length; i++) {
      for (let j = i + 1; j < shapes.length; j++) {
        const intersections = this.findShapeIntersections(shapes[i], shapes[j]);

        for (const intersection of intersections) {
          const distance = GeometryEngine.distance(position, intersection);
          if (distance <= threshold) {
            points.push({
              point: intersection,
              type: SnapType.Intersection,
              targetId: `${shapes[i].id}:${shapes[j].id}`,
              distance
            });
          }
        }
      }
    }

    return points;
  }

  /**
   * Find intersections between two shapes
   */
  private findShapeIntersections(shape1: Shape, shape2: Shape): Point[] {
    // Simplified intersection detection
    // Full implementation would handle all shape type combinations

    if (shape1.type === ShapeType.Line && shape2.type === ShapeType.Line) {
      const line1 = shape1 as any;
      const line2 = shape2 as any;

      const intersection = GeometryEngine.lineIntersection(
        line1.start,
        line1.end,
        line2.start,
        line2.end
      );

      return intersection ? [intersection] : [];
    }

    if (shape1.type === ShapeType.Circle && shape2.type === ShapeType.Line) {
      const circle = shape1 as any;
      const line = shape2 as any;

      return GeometryEngine.circleLineIntersection(
        circle.circle,
        line.start,
        line.end
      );
    }

    if (shape1.type === ShapeType.Circle && shape2.type === ShapeType.Circle) {
      const circle1 = shape1 as any;
      const circle2 = shape2 as any;

      return GeometryEngine.circleCircleIntersection(
        circle1.circle,
        circle2.circle
      );
    }

    return [];
  }

  /**
   * Clear registered shapes
   */
  clear(): void {
    this.shapes.clear();
  }
}
