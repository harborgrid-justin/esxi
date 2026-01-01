/**
 * Raster Mosaic
 * Combine multiple rasters into a single mosaic
 */

import { RasterData, RasterBand, Bounds } from '../types';

export type MosaicMethod = 'first' | 'last' | 'min' | 'max' | 'mean' | 'blend';

export class RasterMosaic {
  /**
   * Create mosaic from multiple rasters
   */
  static mosaic(
    rasters: RasterData[],
    method: MosaicMethod = 'first'
  ): RasterData {
    if (rasters.length === 0) {
      throw new Error('No rasters provided for mosaic');
    }

    // Calculate combined bounds
    const bounds = this.calculateCombinedBounds(rasters);

    // Use cell size from first raster
    const pixelSize = rasters[0].pixelSize;

    const width = Math.ceil((bounds.maxX - bounds.minX) / pixelSize.x);
    const height = Math.ceil((bounds.maxY - bounds.minY) / pixelSize.y);

    // Create output arrays
    const numBands = rasters[0].bands.length;
    const outputBands: RasterBand[] = [];

    for (let b = 0; b < numBands; b++) {
      const data = new Float32Array(width * height);
      const counts = new Uint8Array(width * height); // For averaging

      // Merge rasters
      for (const raster of rasters) {
        this.mergeRaster(
          raster,
          b,
          bounds,
          pixelSize,
          width,
          height,
          data,
          counts,
          method
        );
      }

      // Finalize values for mean method
      if (method === 'mean') {
        for (let i = 0; i < data.length; i++) {
          if (counts[i] > 0) {
            data[i] /= counts[i];
          }
        }
      }

      outputBands.push({
        data,
        statistics: this.calculateStatistics(data),
      });
    }

    return {
      width,
      height,
      bands: outputBands,
      bounds,
      pixelSize,
    };
  }

  /**
   * Merge a single raster into the output
   */
  private static mergeRaster(
    raster: RasterData,
    bandIndex: number,
    outputBounds: Bounds,
    pixelSize: { x: number; y: number },
    outputWidth: number,
    outputHeight: number,
    outputData: Float32Array,
    counts: Uint8Array,
    method: MosaicMethod
  ): void {
    const { bounds: rasterBounds, width, height, bands } = raster;
    const inputData = bands[bandIndex].data;

    // Calculate overlap region
    const overlapMinX = Math.max(outputBounds.minX, rasterBounds.minX);
    const overlapMaxX = Math.min(outputBounds.maxX, rasterBounds.maxX);
    const overlapMinY = Math.max(outputBounds.minY, rasterBounds.minY);
    const overlapMaxY = Math.min(outputBounds.maxY, rasterBounds.maxY);

    if (overlapMinX >= overlapMaxX || overlapMinY >= overlapMaxY) {
      return; // No overlap
    }

    // Process overlap region
    const startCol = Math.floor((overlapMinX - outputBounds.minX) / pixelSize.x);
    const endCol = Math.ceil((overlapMaxX - outputBounds.minX) / pixelSize.x);
    const startRow = Math.floor((outputBounds.maxY - overlapMaxY) / pixelSize.y);
    const endRow = Math.ceil((outputBounds.maxY - overlapMinY) / pixelSize.y);

    for (let outRow = startRow; outRow < endRow; outRow++) {
      for (let outCol = startCol; outCol < endCol; outCol++) {
        if (outRow < 0 || outRow >= outputHeight || outCol < 0 || outCol >= outputWidth) {
          continue;
        }

        // Calculate corresponding position in input raster
        const x = outputBounds.minX + outCol * pixelSize.x + pixelSize.x / 2;
        const y = outputBounds.maxY - outRow * pixelSize.y - pixelSize.y / 2;

        const inCol = Math.floor((x - rasterBounds.minX) / pixelSize.x);
        const inRow = Math.floor((rasterBounds.maxY - y) / pixelSize.y);

        if (inRow >= 0 && inRow < height && inCol >= 0 && inCol < width) {
          const inIdx = inRow * width + inCol;
          const outIdx = outRow * outputWidth + outCol;
          const value = inputData[inIdx];

          if (isFinite(value)) {
            switch (method) {
              case 'first':
                if (counts[outIdx] === 0) {
                  outputData[outIdx] = value;
                  counts[outIdx] = 1;
                }
                break;

              case 'last':
                outputData[outIdx] = value;
                counts[outIdx] = 1;
                break;

              case 'min':
                outputData[outIdx] =
                  counts[outIdx] === 0
                    ? value
                    : Math.min(outputData[outIdx], value);
                counts[outIdx] = 1;
                break;

              case 'max':
                outputData[outIdx] =
                  counts[outIdx] === 0
                    ? value
                    : Math.max(outputData[outIdx], value);
                counts[outIdx] = 1;
                break;

              case 'mean':
                outputData[outIdx] += value;
                counts[outIdx]++;
                break;

              case 'blend':
                // Weighted blend based on distance from edge
                const weight = this.calculateBlendWeight(
                  inRow,
                  inCol,
                  height,
                  width
                );
                if (counts[outIdx] === 0) {
                  outputData[outIdx] = value * weight;
                  counts[outIdx] = 1;
                } else {
                  outputData[outIdx] += value * weight;
                  counts[outIdx]++;
                }
                break;
            }
          }
        }
      }
    }
  }

