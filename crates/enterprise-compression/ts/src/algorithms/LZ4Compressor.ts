/**
 * LZ4 Compression Algorithm
 * High-speed compression with excellent decompression performance
 */

import * as lz4 from 'lz4';
import { createHash } from 'crypto';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionError,
} from '../types';

export class LZ4Compressor {
  private static readonly MAGIC_NUMBER = 0x184D2204;
  private static readonly VERSION = '1.0.0';

  /**
   * Compress data using LZ4 algorithm
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;
      const maxCompressedSize = lz4.encodeBound(originalSize);
      const compressed = Buffer.allocUnsafe(maxCompressedSize);

      // Perform LZ4 compression
      const compressedSize = lz4.encodeBlock(data, compressed);
      const result = compressed.slice(0, compressedSize);

      // Calculate metrics
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000; // bytes per second

      // Generate checksum
      const checksum = createHash('xxhash64')
        .update(data)
        .digest('hex');

      return {
        compressed: result,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.LZ4,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: LZ4Compressor.VERSION,
          custom: {
            magicNumber: LZ4Compressor.MAGIC_NUMBER,
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `LZ4 compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.LZ4,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress LZ4 compressed data
   */
  async decompress(data: Buffer, originalSize?: number): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      // Estimate uncompressed size if not provided
      const uncompressedSize = originalSize || data.length * 4;
      const decompressed = Buffer.allocUnsafe(uncompressedSize);

      // Perform LZ4 decompression
      const actualSize = lz4.decodeBlock(data, decompressed);
      const result = decompressed.slice(0, actualSize);

      const duration = performance.now() - startTime;

      return {
        decompressed: result,
        originalSize: actualSize,
        duration,
        algorithm: CompressionAlgorithm.LZ4,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `LZ4 decompression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.LZ4,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Compress data with high-compression mode
   */
  async compressHC(data: Buffer, level: number = 9): Promise<Buffer> {
    try {
      const maxCompressedSize = lz4.encodeBound(data.length);
      const compressed = Buffer.allocUnsafe(maxCompressedSize);

      const compressedSize = lz4.encodeBlockHC(data, compressed, level);
      return compressed.slice(0, compressedSize);
    } catch (error) {
      throw new CompressionError(
        `LZ4-HC compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.LZ4,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Create streaming compressor
   */
  createEncoder(): NodeJS.ReadWriteStream {
    return lz4.createEncoderStream({
      highCompression: false,
    });
  }

  /**
   * Create streaming decompressor
   */
  createDecoder(): NodeJS.ReadWriteStream {
    return lz4.createDecoderStream();
  }

  /**
   * Benchmark compression performance
   */
  async benchmark(data: Buffer, iterations: number = 100): Promise<{
    avgCompressionTime: number;
    avgDecompressionTime: number;
    compressionRatio: number;
    throughput: number;
  }> {
    let totalCompressTime = 0;
    let totalDecompressTime = 0;
    let compressedSize = 0;

    for (let i = 0; i < iterations; i++) {
      const compressStart = performance.now();
      const result = await this.compress(data, {
        algorithm: CompressionAlgorithm.LZ4,
        level: 1,
      } as CompressionConfig);
      totalCompressTime += performance.now() - compressStart;
      compressedSize = result.compressedSize;

      const decompressStart = performance.now();
      await this.decompress(result.compressed, data.length);
      totalDecompressTime += performance.now() - decompressStart;
    }

    return {
      avgCompressionTime: totalCompressTime / iterations,
      avgDecompressionTime: totalDecompressTime / iterations,
      compressionRatio: data.length / compressedSize,
      throughput: (data.length * iterations) / totalCompressTime,
    };
  }
}
