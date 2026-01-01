/**
 * Contour Generation
 * Generate contour lines from raster elevation data
 */

import { RasterData, Position, LineString, Feature } from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export interface ContourOptions {
  interval: number;
  base?: number;
  smooth?: boolean;
  simplify?: boolean;
  tolerance?: number;
}

export class ContourGeneration {
  /**
   * Generate contour lines from DEM
   */
  static generateContours(
    dem: RasterData,
    options: ContourOptions
  ): Feature<LineString>[] {
    const { interval, base = 0, smooth = false, simplify = false, tolerance = 1 } = options;
    const { width, height, bands, bounds, pixelSize } = dem;
    const elevation = bands[0].data;

    // Find elevation range
    const stats = this.calculateStatistics(elevation);
    const minElev = Math.ceil((stats.min - base) / interval) * interval + base;
    const maxElev = Math.floor((stats.max - base) / interval) * interval + base;

    const features: Feature<LineString>[] = [];

    // Generate contours for each elevation
    for (let elev = minElev; elev <= maxElev; elev += interval) {
      const contours = this.marchingSquares(
        elevation,
        width,
        height,
        bounds,
        pixelSize,
        elev
      );

      for (const contour of contours) {
        let coords = contour;

        if (smooth) {
          coords = this.smoothContour(coords);
        }

        if (simplify) {
          coords = this.simplifyContour(coords, tolerance);
        }

        const geometry = GeometryFactory.createLineString(coords);

        features.push({
          type: 'Feature',
          geometry,
          properties: {
            elevation: elev,
            index: Math.round((elev - base) / interval),
          },
        });
      }
    }

    return features;
  }

  /**
   * Marching squares algorithm for contour extraction
   */
  private static marchingSquares(
    data: Float32Array | Uint8Array | Uint16Array,
    width: number,
    height: number,
    bounds: any,
    pixelSize: any,
    isovalue: number
  ): Position[][] {
    const contours: Position[][] = [];
    const visited = new Set<string>();

    for (let row = 0; row < height - 1; row++) {
      for (let col = 0; col < width - 1; col++) {
        const cellKey = `${row},${col}`;

        if (visited.has(cellKey)) continue;

        const contour = this.traceContour(
          data,
          width,
          height,
          bounds,
          pixelSize,
          row,
          col,
          isovalue,
          visited
        );

        if (contour.length > 1) {
          contours.push(contour);
        }
      }
    }

    return contours;
  }

  /**
   * Trace a single contour line
   */
  private static traceContour(
    data: Float32Array | Uint8Array | Uint16Array,
    width: number,
    height: number,
    bounds: any,
    pixelSize: any,
    startRow: number,
    startCol: number,
    isovalue: number,
    visited: Set<string>
  ): Position[] {
    const contour: Position[] = [];
    let row = startRow;
    let col = startCol;
    let prevDir = -1;

    const maxSteps = width * height; // Prevent infinite loops
    let steps = 0;

    while (steps < maxSteps) {
      const cellKey = `${row},${col}`;
      visited.add(cellKey);

      // Get cell configuration
      const config = this.getCellConfiguration(
        data,
        width,
        height,
        row,
        col,
        isovalue
      );

      if (config === 0 || config === 15) {
        break; // No contour or completely filled
      }

      // Find intersection points
      const points = this.getIntersectionPoints(
        data,
        width,
        height,
        bounds,
        pixelSize,
        row,
        col,
        isovalue,
        config
      );

      for (const point of points) {
        contour.push(point);
      }

      // Move to next cell
      const nextCell = this.getNextCell(config, prevDir);

      if (!nextCell) break;

      prevDir = nextCell.dir;
      row += nextCell.drow;
      col += nextCell.dcol;

      // Check if back at start
      if (
        row === startRow &&
        col === startCol &&
        contour.length > 2
      ) {
        break;
      }

      // Check bounds
      if (row < 0 || row >= height - 1 || col < 0 || col >= width - 1) {
        break;
      }

      steps++;
    }

    return contour;
  }

  /**
   * Get cell configuration (marching squares case)
   */
  private static getCellConfiguration(
    data: Float32Array | Uint8Array | Uint16Array,
    width: number,
    height: number,
    row: number,
    col: number,
    isovalue: number
  ): number {
    if (row < 0 || row >= height - 1 || col < 0 || col >= width - 1) {
      return 0;
    }

    const tl = data[row * width + col] >= isovalue ? 1 : 0;
    const tr = data[row * width + col + 1] >= isovalue ? 2 : 0;
    const br = data[(row + 1) * width + col + 1] >= isovalue ? 4 : 0;
    const bl = data[(row + 1) * width + col] >= isovalue ? 8 : 0;

    return tl | tr | br | bl;
  }

