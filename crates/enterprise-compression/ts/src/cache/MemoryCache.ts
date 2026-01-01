/**
 * Memory Cache
 * Fast in-memory LRU cache
 */

import LRUCache from 'lru-cache';
import { CacheEntry, CacheTier } from '../types';

export class MemoryCache<T = any> {
  private cache: LRUCache<string, CacheEntry<T>>;

  constructor(options: {
    maxSize: number;
    maxEntries?: number;
    ttl?: number;
  }) {
    this.cache = new LRUCache({
      max: options.maxEntries || 10000,
      maxSize: options.maxSize,
      sizeCalculation: (entry) => entry.size,
      ttl: options.ttl,
      ttlAutopurge: true,
    });
  }

  /**
   * Set cache entry
   */
  set(key: string, value: T, ttl?: number): void {
    const entry: CacheEntry<T> = {
      key,
      value,
      size: this.calculateSize(value),
      tier: CacheTier.MEMORY,
      compressed: false,
      createdAt: new Date(),
      accessedAt: new Date(),
      expiresAt: ttl ? new Date(Date.now() + ttl) : undefined,
      hits: 0,
    };

    this.cache.set(key, entry, { ttl });
  }

  /**
   * Get cache entry
   */
  get(key: string): T | undefined {
    const entry = this.cache.get(key);
    if (entry) {
      entry.accessedAt = new Date();
      entry.hits++;
      return entry.value;
    }
    return undefined;
  }

  /**
   * Check if key exists
   */
  has(key: string): boolean {
    return this.cache.has(key);
  }

  /**
   * Delete entry
   */
  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  /**
   * Clear all entries
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size;
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
   * Get all keys
   */
  keys(): string[] {
    return Array.from(this.cache.keys());
  }

  /**
   * Get all values
   */
  values(): T[] {
    return Array.from(this.cache.values()).map(e => e.value);
  }
}
