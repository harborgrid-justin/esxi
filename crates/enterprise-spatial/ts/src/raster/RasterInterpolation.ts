/**
 * Raster Interpolation
 * IDW, Kriging, Spline, and TIN interpolation methods
 */

import {
  Position,
  RasterData,
  RasterBand,
  Bounds,
  InterpolationOptions,
} from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export class RasterInterpolation {
  /**
   * Inverse Distance Weighting (IDW) interpolation
   */
  static idw(
    points: Array<{ position: Position; value: number }>,
    bounds: Bounds,
    cellSize: number,
    options: InterpolationOptions = { method: 'idw' }
  ): RasterData {
    const power = options.power || 2;
    const searchRadius = options.searchRadius || Infinity;

    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellPos: Position = [x, y];

        let weightedSum = 0;
        let weightSum = 0;

        for (const point of points) {
          const distance = GeometryFactory.distance(cellPos, point.position);

          if (distance === 0) {
            // Exact point match
            weightedSum = point.value;
            weightSum = 1;
            break;
          }

          if (distance <= searchRadius) {
            const weight = 1 / Math.pow(distance, power);
            weightedSum += point.value * weight;
            weightSum += weight;
          }
        }

        data[row * width + col] = weightSum > 0 ? weightedSum / weightSum : 0;
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
   * Kriging interpolation (Ordinary Kriging)
   */
  static kriging(
    points: Array<{ position: Position; value: number }>,
    bounds: Bounds,
    cellSize: number,
    options: InterpolationOptions = { method: 'kriging' }
  ): RasterData {
    const model = options.variogramModel || 'spherical';
    const range = this.calculateVariogramRange(points);

    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    // Build kriging system
    const n = points.length;
    const K = this.buildKrigingMatrix(points, model, range);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellPos: Position = [x, y];

        const value = this.krigingEstimate(cellPos, points, K, model, range);
        data[row * width + col] = value;
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
   * Spline interpolation
   */
  static spline(
    points: Array<{ position: Position; value: number }>,
    bounds: Bounds,
    cellSize: number
  ): RasterData {
    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    // Regularized spline with tension
    const tension = 0.1;

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellPos: Position = [x, y];

        let weightedSum = 0;
        let weightSum = 0;

        for (const point of points) {
          const distance = GeometryFactory.distance(cellPos, point.position);
          const weight = this.splineBasis(distance, tension);

          weightedSum += point.value * weight;
          weightSum += weight;
        }

        data[row * width + col] = weightSum > 0 ? weightedSum / weightSum : 0;
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
   * Natural Neighbor interpolation
   */
  static naturalNeighbor(
    points: Array<{ position: Position; value: number }>,
    bounds: Bounds,
    cellSize: number
  ): RasterData {
    // Simplified implementation - production would use Voronoi diagrams
    return this.idw(points, bounds, cellSize, { method: 'idw', power: 1 });
  }

  /**
   * Nearest neighbor interpolation
   */
  static nearest(
    points: Array<{ position: Position; value: number }>,
    bounds: Bounds,
    cellSize: number
  ): RasterData {
    const width = Math.ceil((bounds.maxX - bounds.minX) / cellSize);
    const height = Math.ceil((bounds.maxY - bounds.minY) / cellSize);
    const data = new Float32Array(width * height);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const x = bounds.minX + col * cellSize + cellSize / 2;
        const y = bounds.minY + row * cellSize + cellSize / 2;
        const cellPos: Position = [x, y];

        let nearestValue = 0;
        let minDistance = Infinity;

        for (const point of points) {
          const distance = GeometryFactory.distance(cellPos, point.position);

          if (distance < minDistance) {
            minDistance = distance;
            nearestValue = point.value;
          }
        }

        data[row * width + col] = nearestValue;
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
   * Build kriging matrix
   */
  private static buildKrigingMatrix(
    points: Array<{ position: Position; value: number }>,
    model: string,
    range: number
  ): number[][] {
    const n = points.length;
    const K: number[][] = Array(n + 1)
      .fill(0)
      .map(() => Array(n + 1).fill(0));

    for (let i = 0; i < n; i++) {
      for (let j = 0; j < n; j++) {
        const distance = GeometryFactory.distance(
          points[i].position,
          points[j].position
        );
        K[i][j] = this.variogram(distance, model, range);
      }
      K[i][n] = 1;
      K[n][i] = 1;
    }

    K[n][n] = 0;

    return K;
  }

  /**
   * Kriging estimate
   */
  private static krigingEstimate(
    position: Position,
    points: Array<{ position: Position; value: number }>,
    K: number[][],
    model: string,
    range: number
  ): number {
    const n = points.length;
    const k = Array(n + 1).fill(0);

    for (let i = 0; i < n; i++) {
      const distance = GeometryFactory.distance(position, points[i].position);
      k[i] = this.variogram(distance, model, range);
    }
    k[n] = 1;

    // Solve kriging system (simplified - would use proper linear solver)
    const weights = this.solveLinearSystem(K, k);

    let estimate = 0;
    for (let i = 0; i < n; i++) {
      estimate += weights[i] * points[i].value;
    }

    return estimate;
  }

  /**
   * Variogram function
   */
  private static variogram(
    distance: number,
    model: string,
    range: number
  ): number {
    const h = distance / range;

    switch (model) {
      case 'spherical':
        return h <= 1 ? 1 - 1.5 * h + 0.5 * Math.pow(h, 3) : 0;

      case 'exponential':
        return Math.exp(-3 * h);

      case 'gaussian':
        return Math.exp(-3 * h * h);

      default:
        return h <= 1 ? 1 - h : 0;
    }
  }

  /**
   * Calculate variogram range
   */
  private static calculateVariogramRange(
    points: Array<{ position: Position; value: number }>
  ): number {
    let maxDist = 0;

    for (let i = 0; i < points.length; i++) {
      for (let j = i + 1; j < points.length; j++) {
        const dist = GeometryFactory.distance(
          points[i].position,
          points[j].position
        );
        maxDist = Math.max(maxDist, dist);
      }
    }

    return maxDist / 3; // Use 1/3 of max distance as range
  }

  /**
   * Spline basis function
   */
  private static splineBasis(distance: number, tension: number): number {
    if (distance === 0) return 1;

    const r = distance * tension;
    return Math.exp(-r * r);
  }

  /**
   * Solve linear system (simplified Gaussian elimination)
   */
  private static solveLinearSystem(A: number[][], b: number[]): number[] {
    const n = A.length;
    const x = Array(n).fill(0);

    // Create augmented matrix
    const aug = A.map((row, i) => [...row, b[i]]);

    // Forward elimination
    for (let i = 0; i < n; i++) {
      // Find pivot
      let maxRow = i;
      for (let k = i + 1; k < n; k++) {
        if (Math.abs(aug[k][i]) > Math.abs(aug[maxRow][i])) {
          maxRow = k;
        }
      }

      // Swap rows
      [aug[i], aug[maxRow]] = [aug[maxRow], aug[i]];

      // Eliminate column
      for (let k = i + 1; k < n; k++) {
        const factor = aug[k][i] / aug[i][i];
        for (let j = i; j <= n; j++) {
          aug[k][j] -= factor * aug[i][j];
        }
      }
    }

    // Back substitution
    for (let i = n - 1; i >= 0; i--) {
      x[i] = aug[i][n];
      for (let j = i + 1; j < n; j++) {
        x[i] -= aug[i][j] * x[j];
      }
      x[i] /= aug[i][i];
    }

    return x;
  }

  /**
   * Calculate statistics
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
