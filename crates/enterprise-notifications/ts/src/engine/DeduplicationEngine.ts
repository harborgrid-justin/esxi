/**
 * DeduplicationEngine - Prevents duplicate notifications
 * Uses various strategies to identify and suppress duplicates
 */

import crypto from 'crypto';
import { Notification } from '../types';

export interface DeduplicationEngineConfig {
  strategy: 'fingerprint' | 'key' | 'time-window' | 'content-hash';
  windowMs: number;
  maxEntries: number;
  cleanupInterval: number;
  enableGrouping: boolean;
  groupWindowMs: number;
}

export interface DeduplicationEntry {
  id: string;
  fingerprint: string;
  firstSeenAt: Date;
  lastSeenAt: Date;
  count: number;
  notificationIds: string[];
  metadata?: Record<string, unknown>;
}

export class DeduplicationEngine {
  private config: DeduplicationEngineConfig;
  private cache: Map<string, DeduplicationEntry>;
  private groups: Map<string, string[]>; // groupKey -> fingerprints
  private cleanupInterval?: NodeJS.Timeout;

  constructor(config: Partial<DeduplicationEngineConfig> = {}) {
    this.config = {
      strategy: config.strategy ?? 'fingerprint',
      windowMs: config.windowMs ?? 300000, // 5 minutes
      maxEntries: config.maxEntries ?? 10000,
      cleanupInterval: config.cleanupInterval ?? 60000, // 1 minute
      enableGrouping: config.enableGrouping ?? true,
      groupWindowMs: config.groupWindowMs ?? 600000, // 10 minutes
    };

    this.cache = new Map();
    this.groups = new Map();

    this.startCleanup();
  }

  /**
   * Check if notification is duplicate
   */
  isDuplicate(notification: Notification): boolean {
    const fingerprint = this.generateFingerprint(notification);
    const entry = this.cache.get(fingerprint);

    if (!entry) {
      return false;
    }

    const age = Date.now() - entry.lastSeenAt.getTime();
    return age < this.config.windowMs;
  }

  /**
   * Record notification
   */
  record(notification: Notification): DeduplicationEntry {
    const fingerprint = this.generateFingerprint(notification);
    let entry = this.cache.get(fingerprint);

    if (entry) {
      // Update existing entry
      entry.lastSeenAt = new Date();
      entry.count++;
      entry.notificationIds.push(notification.id);
    } else {
      // Create new entry
      entry = {
        id: this.generateEntryId(),
        fingerprint,
        firstSeenAt: new Date(),
        lastSeenAt: new Date(),
        count: 1,
        notificationIds: [notification.id],
        metadata: {
          tenantId: notification.tenantId,
          type: notification.type,
          priority: notification.priority,
        },
      };

      // Check max entries
      if (this.cache.size >= this.config.maxEntries) {
        this.evictOldest();
      }

      this.cache.set(fingerprint, entry);
    }

    // Handle grouping
    if (this.config.enableGrouping && notification.groupKey) {
      this.addToGroup(notification.groupKey, fingerprint);
    }

    return entry;
  }

  /**
   * Get deduplication entry
   */
  getEntry(fingerprint: string): DeduplicationEntry | undefined {
    return this.cache.get(fingerprint);
  }

  /**
   * Get group members
   */
  getGroup(groupKey: string): string[] {
    return this.groups.get(groupKey) ?? [];
  }

  /**
   * Get grouped notifications count
   */
  getGroupCount(groupKey: string): number {
    const fingerprints = this.groups.get(groupKey) ?? [];
    let total = 0;

    for (const fingerprint of fingerprints) {
      const entry = this.cache.get(fingerprint);
      if (entry) {
        total += entry.count;
      }
    }

    return total;
  }

  /**
   * Clear entry
   */
  clear(fingerprint: string): boolean {
    return this.cache.delete(fingerprint);
  }

  /**
   * Clear all entries
   */
  clearAll(): void {
    this.cache.clear();
    this.groups.clear();
  }

