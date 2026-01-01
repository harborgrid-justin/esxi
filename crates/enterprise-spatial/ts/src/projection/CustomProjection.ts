/**
 * Custom Projection
 * Create and manage custom coordinate reference systems
 */

import { Projection, SpatialReference } from '../types';
import { projectionEngine } from './ProjectionEngine';

export class CustomProjection {
  /**
   * Create custom projection from Proj4 definition
   */
  static fromProj4(code: string, name: string, proj4def: string): Projection {
    // Parse Proj4 string to determine units
    const units = this.parseUnits(proj4def);

    const projection: Projection = {
      code,
      name,
      proj4def,
      units,
    };

    projectionEngine.register(projection);

    return projection;
  }

  /**
   * Create custom projection from WKT
   */
  static fromWKT(code: string, wkt: string): Projection {
    // Parse WKT to extract information
    const name = this.extractNameFromWKT(wkt);
    const units = this.extractUnitsFromWKT(wkt);
    const proj4def = this.wktToProj4(wkt);

    const projection: Projection = {
      code,
      name,
      proj4def,
      units,
    };

    projectionEngine.register(projection);

    return projection;
  }

  /**
   * Create custom Albers Equal Area projection
   */
  static createAlbers(
    code: string,
    name: string,
    centralMeridian: number,
    latitudeOfOrigin: number,
    standardParallel1: number,
    standardParallel2: number
  ): Projection {
    const proj4def = `+proj=aea +lat_1=${standardParallel1} +lat_2=${standardParallel2} +lat_0=${latitudeOfOrigin} +lon_0=${centralMeridian} +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs`;

    return this.fromProj4(code, name, proj4def);
  }

  /**
   * Create custom Lambert Conformal Conic projection
   */
  static createLambertConformalConic(
    code: string,
    name: string,
    centralMeridian: number,
    latitudeOfOrigin: number,
    standardParallel1: number,
    standardParallel2: number
  ): Projection {
    const proj4def = `+proj=lcc +lat_1=${standardParallel1} +lat_2=${standardParallel2} +lat_0=${latitudeOfOrigin} +lon_0=${centralMeridian} +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs`;

    return this.fromProj4(code, name, proj4def);
  }

  /**
   * Create custom Transverse Mercator projection
   */
  static createTransverseMercator(
    code: string,
    name: string,
    centralMeridian: number,
    latitudeOfOrigin: number,
    scaleFactor = 0.9996
  ): Projection {
    const proj4def = `+proj=tmerc +lat_0=${latitudeOfOrigin} +lon_0=${centralMeridian} +k=${scaleFactor} +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs`;

    return this.fromProj4(code, name, proj4def);
  }

  /**
   * Create custom local coordinate system
   */
  static createLocal(
    code: string,
    name: string,
    originX: number,
    originY: number,
    rotation = 0
  ): Projection {
    const proj4def = `+proj=tmerc +lat_0=${originY} +lon_0=${originX} +k=1.0 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs`;

    return this.fromProj4(code, name, proj4def);
  }

  /**
   * Parse units from Proj4 string
   */
  private static parseUnits(proj4def: string): 'degrees' | 'meters' | 'feet' | 'us-feet' {
    if (proj4def.includes('+units=m')) return 'meters';
    if (proj4def.includes('+units=ft')) return 'feet';
    if (proj4def.includes('+units=us-ft')) return 'us-feet';
    if (proj4def.includes('+proj=longlat')) return 'degrees';

    return 'meters'; // default
  }

  /**
   * Extract name from WKT
   */
  private static extractNameFromWKT(wkt: string): string {
    const match = wkt.match(/^[A-Z_]+\["([^"]+)"/);
    return match ? match[1] : 'Custom Projection';
  }

  /**
   * Extract units from WKT
   */
  private static extractUnitsFromWKT(wkt: string): 'degrees' | 'meters' | 'feet' | 'us-feet' {
    if (wkt.includes('UNIT["metre"') || wkt.includes('UNIT["meter"')) {
      return 'meters';
    }
    if (wkt.includes('UNIT["foot"') || wkt.includes('UNIT["ft"')) {
      return 'feet';
    }
    if (wkt.includes('UNIT["degree"')) {
      return 'degrees';
    }

    return 'meters';
  }

  /**
   * Convert WKT to Proj4 (simplified)
   */
  private static wktToProj4(wkt: string): string {
    // Simplified conversion - production would use full WKT parser
    if (wkt.includes('PROJCS')) {
      return '+proj=tmerc +datum=WGS84 +units=m +no_defs';
    }

    return '+proj=longlat +datum=WGS84 +no_defs';
  }

  /**
   * Validate projection definition
   */
  static validate(projection: Projection): { valid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!projection.code) {
      errors.push('Projection code is required');
    }

    if (!projection.name) {
      errors.push('Projection name is required');
    }

    if (!projection.proj4def) {
      errors.push('Proj4 definition is required');
    }

    if (!projection.units) {
      errors.push('Projection units are required');
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Export projection to various formats
   */
  static export(
    projection: Projection,
    format: 'proj4' | 'wkt' | 'json'
  ): string {
    switch (format) {
      case 'proj4':
        return projection.proj4def;

      case 'wkt':
        return this.proj4ToWKT(projection);

      case 'json':
        return JSON.stringify(projection, null, 2);

      default:
        return projection.proj4def;
    }
  }

  /**
   * Convert Proj4 to WKT (simplified)
   */
  private static proj4ToWKT(projection: Projection): string {
    // Simplified - production would generate proper WKT
    const unit = projection.units === 'degrees' ? 'degree' : 'metre';

    return `PROJCS["${projection.name}",
  GEOGCS["WGS 84",
    DATUM["WGS_1984",
      SPHEROID["WGS 84",6378137,298.257223563]],
    PRIMEM["Greenwich",0],
    UNIT["${unit}",1]],
  PROJECTION["Transverse_Mercator"],
  UNIT["${unit}",1]]`;
  }
}
