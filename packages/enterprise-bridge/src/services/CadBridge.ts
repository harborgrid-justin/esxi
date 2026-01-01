/**
 * CAD service bridge for geometry operations.
 *
 * Provides TypeScript wrapper around WASM CAD engine.
 */

import type { CadGeometry, OperationResult } from '../types';
import { BridgeError } from '../types';
import { WasmLoader } from '../loader/WasmLoader';

/**
 * CAD service bridge.
 */
export class CadBridge {
  private cadEngine: any = null;

  constructor(private readonly loader: WasmLoader) {}

  /**
   * Initialize the CAD engine.
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.cadEngine) {
      const instance = this.loader.getInstance();
      // In production, this would be: this.cadEngine = new instance.CadEngine();
      throw new BridgeError(
        'CAD engine not available. Build WASM module first.',
        'CAD_NOT_AVAILABLE'
      );
    }
  }

  /**
   * Validate geometry and check for topology errors.
   *
   * @param geometry - The geometry to validate
   * @returns Validation errors, if any
   */
  async validateGeometry(geometry: CadGeometry): Promise<OperationResult<string[]>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.validate_geometry(geometry);
      return result as OperationResult<string[]>;
    } catch (error) {
      throw new BridgeError(
        `Geometry validation failed: ${error instanceof Error ? error.message : String(error)}`,
        'VALIDATION_ERROR',
        error
      );
    }
  }

  /**
   * Calculate the bounding box for a geometry.
   *
   * @param geometry - The geometry
   * @returns Bounding box [minX, minY, maxX, maxY]
   */
  async calculateBbox(geometry: CadGeometry): Promise<OperationResult<number[]>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.calculate_bbox(geometry);
      return result as OperationResult<number[]>;
    } catch (error) {
      throw new BridgeError(
        `Bbox calculation failed: ${error instanceof Error ? error.message : String(error)}`,
        'BBOX_ERROR',
        error
      );
    }
  }

  /**
   * Simplify a geometry using the Douglas-Peucker algorithm.
   *
   * @param geometry - The geometry to simplify
   * @param tolerance - Simplification tolerance
   * @returns Simplified geometry
   */
  async simplify(
    geometry: CadGeometry,
    tolerance: number
  ): Promise<OperationResult<CadGeometry>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.simplify(geometry, tolerance);
      return result as OperationResult<CadGeometry>;
    } catch (error) {
      throw new BridgeError(
        `Simplification failed: ${error instanceof Error ? error.message : String(error)}`,
        'SIMPLIFY_ERROR',
        error
      );
    }
  }

  /**
   * Create a buffer around a geometry.
   *
   * @param geometry - The geometry to buffer
   * @param distance - Buffer distance
   * @param segments - Number of segments per quadrant (default: 8)
   * @returns Buffered geometry
   */
  async buffer(
    geometry: CadGeometry,
    distance: number,
    segments = 8
  ): Promise<OperationResult<CadGeometry>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.buffer(geometry, distance, segments);
      return result as OperationResult<CadGeometry>;
    } catch (error) {
      throw new BridgeError(
        `Buffer operation failed: ${error instanceof Error ? error.message : String(error)}`,
        'BUFFER_ERROR',
        error
      );
    }
  }

  /**
   * Compute the union of two geometries.
   *
   * @param geom1 - First geometry
   * @param geom2 - Second geometry
   * @returns Union geometry
   */
  async union(
    geom1: CadGeometry,
    geom2: CadGeometry
  ): Promise<OperationResult<CadGeometry>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.union(geom1, geom2);
      return result as OperationResult<CadGeometry>;
    } catch (error) {
      throw new BridgeError(
        `Union operation failed: ${error instanceof Error ? error.message : String(error)}`,
        'UNION_ERROR',
        error
      );
    }
  }

  /**
   * Compute the intersection of two geometries.
   *
   * @param geom1 - First geometry
   * @param geom2 - Second geometry
   * @returns Intersection geometry
   */
  async intersection(
    geom1: CadGeometry,
    geom2: CadGeometry
  ): Promise<OperationResult<CadGeometry>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.intersection(geom1, geom2);
      return result as OperationResult<CadGeometry>;
    } catch (error) {
      throw new BridgeError(
        `Intersection operation failed: ${error instanceof Error ? error.message : String(error)}`,
        'INTERSECTION_ERROR',
        error
      );
    }
  }

  /**
   * Transform coordinates from one CRS to another.
   *
   * @param geometry - The geometry to transform
   * @param fromCrs - Source coordinate reference system (e.g., "EPSG:4326")
   * @param toCrs - Target coordinate reference system (e.g., "EPSG:3857")
   * @returns Transformed geometry
   */
  async transform(
    geometry: CadGeometry,
    fromCrs: string,
    toCrs: string
  ): Promise<OperationResult<CadGeometry>> {
    await this.ensureInitialized();

    try {
      const result = await this.cadEngine.transform(geometry, fromCrs, toCrs);
      return result as OperationResult<CadGeometry>;
    } catch (error) {
      throw new BridgeError(
        `Transform operation failed: ${error instanceof Error ? error.message : String(error)}`,
        'TRANSFORM_ERROR',
        error
      );
    }
  }

  /**
   * Batch process multiple geometries.
   *
   * @param geometries - Array of geometries
   * @param operation - Operation to perform on each geometry
   * @returns Array of results
   */
  async batchProcess<T>(
    geometries: CadGeometry[],
    operation: (geom: CadGeometry) => Promise<OperationResult<T>>
  ): Promise<OperationResult<T>[]> {
    const results = await Promise.all(
      geometries.map(geom => operation(geom))
    );

    return results;
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    this.cadEngine = null;
  }
}
