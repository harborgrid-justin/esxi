/**
 * Datum Transform
 * Datum conversions and transformations
 */

import { Position, DatumTransform as DatumTransformType } from '../types';

export class DatumTransform {
  private transforms: Map<string, DatumTransformType> = new Map();

  /**
   * Register a datum transformation
   */
  register(transform: DatumTransformType): void {
    const key = `${transform.from}->${transform.to}`;
    this.transforms.set(key, transform);
  }

  /**
   * Transform coordinates between datums
   */
  transform(coords: Position, fromDatum: string, toDatum: string): Position {
    if (fromDatum === toDatum) {
      return coords;
    }

    const key = `${fromDatum}->${toDatum}`;
    const transform = this.transforms.get(key);

    if (transform) {
      return this.applyTransform(coords, transform);
    }

    // Try reverse transform
    const reverseKey = `${toDatum}->${fromDatum}`;
    const reverseTransform = this.transforms.get(reverseKey);

    if (reverseTransform) {
      return this.applyReverseTransform(coords, reverseTransform);
    }

    // Try via WGS84
    if (fromDatum !== 'WGS84' && toDatum !== 'WGS84') {
      const wgs84 = this.transform(coords, fromDatum, 'WGS84');
      return this.transform(wgs84, 'WGS84', toDatum);
    }

    // No transformation available, return as-is
    return coords;
  }

  /**
   * Apply Helmert transformation (7-parameter)
   */
  private applyTransform(
    coords: Position,
    transform: DatumTransformType
  ): Position {
    // Parse transformation parameters from definition
    const params = this.parseTransformParams(transform.definition);

    const [x, y, z = 0] = this.geodeticToCartesian(coords);

    // Apply 7-parameter Helmert transformation
    const {
      dx = 0,
      dy = 0,
      dz = 0,
      rx = 0,
      ry = 0,
      rz = 0,
      s = 0,
    } = params;

    // Convert rotations to radians
    const rxRad = (rx / 3600) * (Math.PI / 180);
    const ryRad = (ry / 3600) * (Math.PI / 180);
    const rzRad = (rz / 3600) * (Math.PI / 180);

    // Apply transformation
    const xNew =
      dx +
      (1 + s / 1e6) *
        (x - rz * y + ry * z);
    const yNew =
      dy +
      (1 + s / 1e6) *
        (rz * x + y - rx * z);
    const zNew =
      dz +
      (1 + s / 1e6) *
        (-ry * x + rx * y + z);

    return this.cartesianToGeodetic([xNew, yNew, zNew]);
  }

  /**
   * Apply reverse transformation
   */
  private applyReverseTransform(
    coords: Position,
    transform: DatumTransformType
  ): Position {
    // For reverse, negate all parameters
    const reverseTransform: DatumTransformType = {
      ...transform,
      definition: this.negateTransformParams(transform.definition),
    };

    return this.applyTransform(coords, reverseTransform);
  }

  /**
   * Parse transformation parameters from definition string
   */
  private parseTransformParams(definition: string): {
    dx?: number;
    dy?: number;
    dz?: number;
    rx?: number;
    ry?: number;
    rz?: number;
    s?: number;
  } {
    // Simplified parser - production would handle various formats
    const params: any = {};

    const matches = definition.matchAll(/(\w+)=([-+]?\d+\.?\d*)/g);
    for (const match of matches) {
      params[match[1]] = parseFloat(match[2]);
    }

    return params;
  }

  /**
   * Negate transformation parameters
   */
  private negateTransformParams(definition: string): string {
    return definition.replace(/([-+]?\d+\.?\d*)/g, (match) => {
      const num = parseFloat(match);
      return (-num).toString();
    });
  }

  /**
   * Convert geodetic coordinates to cartesian
   */
  private geodeticToCartesian(coords: Position): Position {
    const [lon, lat, h = 0] = coords;

    const a = 6378137.0; // WGS84 semi-major axis
    const f = 1 / 298.257223563; // WGS84 flattening
    const e2 = 2 * f - f * f;

    const lonRad = (lon * Math.PI) / 180;
    const latRad = (lat * Math.PI) / 180;

    const N = a / Math.sqrt(1 - e2 * Math.sin(latRad) * Math.sin(latRad));

    const x = (N + h) * Math.cos(latRad) * Math.cos(lonRad);
    const y = (N + h) * Math.cos(latRad) * Math.sin(lonRad);
    const z = (N * (1 - e2) + h) * Math.sin(latRad);

    return [x, y, z];
  }

  /**
   * Convert cartesian coordinates to geodetic
   */
  private cartesianToGeodetic(coords: Position): Position {
    const [x, y, z] = coords;

    const a = 6378137.0;
    const f = 1 / 298.257223563;
    const e2 = 2 * f - f * f;

    const lon = Math.atan2(y, x);
    const p = Math.sqrt(x * x + y * y);

    let lat = Math.atan2(z, p * (1 - e2));
    let N = a / Math.sqrt(1 - e2 * Math.sin(lat) * Math.sin(lat));

    // Iterate to converge
    for (let i = 0; i < 5; i++) {
      const sinLat = Math.sin(lat);
      N = a / Math.sqrt(1 - e2 * sinLat * sinLat);
      lat = Math.atan2(z + e2 * N * sinLat, p);
    }

    const h = p / Math.cos(lat) - N;

    return [(lon * 180) / Math.PI, (lat * 180) / Math.PI, h];
  }

  /**
   * Get transformation accuracy
   */
  getAccuracy(fromDatum: string, toDatum: string): number {
    const key = `${fromDatum}->${toDatum}`;
    const transform = this.transforms.get(key);

    return transform?.accuracy || 999; // High value if unknown
  }

  /**
   * Register common datum transformations
   */
  registerCommonTransforms(): void {
    // WGS84 to NAD83 (approximate - they're very similar)
    this.register({
      from: 'WGS84',
      to: 'NAD83',
      definition: 'dx=0,dy=0,dz=0,rx=0,ry=0,rz=0,s=0',
      accuracy: 1.0,
    });

    // WGS84 to NAD27
    this.register({
      from: 'WGS84',
      to: 'NAD27',
      definition: 'dx=-8,dy=160,dz=176,rx=0,ry=0,rz=0,s=0',
      accuracy: 5.0,
    });

    // WGS84 to ETRS89
    this.register({
      from: 'WGS84',
      to: 'ETRS89',
      definition: 'dx=0,dy=0,dz=0,rx=0,ry=0,rz=0,s=0',
      accuracy: 0.1,
    });
  }
}

// Global datum transform instance
export const datumTransform = new DatumTransform();
datumTransform.registerCommonTransforms();
