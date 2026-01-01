/**
 * Pricing Engine - Handles dynamic pricing calculations
 */

import { Decimal } from 'decimal.js';
import {
  Plan,
  PricingModel,
  PricingTier,
  MeteredComponent,
  UsageRecord,
  ComponentUsage,
  Discount,
  DiscountType,
} from '../types';

export class PricingEngine {
  /**
   * Calculate subscription price
   */
  calculateSubscriptionPrice(
    plan: Plan,
    quantity: number = 1,
    usageRecords?: UsageRecord[]
  ): Decimal {
    let total = new Decimal(0);

    switch (plan.pricingModel) {
      case PricingModel.FLAT_RATE:
        total = plan.amount;
        break;

      case PricingModel.PER_SEAT:
        total = plan.amount.times(quantity);
        break;

      case PricingModel.TIERED:
        if (!plan.tiers || plan.tiers.length === 0) {
          throw new Error('Tiered pricing requires tiers');
        }
        total = this.calculateTieredPrice(plan.tiers, quantity);
        break;

      case PricingModel.VOLUME:
        if (!plan.tiers || plan.tiers.length === 0) {
          throw new Error('Volume pricing requires tiers');
        }
        total = this.calculateVolumePrice(plan.tiers, quantity);
        break;

      case PricingModel.METERED:
        if (!plan.meteredComponents || !usageRecords) {
          throw new Error('Metered pricing requires components and usage records');
        }
        total = this.calculateMeteredPrice(plan.meteredComponents, usageRecords);
        break;

      case PricingModel.HYBRID:
        total = plan.amount; // Base amount
        if (usageRecords && plan.meteredComponents) {
          const meteredCost = this.calculateMeteredPrice(
            plan.meteredComponents,
            usageRecords
          );
          total = total.plus(meteredCost);
        }
        break;

      default:
        throw new Error(`Unknown pricing model: ${plan.pricingModel}`);
    }

    return total;
  }

  /**
   * Calculate tiered pricing (graduated)
   * Each tier is charged separately for the quantity within that tier
   */
  private calculateTieredPrice(tiers: PricingTier[], quantity: number): Decimal {
    let total = new Decimal(0);
    let remaining = quantity;

    for (let i = 0; i < tiers.length; i++) {
      const tier = tiers[i];
      const nextTierStart = tier.upTo ?? Infinity;
      const previousTierEnd = i > 0 ? (tiers[i - 1].upTo ?? 0) : 0;
      const tierSize = nextTierStart - previousTierEnd;
      const unitsInTier = Math.min(remaining, tierSize);

      if (unitsInTier <= 0) break;

      // Add flat amount if present (only once per tier reached)
      if (tier.flatAmount) {
        total = total.plus(tier.flatAmount);
      }

      // Add unit cost for units in this tier
      total = total.plus(tier.unitAmount.times(unitsInTier));

      remaining -= unitsInTier;

      if (remaining <= 0) break;
    }

    return total;
  }

  /**
   * Calculate volume pricing
   * All units charged at the rate of the tier they fall into
   */
  private calculateVolumePrice(tiers: PricingTier[], quantity: number): Decimal {
    // Find the applicable tier
    const tier = tiers.find((t) => !t.upTo || quantity <= t.upTo);

    if (!tier) {
      // Use the last tier if quantity exceeds all tiers
      const lastTier = tiers[tiers.length - 1];
      return lastTier.unitAmount.times(quantity);
    }

    let total = tier.unitAmount.times(quantity);

    // Add flat amount if present
    if (tier.flatAmount) {
      total = total.plus(tier.flatAmount);
    }

    return total;
  }

  /**
   * Calculate metered pricing based on usage records
   */
  private calculateMeteredPrice(
    components: MeteredComponent[],
    usageRecords: UsageRecord[]
  ): Decimal {
    let total = new Decimal(0);

    for (const component of components) {
      const componentRecords = usageRecords.filter(
        (r) => r.meteredComponentId === component.id
      );

      const totalUsage = componentRecords.reduce((sum, r) => sum + r.quantity, 0);

      let componentCost: Decimal;

      if (component.tiers && component.tiers.length > 0) {
        // Use tiered pricing for this component
        componentCost = this.calculateTieredPrice(component.tiers, totalUsage);
      } else if (component.unitAmount) {
        // Simple per-unit pricing
        componentCost = component.unitAmount.times(totalUsage);
      } else {
        throw new Error(`Component ${component.id} has no pricing configuration`);
      }

      total = total.plus(componentCost);
    }

    return total;
  }

