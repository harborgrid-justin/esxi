/**
 * Dictionary-Based Compression
 * Optimized for repetitive content with shared patterns
 */

import { createHash } from 'crypto';
import * as pako from 'pako';
import {
  CompressionConfig,
  CompressionResult,
  DecompressionResult,
  CompressionAlgorithm,
  CompressionError,
} from '../types';

interface DictionaryEntry {
  pattern: Buffer;
  frequency: number;
  positions: number[];
}

export class DictionaryCompressor {
  private static readonly VERSION = '1.0.0';
  private static readonly MIN_PATTERN_LENGTH = 4;
  private static readonly MAX_PATTERN_LENGTH = 255;
  private static readonly MAX_DICTIONARY_SIZE = 32768;

  /**
   * Compress data using dictionary-based compression
   */
  async compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      const originalSize = data.length;

      // Build dictionary from data or use provided dictionary
      const dictionary = config.dictionary || (await this.buildDictionary(data));

      // Compress using dictionary
      const compressed = this.compressWithDictionary(data, dictionary);

      // Store dictionary with compressed data
      const result = this.packWithDictionary(compressed, dictionary);

      const compressedSize = result.length;
      const duration = performance.now() - startTime;
      const compressionRatio = originalSize / compressedSize;
      const throughput = (originalSize / duration) * 1000;

      const checksum = createHash('md5')
        .update(data)
        .digest('hex');

