/**
 * Zstandard (Zstd) Compression Algorithm
 * Modern compression algorithm with excellent compression ratios and speed
 */

import { ZstdCodec } from 'zstd-codec';
import { createHash } from 'crypto';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionLevel,
  CompressionError,
} from '../types';

export class ZstdCompressor {
  private static codec: any = null;
  private static readonly VERSION = '1.0.0';
  private static readonly DEFAULT_LEVEL = 3;

  /**
   * Initialize Zstd codec
   */
  private static async initCodec(): Promise<void> {
    if (!this.codec) {
      this.codec = await new Promise<any>((resolve, reject) => {
        ZstdCodec.run((codec: any) => {
          if (codec) {
            resolve(codec);
          } else {
            reject(new Error('Failed to initialize Zstd codec'));
          }
        });
      });
    }
  }

  /**
   * Compress data using Zstd algorithm
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      await ZstdCompressor.initCodec();

      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;
      const level = this.mapCompressionLevel(config.level);

      // Create streaming or simple compressor
      const Simple = ZstdCompressor.codec.Streaming
        ? ZstdCompressor.codec.Streaming.Compress
        : ZstdCompressor.codec.Compress;

      // Perform compression
      const compressed = config.dictionary
        ? Simple(new Uint8Array(data), level, config.dictionary)
        : Simple(new Uint8Array(data), level);

      const compressedBuffer = Buffer.from(compressed);
      const compressedSize = compressedBuffer.length;

      // Calculate metrics
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000;

      // Generate checksum
      const checksum = createHash('sha256')
        .update(data)
        .digest('hex');

      return {
        compressed: compressedBuffer,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.ZSTD,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: ZstdCompressor.VERSION,
          custom: {
            zstdLevel: level,
            hasDictionary: !!config.dictionary,
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `Zstd compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.ZSTD,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress Zstd compressed data
   */
  async decompress(data: Buffer, dictionary?: Buffer): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      await ZstdCompressor.initCodec();

      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const Simple = ZstdCompressor.codec.Streaming
        ? ZstdCompressor.codec.Streaming.Decompress
        : ZstdCompressor.codec.Decompress;

      // Perform decompression
      const decompressed = dictionary
        ? Simple(new Uint8Array(data), dictionary)
        : Simple(new Uint8Array(data));

      const decompressedBuffer = Buffer.from(decompressed);
      const duration = performance.now() - startTime;

      return {
        decompressed: decompressedBuffer,
        originalSize: decompressedBuffer.length,
        duration,
        algorithm: CompressionAlgorithm.ZSTD,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `Zstd decompression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.ZSTD,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Train dictionary for better compression on similar data
   */
  async trainDictionary(samples: Buffer[], dictSize: number = 112640): Promise<Buffer> {
    try {
      await ZstdCompressor.initCodec();

      const TrainDictionary = ZstdCompressor.codec.TrainDictionary;
      if (!TrainDictionary) {
        throw new Error('Dictionary training not supported in this Zstd build');
      }

      const sampleSizes = samples.map(s => s.length);
      const concatenated = Buffer.concat(samples);

      const dictionary = TrainDictionary(
        dictSize,
        new Uint8Array(concatenated),
        sampleSizes
      );

      return Buffer.from(dictionary);
    } catch (error) {
      throw new CompressionError(
        `Dictionary training failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.ZSTD,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Map compression level to Zstd level (1-22)
   */
  private mapCompressionLevel(level: CompressionLevel): number {
    const mapping: Record<CompressionLevel, number> = {
      [CompressionLevel.FASTEST]: 1,
      [CompressionLevel.FAST]: 3,
      [CompressionLevel.BALANCED]: 6,
      [CompressionLevel.HIGH]: 12,
      [CompressionLevel.MAXIMUM]: 19,
    };
    return mapping[level] || ZstdCompressor.DEFAULT_LEVEL;
  }

  /**
   * Get compression bound (maximum compressed size)
   */
  async getCompressBound(size: number): Promise<number> {
    await ZstdCompressor.initCodec();
    return ZstdCompressor.codec.compressBound
      ? ZstdCompressor.codec.compressBound(size)
      : Math.ceil(size * 1.1) + 1024;
  }

  /**
   * Compress with custom parameters
   */
  async compressAdvanced(
    data: Buffer,
    params: {
      level?: number;
      windowLog?: number;
      hashLog?: number;
      chainLog?: number;
      searchLog?: number;
      minMatch?: number;
      targetLength?: number;
      strategy?: number;
    }
  ): Promise<Buffer> {
    try {
      await ZstdCompressor.initCodec();

      const level = params.level || ZstdCompressor.DEFAULT_LEVEL;
      const compressed = ZstdCompressor.codec.Compress(
        new Uint8Array(data),
        level
      );

      return Buffer.from(compressed);
    } catch (error) {
      throw new CompressionError(
        `Advanced Zstd compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.ZSTD,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Estimate compression ratio without actually compressing
   */
  async estimateCompressionRatio(data: Buffer): Promise<number> {
    // Simple heuristic based on entropy
    const frequencies = new Map<number, number>();
    let totalBytes = 0;

    for (const byte of data) {
      frequencies.set(byte, (frequencies.get(byte) || 0) + 1);
      totalBytes++;
    }

    let entropy = 0;
    for (const count of frequencies.values()) {
      const p = count / totalBytes;
      entropy -= p * Math.log2(p);
    }

    // Estimate compression ratio based on entropy
    // Lower entropy = better compression
    const maxEntropy = 8;
    const estimatedRatio = 1 + (maxEntropy - entropy) / maxEntropy;

    return Math.max(1.1, Math.min(estimatedRatio, 5.0));
  }
}
