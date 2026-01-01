/**
 * Raster Calculator
 * Map algebra and raster calculations
 */

import { RasterData, RasterBand } from '../types';

export type RasterOperation =
  | 'add'
  | 'subtract'
  | 'multiply'
  | 'divide'
  | 'power'
  | 'sqrt'
  | 'abs'
  | 'min'
  | 'max'
  | 'mean';

export class RasterCalculator {
  /**
   * Perform raster calculation with expression
   */
  static calculate(
    expression: string,
    rasters: Record<string, RasterData>
  ): RasterData {
    // Parse and evaluate expression
    // Simplified - production would use proper expression parser
    const result = this.evaluateExpression(expression, rasters);
    return result;
  }

  /**
   * Add two rasters
   */
  static add(raster1: RasterData, raster2: RasterData): RasterData {
    return this.binaryOperation(raster1, raster2, (a, b) => a + b);
  }

  /**
   * Subtract rasters
   */
  static subtract(raster1: RasterData, raster2: RasterData): RasterData {
    return this.binaryOperation(raster1, raster2, (a, b) => a - b);
  }

  /**
   * Multiply rasters
   */
  static multiply(raster1: RasterData, raster2: RasterData): RasterData {
    return this.binaryOperation(raster1, raster2, (a, b) => a * b);
  }

  /**
   * Divide rasters
   */
  static divide(raster1: RasterData, raster2: RasterData): RasterData {
    return this.binaryOperation(raster1, raster2, (a, b) =>
      b !== 0 ? a / b : 0
    );
  }

  /**
   * Scalar addition
   */
  static addScalar(raster: RasterData, value: number): RasterData {
    return this.unaryOperation(raster, (a) => a + value);
  }

  /**
   * Scalar multiplication
   */
  static multiplyScalar(raster: RasterData, value: number): RasterData {
    return this.unaryOperation(raster, (a) => a * value);
  }

  /**
   * Power operation
   */
  static power(raster: RasterData, exponent: number): RasterData {
    return this.unaryOperation(raster, (a) => Math.pow(a, exponent));
  }

  /**
   * Square root
   */
  static sqrt(raster: RasterData): RasterData {
    return this.unaryOperation(raster, (a) => Math.sqrt(Math.max(0, a)));
  }

  /**
   * Absolute value
   */
  static abs(raster: RasterData): RasterData {
    return this.unaryOperation(raster, Math.abs);
  }

  /**
   * Logarithm
   */
  static log(raster: RasterData, base = Math.E): RasterData {
    const factor = Math.log(base);
    return this.unaryOperation(raster, (a) =>
      a > 0 ? Math.log(a) / factor : 0
    );
  }

  /**
   * Exponential
   */
  static exp(raster: RasterData): RasterData {
    return this.unaryOperation(raster, Math.exp);
  }

  /**
   * Sine
   */
  static sin(raster: RasterData): RasterData {
    return this.unaryOperation(raster, Math.sin);
  }

  /**
   * Cosine
   */
  static cos(raster: RasterData): RasterData {
    return this.unaryOperation(raster, Math.cos);
  }

  /**
   * Tangent
   */
  static tan(raster: RasterData): RasterData {
    return this.unaryOperation(raster, Math.tan);
  }

  /**
   * Cell-by-cell minimum
   */
  static min(raster1: RasterData, raster2: RasterData): RasterData {
    return this.binaryOperation(raster1, raster2, Math.min);
  }

  /**
   * Cell-by-cell maximum
   */
  static max(raster1: RasterData, raster2: RasterData): RasterData {
    return this.binaryOperation(raster1, raster2, Math.max);
  }

  /**
   * Reclassify raster values
   */
  static reclassify(
    raster: RasterData,
    ranges: Array<{ min: number; max: number; value: number }>
  ): RasterData {
    return this.unaryOperation(raster, (value) => {
      for (const range of ranges) {
        if (value >= range.min && value <= range.max) {
          return range.value;
        }
      }
      return value;
    });
  }

  /**
   * Conditional operation (Con)
   */
  static con(
    condition: RasterData,
    trueRaster: RasterData,
    falseRaster: RasterData,
    threshold = 0
  ): RasterData {
    this.validateDimensions(condition, trueRaster);
    this.validateDimensions(condition, falseRaster);

    const { width, height } = condition;
    const condData = condition.bands[0].data;
    const trueData = trueRaster.bands[0].data;
    const falseData = falseRaster.bands[0].data;
    const resultData = new Float32Array(width * height);

    for (let i = 0; i < resultData.length; i++) {
      resultData[i] = condData[i] > threshold ? trueData[i] : falseData[i];
    }

    const band: RasterBand = {
      data: resultData,
      statistics: this.calculateStatistics(resultData),
    };

    return {
      ...condition,
      bands: [band],
    };
  }

  /**
   * Set null (mask) values
   */
  static setNull(
    raster: RasterData,
    condition: (value: number) => boolean
  ): RasterData {
    const noDataValue = raster.noDataValue || -9999;

    return this.unaryOperation(raster, (value) =>
      condition(value) ? noDataValue : value
    );
  }

