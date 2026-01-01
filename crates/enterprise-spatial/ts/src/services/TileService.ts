/**
 * Tile Service
 * Vector and raster tile management
 */

import { TileCoordinate, TileRequest, VectorTile, Bounds } from '../types';

export class TileService {
  private tileCache: Map<string, any> = new Map();
  private maxCacheSize = 1000;

  /**
   * Get tile URL for given coordinate
   */
  getTileUrl(
    baseUrl: string,
    coord: TileCoordinate,
    format: 'png' | 'jpg' | 'pbf' | 'mvt' = 'png'
  ): string {
    const { x, y, z } = coord;
    return baseUrl
      .replace('{z}', z.toString())
      .replace('{x}', x.toString())
      .replace('{y}', y.toString())
      .replace('{ext}', format);
  }

  /**
   * Fetch raster tile
   */
  async fetchRasterTile(
    baseUrl: string,
    coord: TileCoordinate,
    format: 'png' | 'jpg' = 'png'
  ): Promise<ImageBitmap | null> {
    const cacheKey = this.getCacheKey(coord, format);

    // Check cache
    if (this.tileCache.has(cacheKey)) {
      return this.tileCache.get(cacheKey);
    }

    const url = this.getTileUrl(baseUrl, coord, format);

    try {
      const response = await fetch(url);
      const blob = await response.blob();
      const bitmap = await createImageBitmap(blob);

      this.cacheSet(cacheKey, bitmap);

      return bitmap;
    } catch (error) {
      console.error('Error fetching raster tile:', error);
      return null;
    }
  }

  /**
   * Fetch vector tile
   */
  async fetchVectorTile(
    baseUrl: string,
    coord: TileCoordinate
  ): Promise<VectorTile | null> {
    const cacheKey = this.getCacheKey(coord, 'pbf');

    // Check cache
    if (this.tileCache.has(cacheKey)) {
      return this.tileCache.get(cacheKey);
    }

    const url = this.getTileUrl(baseUrl, coord, 'pbf');

    try {
      const response = await fetch(url);
      const buffer = await response.arrayBuffer();
      const tile = this.parseVectorTile(buffer);

      this.cacheSet(cacheKey, tile);

      return tile;
    } catch (error) {
      console.error('Error fetching vector tile:', error);
      return null;
    }
  }

  /**
   * Parse vector tile from Protocol Buffer
   */
  private parseVectorTile(buffer: ArrayBuffer): VectorTile {
    // Simplified parser - production would use proper MVT/PBF parser
    return {
      layers: {},
      extent: 4096,
    };
  }

  /**
   * Get tile coordinates for bounds
   */
  getTilesForBounds(bounds: Bounds, zoom: number): TileCoordinate[] {
    const tiles: TileCoordinate[] = [];

    const { minX, minY, maxX, maxY } = bounds;

    // Convert bounds to tile coordinates
    const minTileX = this.lonToTile(minX, zoom);
    const maxTileX = this.lonToTile(maxX, zoom);
    const minTileY = this.latToTile(maxY, zoom); // Note: Y is inverted
    const maxTileY = this.latToTile(minY, zoom);

    for (let x = minTileX; x <= maxTileX; x++) {
      for (let y = minTileY; y <= maxTileY; y++) {
        tiles.push({ x, y, z: zoom });
      }
    }

    return tiles;
  }

  /**
   * Convert longitude to tile X coordinate
   */
  private lonToTile(lon: number, zoom: number): number {
    return Math.floor(((lon + 180) / 360) * Math.pow(2, zoom));
  }

  /**
   * Convert latitude to tile Y coordinate
   */
  private latToTile(lat: number, zoom: number): number {
    const latRad = (lat * Math.PI) / 180;
    return Math.floor(
      ((1 - Math.log(Math.tan(latRad) + 1 / Math.cos(latRad)) / Math.PI) / 2) *
        Math.pow(2, zoom)
    );
  }

  /**
   * Get tile bounds
   */
  getTileBounds(coord: TileCoordinate): Bounds {
    const { x, y, z } = coord;
    const n = Math.pow(2, z);

    const minX = (x / n) * 360 - 180;
    const maxX = ((x + 1) / n) * 360 - 180;

    const minY = this.tileToLat(y + 1, z);
    const maxY = this.tileToLat(y, z);

    return { minX, minY, maxX, maxY };
  }

  /**
   * Convert tile Y to latitude
   */
  private tileToLat(y: number, z: number): number {
    const n = Math.pow(2, z);
    const latRad = Math.atan(Math.sinh(Math.PI * (1 - (2 * y) / n)));
    return (latRad * 180) / Math.PI;
  }

  /**
   * Prefetch tiles for a bounds
   */
  async prefetchTiles(
    baseUrl: string,
    bounds: Bounds,
    zoom: number,
    format: 'png' | 'jpg' | 'pbf' = 'png'
  ): Promise<void> {
    const tiles = this.getTilesForBounds(bounds, zoom);

    const promises = tiles.map((coord) => {
      if (format === 'pbf') {
        return this.fetchVectorTile(baseUrl, coord);
      } else {
        return this.fetchRasterTile(baseUrl, coord, format);
      }
    });

    await Promise.all(promises);
  }

  /**
   * Clear tile cache
   */
  clearCache(): void {
    this.tileCache.clear();
  }

  /**
   * Get cache key
   */
  private getCacheKey(coord: TileCoordinate, format: string): string {
    return `${coord.z}/${coord.x}/${coord.y}.${format}`;
  }

  /**
   * Cache set with size limit
   */
  private cacheSet(key: string, value: any): void {
    if (this.tileCache.size >= this.maxCacheSize) {
      // Remove oldest entry
      const firstKey = this.tileCache.keys().next().value;
      this.tileCache.delete(firstKey);
    }

    this.tileCache.set(key, value);
  }

  /**
   * Get tile statistics
   */
  getStats(): {
    cacheSize: number;
    maxCacheSize: number;
    cacheUsage: number;
  } {
    return {
      cacheSize: this.tileCache.size,
      maxCacheSize: this.maxCacheSize,
      cacheUsage: (this.tileCache.size / this.maxCacheSize) * 100,
    };
  }
}
