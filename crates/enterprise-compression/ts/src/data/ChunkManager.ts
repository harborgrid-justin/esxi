/**
 * Chunk Manager
 * Manage chunked data processing and reassembly
 */

import { createHash } from 'crypto';
import { ChunkInfo, OptimizationError } from '../types';

export class ChunkManager {
  private static readonly DEFAULT_CHUNK_SIZE = 64 * 1024; // 64 KB
  private static readonly MAX_CHUNK_SIZE = 10 * 1024 * 1024; // 10 MB

  /**
   * Split data into chunks
   */
  splitIntoChunks(
    data: Buffer,
    chunkSize: number = ChunkManager.DEFAULT_CHUNK_SIZE
  ): ChunkInfo[] {
    if (chunkSize > ChunkManager.MAX_CHUNK_SIZE) {
      throw new OptimizationError(
        `Chunk size ${chunkSize} exceeds maximum ${ChunkManager.MAX_CHUNK_SIZE}`,
        'chunk-split',
      );
    }

    const chunks: ChunkInfo[] = [];
    let offset = 0;
    let index = 0;

    while (offset < data.length) {
      const size = Math.min(chunkSize, data.length - offset);
      const chunk = data.slice(offset, offset + size);

      chunks.push({
        index,
        offset,
        size,
        compressedSize: 0, // To be filled during compression
        checksum: createHash('md5').update(chunk).digest('hex'),
        timestamp: new Date(),
      });

      offset += size;
      index++;
    }

    return chunks;
  }

  /**
   * Reassemble chunks into data
   */
  assembleChunks(chunks: Array<{ info: ChunkInfo; data: Buffer }>): Buffer {
    // Sort by index
    const sorted = chunks.sort((a, b) => a.info.index - b.info.index);

    // Validate sequence
    this.validateChunkSequence(sorted.map(c => c.info));

    // Concatenate data
    return Buffer.concat(sorted.map(c => c.data));
  }

  /**
   * Validate chunk sequence
   */
  private validateChunkSequence(chunks: ChunkInfo[]): void {
    for (let i = 0; i < chunks.length; i++) {
      if (chunks[i].index !== i) {
        throw new OptimizationError(
          `Missing chunk at index ${i}`,
          'chunk-validation',
        );
      }
    }
  }

  /**
   * Verify chunk integrity
   */
  verifyChunk(data: Buffer, chunkInfo: ChunkInfo): boolean {
    const checksum = createHash('md5').update(data).digest('hex');
    return checksum === chunkInfo.checksum;
  }

  /**
   * Calculate optimal chunk size
   */
  calculateOptimalChunkSize(totalSize: number, targetChunks: number = 100): number {
    const chunkSize = Math.ceil(totalSize / targetChunks);

    // Align to 4KB boundaries for better I/O performance
    const aligned = Math.ceil(chunkSize / 4096) * 4096;

    return Math.min(
      Math.max(aligned, ChunkManager.DEFAULT_CHUNK_SIZE),
      ChunkManager.MAX_CHUNK_SIZE
    );
  }

  /**
   * Estimate chunk count
   */
  estimateChunkCount(totalSize: number, chunkSize: number): number {
    return Math.ceil(totalSize / chunkSize);
  }

  /**
   * Get chunk metadata
   */
  getChunkMetadata(chunks: ChunkInfo[]): {
    totalChunks: number;
    totalSize: number;
    totalCompressedSize: number;
    averageChunkSize: number;
    compressionRatio: number;
  } {
    const totalSize = chunks.reduce((sum, c) => sum + c.size, 0);
    const totalCompressedSize = chunks.reduce((sum, c) => sum + c.compressedSize, 0);

    return {
      totalChunks: chunks.length,
      totalSize,
      totalCompressedSize,
      averageChunkSize: totalSize / chunks.length,
      compressionRatio: totalSize / totalCompressedSize,
    };
  }

  /**
   * Find missing chunks
   */
  findMissingChunks(chunks: ChunkInfo[], expectedCount: number): number[] {
    const existing = new Set(chunks.map(c => c.index));
    const missing: number[] = [];

    for (let i = 0; i < expectedCount; i++) {
      if (!existing.has(i)) {
        missing.push(i);
      }
    }

    return missing;
  }

  /**
   * Merge overlapping chunks
   */
  mergeChunks(chunks: Array<{ info: ChunkInfo; data: Buffer }>): Buffer {
    if (chunks.length === 0) {
      return Buffer.alloc(0);
    }

    // Sort by offset
    const sorted = chunks.sort((a, b) => a.info.offset - b.info.offset);

    // Calculate total size
    const lastChunk = sorted[sorted.length - 1];
    const totalSize = lastChunk.info.offset + lastChunk.info.size;

    // Create result buffer
    const result = Buffer.alloc(totalSize);

    // Copy chunks
    for (const chunk of sorted) {
      chunk.data.copy(result, chunk.info.offset);
    }

    return result;
  }

  /**
   * Create chunk manifest
   */
  createManifest(chunks: ChunkInfo[]): string {
    return JSON.stringify({
      version: 1,
      timestamp: new Date().toISOString(),
      chunks: chunks.map(c => ({
        index: c.index,
        offset: c.offset,
        size: c.size,
        compressedSize: c.compressedSize,
        checksum: c.checksum,
      })),
    });
  }

  /**
   * Parse chunk manifest
   */
  parseManifest(manifest: string): ChunkInfo[] {
    const data = JSON.parse(manifest);

    return data.chunks.map((c: any) => ({
      index: c.index,
      offset: c.offset,
      size: c.size,
      compressedSize: c.compressedSize,
      checksum: c.checksum,
      timestamp: new Date(data.timestamp),
    }));
  }
}
