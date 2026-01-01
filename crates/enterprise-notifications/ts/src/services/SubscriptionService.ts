/**
 * SubscriptionService - Manages user subscriptions to entities/events
 */

import { EventEmitter } from 'events';
import { Subscription } from '../types';

export class SubscriptionService extends EventEmitter {
  private subscriptions: Map<string, Subscription>;
  private userSubscriptions: Map<string, Set<string>>;
  private entitySubscriptions: Map<string, Set<string>>;

  constructor() {
    super();
    this.subscriptions = new Map();
    this.userSubscriptions = new Map();
    this.entitySubscriptions = new Map();
  }

  /**
   * Create subscription
   */
  async create(subscription: Omit<Subscription, 'id' | 'createdAt' | 'updatedAt'>): Promise<Subscription> {
    const fullSubscription: Subscription = {
      ...subscription,
      id: this.generateId(),
      createdAt: new Date(),
      updatedAt: new Date(),
    };

    this.subscriptions.set(fullSubscription.id, fullSubscription);

    // Index by user
    let userSubs = this.userSubscriptions.get(subscription.userId);
    if (!userSubs) {
      userSubs = new Set();
      this.userSubscriptions.set(subscription.userId, userSubs);
    }
    userSubs.add(fullSubscription.id);

    // Index by entity
    const entityKey = `${subscription.entityType}:${subscription.entityId}`;
    let entitySubs = this.entitySubscriptions.get(entityKey);
    if (!entitySubs) {
      entitySubs = new Set();
      this.entitySubscriptions.set(entityKey, entitySubs);
    }
    entitySubs.add(fullSubscription.id);

    this.emit('subscription:created', fullSubscription);
    return fullSubscription;
  }

  /**
   * Get user subscriptions
   */
  async getUserSubscriptions(userId: string): Promise<Subscription[]> {
    const subscriptionIds = this.userSubscriptions.get(userId) ?? new Set();
    return Array.from(subscriptionIds)
      .map(id => this.subscriptions.get(id))
      .filter((s): s is Subscription => s !== undefined && s.enabled);
  }

  /**
   * Get entity subscribers
   */
  async getEntitySubscribers(entityType: string, entityId: string): Promise<Subscription[]> {
    const entityKey = `${entityType}:${entityId}`;
    const subscriptionIds = this.entitySubscriptions.get(entityKey) ?? new Set();
    return Array.from(subscriptionIds)
      .map(id => this.subscriptions.get(id))
      .filter((s): s is Subscription => s !== undefined && s.enabled);
  }

  /**
   * Update subscription
   */
  async update(subscriptionId: string, updates: Partial<Subscription>): Promise<Subscription | undefined> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      return undefined;
    }

    Object.assign(subscription, updates, { updatedAt: new Date() });
    this.emit('subscription:updated', subscription);
    return subscription;
  }

  /**
   * Delete subscription
   */
  async delete(subscriptionId: string): Promise<boolean> {
    const subscription = this.subscriptions.get(subscriptionId);
    if (!subscription) {
      return false;
    }

    this.subscriptions.delete(subscriptionId);
    this.userSubscriptions.get(subscription.userId)?.delete(subscriptionId);

    const entityKey = `${subscription.entityType}:${subscription.entityId}`;
    this.entitySubscriptions.get(entityKey)?.delete(subscriptionId);

    this.emit('subscription:deleted', subscriptionId);
    return true;
  }

  /**
   * Generate subscription ID
   */
  private generateId(): string {
    return `sub_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

export default SubscriptionService;