  /**
   * Calculate blend weight based on distance from edge
   */
  private static calculateBlendWeight(
    row: number,
    col: number,
    height: number,
    width: number
  ): number {
    const distFromTop = row;
    const distFromBottom = height - row - 1;
    const distFromLeft = col;
    const distFromRight = width - col - 1;

    const minDist = Math.min(
      distFromTop,
      distFromBottom,
      distFromLeft,
      distFromRight
    );

    const edgeWidth = Math.min(height, width) / 10; // 10% edge blend

    return minDist >= edgeWidth ? 1 : minDist / edgeWidth;
  }

  /**
   * Calculate combined bounds of all rasters
   */
  private static calculateCombinedBounds(rasters: RasterData[]): Bounds {
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    for (const raster of rasters) {
      minX = Math.min(minX, raster.bounds.minX);
      minY = Math.min(minY, raster.bounds.minY);
      maxX = Math.max(maxX, raster.bounds.maxX);
      maxY = Math.max(maxY, raster.bounds.maxY);
    }

    return { minX, minY, maxX, maxY };
  }

  /**
   * Resample raster to match target resolution
   */
  static resample(
    raster: RasterData,
    targetCellSize: { x: number; y: number },
    method: 'nearest' | 'bilinear' | 'cubic' = 'bilinear'
  ): RasterData {
    const { bounds } = raster;
    const width = Math.ceil((bounds.maxX - bounds.minX) / targetCellSize.x);
    const height = Math.ceil((bounds.maxY - bounds.minY) / targetCellSize.y);

    const outputBands: RasterBand[] = [];

    for (const band of raster.bands) {
      const data = new Float32Array(width * height);

      for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
          const x = bounds.minX + col * targetCellSize.x + targetCellSize.x / 2;
          const y = bounds.maxY - row * targetCellSize.y - targetCellSize.y / 2;

          let value: number;

          if (method === 'nearest') {
            value = this.sampleNearest(raster, band, x, y);
          } else if (method === 'bilinear') {
            value = this.sampleBilinear(raster, band, x, y);
          } else {
            value = this.sampleCubic(raster, band, x, y);
          }

          data[row * width + col] = value;
        }
      }

      outputBands.push({
        data,
        statistics: this.calculateStatistics(data),
      });
    }

    return {
      width,
      height,
      bands: outputBands,
      bounds,
      pixelSize: targetCellSize,
    };
  }

  /**
   * Nearest neighbor sampling
   */
  private static sampleNearest(
    raster: RasterData,
    band: RasterBand,
    x: number,
    y: number
  ): number {
    const { bounds, width, height, pixelSize } = raster;
    const col = Math.floor((x - bounds.minX) / pixelSize.x);
    const row = Math.floor((bounds.maxY - y) / pixelSize.y);

    if (row >= 0 && row < height && col >= 0 && col < width) {
      return band.data[row * width + col];
    }

    return 0;
  }

  /**
   * Bilinear interpolation sampling
   */
  private static sampleBilinear(
    raster: RasterData,
    band: RasterBand,
    x: number,
    y: number
  ): number {
    const { bounds, width, height, pixelSize } = raster;
    const colF = (x - bounds.minX) / pixelSize.x;
    const rowF = (bounds.maxY - y) / pixelSize.y;

    const col = Math.floor(colF);
    const row = Math.floor(rowF);

    const dx = colF - col;
    const dy = rowF - row;

    if (row >= 0 && row < height - 1 && col >= 0 && col < width - 1) {
      const v00 = band.data[row * width + col];
      const v10 = band.data[row * width + col + 1];
      const v01 = band.data[(row + 1) * width + col];
      const v11 = band.data[(row + 1) * width + col + 1];

      const v0 = v00 * (1 - dx) + v10 * dx;
      const v1 = v01 * (1 - dx) + v11 * dx;

      return v0 * (1 - dy) + v1 * dy;
    }

    return this.sampleNearest(raster, band, x, y);
  }

  /**
   * Cubic convolution sampling
   */
  private static sampleCubic(
    raster: RasterData,
    band: RasterBand,
    x: number,
    y: number
  ): number {
    // Simplified - would use full cubic convolution kernel
    return this.sampleBilinear(raster, band, x, y);
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
