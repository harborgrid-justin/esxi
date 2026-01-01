/**
 * Proximity Analysis
 * Distance calculations and nearest neighbor analysis
 */

import {
  Geometry,
  Feature,
  Position,
  DistanceResult,
  Bounds,
} from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';
import { TopologyEngine } from '../geometry/TopologyEngine';

export class ProximityAnalysis {
  /**
   * Calculate distance between two geometries
   */
  static distance(geom1: Geometry, geom2: Geometry, geodetic = false): DistanceResult {
    const points1 = GeometryFactory.extractPositions(geom1);
    const points2 = GeometryFactory.extractPositions(geom2);

    let minDist = Infinity;
    let closestPair: [Position, Position] | null = null;

    for (const p1 of points1) {
      for (const p2 of points2) {
        const dist = geodetic
          ? GeometryFactory.haversineDistance(p1, p2)
          : GeometryFactory.distance(p1, p2);

        if (dist < minDist) {
          minDist = dist;
          closestPair = [p1, p2];
        }
      }
    }

    return {
      distance: minDist,
      units: geodetic ? 'meters' : 'units',
      path: closestPair || undefined,
    };
  }

  /**
   * Find nearest feature to a geometry
   */
  static nearest(
    geometry: Geometry,
    features: Feature[],
    options: { limit?: number; maxDistance?: number; geodetic?: boolean } = {}
  ): Feature[] {
    const { limit = 1, maxDistance = Infinity, geodetic = false } = options;

    const distances = features.map((feature) => ({
      feature,
      distance: this.distance(geometry, feature.geometry, geodetic).distance,
    }));

    // Filter by max distance
    const filtered = distances.filter((d) => d.distance <= maxDistance);

    // Sort by distance
    filtered.sort((a, b) => a.distance - b.distance);

    // Return top N
    return filtered.slice(0, limit).map((d) => d.feature);
  }

  /**
   * Find all features within distance of geometry
   */
  static within(
    geometry: Geometry,
    features: Feature[],
    distance: number,
    geodetic = false
  ): Feature[] {
    return features.filter((feature) => {
      const dist = this.distance(geometry, feature.geometry, geodetic).distance;
      return dist <= distance;
    });
  }

  /**
   * Generate near table (all features within distance)
   */
  static nearTable(
    sourceFeatures: Feature[],
    targetFeatures: Feature[],
    searchRadius: number,
    geodetic = false
  ): Array<{
    source: Feature;
    target: Feature;
    distance: number;
  }> {
    const results: Array<{
      source: Feature;
      target: Feature;
      distance: number;
    }> = [];

    for (const source of sourceFeatures) {
      for (const target of targetFeatures) {
        const dist = this.distance(
          source.geometry,
          target.geometry,
          geodetic
        ).distance;

        if (dist <= searchRadius) {
          results.push({ source, target, distance: dist });
        }
      }
    }

    return results.sort((a, b) => a.distance - b.distance);
  }

  /**
   * K-nearest neighbors
   */
  static knn(
    point: Position,
    features: Feature[],
    k: number,
    geodetic = false
  ): Feature[] {
    const distances = features.map((feature) => {
      const geomPoints = GeometryFactory.extractPositions(feature.geometry);
      let minDist = Infinity;

      for (const p of geomPoints) {
        const dist = geodetic
          ? GeometryFactory.haversineDistance(point, p)
          : GeometryFactory.distance(point, p);
        minDist = Math.min(minDist, dist);
      }

      return { feature, distance: minDist };
    });

    distances.sort((a, b) => a.distance - b.distance);

    return distances.slice(0, k).map((d) => d.feature);
  }

  /**
   * Calculate Euclidean distance matrix
   */
  static distanceMatrix(
    features: Feature[],
    geodetic = false
  ): number[][] {
    const n = features.length;
    const matrix: number[][] = Array(n)
      .fill(0)
      .map(() => Array(n).fill(0));

    for (let i = 0; i < n; i++) {
      for (let j = i + 1; j < n; j++) {
        const dist = this.distance(
          features[i].geometry,
          features[j].geometry,
          geodetic
        ).distance;
        matrix[i][j] = dist;
        matrix[j][i] = dist;
      }
    }

    return matrix;
  }

  /**
   * Calculate centroid distance
   */
  static centroidDistance(
    geom1: Geometry,
    geom2: Geometry,
    geodetic = false
  ): number {
    const c1 = GeometryFactory.getCentroid(geom1);
    const c2 = GeometryFactory.getCentroid(geom2);

    return geodetic
      ? GeometryFactory.haversineDistance(c1, c2)
      : GeometryFactory.distance(c1, c2);
  }

  /**
   * Find features along a route
   */
  static alongRoute(
    route: Position[],
    features: Feature[],
    searchDistance: number,
    geodetic = false
  ): Feature[] {
    const results: Feature[] = [];

    for (const feature of features) {
      const geomPoints = GeometryFactory.extractPositions(feature.geometry);

      for (const point of geomPoints) {
        let minDist = Infinity;

        // Check distance to each route segment
        for (let i = 0; i < route.length - 1; i++) {
          const dist = this.pointToSegmentDistance(
            point,
            route[i],
            route[i + 1],
            geodetic
          );
          minDist = Math.min(minDist, dist);
        }

        if (minDist <= searchDistance) {
          results.push(feature);
          break;
        }
      }
    }

    return results;
  }

  /**
   * Calculate point to line segment distance
   */
  private static pointToSegmentDistance(
    point: Position,
    segStart: Position,
    segEnd: Position,
    geodetic: boolean
  ): number {
    const [px, py] = point;
    const [x1, y1] = segStart;
    const [x2, y2] = segEnd;

    const dx = x2 - x1;
    const dy = y2 - y1;

    if (dx === 0 && dy === 0) {
      return geodetic
        ? GeometryFactory.haversineDistance(point, segStart)
        : GeometryFactory.distance(point, segStart);
    }

    const t = Math.max(
      0,
      Math.min(1, ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy))
    );

    const nearestPoint: Position = [x1 + t * dx, y1 + t * dy];

    return geodetic
      ? GeometryFactory.haversineDistance(point, nearestPoint)
      : GeometryFactory.distance(point, nearestPoint);
  }

  /**
   * Generate service areas (isochrones)
   */
  static serviceArea(
    center: Position,
    distances: number[],
    steps = 32
  ): Geometry[] {
    return distances.map((distance) =>
      GeometryFactory.createCircle(center, distance, steps)
    );
  }

  /**
   * Point in proximity analysis
   */
  static pointsInProximity(
    points: Position[],
    threshold: number,
    geodetic = false
  ): Array<[Position, Position]> {
    const pairs: Array<[Position, Position]> = [];

    for (let i = 0; i < points.length; i++) {
      for (let j = i + 1; j < points.length; j++) {
        const dist = geodetic
          ? GeometryFactory.haversineDistance(points[i], points[j])
          : GeometryFactory.distance(points[i], points[j]);

        if (dist <= threshold) {
          pairs.push([points[i], points[j]]);
        }
      }
    }

    return pairs;
  }

  /**
   * Thiessen polygons (Voronoi diagram)
   */
  static thiessenPolygons(points: Position[], bounds: Bounds): Geometry[] {
    // Simplified Voronoi - production would use proper algorithm
    return points.map((point) =>
      GeometryFactory.createCircle(point, 100, 16)
    );
  }
}
