/**
 * Usage Tracker - Tracks and aggregates metered usage
 */

import { v4 as uuidv4 } from 'uuid';
import { startOfDay, endOfDay, isWithinInterval } from 'date-fns';
import {
  UsageRecord,
  UsageAggregation,
  MeteredComponent,
  UsageSummary,
  ComponentUsage,
  Subscription,
} from '../types';
import { PricingEngine } from './PricingEngine';
import { Decimal } from 'decimal.js';

export class UsageTracker {
  private usageRecords: Map<string, UsageRecord> = new Map();
  private pricingEngine: PricingEngine;
  private idempotencyKeys: Set<string> = new Set();

  constructor(pricingEngine?: PricingEngine) {
    this.pricingEngine = pricingEngine ?? new PricingEngine();
  }

  /**
   * Record usage with idempotency support
   */
  async recordUsage(
    tenantId: string,
    subscriptionId: string,
    meteredComponentId: string,
    quantity: number,
    idempotencyKey?: string,
    metadata?: Record<string, any>
  ): Promise<UsageRecord> {
    // Check idempotency
    if (idempotencyKey) {
      if (this.idempotencyKeys.has(idempotencyKey)) {
        // Find and return existing record
        const existing = Array.from(this.usageRecords.values()).find(
          (r) => r.idempotencyKey === idempotencyKey
        );
        if (existing) {
          return existing;
        }
      }
      this.idempotencyKeys.add(idempotencyKey);
    }

    const record: UsageRecord = {
      id: uuidv4(),
      tenantId,
      subscriptionId,
      meteredComponentId,
      quantity,
      timestamp: new Date(),
      idempotencyKey,
      metadata: metadata ?? {},
    };

    this.usageRecords.set(record.id, record);
    return record;
  }

  /**
   * Batch record multiple usage events
   */
  async recordBatchUsage(
    records: Omit<UsageRecord, 'id' | 'timestamp'>[]
  ): Promise<UsageRecord[]> {
    const results: UsageRecord[] = [];

    for (const record of records) {
      const created = await this.recordUsage(
        record.tenantId,
        record.subscriptionId,
        record.meteredComponentId,
        record.quantity,
        record.idempotencyKey,
        record.metadata
      );
      results.push(created);
    }

    return results;
  }

  /**
   * Get usage records for a subscription in a period
   */
  getUsageRecords(
    subscriptionId: string,
    periodStart: Date,
    periodEnd: Date
  ): UsageRecord[] {
    return Array.from(this.usageRecords.values()).filter(
      (record) =>
        record.subscriptionId === subscriptionId &&
        isWithinInterval(record.timestamp, { start: periodStart, end: periodEnd })
    );
  }

  /**
   * Get usage records for a tenant
   */
  getTenantUsageRecords(
    tenantId: string,
    periodStart?: Date,
    periodEnd?: Date
  ): UsageRecord[] {
    return Array.from(this.usageRecords.values()).filter((record) => {
      if (record.tenantId !== tenantId) return false;
      if (periodStart && periodEnd) {
        return isWithinInterval(record.timestamp, { start: periodStart, end: periodEnd });
      }
      return true;
    });
  }

  /**
   * Aggregate usage for a component
   */
  aggregateUsage(
    records: UsageRecord[],
    componentId: string,
    aggregation: UsageAggregation
  ): number {
    const componentRecords = records.filter((r) => r.meteredComponentId === componentId);

    switch (aggregation) {
      case UsageAggregation.SUM:
        return componentRecords.reduce((sum, r) => sum + r.quantity, 0);

      case UsageAggregation.MAX:
        return componentRecords.reduce(
          (max, r) => Math.max(max, r.quantity),
          0
        );

      case UsageAggregation.LAST_DURING_PERIOD:
        if (componentRecords.length === 0) return 0;
        const sorted = componentRecords.sort(
          (a, b) => b.timestamp.getTime() - a.timestamp.getTime()
        );
        return sorted[0].quantity;

      default:
        throw new Error(`Unknown aggregation type: ${aggregation}`);
    }
  }

