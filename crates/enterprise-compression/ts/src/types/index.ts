/**
 * Enterprise Compression & Optimization Engine - Core Types
 * Comprehensive type definitions for compression, optimization, and caching
 */

// ============================================================================
// Compression Types
// ============================================================================

export enum CompressionAlgorithm {
  LZ4 = 'lz4',
  ZSTD = 'zstd',
  BROTLI = 'brotli',
  GZIP = 'gzip',
  DEFLATE = 'deflate',
  DELTA = 'delta',
  DICTIONARY = 'dictionary',
  HUFFMAN = 'huffman',
  RLE = 'rle',
  NONE = 'none',
}

export enum CompressionLevel {
  FASTEST = 1,
  FAST = 3,
  BALANCED = 6,
  HIGH = 9,
  MAXIMUM = 11,
}

export interface CompressionConfig {
  algorithm: CompressionAlgorithm;
  level: CompressionLevel;
  enableStreaming?: boolean;
  chunkSize?: number;
  dictionary?: Buffer;
  windowSize?: number;
  enableParallel?: boolean;
  threads?: number;
  metadata?: Record<string, any>;
}

export interface CompressionResult {
  compressed: Buffer;
  originalSize: number;
  compressedSize: number;
  compressionRatio: number;
  algorithm: CompressionAlgorithm;
  level: CompressionLevel;
  duration: number;
  throughput: number;
  metadata?: CompressionMetadata;
}

export interface CompressionMetadata {
  timestamp: Date;
  checksum: string;
  chunks?: number;
  dictionary?: string;
  version: string;
  custom?: Record<string, any>;
}

export interface DecompressionResult {
  decompressed: Buffer;
  originalSize: number;
  duration: number;
  algorithm: CompressionAlgorithm;
  verified: boolean;
}

// ============================================================================
// Stream Processing Types
// ============================================================================

export interface StreamProcessor {
  id: string;
  algorithm: CompressionAlgorithm;
  totalBytes: number;
  processedBytes: number;
  chunks: ChunkInfo[];
  startTime: Date;
  state: ProcessorState;
}

export enum ProcessorState {
  IDLE = 'idle',
  PROCESSING = 'processing',
  PAUSED = 'paused',
  COMPLETED = 'completed',
  ERROR = 'error',
}

export interface ChunkInfo {
  index: number;
  offset: number;
  size: number;
  compressedSize: number;
  checksum: string;
  timestamp: Date;
}

export interface StreamConfig {
  algorithm: CompressionAlgorithm;
  level: CompressionLevel;
  chunkSize: number;
  bufferSize: number;
  enableChecksum?: boolean;
  onProgress?: (progress: ProgressInfo) => void;
  onChunk?: (chunk: ChunkInfo) => void;
  onError?: (error: Error) => void;
}

export interface ProgressInfo {
  processedBytes: number;
  totalBytes: number;
  percentage: number;
  currentChunk: number;
  totalChunks: number;
  estimatedTimeRemaining: number;
  throughput: number;
}

// ============================================================================
// Image Optimization Types
// ============================================================================

export enum ImageFormat {
  JPEG = 'jpeg',
  PNG = 'png',
  WEBP = 'webp',
  AVIF = 'avif',
  GIF = 'gif',
  SVG = 'svg',
}

export interface ImageOptimizationConfig {
  format?: ImageFormat;
  quality?: number;
  width?: number;
  height?: number;
  fit?: 'cover' | 'contain' | 'fill' | 'inside' | 'outside';
  progressive?: boolean;
  lossless?: boolean;
  effort?: number;
  stripMetadata?: boolean;
  enableLazyLoad?: boolean;
}

export interface ImageOptimizationResult {
  optimized: Buffer;
  originalSize: number;
  optimizedSize: number;
  savingsPercent: number;
  format: ImageFormat;
  width: number;
  height: number;
  duration: number;
  metadata?: ImageMetadata;
}

export interface ImageMetadata {
  format: ImageFormat;
  width: number;
  height: number;
  hasAlpha: boolean;
  colorSpace: string;
  density?: number;
  orientation?: number;
  originalMetadata?: Record<string, any>;
}

export interface ThumbnailConfig {
  sizes: ThumbnailSize[];
  format: ImageFormat;
  quality: number;
  naming?: 'suffix' | 'directory';
  retina?: boolean;
}

export interface ThumbnailSize {
  width: number;
  height?: number;
  suffix: string;
  fit?: 'cover' | 'contain';
}

export interface ResponsiveImageSet {
  original: Buffer;
  thumbnails: Map<string, Buffer>;
  srcset: string;
  sizes: string;
  metadata: ImageMetadata;
}

// ============================================================================
// Data Optimization Types
// ============================================================================

export interface DataOptimizationProfile {
  name: string;
  algorithm: CompressionAlgorithm;
  level: CompressionLevel;
  enableMinification?: boolean;
  enableDedupe?: boolean;
  enableSchemaOptimization?: boolean;
  customTransforms?: DataTransform[];
}

export interface DataTransform {
  name: string;
  apply: (data: any) => any;
  revert: (data: any) => any;
}

export interface JSONCompressionResult {
  compressed: Buffer;
  originalSize: number;
  compressedSize: number;
  savingsPercent: number;
  minified: boolean;
  schema?: string;
}

export interface BinaryPackResult {
  packed: Buffer;
  schema: PackSchema;
  originalSize: number;
  packedSize: number;
  savingsPercent: number;
}

export interface PackSchema {
  version: number;
  fields: PackField[];
  checksum: string;
}

