/**
 * Proration Engine - Handles plan changes and prorated charges
 */

import { Decimal } from 'decimal.js';
import { differenceInDays, differenceInSeconds } from 'date-fns';
import {
  Plan,
  Subscription,
  PlanChangeRequest,
  ProrationPreview,
  ProrationItem,
  PlanInterval,
} from '../types';

export class ProrationEngine {
  /**
   * Calculate proration for plan change
   */
  calculateProration(
    currentPlan: Plan,
    newPlan: Plan,
    subscription: Subscription,
    changeDate: Date = new Date()
  ): ProrationPreview {
    const proratedItems: ProrationItem[] = [];

    // Calculate unused time on current plan
    const totalPeriodSeconds = differenceInSeconds(
      subscription.currentPeriodEnd,
      subscription.currentPeriodStart
    );
    const usedPeriodSeconds = differenceInSeconds(
      changeDate,
      subscription.currentPeriodStart
    );
    const unusedPeriodSeconds = totalPeriodSeconds - usedPeriodSeconds;
    const unusedRatio = new Decimal(unusedPeriodSeconds).dividedBy(totalPeriodSeconds);

    // Calculate credit from current plan
    const currentPlanTotal = currentPlan.amount.times(subscription.quantity);
    const creditApplied = currentPlanTotal.times(unusedRatio);

    proratedItems.push({
      description: `Credit for unused time on ${currentPlan.name}`,
      amount: creditApplied.negated(),
      periodStart: changeDate,
      periodEnd: subscription.currentPeriodEnd,
    });

    // Calculate charge for new plan
    const newPlanTotal = newPlan.amount.times(subscription.quantity);
    const newPlanCharge = newPlanTotal.times(unusedRatio);

    proratedItems.push({
      description: `Prorated charge for ${newPlan.name}`,
      amount: newPlanCharge,
      periodStart: changeDate,
      periodEnd: subscription.currentPeriodEnd,
    });

    // Calculate immediate charge
    const immediateCharge = newPlanCharge.minus(creditApplied);

    // Next invoice will be full price
    const nextInvoiceAmount = newPlanTotal;

    return {
      immediateCharge: immediateCharge.isNegative()
        ? new Decimal(0)
        : immediateCharge,
      creditApplied,
      nextInvoiceAmount,
      proratedItems,
    };
  }

  /**
   * Calculate proration for quantity change
   */
  calculateQuantityChangeProration(
    plan: Plan,
    subscription: Subscription,
    newQuantity: number,
    changeDate: Date = new Date()
  ): ProrationPreview {
    const proratedItems: ProrationItem[] = [];

    const quantityDiff = newQuantity - subscription.quantity;

    if (quantityDiff === 0) {
      return {
        immediateCharge: new Decimal(0),
        creditApplied: new Decimal(0),
        nextInvoiceAmount: plan.amount.times(newQuantity),
        proratedItems: [],
      };
    }

    // Calculate unused time ratio
    const totalPeriodSeconds = differenceInSeconds(
      subscription.currentPeriodEnd,
      subscription.currentPeriodStart
    );
    const usedPeriodSeconds = differenceInSeconds(
      changeDate,
      subscription.currentPeriodStart
    );
    const unusedPeriodSeconds = totalPeriodSeconds - usedPeriodSeconds;
    const unusedRatio = new Decimal(unusedPeriodSeconds).dividedBy(totalPeriodSeconds);

    // Calculate proration for the difference in quantity
    const seatPrice = plan.amount;
    const proratedSeatCharge = seatPrice.times(Math.abs(quantityDiff)).times(unusedRatio);

    if (quantityDiff > 0) {
      // Adding seats
      proratedItems.push({
        description: `Prorated charge for ${quantityDiff} additional seat(s)`,
        amount: proratedSeatCharge,
        periodStart: changeDate,
        periodEnd: subscription.currentPeriodEnd,
      });

      return {
        immediateCharge: proratedSeatCharge,
        creditApplied: new Decimal(0),
        nextInvoiceAmount: plan.amount.times(newQuantity),
        proratedItems,
      };
    } else {
      // Removing seats
      proratedItems.push({
        description: `Credit for ${Math.abs(quantityDiff)} removed seat(s)`,
        amount: proratedSeatCharge.negated(),
        periodStart: changeDate,
        periodEnd: subscription.currentPeriodEnd,
      });

      return {
        immediateCharge: new Decimal(0),
        creditApplied: proratedSeatCharge,
        nextInvoiceAmount: plan.amount.times(newQuantity),
        proratedItems,
      };
    }
  }