  /**
   * Fill null values
   */
  static fillNull(raster: RasterData, fillValue: number): RasterData {
    const noDataValue = raster.noDataValue || -9999;

    return this.unaryOperation(raster, (value) =>
      value === noDataValue ? fillValue : value
    );
  }

  /**
   * Focal statistics (neighborhood operations)
   */
  static focalStatistics(
    raster: RasterData,
    windowSize: number,
    statistic: 'mean' | 'sum' | 'min' | 'max' | 'std' = 'mean'
  ): RasterData {
    const { width, height, bands } = raster;
    const inputData = bands[0].data;
    const resultData = new Float32Array(width * height);
    const radius = Math.floor(windowSize / 2);

    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        const values: number[] = [];

        // Collect neighborhood values
        for (let dr = -radius; dr <= radius; dr++) {
          for (let dc = -radius; dc <= radius; dc++) {
            const r = row + dr;
            const c = col + dc;

            if (r >= 0 && r < height && c >= 0 && c < width) {
              values.push(inputData[r * width + c]);
            }
          }
        }

        // Calculate statistic
        resultData[row * width + col] = this.calculateStatistic(
          values,
          statistic
        );
      }
    }

    const band: RasterBand = {
      data: resultData,
      statistics: this.calculateStatistics(resultData),
    };

    return {
      ...raster,
      bands: [band],
    };
  }

  /**
   * Zonal statistics
   */
  static zonalStatistics(
    valueRaster: RasterData,
    zoneRaster: RasterData,
    statistic: 'mean' | 'sum' | 'min' | 'max' | 'count' = 'mean'
  ): Map<number, number> {
    this.validateDimensions(valueRaster, zoneRaster);

    const { width, height } = valueRaster;
    const valueData = valueRaster.bands[0].data;
    const zoneData = zoneRaster.bands[0].data;

    const zones = new Map<number, number[]>();

    // Collect values by zone
    for (let i = 0; i < width * height; i++) {
      const zone = zoneData[i];
      const value = valueData[i];

      if (!zones.has(zone)) {
        zones.set(zone, []);
      }

      zones.get(zone)!.push(value);
    }

    // Calculate statistics for each zone
    const results = new Map<number, number>();

    for (const [zone, values] of zones.entries()) {
      results.set(zone, this.calculateStatistic(values, statistic));
    }

    return results;
  }

  /**
   * Binary operation on two rasters
   */
  private static binaryOperation(
    raster1: RasterData,
    raster2: RasterData,
    op: (a: number, b: number) => number
  ): RasterData {
    this.validateDimensions(raster1, raster2);

    const { width, height } = raster1;
    const data1 = raster1.bands[0].data;
    const data2 = raster2.bands[0].data;
    const resultData = new Float32Array(width * height);

    for (let i = 0; i < resultData.length; i++) {
      resultData[i] = op(data1[i], data2[i]);
    }

    const band: RasterBand = {
      data: resultData,
      statistics: this.calculateStatistics(resultData),
    };

    return {
      ...raster1,
      bands: [band],
    };
  }

  /**
   * Unary operation on raster
   */
  private static unaryOperation(
    raster: RasterData,
    op: (a: number) => number
  ): RasterData {
    const { width, height, bands } = raster;
    const inputData = bands[0].data;
    const resultData = new Float32Array(width * height);

    for (let i = 0; i < resultData.length; i++) {
      resultData[i] = op(inputData[i]);
    }

    const band: RasterBand = {
      data: resultData,
      statistics: this.calculateStatistics(resultData),
    };

    return {
      ...raster,
      bands: [band],
    };
  }

  /**
   * Calculate statistic on array of values
   */
  private static calculateStatistic(
    values: number[],
    statistic: string
  ): number {
    if (values.length === 0) return 0;

    switch (statistic) {
      case 'mean':
        return values.reduce((a, b) => a + b, 0) / values.length;
      case 'sum':
        return values.reduce((a, b) => a + b, 0);
      case 'min':
        return Math.min(...values);
      case 'max':
        return Math.max(...values);
      case 'count':
        return values.length;
      case 'std': {
        const mean = values.reduce((a, b) => a + b, 0) / values.length;
        const variance =
          values.reduce((sum, v) => sum + Math.pow(v - mean, 2), 0) /
          values.length;
        return Math.sqrt(variance);
      }
      default:
        return 0;
    }
  }

  /**
   * Evaluate expression (simplified)
   */
  private static evaluateExpression(
    expression: string,
    rasters: Record<string, RasterData>
  ): RasterData {
    // This is a simplified implementation
    // Production would use proper expression parser
    const keys = Object.keys(rasters);
    if (keys.length === 0) {
      throw new Error('No rasters provided');
    }

    return rasters[keys[0]];
  }

  /**
   * Validate that rasters have same dimensions
   */
  private static validateDimensions(
    raster1: RasterData,
    raster2: RasterData
  ): void {
    if (
      raster1.width !== raster2.width ||
      raster1.height !== raster2.height
    ) {
      throw new Error('Rasters must have the same dimensions');
    }
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
