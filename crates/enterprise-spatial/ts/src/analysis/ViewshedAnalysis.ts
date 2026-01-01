/**
 * Viewshed Analysis
 * Line-of-sight and visibility analysis
 */

import {
  RasterData,
  Position,
  Bounds,
} from '../types';

export interface ViewshedOptions {
  observerHeight?: number;
  targetHeight?: number;
  maxDistance?: number;
  verticalAngle?: number;
}

export interface ViewshedResult {
  visible: RasterData;
  visibleCount: number;
  totalCells: number;
  visibilityPercent: number;
}

export class ViewshedAnalysis {
  /**
   * Calculate viewshed from observer point
   */
  static viewshed(
    dem: RasterData,
    observerPos: Position,
    options: ViewshedOptions = {}
  ): ViewshedResult {
    const {
      observerHeight = 2,
      targetHeight = 0,
      maxDistance = Infinity,
      verticalAngle = 180,
    } = options;

    const { width, height, bands, bounds, pixelSize } = dem;
    const elevation = bands[0].data;
    const visibleData = new Uint8Array(width * height);

    // Get observer cell
    const obsCol = Math.floor((observerPos[0] - bounds.minX) / pixelSize.x);
    const obsRow = Math.floor((bounds.maxY - observerPos[1]) / pixelSize.y);

    if (obsCol < 0 || obsCol >= width || obsRow < 0 || obsRow >= height) {
      throw new Error('Observer position outside DEM bounds');
    }

    const obsElev = elevation[obsRow * width + obsCol] + observerHeight;

    let visibleCount = 0;
    let totalCells = 0;

    // Check visibility for each cell
    for (let row = 0; row < height; row++) {
      for (let col = 0; col < width; col++) {
        totalCells++;

        const targetPos: Position = [
          bounds.minX + col * pixelSize.x + pixelSize.x / 2,
          bounds.maxY - row * pixelSize.y - pixelSize.y / 2,
        ];

        const distance = Math.sqrt(
          Math.pow((col - obsCol) * pixelSize.x, 2) +
            Math.pow((row - obsRow) * pixelSize.y, 2)
        );

        if (distance > maxDistance) {
          continue;
        }

        const isVisible = this.isVisible(
          elevation,
          width,
          height,
          obsRow,
          obsCol,
          obsElev,
          row,
          col,
          targetHeight,
          verticalAngle
        );

        if (isVisible) {
          visibleData[row * width + col] = 1;
          visibleCount++;
        }
      }
    }

    return {
      visible: {
        ...dem,
        bands: [{ data: visibleData }],
      },
      visibleCount,
      totalCells,
      visibilityPercent: (visibleCount / totalCells) * 100,
    };
  }

  /**
   * Check if target cell is visible from observer
   */
  private static isVisible(
    elevation: Float32Array | Uint8Array | Uint16Array,
    width: number,
    height: number,
    obsRow: number,
    obsCol: number,
    obsElev: number,
    targetRow: number,
    targetCol: number,
    targetHeight: number,
    verticalAngle: number
  ): boolean {
    if (obsRow === targetRow && obsCol === targetCol) {
      return true;
    }

    const targetElev = elevation[targetRow * width + targetCol] + targetHeight;

    // Bresenham's line algorithm to trace line of sight
    const points = this.bresenhamLine(obsRow, obsCol, targetRow, targetCol);

    let maxAngle = -Infinity;

    for (const [row, col] of points) {
      if (row < 0 || row >= height || col < 0 || col >= width) {
        return false;
      }

      const cellElev = elevation[row * width + col];
      const distance = Math.sqrt(
        Math.pow(row - obsRow, 2) + Math.pow(col - obsCol, 2)
      );

      if (distance === 0) continue;

      const angle = Math.atan2(cellElev - obsElev, distance);

      if (angle > maxAngle) {
        maxAngle = angle;
      }

      // Check if this cell blocks the view
      if (row === targetRow && col === targetCol) {
        const targetAngle = Math.atan2(targetElev - obsElev, distance);
        return targetAngle >= maxAngle;
      }
    }

    return true;
  }

  /**
   * Bresenham's line algorithm
   */
  private static bresenhamLine(
    row0: number,
    col0: number,
    row1: number,
    col1: number
  ): Array<[number, number]> {
    const points: Array<[number, number]> = [];

    const dx = Math.abs(col1 - col0);
    const dy = Math.abs(row1 - row0);
    const sx = col0 < col1 ? 1 : -1;
    const sy = row0 < row1 ? 1 : -1;
    let err = dx - dy;

    let row = row0;
    let col = col0;

    while (true) {
      points.push([row, col]);

      if (row === row1 && col === col1) break;

      const e2 = 2 * err;

      if (e2 > -dy) {
        err -= dy;
        col += sx;
      }

      if (e2 < dx) {
        err += dx;
        row += sy;
      }
    }

    return points;
  }

