/**
 * Brotli Compression Algorithm
 * Optimized for web content with excellent compression ratios
 */

import { brotliCompress, brotliDecompress, constants } from 'zlib';
import { promisify } from 'util';
import { createHash } from 'crypto';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionLevel,
  CompressionError,
} from '../types';

const brotliCompressAsync = promisify(brotliCompress);
const brotliDecompressAsync = promisify(brotliDecompress);

export class BrotliCompressor {
  private static readonly VERSION = '1.0.0';
  private static readonly DEFAULT_WINDOW_SIZE = 22;
  private static readonly DEFAULT_BLOCK_SIZE = 16;

  /**
   * Compress data using Brotli algorithm
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;
      const quality = this.mapCompressionLevel(config.level);

      const options = {
        params: {
          [constants.BROTLI_PARAM_QUALITY]: quality,
          [constants.BROTLI_PARAM_LGWIN]: config.windowSize || BrotliCompressor.DEFAULT_WINDOW_SIZE,
          [constants.BROTLI_PARAM_LGBLOCK]: BrotliCompressor.DEFAULT_BLOCK_SIZE,
          [constants.BROTLI_PARAM_MODE]: this.detectMode(data),
        },
      };

      // Perform Brotli compression
      const compressed = await brotliCompressAsync(data, options);
      const compressedSize = compressed.length;

      // Calculate metrics
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000;

      // Generate checksum
      const checksum = createHash('sha256')
        .update(data)
        .digest('hex');

      return {
        compressed,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.BROTLI,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: BrotliCompressor.VERSION,
          custom: {
            quality,
            windowSize: config.windowSize || BrotliCompressor.DEFAULT_WINDOW_SIZE,
            mode: this.detectMode(data),
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `Brotli compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.BROTLI,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress Brotli compressed data
   */
  async decompress(data: Buffer): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      // Perform Brotli decompression
      const decompressed = await brotliDecompressAsync(data);
      const duration = performance.now() - startTime;

      return {
        decompressed,
        originalSize: decompressed.length,
        duration,
        algorithm: CompressionAlgorithm.BROTLI,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `Brotli decompression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.BROTLI,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Map compression level to Brotli quality (0-11)
   */
  private mapCompressionLevel(level: CompressionLevel): number {
    const mapping: Record<CompressionLevel, number> = {
      [CompressionLevel.FASTEST]: 1,
      [CompressionLevel.FAST]: 4,
      [CompressionLevel.BALANCED]: 6,
      [CompressionLevel.HIGH]: 9,
      [CompressionLevel.MAXIMUM]: 11,
    };
    return mapping[level] || 6;
  }

  /**
   * Detect best Brotli mode for data
   */
  private detectMode(data: Buffer): number {
    // Check if data is text/UTF-8
    const isText = this.isTextData(data);

    if (isText) {
      return constants.BROTLI_MODE_TEXT;
    }

    // Check for font data (WOFF, WOFF2, TTF, OTF)
    if (this.isFontData(data)) {
      return constants.BROTLI_MODE_FONT;
    }

    return constants.BROTLI_MODE_GENERIC;
  }

  /**
   * Check if data appears to be text
   */
  private isTextData(data: Buffer): boolean {
    const sampleSize = Math.min(1024, data.length);
    let textChars = 0;

    for (let i = 0; i < sampleSize; i++) {
      const byte = data[i];
      // ASCII printable, tab, newline, carriage return
      if (
        (byte >= 32 && byte <= 126) ||
        byte === 9 ||
        byte === 10 ||
        byte === 13
      ) {
        textChars++;
      }
    }

    return textChars / sampleSize > 0.85;
  }

  /**
   * Check if data is font data
   */
  private isFontData(data: Buffer): boolean {
    if (data.length < 4) return false;

    const header = data.toString('ascii', 0, 4);
    return (
      header === 'wOFF' ||
      header === 'wOF2' ||
      header === '\x00\x01\x00\x00' || // TrueType
      header === 'OTTO' // OpenType
    );
  }

  /**
   * Compress with custom parameters for web optimization
   */
  async compressForWeb(
    data: Buffer,
    contentType: 'html' | 'css' | 'js' | 'json' | 'font' | 'generic'
  ): Promise<Buffer> {
    const modeMap = {
      html: constants.BROTLI_MODE_TEXT,
      css: constants.BROTLI_MODE_TEXT,
      js: constants.BROTLI_MODE_TEXT,
      json: constants.BROTLI_MODE_TEXT,
      font: constants.BROTLI_MODE_FONT,
      generic: constants.BROTLI_MODE_GENERIC,
    };

    const qualityMap = {
      html: 9,
      css: 9,
      js: 9,
      json: 8,
      font: 11,
      generic: 6,
    };

    const options = {
      params: {
        [constants.BROTLI_PARAM_QUALITY]: qualityMap[contentType],
        [constants.BROTLI_PARAM_MODE]: modeMap[contentType],
        [constants.BROTLI_PARAM_LGWIN]: 22,
      },
    };

    return await brotliCompressAsync(data, options);
  }

  /**
   * Create streaming compressor
   */
  createCompressStream(quality: number = 6): NodeJS.ReadWriteStream {
    const { createBrotliCompress } = require('zlib');
    return createBrotliCompress({
      params: {
        [constants.BROTLI_PARAM_QUALITY]: quality,
      },
    });
  }

  /**
   * Create streaming decompressor
   */
  createDecompressStream(): NodeJS.ReadWriteStream {
    const { createBrotliDecompress } = require('zlib');
    return createBrotliDecompress();
  }

  /**
   * Compare compression with different quality levels
   */
  async compareQualities(data: Buffer): Promise<Array<{
    quality: number;
    size: number;
    ratio: number;
    duration: number;
  }>> {
    const results = [];
    const qualities = [1, 4, 6, 9, 11];

    for (const quality of qualities) {
      const startTime = performance.now();
      const compressed = await brotliCompressAsync(data, {
        params: {
          [constants.BROTLI_PARAM_QUALITY]: quality,
        },
      });
      const duration = performance.now() - startTime;

      results.push({
        quality,
        size: compressed.length,
        ratio: data.length / compressed.length,
        duration,
      });
    }

    return results;
  }
}
