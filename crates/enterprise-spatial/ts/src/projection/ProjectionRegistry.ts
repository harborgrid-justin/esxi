/**
 * Projection Registry
 * Database of common coordinate reference systems (EPSG codes)
 */

import { Projection } from '../types';
import { projectionEngine } from './ProjectionEngine';

export class ProjectionRegistry {
  /**
   * Register common projections
   */
  static registerCommonProjections(): void {
    // WGS84 Geographic
    projectionEngine.register({
      code: 'EPSG:4326',
      name: 'WGS 84',
      proj4def: '+proj=longlat +datum=WGS84 +no_defs',
      units: 'degrees',
    });

    // Web Mercator
    projectionEngine.register({
      code: 'EPSG:3857',
      name: 'WGS 84 / Pseudo-Mercator',
      proj4def: '+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs',
      bounds: {
        minX: -20037508.34,
        minY: -20037508.34,
        maxX: 20037508.34,
        maxY: 20037508.34,
      },
      units: 'meters',
    });

    // UTM Zones North (sample zones)
    for (let zone = 1; zone <= 60; zone++) {
      const code = `EPSG:${32600 + zone}`;
      const centralMeridian = zone * 6 - 183;

      projectionEngine.register({
        code,
        name: `WGS 84 / UTM zone ${zone}N`,
        proj4def: `+proj=utm +zone=${zone} +datum=WGS84 +units=m +no_defs`,
        units: 'meters',
      });
    }

    // UTM Zones South (sample zones)
    for (let zone = 1; zone <= 60; zone++) {
      const code = `EPSG:${32700 + zone}`;

      projectionEngine.register({
        code,
        name: `WGS 84 / UTM zone ${zone}S`,
        proj4def: `+proj=utm +zone=${zone} +south +datum=WGS84 +units=m +no_defs`,
        units: 'meters',
      });
    }

    // NAD83
    projectionEngine.register({
      code: 'EPSG:4269',
      name: 'NAD83',
      proj4def: '+proj=longlat +ellps=GRS80 +datum=NAD83 +no_defs',
      units: 'degrees',
    });

    // NAD83 / Conus Albers
    projectionEngine.register({
      code: 'EPSG:5070',
      name: 'NAD83 / Conus Albers',
      proj4def: '+proj=aea +lat_1=29.5 +lat_2=45.5 +lat_0=37.5 +lon_0=-96 +x_0=0 +y_0=0 +ellps=GRS80 +datum=NAD83 +units=m +no_defs',
      units: 'meters',
    });

    // British National Grid
    projectionEngine.register({
      code: 'EPSG:27700',
      name: 'OSGB 1936 / British National Grid',
      proj4def: '+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +datum=OSGB36 +units=m +no_defs',
      units: 'meters',
    });

    // ETRS89
    projectionEngine.register({
      code: 'EPSG:4258',
      name: 'ETRS89',
      proj4def: '+proj=longlat +ellps=GRS80 +no_defs',
      units: 'degrees',
    });

    // WGS 84 / World Equidistant Cylindrical
    projectionEngine.register({
      code: 'EPSG:4087',
      name: 'WGS 84 / World Equidistant Cylindrical',
      proj4def: '+proj=eqc +lat_ts=0 +lat_0=0 +lon_0=0 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs',
      units: 'meters',
    });

    // NAD27
    projectionEngine.register({
      code: 'EPSG:4267',
      name: 'NAD27',
      proj4def: '+proj=longlat +ellps=clrk66 +datum=NAD27 +no_defs',
      units: 'degrees',
    });
  }

  /**
   * Search projections by name or code
   */
  static search(query: string): Projection[] {
    const lowerQuery = query.toLowerCase();
    return projectionEngine
      .getAllProjections()
      .filter(
        (proj) =>
          proj.code.toLowerCase().includes(lowerQuery) ||
          proj.name.toLowerCase().includes(lowerQuery)
      );
  }

  /**
   * Get projection by EPSG code
   */
  static getByEPSG(code: number): Projection | undefined {
    return projectionEngine.getProjection(`EPSG:${code}`);
  }

  /**
   * Get all geographic projections
   */
  static getGeographic(): Projection[] {
    return projectionEngine
      .getAllProjections()
      .filter((proj) => proj.units === 'degrees');
  }

  /**
   * Get all projected coordinate systems
   */
  static getProjected(): Projection[] {
    return projectionEngine
      .getAllProjections()
      .filter((proj) => proj.units !== 'degrees');
  }

  /**
   * Get UTM zone for a location
   */
  static getUTMZone(lon: number, lat: number): string {
    const zone = Math.floor((lon + 180) / 6) + 1;
    const hemisphere = lat >= 0 ? 'N' : 'S';
    const epsgBase = hemisphere === 'N' ? 32600 : 32700;

    return `EPSG:${epsgBase + zone}`;
  }

  /**
   * Suggest projection for bounds
   */
  static suggestProjection(bounds: {
    minX: number;
    minY: number;
    maxX: number;
    maxY: number;
  }): string {
    const centerLon = (bounds.minX + bounds.maxX) / 2;
    const centerLat = (bounds.minY + bounds.maxY) / 2;

    const lonSpan = bounds.maxX - bounds.minX;
    const latSpan = bounds.maxY - bounds.minY;

    // For global or continental scale, use Web Mercator or geographic
    if (lonSpan > 90 || latSpan > 45) {
      return 'EPSG:3857'; // Web Mercator
    }

    // For regional scale, use UTM
    if (lonSpan < 30 && latSpan < 20) {
      return this.getUTMZone(centerLon, centerLat);
    }

    // For national/continental scale, use Albers if in North America
    if (
      centerLon > -130 &&
      centerLon < -60 &&
      centerLat > 20 &&
      centerLat < 50
    ) {
      return 'EPSG:5070'; // Conus Albers
    }

    // Default to WGS84
    return 'EPSG:4326';
  }
}

// Initialize common projections
ProjectionRegistry.registerCommonProjections();
