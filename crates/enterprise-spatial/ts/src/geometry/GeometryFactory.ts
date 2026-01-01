/**
 * Geometry Factory
 * Creates and manipulates geometric objects
 */

import {
  Geometry,
  Point,
  LineString,
  Polygon,
  MultiPoint,
  MultiLineString,
  MultiPolygon,
  GeometryCollection,
  Position,
  Coordinates,
  Bounds,
  GeometryError,
} from '../types';

export class GeometryFactory {
  /**
   * Create a point geometry
   */
  static createPoint(x: number, y: number, z?: number): Point {
    const coordinates: Position = z !== undefined ? [x, y, z] : [x, y];
    return {
      type: 'Point',
      coordinates,
    };
  }

  /**
   * Create a line string geometry
   */
  static createLineString(positions: Position[]): LineString {
    if (positions.length < 2) {
      throw new GeometryError('LineString must have at least 2 positions');
    }
    return {
      type: 'LineString',
      coordinates: positions,
    };
  }

  /**
   * Create a polygon geometry
   */
  static createPolygon(rings: Position[][]): Polygon {
    if (rings.length === 0) {
      throw new GeometryError('Polygon must have at least one ring');
    }

    // Validate rings
    for (const ring of rings) {
      if (ring.length < 4) {
        throw new GeometryError('Polygon ring must have at least 4 positions');
      }
      if (!this.isRingClosed(ring)) {
        throw new GeometryError('Polygon ring must be closed');
      }
    }

    return {
      type: 'Polygon',
      coordinates: rings,
    };
  }

  /**
   * Create a multi-point geometry
   */
  static createMultiPoint(positions: Position[]): MultiPoint {
    if (positions.length === 0) {
      throw new GeometryError('MultiPoint must have at least one position');
    }
    return {
      type: 'MultiPoint',
      coordinates: positions,
    };
  }

  /**
   * Create a multi-line string geometry
   */
  static createMultiLineString(lines: Position[][]): MultiLineString {
    if (lines.length === 0) {
      throw new GeometryError('MultiLineString must have at least one line');
    }
    for (const line of lines) {
      if (line.length < 2) {
        throw new GeometryError('Each line must have at least 2 positions');
      }
    }
    return {
      type: 'MultiLineString',
      coordinates: lines,
    };
  }

  /**
   * Create a multi-polygon geometry
   */
  static createMultiPolygon(polygons: Position[][][]): MultiPolygon {
    if (polygons.length === 0) {
      throw new GeometryError('MultiPolygon must have at least one polygon');
    }
    return {
      type: 'MultiPolygon',
      coordinates: polygons,
    };
  }

  /**
   * Create a geometry collection
   */
  static createGeometryCollection(geometries: Geometry[]): GeometryCollection {
    return {
      type: 'GeometryCollection',
      geometries,
    };
  }

  /**
   * Create a rectangle from bounds
   */
  static createRectangle(bounds: Bounds): Polygon {
    const { minX, minY, maxX, maxY } = bounds;
    return this.createPolygon([
      [
        [minX, minY],
        [maxX, minY],
        [maxX, maxY],
        [minX, maxY],
        [minX, minY],
      ],
    ]);
  }

  /**
   * Create a circle as a polygon
   */
  static createCircle(center: Position, radius: number, steps = 64): Polygon {
    const [cx, cy] = center;
    const coordinates: Position[] = [];

    for (let i = 0; i <= steps; i++) {
      const angle = (i / steps) * 2 * Math.PI;
      const x = cx + radius * Math.cos(angle);
      const y = cy + radius * Math.sin(angle);
      coordinates.push([x, y]);
    }

    return this.createPolygon([coordinates]);
  }

  /**
   * Create an ellipse as a polygon
   */
  static createEllipse(
    center: Position,
    radiusX: number,
    radiusY: number,
    steps = 64
  ): Polygon {
    const [cx, cy] = center;
    const coordinates: Position[] = [];

    for (let i = 0; i <= steps; i++) {
      const angle = (i / steps) * 2 * Math.PI;
      const x = cx + radiusX * Math.cos(angle);
      const y = cy + radiusY * Math.sin(angle);
      coordinates.push([x, y]);
    }

    return this.createPolygon([coordinates]);
  }

