/**
 * Quota Service - Feature quota management and enforcement
 */

import { QuotaUsage, Plan, Subscription, PlanFeature } from '../types';
import { startOfMonth, endOfMonth } from 'date-fns';

export interface QuotaCheck {
  allowed: boolean;
  remaining: number;
  limit: number;
  unlimited: boolean;
  resetDate?: Date;
}

export class QuotaService {
  private usageCache: Map<string, Map<string, number>> = new Map();

  /**
   * Check if tenant can use a feature
   */
  async checkQuota(
    tenantId: string,
    featureId: string,
    plan: Plan,
    requestedAmount: number = 1
  ): Promise<QuotaCheck> {
    const feature = plan.features.find((f) => f.id === featureId);

    if (!feature) {
      return {
        allowed: false,
        remaining: 0,
        limit: 0,
        unlimited: false,
      };
    }

    if (!feature.enabled) {
      return {
        allowed: false,
        remaining: 0,
        limit: 0,
        unlimited: false,
      };
    }

    if (feature.unlimited) {
      return {
        allowed: true,
        remaining: Infinity,
        limit: Infinity,
        unlimited: true,
      };
    }

    const currentUsage = this.getCurrentUsage(tenantId, featureId);
    const limit = feature.quota || 0;
    const remaining = Math.max(0, limit - currentUsage);
    const allowed = currentUsage + requestedAmount <= limit;

    return {
      allowed,
      remaining,
      limit,
      unlimited: false,
      resetDate: endOfMonth(new Date()),
    };
  }

  /**
   * Increment feature usage
   */
  async incrementUsage(
    tenantId: string,
    featureId: string,
    amount: number = 1
  ): Promise<number> {
    if (!this.usageCache.has(tenantId)) {
      this.usageCache.set(tenantId, new Map());
    }

    const tenantUsage = this.usageCache.get(tenantId)!;
    const currentUsage = tenantUsage.get(featureId) || 0;
    const newUsage = currentUsage + amount;

    tenantUsage.set(featureId, newUsage);
    return newUsage;
  }

  /**
   * Decrement feature usage (for rollback scenarios)
   */
  async decrementUsage(
    tenantId: string,
    featureId: string,
    amount: number = 1
  ): Promise<number> {
    if (!this.usageCache.has(tenantId)) {
      return 0;
    }

    const tenantUsage = this.usageCache.get(tenantId)!;
    const currentUsage = tenantUsage.get(featureId) || 0;
    const newUsage = Math.max(0, currentUsage - amount);

    tenantUsage.set(featureId, newUsage);
    return newUsage;
  }

  /**
   * Get current usage for a feature
   */
  getCurrentUsage(tenantId: string, featureId: string): number {
    const tenantUsage = this.usageCache.get(tenantId);
    return tenantUsage?.get(featureId) || 0;
  }

  /**
   * Get all quota usage for a tenant
   */
  async getQuotaUsage(
    tenantId: string,
    plan: Plan,
    subscription: Subscription
  ): Promise<QuotaUsage[]> {
    const periodStart = subscription.currentPeriodStart;
    const periodEnd = subscription.currentPeriodEnd;

    return plan.features.map((feature) => {
      const used = this.getCurrentUsage(tenantId, feature.id);

      return {
        tenantId,
        featureId: feature.id,
        used,
        limit: feature.quota || 0,
        unlimited: feature.unlimited,
        periodStart,
        periodEnd,
      };
    });
  }

  /**
   * Reset quota usage (called at period end)
   */
  async resetQuota(tenantId: string, featureId?: string): Promise<void> {
    if (!this.usageCache.has(tenantId)) {
      return;
    }

    const tenantUsage = this.usageCache.get(tenantId)!;

    if (featureId) {
      tenantUsage.delete(featureId);
    } else {
      tenantUsage.clear();
    }
  }

  /**
   * Check if feature is available in plan
   */
  isFeatureAvailable(plan: Plan, featureId: string): boolean {
    const feature = plan.features.find((f) => f.id === featureId);
    return feature ? feature.enabled : false;
  }

  /**
   * Get feature details
   */
  getFeature(plan: Plan, featureId: string): PlanFeature | undefined {
    return plan.features.find((f) => f.id === featureId);
  }

  /**
   * Check if quota is near limit (for warnings)
   */
  async isNearLimit(
    tenantId: string,
    featureId: string,
    plan: Plan,
    threshold: number = 0.8
  ): Promise<boolean> {
    const check = await this.checkQuota(tenantId, featureId, plan);

    if (check.unlimited) {
      return false;
    }

    const usageRatio = 1 - check.remaining / check.limit;
    return usageRatio >= threshold;
  }

  /**
   * Get quota warnings for all features
   */
  async getQuotaWarnings(
    tenantId: string,
    plan: Plan
  ): Promise<Array<{ featureId: string; featureName: string; percentUsed: number }>> {
    const warnings: Array<{ featureId: string; featureName: string; percentUsed: number }> = [];

    for (const feature of plan.features) {
      if (feature.unlimited || !feature.enabled) {
        continue;
      }

      const used = this.getCurrentUsage(tenantId, feature.id);
      const limit = feature.quota || 0;
      const percentUsed = (used / limit) * 100;

      if (percentUsed >= 80) {
        warnings.push({
          featureId: feature.id,
          featureName: feature.name,
          percentUsed,
        });
      }
    }

    return warnings;
  }

  /**
   * Batch check multiple features
   */
  async batchCheckQuotas(
    tenantId: string,
    plan: Plan,
    checks: Array<{ featureId: string; amount: number }>
  ): Promise<Map<string, QuotaCheck>> {
    const results = new Map<string, QuotaCheck>();

    for (const check of checks) {
      const result = await this.checkQuota(
        tenantId,
        check.featureId,
        plan,
        check.amount
      );
      results.set(check.featureId, result);
    }

    return results;
  }

  /**
   * Set custom quota override (for enterprise customers)
   */
  async setCustomQuota(
    tenantId: string,
    featureId: string,
    customLimit: number
  ): Promise<void> {
    // In production, this would persist to database
    console.log(`Setting custom quota for ${tenantId}/${featureId}: ${customLimit}`);
  }

  /**
   * Get quota statistics
   */
  async getQuotaStatistics(tenantId: string, plan: Plan): Promise<{
    totalFeatures: number;
    featuresUsed: number;
    featuresNearLimit: number;
    featuresAtLimit: number;
  }> {
    const features = plan.features.filter((f) => f.enabled && !f.unlimited);

    let featuresUsed = 0;
    let featuresNearLimit = 0;
    let featuresAtLimit = 0;

    for (const feature of features) {
      const used = this.getCurrentUsage(tenantId, feature.id);
      const limit = feature.quota || 0;

      if (used > 0) {
        featuresUsed++;
      }

      const percentUsed = (used / limit) * 100;

      if (percentUsed >= 100) {
        featuresAtLimit++;
      } else if (percentUsed >= 80) {
        featuresNearLimit++;
      }
    }

    return {
      totalFeatures: features.length,
      featuresUsed,
      featuresNearLimit,
      featuresAtLimit,
    };
  }
}
