/**
 * Disk Cache
 * Persistent filesystem-based cache
 */

import { promises as fs } from 'fs';
import { join } from 'path';
import { createHash } from 'crypto';
import { CacheEntry, CacheTier, CacheError } from '../types';

export class DiskCache<T = any> {
  private indexPath: string;
  private index = new Map<string, CacheEntry<T>>();

  constructor(private cacheDir: string) {
    this.indexPath = join(cacheDir, 'index.json');
  }

  /**
   * Initialize cache
   */
  async initialize(): Promise<void> {
    try {
      await fs.mkdir(this.cacheDir, { recursive: true });
      await this.loadIndex();
    } catch (error) {
      throw new CacheError(
        `Failed to initialize disk cache: ${error instanceof Error ? error.message : 'Unknown'}`,
        CacheTier.DISK,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Set cache entry
   */
  async set(key: string, value: T, ttl?: number): Promise<void> {
    try {
      const filename = this.getFilename(key);
      const filepath = join(this.cacheDir, filename);
      const data = JSON.stringify(value);

      await fs.writeFile(filepath, data, 'utf8');

      const entry: CacheEntry<T> = {
        key,
        value,
        size: Buffer.byteLength(data),
        tier: CacheTier.DISK,
        compressed: false,
        createdAt: new Date(),
        accessedAt: new Date(),
        expiresAt: ttl ? new Date(Date.now() + ttl) : undefined,
        hits: 0,
      };

      this.index.set(key, entry);
      await this.saveIndex();
    } catch (error) {
      throw new CacheError(
        `Failed to set disk cache entry: ${error instanceof Error ? error.message : 'Unknown'}`,
        CacheTier.DISK,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Get cache entry
   */
  async get(key: string): Promise<T | undefined> {
    try {
      const entry = this.index.get(key);
      if (!entry) return undefined;

      if (entry.expiresAt && entry.expiresAt < new Date()) {
        await this.delete(key);
        return undefined;
      }

      const filename = this.getFilename(key);
      const filepath = join(this.cacheDir, filename);
      const data = await fs.readFile(filepath, 'utf8');

      entry.accessedAt = new Date();
      entry.hits++;
      await this.saveIndex();

      return JSON.parse(data);
    } catch (error) {
      return undefined;
    }
  }

  /**
   * Delete cache entry
   */
  async delete(key: string): Promise<boolean> {
    try {
      const filename = this.getFilename(key);
      const filepath = join(this.cacheDir, filename);

      await fs.unlink(filepath);
      this.index.delete(key);
      await this.saveIndex();

      return true;
    } catch (error) {
      return false;
    }
  }

  /**
   * Clear all cache
   */
  async clear(): Promise<void> {
    try {
      const files = await fs.readdir(this.cacheDir);
      await Promise.all(
        files
          .filter(f => f !== 'index.json')
          .map(f => fs.unlink(join(this.cacheDir, f)))
      );

      this.index.clear();
      await this.saveIndex();
    } catch (error) {
      throw new CacheError(
        `Failed to clear disk cache: ${error instanceof Error ? error.message : 'Unknown'}`,
        CacheTier.DISK,
        error instanceof Error ? error : undefined
      );
    }
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.index.size;
  }

  /**
   * Get filename for key
   */
  private getFilename(key: string): string {
    return createHash('md5').update(key).digest('hex') + '.cache';
  }

  /**
   * Load index from disk
   */
  private async loadIndex(): Promise<void> {
    try {
      const data = await fs.readFile(this.indexPath, 'utf8');
      const entries = JSON.parse(data);

      this.index.clear();
      for (const entry of entries) {
        entry.createdAt = new Date(entry.createdAt);
        entry.accessedAt = new Date(entry.accessedAt);
        if (entry.expiresAt) {
          entry.expiresAt = new Date(entry.expiresAt);
        }
        this.index.set(entry.key, entry);
      }
    } catch (error) {
      // Index doesn't exist yet, that's okay
      this.index.clear();
    }
  }

  /**
   * Save index to disk
   */
  private async saveIndex(): Promise<void> {
    const entries = Array.from(this.index.values());
    await fs.writeFile(this.indexPath, JSON.stringify(entries, null, 2), 'utf8');
  }

  /**
   * Prune expired entries
   */
  async prune(): Promise<number> {
    let pruned = 0;
    const now = new Date();

    for (const [key, entry] of this.index.entries()) {
      if (entry.expiresAt && entry.expiresAt < now) {
        await this.delete(key);
        pruned++;
      }
    }

    return pruned;
  }
}
