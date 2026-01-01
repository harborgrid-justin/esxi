/**
 * Overlay Analysis
 * Performs geometric overlay operations (union, intersection, difference)
 */

import {
  Geometry,
  Polygon,
  Position,
  GeometryError,
} from '../types';
import { GeometryFactory } from './GeometryFactory';
import { TopologyEngine } from './TopologyEngine';

export class OverlayAnalysis {
  /**
   * Union of two polygons
   */
  static union(poly1: Polygon, poly2: Polygon): Polygon {
    if (!TopologyEngine.intersects(poly1, poly2)) {
      // If they don't intersect, return a multi-polygon or combined boundary
      return this.combineNonIntersecting(poly1, poly2);
    }

    // Merge overlapping polygons
    return this.mergePolygons(poly1, poly2);
  }

  /**
   * Intersection of two polygons
   */
  static intersection(poly1: Polygon, poly2: Polygon): Polygon | null {
    if (!TopologyEngine.intersects(poly1, poly2)) {
      return null;
    }

    // Find overlapping area
    const intersectingPoints = this.findIntersectingRegion(poly1, poly2);

    if (intersectingPoints.length < 3) {
      return null;
    }

    // Create convex hull of intersecting points
    const hull = this.convexHull(intersectingPoints);

    if (hull.length < 3) {
      return null;
    }

    return GeometryFactory.createPolygon([hull]);
  }

  /**
   * Difference: poly1 - poly2
   */
  static difference(poly1: Polygon, poly2: Polygon): Polygon | null {
    if (!TopologyEngine.intersects(poly1, poly2)) {
      return poly1; // No overlap, return original
    }

    // Subtract poly2 from poly1
    return this.subtractPolygon(poly1, poly2);
  }

  /**
   * Symmetric difference (XOR)
   */
  static symmetricDifference(poly1: Polygon, poly2: Polygon): Polygon {
    const diff1 = this.difference(poly1, poly2);
    const diff2 = this.difference(poly2, poly1);

    if (!diff1) return diff2 || poly2;
    if (!diff2) return diff1;

    return this.combineNonIntersecting(diff1, diff2);
  }

  /**
   * Clip polygon by bounds
   */
  static clip(polygon: Polygon, clipBounds: Polygon): Polygon | null {
    return this.intersection(polygon, clipBounds);
  }

  /**
   * Merge multiple polygons
   */
  static mergePolygons(...polygons: Polygon[]): Polygon {
    if (polygons.length === 0) {
      throw new GeometryError('Cannot merge zero polygons');
    }

    if (polygons.length === 1) {
      return polygons[0];
    }

    let result = polygons[0];
    for (let i = 1; i < polygons.length; i++) {
      result = this.union(result, polygons[i]);
    }

    return result;
  }

  /**
   * Dissolve polygons with common boundaries
   */
  static dissolve(polygons: Polygon[]): Polygon[] {
    const dissolved: Polygon[] = [];
    const processed = new Set<number>();

    for (let i = 0; i < polygons.length; i++) {
      if (processed.has(i)) continue;

      let current = polygons[i];
      processed.add(i);

      // Find all intersecting polygons
      for (let j = i + 1; j < polygons.length; j++) {
        if (processed.has(j)) continue;

        if (TopologyEngine.intersects(current, polygons[j])) {
          current = this.union(current, polygons[j]);
          processed.add(j);
        }
      }

      dissolved.push(current);
    }

    return dissolved;
  }

  /**
   * Combine non-intersecting polygons into single polygon
   */
  private static combineNonIntersecting(
    poly1: Polygon,
    poly2: Polygon
  ): Polygon {
    // Create a polygon that encompasses both
    const allPoints = [
      ...poly1.coordinates[0],
      ...poly2.coordinates[0],
    ];

    // Remove duplicate points
    const uniquePoints = this.removeDuplicatePoints(allPoints);

    // Create convex hull
    const hull = this.convexHull(uniquePoints);

    return GeometryFactory.createPolygon([hull]);
  }