  /**
   * Get usage summary for a subscription
   */
  async getUsageSummary(
    subscription: Subscription,
    components: MeteredComponent[],
    periodStart: Date,
    periodEnd: Date
  ): Promise<UsageSummary> {
    const records = this.getUsageRecords(subscription.id, periodStart, periodEnd);

    const componentUsages: ComponentUsage[] = components.map((component) => {
      const totalUsage = this.aggregateUsage(records, component.id, component.aggregation);

      return this.pricingEngine.calculateComponentUsage(component, records);
    });

    const estimatedCost = componentUsages.reduce(
      (sum, cu) => sum.plus(cu.cost),
      new Decimal(0)
    );

    return {
      tenantId: subscription.tenantId,
      subscriptionId: subscription.id,
      periodStart,
      periodEnd,
      components: componentUsages,
      estimatedCost,
    };
  }

  /**
   * Get daily usage breakdown
   */
  getDailyUsage(
    subscriptionId: string,
    componentId: string,
    periodStart: Date,
    periodEnd: Date
  ): Array<{ date: Date; usage: number }> {
    const records = this.getUsageRecords(subscriptionId, periodStart, periodEnd);
    const componentRecords = records.filter((r) => r.meteredComponentId === componentId);

    const dailyUsage = new Map<string, number>();

    for (const record of componentRecords) {
      const dateKey = startOfDay(record.timestamp).toISOString();
      const current = dailyUsage.get(dateKey) ?? 0;
      dailyUsage.set(dateKey, current + record.quantity);
    }

    return Array.from(dailyUsage.entries())
      .map(([dateStr, usage]) => ({
        date: new Date(dateStr),
        usage,
      }))
      .sort((a, b) => a.date.getTime() - b.date.getTime());
  }

  /**
   * Get top usage by tenant
   */
  getTopUsageTenants(
    componentId: string,
    periodStart: Date,
    periodEnd: Date,
    limit: number = 10
  ): Array<{ tenantId: string; totalUsage: number }> {
    const allRecords = Array.from(this.usageRecords.values()).filter(
      (record) =>
        record.meteredComponentId === componentId &&
        isWithinInterval(record.timestamp, { start: periodStart, end: periodEnd })
    );

    const tenantUsage = new Map<string, number>();

    for (const record of allRecords) {
      const current = tenantUsage.get(record.tenantId) ?? 0;
      tenantUsage.set(record.tenantId, current + record.quantity);
    }

    return Array.from(tenantUsage.entries())
      .map(([tenantId, totalUsage]) => ({ tenantId, totalUsage }))
      .sort((a, b) => b.totalUsage - a.totalUsage)
      .slice(0, limit);
  }

  /**
   * Delete usage records (for GDPR compliance)
   */
  async deleteUsageRecords(tenantId: string): Promise<number> {
    let deleted = 0;

    for (const [id, record] of this.usageRecords) {
      if (record.tenantId === tenantId) {
        this.usageRecords.delete(id);
        deleted++;
      }
    }

    return deleted;
  }

  /**
   * Get usage statistics
   */
  getUsageStatistics(
    subscriptionId: string,
    componentId: string,
    periodStart: Date,
    periodEnd: Date
  ): {
    total: number;
    average: number;
    min: number;
    max: number;
    count: number;
  } {
    const records = this.getUsageRecords(subscriptionId, periodStart, periodEnd).filter(
      (r) => r.meteredComponentId === componentId
    );

    if (records.length === 0) {
      return { total: 0, average: 0, min: 0, max: 0, count: 0 };
    }

    const quantities = records.map((r) => r.quantity);
    const total = quantities.reduce((sum, q) => sum + q, 0);
    const average = total / quantities.length;
    const min = Math.min(...quantities);
    const max = Math.max(...quantities);

    return {
      total,
      average,
      min,
      max,
      count: records.length,
    };
  }

  /**
   * Check if usage exceeds quota
   */
  isQuotaExceeded(
    currentUsage: number,
    quota: number,
    unlimited: boolean
  ): boolean {
    if (unlimited) return false;
    return currentUsage >= quota;
  }

  /**
   * Calculate usage percentage
   */
  calculateUsagePercentage(currentUsage: number, quota: number): number {
    if (quota === 0) return 0;
    return (currentUsage / quota) * 100;
  }

  /**
   * Archive old usage records (for performance)
   */
  async archiveOldRecords(cutoffDate: Date): Promise<number> {
    let archived = 0;

    for (const [id, record] of this.usageRecords) {
      if (record.timestamp < cutoffDate) {
        // In production, this would move to archive storage
        this.usageRecords.delete(id);
        archived++;
      }
    }

    return archived;
  }
}
