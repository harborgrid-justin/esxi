/**
 * Edge Cache
 * Distributed edge caching for global content delivery
 */

import { EdgeCacheConfig, CacheEntry, CacheTier } from '../types';

export class EdgeCache<T = any> {
  private regionCaches = new Map<string, Map<string, CacheEntry<T>>>();

  constructor(private config: EdgeCacheConfig) {
    this.initializeRegions();
  }

  /**
   * Initialize regional caches
   */
  private initializeRegions(): void {
    for (const region of this.config.regions) {
      this.regionCaches.set(region, new Map());
    }
  }

  /**
   * Set value in edge cache
   */
  async set(
    key: string,
    value: T,
    options?: {
      regions?: string[];
      ttl?: number;
    }
  ): Promise<void> {
    const regions = options?.regions || this.config.regions;
    const expiresAt = options?.ttl
      ? new Date(Date.now() + options.ttl)
      : new Date(Date.now() + this.config.ttl);

    const entry: CacheEntry<T> = {
      key,
      value,
      size: this.calculateSize(value),
      tier: CacheTier.EDGE,
      compressed: false,
      createdAt: new Date(),
      accessedAt: new Date(),
      expiresAt,
      hits: 0,
    };

    if (this.config.replication === 'sync') {
      await this.replicateSync(entry, regions);
    } else {
      this.replicateAsync(entry, regions);
    }
  }

  /**
   * Get value from edge cache
   */
  async get(key: string, region?: string): Promise<T | undefined> {
    const targetRegion = region || this.getNearestRegion();
    const cache = this.regionCaches.get(targetRegion);

    if (!cache) return undefined;

    const entry = cache.get(key);
    if (!entry) return undefined;

    if (entry.expiresAt && entry.expiresAt < new Date()) {
      cache.delete(key);
      return undefined;
    }

    entry.accessedAt = new Date();
    entry.hits++;

    return entry.value;
  }

  /**
   * Delete from edge cache
   */
  async delete(key: string, regions?: string[]): Promise<void> {
    const targetRegions = regions || this.config.regions;

    for (const region of targetRegions) {
      const cache = this.regionCaches.get(region);
      if (cache) {
        cache.delete(key);
      }
    }
  }

  /**
   * Clear region cache
   */
  async clearRegion(region: string): Promise<void> {
    const cache = this.regionCaches.get(region);
    if (cache) {
      cache.clear();
    }
  }

  /**
   * Synchronous replication
   */
  private async replicateSync(
    entry: CacheEntry<T>,
    regions: string[]
  ): Promise<void> {
    await Promise.all(
      regions.map(async region => {
        const cache = this.regionCaches.get(region);
        if (cache) {
          cache.set(entry.key, { ...entry });
        }
      })
    );
  }

  /**
   * Asynchronous replication
   */
  private replicateAsync(entry: CacheEntry<T>, regions: string[]): void {
    setImmediate(() => {
      for (const region of regions) {
        const cache = this.regionCaches.get(region);
        if (cache) {
          cache.set(entry.key, { ...entry });
        }
      }
    });
  }

  /**
   * Get nearest region (simplified)
   */
  private getNearestRegion(): string {
    return this.config.regions[0] || 'default';
  }

  /**
   * Get cache statistics by region
   */
  getRegionStats(region: string): {
    entries: number;
    size: number;
    hits: number;
  } {
    const cache = this.regionCaches.get(region);
    if (!cache) {
      return { entries: 0, size: 0, hits: 0 };
    }

    let totalSize = 0;
    let totalHits = 0;

    for (const entry of cache.values()) {
      totalSize += entry.size;
      totalHits += entry.hits;
    }

    return {
      entries: cache.size,
      size: totalSize,
      hits: totalHits,
    };
  }

  /**
   * Calculate value size
   */
  private calculateSize(value: T): number {
    if (typeof value === 'string') {
      return Buffer.byteLength(value);
    }
    if (Buffer.isBuffer(value)) {
      return value.length;
    }
    return JSON.stringify(value).length;
  }

  /**
   * Prune expired entries across all regions
   */
  async pruneExpired(): Promise<number> {
    let pruned = 0;
    const now = new Date();

    for (const cache of this.regionCaches.values()) {
      for (const [key, entry] of cache.entries()) {
        if (entry.expiresAt && entry.expiresAt < now) {
          cache.delete(key);
          pruned++;
        }
      }
    }

    return pruned;
  }
}
