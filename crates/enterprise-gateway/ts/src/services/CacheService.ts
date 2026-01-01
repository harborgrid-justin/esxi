/**
 * Enterprise API Gateway - Cache Service
 *
 * Response caching with multiple strategies
 */

import type { GatewayRequest, GatewayResponse, CacheConfig, CacheEntry } from '../types';
import { createHash } from 'crypto';

export class CacheService {
  private cache: Map<string, CacheEntry> = new Map();
  private lruQueue: string[] = [];
  private accessCounts: Map<string, number> = new Map();

  constructor(private config: CacheConfig) {}

  /**
   * Get cached response
   */
  public get(request: GatewayRequest): GatewayResponse | null {
    if (!this.config.enabled) {
      return null;
    }

    // Only cache configured methods
    if (!this.config.cacheMethods.includes(request.method)) {
      return null;
    }

    const key = this.generateKey(request);
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    // Check if expired
    if (Date.now() > entry.expiresAt) {
      this.cache.delete(key);
      return null;
    }

    // Update access statistics
    entry.hits++;
    this.recordAccess(key);

    return entry.value;
  }

  /**
   * Set cached response
   */
  public set(request: GatewayRequest, response: GatewayResponse): void {
    if (!this.config.enabled) {
      return;
    }

    // Only cache configured methods
    if (!this.config.cacheMethods.includes(request.method)) {
      return;
    }

    // Only cache configured status codes
    if (!this.config.cacheableStatusCodes.includes(response.statusCode)) {
      return;
    }

    const key = this.generateKey(request);
    const size = this.calculateSize(response);

    // Check if we need to evict
    while (this.shouldEvict(size)) {
      this.evict();
    }

    const entry: CacheEntry = {
      key,
      value: response,
      size,
      createdAt: Date.now(),
      expiresAt: Date.now() + this.config.ttl,
      hits: 0,
    };

    this.cache.set(key, entry);
    this.recordAccess(key);
  }

  /**
   * Generate cache key
   */
  private generateKey(request: GatewayRequest): string {
    const parts = [request.method, request.path];

    // Include query parameters
    const query = new URLSearchParams();
    for (const [key, value] of Object.entries(request.query)) {
      const valueStr = Array.isArray(value) ? value.join(',') : value;
      query.append(key, valueStr);
    }
    parts.push(query.toString());

    // Include vary headers
    if (this.config.varyHeaders) {
      for (const header of this.config.varyHeaders) {
        const value = request.headers[header.toLowerCase()];
        if (value) {
          parts.push(`${header}:${value}`);
        }
      }
    }

    const keyString = parts.join('|');
    return createHash('md5').update(keyString).digest('hex');
  }

  /**
   * Calculate response size
   */
  private calculateSize(response: GatewayResponse): number {
    const bodySize = response.body
      ? new Blob([JSON.stringify(response.body)]).size
      : 0;

    const headersSize = new Blob([JSON.stringify(response.headers)]).size;

    return bodySize + headersSize;
  }

  /**
   * Check if we should evict entries
   */
  private shouldEvict(newSize: number): boolean {
    const currentSize = this.getTotalSize();
    return currentSize + newSize > this.config.maxSize;
  }

  /**
   * Get total cache size
   */
  private getTotalSize(): number {
    let total = 0;
    for (const entry of this.cache.values()) {
      total += entry.size;
    }
    return total;
  }

  /**
   * Evict entries based on strategy
   */
  private evict(): void {
    if (this.cache.size === 0) {
      return;
    }

    let keyToEvict: string | null = null;

    switch (this.config.strategy) {
      case 'lru':
        keyToEvict = this.evictLRU();
        break;

      case 'lfu':
        keyToEvict = this.evictLFU();
        break;

      case 'time-based':
        keyToEvict = this.evictOldest();
        break;
    }

    if (keyToEvict) {
      this.cache.delete(keyToEvict);
      this.accessCounts.delete(keyToEvict);

      const index = this.lruQueue.indexOf(keyToEvict);
      if (index >= 0) {
        this.lruQueue.splice(index, 1);
      }
    }
  }

  /**
   * Evict least recently used
   */
  private evictLRU(): string | null {
    return this.lruQueue.length > 0 ? this.lruQueue[0]! : null;
  }

  /**
   * Evict least frequently used
   */
  private evictLFU(): string | null {
    let minAccess = Infinity;
    let keyToEvict: string | null = null;

    for (const [key, count] of this.accessCounts) {
      if (count < minAccess) {
        minAccess = count;
        keyToEvict = key;
      }
    }

    return keyToEvict;
  }

  /**
   * Evict oldest entry
   */
  private evictOldest(): string | null {
    let oldest = Infinity;
    let keyToEvict: string | null = null;

    for (const [key, entry] of this.cache) {
      if (entry.createdAt < oldest) {
        oldest = entry.createdAt;
        keyToEvict = key;
      }
    }

    return keyToEvict;
  }

  /**
   * Record access for LRU/LFU tracking
   */
  private recordAccess(key: string): void {
    // Update LRU queue
    const index = this.lruQueue.indexOf(key);
    if (index >= 0) {
      this.lruQueue.splice(index, 1);
    }
    this.lruQueue.push(key);

    // Update access count for LFU
    const count = this.accessCounts.get(key) || 0;
    this.accessCounts.set(key, count + 1);
  }

  /**
   * Invalidate cache entry
   */
  public invalidate(request: GatewayRequest): void {
    const key = this.generateKey(request);
    this.cache.delete(key);
    this.accessCounts.delete(key);

    const index = this.lruQueue.indexOf(key);
    if (index >= 0) {
      this.lruQueue.splice(index, 1);
    }
  }

  /**
   * Clear all cache
   */
  public clear(): void {
    this.cache.clear();
    this.lruQueue = [];
    this.accessCounts.clear();
  }

  /**
   * Clear expired entries
   */
  public clearExpired(): number {
    const now = Date.now();
    let cleared = 0;

    for (const [key, entry] of this.cache) {
      if (now > entry.expiresAt) {
        this.cache.delete(key);
        this.accessCounts.delete(key);

        const index = this.lruQueue.indexOf(key);
        if (index >= 0) {
          this.lruQueue.splice(index, 1);
        }

        cleared++;
      }
    }

    return cleared;
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    entries: number;
    totalSize: number;
    maxSize: number;
    hitRate: number;
  } {
    let totalHits = 0;
    let totalRequests = 0;

    for (const entry of this.cache.values()) {
      totalHits += entry.hits;
      totalRequests += entry.hits + 1; // +1 for the initial set
    }

    return {
      entries: this.cache.size,
      totalSize: this.getTotalSize(),
      maxSize: this.config.maxSize,
      hitRate: totalRequests > 0 ? totalHits / totalRequests : 0,
    };
  }

  /**
   * Get all cache entries
   */
  public getEntries(): CacheEntry[] {
    return Array.from(this.cache.values());
  }
}
