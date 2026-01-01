/**
 * Raster Classification
 * Supervised and unsupervised classification methods
 */

import { RasterData, RasterBand } from '../types';

export interface TrainingSample {
  classId: number;
  values: number[];
}

export interface ClassificationResult {
  classified: RasterData;
  accuracy?: number;
  confusionMatrix?: number[][];
}

export class RasterClassification {
  /**
   * Supervised classification - Maximum Likelihood
   */
  static maximumLikelihood(
    raster: RasterData,
    trainingSamples: TrainingSample[]
  ): ClassificationResult {
    const { width, height, bands } = raster;
    const numBands = bands.length;

    // Calculate statistics for each class
    const classStats = this.calculateClassStatistics(
      trainingSamples,
      numBands
    );

    // Classify each pixel
    const resultData = new Uint8Array(width * height);

    for (let i = 0; i < width * height; i++) {
      const pixelValues = bands.map((band) => band.data[i]);

      let maxLikelihood = -Infinity;
      let bestClass = 0;

      for (const [classId, stats] of classStats.entries()) {
        const likelihood = this.calculateLikelihood(pixelValues, stats);

        if (likelihood > maxLikelihood) {
          maxLikelihood = likelihood;
          bestClass = classId;
        }
      }

      resultData[i] = bestClass;
    }

    const band: RasterBand = {
      data: resultData,
      statistics: this.calculateStatistics(resultData),
    };

    return {
      classified: {
        ...raster,
        bands: [band],
      },
    };
  }

  /**
   * Supervised classification - Minimum Distance
   */
  static minimumDistance(
    raster: RasterData,
    trainingSamples: TrainingSample[]
  ): ClassificationResult {
    const { width, height, bands } = raster;
    const numBands = bands.length;

    // Calculate mean for each class
    const classMeans = new Map<number, number[]>();

    for (const sample of trainingSamples) {
      if (!classMeans.has(sample.classId)) {
        classMeans.set(sample.classId, Array(numBands).fill(0));
      }
    }

    const classCounts = new Map<number, number>();

    for (const sample of trainingSamples) {
      const means = classMeans.get(sample.classId)!;
      const count = classCounts.get(sample.classId) || 0;

      for (let b = 0; b < numBands; b++) {
        means[b] += sample.values[b];
      }

      classCounts.set(sample.classId, count + 1);
    }

    // Calculate averages
    for (const [classId, means] of classMeans.entries()) {
      const count = classCounts.get(classId)!;
      for (let b = 0; b < numBands; b++) {
        means[b] /= count;
      }
    }

    // Classify pixels
    const resultData = new Uint8Array(width * height);

    for (let i = 0; i < width * height; i++) {
      const pixelValues = bands.map((band) => band.data[i]);

      let minDistance = Infinity;
      let bestClass = 0;

      for (const [classId, means] of classMeans.entries()) {
        const distance = this.euclideanDistance(pixelValues, means);

        if (distance < minDistance) {
          minDistance = distance;
          bestClass = classId;
        }
      }

      resultData[i] = bestClass;
    }

    const band: RasterBand = {
      data: resultData,
      statistics: this.calculateStatistics(resultData),
    };

    return {
      classified: {
        ...raster,
        bands: [band],
      },
    };
  }

  /**
   * Unsupervised classification - K-means
   */
  static kmeans(
    raster: RasterData,
    numClasses: number,
    maxIterations = 100
  ): ClassificationResult {
    const { width, height, bands } = raster;
    const numBands = bands.length;
    const numPixels = width * height;

    // Initialize cluster centers randomly
    let centers: number[][] = [];
    for (let k = 0; k < numClasses; k++) {
      const randomIdx = Math.floor(Math.random() * numPixels);
      const center = bands.map((band) => band.data[randomIdx]);
      centers.push(center);
    }

    let labels = new Uint8Array(numPixels);
    let converged = false;
    let iteration = 0;

    while (!converged && iteration < maxIterations) {
      // Assignment step
      const newLabels = new Uint8Array(numPixels);

      for (let i = 0; i < numPixels; i++) {
        const pixelValues = bands.map((band) => band.data[i]);

        let minDist = Infinity;
        let bestCluster = 0;

        for (let k = 0; k < numClasses; k++) {
          const dist = this.euclideanDistance(pixelValues, centers[k]);

          if (dist < minDist) {
            minDist = dist;
            bestCluster = k;
          }
        }

        newLabels[i] = bestCluster;
      }

      // Update step
      const newCenters: number[][] = Array(numClasses)
        .fill(0)
        .map(() => Array(numBands).fill(0));
      const counts = Array(numClasses).fill(0);

      for (let i = 0; i < numPixels; i++) {
        const label = newLabels[i];
        counts[label]++;

        for (let b = 0; b < numBands; b++) {
          newCenters[label][b] += bands[b].data[i];
        }
      }

      for (let k = 0; k < numClasses; k++) {
        if (counts[k] > 0) {
          for (let b = 0; b < numBands; b++) {
            newCenters[k][b] /= counts[k];
          }
        }
      }

      // Check convergence
      converged = this.centersConverged(centers, newCenters);

      centers = newCenters;
      labels = newLabels;
      iteration++;
    }

    const band: RasterBand = {
      data: labels,
      statistics: this.calculateStatistics(labels),
    };

    return {
      classified: {
        ...raster,
        bands: [band],
      },
    };
  }