  /**
   * Create a regular polygon
   */
  static createRegularPolygon(
    center: Position,
    radius: number,
    sides: number
  ): Polygon {
    if (sides < 3) {
      throw new GeometryError('Regular polygon must have at least 3 sides');
    }

    const [cx, cy] = center;
    const coordinates: Position[] = [];

    for (let i = 0; i <= sides; i++) {
      const angle = (i / sides) * 2 * Math.PI - Math.PI / 2;
      const x = cx + radius * Math.cos(angle);
      const y = cy + radius * Math.sin(angle);
      coordinates.push([x, y]);
    }

    return this.createPolygon([coordinates]);
  }

  /**
   * Create a star polygon
   */
  static createStar(
    center: Position,
    outerRadius: number,
    innerRadius: number,
    points: number
  ): Polygon {
    if (points < 3) {
      throw new GeometryError('Star must have at least 3 points');
    }

    const [cx, cy] = center;
    const coordinates: Position[] = [];

    for (let i = 0; i <= points * 2; i++) {
      const radius = i % 2 === 0 ? outerRadius : innerRadius;
      const angle = (i / (points * 2)) * 2 * Math.PI - Math.PI / 2;
      const x = cx + radius * Math.cos(angle);
      const y = cy + radius * Math.sin(angle);
      coordinates.push([x, y]);
    }

    return this.createPolygon([coordinates]);
  }

  /**
   * Get the bounds of a geometry
   */
  static getBounds(geometry: Geometry): Bounds {
    const positions = this.extractPositions(geometry);
    if (positions.length === 0) {
      throw new GeometryError('Cannot get bounds of empty geometry');
    }

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;
    let minZ: number | undefined;
    let maxZ: number | undefined;

    for (const pos of positions) {
      minX = Math.min(minX, pos[0]);
      minY = Math.min(minY, pos[1]);
      maxX = Math.max(maxX, pos[0]);
      maxY = Math.max(maxY, pos[1]);

      if (pos.length > 2) {
        if (minZ === undefined || pos[2] < minZ) minZ = pos[2];
        if (maxZ === undefined || pos[2] > maxZ) maxZ = pos[2];
      }
    }

    const bounds: Bounds = { minX, minY, maxX, maxY };
    if (minZ !== undefined && maxZ !== undefined) {
      bounds.minZ = minZ;
      bounds.maxZ = maxZ;
    }

    return bounds;
  }

  /**
   * Extract all positions from a geometry
   */
  static extractPositions(geometry: Geometry): Position[] {
    const positions: Position[] = [];

    const extract = (geom: Geometry): void => {
      switch (geom.type) {
        case 'Point':
          positions.push((geom as Point).coordinates);
          break;
        case 'LineString':
          positions.push(...(geom as LineString).coordinates);
          break;
        case 'Polygon':
          for (const ring of (geom as Polygon).coordinates) {
            positions.push(...ring);
          }
          break;
        case 'MultiPoint':
          positions.push(...(geom as MultiPoint).coordinates);
          break;
        case 'MultiLineString':
          for (const line of (geom as MultiLineString).coordinates) {
            positions.push(...line);
          }
          break;
        case 'MultiPolygon':
          for (const polygon of (geom as MultiPolygon).coordinates) {
            for (const ring of polygon) {
              positions.push(...ring);
            }
          }
          break;
        case 'GeometryCollection':
          for (const g of (geom as GeometryCollection).geometries) {
            extract(g);
          }
          break;
      }
    };

    extract(geometry);
    return positions;
  }

