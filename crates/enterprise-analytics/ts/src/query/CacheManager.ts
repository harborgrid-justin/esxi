/**
 * Query Result Caching System
 * @module @harborgrid/enterprise-analytics/query
 */

import Dexie, { Table } from 'dexie';

interface CacheEntry<T = unknown> {
  key: string;
  value: T;
  timestamp: number;
  ttl: number;
  hits: number;
  size: number;
  tags?: string[];
}

export interface CacheStats {
  totalEntries: number;
  totalSize: number;
  hitRate: number;
  missRate: number;
  evictions: number;
}

export interface CacheConfig {
  maxSize?: number; // Maximum cache size in bytes
  maxEntries?: number; // Maximum number of entries
  defaultTTL?: number; // Default TTL in milliseconds
  strategy?: 'lru' | 'lfu' | 'fifo';
  enablePersistence?: boolean;
}

export class CacheManager extends Dexie {
  private cache!: Table<CacheEntry, string>;
  private memoryCache: Map<string, CacheEntry>;
  private config: Required<CacheConfig>;
  private stats: {
    hits: number;
    misses: number;
    evictions: number;
  };

  constructor(config: CacheConfig = {}) {
    super('AnalyticsCacheDB');

    this.config = {
      maxSize: config.maxSize || 100 * 1024 * 1024, // 100MB default
      maxEntries: config.maxEntries || 1000,
      defaultTTL: config.defaultTTL || 300000, // 5 minutes default
      strategy: config.strategy || 'lru',
      enablePersistence: config.enablePersistence ?? true,
    };

    this.memoryCache = new Map();
    this.stats = {
      hits: 0,
      misses: 0,
      evictions: 0,
    };

    this.version(1).stores({
      cache: 'key, timestamp, ttl, tags',
    });

    if (this.config.enablePersistence) {
      this.initialize();
    }
  }

  private async initialize(): Promise<void> {
    try {
      await this.open();
    } catch (error) {
      console.warn('IndexedDB not available, using memory cache only', error);
    }
  }

  // ============================================================================
  // Core Cache Operations
  // ============================================================================

  async get<T = unknown>(key: string): Promise<T | null> {
    // Try memory cache first
    const memEntry = this.memoryCache.get(key);
    if (memEntry) {
      if (this.isExpired(memEntry)) {
        await this.delete(key);
        this.stats.misses++;
        return null;
      }

      memEntry.hits++;
      this.stats.hits++;
      return memEntry.value as T;
    }

    // Try persistent cache
    if (this.config.enablePersistence) {
      try {
        const entry = await this.cache.get(key);
        if (entry) {
          if (this.isExpired(entry)) {
            await this.delete(key);
            this.stats.misses++;
            return null;
          }

          // Promote to memory cache
          this.memoryCache.set(key, entry);
          entry.hits++;
          await this.cache.put(entry);
          this.stats.hits++;
          return entry.value as T;
        }
      } catch (error) {
        console.warn('Error reading from persistent cache', error);
      }
    }

    this.stats.misses++;
    return null;
  }

  async set<T = unknown>(
    key: string,
    value: T,
    ttl: number = this.config.defaultTTL,
    tags?: string[]
  ): Promise<void> {
    const entry: CacheEntry<T> = {
      key,
      value,
      timestamp: Date.now(),
      ttl,
      hits: 0,
      size: this.estimateSize(value),
      tags,
    };

    // Check if we need to evict entries
    await this.evictIfNeeded(entry.size);

    // Set in memory cache
    this.memoryCache.set(key, entry);

    // Set in persistent cache
    if (this.config.enablePersistence) {
      try {
        await this.cache.put(entry);
      } catch (error) {
        console.warn('Error writing to persistent cache', error);
      }
    }
  }

  async delete(key: string): Promise<void> {
    this.memoryCache.delete(key);

    if (this.config.enablePersistence) {
      try {
        await this.cache.delete(key);
      } catch (error) {
        console.warn('Error deleting from persistent cache', error);
      }
    }
  }

  async has(key: string): Promise<boolean> {
    if (this.memoryCache.has(key)) {
      return true;
    }

    if (this.config.enablePersistence) {
      try {
        const entry = await this.cache.get(key);
        return entry !== undefined && !this.isExpired(entry);
      } catch (error) {
        return false;
      }
    }

    return false;
  }

  async clear(): Promise<void> {
    this.memoryCache.clear();

    if (this.config.enablePersistence) {
      try {
        await this.cache.clear();
      } catch (error) {
        console.warn('Error clearing persistent cache', error);
      }
    }

    this.stats = {
      hits: 0,
      misses: 0,
      evictions: 0,
    };
  }

  // ============================================================================
  // Tag-based Operations
  // ============================================================================

  async invalidateByTag(tag: string): Promise<void> {
    // Invalidate from memory cache
    for (const [key, entry] of this.memoryCache.entries()) {
      if (entry.tags?.includes(tag)) {
        this.memoryCache.delete(key);
      }
    }

    // Invalidate from persistent cache
    if (this.config.enablePersistence) {
      try {
        const entries = await this.cache.where('tags').equals(tag).toArray();
        await Promise.all(entries.map((entry) => this.cache.delete(entry.key)));
      } catch (error) {
        console.warn('Error invalidating by tag', error);
      }
    }
  }