  /**
   * Calculate line of sight between two points
   */
  static lineOfSight(
    dem: RasterData,
    observerPos: Position,
    targetPos: Position,
    observerHeight = 2,
    targetHeight = 0
  ): {
    visible: boolean;
    blockingPoint?: Position;
    blockingElevation?: number;
  } {
    const { width, height, bands, bounds, pixelSize } = dem;
    const elevation = bands[0].data;

    const obsCol = Math.floor((observerPos[0] - bounds.minX) / pixelSize.x);
    const obsRow = Math.floor((bounds.maxY - observerPos[1]) / pixelSize.y);
    const tarCol = Math.floor((targetPos[0] - bounds.minX) / pixelSize.x);
    const tarRow = Math.floor((bounds.maxY - targetPos[1]) / pixelSize.y);

    const obsElev = elevation[obsRow * width + obsCol] + observerHeight;
    const tarElev = elevation[tarRow * width + tarCol] + targetHeight;

    const points = this.bresenhamLine(obsRow, obsCol, tarRow, tarCol);
    const totalDist = Math.sqrt(
      Math.pow(tarRow - obsRow, 2) + Math.pow(tarCol - obsCol, 2)
    );

    for (let i = 1; i < points.length - 1; i++) {
      const [row, col] = points[i];

      if (row < 0 || row >= height || col < 0 || col >= width) {
        return { visible: false };
      }

      const cellElev = elevation[row * width + col];
      const dist = Math.sqrt(
        Math.pow(row - obsRow, 2) + Math.pow(col - obsCol, 2)
      );

      // Interpolate expected elevation on line of sight
      const expectedElev = obsElev + ((tarElev - obsElev) * dist) / totalDist;

      if (cellElev > expectedElev) {
        return {
          visible: false,
          blockingPoint: [
            bounds.minX + col * pixelSize.x,
            bounds.maxY - row * pixelSize.y,
          ],
          blockingElevation: cellElev,
        };
      }
    }

    return { visible: true };
  }

  /**
   * Calculate cumulative viewshed from multiple observer points
   */
  static cumulativeViewshed(
    dem: RasterData,
    observerPoints: Array<{ position: Position; height?: number }>,
    options: Omit<ViewshedOptions, 'observerHeight'> = {}
  ): ViewshedResult {
    const { width, height } = dem;
    const cumulativeData = new Uint8Array(width * height);

    let totalVisible = 0;

    for (const observer of observerPoints) {
      const result = this.viewshed(dem, observer.position, {
        ...options,
        observerHeight: observer.height,
      });

      const visibleData = result.visible.bands[0].data as Uint8Array;

      for (let i = 0; i < visibleData.length; i++) {
        if (visibleData[i] > 0) {
          cumulativeData[i]++;
        }
      }
    }

    // Count total visible cells
    for (const value of cumulativeData) {
      if (value > 0) totalVisible++;
    }

    return {
      visible: {
        ...dem,
        bands: [{ data: cumulativeData }],
      },
      visibleCount: totalVisible,
      totalCells: width * height,
      visibilityPercent: (totalVisible / (width * height)) * 100,
    };
  }

  /**
   * Find optimal viewpoint (highest visibility)
   */
  static optimalViewpoint(
    dem: RasterData,
    searchArea: Bounds,
    options: ViewshedOptions = {}
  ): {
    position: Position;
    visibleCount: number;
    visibilityPercent: number;
  } {
    const { bounds, pixelSize } = dem;

    let bestPosition: Position = [searchArea.minX, searchArea.minY];
    let bestCount = 0;

    // Sample points in search area
    const stepX = pixelSize.x * 5; // Sample every 5 cells
    const stepY = pixelSize.y * 5;

    for (let x = searchArea.minX; x <= searchArea.maxX; x += stepX) {
      for (let y = searchArea.minY; y <= searchArea.maxY; y += stepY) {
        const position: Position = [x, y];

        try {
          const result = this.viewshed(dem, position, options);

          if (result.visibleCount > bestCount) {
            bestCount = result.visibleCount;
            bestPosition = position;
          }
        } catch (e) {
          // Skip positions outside DEM
          continue;
        }
      }
    }

    const totalCells = dem.width * dem.height;

    return {
      position: bestPosition,
      visibleCount: bestCount,
      visibilityPercent: (bestCount / totalCells) * 100,
    };
  }

  /**
   * Calculate horizon angles for a point
   */
  static horizonAngles(
    dem: RasterData,
    position: Position,
    observerHeight = 2,
    directions = 36
  ): number[] {
    const { width, height, bands, bounds, pixelSize } = dem;
    const elevation = bands[0].data;

    const col = Math.floor((position[0] - bounds.minX) / pixelSize.x);
    const row = Math.floor((bounds.maxY - position[1]) / pixelSize.y);

    if (col < 0 || col >= width || row < 0 || row >= height) {
      throw new Error('Position outside DEM bounds');
    }

    const obsElev = elevation[row * width + col] + observerHeight;
    const angles: number[] = [];

    for (let i = 0; i < directions; i++) {
      const angle = (i * 360) / directions;
      const radians = (angle * Math.PI) / 180;

      let maxHorizonAngle = -90;

      // Cast ray in this direction
      for (let dist = 1; dist < Math.max(width, height); dist++) {
        const targetCol = Math.round(col + dist * Math.sin(radians));
        const targetRow = Math.round(row - dist * Math.cos(radians));

        if (
          targetCol < 0 ||
          targetCol >= width ||
          targetRow < 0 ||
          targetRow >= height
        ) {
          break;
        }

        const targetElev = elevation[targetRow * width + targetCol];
        const distance = dist * Math.max(pixelSize.x, pixelSize.y);
        const horizonAngle =
          (Math.atan2(targetElev - obsElev, distance) * 180) / Math.PI;

        maxHorizonAngle = Math.max(maxHorizonAngle, horizonAngle);
      }

      angles.push(maxHorizonAngle);
    }

    return angles;
  }
}