  /**
   * Calculate the centroid of a geometry
   */
  static getCentroid(geometry: Geometry): Position {
    const positions = this.extractPositions(geometry);
    if (positions.length === 0) {
      throw new GeometryError('Cannot get centroid of empty geometry');
    }

    let sumX = 0;
    let sumY = 0;
    let sumZ = 0;
    let hasZ = false;

    for (const pos of positions) {
      sumX += pos[0];
      sumY += pos[1];
      if (pos.length > 2) {
        sumZ += pos[2];
        hasZ = true;
      }
    }

    const count = positions.length;
    return hasZ
      ? [sumX / count, sumY / count, sumZ / count]
      : [sumX / count, sumY / count];
  }

  /**
   * Clone a geometry
   */
  static clone(geometry: Geometry): Geometry {
    return JSON.parse(JSON.stringify(geometry));
  }

  /**
   * Check if a ring is closed
   */
  private static isRingClosed(ring: Position[]): boolean {
    if (ring.length < 4) return false;
    const first = ring[0];
    const last = ring[ring.length - 1];
    return first[0] === last[0] && first[1] === last[1];
  }

  /**
   * Close a ring if not already closed
   */
  static closeRing(ring: Position[]): Position[] {
    if (this.isRingClosed(ring)) {
      return ring;
    }
    return [...ring, ring[0]];
  }

  /**
   * Reverse the order of positions in a geometry
   */
  static reverse(geometry: Geometry): Geometry {
    const reversed = this.clone(geometry);

    switch (reversed.type) {
      case 'LineString':
        (reversed as LineString).coordinates.reverse();
        break;
      case 'Polygon':
        for (const ring of (reversed as Polygon).coordinates) {
          ring.reverse();
        }
        break;
      case 'MultiLineString':
        for (const line of (reversed as MultiLineString).coordinates) {
          line.reverse();
        }
        break;
      case 'MultiPolygon':
        for (const polygon of (reversed as MultiPolygon).coordinates) {
          for (const ring of polygon) {
            ring.reverse();
          }
        }
        break;
    }

    return reversed;
  }

  /**
   * Calculate the length of a line string
   */
  static getLength(geometry: LineString): number {
    let length = 0;
    const coords = geometry.coordinates;

    for (let i = 0; i < coords.length - 1; i++) {
      length += this.distance(coords[i], coords[i + 1]);
    }

    return length;
  }

  /**
   * Calculate the area of a polygon
   */
  static getArea(geometry: Polygon): number {
    let area = 0;

    for (let i = 0; i < geometry.coordinates.length; i++) {
      const ring = geometry.coordinates[i];
      const ringArea = this.calculateRingArea(ring);
      area += i === 0 ? ringArea : -ringArea; // Subtract holes
    }

    return Math.abs(area);
  }

  /**
   * Calculate area of a ring using shoelace formula
   */
  private static calculateRingArea(ring: Position[]): number {
    let area = 0;

    for (let i = 0, j = ring.length - 1; i < ring.length; j = i++) {
      area += ring[j][0] * ring[i][1];
      area -= ring[i][0] * ring[j][1];
    }

    return Math.abs(area / 2);
  }

  /**
   * Calculate Euclidean distance between two positions
   */
  static distance(pos1: Position, pos2: Position): number {
    const dx = pos2[0] - pos1[0];
    const dy = pos2[1] - pos1[1];
    const dz = pos1.length > 2 && pos2.length > 2 ? pos2[2] - pos1[2] : 0;
    return Math.sqrt(dx * dx + dy * dy + dz * dz);
  }

  /**
   * Calculate geodetic distance using Haversine formula
   */
  static haversineDistance(
    pos1: Position,
    pos2: Position,
    radius = 6371000
  ): number {
    const [lon1, lat1] = pos1;
    const [lon2, lat2] = pos2;

    const dLat = ((lat2 - lat1) * Math.PI) / 180;
    const dLon = ((lon2 - lon1) * Math.PI) / 180;

    const a =
      Math.sin(dLat / 2) * Math.sin(dLat / 2) +
      Math.cos((lat1 * Math.PI) / 180) *
        Math.cos((lat2 * Math.PI) / 180) *
        Math.sin(dLon / 2) *
        Math.sin(dLon / 2);

    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    return radius * c;
  }
}
