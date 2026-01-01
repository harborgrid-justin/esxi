/**
 * Bandwidth Analyzer
 * Analyze and track bandwidth usage and savings
 */

import {
  BandwidthMetrics,
  CompressionAlgorithm,
  AlgorithmMetrics,
  CompressionResult,
} from '../types';

export class BandwidthAnalyzer {
  private metrics: BandwidthMetrics;
  private history: Array<{ timestamp: Date; metrics: BandwidthMetrics }> = [];

  constructor() {
    this.metrics = this.initializeMetrics();
  }

  /**
   * Initialize metrics
   */
  private initializeMetrics(): BandwidthMetrics {
    return {
      timestamp: new Date(),
      totalBytes: 0,
      compressedBytes: 0,
      savedBytes: 0,
      savingsPercent: 0,
      requestCount: 0,
      averageSize: 0,
      peakBandwidth: 0,
      byAlgorithm: new Map(),
    };
  }

  /**
   * Record compression result
   */
  record(result: CompressionResult): void {
    this.metrics.totalBytes += result.originalSize;
    this.metrics.compressedBytes += result.compressedSize;
    this.metrics.savedBytes = this.metrics.totalBytes - this.metrics.compressedBytes;
    this.metrics.savingsPercent = this.metrics.totalBytes > 0
      ? (this.metrics.savedBytes / this.metrics.totalBytes) * 100
      : 0;
    this.metrics.requestCount++;
    this.metrics.averageSize = this.metrics.totalBytes / this.metrics.requestCount;

    this.updateAlgorithmMetrics(result);
  }

  /**
   * Update algorithm-specific metrics
   */
  private updateAlgorithmMetrics(result: CompressionResult): void {
    const existing = this.metrics.byAlgorithm.get(result.algorithm);

    if (existing) {
      existing.usageCount++;
      existing.totalOriginalSize += result.originalSize;
      existing.totalCompressedSize += result.compressedSize;
      existing.averageRatio =
        existing.totalOriginalSize / existing.totalCompressedSize;
      existing.averageDuration =
        (existing.averageDuration * (existing.usageCount - 1) + result.duration) /
        existing.usageCount;
      existing.averageThroughput =
        (existing.averageThroughput * (existing.usageCount - 1) + result.throughput) /
        existing.usageCount;
    } else {
      this.metrics.byAlgorithm.set(result.algorithm, {
        algorithm: result.algorithm,
        usageCount: 1,
        totalOriginalSize: result.originalSize,
        totalCompressedSize: result.compressedSize,
        averageRatio: result.compressionRatio,
        averageDuration: result.duration,
        averageThroughput: result.throughput,
      });
    }
  }

  /**
   * Get current metrics
   */
  getMetrics(): BandwidthMetrics {
    return { ...this.metrics };
  }

  /**
   * Get metrics for specific algorithm
   */
  getAlgorithmMetrics(algorithm: CompressionAlgorithm): AlgorithmMetrics | undefined {
    return this.metrics.byAlgorithm.get(algorithm);
  }

  /**
   * Get all algorithm metrics
   */
  getAllAlgorithmMetrics(): AlgorithmMetrics[] {
    return Array.from(this.metrics.byAlgorithm.values());
  }

  /**
   * Reset metrics
   */
  reset(): void {
    this.history.push({
      timestamp: new Date(),
      metrics: { ...this.metrics },
    });
    this.metrics = this.initializeMetrics();
  }

  /**
   * Get historical metrics
   */
  getHistory(): Array<{ timestamp: Date; metrics: BandwidthMetrics }> {
    return [...this.history];
  }

  /**
   * Calculate bandwidth trends
   */
  getTrends(period: 'hour' | 'day' | 'week' | 'month'): {
    trend: 'increasing' | 'decreasing' | 'stable';
    changePercent: number;
    projection: number;
  } {
    if (this.history.length < 2) {
      return { trend: 'stable', changePercent: 0, projection: 0 };
    }

    const recent = this.history[this.history.length - 1];
    const previous = this.history[this.history.length - 2];

    const change = recent.metrics.savedBytes - previous.metrics.savedBytes;
    const changePercent = previous.metrics.savedBytes > 0
      ? (change / previous.metrics.savedBytes) * 100
      : 0;

    let trend: 'increasing' | 'decreasing' | 'stable';
    if (Math.abs(changePercent) < 5) {
      trend = 'stable';
    } else if (changePercent > 0) {
      trend = 'increasing';
    } else {
      trend = 'decreasing';
    }

    const projection = recent.metrics.savedBytes + change;

    return { trend, changePercent, projection };
  }

  /**
   * Generate report
   */
  generateReport(): {
    summary: string;
    totalSavings: number;
    topAlgorithm: string;
    efficiency: number;
  } {
    const topAlgo = this.getTopAlgorithm();
    const efficiency = this.metrics.savingsPercent;

    return {
      summary: `Saved ${this.formatBytes(this.metrics.savedBytes)} across ${this.metrics.requestCount} requests`,
      totalSavings: this.metrics.savedBytes,
      topAlgorithm: topAlgo?.algorithm || 'none',
      efficiency,
    };
  }

  /**
   * Get top performing algorithm
   */
  private getTopAlgorithm(): AlgorithmMetrics | undefined {
    const algorithms = Array.from(this.metrics.byAlgorithm.values());
    algorithms.sort((a, b) => b.averageRatio - a.averageRatio);
    return algorithms[0];
  }

  /**
   * Format bytes
   */
  private formatBytes(bytes: number): string {
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  }

  /**
   * Export metrics as JSON
   */
  export(): string {
    return JSON.stringify({
      current: this.metrics,
      history: this.history,
    }, null, 2);
  }

  /**
   * Import metrics from JSON
   */
  import(data: string): void {
    const parsed = JSON.parse(data);
    this.metrics = parsed.current;
    this.history = parsed.history;
  }
}
