/**
 * Density Analysis
 * Kernel density estimation and point density calculations
 */

import {
  Position,
  Feature,
  RasterData,
  RasterBand,
  Bounds,
} from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export class DensityAnalysis {
  /**
   * Calculate kernel density estimation
   */
  static kernelDensity(
    points: Position[],
    bounds: Bounds,
    cellSize: number,
    searchRadius: number,
    kernel: 'gaussian' | 'quartic' | 'triangular' | 'uniform' = 'gaussian'
  ): RasterData {
    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    // Calculate density for each cell
    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellCenter: Position = [x, y];

        let density = 0;

        // Sum contributions from all points
        for (const point of points) {
          const distance = GeometryFactory.distance(cellCenter, point);

          if (distance <= searchRadius) {
            density += this.kernelFunction(
              distance,
              searchRadius,
              kernel
            );
          }
        }

        data[row * width + col] = density;
      }
    }

    const band: RasterBand = {
      data,
      statistics: this.calculateStatistics(data),
    };

    return {
      width,
      height,
      bands: [band],
      bounds,
      pixelSize: { x: cellSize, y: cellSize },
    };
  }

  /**
   * Calculate point density
   */
  static pointDensity(
    points: Position[],
    bounds: Bounds,
    cellSize: number,
    searchRadius: number,
    areaUnits: 'square-meters' | 'square-kilometers' | 'square-miles' = 'square-meters'
  ): RasterData {
    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    const cellArea = cellSize * cellSize;
    const areaMultiplier = this.getAreaMultiplier(areaUnits);
    const searchArea = Math.PI * searchRadius * searchRadius * areaMultiplier;

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellCenter: Position = [x, y];

        let count = 0;
        for (const point of points) {
          const distance = GeometryFactory.distance(cellCenter, point);
          if (distance <= searchRadius) {
            count++;
          }
        }

        // Density = count / search area
        data[row * width + col] = count / searchArea;
      }
    }

    const band: RasterBand = {
      data,
      statistics: this.calculateStatistics(data),
    };

    return {
      width,
      height,
      bands: [band],
      bounds,
      pixelSize: { x: cellSize, y: cellSize },
    };
  }

  /**
   * Calculate line density
   */
  static lineDensity(
    lines: Position[][],
    bounds: Bounds,
    cellSize: number,
    searchRadius: number
  ): RasterData {
    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellCenter: Position = [x, y];

        let totalLength = 0;

        for (const line of lines) {
          // Calculate length of line segments within search radius
          for (let i = 0; i < line.length - 1; i++) {
            const segmentLength = this.segmentLengthInRadius(
              line[i],
              line[i + 1],
              cellCenter,
              searchRadius
            );
            totalLength += segmentLength;
          }
        }

        const searchArea = Math.PI * searchRadius * searchRadius;
        data[row * width + col] = totalLength / searchArea;
      }
    }

    const band: RasterBand = {
      data,
      statistics: this.calculateStatistics(data),
    };

    return {
      width,
      height,
      bands: [band],
      bounds,
      pixelSize: { x: cellSize, y: cellSize },
    };
  }

  /**
   * Calculate heat map
   */
  static heatMap(
    points: Array<{ position: Position; weight?: number }>,
    bounds: Bounds,
    cellSize: number,
    searchRadius: number
  ): RasterData {
    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellCenter: Position = [x, y];

        let heat = 0;

        for (const point of points) {
          const distance = GeometryFactory.distance(cellCenter, point.position);

          if (distance <= searchRadius) {
            const weight = point.weight || 1;
            const influence = this.kernelFunction(distance, searchRadius, 'gaussian');
            heat += weight * influence;
          }
        }

        data[row * width + col] = heat;
      }
    }

    // Normalize to 0-1 range
    const max = Math.max(...Array.from(data));
    if (max > 0) {
      for (let i = 0; i < data.length; i++) {
        data[i] /= max;
      }
    }

    const band: RasterBand = {
      data,
      statistics: this.calculateStatistics(data),
    };

    return {
      width,
      height,
      bands: [band],
      bounds,
      pixelSize: { x: cellSize, y: cellSize },
    };
  }

  /**
   * Kernel function implementations
   */
  private static kernelFunction(
    distance: number,
    bandwidth: number,
    type: 'gaussian' | 'quartic' | 'triangular' | 'uniform'
  ): number {
    const u = distance / bandwidth;

    switch (type) {
      case 'gaussian':
        return (1 / Math.sqrt(2 * Math.PI)) * Math.exp(-0.5 * u * u);

      case 'quartic':
        return u <= 1 ? (15 / 16) * Math.pow(1 - u * u, 2) : 0;

      case 'triangular':
        return u <= 1 ? 1 - u : 0;

      case 'uniform':
        return u <= 1 ? 0.5 : 0;

      default:
        return 0;
    }
  }

  /**
   * Calculate segment length within radius
   */
  private static segmentLengthInRadius(
    start: Position,
    end: Position,
    center: Position,
    radius: number
  ): number {
    const distStart = GeometryFactory.distance(start, center);
    const distEnd = GeometryFactory.distance(end, center);
    const segmentLength = GeometryFactory.distance(start, end);

    // Both endpoints inside radius
    if (distStart <= radius && distEnd <= radius) {
      return segmentLength;
    }

    // Both endpoints outside radius
    if (distStart > radius && distEnd > radius) {
      return 0;
    }

    // One endpoint inside, one outside
    // Approximate the portion inside
    const ratio = distStart <= radius
      ? (radius - distStart) / (distEnd - distStart)
      : (radius - distEnd) / (distStart - distEnd);

    return segmentLength * Math.min(1, Math.max(0, ratio));
  }

  /**
   * Calculate statistics for raster data
   */
  private static calculateStatistics(data: Float32Array): any {
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

    // Calculate standard deviation
    let sumSquaredDiff = 0;
    for (const value of data) {
      if (isFinite(value)) {
        sumSquaredDiff += Math.pow(value - mean, 2);
      }
    }
    const stdDev = Math.sqrt(sumSquaredDiff / count);

    return { min, max, mean, stdDev, count };
  }

  /**
   * Get area multiplier based on units
   */
  private static getAreaMultiplier(units: string): number {
    switch (units) {
      case 'square-meters':
        return 1;
      case 'square-kilometers':
        return 1000000;
      case 'square-miles':
        return 2589988;
      default:
        return 1;
    }
  }

  /**
   * Calculate local density (density at each point)
   */
  static localDensity(
    points: Position[],
    searchRadius: number
  ): number[] {
    return points.map((point) => {
      let count = 0;
      for (const other of points) {
        if (point !== other) {
          const distance = GeometryFactory.distance(point, other);
          if (distance <= searchRadius) {
            count++;
          }
        }
      }
      return count / (Math.PI * searchRadius * searchRadius);
    });
  }

  /**
   * Find density hotspots using Getis-Ord Gi* statistic
   */
  static hotspots(
    points: Position[],
    values: number[],
    searchRadius: number
  ): Array<{ position: Position; score: number; pValue: number }> {
    const n = points.length;
    const mean = values.reduce((a, b) => a + b, 0) / n;

    const variance =
      values.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) / n;
    const stdDev = Math.sqrt(variance);

    return points.map((point, i) => {
      let sumValues = 0;
      let sumWeights = 0;

      for (let j = 0; j < n; j++) {
        const distance = GeometryFactory.distance(point, points[j]);
        if (distance <= searchRadius) {
          sumValues += values[j];
          sumWeights += 1;
        }
      }

      const localMean = sumValues / sumWeights;
      const zScore = (localMean - mean) / (stdDev / Math.sqrt(sumWeights));

      // Calculate p-value (simplified)
      const pValue = 1 - this.normalCDF(Math.abs(zScore));

      return {
        position: point,
        score: zScore,
        pValue,
      };
    });
  }

  /**
   * Normal cumulative distribution function (approximation)
   */
  private static normalCDF(x: number): number {
    const t = 1 / (1 + 0.2316419 * Math.abs(x));
    const d = 0.3989423 * Math.exp(-x * x / 2);
    const prob =
      d *
      t *
      (0.3193815 +
        t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
    return x > 0 ? 1 - prob : prob;
  }
}
