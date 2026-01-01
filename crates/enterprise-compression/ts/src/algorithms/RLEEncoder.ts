/**
 * Run-Length Encoding (RLE) Compression
 * Simple and efficient for data with many repeated values
 */

import { createHash } from 'crypto';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionError,
} from '../types';

export class RLEEncoder {
  private static readonly VERSION = '1.0.0';
  private static readonly MAX_RUN_LENGTH = 255;

  /**
   * Compress data using run-length encoding
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;

      // Perform RLE encoding
      const encoded = this.encode(data);

      const compressedSize = encoded.length;
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000;

      const checksum = createHash('md5')
        .update(data)
        .digest('hex');

      return {
        compressed: encoded,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.RLE,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: RLEEncoder.VERSION,
          custom: {
            encoding: 'standard-rle',
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `RLE encoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.RLE,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress RLE encoded data
   */
  async decompress(data: Buffer): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      // Perform RLE decoding
      const decoded = this.decode(data);

      const duration = performance.now() - startTime;

      return {
        decompressed: decoded,
        originalSize: decoded.length,
        duration,
        algorithm: CompressionAlgorithm.RLE,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `RLE decoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.RLE,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Encode data using RLE
   */
  private encode(data: Buffer): Buffer {
    const encoded: number[] = [];
    let i = 0;

    while (i < data.length) {
      const currentByte = data[i];
      let runLength = 1;

      // Count consecutive identical bytes
      while (
        i + runLength < data.length &&
        data[i + runLength] === currentByte &&
        runLength < RLEEncoder.MAX_RUN_LENGTH
      ) {
        runLength++;
      }

      if (runLength >= 3) {
        // Encode as run: [marker][length][value]
        encoded.push(0xff); // RLE marker
        encoded.push(runLength);
        encoded.push(currentByte);
      } else {
        // Encode literally
        for (let j = 0; j < runLength; j++) {
          // Escape marker byte if it appears in data
          if (currentByte === 0xff) {
            encoded.push(0xff);
            encoded.push(0);
          } else {
            encoded.push(currentByte);
          }
        }
      }

      i += runLength;
    }

    return Buffer.from(encoded);
  }

  /**
   * Decode RLE encoded data
   */
  private decode(data: Buffer): Buffer {
    const decoded: number[] = [];
    let i = 0;

    while (i < data.length) {
      if (data[i] === 0xff) {
        if (i + 1 < data.length && data[i + 1] === 0) {
          // Escaped marker byte
          decoded.push(0xff);
          i += 2;
        } else if (i + 2 < data.length) {
          // RLE run
          const runLength = data[i + 1];
          const value = data[i + 2];
          for (let j = 0; j < runLength; j++) {
            decoded.push(value);
          }
          i += 3;
        } else {
          throw new Error('Invalid RLE data: incomplete run at end');
        }
      } else {
        // Literal byte
        decoded.push(data[i]);
        i++;
      }
    }

    return Buffer.from(decoded);
  }

  /**
   * Advanced RLE with bit packing for small run lengths
   */
  async encodeAdvanced(data: Buffer): Promise<Buffer> {
    const encoded: number[] = [];
    let i = 0;

    while (i < data.length) {
      const currentByte = data[i];
      let runLength = 1;

      while (
        i + runLength < data.length &&
        data[i + runLength] === currentByte &&
        runLength < RLEEncoder.MAX_RUN_LENGTH
      ) {
        runLength++;
      }

      if (runLength >= 3) {
        // Use compact encoding for common run lengths
        if (runLength <= 15) {
          // Pack length in 4 bits: [1111][length-3][value]
          encoded.push(0xf0 | (runLength - 3));
          encoded.push(currentByte);
        } else {
          // Full encoding: [11111111][length][value]
          encoded.push(0xff);
          encoded.push(runLength);
          encoded.push(currentByte);
        }
      } else {
        // Literal bytes
        for (let j = 0; j < runLength; j++) {
          const byte = data[i + j];
          if ((byte & 0xf0) === 0xf0) {
            // Escape bytes starting with 1111
            encoded.push(0);
            encoded.push(byte);
          } else {
            encoded.push(byte);
          }
        }
      }

      i += runLength;
    }

    return Buffer.from(encoded);
  }

  /**
   * Decode advanced RLE
   */
  async decodeAdvanced(data: Buffer): Promise<Buffer> {
    const decoded: number[] = [];
    let i = 0;

    while (i < data.length) {
      const byte = data[i];

      if ((byte & 0xf0) === 0xf0) {
        if (byte === 0xff) {
          // Full run encoding
          const runLength = data[i + 1];
          const value = data[i + 2];
          for (let j = 0; j < runLength; j++) {
            decoded.push(value);
          }
          i += 3;
        } else {
          // Compact run encoding
          const runLength = (byte & 0x0f) + 3;
          const value = data[i + 1];
          for (let j = 0; j < runLength; j++) {
            decoded.push(value);
          }
          i += 2;
        }
      } else if (byte === 0) {
        // Escaped byte
        decoded.push(data[i + 1]);
        i += 2;
      } else {
        // Literal byte
        decoded.push(byte);
        i++;
      }
    }

    return Buffer.from(decoded);
  }

  /**
   * Analyze data for RLE suitability
   */
  analyzeData(data: Buffer): {
    averageRunLength: number;
    maxRunLength: number;
    runCount: number;
    repeatedBytes: number;
    estimatedRatio: number;
  } {
    let runCount = 0;
    let totalRunLength = 0;
    let maxRunLength = 0;
    let repeatedBytes = 0;
    let i = 0;

    while (i < data.length) {
      const currentByte = data[i];
      let runLength = 1;

      while (
        i + runLength < data.length &&
        data[i + runLength] === currentByte
      ) {
        runLength++;
      }

      if (runLength > 1) {
        runCount++;
        totalRunLength += runLength;
        maxRunLength = Math.max(maxRunLength, runLength);
        repeatedBytes += runLength - 1;
      }

      i += runLength;
    }

    // Estimate compression ratio
    // Each run of 3+ bytes becomes 3 bytes (marker + length + value)
    const estimatedCompressedSize =
      data.length - repeatedBytes + runCount * 3;
    const estimatedRatio = data.length / estimatedCompressedSize;

    return {
      averageRunLength: runCount > 0 ? totalRunLength / runCount : 1,
      maxRunLength,
      runCount,
      repeatedBytes,
      estimatedRatio,
    };
  }

  /**
   * Check if RLE is suitable for this data
   */
  isSuitable(data: Buffer): boolean {
    const analysis = this.analyzeData(data);
    // RLE is suitable if we can save at least 10% and have decent run counts
    return analysis.estimatedRatio > 1.1 && analysis.runCount > data.length / 100;
  }

  /**
   * Encode image data (optimized for scanlines)
   */
  async encodeImage(
    data: Buffer,
    width: number,
    height: number,
    channels: number
  ): Promise<Buffer> {
    const encoded: number[] = [];
    const bytesPerRow = width * channels;

    // Process scanline by scanline
    for (let row = 0; row < height; row++) {
      const rowStart = row * bytesPerRow;
      const rowData = data.slice(rowStart, rowStart + bytesPerRow);

      // Encode each row
      const rowEncoded = await this.encode(rowData, {
        algorithm: CompressionAlgorithm.RLE,
        level: 6,
      } as CompressionConfig);

      encoded.push(...rowEncoded.compressed);
    }

    return Buffer.from(encoded);
  }
}