  /**
   * Clear expired entries
   */
  clearExpired(): number {
    const now = Date.now();
    const toDelete: string[] = [];

    for (const [fingerprint, entry] of this.cache.entries()) {
      const age = now - entry.lastSeenAt.getTime();
      if (age > this.config.windowMs) {
        toDelete.push(fingerprint);
      }
    }

    for (const fingerprint of toDelete) {
      this.cache.delete(fingerprint);
    }

    // Clean up groups
    for (const [groupKey, fingerprints] of this.groups.entries()) {
      const validFingerprints = fingerprints.filter(fp => this.cache.has(fp));
      if (validFingerprints.length === 0) {
        this.groups.delete(groupKey);
      } else if (validFingerprints.length !== fingerprints.length) {
        this.groups.set(groupKey, validFingerprints);
      }
    }

    return toDelete.length;
  }

  /**
   * Generate fingerprint based on strategy
   */
  private generateFingerprint(notification: Notification): string {
    switch (this.config.strategy) {
      case 'key':
        return notification.deduplicationKey ?? this.generateContentHash(notification);

      case 'content-hash':
        return this.generateContentHash(notification);

      case 'time-window':
        // Include time bucket in fingerprint
        const bucket = Math.floor(Date.now() / this.config.windowMs);
        return `${bucket}:${this.generateContentHash(notification)}`;

      case 'fingerprint':
      default:
        return this.generateFingerprintHash(notification);
    }
  }

  /**
   * Generate fingerprint hash
   */
  private generateFingerprintHash(notification: Notification): string {
    const parts = [
      notification.tenantId,
      notification.userId ?? '',
      notification.type,
      notification.title,
      notification.message,
      notification.channels.sort().join(','),
    ];

    return this.hash(parts.join(':'));
  }

  /**
   * Generate content hash
   */
  private generateContentHash(notification: Notification): string {
    const content = {
      title: notification.title,
      message: notification.message,
      type: notification.type,
      category: notification.category,
      data: notification.data,
    };

    return this.hash(JSON.stringify(content));
  }

  /**
   * Hash string using SHA-256
   */
  private hash(input: string): string {
    return crypto.createHash('sha256').update(input).digest('hex');
  }

  /**
   * Add fingerprint to group
   */
  private addToGroup(groupKey: string, fingerprint: string): void {
    let group = this.groups.get(groupKey);
    if (!group) {
      group = [];
      this.groups.set(groupKey, group);
    }

    if (!group.includes(fingerprint)) {
      group.push(fingerprint);
    }
  }

  /**
   * Evict oldest entry
   */
  private evictOldest(): void {
    let oldest: { fingerprint: string; timestamp: number } | undefined;

    for (const [fingerprint, entry] of this.cache.entries()) {
      const timestamp = entry.lastSeenAt.getTime();
      if (!oldest || timestamp < oldest.timestamp) {
        oldest = { fingerprint, timestamp };
      }
    }

    if (oldest) {
      this.cache.delete(oldest.fingerprint);
    }
  }

  /**
   * Start cleanup interval
   */
  private startCleanup(): void {
    this.cleanupInterval = setInterval(() => {
      this.clearExpired();
    }, this.config.cleanupInterval);
  }

  /**
   * Get statistics
   */
  getStats(): {
    total: number;
    groups: number;
    duplicates: number;
    averageCount: number;
  } {
    let totalCount = 0;
    let duplicates = 0;

    for (const entry of this.cache.values()) {
      totalCount += entry.count;
      if (entry.count > 1) {
        duplicates++;
      }
    }

    return {
      total: this.cache.size,
      groups: this.groups.size,
      duplicates,
      averageCount: this.cache.size > 0 ? totalCount / this.cache.size : 0,
    };
  }

  /**
   * Generate unique entry ID
   */
  private generateEntryId(): string {
    return `entry_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Cleanup on destroy
   */
  destroy(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }
    this.clearAll();
  }
}

export default DeduplicationEngine;