  /**
   * Find intersecting region between two polygons
   */
  private static findIntersectingRegion(
    poly1: Polygon,
    poly2: Polygon
  ): Position[] {
    const points: Position[] = [];

    // Add vertices from poly1 that are inside poly2
    for (const pos of poly1.coordinates[0]) {
      if (TopologyEngine.pointInPolygon(pos, poly2)) {
        points.push(pos);
      }
    }

    // Add vertices from poly2 that are inside poly1
    for (const pos of poly2.coordinates[0]) {
      if (TopologyEngine.pointInPolygon(pos, poly1)) {
        points.push(pos);
      }
    }

    // Find edge intersections
    const edges1 = this.getEdges(poly1.coordinates[0]);
    const edges2 = this.getEdges(poly2.coordinates[0]);

    for (const edge1 of edges1) {
      for (const edge2 of edges2) {
        const intersection = TopologyEngine.lineIntersection(
          edge1[0],
          edge1[1],
          edge2[0],
          edge2[1]
        );
        if (intersection) {
          points.push(intersection);
        }
      }
    }

    return this.removeDuplicatePoints(points);
  }

  /**
   * Subtract poly2 from poly1
   */
  private static subtractPolygon(
    poly1: Polygon,
    poly2: Polygon
  ): Polygon | null {
    // Get points from poly1 that are not in poly2
    const remainingPoints = poly1.coordinates[0].filter(
      (pos) => !TopologyEngine.pointInPolygon(pos, poly2)
    );

    if (remainingPoints.length < 3) {
      return null;
    }

    // Create convex hull of remaining points
    const hull = this.convexHull(remainingPoints);

    if (hull.length < 3) {
      return null;
    }

    return GeometryFactory.createPolygon([hull]);
  }

  /**
   * Calculate convex hull using Graham scan
   */
  static convexHull(points: Position[]): Position[] {
    if (points.length < 3) {
      return points;
    }

    // Find the bottom-most point (or left most if tied)
    let bottom = 0;
    for (let i = 1; i < points.length; i++) {
      if (
        points[i][1] < points[bottom][1] ||
        (points[i][1] === points[bottom][1] && points[i][0] < points[bottom][0])
      ) {
        bottom = i;
      }
    }

    // Swap bottom point to first position
    [points[0], points[bottom]] = [points[bottom], points[0]];

    const p0 = points[0];

    // Sort points by polar angle with respect to p0
    const sorted = points.slice(1).sort((a, b) => {
      const angleA = Math.atan2(a[1] - p0[1], a[0] - p0[0]);
      const angleB = Math.atan2(b[1] - p0[1], b[0] - p0[0]);
      return angleA - angleB;
    });

    const hull: Position[] = [p0, sorted[0], sorted[1]];

    for (let i = 2; i < sorted.length; i++) {
      while (
        hull.length > 1 &&
        this.crossProduct(hull[hull.length - 2], hull[hull.length - 1], sorted[i]) <= 0
      ) {
        hull.pop();
      }
      hull.push(sorted[i]);
    }

    // Close the hull
    hull.push(hull[0]);

    return hull;
  }

  /**
   * Calculate cross product for three points
   */
  private static crossProduct(p1: Position, p2: Position, p3: Position): number {
    return (p2[0] - p1[0]) * (p3[1] - p1[1]) - (p2[1] - p1[1]) * (p3[0] - p1[0]);
  }

  /**
   * Get edges from a ring
   */
  private static getEdges(ring: Position[]): [Position, Position][] {
    const edges: [Position, Position][] = [];
    for (let i = 0; i < ring.length - 1; i++) {
      edges.push([ring[i], ring[i + 1]]);
    }
    return edges;
  }

  /**
   * Remove duplicate points
   */
  private static removeDuplicatePoints(points: Position[]): Position[] {
    const unique: Position[] = [];
    const seen = new Set<string>();

    for (const point of points) {
      const key = `${point[0]},${point[1]}`;
      if (!seen.has(key)) {
        seen.add(key);
        unique.push(point);
      }
    }

    return unique;
  }

  /**
   * Erase features: remove poly2 areas from poly1
   */
  static erase(poly1: Polygon, poly2: Polygon): Polygon | null {
    return this.difference(poly1, poly2);
  }

  /**
   * Identity: preserve poly1 and split by poly2
   */
  static identity(poly1: Polygon, poly2: Polygon): Polygon[] {
    const results: Polygon[] = [];

    // Add intersection if exists
    const intersection = this.intersection(poly1, poly2);
    if (intersection) {
      results.push(intersection);
    }

    // Add difference
    const difference = this.difference(poly1, poly2);
    if (difference) {
      results.push(difference);
    }

    return results;
  }

  /**
   * Split polygon by line
   */
  static split(polygon: Polygon, line: Position[]): Polygon[] {
    // Simplified split - would need more complex implementation
    // for production use
    return [polygon];
  }
}
