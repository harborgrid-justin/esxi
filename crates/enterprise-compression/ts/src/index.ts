/**
 * Enterprise Compression & Data Optimization Engine
 * Complete compression solution with multi-algorithm support
 *
 * @module @enterprise/compression
 * @version 1.0.0
 */

// Types
export * from './types';

// Compression Algorithms
export { LZ4Compressor } from './algorithms/LZ4Compressor';
export { ZstdCompressor } from './algorithms/ZstdCompressor';
export { BrotliCompressor } from './algorithms/BrotliCompressor';
export { DeltaEncoder } from './algorithms/DeltaEncoder';
export { DictionaryCompressor } from './algorithms/DictionaryCompressor';
export { HuffmanEncoder } from './algorithms/HuffmanEncoder';
export { RLEEncoder } from './algorithms/RLEEncoder';

// Image Optimization
export { ImageOptimizer } from './image/ImageOptimizer';
export { WebPConverter } from './image/WebPConverter';
export { AVIFConverter } from './image/AVIFConverter';
export { ThumbnailGenerator } from './image/ThumbnailGenerator';
export { LazyLoadManager } from './image/LazyLoadManager';

// Data Optimization
export { JSONCompressor } from './data/JSONCompressor';
export { BinaryPacker } from './data/BinaryPacker';
export { SchemaOptimizer } from './data/SchemaOptimizer';
export { StreamCompressor } from './data/StreamCompressor';
export { ChunkManager } from './data/ChunkManager';

// Cache
export { CacheManager } from './cache/CacheManager';
export { CDNIntegration } from './cache/CDNIntegration';
export { EdgeCache } from './cache/EdgeCache';
export { MemoryCache } from './cache/MemoryCache';
export { DiskCache } from './cache/DiskCache';

// React Components
export { CompressionDashboard } from './components/CompressionDashboard';
export { OptimizationSettings } from './components/OptimizationSettings';
export { CompressionProgress } from './components/CompressionProgress';
export { SavingsReport } from './components/SavingsReport';
export { AssetOptimizer } from './components/AssetOptimizer';

// Services
export { CompressionService } from './services/CompressionService';
export { OptimizationPipeline, PipelineBuilder } from './services/OptimizationPipeline';
export { BandwidthAnalyzer } from './services/BandwidthAnalyzer';
export { StorageOptimizer } from './services/StorageOptimizer';

// Default exports for convenience
import { CompressionService } from './services/CompressionService';
import { ImageOptimizer } from './image/ImageOptimizer';
import { CacheManager } from './cache/CacheManager';
import { BandwidthAnalyzer } from './services/BandwidthAnalyzer';
import { StorageOptimizer } from './services/StorageOptimizer';

export default {
  CompressionService,
  ImageOptimizer,
  CacheManager,
  BandwidthAnalyzer,
  StorageOptimizer,
};
