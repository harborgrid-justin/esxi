/**
 * Stream Compressor
 * Compress data streams with chunking support
 */

import { Transform } from 'stream';
import * as pako from 'pako';
import { createHash } from 'crypto';
import {
  StreamConfig,
  StreamProcessor,
  ChunkInfo,
  ProcessorState,
  ProgressInfo,
  CompressionAlgorithm,
} from '../types';

export class StreamCompressor {
  private processors = new Map<string, StreamProcessor>();

  /**
   * Create compression stream
   */
  createCompressionStream(config: StreamConfig): NodeJS.ReadWriteStream {
    const processor = this.createProcessor(config);
    this.processors.set(processor.id, processor);

    const transform = new Transform({
      transform: (chunk: Buffer, encoding, callback) => {
        this.processChunk(processor, chunk, config)
          .then(compressed => callback(null, compressed))
          .catch(error => callback(error));
      },
      flush: (callback) => {
        this.finalizeProcessor(processor, config);
        callback();
      },
    });

    return transform;
  }

  /**
   * Create decompression stream
   */
  createDecompressionStream(algorithm: CompressionAlgorithm): NodeJS.ReadWriteStream {
    return new Transform({
      transform: (chunk: Buffer, encoding, callback) => {
        this.decompressChunk(chunk, algorithm)
          .then(decompressed => callback(null, decompressed))
          .catch(error => callback(error));
      },
    });
  }

  /**
   * Create stream processor
   */
  private createProcessor(config: StreamConfig): StreamProcessor {
    return {
      id: this.generateId(),
      algorithm: config.algorithm,
      totalBytes: 0,
      processedBytes: 0,
      chunks: [],
      startTime: new Date(),
      state: ProcessorState.PROCESSING,
    };
  }

  /**
   * Process single chunk
   */
  private async processChunk(
    processor: StreamProcessor,
    chunk: Buffer,
    config: StreamConfig
  ): Promise<Buffer> {
    const compressed = await this.compressChunk(chunk, config.algorithm);

    const chunkInfo: ChunkInfo = {
      index: processor.chunks.length,
      offset: processor.processedBytes,
      size: chunk.length,
      compressedSize: compressed.length,
      checksum: config.enableChecksum
        ? createHash('md5').update(chunk).digest('hex')
        : '',
      timestamp: new Date(),
    };

    processor.chunks.push(chunkInfo);
    processor.processedBytes += chunk.length;
    processor.totalBytes += chunk.length;

    if (config.onChunk) {
      config.onChunk(chunkInfo);
    }

    if (config.onProgress) {
      config.onProgress(this.calculateProgress(processor));
    }

    return compressed;
  }

  /**
   * Compress chunk based on algorithm
   */
  private async compressChunk(
    chunk: Buffer,
    algorithm: CompressionAlgorithm
  ): Promise<Buffer> {
    switch (algorithm) {
      case CompressionAlgorithm.GZIP:
        return Buffer.from(pako.gzip(chunk));
      case CompressionAlgorithm.DEFLATE:
        return Buffer.from(pako.deflate(chunk));
      default:
        return Buffer.from(pako.gzip(chunk));
    }
  }

  /**
   * Decompress chunk
   */
  private async decompressChunk(
    chunk: Buffer,
    algorithm: CompressionAlgorithm
  ): Promise<Buffer> {
    switch (algorithm) {
      case CompressionAlgorithm.GZIP:
        return Buffer.from(pako.ungzip(chunk));
      case CompressionAlgorithm.DEFLATE:
        return Buffer.from(pako.inflate(chunk));
      default:
        return Buffer.from(pako.ungzip(chunk));
    }
  }

  /**
   * Calculate progress
   */
  private calculateProgress(processor: StreamProcessor): ProgressInfo {
    const elapsed = Date.now() - processor.startTime.getTime();
    const throughput = processor.processedBytes / (elapsed / 1000);
    const remaining = processor.totalBytes - processor.processedBytes;
    const estimatedTimeRemaining = remaining / throughput;

    return {
      processedBytes: processor.processedBytes,
      totalBytes: processor.totalBytes,
      percentage: (processor.processedBytes / processor.totalBytes) * 100,
      currentChunk: processor.chunks.length,
      totalChunks: Math.ceil(processor.totalBytes / 65536), // Estimate
      estimatedTimeRemaining,
      throughput,
    };
  }

  /**
   * Finalize processor
   */
  private finalizeProcessor(processor: StreamProcessor, config: StreamConfig): void {
    processor.state = ProcessorState.COMPLETED;

    if (config.onProgress) {
      config.onProgress({
        processedBytes: processor.totalBytes,
        totalBytes: processor.totalBytes,
        percentage: 100,
        currentChunk: processor.chunks.length,
        totalChunks: processor.chunks.length,
        estimatedTimeRemaining: 0,
        throughput: 0,
      });
    }
  }

  /**
   * Get processor status
   */
  getProcessorStatus(processorId: string): StreamProcessor | undefined {
    return this.processors.get(processorId);
  }

  /**
   * Pause processor
   */
  pauseProcessor(processorId: string): void {
    const processor = this.processors.get(processorId);
    if (processor) {
      processor.state = ProcessorState.PAUSED;
    }
  }

  /**
   * Resume processor
   */
  resumeProcessor(processorId: string): void {
    const processor = this.processors.get(processorId);
    if (processor && processor.state === ProcessorState.PAUSED) {
      processor.state = ProcessorState.PROCESSING;
    }
  }

  /**
   * Cancel processor
   */
  cancelProcessor(processorId: string): void {
    this.processors.delete(processorId);
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return `proc_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Get all processors
   */
  getAllProcessors(): StreamProcessor[] {
    return Array.from(this.processors.values());
  }
}
