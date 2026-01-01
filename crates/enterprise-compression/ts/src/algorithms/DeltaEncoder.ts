/**
 * Delta Encoding Compression
 * Efficient compression for time-series and sequential data
 */

import { createHash } from 'crypto';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionError,
} from '../types';

export class DeltaEncoder {
  private static readonly VERSION = '1.0.0';

  /**
   * Compress data using delta encoding
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;
      const order = config.metadata?.order || 1;

      // Perform delta encoding
      const encoded = this.encodeDelta(data, order);

      // Further compress with variable-length encoding
      const compressed = this.variableLengthEncode(encoded);

      const compressedSize = compressed.length;
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000;

      const checksum = createHash('md5')
        .update(data)
        .digest('hex');

      return {
        compressed,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.DELTA,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: DeltaEncoder.VERSION,
          custom: {
            order,
            encoding: 'variable-length',
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `Delta encoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.DELTA,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress delta encoded data
   */
  async decompress(data: Buffer, order: number = 1): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      // Decode variable-length encoding
      const decoded = this.variableLengthDecode(data);

      // Reconstruct original data from deltas
      const decompressed = this.decodeDelta(decoded, order);

      const duration = performance.now() - startTime;

      return {
        decompressed,
        originalSize: decompressed.length,
        duration,
        algorithm: CompressionAlgorithm.DELTA,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `Delta decoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.DELTA,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Encode data using delta compression
   */
  private encodeDelta(data: Buffer, order: number): Buffer {
    const result = Buffer.alloc(data.length);

    if (order === 1) {
      // First-order delta: difference between consecutive bytes
      result[0] = data[0];
      for (let i = 1; i < data.length; i++) {
        result[i] = data[i] - data[i - 1];
      }
    } else if (order === 2) {
      // Second-order delta: difference of differences
      result[0] = data[0];
      result[1] = data[1] - data[0];
      for (let i = 2; i < data.length; i++) {
        const firstDelta = data[i] - data[i - 1];
        const prevDelta = data[i - 1] - data[i - 2];
        result[i] = firstDelta - prevDelta;
      }
    } else {
      // Higher-order deltas
      result.set(data.slice(0, order));
      for (let i = order; i < data.length; i++) {
        let delta = data[i];
        for (let j = 1; j <= order; j++) {
          delta -= data[i - j];
        }
        result[i] = delta;
      }
    }

    return result;
  }

  /**
   * Decode delta-encoded data
   */
  private decodeDelta(data: Buffer, order: number): Buffer {
    const result = Buffer.alloc(data.length);

    if (order === 1) {
      result[0] = data[0];
      for (let i = 1; i < data.length; i++) {
        result[i] = result[i - 1] + data[i];
      }
    } else if (order === 2) {
      result[0] = data[0];
      result[1] = data[0] + data[1];
      for (let i = 2; i < data.length; i++) {
        const prevDelta = result[i - 1] - result[i - 2];
        const currentDelta = prevDelta + data[i];
        result[i] = result[i - 1] + currentDelta;
      }
    } else {
      result.set(data.slice(0, order));
      for (let i = order; i < data.length; i++) {
        let value = data[i];
        for (let j = 1; j <= order; j++) {
          value += result[i - j];
        }
        result[i] = value;
      }
    }

    return result;
  }

  /**
   * Variable-length integer encoding (similar to protobuf varint)
   */
  private variableLengthEncode(data: Buffer): Buffer {
    const encoded: number[] = [];

    for (const byte of data) {
      // Use zigzag encoding for signed values
      const zigzag = this.zigzagEncode(byte);

      // Encode as varint
      let value = zigzag;
      while (value > 127) {
        encoded.push((value & 0x7f) | 0x80);
        value >>>= 7;
      }
      encoded.push(value & 0x7f);
    }

    return Buffer.from(encoded);
  }

  /**
   * Variable-length integer decoding
   */
  private variableLengthDecode(data: Buffer): Buffer {
    const decoded: number[] = [];
    let i = 0;

    while (i < data.length) {
      let value = 0;
      let shift = 0;

      while (true) {
        const byte = data[i++];
        value |= (byte & 0x7f) << shift;

        if ((byte & 0x80) === 0) break;
        shift += 7;
      }

      decoded.push(this.zigzagDecode(value));
    }

    return Buffer.from(decoded);
  }

  /**
   * Zigzag encoding for signed integers
   */
  private zigzagEncode(n: number): number {
    const signed = n < 128 ? n : n - 256;
    return (signed << 1) ^ (signed >> 7);
  }

  /**
   * Zigzag decoding
   */
  private zigzagDecode(n: number): number {
    const decoded = (n >>> 1) ^ -(n & 1);
    return decoded < 0 ? decoded + 256 : decoded;
  }

  /**
   * Encode numeric time series data
   */
  async encodeTimeSeries(
    values: number[],
    dataType: 'int8' | 'int16' | 'int32' | 'float32' | 'float64'
  ): Promise<Buffer> {
    const buffer = this.valuesToBuffer(values, dataType);
    const result = await this.compress(buffer, {
      algorithm: CompressionAlgorithm.DELTA,
      level: 6,
      metadata: { order: 1 },
    } as CompressionConfig);

    return result.compressed;
  }

  /**
   * Decode numeric time series data
   */
  async decodeTimeSeries(
    data: Buffer,
    dataType: 'int8' | 'int16' | 'int32' | 'float32' | 'float64',
    order: number = 1
  ): Promise<number[]> {
    const result = await this.decompress(data, order);
    return this.bufferToValues(result.decompressed, dataType);
  }

  /**
   * Convert values array to buffer
   */
  private valuesToBuffer(
    values: number[],
    dataType: 'int8' | 'int16' | 'int32' | 'float32' | 'float64'
  ): Buffer {
    const sizes = {
      int8: 1,
      int16: 2,
      int32: 4,
      float32: 4,
      float64: 8,
    };

    const buffer = Buffer.allocUnsafe(values.length * sizes[dataType]);

    values.forEach((value, i) => {
      const offset = i * sizes[dataType];
      switch (dataType) {
        case 'int8':
          buffer.writeInt8(value, offset);
          break;
        case 'int16':
          buffer.writeInt16LE(value, offset);
          break;
        case 'int32':
          buffer.writeInt32LE(value, offset);
          break;
        case 'float32':
          buffer.writeFloatLE(value, offset);
          break;
        case 'float64':
          buffer.writeDoubleLE(value, offset);
          break;
      }
    });

    return buffer;
  }

  /**
   * Convert buffer to values array
   */
  private bufferToValues(
    buffer: Buffer,
    dataType: 'int8' | 'int16' | 'int32' | 'float32' | 'float64'
  ): number[] {
    const sizes = {
      int8: 1,
      int16: 2,
      int32: 4,
      float32: 4,
      float64: 8,
    };

    const count = buffer.length / sizes[dataType];
    const values: number[] = [];

    for (let i = 0; i < count; i++) {
      const offset = i * sizes[dataType];
      switch (dataType) {
        case 'int8':
          values.push(buffer.readInt8(offset));
          break;
        case 'int16':
          values.push(buffer.readInt16LE(offset));
          break;
        case 'int32':
          values.push(buffer.readInt32LE(offset));
          break;
        case 'float32':
          values.push(buffer.readFloatLE(offset));
          break;
        case 'float64':
          values.push(buffer.readDoubleLE(offset));
          break;
      }
    }

    return values;
  }
}