  /**
   * Get intersection points for cell
   */
  private static getIntersectionPoints(
    data: Float32Array | Uint8Array | Uint16Array,
    width: number,
    height: number,
    bounds: any,
    pixelSize: any,
    row: number,
    col: number,
    isovalue: number,
    config: number
  ): Position[] {
    const points: Position[] = [];

    const x = bounds.minX + col * pixelSize.x;
    const y = bounds.maxY - row * pixelSize.y;

    const tl = data[row * width + col];
    const tr = data[row * width + col + 1];
    const br = data[(row + 1) * width + col + 1];
    const bl = data[(row + 1) * width + col];

    // Top edge
    if ((config & 3) === 1 || (config & 3) === 2) {
      const t = (isovalue - tl) / (tr - tl);
      points.push([x + t * pixelSize.x, y]);
    }

    // Right edge
    if ((config & 6) === 2 || (config & 6) === 4) {
      const t = (isovalue - tr) / (br - tr);
      points.push([x + pixelSize.x, y - t * pixelSize.y]);
    }

    // Bottom edge
    if ((config & 12) === 4 || (config & 12) === 8) {
      const t = (isovalue - bl) / (br - bl);
      points.push([x + t * pixelSize.x, y - pixelSize.y]);
    }

    // Left edge
    if ((config & 9) === 1 || (config & 9) === 8) {
      const t = (isovalue - tl) / (bl - tl);
      points.push([x, y - t * pixelSize.y]);
    }

    return points;
  }

  /**
   * Get next cell to visit
   */
  private static getNextCell(
    config: number,
    prevDir: number
  ): { drow: number; dcol: number; dir: number } | null {
    // Lookup table for next cell based on configuration
    const nextCells: Record<
      number,
      { drow: number; dcol: number; dir: number }
    > = {
      1: { drow: 0, dcol: -1, dir: 0 },
      2: { drow: -1, dcol: 0, dir: 1 },
      3: { drow: -1, dcol: 0, dir: 1 },
      4: { drow: 0, dcol: 1, dir: 2 },
      6: { drow: -1, dcol: 0, dir: 1 },
      7: { drow: -1, dcol: 0, dir: 1 },
      8: { drow: 1, dcol: 0, dir: 3 },
      9: { drow: 0, dcol: -1, dir: 0 },
      11: { drow: 0, dcol: -1, dir: 0 },
      12: { drow: 1, dcol: 0, dir: 3 },
      13: { drow: 1, dcol: 0, dir: 3 },
      14: { drow: 1, dcol: 0, dir: 3 },
    };

    return nextCells[config] || null;
  }

  /**
   * Smooth contour using Chaikin's algorithm
   */
  private static smoothContour(coords: Position[], iterations = 2): Position[] {
    let smoothed = coords;

    for (let iter = 0; iter < iterations; iter++) {
      const newCoords: Position[] = [];

      for (let i = 0; i < smoothed.length - 1; i++) {
        const p1 = smoothed[i];
        const p2 = smoothed[i + 1];

        newCoords.push([
          p1[0] * 0.75 + p2[0] * 0.25,
          p1[1] * 0.75 + p2[1] * 0.25,
        ]);

        newCoords.push([
          p1[0] * 0.25 + p2[0] * 0.75,
          p1[1] * 0.25 + p2[1] * 0.75,
        ]);
      }

      newCoords.push(smoothed[smoothed.length - 1]);
      smoothed = newCoords;
    }

    return smoothed;
  }

  /**
   * Simplify contour using Douglas-Peucker
   */
  private static simplifyContour(coords: Position[], tolerance: number): Position[] {
    if (coords.length <= 2) {
      return coords;
    }

    const first = coords[0];
    const last = coords[coords.length - 1];

    let maxDist = 0;
    let maxIndex = 0;

    for (let i = 1; i < coords.length - 1; i++) {
      const dist = this.perpendicularDistance(coords[i], first, last);
      if (dist > maxDist) {
        maxDist = dist;
        maxIndex = i;
      }
    }

    if (maxDist > tolerance) {
      const left = this.simplifyContour(coords.slice(0, maxIndex + 1), tolerance);
      const right = this.simplifyContour(coords.slice(maxIndex), tolerance);
      return [...left.slice(0, -1), ...right];
    }

    return [first, last];
  }

  /**
   * Calculate perpendicular distance from point to line
   */
  private static perpendicularDistance(
    point: Position,
    lineStart: Position,
    lineEnd: Position
  ): number {
    const [x, y] = point;
    const [x1, y1] = lineStart;
    const [x2, y2] = lineEnd;

    const dx = x2 - x1;
    const dy = y2 - y1;

    if (dx === 0 && dy === 0) {
      return GeometryFactory.distance(point, lineStart);
    }

    const num = Math.abs(dy * x - dx * y + x2 * y1 - y2 * x1);
    const den = Math.sqrt(dx * dx + dy * dy);

    return num / den;
  }

  /**
   * Calculate statistics
   */
  private static calculateStatistics(
    data: Float32Array | Uint8Array | Uint16Array
  ): any {
    let min = Infinity;
    let max = -Infinity;

    for (const value of data) {
      if (isFinite(value)) {
        min = Math.min(min, value);
        max = Math.max(max, value);
      }
    }

    return { min, max };
  }

  /**
   * Generate index contours (thicker lines at intervals)
   */
  static generateIndexContours(
    dem: RasterData,
    options: ContourOptions,
    indexInterval: number
  ): {
    regular: Feature<LineString>[];
    index: Feature<LineString>[];
  } {
    const all = this.generateContours(dem, options);

    const regular: Feature<LineString>[] = [];
    const index: Feature<LineString>[] = [];

    for (const feature of all) {
      const elev = feature.properties.elevation;
      if (elev % indexInterval === 0) {
        index.push(feature);
      } else {
        regular.push(feature);
      }
    }

    return { regular, index };
  }
}
