/**
 * Projection Engine
 * Coordinate transformation and projection operations
 */

import {
  Position,
  Projection,
  SpatialReference,
  ProjectionError,
} from '../types';

export class ProjectionEngine {
  private projections: Map<string, Projection> = new Map();

  /**
   * Register a projection
   */
  register(projection: Projection): void {
    this.projections.set(projection.code, projection);
  }

  /**
   * Get projection by code
   */
  getProjection(code: string): Projection | undefined {
    return this.projections.get(code);
  }

  /**
   * Transform coordinates between projections
   */
  transform(
    coords: Position | Position[],
    fromProj: string | Projection,
    toProj: string | Projection
  ): Position | Position[] {
    const from =
      typeof fromProj === 'string'
        ? this.projections.get(fromProj)
        : fromProj;
    const to =
      typeof toProj === 'string' ? this.projections.get(toProj) : toProj;

    if (!from || !to) {
      throw new ProjectionError('Projection not found');
    }

    if (Array.isArray(coords[0])) {
      return (coords as Position[]).map((coord) =>
        this.transformSingle(coord, from, to)
      );
    }

    return this.transformSingle(coords as Position, from, to);
  }

  /**
   * Transform a single coordinate
   */
  private transformSingle(
    coord: Position,
    from: Projection,
    to: Projection
  ): Position {
    // If projections are the same, return as-is
    if (from.code === to.code) {
      return coord;
    }

    // Transform to WGS84 if source is not WGS84
    let wgs84Coord = coord;
    if (from.code !== 'EPSG:4326') {
      wgs84Coord = this.toWGS84(coord, from);
    }

    // Transform from WGS84 to target if target is not WGS84
    if (to.code !== 'EPSG:4326') {
      return this.fromWGS84(wgs84Coord, to);
    }

    return wgs84Coord;
  }

  /**
   * Transform from projection to WGS84
   */
  private toWGS84(coord: Position, proj: Projection): Position {
    const [x, y] = coord;

    // Handle common projections
    if (proj.code.startsWith('EPSG:326') || proj.code.startsWith('EPSG:327')) {
      // UTM zones
      return this.utmToWGS84(x, y, proj.code);
    }

    if (proj.code === 'EPSG:3857') {
      // Web Mercator
      return this.webMercatorToWGS84(x, y);
    }

    // Default: assume already in WGS84
    return [x, y];
  }

  /**
   * Transform from WGS84 to projection
   */
  private fromWGS84(coord: Position, proj: Projection): Position {
    const [lon, lat] = coord;

    // Handle common projections
    if (proj.code.startsWith('EPSG:326') || proj.code.startsWith('EPSG:327')) {
      // UTM zones
      return this.wgs84ToUTM(lon, lat, proj.code);
    }

    if (proj.code === 'EPSG:3857') {
      // Web Mercator
      return this.wgs84ToWebMercator(lon, lat);
    }

    // Default: return as-is
    return [lon, lat];
  }

  /**
   * WGS84 to Web Mercator (EPSG:3857)
   */
  private wgs84ToWebMercator(lon: number, lat: number): Position {
    const R = 6378137; // Earth's radius in meters

    const x = (lon * Math.PI * R) / 180;
    const y =
      Math.log(Math.tan((90 + lat) * (Math.PI / 360))) * R;

    return [x, y];
  }

  /**
   * Web Mercator to WGS84
   */
  private webMercatorToWGS84(x: number, y: number): Position {
    const R = 6378137;

    const lon = (x * 180) / (Math.PI * R);
    const lat =
      (360 / Math.PI) * Math.atan(Math.exp(y / R)) - 90;

    return [lon, lat];
  }

  /**
   * WGS84 to UTM
   */
  private wgs84ToUTM(lon: number, lat: number, utmCode: string): Position {
    // Extract zone number from EPSG code
    const zone = this.extractUTMZone(utmCode);
    const hemisphere = utmCode.startsWith('EPSG:327') ? 'S' : 'N';

    const a = 6378137.0; // WGS84 semi-major axis
    const f = 1 / 298.257223563; // WGS84 flattening
    const k0 = 0.9996; // UTM scale factor

    const e2 = 2 * f - f * f;
    const e = Math.sqrt(e2);

    const lonRad = (lon * Math.PI) / 180;
    const latRad = (lat * Math.PI) / 180;

    const lonOrigin = ((zone - 1) * 6 - 180 + 3) * (Math.PI / 180);

    const N = a / Math.sqrt(1 - e2 * Math.sin(latRad) * Math.sin(latRad));
    const T = Math.tan(latRad) * Math.tan(latRad);
    const C =
      (e2 / (1 - e2)) * Math.cos(latRad) * Math.cos(latRad);
    const A = (lonRad - lonOrigin) * Math.cos(latRad);

    const M = this.calculateM(latRad, a, e2);

    const x =
      k0 *
        N *
        (A +
          ((1 - T + C) * Math.pow(A, 3)) / 6 +
          ((5 - 18 * T + T * T + 72 * C - 58 * (e2 / (1 - e2))) *
            Math.pow(A, 5)) /
            120) +
      500000;

    const y =
      k0 *
      (M +
        N *
          Math.tan(latRad) *
          (Math.pow(A, 2) / 2 +
            ((5 - T + 9 * C + 4 * C * C) * Math.pow(A, 4)) / 24 +
            ((61 - 58 * T + T * T + 600 * C - 330 * (e2 / (1 - e2))) *
              Math.pow(A, 6)) /
              720));

    const yOffset = hemisphere === 'S' ? 10000000 : 0;

    return [x, y + yOffset];
  }

