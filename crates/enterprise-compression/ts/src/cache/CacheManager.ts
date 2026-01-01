/**
 * Cache Manager
 * Multi-tier caching with compression support
 */

import LRUCache from 'lru-cache';
import {
  CacheEntry,
  CacheConfig,
  CacheStats,
  CacheTier,
  CacheError,
  CompressionAlgorithm,
} from '../types';

export class CacheManager<T = any> {
  private cache: LRUCache<string, CacheEntry<T>>;
  private stats: Map<CacheTier, CacheStats>;

  constructor(private config: CacheConfig) {
    this.cache = new LRUCache({
      max: config.maxEntries,
      maxSize: config.maxSize,
      sizeCalculation: (entry) => entry.size,
      ttl: config.ttl,
      updateAgeOnGet: true,
      updateAgeOnHas: true,
    });

    this.stats = new Map();
    this.initializeStats();
  }

  /**
   * Initialize statistics
   */
  private initializeStats(): void {
    for (const tier of this.config.tiers || [CacheTier.MEMORY]) {
      this.stats.set(tier, {
        tier,
        hits: 0,
        misses: 0,
        hitRate: 0,
        entries: 0,
        size: 0,
        maxSize: this.config.maxSize,
        utilization: 0,
        evictions: 0,
        compressionSavings: 0,
      });
    }
  }

  /**
   * Get value from cache
   */
  async get(key: string): Promise<T | undefined> {
    const entry = this.cache.get(key);

    if (entry) {
      entry.accessedAt = new Date();
      entry.hits++;
      this.updateStats(CacheTier.MEMORY, 'hit');
      return entry.value;
    }

    this.updateStats(CacheTier.MEMORY, 'miss');
    return undefined;
  }

  /**
   * Set value in cache
   */
  async set(
    key: string,
    value: T,
    options?: {
      ttl?: number;
      compress?: boolean;
      metadata?: any;
    }
  ): Promise<void> {
    try {
      const size = this.calculateSize(value);
      const shouldCompress =
        options?.compress !== false &&
        this.config.enableCompression &&
        size > (this.config.compressionThreshold || 1024);

      const entry: CacheEntry<T> = {
        key,
        value,
        size,
        tier: CacheTier.MEMORY,
        compressed: shouldCompress,
        algorithm: shouldCompress ? this.config.compressionAlgorithm : undefined,
        createdAt: new Date(),
        accessedAt: new Date(),
        expiresAt: options?.ttl
          ? new Date(Date.now() + options.ttl)
          : undefined,
        hits: 0,
        metadata: options?.metadata,
      };

      this.cache.set(key, entry);
      this.updateStats(CacheTier.MEMORY, 'set', size);
    } catch (error) {
      throw new CacheError(
        `Failed to set cache entry: ${error instanceof Error ? error.message : 'Unknown error'}`,
        CacheTier.MEMORY,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Delete from cache
   */
  async delete(key: string): Promise<boolean> {
    const deleted = this.cache.delete(key);
    if (deleted) {
      this.updateStats(CacheTier.MEMORY, 'delete');
    }
    return deleted;
  }

  /**
   * Check if key exists
   */
  async has(key: string): Promise<boolean> {
    return this.cache.has(key);
  }

  /**
   * Clear all cache
   */
  async clear(): Promise<void> {
    this.cache.clear();
    this.initializeStats();
  }

  /**
   * Get cache statistics
   */
  getStats(tier: CacheTier = CacheTier.MEMORY): CacheStats {
    return this.stats.get(tier) || this.createEmptyStats(tier);
  }

  /**
   * Get all statistics
   */
  getAllStats(): CacheStats[] {
    return Array.from(this.stats.values());
  }

  /**
   * Update statistics
   */
  private updateStats(
    tier: CacheTier,
    operation: 'hit' | 'miss' | 'set' | 'delete',
    size: number = 0
  ): void {
    const stats = this.stats.get(tier);
    if (!stats) return;

    switch (operation) {
      case 'hit':
        stats.hits++;
        break;
      case 'miss':
        stats.misses++;
        break;
      case 'set':
        stats.entries = this.cache.size;
        stats.size += size;
        break;
      case 'delete':
        stats.entries = this.cache.size;
        break;
    }

    stats.hitRate = stats.hits / (stats.hits + stats.misses) || 0;
    stats.utilization = stats.size / stats.maxSize;
  }

  /**
   * Calculate object size
   */
  private calculateSize(value: any): number {
    if (typeof value === 'string') {
      return Buffer.byteLength(value);
    }
    if (Buffer.isBuffer(value)) {
      return value.length;
    }
    return JSON.stringify(value).length;
  }

  /**
   * Create empty stats
   */
  private createEmptyStats(tier: CacheTier): CacheStats {
    return {
      tier,
      hits: 0,
      misses: 0,
      hitRate: 0,
      entries: 0,
      size: 0,
      maxSize: this.config.maxSize,
      utilization: 0,
      evictions: 0,
      compressionSavings: 0,
    };
  }

  /**
   * Get all keys
   */
  keys(): string[] {
    return Array.from(this.cache.keys());
  }

  /**
   * Get all values
   */
  values(): T[] {
    return Array.from(this.cache.values()).map(entry => entry.value);
  }

  /**
   * Get all entries
   */
  entries(): Array<[string, T]> {
    return Array.from(this.cache.entries()).map(([key, entry]) => [key, entry.value]);
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size;
  }

  /**
   * Prune expired entries
   */
  async prune(): Promise<number> {
    let pruned = 0;
    const now = new Date();

    for (const [key, entry] of this.cache.entries()) {
      if (entry.expiresAt && entry.expiresAt < now) {
        this.cache.delete(key);
        pruned++;
      }
    }

    return pruned;
  }
}
