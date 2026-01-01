/**
 * Storage Optimizer
 * Optimize storage usage through compression and deduplication
 */

import { createHash } from 'crypto';
import {
  StorageMetrics,
  TypeMetrics,
  CompressionAlgorithm,
  CompressionResult,
} from '../types';

export class StorageOptimizer {
  private metrics: StorageMetrics;
  private fileHashes = new Map<string, string>();
  private duplicates = new Map<string, string[]>();

  constructor() {
    this.metrics = this.initializeMetrics();
  }

  /**
   * Initialize metrics
   */
  private initializeMetrics(): StorageMetrics {
    return {
      totalSize: 0,
      compressedSize: 0,
      savingsPercent: 0,
      fileCount: 0,
      byType: new Map(),
      byAlgorithm: new Map(),
    };
  }

  /**
   * Record file compression
   */
  recordFile(
    path: string,
    type: string,
    result: CompressionResult
  ): void {
    this.metrics.totalSize += result.originalSize;
    this.metrics.compressedSize += result.compressedSize;
    this.metrics.savingsPercent =
      (this.metrics.totalSize - this.metrics.compressedSize) /
      this.metrics.totalSize *
      100;
    this.metrics.fileCount++;

    this.updateTypeMetrics(type, result);
    this.updateAlgorithmMetrics(result.algorithm, result.compressedSize);
  }

  /**
   * Update type-specific metrics
   */
  private updateTypeMetrics(type: string, result: CompressionResult): void {
    const existing = this.metrics.byType.get(type);

    if (existing) {
      existing.count++;
      existing.originalSize += result.originalSize;
      existing.compressedSize += result.compressedSize;
      existing.savingsPercent =
        ((existing.originalSize - existing.compressedSize) /
          existing.originalSize) *
        100;
    } else {
      this.metrics.byType.set(type, {
        type,
        count: 1,
        originalSize: result.originalSize,
        compressedSize: result.compressedSize,
        savingsPercent:
          ((result.originalSize - result.compressedSize) / result.originalSize) *
          100,
      });
    }
  }

  /**
   * Update algorithm usage metrics
   */
  private updateAlgorithmMetrics(
    algorithm: CompressionAlgorithm,
    size: number
  ): void {
    const current = this.metrics.byAlgorithm.get(algorithm) || 0;
    this.metrics.byAlgorithm.set(algorithm, current + size);
  }

  /**
   * Detect duplicate files
   */
  async detectDuplicate(path: string, content: Buffer): Promise<string | null> {
    const hash = createHash('sha256').update(content).digest('hex');

    const existing = this.fileHashes.get(hash);
    if (existing) {
      // Duplicate found
      const duplicateList = this.duplicates.get(hash) || [];
      duplicateList.push(path);
      this.duplicates.set(hash, duplicateList);
      return existing;
    }

    this.fileHashes.set(hash, path);
    return null;
  }

  /**
   * Get duplicate files
   */
  getDuplicates(): Map<string, string[]> {
    return new Map(this.duplicates);
  }

  /**
   * Calculate duplicate savings
   */
  calculateDuplicateSavings(): number {
    let savings = 0;

    for (const paths of this.duplicates.values()) {
      // Each duplicate (except the first) is wasted space
      savings += paths.length - 1;
    }

    return savings;
  }

  /**
   * Get metrics
   */
  getMetrics(): StorageMetrics {
    return { ...this.metrics };
  }

  /**
   * Get type metrics
   */
  getTypeMetrics(type: string): TypeMetrics | undefined {
    return this.metrics.byType.get(type);
  }

  /**
   * Get all type metrics
   */
  getAllTypeMetrics(): TypeMetrics[] {
    return Array.from(this.metrics.byType.values());
  }

  /**
   * Get top file types by size
   */
  getTopFileTypes(limit: number = 10): TypeMetrics[] {
    const types = Array.from(this.metrics.byType.values());
    types.sort((a, b) => b.originalSize - a.originalSize);
    return types.slice(0, limit);
  }

  /**
   * Get algorithm distribution
   */
  getAlgorithmDistribution(): Array<{
    algorithm: CompressionAlgorithm;
    size: number;
    percentage: number;
  }> {
    const total = Array.from(this.metrics.byAlgorithm.values()).reduce(
      (sum, size) => sum + size,
      0
    );

    return Array.from(this.metrics.byAlgorithm.entries()).map(
      ([algorithm, size]) => ({
        algorithm,
        size,
        percentage: (size / total) * 100,
      })
    );
  }

  /**
   * Generate optimization recommendations
   */
  getRecommendations(): Array<{
    type: string;
    suggestion: string;
    potentialSavings: number;
  }> {
    const recommendations: Array<{
      type: string;
      suggestion: string;
      potentialSavings: number;
    }> = [];

    // Check for low compression ratios
    for (const typeMetric of this.metrics.byType.values()) {
      if (typeMetric.savingsPercent < 20 && typeMetric.originalSize > 1024 * 1024) {
        recommendations.push({
          type: typeMetric.type,
          suggestion: `Consider using a different compression algorithm for ${typeMetric.type} files`,
          potentialSavings: typeMetric.originalSize * 0.2,
        });
      }
    }

    // Check for duplicates
    const duplicateSavings = this.calculateDuplicateSavings();
    if (duplicateSavings > 0) {
      recommendations.push({
        type: 'duplicates',
        suggestion: 'Remove duplicate files to save storage',
        potentialSavings: duplicateSavings,
      });
    }

    return recommendations;
  }

  /**
   * Reset metrics
   */
  reset(): void {
    this.metrics = this.initializeMetrics();
    this.fileHashes.clear();
    this.duplicates.clear();
  }

  /**
   * Export metrics
   */
  export(): string {
    return JSON.stringify(
      {
        metrics: this.metrics,
        duplicates: Array.from(this.duplicates.entries()),
      },
      null,
      2
    );
  }
}