  /**
   * UTM to WGS84
   */
  private utmToWGS84(x: number, y: number, utmCode: string): Position {
    const zone = this.extractUTMZone(utmCode);
    const hemisphere = utmCode.startsWith('EPSG:327') ? 'S' : 'N';

    const a = 6378137.0;
    const f = 1 / 298.257223563;
    const k0 = 0.9996;

    const e2 = 2 * f - f * f;
    const e = Math.sqrt(e2);

    const xOrigin = 500000;
    const yOrigin = hemisphere === 'S' ? 10000000 : 0;

    const x1 = x - xOrigin;
    const y1 = y - yOrigin;

    const M = y1 / k0;
    const mu = M / (a * (1 - e2 / 4 - (3 * e2 * e2) / 64 - (5 * e2 * e2 * e2) / 256));

    const latRad =
      mu +
      ((3 * e / 2 - (27 * e * e * e) / 32) * Math.sin(2 * mu) +
        ((21 * e * e) / 16 - (55 * e * e * e * e) / 32) * Math.sin(4 * mu) +
        ((151 * e * e * e) / 96) * Math.sin(6 * mu) +
        ((1097 * e * e * e * e) / 512) * Math.sin(8 * mu));

    const N = a / Math.sqrt(1 - e2 * Math.sin(latRad) * Math.sin(latRad));
    const T = Math.tan(latRad) * Math.tan(latRad);
    const C = (e2 / (1 - e2)) * Math.cos(latRad) * Math.cos(latRad);
    const R = (a * (1 - e2)) / Math.pow(1 - e2 * Math.sin(latRad) * Math.sin(latRad), 1.5);
    const D = x1 / (N * k0);

    const lat =
      latRad -
      ((N * Math.tan(latRad)) / R) *
        (Math.pow(D, 2) / 2 -
          ((5 + 3 * T + 10 * C - 4 * C * C - 9 * (e2 / (1 - e2))) *
            Math.pow(D, 4)) /
            24 +
          ((61 + 90 * T + 298 * C + 45 * T * T - 252 * (e2 / (1 - e2)) -
            3 * C * C) *
            Math.pow(D, 6)) /
            720);

    const lon =
      ((zone - 1) * 6 - 180 + 3) * (Math.PI / 180) +
      (D -
        ((1 + 2 * T + C) * Math.pow(D, 3)) / 6 +
        ((5 - 2 * C + 28 * T - 3 * C * C + 8 * (e2 / (1 - e2)) + 24 * T * T) *
          Math.pow(D, 5)) /
          120) /
        Math.cos(latRad);

    return [(lon * 180) / Math.PI, (lat * 180) / Math.PI];
  }

  /**
   * Extract UTM zone from EPSG code
   */
  private extractUTMZone(code: string): number {
    const match = code.match(/\d+$/);
    if (!match) {
      throw new ProjectionError(`Invalid UTM code: ${code}`);
    }

    const epsgNum = parseInt(match[0]);

    if (epsgNum >= 32601 && epsgNum <= 32660) {
      return epsgNum - 32600; // North
    } else if (epsgNum >= 32701 && epsgNum <= 32760) {
      return epsgNum - 32700; // South
    }

    throw new ProjectionError(`Invalid UTM code: ${code}`);
  }

  /**
   * Calculate M (meridian arc distance)
   */
  private calculateM(lat: number, a: number, e2: number): number {
    return (
      a *
      ((1 - e2 / 4 - (3 * e2 * e2) / 64 - (5 * e2 * e2 * e2) / 256) * lat -
        ((3 * e2) / 8 + (3 * e2 * e2) / 32 + (45 * e2 * e2 * e2) / 1024) *
          Math.sin(2 * lat) +
        ((15 * e2 * e2) / 256 + (45 * e2 * e2 * e2) / 1024) * Math.sin(4 * lat) -
        ((35 * e2 * e2 * e2) / 3072) * Math.sin(6 * lat))
    );
  }

  /**
   * Get all registered projections
   */
  getAllProjections(): Projection[] {
    return Array.from(this.projections.values());
  }

  /**
   * Check if projection is supported
   */
  isSupported(code: string): boolean {
    return this.projections.has(code);
  }
}

// Global projection engine instance
export const projectionEngine = new ProjectionEngine();
