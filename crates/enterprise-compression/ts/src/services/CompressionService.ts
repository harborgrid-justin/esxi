/**
 * Compression Service
 * Core compression service orchestrating multiple algorithms
 */

import { LZ4Compressor } from '../algorithms/LZ4Compressor';
import { ZstdCompressor } from '../algorithms/ZstdCompressor';
import { BrotliCompressor } from '../algorithms/BrotliCompressor';
import { DeltaEncoder } from '../algorithms/DeltaEncoder';
import { DictionaryCompressor } from '../algorithms/DictionaryCompressor';
import { HuffmanEncoder } from '../algorithms/HuffmanEncoder';
import { RLEEncoder } from '../algorithms/RLEEncoder';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  StreamConfig,
  CompressionService as ICompressionService,
} from '../types';

export class CompressionService implements ICompressionService {
  private lz4 = new LZ4Compressor();
  private zstd = new ZstdCompressor();
  private brotli = new BrotliCompressor();
  private delta = new DeltaEncoder();
  private dictionary = new DictionaryCompressor();
  private huffman = new HuffmanEncoder();
  private rle = new RLEEncoder();

  /**
   * Compress data with specified algorithm
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    switch (config.algorithm) {
      case CompressionAlgorithm.LZ4:
        return this.lz4.compress(data, config);

      case CompressionAlgorithm.ZSTD:
        return this.zstd.compress(data, config);

      case CompressionAlgorithm.BROTLI:
        return this.brotli.compress(data, config);

      case CompressionAlgorithm.DELTA:
        return this.delta.compress(data, config);

      case CompressionAlgorithm.DICTIONARY:
        return this.dictionary.compress(data, config);

      case CompressionAlgorithm.HUFFMAN:
        return this.huffman.compress(data, config);

      case CompressionAlgorithm.RLE:
        return this.rle.compress(data, config);

      default:
        return this.brotli.compress(data, config);
    }
  }

  /**
   * Decompress data with specified algorithm
   */
  async decompress(
    data: Buffer,
    algorithm: CompressionAlgorithm
  ): Promise<DecompressionResult> {
    switch (algorithm) {
      case CompressionAlgorithm.LZ4:
        return this.lz4.decompress(data);

      case CompressionAlgorithm.ZSTD:
        return this.zstd.decompress(data);

      case CompressionAlgorithm.BROTLI:
        return this.brotli.decompress(data);

      case CompressionAlgorithm.DELTA:
        return this.delta.decompress(data);

      case CompressionAlgorithm.DICTIONARY:
        return this.dictionary.decompress(data);

      case CompressionAlgorithm.HUFFMAN:
        return this.huffman.decompress(data);

      case CompressionAlgorithm.RLE:
        return this.rle.decompress(data);

      default:
        return this.brotli.decompress(data);
    }
  }

  /**
   * Create compression stream
   */
  compressStream(config: StreamConfig): NodeJS.ReadWriteStream {
    switch (config.algorithm) {
      case CompressionAlgorithm.LZ4:
        return this.lz4.createEncoder();
      case CompressionAlgorithm.BROTLI:
        return this.brotli.createCompressStream(config.level);
      default:
        return this.brotli.createCompressStream(6);
    }
  }

  /**
   * Create decompression stream
   */
  decompressStream(algorithm: CompressionAlgorithm): NodeJS.ReadWriteStream {
    switch (algorithm) {
      case CompressionAlgorithm.LZ4:
        return this.lz4.createDecoder();
      case CompressionAlgorithm.BROTLI:
        return this.brotli.createDecompressStream();
      default:
        return this.brotli.createDecompressStream();
    }
  }

  /**
   * Determine best algorithm for data
   */
  async getBestAlgorithm(data: Buffer): Promise<CompressionAlgorithm> {
    const sampleSize = Math.min(10000, data.length);
    const sample = data.slice(0, sampleSize);

    // Test multiple algorithms on sample
    const results = await Promise.all([
      this.testAlgorithm(sample, CompressionAlgorithm.LZ4),
      this.testAlgorithm(sample, CompressionAlgorithm.ZSTD),
      this.testAlgorithm(sample, CompressionAlgorithm.BROTLI),
      this.testAlgorithm(sample, CompressionAlgorithm.RLE),
    ]);

    // Find best ratio/speed tradeoff
    const scored = results.map(r => ({
      algorithm: r.algorithm,
      score: r.ratio * 0.7 + (1 / r.duration) * 0.3,
    }));

    scored.sort((a, b) => b.score - a.score);
    return scored[0].algorithm;
  }

  /**
   * Test algorithm on sample data
   */
  private async testAlgorithm(
    sample: Buffer,
    algorithm: CompressionAlgorithm
  ): Promise<{
    algorithm: CompressionAlgorithm;
    ratio: number;
    duration: number;
  }> {
    const config: CompressionConfig = {
      algorithm,
      level: 6,
    };

    try {
      const result = await this.compress(sample, config);
      return {
        algorithm,
        ratio: result.compressionRatio,
        duration: result.duration,
      };
    } catch {
      return {
        algorithm,
        ratio: 1,
        duration: Infinity,
      };
    }
  }

  /**
   * Batch compress multiple buffers
   */
  async compressBatch(
    items: Array<{ data: Buffer; config: CompressionConfig }>,
    concurrency: number = 4
  ): Promise<CompressionResult[]> {
    const results: CompressionResult[] = [];

    for (let i = 0; i < items.length; i += concurrency) {
      const batch = items.slice(i, i + concurrency);
      const batchResults = await Promise.all(
        batch.map(item => this.compress(item.data, item.config))
      );
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Compare compression algorithms
   */
  async compareAlgorithms(data: Buffer): Promise<Array<{
    algorithm: CompressionAlgorithm;
    ratio: number;
    duration: number;
    size: number;
  }>> {
    const algorithms = [
      CompressionAlgorithm.LZ4,
      CompressionAlgorithm.ZSTD,
      CompressionAlgorithm.BROTLI,
      CompressionAlgorithm.DELTA,
      CompressionAlgorithm.RLE,
    ];

    const results = [];

    for (const algorithm of algorithms) {
      try {
        const result = await this.compress(data, { algorithm, level: 6 });
        results.push({
          algorithm,
          ratio: result.compressionRatio,
          duration: result.duration,
          size: result.compressedSize,
        });
      } catch (error) {
        // Skip algorithms that fail
        continue;
      }
    }

    return results.sort((a, b) => b.ratio - a.ratio);
  }
}
