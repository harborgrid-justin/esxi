/**
 * Topology Engine
 * Handles topological operations and spatial relationships
 */

import {
  Geometry,
  Point,
  LineString,
  Polygon,
  Position,
  Bounds,
  SpatialRelationship,
  TopologyError,
} from '../types';
import { GeometryFactory } from './GeometryFactory';

export class TopologyEngine {
  /**
   * Test spatial relationship between geometries
   */
  static relate(
    geom1: Geometry,
    geom2: Geometry,
    relationship: SpatialRelationship
  ): boolean {
    switch (relationship) {
      case 'intersects':
        return this.intersects(geom1, geom2);
      case 'contains':
        return this.contains(geom1, geom2);
      case 'within':
        return this.within(geom1, geom2);
      case 'overlaps':
        return this.overlaps(geom1, geom2);
      case 'touches':
        return this.touches(geom1, geom2);
      case 'crosses':
        return this.crosses(geom1, geom2);
      case 'disjoint':
        return this.disjoint(geom1, geom2);
      case 'equals':
        return this.equals(geom1, geom2);
      default:
        throw new TopologyError(`Unknown spatial relationship: ${relationship}`);
    }
  }

  /**
   * Test if geometries intersect
   */
  static intersects(geom1: Geometry, geom2: Geometry): boolean {
    // Quick bounds check
    const bounds1 = GeometryFactory.getBounds(geom1);
    const bounds2 = GeometryFactory.getBounds(geom2);

    if (!this.boundsIntersect(bounds1, bounds2)) {
      return false;
    }

    // Detailed intersection test
    return this.geometriesIntersect(geom1, geom2);
  }

  /**
   * Test if geom1 contains geom2
   */
  static contains(geom1: Geometry, geom2: Geometry): boolean {
    if (geom1.type !== 'Polygon' && geom1.type !== 'MultiPolygon') {
      return false;
    }

    const points2 = GeometryFactory.extractPositions(geom2);
    return points2.every((point) => this.pointInGeometry(point, geom1));
  }

  /**
   * Test if geom1 is within geom2
   */
  static within(geom1: Geometry, geom2: Geometry): boolean {
    return this.contains(geom2, geom1);
  }

  /**
   * Test if geometries overlap
   */
  static overlaps(geom1: Geometry, geom2: Geometry): boolean {
    return (
      this.intersects(geom1, geom2) &&
      !this.contains(geom1, geom2) &&
      !this.contains(geom2, geom1)
    );
  }

  /**
   * Test if geometries touch
   */
  static touches(geom1: Geometry, geom2: Geometry): boolean {
    // Geometries touch if they have at least one point in common
    // but their interiors do not intersect
    return this.intersects(geom1, geom2) && !this.overlaps(geom1, geom2);
  }

  /**
   * Test if geometries cross
   */
  static crosses(geom1: Geometry, geom2: Geometry): boolean {
    // Simplified crossing test
    return (
      this.intersects(geom1, geom2) &&
      !this.contains(geom1, geom2) &&
      !this.within(geom1, geom2)
    );
  }

  /**
   * Test if geometries are disjoint
   */
  static disjoint(geom1: Geometry, geom2: Geometry): boolean {
    return !this.intersects(geom1, geom2);
  }

  /**
   * Test if geometries are equal
   */
  static equals(geom1: Geometry, geom2: Geometry): boolean {
    if (geom1.type !== geom2.type) {
      return false;
    }

    const positions1 = GeometryFactory.extractPositions(geom1);
    const positions2 = GeometryFactory.extractPositions(geom2);

    if (positions1.length !== positions2.length) {
      return false;
    }

    return positions1.every((pos1, i) => {
      const pos2 = positions2[i];
      return pos1[0] === pos2[0] && pos1[1] === pos2[1];
    });
  }

  /**
   * Test if point is inside geometry
   */
  static pointInGeometry(point: Position, geometry: Geometry): boolean {
    switch (geometry.type) {
      case 'Point':
        return this.pointEquals(point, (geometry as Point).coordinates);
      case 'LineString':
        return this.pointOnLine(point, geometry as LineString);
      case 'Polygon':
        return this.pointInPolygon(point, geometry as Polygon);
      case 'MultiPolygon':
        return (geometry as any).coordinates.some((coords: Position[][]) =>
          this.pointInPolygon(point, { type: 'Polygon', coordinates: coords })
        );
      default:
        return false;
    }
  }

  /**
   * Test if point is inside polygon using ray casting algorithm
   */
  static pointInPolygon(point: Position, polygon: Polygon): boolean {
    const [x, y] = point;
    let inside = false;

    // Test exterior ring
    const ring = polygon.coordinates[0];
    for (let i = 0, j = ring.length - 1; i < ring.length; j = i++) {
      const xi = ring[i][0];
      const yi = ring[i][1];
      const xj = ring[j][0];
      const yj = ring[j][1];

      const intersect =
        yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi;

      if (intersect) inside = !inside;
    }

    // If inside exterior ring, check holes
    if (inside) {
      for (let h = 1; h < polygon.coordinates.length; h++) {
        const hole = polygon.coordinates[h];
        let inHole = false;

        for (let i = 0, j = hole.length - 1; i < hole.length; j = i++) {
          const xi = hole[i][0];
          const yi = hole[i][1];
          const xj = hole[j][0];
          const yj = hole[j][1];

          const intersect =
            yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi;

          if (intersect) inHole = !inHole;
        }

        if (inHole) return false;
      }
    }

    return inside;
  }

