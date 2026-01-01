/**
 * AnalyticsService - Notification analytics and reporting
 */

import { EventEmitter } from 'events';
import { NotificationAnalytics, TimeSeriesPoint, NotificationChannelType, NotificationPriority } from '../types';

export class AnalyticsService extends EventEmitter {
  private metrics: Map<string, TimeSeriesPoint[]>;

  constructor() {
    super();
    this.metrics = new Map();
  }

  /**
   * Record notification metric
   */
  recordMetric(event: 'sent' | 'delivered' | 'failed' | 'read' | 'clicked', metadata: {
    tenantId: string;
    channel: NotificationChannelType;
    priority: NotificationPriority;
    timestamp?: Date;
  }): void {
    const key = `${metadata.tenantId}:${event}`;
    let points = this.metrics.get(key);
    if (!points) {
      points = [];
      this.metrics.set(key, points);
    }

    const timestamp = metadata.timestamp ?? new Date();
    const existing = points.find(p =>
      Math.abs(p.timestamp.getTime() - timestamp.getTime()) < 60000 // 1 minute window
    );

    if (existing) {
      existing[event]++;
    } else {
      points.push({
        timestamp,
        sent: event === 'sent' ? 1 : 0,
        delivered: event === 'delivered' ? 1 : 0,
        failed: event === 'failed' ? 1 : 0,
        read: event === 'read' ? 1 : 0,
        clicked: event === 'clicked' ? 1 : 0,
      });
    }

    // Keep only last 10000 points
    if (points.length > 10000) {
      points.shift();
    }
  }

  /**
   * Get analytics for period
   */
  async getAnalytics(tenantId: string, period: { start: Date; end: Date }): Promise<NotificationAnalytics> {
    const analytics: NotificationAnalytics = {
      tenantId,
      period,
      totalSent: 0,
      totalDelivered: 0,
      totalFailed: 0,
      totalRead: 0,
      totalClicked: 0,
      deliveryRate: 0,
      readRate: 0,
      clickRate: 0,
      failureRate: 0,
      byChannel: {} as Record<NotificationChannelType, any>,
      byPriority: {} as Record<NotificationPriority, number>,
      timeSeries: [],
    };

    // Aggregate metrics
    for (const [key, points] of this.metrics.entries()) {
      if (!key.startsWith(tenantId)) continue;

      const filtered = points.filter(
        p => p.timestamp >= period.start && p.timestamp <= period.end
      );

      for (const point of filtered) {
        analytics.totalSent += point.sent;
        analytics.totalDelivered += point.delivered;
        analytics.totalFailed += point.failed;
        analytics.totalRead += point.read;
        analytics.totalClicked += point.clicked;
      }
    }

    // Calculate rates
    if (analytics.totalSent > 0) {
      analytics.deliveryRate = analytics.totalDelivered / analytics.totalSent;
      analytics.failureRate = analytics.totalFailed / analytics.totalSent;
      analytics.readRate = analytics.totalRead / analytics.totalSent;
      analytics.clickRate = analytics.totalClicked / analytics.totalSent;
    }

    return analytics;
  }

  /**
   * Clear old metrics
   */
  clearOldMetrics(olderThan: Date): number {
    let cleared = 0;

    for (const [key, points] of this.metrics.entries()) {
      const filtered = points.filter(p => p.timestamp >= olderThan);
      const removed = points.length - filtered.length;

      if (removed > 0) {
        this.metrics.set(key, filtered);
        cleared += removed;
      }
    }

    return cleared;
  }
}

export default AnalyticsService;
