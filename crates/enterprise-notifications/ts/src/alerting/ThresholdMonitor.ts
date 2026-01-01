/**
 * ThresholdMonitor - Monitors metrics against thresholds
 * Tracks metric values and triggers alerts when thresholds are breached
 */

import { EventEmitter } from 'events';
import { Threshold, ThresholdType, RuleConditionOperator } from '../types';

export interface MetricPoint {
  metric: string;
  value: number;
  timestamp: Date;
  tags?: Record<string, string>;
}

export interface ThresholdBreach {
  threshold: Threshold;
  metric: string;
  currentValue: number;
  thresholdValue: number;
  breachedAt: Date;
  duration?: number;
}

export class ThresholdMonitor extends EventEmitter {
  private thresholds: Map<string, Threshold>;
  private metricHistory: Map<string, MetricPoint[]>;
  private breaches: Map<string, ThresholdBreach>;
  private monitoringInterval?: NodeJS.Timeout;

  constructor() {
    super();
    this.thresholds = new Map();
    this.metricHistory = new Map();
    this.breaches = new Map();
  }

  /**
   * Add threshold to monitor
   */
  addThreshold(threshold: Threshold): void {
    this.thresholds.set(threshold.id, threshold);
    this.emit('threshold:added', threshold);
  }

  /**
   * Remove threshold
   */
  removeThreshold(thresholdId: string): void {
    this.thresholds.delete(thresholdId);
    this.emit('threshold:removed', thresholdId);
  }

  /**
   * Record metric point
   */
  recordMetric(point: MetricPoint): void {
    let history = this.metricHistory.get(point.metric);
    if (!history) {
      history = [];
      this.metricHistory.set(point.metric, history);
    }

    history.push(point);

    // Keep only last 1000 points per metric
    if (history.length > 1000) {
      history.shift();
    }

    // Check thresholds
    this.checkThresholds(point);
  }

  /**
   * Check metric against all thresholds
   */
  private checkThresholds(point: MetricPoint): void {
    for (const threshold of this.thresholds.values()) {
      if (threshold.metric !== point.metric) {
        continue;
      }

      const thresholdValue = this.calculateThresholdValue(threshold);
      const isBreached = this.isThresholdBreached(point.value, threshold.operator, thresholdValue);

      if (isBreached) {
        this.handleBreach(threshold, point, thresholdValue);
      } else {
        this.clearBreach(threshold.id);
      }
    }
  }

  /**
   * Calculate threshold value
   */
  private calculateThresholdValue(threshold: Threshold): number {
    switch (threshold.type) {
      case ThresholdType.STATIC:
        return threshold.value;

      case ThresholdType.DYNAMIC:
        return this.calculateDynamicThreshold(threshold);

      case ThresholdType.BASELINE:
        return this.calculateBaselineThreshold(threshold);

      case ThresholdType.PERCENTAGE:
        return this.calculatePercentageThreshold(threshold);

      default:
        return threshold.value;
    }
  }

  /**
   * Calculate dynamic threshold
   */
  private calculateDynamicThreshold(threshold: Threshold): number {
    if (!threshold.baselineWindow || !threshold.deviationMultiplier) {
      return threshold.value;
    }

    const baseline = this.calculateBaseline(threshold.metric, threshold.baselineWindow);
    return baseline * threshold.deviationMultiplier;
  }

  /**
   * Calculate baseline threshold
   */
  private calculateBaselineThreshold(threshold: Threshold): number {
    if (!threshold.baselineWindow) {
      return threshold.value;
    }

    return this.calculateBaseline(threshold.metric, threshold.baselineWindow);
  }

  /**
   * Calculate percentage threshold
   */
  private calculatePercentageThreshold(threshold: Threshold): number {
    if (!threshold.percentageOf) {
      return threshold.value;
    }

    const baseMetric = this.getLatestValue(threshold.percentageOf) ?? 0;
    return (baseMetric * threshold.value) / 100;
  }

  /**
   * Calculate baseline from history
   */
  private calculateBaseline(metric: string, windowSeconds: number): number {
    const history = this.metricHistory.get(metric) ?? [];
    const cutoff = Date.now() - windowSeconds * 1000;
    const recent = history.filter(p => p.timestamp.getTime() > cutoff);

    if (recent.length === 0) {
      return 0;
    }

    const sum = recent.reduce((acc, p) => acc + p.value, 0);
    return sum / recent.length;
  }

  /**
   * Get latest metric value
   */
  private getLatestValue(metric: string): number | undefined {
    const history = this.metricHistory.get(metric);
    return history?.[history.length - 1]?.value;
  }

  /**
   * Check if threshold is breached
   */
  private isThresholdBreached(
    value: number,
    operator: RuleConditionOperator,
    threshold: number
  ): boolean {
    switch (operator) {
      case RuleConditionOperator.GREATER_THAN:
        return value > threshold;
      case RuleConditionOperator.GREATER_THAN_OR_EQUAL:
        return value >= threshold;
      case RuleConditionOperator.LESS_THAN:
        return value < threshold;
      case RuleConditionOperator.LESS_THAN_OR_EQUAL:
        return value <= threshold;
      case RuleConditionOperator.EQUALS:
        return value === threshold;
      case RuleConditionOperator.NOT_EQUALS:
        return value !== threshold;
      default:
        return false;
    }
  }

  /**
   * Handle threshold breach
   */
  private handleBreach(threshold: Threshold, point: MetricPoint, thresholdValue: number): void {
    const breachKey = threshold.id;
    const existing = this.breaches.get(breachKey);

    if (existing) {
      // Update existing breach
      existing.duration = Date.now() - existing.breachedAt.getTime();
    } else {
      // New breach
      const breach: ThresholdBreach = {
        threshold,
        metric: point.metric,
        currentValue: point.value,
        thresholdValue,
        breachedAt: point.timestamp,
      };

      this.breaches.set(breachKey, breach);
      this.emit('threshold:breached', breach);
    }
  }

  /**
   * Clear breach
   */
  private clearBreach(thresholdId: string): void {
    const breach = this.breaches.get(thresholdId);
    if (breach) {
      this.breaches.delete(thresholdId);
      this.emit('threshold:recovered', breach);
    }
  }

  /**
   * Get active breaches
   */
  getActiveBreaches(): ThresholdBreach[] {
    return Array.from(this.breaches.values());
  }

  /**
   * Get metric history
   */
  getHistory(metric: string, limit: number = 100): MetricPoint[] {
    const history = this.metricHistory.get(metric) ?? [];
    return history.slice(-limit);
  }

  /**
   * Clear metric history
   */
  clearHistory(metric?: string): void {
    if (metric) {
      this.metricHistory.delete(metric);
    } else {
      this.metricHistory.clear();
    }
  }
}

export default ThresholdMonitor;
