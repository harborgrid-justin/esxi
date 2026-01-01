/**
 * Revenue Analytics - Track MRR, ARR, churn, and other revenue metrics
 */

import { Decimal } from 'decimal.js';
import {
  RevenueMetrics,
  ChurnMetrics,
  CohortAnalysis,
  Subscription,
  Plan,
  Invoice,
  PlanInterval,
} from '../types';
import { startOfMonth, endOfMonth, format, differenceInMonths } from 'date-fns';

export class RevenueAnalytics {
  private subscriptions: Subscription[] = [];
  private plans: Map<string, Plan> = new Map();

  /**
   * Set data sources
   */
  setData(subscriptions: Subscription[], plans: Plan[]): void {
    this.subscriptions = subscriptions;
    plans.forEach((plan) => this.plans.set(plan.id, plan));
  }

  /**
   * Calculate Monthly Recurring Revenue (MRR)
   */
  calculateMRR(date: Date = new Date()): Decimal {
    let mrr = new Decimal(0);

    for (const subscription of this.subscriptions) {
      if (
        subscription.status !== 'active' &&
        subscription.status !== 'trial'
      ) {
        continue;
      }

      const plan = this.plans.get(subscription.planId);
      if (!plan) continue;

      const monthlyAmount = this.normalizeToMonthly(
        plan.amount.times(subscription.quantity),
        plan.interval
      );

      mrr = mrr.plus(monthlyAmount);
    }

    return mrr;
  }

  /**
   * Calculate Annual Recurring Revenue (ARR)
   */
  calculateARR(date: Date = new Date()): Decimal {
    return this.calculateMRR(date).times(12);
  }

  /**
   * Calculate revenue metrics for a period
   */
  calculateRevenueMetrics(
    periodStart: Date,
    periodEnd: Date
  ): RevenueMetrics {
    const currentMrr = this.calculateMRR(periodEnd);
    const previousMrr = this.calculateMRR(periodStart);

    // Calculate MRR movements
    const { newMrr, expansionMrr, contractionMrr, churnedMrr } =
      this.calculateMrrMovements(periodStart, periodEnd);

    const netMrr = newMrr.plus(expansionMrr).minus(contractionMrr).minus(churnedMrr);

    return {
      mrr: currentMrr,
      arr: currentMrr.times(12),
      newMrr,
      expansionMrr,
      contractionMrr,
      churnedMrr,
      netMrr,
      period: `${format(periodStart, 'yyyy-MM')} - ${format(periodEnd, 'yyyy-MM')}`,
      timestamp: new Date(),
    };
  }

  /**
   * Calculate MRR movements (new, expansion, contraction, churn)
   */
  private calculateMrrMovements(
    periodStart: Date,
    periodEnd: Date
  ): {
    newMrr: Decimal;
    expansionMrr: Decimal;
    contractionMrr: Decimal;
    churnedMrr: Decimal;
  } {
    let newMrr = new Decimal(0);
    let expansionMrr = new Decimal(0);
    let contractionMrr = new Decimal(0);
    let churnedMrr = new Decimal(0);

    for (const subscription of this.subscriptions) {
      const plan = this.plans.get(subscription.planId);
      if (!plan) continue;

      const monthlyAmount = this.normalizeToMonthly(
        plan.amount.times(subscription.quantity),
        plan.interval
      );

      // New MRR - subscriptions created in period
      if (
        subscription.createdAt >= periodStart &&
        subscription.createdAt <= periodEnd
      ) {
        newMrr = newMrr.plus(monthlyAmount);
      }

      // Churned MRR - subscriptions canceled in period
      if (
        subscription.canceledAt &&
        subscription.canceledAt >= periodStart &&
        subscription.canceledAt <= periodEnd
      ) {
        churnedMrr = churnedMrr.plus(monthlyAmount);
      }

      // Expansion/Contraction would require tracking quantity changes
      // This is a simplified implementation
    }

    return {
      newMrr,
      expansionMrr,
      contractionMrr,
      churnedMrr,
    };
  }

  /**
   * Calculate churn metrics
   */
  calculateChurnMetrics(
    periodStart: Date,
    periodEnd: Date
  ): ChurnMetrics {
    const startSubscriptions = this.subscriptions.filter(
      (s) => s.createdAt < periodStart && s.status === 'active'
    );

    const churnedSubscriptions = this.subscriptions.filter(
      (s) =>
        s.canceledAt &&
        s.canceledAt >= periodStart &&
        s.canceledAt <= periodEnd
    );

    const totalCustomers = startSubscriptions.length;
    const churnedCustomers = churnedSubscriptions.length;
    const customerChurnRate =
      totalCustomers > 0 ? churnedCustomers / totalCustomers : 0;

    // Calculate revenue churn
    let totalRevenue = new Decimal(0);
    let churnedRevenue = new Decimal(0);

    for (const subscription of startSubscriptions) {
      const plan = this.plans.get(subscription.planId);
      if (!plan) continue;

      const monthlyAmount = this.normalizeToMonthly(
        plan.amount.times(subscription.quantity),
        plan.interval
      );

      totalRevenue = totalRevenue.plus(monthlyAmount);

      if (churnedSubscriptions.some((s) => s.id === subscription.id)) {
        churnedRevenue = churnedRevenue.plus(monthlyAmount);
      }
    }

    const revenueChurnRate = totalRevenue.greaterThan(0)
      ? churnedRevenue.dividedBy(totalRevenue).toNumber()
      : 0;

    return {
      customerChurnRate,
      revenueChurnRate,
      period: `${format(periodStart, 'yyyy-MM')} - ${format(periodEnd, 'yyyy-MM')}`,
      churnedCustomers,
      totalCustomers,
      churnedRevenue,
      totalRevenue,
    };
  }