  /**
   * Test if point is on line
   */
  static pointOnLine(point: Position, line: LineString): boolean {
    const [x, y] = point;
    const coords = line.coordinates;

    for (let i = 0; i < coords.length - 1; i++) {
      const [x1, y1] = coords[i];
      const [x2, y2] = coords[i + 1];

      // Check if point is on line segment
      const crossProduct = (y - y1) * (x2 - x1) - (x - x1) * (y2 - y1);

      if (Math.abs(crossProduct) < 1e-10) {
        // Point is on the line, check if it's between endpoints
        if (x >= Math.min(x1, x2) && x <= Math.max(x1, x2)) {
          if (y >= Math.min(y1, y2) && y <= Math.max(y1, y2)) {
            return true;
          }
        }
      }
    }

    return false;
  }

  /**
   * Test if two points are equal
   */
  static pointEquals(p1: Position, p2: Position, tolerance = 1e-10): boolean {
    return (
      Math.abs(p1[0] - p2[0]) < tolerance &&
      Math.abs(p1[1] - p2[1]) < tolerance
    );
  }

  /**
   * Test if bounds intersect
   */
  static boundsIntersect(bounds1: Bounds, bounds2: Bounds): boolean {
    return !(
      bounds1.maxX < bounds2.minX ||
      bounds1.minX > bounds2.maxX ||
      bounds1.maxY < bounds2.minY ||
      bounds1.minY > bounds2.maxY
    );
  }

  /**
   * Detailed geometry intersection test
   */
  private static geometriesIntersect(geom1: Geometry, geom2: Geometry): boolean {
    // Handle point-geometry intersections
    if (geom1.type === 'Point') {
      return this.pointInGeometry((geom1 as Point).coordinates, geom2);
    }
    if (geom2.type === 'Point') {
      return this.pointInGeometry((geom2 as Point).coordinates, geom1);
    }

    // Handle line-line intersections
    if (geom1.type === 'LineString' && geom2.type === 'LineString') {
      return this.lineIntersectsLine(geom1 as LineString, geom2 as LineString);
    }

    // Handle polygon-polygon intersections
    if (geom1.type === 'Polygon' && geom2.type === 'Polygon') {
      return this.polygonIntersectsPolygon(
        geom1 as Polygon,
        geom2 as Polygon
      );
    }

    // Default: check if any points from one are in the other
    const points1 = GeometryFactory.extractPositions(geom1);
    const points2 = GeometryFactory.extractPositions(geom2);

    return (
      points1.some((p) => this.pointInGeometry(p, geom2)) ||
      points2.some((p) => this.pointInGeometry(p, geom1))
    );
  }

  /**
   * Test if two line strings intersect
   */
  private static lineIntersectsLine(
    line1: LineString,
    line2: LineString
  ): boolean {
    const coords1 = line1.coordinates;
    const coords2 = line2.coordinates;

    for (let i = 0; i < coords1.length - 1; i++) {
      for (let j = 0; j < coords2.length - 1; j++) {
        if (
          this.segmentsIntersect(
            coords1[i],
            coords1[i + 1],
            coords2[j],
            coords2[j + 1]
          )
        ) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Test if two line segments intersect
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
   * Test if two polygons intersect
   */
  private static polygonIntersectsPolygon(
    poly1: Polygon,
    poly2: Polygon
  ): boolean {
    // Check if any vertices of poly1 are inside poly2
    for (const pos of poly1.coordinates[0]) {
      if (this.pointInPolygon(pos, poly2)) {
        return true;
      }
    }

    // Check if any vertices of poly2 are inside poly1
    for (const pos of poly2.coordinates[0]) {
      if (this.pointInPolygon(pos, poly1)) {
        return true;
      }
    }

    // Check if any edges intersect
    const edges1 = this.getPolygonEdges(poly1);
    const edges2 = this.getPolygonEdges(poly2);

    for (const edge1 of edges1) {
      for (const edge2 of edges2) {
        if (this.segmentsIntersect(edge1[0], edge1[1], edge2[0], edge2[1])) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Get all edges from a polygon
   */
  private static getPolygonEdges(polygon: Polygon): [Position, Position][] {
    const edges: [Position, Position][] = [];

    for (const ring of polygon.coordinates) {
      for (let i = 0; i < ring.length - 1; i++) {
        edges.push([ring[i], ring[i + 1]]);
      }
    }

    return edges;
  }

  /**
   * Calculate the intersection point of two line segments
   */
  static lineIntersection(
    p1: Position,
    p2: Position,
    p3: Position,
    p4: Position
  ): Position | null {
    const x1 = p1[0];
    const y1 = p1[1];
    const x2 = p2[0];
    const y2 = p2[1];
    const x3 = p3[0];
    const y3 = p3[1];
    const x4 = p4[0];
    const y4 = p4[1];

    const denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

    if (Math.abs(denom) < 1e-10) {
      return null; // Lines are parallel
    }

    const t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    const u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

    if (t >= 0 && t <= 1 && u >= 0 && u <= 1) {
      return [x1 + t * (x2 - x1), y1 + t * (y2 - y1)];
    }

    return null;
  }

  /**
   * Find the nearest point on a geometry to a given point
   */
  static nearestPoint(point: Position, geometry: Geometry): Position {
    const positions = GeometryFactory.extractPositions(geometry);
    let nearest = positions[0];
    let minDist = GeometryFactory.distance(point, nearest);

    for (const pos of positions) {
      const dist = GeometryFactory.distance(point, pos);
      if (dist < minDist) {
        minDist = dist;
        nearest = pos;
      }
    }

    return nearest;
  }

  /**
   * Calculate the distance from a point to a geometry
   */
  static distanceToGeometry(point: Position, geometry: Geometry): number {
    if (this.pointInGeometry(point, geometry)) {
      return 0;
    }

    const nearest = this.nearestPoint(point, geometry);
    return GeometryFactory.distance(point, nearest);
  }
}