      return {
        compressed: result,
        originalSize,
        compressedSize,
        compressionRatio,
        algorithm: CompressionAlgorithm.DICTIONARY,
        level: config.level,
        duration,
        throughput,
        metadata: {
          timestamp: new Date(),
          checksum,
          version: DictionaryCompressor.VERSION,
          custom: {
            dictionarySize: dictionary.length,
            dictionaryChecksum: createHash('md5').update(dictionary).digest('hex'),
          },
        },
      };
    } catch (error) {
      throw new CompressionError(
        `Dictionary compression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.DICTIONARY,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Decompress dictionary-compressed data
   */
  async decompress(data: Buffer): Promise<DecompressionResult> {
    const startTime = performance.now();

    try {
      if (!data || data.length === 0) {
        throw new Error('Input data is empty');
      }

      // Unpack dictionary and compressed data
      const { dictionary, compressed } = this.unpackWithDictionary(data);

      // Decompress using dictionary
      const decompressed = this.decompressWithDictionary(compressed, dictionary);

      const duration = performance.now() - startTime;

      return {
        decompressed,
        originalSize: decompressed.length,
        duration,
        algorithm: CompressionAlgorithm.DICTIONARY,
        verified: true,
      };
    } catch (error) {
      throw new CompressionError(
        `Dictionary decompression failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CompressionAlgorithm.DICTIONARY,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Build optimal dictionary from data
   */
  async buildDictionary(data: Buffer, maxSize: number = DictionaryCompressor.MAX_DICTIONARY_SIZE): Promise<Buffer> {
    // Find frequent patterns
    const patterns = this.findFrequentPatterns(data);

    // Sort by compression value (frequency * length)
    const sorted = patterns.sort((a, b) => {
      const valueA = a.frequency * a.pattern.length;
      const valueB = b.frequency * b.pattern.length;
      return valueB - valueA;
    });

    // Build dictionary from top patterns
    const dictionary: Buffer[] = [];
    let totalSize = 0;

    for (const entry of sorted) {
      if (totalSize + entry.pattern.length > maxSize) break;
      dictionary.push(entry.pattern);
      totalSize += entry.pattern.length;
    }

    return Buffer.concat(dictionary);
  }

  /**
   * Find frequent patterns in data
   */
  private findFrequentPatterns(data: Buffer): DictionaryEntry[] {
    const patterns = new Map<string, DictionaryEntry>();
    const minFreq = 3;

    // Sliding window to find patterns
    for (let len = DictionaryCompressor.MIN_PATTERN_LENGTH; len <= DictionaryCompressor.MAX_PATTERN_LENGTH; len++) {
      for (let i = 0; i <= data.length - len; i++) {
        const pattern = data.slice(i, i + len);
        const key = pattern.toString('hex');

        const entry = patterns.get(key);
        if (entry) {
          entry.frequency++;
          entry.positions.push(i);
        } else {
          patterns.set(key, {
            pattern,
            frequency: 1,
            positions: [i],
          });
        }
      }
    }

    // Filter by minimum frequency
    return Array.from(patterns.values()).filter(
      entry => entry.frequency >= minFreq
    );
  }

  /**
   * Compress data using dictionary
   */
  private compressWithDictionary(data: Buffer, dictionary: Buffer): Buffer {
    const encoded: number[] = [];
    let pos = 0;

    while (pos < data.length) {
      const match = this.findLongestMatch(data, pos, dictionary);

      if (match) {
        // Encode as dictionary reference
        encoded.push(0xff); // Dictionary marker
        encoded.push(match.offset >> 8);
        encoded.push(match.offset & 0xff);
        encoded.push(match.length);
        pos += match.length;
      } else {
        // Encode literal byte
        encoded.push(data[pos]);
        pos++;
      }
    }

    // Further compress with deflate
    return pako.deflate(Buffer.from(encoded));
  }

  /**
   * Decompress data using dictionary
   */
  private decompressWithDictionary(data: Buffer, dictionary: Buffer): Buffer {
    // Inflate data
    const inflated = pako.inflate(data);
    const decoded: number[] = [];
    let pos = 0;

    while (pos < inflated.length) {
      if (inflated[pos] === 0xff) {
        // Dictionary reference
        const offset = (inflated[pos + 1] << 8) | inflated[pos + 2];
        const length = inflated[pos + 3];
        const pattern = dictionary.slice(offset, offset + length);
        decoded.push(...pattern);
        pos += 4;
      } else {
        // Literal byte
        decoded.push(inflated[pos]);
        pos++;
      }
    }

    return Buffer.from(decoded);
  }

  /**
   * Find longest match in dictionary
   */
  private findLongestMatch(
    data: Buffer,
    pos: number,
    dictionary: Buffer
  ): { offset: number; length: number } | null {
    let bestMatch: { offset: number; length: number } | null = null;
    let maxLength = 0;

    for (let offset = 0; offset < dictionary.length; offset++) {
      let length = 0;
      while (
        offset + length < dictionary.length &&
        pos + length < data.length &&
        dictionary[offset + length] === data[pos + length]
      ) {
        length++;
      }

      if (length >= DictionaryCompressor.MIN_PATTERN_LENGTH && length > maxLength) {
        maxLength = length;
        bestMatch = { offset, length };
      }
    }

    return bestMatch;
  }

  /**
   * Pack compressed data with dictionary
   */
  private packWithDictionary(compressed: Buffer, dictionary: Buffer): Buffer {
    const header = Buffer.allocUnsafe(8);
    header.writeUInt32LE(dictionary.length, 0);
    header.writeUInt32LE(compressed.length, 4);

    return Buffer.concat([header, dictionary, compressed]);
  }

  /**
   * Unpack dictionary and compressed data
   */
  private unpackWithDictionary(data: Buffer): {
    dictionary: Buffer;
    compressed: Buffer;
  } {
    const dictLength = data.readUInt32LE(0);
    const compressedLength = data.readUInt32LE(4);

    const dictionary = data.slice(8, 8 + dictLength);
    const compressed = data.slice(8 + dictLength, 8 + dictLength + compressedLength);

    return { dictionary, compressed };
  }

  /**
   * Train dictionary from multiple samples
   */
  async trainDictionary(samples: Buffer[], maxSize: number = DictionaryCompressor.MAX_DICTIONARY_SIZE): Promise<Buffer> {
    const combined = Buffer.concat(samples);
    return this.buildDictionary(combined, maxSize);
  }

  /**
   * Evaluate dictionary effectiveness
   */
  async evaluateDictionary(data: Buffer, dictionary: Buffer): Promise<{
    coveragePercent: number;
    estimatedRatio: number;
    patternCount: number;
  }> {
    let coveredBytes = 0;
    let patternCount = 0;
    let pos = 0;

    while (pos < data.length) {
      const match = this.findLongestMatch(data, pos, dictionary);
      if (match) {
        coveredBytes += match.length;
        patternCount++;
        pos += match.length;
      } else {
        pos++;
      }
    }

    return {
      coveragePercent: (coveredBytes / data.length) * 100,
      estimatedRatio: data.length / (data.length - coveredBytes + patternCount * 4),
      patternCount,
    };
  }
}