export interface PackField {
  name: string;
  type: 'uint8' | 'uint16' | 'uint32' | 'int8' | 'int16' | 'int32' | 'float32' | 'float64' | 'string' | 'buffer' | 'array' | 'object';
  optional?: boolean;
  default?: any;
  encoding?: 'utf8' | 'ascii' | 'base64';
}

// ============================================================================
// Cache Types
// ============================================================================

export enum CacheTier {
  MEMORY = 'memory',
  DISK = 'disk',
  CDN = 'cdn',
  EDGE = 'edge',
  DISTRIBUTED = 'distributed',
}

export interface CacheEntry<T = any> {
  key: string;
  value: T;
  size: number;
  tier: CacheTier;
  compressed: boolean;
  algorithm?: CompressionAlgorithm;
  createdAt: Date;
  accessedAt: Date;
  expiresAt?: Date;
  hits: number;
  metadata?: CacheMetadata;
}

export interface CacheMetadata {
  tags?: string[];
  version?: string;
  etag?: string;
  contentType?: string;
  custom?: Record<string, any>;
}

export interface CacheConfig {
  maxSize: number;
  maxEntries: number;
  ttl: number;
  enableCompression?: boolean;
  compressionThreshold?: number;
  compressionAlgorithm?: CompressionAlgorithm;
  evictionPolicy?: 'lru' | 'lfu' | 'fifo' | 'ttl';
  tiers?: CacheTier[];
}

export interface CacheStats {
  tier: CacheTier;
  hits: number;
  misses: number;
  hitRate: number;
  entries: number;
  size: number;
  maxSize: number;
  utilization: number;
  evictions: number;
  compressionSavings: number;
}

export interface CDNConfig {
  provider: 'cloudflare' | 'fastly' | 'akamai' | 'cloudfront' | 'custom';
  endpoint: string;
  apiKey?: string;
  zoneId?: string;
  enablePurge?: boolean;
  cacheControl?: string;
  customHeaders?: Record<string, string>;
}

export interface EdgeCacheConfig {
  regions: string[];
  replication?: 'sync' | 'async' | 'eventual';
  consistencyLevel?: 'strong' | 'eventual';
  ttl: number;
  maxAge?: number;
  staleWhileRevalidate?: number;
}

// ============================================================================
// Optimization Pipeline Types
// ============================================================================

export interface OptimizationPipeline {
  id: string;
  name: string;
  stages: OptimizationStage[];
  status: PipelineStatus;
  stats: PipelineStats;
}

export enum PipelineStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled',
}

export interface OptimizationStage {
  name: string;
  type: 'compression' | 'image' | 'data' | 'cache';
  config: any;
  enabled: boolean;
  order: number;
  execute: (input: Buffer, context: StageContext) => Promise<Buffer>;
}

export interface StageContext {
  pipeline: string;
  stage: string;
  metadata: Record<string, any>;
  stats: Map<string, number>;
}

export interface PipelineStats {
  totalProcessed: number;
  totalSavings: number;
  averageRatio: number;
  duration: number;
  throughput: number;
  errors: number;
  stageStats: Map<string, StageStats>;
}

export interface StageStats {
  processed: number;
  savings: number;
  duration: number;
  errors: number;
}

// ============================================================================
// Bandwidth & Analytics Types
// ============================================================================

export interface BandwidthMetrics {
  timestamp: Date;
  totalBytes: number;
  compressedBytes: number;
  savedBytes: number;
  savingsPercent: number;
  requestCount: number;
  averageSize: number;
  peakBandwidth: number;
  byAlgorithm: Map<CompressionAlgorithm, AlgorithmMetrics>;
}

export interface AlgorithmMetrics {
  algorithm: CompressionAlgorithm;
  usageCount: number;
  totalOriginalSize: number;
  totalCompressedSize: number;
  averageRatio: number;
  averageDuration: number;
  averageThroughput: number;
}

export interface StorageMetrics {
  totalSize: number;
  compressedSize: number;
  savingsPercent: number;
  fileCount: number;
  byType: Map<string, TypeMetrics>;
  byAlgorithm: Map<CompressionAlgorithm, number>;
}

export interface TypeMetrics {
  type: string;
  count: number;
  originalSize: number;
  compressedSize: number;
  savingsPercent: number;
}

// ============================================================================
// Service Types
// ============================================================================

export interface CompressionService {
  compress(data: Buffer, config: CompressionConfig): Promise<CompressionResult>;
  decompress(data: Buffer, algorithm: CompressionAlgorithm): Promise<DecompressionResult>;
  compressStream(config: StreamConfig): NodeJS.ReadWriteStream;
  decompressStream(algorithm: CompressionAlgorithm): NodeJS.ReadWriteStream;
  getBestAlgorithm(data: Buffer): Promise<CompressionAlgorithm>;
}

export interface OptimizationService {
  optimizeImage(image: Buffer, config: ImageOptimizationConfig): Promise<ImageOptimizationResult>;
  optimizeJSON(data: any, profile: DataOptimizationProfile): Promise<JSONCompressionResult>;
  optimizeBinary(data: any, schema: PackSchema): Promise<BinaryPackResult>;
  createPipeline(stages: OptimizationStage[]): OptimizationPipeline;
}

// ============================================================================
// Error Types
// ============================================================================

export class CompressionError extends Error {
  constructor(
    message: string,
    public algorithm: CompressionAlgorithm,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'CompressionError';
  }
}

export class OptimizationError extends Error {
  constructor(
    message: string,
    public stage: string,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'OptimizationError';
  }
}

export class CacheError extends Error {
  constructor(
    message: string,
    public tier: CacheTier,
    public originalError?: Error
  ) {
    super(message);
    this.name = 'CacheError';
  }
}