  /**
   * Calculate usage cost for specific component
   */
  calculateComponentUsage(
    component: MeteredComponent,
    usageRecords: UsageRecord[]
  ): ComponentUsage {
    const componentRecords = usageRecords.filter(
      (r) => r.meteredComponentId === component.id
    );

    const totalUsage = componentRecords.reduce((sum, r) => sum + r.quantity, 0);

    let cost: Decimal;
    if (component.tiers && component.tiers.length > 0) {
      cost = this.calculateTieredPrice(component.tiers, totalUsage);
    } else if (component.unitAmount) {
      cost = component.unitAmount.times(totalUsage);
    } else {
      cost = new Decimal(0);
    }

    return {
      meteredComponentId: component.id,
      componentName: component.name,
      totalUsage,
      unit: component.unit,
      cost,
    };
  }

  /**
   * Apply discount to amount
   */
  applyDiscount(amount: Decimal, discount: Discount): Decimal {
    switch (discount.type) {
      case DiscountType.PERCENTAGE:
        const discountAmount = amount.times(discount.value.dividedBy(100));
        return amount.minus(discountAmount);

      case DiscountType.FIXED_AMOUNT:
        const discounted = amount.minus(discount.value);
        return discounted.isNegative() ? new Decimal(0) : discounted;

      case DiscountType.FREE_TRIAL:
        return new Decimal(0);

      default:
        throw new Error(`Unknown discount type: ${discount.type}`);
    }
  }

  /**
   * Calculate tax amount
   */
  calculateTax(subtotal: Decimal, taxRate: number): Decimal {
    return subtotal.times(taxRate);
  }

  /**
   * Calculate total with tax
   */
  calculateTotal(subtotal: Decimal, taxRate: number): Decimal {
    const tax = this.calculateTax(subtotal, taxRate);
    return subtotal.plus(tax);
  }

  /**
   * Estimate monthly cost for metered billing
   */
  estimateMonthlyMeteredCost(
    components: MeteredComponent[],
    estimatedUsage: Map<string, number>
  ): Decimal {
    let total = new Decimal(0);

    for (const component of components) {
      const usage = estimatedUsage.get(component.id) ?? 0;

      let componentCost: Decimal;

      if (component.tiers && component.tiers.length > 0) {
        componentCost = this.calculateTieredPrice(component.tiers, usage);
      } else if (component.unitAmount) {
        componentCost = component.unitAmount.times(usage);
      } else {
        componentCost = new Decimal(0);
      }

      total = total.plus(componentCost);
    }

    return total;
  }

  /**
   * Calculate price comparison between plans
   */
  comparePlans(
    plans: Plan[],
    quantity: number,
    usageEstimates?: Map<string, Map<string, number>>
  ): Array<{ planId: string; planName: string; estimatedCost: Decimal }> {
    return plans.map((plan) => {
      const usageRecords = usageEstimates?.get(plan.id);
      let cost: Decimal;

      if (plan.pricingModel === PricingModel.METERED && plan.meteredComponents) {
        const mockRecords: UsageRecord[] = [];
        const estimates = usageRecords ?? new Map();

        for (const component of plan.meteredComponents) {
          const usage = estimates.get(component.id) ?? 0;
          if (usage > 0) {
            mockRecords.push({
              id: '',
              tenantId: '',
              subscriptionId: '',
              meteredComponentId: component.id,
              quantity: usage,
              timestamp: new Date(),
              metadata: {},
            });
          }
        }

        cost = this.calculateMeteredPrice(plan.meteredComponents, mockRecords);
      } else {
        cost = this.calculateSubscriptionPrice(plan, quantity);
      }

      return {
        planId: plan.id,
        planName: plan.name,
        estimatedCost: cost,
      };
    });
  }

  /**
   * Calculate annual savings for yearly vs monthly billing
   */
  calculateAnnualSavings(monthlyPrice: Decimal, yearlyPrice: Decimal): {
    savingsAmount: Decimal;
    savingsPercentage: Decimal;
  } {
    const annualMonthly = monthlyPrice.times(12);
    const savingsAmount = annualMonthly.minus(yearlyPrice);
    const savingsPercentage = savingsAmount.dividedBy(annualMonthly).times(100);

    return {
      savingsAmount,
      savingsPercentage,
    };
  }
}
