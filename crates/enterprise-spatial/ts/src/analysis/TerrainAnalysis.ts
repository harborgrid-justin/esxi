/**
 * Terrain Analysis
 * Digital Elevation Model (DEM) processing and terrain analysis
 */

import {
  RasterData,
  RasterBand,
  Position,
  Bounds,
  ElevationProfile,
} from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export class TerrainAnalysis {
  /**
   * Calculate slope from DEM
   */
  static slope(dem: RasterData, units: 'degrees' | 'percent' = 'degrees'): RasterData {
    const { width, height, bands, pixelSize } = dem;
    const elevation = bands[0].data;
    const slopeData = new Float32Array(width * height);

    for (let row = 1; row < height - 1; row++) {
      for (let col = 1; col < width - 1; col++) {
        const dzdx = this.calculateGradient(
          elevation,
          row,
          col,
          width,
          height,
          pixelSize.x,
          'x'
        );

        const dzdy = this.calculateGradient(
          elevation,
          row,
          col,
          width,
          height,
          pixelSize.y,
          'y'
        );

        const slopeRad = Math.atan(Math.sqrt(dzdx * dzdx + dzdy * dzdy));
        const slope = units === 'degrees'
          ? (slopeRad * 180) / Math.PI
          : Math.tan(slopeRad) * 100;

        slopeData[row * width + col] = slope;
      }
    }

    const band: RasterBand = {
      data: slopeData,
      statistics: this.calculateStatistics(slopeData),
    };

    return {
      ...dem,
      bands: [band],
    };
  }

  /**
   * Calculate aspect (direction of slope)
   */
  static aspect(dem: RasterData): RasterData {
    const { width, height, bands, pixelSize } = dem;
    const elevation = bands[0].data;
    const aspectData = new Float32Array(width * height);

    for (let row = 1; row < height - 1; row++) {
      for (let col = 1; col < width - 1; col++) {
        const dzdx = this.calculateGradient(
          elevation,
          row,
          col,
          width,
          height,
          pixelSize.x,
          'x'
        );

        const dzdy = this.calculateGradient(
          elevation,
          row,
          col,
          width,
          height,
          pixelSize.y,
          'y'
        );

        let aspect = Math.atan2(dzdy, -dzdx);

        // Convert to degrees (0-360, north = 0)
        aspect = ((aspect * 180) / Math.PI + 90) % 360;
        if (aspect < 0) aspect += 360;

        aspectData[row * width + col] = aspect;
      }
    }

    const band: RasterBand = {
      data: aspectData,
      statistics: this.calculateStatistics(aspectData),
    };

    return {
      ...dem,
      bands: [band],
    };
  }

  /**
   * Calculate hillshade
   */
  static hillshade(
    dem: RasterData,
    azimuth = 315,
    altitude = 45
  ): RasterData {
    const { width, height, bands, pixelSize } = dem;
    const elevation = bands[0].data;
    const hillshadeData = new Uint8Array(width * height);

    const azimuthRad = (azimuth * Math.PI) / 180;
    const altitudeRad = (altitude * Math.PI) / 180;

    for (let row = 1; row < height - 1; row++) {
      for (let col = 1; col < width - 1; col++) {
        const dzdx = this.calculateGradient(
          elevation,
          row,
          col,
          width,
          height,
          pixelSize.x,
          'x'
        );

        const dzdy = this.calculateGradient(
          elevation,
          row,
          col,
          width,
          height,
          pixelSize.y,
          'y'
        );

        const slopeRad = Math.atan(Math.sqrt(dzdx * dzdx + dzdy * dzdy));
        const aspectRad = Math.atan2(dzdy, -dzdx);

        const hillshade =
          255 *
          ((Math.cos(altitudeRad) * Math.cos(slopeRad) +
            Math.sin(altitudeRad) *
              Math.sin(slopeRad) *
              Math.cos(azimuthRad - aspectRad)) *
            0.5 +
            0.5);

        hillshadeData[row * width + col] = Math.max(0, Math.min(255, hillshade));
      }
    }

    const band: RasterBand = {
      data: hillshadeData,
      statistics: this.calculateStatistics(hillshadeData),
    };

    return {
      ...dem,
      bands: [band],
    };
  }

  /**
   * Calculate curvature
   */
  static curvature(dem: RasterData): RasterData {
    const { width, height, bands, pixelSize } = dem;
    const elevation = bands[0].data;
    const curvatureData = new Float32Array(width * height);

    for (let row = 2; row < height - 2; row++) {
      for (let col = 2; col < width - 2; col++) {
        // Second derivatives
        const d2zdx2 = this.secondDerivative(
          elevation,
          row,
          col,
          width,
          pixelSize.x,
          'x'
        );

        const d2zdy2 = this.secondDerivative(
          elevation,
          row,
          col,
          width,
          pixelSize.y,
          'y'
        );

        const curvature = -2 * (d2zdx2 + d2zdy2) * 100;
        curvatureData[row * width + col] = curvature;
      }
    }

    const band: RasterBand = {
      data: curvatureData,
      statistics: this.calculateStatistics(curvatureData),
    };

    return {
      ...dem,
      bands: [band],
    };
  }

  /**
   * Calculate elevation profile along a line
   */
  static elevationProfile(
    dem: RasterData,
    line: Position[]
  ): ElevationProfile {
    const distances: number[] = [];
    const elevations: number[] = [];
    let totalDistance = 0;

    // Sample elevations along the line
    for (let i = 0; i < line.length; i++) {
      const pos = line[i];
      const elevation = this.sampleElevation(dem, pos);

      distances.push(totalDistance);
      elevations.push(elevation);

      if (i < line.length - 1) {
        totalDistance += GeometryFactory.distance(pos, line[i + 1]);
      }
    }

    // Calculate statistics
    let gain = 0;
    let loss = 0;
    let minElevation = Infinity;
    let maxElevation = -Infinity;

    for (let i = 0; i < elevations.length; i++) {
      minElevation = Math.min(minElevation, elevations[i]);
      maxElevation = Math.max(maxElevation, elevations[i]);

      if (i > 0) {
        const change = elevations[i] - elevations[i - 1];
        if (change > 0) gain += change;
        else loss += Math.abs(change);
      }
    }

    return {
      distances,
      elevations,
      totalDistance,
      gain,
      loss,
      minElevation,
      maxElevation,
    };
  }

  /**
   * Sample elevation at a position
   */
  private static sampleElevation(dem: RasterData, pos: Position): number {
    const { bounds, width, height, bands, pixelSize } = dem;
    const elevation = bands[0].data;

    const col = Math.floor((pos[0] - bounds.minX) / pixelSize.x);
    const row = Math.floor((bounds.maxY - pos[1]) / pixelSize.y);

    if (col < 0 || col >= width || row < 0 || row >= height) {
      return 0;
    }

    return elevation[row * width + col];
  }

  /**
   * Calculate gradient in x or y direction
   */
  private static calculateGradient(
    data: Float32Array | Uint8Array | Uint16Array,
    row: number,
    col: number,
    width: number,
    height: number,
    pixelSize: number,
    direction: 'x' | 'y'
  ): number {
    if (direction === 'x') {
      const left = data[row * width + (col - 1)];
      const right = data[row * width + (col + 1)];
      return (right - left) / (2 * pixelSize);
    } else {
      const top = data[(row - 1) * width + col];
      const bottom = data[(row + 1) * width + col];
      return (top - bottom) / (2 * pixelSize);
    }
  }

  /**
   * Calculate second derivative
   */
  private static secondDerivative(
    data: Float32Array | Uint8Array | Uint16Array,
    row: number,
    col: number,
    width: number,
    pixelSize: number,
    direction: 'x' | 'y'
  ): number {
    const center = data[row * width + col];

    if (direction === 'x') {
      const left = data[row * width + (col - 1)];
      const right = data[row * width + (col + 1)];
      return (left - 2 * center + right) / (pixelSize * pixelSize);
    } else {
      const top = data[(row - 1) * width + col];
      const bottom = data[(row + 1) * width + col];
      return (top - 2 * center + bottom) / (pixelSize * pixelSize);
    }
  }

  /**
   * Calculate flow direction
   */
  static flowDirection(dem: RasterData): RasterData {
    const { width, height, bands } = dem;
    const elevation = bands[0].data;
    const flowData = new Uint8Array(width * height);

    // D8 flow direction (8 directions)
    const directions = [
      [0, -1], // North: 1
      [1, -1], // NE: 2
      [1, 0],  // East: 4
      [1, 1],  // SE: 8
      [0, 1],  // South: 16
      [-1, 1], // SW: 32
      [-1, 0], // West: 64
      [-1, -1], // NW: 128
    ];

    for (let row = 1; row < height - 1; row++) {
      for (let col = 1; col < width - 1; col++) {
        const centerElev = elevation[row * width + col];
        let maxSlope = -Infinity;
        let flowDir = 0;

        for (let i = 0; i < directions.length; i++) {
          const [dx, dy] = directions[i];
          const neighborRow = row + dy;
          const neighborCol = col + dx;
          const neighborElev = elevation[neighborRow * width + neighborCol];

          const slope = (centerElev - neighborElev) / Math.sqrt(dx * dx + dy * dy);

          if (slope > maxSlope) {
            maxSlope = slope;
            flowDir = Math.pow(2, i);
          }
        }

        flowData[row * width + col] = flowDir;
      }
    }

    const band: RasterBand = {
      data: flowData,
      statistics: this.calculateStatistics(flowData),
    };

    return {
      ...dem,
      bands: [band],
    };
  }

  /**
   * Calculate flow accumulation
   */
  static flowAccumulation(flowDirection: RasterData): RasterData {
    const { width, height } = flowDirection;
    const flowDir = flowDirection.bands[0].data;
    const accumData = new Float32Array(width * height);

    // Initialize all cells with 1
    accumData.fill(1);

    // Process cells from highest to lowest elevation
    // Simplified - would need actual elevation sorting
    for (let row = height - 2; row > 0; row--) {
      for (let col = width - 2; col > 0; col--) {
        const dir = flowDir[row * width + col];

        // Determine downstream cell based on flow direction
        const { drow, dcol } = this.getFlowOffset(dir);
        const downRow = row + drow;
        const downCol = col + dcol;

        if (downRow >= 0 && downRow < height && downCol >= 0 && downCol < width) {
          accumData[downRow * width + downCol] += accumData[row * width + col];
        }
      }
    }

    const band: RasterBand = {
      data: accumData,
      statistics: this.calculateStatistics(accumData),
    };

    return {
      ...flowDirection,
      bands: [band],
    };
  }

  /**
   * Get flow offset from direction code
   */
  private static getFlowOffset(direction: number): { drow: number; dcol: number } {
    const offsets: Record<number, { drow: number; dcol: number }> = {
      1: { drow: -1, dcol: 0 },   // North
      2: { drow: -1, dcol: 1 },   // NE
      4: { drow: 0, dcol: 1 },    // East
      8: { drow: 1, dcol: 1 },    // SE
      16: { drow: 1, dcol: 0 },   // South
      32: { drow: 1, dcol: -1 },  // SW
      64: { drow: 0, dcol: -1 },  // West
      128: { drow: -1, dcol: -1 }, // NW
    };

    return offsets[direction] || { drow: 0, dcol: 0 };
  }

  /**
   * Calculate statistics
   */
  private static calculateStatistics(
    data: Float32Array | Uint8Array | Uint16Array
  ): any {
    let min = Infinity;
    let max = -Infinity;
    let sum = 0;
    let count = 0;

    for (const value of data) {
      if (isFinite(value)) {
        min = Math.min(min, value);
        max = Math.max(max, value);
        sum += value;
        count++;
      }
    }

    const mean = sum / count;
    let sumSquaredDiff = 0;

    for (const value of data) {
      if (isFinite(value)) {
        sumSquaredDiff += Math.pow(value - mean, 2);
      }
    }

    const stdDev = Math.sqrt(sumSquaredDiff / count);

    return { min, max, mean, stdDev, count };
  }
}
