/**
 * Compression service bridge for high-performance data compression.
 *
 * Provides TypeScript wrapper around WASM compression engine.
 */

import type {
  CompressionParams,
  CompressionRecommendation,
  OperationResult,
} from '../types';
import { BridgeError } from '../types';
import { WasmLoader } from '../loader/WasmLoader';

/**
 * Compression service bridge.
 */
export class CompressionBridge {
  private compressionEngine: any = null;

  constructor(private readonly loader: WasmLoader) {}

  /**
   * Initialize the compression engine.
   */
  private async ensureInitialized(): Promise<void> {
    if (!this.compressionEngine) {
      const instance = this.loader.getInstance();
      // In production: this.compressionEngine = new instance.CompressionEngine();
      throw new BridgeError(
        'Compression engine not available. Build WASM module first.',
        'COMPRESSION_NOT_AVAILABLE'
      );
    }
  }

  /**
   * Compress data using the specified algorithm.
   *
   * @param data - Input data to compress
   * @param params - Compression parameters
   * @returns Compressed data
   */
  async compress(
    data: Uint8Array,
    params: CompressionParams
  ): Promise<OperationResult<Uint8Array>> {
    await this.ensureInitialized();

    try {
      const result = await this.compressionEngine.compress(data, params);
      return result as OperationResult<Uint8Array>;
    } catch (error) {
      throw new BridgeError(
        `Compression failed: ${error instanceof Error ? error.message : String(error)}`,
        'COMPRESS_ERROR',
        error
      );
    }
  }

  /**
   * Decompress data using the specified algorithm.
   *
   * @param data - Compressed data
   * @param algorithm - Compression algorithm used
   * @returns Decompressed data
   */
  async decompress(
    data: Uint8Array,
    algorithm: CompressionParams['algorithm']
  ): Promise<OperationResult<Uint8Array>> {
    await this.ensureInitialized();

    try {
      const result = await this.compressionEngine.decompress(data, algorithm);
      return result as OperationResult<Uint8Array>;
    } catch (error) {
      throw new BridgeError(
        `Decompression failed: ${error instanceof Error ? error.message : String(error)}`,
        'DECOMPRESS_ERROR',
        error
      );
    }
  }

  /**
   * Get compression ratio for given data and parameters.
   *
   * @param data - Data to estimate
   * @param params - Compression parameters
   * @returns Estimated compression ratio
   */
  async estimateRatio(
    data: Uint8Array,
    params: CompressionParams
  ): Promise<number> {
    await this.ensureInitialized();

    try {
      const ratio = await this.compressionEngine.estimate_ratio(data, params);
      return ratio;
    } catch (error) {
      throw new BridgeError(
        `Ratio estimation failed: ${error instanceof Error ? error.message : String(error)}`,
        'ESTIMATE_ERROR',
        error
      );
    }
  }

  /**
   * Select optimal compression algorithm for given data.
   *
   * @param data - Data to analyze
   * @returns Recommended algorithm and level
   */
  async selectAlgorithm(data: Uint8Array): Promise<CompressionRecommendation> {
    await this.ensureInitialized();

    try {
      const recommendation = await this.compressionEngine.select_algorithm(data);
      return recommendation as CompressionRecommendation;
    } catch (error) {
      throw new BridgeError(
        `Algorithm selection failed: ${error instanceof Error ? error.message : String(error)}`,
        'SELECT_ERROR',
        error
      );
    }
  }

  /**
   * Compress a string.
   *
   * @param str - String to compress
   * @param params - Compression parameters
   * @returns Compressed data
   */
  async compressString(
    str: string,
    params: CompressionParams
  ): Promise<OperationResult<Uint8Array>> {
    const encoder = new TextEncoder();
    const data = encoder.encode(str);
    return this.compress(data, params);
  }

  /**
   * Decompress to a string.
   *
   * @param data - Compressed data
   * @param algorithm - Compression algorithm used
   * @returns Decompressed string
   */
  async decompressString(
    data: Uint8Array,
    algorithm: CompressionParams['algorithm']
  ): Promise<string> {
    const result = await this.decompress(data, algorithm);

    if (!result.success || !result.data) {
      throw new BridgeError(
        result.error || 'Decompression failed',
        'DECOMPRESS_STRING_ERROR'
      );
    }

    const decoder = new TextDecoder();
    return decoder.decode(result.data);
  }

  /**
   * Compress JSON data.
   *
   * @param obj - Object to compress
   * @param params - Compression parameters
   * @returns Compressed data
   */
  async compressJson(
    obj: unknown,
    params: CompressionParams
  ): Promise<OperationResult<Uint8Array>> {
    const json = JSON.stringify(obj);
    return this.compressString(json, params);
  }

  /**
   * Decompress JSON data.
   *
   * @param data - Compressed data
   * @param algorithm - Compression algorithm used
   * @returns Decompressed object
   */
  async decompressJson<T = unknown>(
    data: Uint8Array,
    algorithm: CompressionParams['algorithm']
  ): Promise<T> {
    const json = await this.decompressString(data, algorithm);
    return JSON.parse(json) as T;
  }

  /**
   * Clean up resources.
   */
  dispose(): void {
    this.compressionEngine = null;
  }
}