  /**
   * Unsupervised classification - ISODATA
   */
  static isodata(
    raster: RasterData,
    initialClusters: number,
    maxIterations = 100
  ): ClassificationResult {
    // Simplified ISODATA - similar to k-means with splitting/merging
    return this.kmeans(raster, initialClusters, maxIterations);
  }

  /**
   * Calculate class statistics
   */
  private static calculateClassStatistics(
    samples: TrainingSample[],
    numBands: number
  ): Map<number, { mean: number[]; cov: number[][] }> {
    const stats = new Map<number, { mean: number[]; cov: number[][] }>();
    const classSamples = new Map<number, number[][]>();

    // Group samples by class
    for (const sample of samples) {
      if (!classSamples.has(sample.classId)) {
        classSamples.set(sample.classId, []);
      }
      classSamples.get(sample.classId)!.push(sample.values);
    }

    // Calculate statistics for each class
    for (const [classId, samples] of classSamples.entries()) {
      const n = samples.length;

      // Calculate mean
      const mean = Array(numBands).fill(0);
      for (const sample of samples) {
        for (let i = 0; i < numBands; i++) {
          mean[i] += sample[i];
        }
      }
      for (let i = 0; i < numBands; i++) {
        mean[i] /= n;
      }

      // Calculate covariance matrix
      const cov: number[][] = Array(numBands)
        .fill(0)
        .map(() => Array(numBands).fill(0));

      for (const sample of samples) {
        for (let i = 0; i < numBands; i++) {
          for (let j = 0; j < numBands; j++) {
            cov[i][j] += (sample[i] - mean[i]) * (sample[j] - mean[j]);
          }
        }
      }

      for (let i = 0; i < numBands; i++) {
        for (let j = 0; j < numBands; j++) {
          cov[i][j] /= n - 1;
        }
      }

      stats.set(classId, { mean, cov });
    }

    return stats;
  }

  /**
   * Calculate likelihood using multivariate normal distribution
   */
  private static calculateLikelihood(
    values: number[],
    stats: { mean: number[]; cov: number[][] }
  ): number {
    const n = values.length;
    const diff = values.map((v, i) => v - stats.mean[i]);

    // Simplified - assumes diagonal covariance matrix
    let exponent = 0;
    for (let i = 0; i < n; i++) {
      const variance = stats.cov[i][i];
      if (variance > 0) {
        exponent += (diff[i] * diff[i]) / variance;
      }
    }

    return -0.5 * exponent;
  }

  /**
   * Calculate Euclidean distance
   */
  private static euclideanDistance(a: number[], b: number[]): number {
    let sum = 0;
    for (let i = 0; i < a.length; i++) {
      sum += Math.pow(a[i] - b[i], 2);
    }
    return Math.sqrt(sum);
  }

  /**
   * Check if centers have converged
   */
  private static centersConverged(
    centers1: number[][],
    centers2: number[][],
    tolerance = 1e-6
  ): boolean {
    for (let i = 0; i < centers1.length; i++) {
      for (let j = 0; j < centers1[i].length; j++) {
        if (Math.abs(centers1[i][j] - centers2[i][j]) > tolerance) {
          return false;
        }
      }
    }
    return true;
  }

  /**
   * Calculate accuracy from confusion matrix
   */
  static calculateAccuracy(confusionMatrix: number[][]): number {
    let correct = 0;
    let total = 0;

    for (let i = 0; i < confusionMatrix.length; i++) {
      for (let j = 0; j < confusionMatrix[i].length; j++) {
        total += confusionMatrix[i][j];
        if (i === j) {
          correct += confusionMatrix[i][j];
        }
      }
    }

    return total > 0 ? correct / total : 0;
  }

  /**
   * Calculate statistics
   */
  private static calculateStatistics(data: Uint8Array | Float32Array): any {
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