  /**
   * Generate cohort analysis
   */
  generateCohortAnalysis(cohortDate: Date): CohortAnalysis[] {
    const cohortMonth = format(cohortDate, 'yyyy-MM');

    // Get subscriptions that started in the cohort month
    const cohortSubscriptions = this.subscriptions.filter(
      (s) => format(s.createdAt, 'yyyy-MM') === cohortMonth
    );

    const cohortSize = cohortSubscriptions.length;
    const analysis: CohortAnalysis[] = [];

    // Analyze retention for each month after cohort
    for (let month = 0; month <= 12; month++) {
      const checkDate = new Date(cohortDate);
      checkDate.setMonth(checkDate.getMonth() + month);

      const retained = cohortSubscriptions.filter((s) => {
        if (s.status === 'canceled' && s.canceledAt && s.canceledAt < checkDate) {
          return false;
        }
        return true;
      });

      let revenue = new Decimal(0);
      for (const subscription of retained) {
        const plan = this.plans.get(subscription.planId);
        if (!plan) continue;

        revenue = revenue.plus(
          this.normalizeToMonthly(
            plan.amount.times(subscription.quantity),
            plan.interval
          )
        );
      }

      analysis.push({
        cohort: cohortMonth,
        month,
        customersRetained: retained.length,
        retentionRate: cohortSize > 0 ? retained.length / cohortSize : 0,
        revenue,
      });
    }

    return analysis;
  }

  /**
   * Calculate Customer Lifetime Value (LTV)
   */
  calculateLTV(
    averageMonthlyRevenue: Decimal,
    averageCustomerLifespanMonths: number
  ): Decimal {
    return averageMonthlyRevenue.times(averageCustomerLifespanMonths);
  }

  /**
   * Calculate Average Revenue Per User (ARPU)
   */
  calculateARPU(): Decimal {
    const activeSubscriptions = this.subscriptions.filter(
      (s) => s.status === 'active' || s.status === 'trial'
    );

    if (activeSubscriptions.length === 0) {
      return new Decimal(0);
    }

    const mrr = this.calculateMRR();
    return mrr.dividedBy(activeSubscriptions.length);
  }

  /**
   * Calculate MRR growth rate
   */
  calculateMrrGrowthRate(
    currentMrr: Decimal,
    previousMrr: Decimal
  ): number {
    if (previousMrr.equals(0)) {
      return 0;
    }

    return currentMrr
      .minus(previousMrr)
      .dividedBy(previousMrr)
      .times(100)
      .toNumber();
  }

  /**
   * Get revenue breakdown by plan
   */
  getRevenueByPlan(): Array<{
    planId: string;
    planName: string;
    mrr: Decimal;
    subscribers: number;
    percentage: number;
  }> {
    const totalMrr = this.calculateMRR();
    const breakdown = new Map<string, { mrr: Decimal; subscribers: number }>();

    for (const subscription of this.subscriptions) {
      if (
        subscription.status !== 'active' &&
        subscription.status !== 'trial'
      ) {
        continue;
      }

      const plan = this.plans.get(subscription.planId);
      if (!plan) continue;

      const monthlyAmount = this.normalizeToMonthly(
        plan.amount.times(subscription.quantity),
        plan.interval
      );

      const existing = breakdown.get(subscription.planId) || {
        mrr: new Decimal(0),
        subscribers: 0,
      };

      breakdown.set(subscription.planId, {
        mrr: existing.mrr.plus(monthlyAmount),
        subscribers: existing.subscribers + 1,
      });
    }

    return Array.from(breakdown.entries()).map(([planId, data]) => {
      const plan = this.plans.get(planId)!;
      return {
        planId,
        planName: plan.name,
        mrr: data.mrr,
        subscribers: data.subscribers,
        percentage: totalMrr.greaterThan(0)
          ? data.mrr.dividedBy(totalMrr).times(100).toNumber()
          : 0,
      };
    });
  }

  /**
   * Normalize amount to monthly equivalent
   */
  private normalizeToMonthly(amount: Decimal, interval: PlanInterval): Decimal {
    switch (interval) {
      case PlanInterval.MONTHLY:
        return amount;
      case PlanInterval.QUARTERLY:
        return amount.dividedBy(3);
      case PlanInterval.YEARLY:
        return amount.dividedBy(12);
      case PlanInterval.LIFETIME:
        return amount.dividedBy(120); // Assume 10 year lifetime
      default:
        return amount;
    }
  }

  /**
   * Get quick revenue metrics
   */
  getQuickMetrics(): {
    mrr: string;
    arr: string;
    activeSubscriptions: number;
    arpu: string;
    trialConversionRate: number;
  } {
    const mrr = this.calculateMRR();
    const arr = mrr.times(12);
    const activeSubscriptions = this.subscriptions.filter(
      (s) => s.status === 'active'
    ).length;
    const arpu = this.calculateARPU();

    const totalTrials = this.subscriptions.filter((s) => s.trialEnd).length;
    const convertedTrials = this.subscriptions.filter(
      (s) => s.trialEnd && s.status === 'active'
    ).length;
    const trialConversionRate =
      totalTrials > 0 ? (convertedTrials / totalTrials) * 100 : 0;

    return {
      mrr: `$${mrr.toFixed(2)}`,
      arr: `$${arr.toFixed(2)}`,
      activeSubscriptions,
      arpu: `$${arpu.toFixed(2)}`,
      trialConversionRate,
    };
  }
}