  /**
   * Calculate proration for plan upgrade/downgrade
   */
  calculatePlanChangeProration(
    request: PlanChangeRequest,
    currentPlan: Plan,
    newPlan: Plan,
    subscription: Subscription
  ): ProrationPreview {
    const changeDate = request.effectiveDate ?? new Date();

    if (!request.prorate) {
      // No proration - change takes effect at end of period
      return {
        immediateCharge: new Decimal(0),
        creditApplied: new Decimal(0),
        nextInvoiceAmount: newPlan.amount.times(request.quantity ?? subscription.quantity),
        proratedItems: [],
      };
    }

    const newQuantity = request.quantity ?? subscription.quantity;
    let preview = this.calculateProration(currentPlan, newPlan, subscription, changeDate);

    // Adjust for quantity change if applicable
    if (newQuantity !== subscription.quantity) {
      const quantityChange = this.calculateQuantityChangeProration(
        newPlan,
        { ...subscription, quantity: newQuantity },
        newQuantity,
        changeDate
      );

      // Combine the results
      preview = {
        immediateCharge: preview.immediateCharge.plus(quantityChange.immediateCharge),
        creditApplied: preview.creditApplied.plus(quantityChange.creditApplied),
        nextInvoiceAmount: newPlan.amount.times(newQuantity),
        proratedItems: [...preview.proratedItems, ...quantityChange.proratedItems],
      };
    }

    return preview;
  }

  /**
   * Calculate daily proration amount
   */
  calculateDailyProration(monthlyAmount: Decimal, daysInMonth: number = 30): Decimal {
    return monthlyAmount.dividedBy(daysInMonth);
  }

  /**
   * Calculate proration for partial month
   */
  calculatePartialMonthProration(
    monthlyAmount: Decimal,
    startDate: Date,
    endDate: Date
  ): Decimal {
    const days = differenceInDays(endDate, startDate);
    const dailyRate = this.calculateDailyProration(monthlyAmount);
    return dailyRate.times(days);
  }

  /**
   * Calculate refund amount for canceled subscription
   */
  calculateCancellationRefund(
    subscription: Subscription,
    plan: Plan,
    cancellationDate: Date = new Date()
  ): {
    refundAmount: Decimal;
    refundRatio: Decimal;
  } {
    const totalPeriodSeconds = differenceInSeconds(
      subscription.currentPeriodEnd,
      subscription.currentPeriodStart
    );
    const usedPeriodSeconds = differenceInSeconds(
      cancellationDate,
      subscription.currentPeriodStart
    );
    const unusedPeriodSeconds = totalPeriodSeconds - usedPeriodSeconds;

    if (unusedPeriodSeconds <= 0) {
      return {
        refundAmount: new Decimal(0),
        refundRatio: new Decimal(0),
      };
    }

    const unusedRatio = new Decimal(unusedPeriodSeconds).dividedBy(totalPeriodSeconds);
    const paidAmount = plan.amount.times(subscription.quantity);
    const refundAmount = paidAmount.times(unusedRatio);

    return {
      refundAmount,
      refundRatio: unusedRatio,
    };
  }

  /**
   * Calculate trial conversion proration
   */
  calculateTrialConversionProration(
    subscription: Subscription,
    plan: Plan
  ): ProrationPreview {
    const now = new Date();

    // Calculate remaining time in current period
    const totalPeriodSeconds = differenceInSeconds(
      subscription.currentPeriodEnd,
      subscription.currentPeriodStart
    );
    const usedPeriodSeconds = differenceInSeconds(now, subscription.currentPeriodStart);
    const unusedPeriodSeconds = totalPeriodSeconds - usedPeriodSeconds;
    const unusedRatio = new Decimal(unusedPeriodSeconds).dividedBy(totalPeriodSeconds);

    const proratedCharge = plan.amount.times(subscription.quantity).times(unusedRatio);

    const items: ProrationItem[] = [
      {
        description: `Prorated charge from trial conversion`,
        amount: proratedCharge,
        periodStart: now,
        periodEnd: subscription.currentPeriodEnd,
      },
    ];

    return {
      immediateCharge: proratedCharge,
      creditApplied: new Decimal(0),
      nextInvoiceAmount: plan.amount.times(subscription.quantity),
      proratedItems: items,
    };
  }

  /**
   * Check if proration is beneficial for upgrade
   */
  isUpgradeBeneficial(
    currentPlan: Plan,
    newPlan: Plan,
    subscription: Subscription,
    changeDate: Date = new Date()
  ): {
    beneficial: boolean;
    savingsAmount: Decimal;
    breakEvenDays: number;
  } {
    const proration = this.calculateProration(
      currentPlan,
      newPlan,
      subscription,
      changeDate
    );

    const priceDifference = newPlan.amount.minus(currentPlan.amount);
    const daysRemaining = differenceInDays(subscription.currentPeriodEnd, changeDate);

    let beneficial = false;
    let savingsAmount = new Decimal(0);

    if (priceDifference.isNegative()) {
      // Downgrade - always beneficial cost-wise
      beneficial = true;
      savingsAmount = priceDifference.abs().times(subscription.quantity);
    }

    // Calculate break-even point
    let breakEvenDays = 0;
    if (priceDifference.greaterThan(0)) {
      const monthlyDifference = priceDifference.times(subscription.quantity);
      const dailyRate = this.calculateDailyProration(monthlyDifference);
      breakEvenDays = Math.ceil(
        proration.immediateCharge.dividedBy(dailyRate).toNumber()
      );
    }

    return {
      beneficial,
      savingsAmount,
      breakEvenDays,
    };
  }
}