  async getByTag(tag: string): Promise<CacheEntry[]> {
    const results: CacheEntry[] = [];

    // Get from memory cache
    for (const entry of this.memoryCache.values()) {
      if (entry.tags?.includes(tag) && !this.isExpired(entry)) {
        results.push(entry);
      }
    }

    // Get from persistent cache
    if (this.config.enablePersistence) {
      try {
        const entries = await this.cache.where('tags').equals(tag).toArray();
        for (const entry of entries) {
          if (!this.isExpired(entry)) {
            results.push(entry);
          }
        }
      } catch (error) {
        console.warn('Error getting by tag', error);
      }
    }

    return results;
  }

  // ============================================================================
  // Cache Maintenance
  // ============================================================================

  async evictExpired(): Promise<number> {
    let evicted = 0;

    // Evict from memory cache
    for (const [key, entry] of this.memoryCache.entries()) {
      if (this.isExpired(entry)) {
        this.memoryCache.delete(key);
        evicted++;
      }
    }

    // Evict from persistent cache
    if (this.config.enablePersistence) {
      try {
        const now = Date.now();
        const expired = await this.cache
          .filter((entry) => entry.timestamp + entry.ttl < now)
          .toArray();

        await Promise.all(expired.map((entry) => this.cache.delete(entry.key)));
        evicted += expired.length;
      } catch (error) {
        console.warn('Error evicting expired entries', error);
      }
    }

    this.stats.evictions += evicted;
    return evicted;
  }

  private async evictIfNeeded(newEntrySize: number): Promise<void> {
    const currentSize = this.getCurrentSize();
    const currentCount = this.memoryCache.size;

    // Check if we need to evict based on size
    if (currentSize + newEntrySize > this.config.maxSize) {
      await this.evictByStrategy(newEntrySize);
    }

    // Check if we need to evict based on count
    if (currentCount >= this.config.maxEntries) {
      await this.evictByStrategy(0);
    }
  }

  private async evictByStrategy(spaceNeeded: number): Promise<void> {
    const entries = Array.from(this.memoryCache.entries());

    let toEvict: string[] = [];

    switch (this.config.strategy) {
      case 'lru':
        // Evict least recently used
        entries.sort(([, a], [, b]) => a.timestamp - b.timestamp);
        toEvict = entries.slice(0, Math.max(1, entries.length * 0.1)).map(([key]) => key);
        break;

      case 'lfu':
        // Evict least frequently used
        entries.sort(([, a], [, b]) => a.hits - b.hits);
        toEvict = entries.slice(0, Math.max(1, entries.length * 0.1)).map(([key]) => key);
        break;

      case 'fifo':
        // Evict oldest entries
        entries.sort(([, a], [, b]) => a.timestamp - b.timestamp);
        toEvict = entries.slice(0, Math.max(1, entries.length * 0.1)).map(([key]) => key);
        break;
    }

    // Evict selected entries
    for (const key of toEvict) {
      await this.delete(key);
      this.stats.evictions++;
    }
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  async getStats(): Promise<CacheStats> {
    const totalEntries = this.memoryCache.size;
    const totalSize = this.getCurrentSize();
    const totalRequests = this.stats.hits + this.stats.misses;

    return {
      totalEntries,
      totalSize,
      hitRate: totalRequests > 0 ? this.stats.hits / totalRequests : 0,
      missRate: totalRequests > 0 ? this.stats.misses / totalRequests : 0,
      evictions: this.stats.evictions,
    };
  }

  resetStats(): void {
    this.stats = {
      hits: 0,
      misses: 0,
      evictions: 0,
    };
  }

  // ============================================================================
  // Helper Methods
  // ============================================================================

  private isExpired(entry: CacheEntry): boolean {
    return Date.now() > entry.timestamp + entry.ttl;
  }

  private getCurrentSize(): number {
    let size = 0;
    for (const entry of this.memoryCache.values()) {
      size += entry.size;
    }
    return size;
  }

  private estimateSize(value: unknown): number {
    try {
      // Rough estimation of object size
      const str = JSON.stringify(value);
      return str.length * 2; // Approximate bytes (UTF-16)
    } catch {
      return 1024; // Default 1KB if can't estimate
    }
  }

  // ============================================================================
  // Batch Operations
  // ============================================================================

  async getMany<T = unknown>(keys: string[]): Promise<Map<string, T>> {
    const results = new Map<string, T>();

    await Promise.all(
      keys.map(async (key) => {
        const value = await this.get<T>(key);
        if (value !== null) {
          results.set(key, value);
        }
      })
    );

    return results;
  }

  async setMany<T = unknown>(
    entries: Array<{ key: string; value: T; ttl?: number; tags?: string[] }>
  ): Promise<void> {
    await Promise.all(
      entries.map((entry) => this.set(entry.key, entry.value, entry.ttl, entry.tags))
    );
  }

  async deleteMany(keys: string[]): Promise<void> {
    await Promise.all(keys.map((key) => this.delete(key)));
  }
}

// Factory function
export function createCacheManager(config?: CacheConfig): CacheManager {
  return new CacheManager(config);
}
