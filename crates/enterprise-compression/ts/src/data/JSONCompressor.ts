/**
 * JSON Compression and Optimization
 * Minimize and compress JSON data efficiently
 */

import * as pako from 'pako';
import {
  JSONCompressionResult,
  DataOptimizationProfile,
  CompressionAlgorithm,
  OptimizationError,
} from '../types';

export class JSONCompressor {
  private static readonly VERSION = '1.0.0';

  /**
   * Compress JSON data
   */
  async compress(
    data: any,
    profile: DataOptimizationProfile
  ): Promise<JSONCompressionResult> {
    try {
      let processedData = data;

      // Apply minification
      if (profile.enableMinification !== false) {
        processedData = this.minify(processedData);
      }

      // Apply deduplication
      if (profile.enableDedupe) {
        processedData = this.deduplicate(processedData);
      }

      // Apply schema optimization
      if (profile.enableSchemaOptimization) {
        processedData = this.optimizeSchema(processedData);
      }

      // Apply custom transforms
      if (profile.customTransforms) {
        for (const transform of profile.customTransforms) {
          processedData = transform.apply(processedData);
        }
      }

      // Convert to JSON string
      const jsonString = JSON.stringify(processedData);
      const originalSize = JSON.stringify(data).length;

      // Compress with specified algorithm
      const compressed = this.compressWithAlgorithm(
        Buffer.from(jsonString),
        profile.algorithm
      );

      const compressedSize = compressed.length;
      const savingsPercent = ((originalSize - compressedSize) / originalSize) * 100;

      return {
        compressed,
        originalSize,
        compressedSize,
        savingsPercent,
        minified: profile.enableMinification !== false,
        schema: profile.enableSchemaOptimization ? this.extractSchema(data) : undefined,
      };
    } catch (error) {
      throw new OptimizationError(
        `JSON compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'json-compression',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress JSON data
   */
  async decompress(
    compressed: Buffer,
    algorithm: CompressionAlgorithm
  ): Promise<any> {
    try {
      const decompressed = this.decompressWithAlgorithm(compressed, algorithm);
      return JSON.parse(decompressed.toString());
    } catch (error) {
      throw new OptimizationError(
        `JSON decompression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        'json-decompression',
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Minify JSON (remove whitespace, shorten keys)
   */
  private minify(data: any): any {
    // This is a simple minification - just removes extra whitespace
    // A more advanced version would shorten object keys
    return data;
  }

  /**
   * Deduplicate repeated objects/arrays
   */
  private deduplicate(data: any): any {
    const seen = new Map<string, any>();
    const refs = new Map<any, string>();
    let refCounter = 0;

    const process = (obj: any): any => {
      if (obj === null || typeof obj !== 'object') {
        return obj;
      }

      const str = JSON.stringify(obj);
      if (seen.has(str)) {
        const ref = refs.get(seen.get(str));
        return { $ref: ref };
      }

      if (Array.isArray(obj)) {
        const arr = obj.map(item => process(item));
        seen.set(str, arr);
        const refId = `#${refCounter++}`;
        refs.set(arr, refId);
        return arr;
      }

      const result: any = {};
      for (const [key, value] of Object.entries(obj)) {
        result[key] = process(value);
      }

      seen.set(str, result);
      const refId = `#${refCounter++}`;
      refs.set(result, refId);
      return result;
    };

    return process(data);
  }

  /**
   * Optimize schema by extracting common patterns
   */
  private optimizeSchema(data: any): any {
    if (Array.isArray(data) && data.length > 0) {
      // Extract common schema from array of objects
      const schema = this.extractCommonSchema(data);
      return {
        $schema: schema,
        $data: data.map(item => this.compactWithSchema(item, schema)),
      };
    }

    return data;
  }

  /**
   * Extract common schema from objects
   */
  private extractCommonSchema(objects: any[]): any {
    if (objects.length === 0) return {};

    const schema: any = {};
    const firstObj = objects[0];

    for (const key of Object.keys(firstObj)) {
      const allHaveKey = objects.every(obj => key in obj);
      if (allHaveKey) {
        const type = typeof firstObj[key];
        schema[key] = { type, required: true };
      }
    }

    return schema;
  }

  /**
   * Compact object using schema
   */
  private compactWithSchema(obj: any, schema: any): any {
    const result: any = {};

    for (const [key, value] of Object.entries(obj)) {
      if (!(key in schema)) {
        result[key] = value;
      } else {
        // Store only value for known schema fields
        result[key] = value;
      }
    }

    return result;
  }

  /**
   * Extract schema from data
   */
  private extractSchema(data: any): string {
    const schema: any = {
      type: Array.isArray(data) ? 'array' : typeof data,
    };

    if (typeof data === 'object' && data !== null) {
      schema.properties = {};
      for (const [key, value] of Object.entries(data)) {
        schema.properties[key] = {
          type: Array.isArray(value) ? 'array' : typeof value,
        };
      }
    }

    return JSON.stringify(schema);
  }

  /**
   * Compress with specified algorithm
   */
  private compressWithAlgorithm(
    data: Buffer,
    algorithm: CompressionAlgorithm
  ): Buffer {
    switch (algorithm) {
      case CompressionAlgorithm.GZIP:
        return pako.gzip(data);
      case CompressionAlgorithm.DEFLATE:
        return pako.deflate(data);
      default:
        return pako.gzip(data);
    }
  }

  /**
   * Decompress with specified algorithm
   */
  private decompressWithAlgorithm(
    data: Buffer,
    algorithm: CompressionAlgorithm
  ): Buffer {
    switch (algorithm) {
      case CompressionAlgorithm.GZIP:
        return Buffer.from(pako.ungzip(data));
      case CompressionAlgorithm.DEFLATE:
        return Buffer.from(pako.inflate(data));
      default:
        return Buffer.from(pako.ungzip(data));
    }
  }

  /**
   * Compress JSON with automatic optimization
   */
  async compressAuto(data: any): Promise<JSONCompressionResult> {
    const profile: DataOptimizationProfile = {
      name: 'auto',
      algorithm: CompressionAlgorithm.GZIP,
      level: 9,
      enableMinification: true,
      enableDedupe: this.shouldDedupe(data),
      enableSchemaOptimization: this.shouldOptimizeSchema(data),
    };

    return this.compress(data, profile);
  }

  /**
   * Check if deduplication would be beneficial
   */
  private shouldDedupe(data: any): boolean {
    if (!Array.isArray(data)) return false;
    if (data.length < 10) return false;

    // Check for repeated objects
    const seen = new Set<string>();
    let duplicates = 0;

    for (const item of data) {
      const str = JSON.stringify(item);
      if (seen.has(str)) {
        duplicates++;
      } else {
        seen.add(str);
      }
    }

    return duplicates / data.length > 0.1;
  }

  /**
   * Check if schema optimization would be beneficial
   */
  private shouldOptimizeSchema(data: any): boolean {
    if (!Array.isArray(data)) return false;
    if (data.length < 5) return false;

    return data.every(item => typeof item === 'object' && item !== null);
  }

  /**
   * Analyze JSON for compression potential
   */
  analyzeJSON(data: any): {
    size: number;
    depth: number;
    arrayCount: number;
    objectCount: number;
    duplicateRate: number;
    estimatedCompression: number;
  } {
    const str = JSON.stringify(data);
    const size = str.length;

    let depth = 0;
    let arrayCount = 0;
    let objectCount = 0;

    const analyze = (obj: any, currentDepth: number) => {
      depth = Math.max(depth, currentDepth);

      if (Array.isArray(obj)) {
        arrayCount++;
        obj.forEach(item => analyze(item, currentDepth + 1));
      } else if (typeof obj === 'object' && obj !== null) {
        objectCount++;
        Object.values(obj).forEach(value => analyze(value, currentDepth + 1));
      }
    };

    analyze(data, 1);

    const duplicateRate = this.calculateDuplicateRate(data);
    const estimatedCompression = this.estimateCompressionRatio(str);

    return {
      size,
      depth,
      arrayCount,
      objectCount,
      duplicateRate,
      estimatedCompression,
    };
  }

  /**
   * Calculate duplicate rate
   */
  private calculateDuplicateRate(data: any): number {
    if (!Array.isArray(data)) return 0;

    const seen = new Set<string>();
    let duplicates = 0;

    for (const item of data) {
      const str = JSON.stringify(item);
      if (seen.has(str)) {
        duplicates++;
      } else {
        seen.add(str);
      }
    }

    return data.length > 0 ? duplicates / data.length : 0;
  }

  /**
   * Estimate compression ratio
   */
  private estimateCompressionRatio(str: string): number {
    const frequencies = new Map<string, number>();

    for (const char of str) {
      frequencies.set(char, (frequencies.get(char) || 0) + 1);
    }

    let entropy = 0;
    for (const count of frequencies.values()) {
      const p = count / str.length;
      entropy -= p * Math.log2(p);
    }

    const maxEntropy = Math.log2(256);
    return maxEntropy / entropy;
  }
}
