/**
 * PriorityQueue - Priority-based queue for notifications
 * Ensures high-priority notifications are processed first
 */

import { NotificationPriority, QueuedNotification } from '../types';

export interface PriorityQueueConfig {
  maxSize?: number;
  enableScheduling: boolean;
  enableRetries: boolean;
  maxRetries: number;
}

export class PriorityQueue<T extends QueuedNotification = QueuedNotification> {
  private config: PriorityQueueConfig;
  private queues: Map<NotificationPriority, T[]>;
  private scheduled: Map<string, NodeJS.Timeout>;
  private priorityOrder: NotificationPriority[];

  constructor(config: Partial<PriorityQueueConfig> = {}) {
    this.config = {
      maxSize: config.maxSize,
      enableScheduling: config.enableScheduling ?? true,
      enableRetries: config.enableRetries ?? true,
      maxRetries: config.maxRetries ?? 3,
    };

    this.priorityOrder = [
      NotificationPriority.CRITICAL,
      NotificationPriority.URGENT,
      NotificationPriority.HIGH,
      NotificationPriority.NORMAL,
      NotificationPriority.LOW,
    ];

    this.queues = new Map(this.priorityOrder.map(p => [p, []]));
    this.scheduled = new Map();
  }

  /**
   * Enqueue an item
   */
  enqueue(item: T): boolean {
    const queue = this.queues.get(item.priority);
    if (!queue) {
      return false;
    }

    // Check max size
    if (this.config.maxSize && this.size() >= this.config.maxSize) {
      return false;
    }

    // Handle scheduled items
    if (this.config.enableScheduling && item.scheduledFor > new Date()) {
      this.scheduleItem(item);
      return true;
    }

    queue.push(item);
    return true;
  }

  /**
   * Dequeue highest priority item
   */
  dequeue(): T | undefined {
    for (const priority of this.priorityOrder) {
      const queue = this.queues.get(priority);
      if (queue && queue.length > 0) {
        return queue.shift();
      }
    }
    return undefined;
  }

  /**
   * Peek at highest priority item without removing
   */
  peek(): T | undefined {
    for (const priority of this.priorityOrder) {
      const queue = this.queues.get(priority);
      if (queue && queue.length > 0) {
        return queue[0];
      }
    }
    return undefined;
  }

  /**
   * Dequeue all items of a specific priority
   */
  dequeueByPriority(priority: NotificationPriority, limit?: number): T[] {
    const queue = this.queues.get(priority);
    if (!queue || queue.length === 0) {
      return [];
    }

    if (limit && limit < queue.length) {
      return queue.splice(0, limit);
    }

    const items = [...queue];
    queue.length = 0;
    return items;
  }

  /**
   * Dequeue batch of highest priority items
   */
  dequeueBatch(maxItems: number): T[] {
    const items: T[] = [];

    for (const priority of this.priorityOrder) {
      if (items.length >= maxItems) {
        break;
      }

      const queue = this.queues.get(priority);
      if (queue && queue.length > 0) {
        const count = Math.min(maxItems - items.length, queue.length);
        items.push(...queue.splice(0, count));
      }
    }

    return items;
  }

  /**
   * Remove specific item by ID
   */
  remove(itemId: string): T | undefined {
    for (const queue of this.queues.values()) {
      const index = queue.findIndex(item => item.id === itemId);
      if (index !== -1) {
        return queue.splice(index, 1)[0];
      }
    }

    // Check scheduled items
    const timeout = this.scheduled.get(itemId);
    if (timeout) {
      clearTimeout(timeout);
      this.scheduled.delete(itemId);
    }

    return undefined;
  }

  /**
   * Find item by ID
   */
  find(itemId: string): T | undefined {
    for (const queue of this.queues.values()) {
      const item = queue.find(item => item.id === itemId);
      if (item) {
        return item;
      }
    }
    return undefined;
  }

  /**
   * Update item priority
   */
  updatePriority(itemId: string, newPriority: NotificationPriority): boolean {
    const item = this.remove(itemId);
    if (!item) {
      return false;
    }

    item.priority = newPriority;
    return this.enqueue(item);
  }

  /**
   * Retry failed item
   */
  retry(item: T): boolean {
    if (!this.config.enableRetries) {
      return false;
    }

    if (item.attempts >= this.config.maxRetries) {
      return false;
    }

    item.attempts++;

    // Calculate exponential backoff
    const backoffMs = Math.min(30000, Math.pow(2, item.attempts) * 1000);
    item.nextRetryAt = new Date(Date.now() + backoffMs);

    if (item.nextRetryAt > new Date()) {
      this.scheduleItem(item);
    } else {
      this.enqueue(item);
    }

    return true;
  }

  /**
   * Schedule item for future processing
   */
  private scheduleItem(item: T): void {
    const delay = item.scheduledFor.getTime() - Date.now();
    if (delay <= 0) {
      this.enqueue(item);
      return;
    }

    const timeout = setTimeout(() => {
      this.enqueue(item);
      this.scheduled.delete(item.id);
    }, delay);

    this.scheduled.set(item.id, timeout);
  }

  /**
   * Clear all queues
   */
  clear(): void {
    for (const queue of this.queues.values()) {
      queue.length = 0;
    }

    for (const timeout of this.scheduled.values()) {
      clearTimeout(timeout);
    }
    this.scheduled.clear();
  }

  /**
   * Clear specific priority queue
   */
  clearPriority(priority: NotificationPriority): void {
    const queue = this.queues.get(priority);
    if (queue) {
      queue.length = 0;
    }
  }

  /**
   * Get total size
   */
  size(): number {
    let total = 0;
    for (const queue of this.queues.values()) {
      total += queue.length;
    }
    return total + this.scheduled.size;
  }

  /**
   * Get size by priority
   */
  sizeByPriority(priority: NotificationPriority): number {
    return this.queues.get(priority)?.length ?? 0;
  }

  /**
   * Check if empty
   */
  isEmpty(): boolean {
    return this.size() === 0;
  }

  /**
   * Get all items (for inspection, doesn't remove)
   */
  toArray(): T[] {
    const items: T[] = [];
    for (const priority of this.priorityOrder) {
      const queue = this.queues.get(priority);
      if (queue) {
        items.push(...queue);
      }
    }
    return items;
  }

  /**
   * Get statistics
   */
  getStats(): {
    total: number;
    scheduled: number;
    byPriority: Record<NotificationPriority, number>;
  } {
    const byPriority: Record<NotificationPriority, number> = {
      [NotificationPriority.CRITICAL]: 0,
      [NotificationPriority.URGENT]: 0,
      [NotificationPriority.HIGH]: 0,
      [NotificationPriority.NORMAL]: 0,
      [NotificationPriority.LOW]: 0,
    };

    for (const [priority, queue] of this.queues.entries()) {
      byPriority[priority] = queue.length;
    }

    return {
      total: this.size(),
      scheduled: this.scheduled.size,
      byPriority,
    };
  }

  /**
   * Destroy queue and cleanup
   */
  destroy(): void {
    this.clear();
  }
}

export default PriorityQueue;
