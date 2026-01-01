/**
 * Enterprise API Gateway - Metrics Service
 *
 * Collect and aggregate gateway metrics
 */

import type {
  GatewayRequest,
  GatewayResponse,
  RequestMetrics,
  AggregatedMetrics,
  TrafficMetrics,
} from '../types';

export class MetricsService {
  private metrics: RequestMetrics[] = [];
  private readonly maxMetrics = 10000; // Keep last 10k requests
  private readonly retentionMs = 3600000; // 1 hour

  /**
   * Record a request
   */
  public recordRequest(
    request: GatewayRequest,
    response: GatewayResponse,
    cached: boolean
  ): void {
    const metric: RequestMetrics = {
      routeId: '', // Would be set by route resolver
      consumerId: request.consumer?.id,
      method: request.method,
      path: request.path,
      statusCode: response.statusCode,
      duration: response.duration,
      upstream: response.upstream,
      cached,
      rateLimited: false,
      timestamp: Date.now(),
    };

    this.metrics.push(metric);
    this.cleanup();
  }

  /**
   * Record an error
   */
  public recordError(request: GatewayRequest): void {
    const metric: RequestMetrics = {
      routeId: '',
      consumerId: request.consumer?.id,
      method: request.method,
      path: request.path,
      statusCode: 500,
      duration: 0,
      cached: false,
      rateLimited: false,
      timestamp: Date.now(),
    };

    this.metrics.push(metric);
    this.cleanup();
  }

  /**
   * Record rate limit
   */
  public recordRateLimit(request: GatewayRequest): void {
    const metric: RequestMetrics = {
      routeId: '',
      consumerId: request.consumer?.id,
      method: request.method,
      path: request.path,
      statusCode: 429,
      duration: 0,
      cached: false,
      rateLimited: true,
      timestamp: Date.now(),
    };

    this.metrics.push(metric);
    this.cleanup();
  }

  /**
   * Get aggregated metrics
   */
  public getAggregatedMetrics(timeRange?: number): AggregatedMetrics {
    const now = Date.now();
    const startTime = timeRange ? now - timeRange : now - this.retentionMs;

    const relevantMetrics = this.metrics.filter((m) => m.timestamp >= startTime);

    if (relevantMetrics.length === 0) {
      return {
        totalRequests: 0,
        successRate: 0,
        averageLatency: 0,
        p50Latency: 0,
        p95Latency: 0,
        p99Latency: 0,
        requestsPerSecond: 0,
        errorRate: 0,
        cacheHitRate: 0,
        rateLimitRate: 0,
      };
    }

    const totalRequests = relevantMetrics.length;
    const successCount = relevantMetrics.filter((m) => m.statusCode >= 200 && m.statusCode < 300).length;
    const errorCount = relevantMetrics.filter((m) => m.statusCode >= 500).length;
    const cachedCount = relevantMetrics.filter((m) => m.cached).length;
    const rateLimitedCount = relevantMetrics.filter((m) => m.rateLimited).length;

    const durations = relevantMetrics.map((m) => m.duration).sort((a, b) => a - b);
    const avgLatency = durations.reduce((sum, d) => sum + d, 0) / durations.length;

    const timeRangeSeconds = (now - startTime) / 1000;
    const requestsPerSecond = totalRequests / timeRangeSeconds;

    return {
      totalRequests,
      successRate: successCount / totalRequests,
      averageLatency: avgLatency,
      p50Latency: this.percentile(durations, 0.5),
      p95Latency: this.percentile(durations, 0.95),
      p99Latency: this.percentile(durations, 0.99),
      requestsPerSecond,
      errorRate: errorCount / totalRequests,
      cacheHitRate: cachedCount / totalRequests,
      rateLimitRate: rateLimitedCount / totalRequests,
    };
  }

  /**
   * Get traffic metrics by route
   */
  public getTrafficByRoute(): TrafficMetrics[] {
    const routeMetrics = new Map<string, RequestMetrics[]>();

    for (const metric of this.metrics) {
      const key = metric.routeId || metric.path;
      const existing = routeMetrics.get(key) || [];
      existing.push(metric);
      routeMetrics.set(key, existing);
    }

    const result: TrafficMetrics[] = [];

    for (const [route, metrics] of routeMetrics) {
      const durations = metrics.map((m) => m.duration).sort((a, b) => a - b);
      const errors = metrics.filter((m) => m.statusCode >= 400).length;
      const bandwidth = metrics.reduce((sum, m) => sum + (m.duration || 0), 0);

      result.push({
        route,
        requests: metrics.length,
        bandwidth,
        errors,
        latency: {
          avg: durations.reduce((sum, d) => sum + d, 0) / durations.length,
          p50: this.percentile(durations, 0.5),
          p95: this.percentile(durations, 0.95),
          p99: this.percentile(durations, 0.99),
        },
      });
    }

    return result.sort((a, b) => b.requests - a.requests);
  }

  /**
   * Get metrics by status code
   */
  public getStatusCodeDistribution(): Record<number, number> {
    const distribution: Record<number, number> = {};

    for (const metric of this.metrics) {
      distribution[metric.statusCode] = (distribution[metric.statusCode] || 0) + 1;
    }

    return distribution;
  }

  /**
   * Get metrics by consumer
   */
  public getConsumerMetrics(consumerId: string): AggregatedMetrics {
    const consumerMetrics = this.metrics.filter((m) => m.consumerId === consumerId);

    if (consumerMetrics.length === 0) {
      return {
        totalRequests: 0,
        successRate: 0,
        averageLatency: 0,
        p50Latency: 0,
        p95Latency: 0,
        p99Latency: 0,
        requestsPerSecond: 0,
        errorRate: 0,
        cacheHitRate: 0,
        rateLimitRate: 0,
      };
    }

    const successCount = consumerMetrics.filter((m) => m.statusCode >= 200 && m.statusCode < 300).length;
    const errorCount = consumerMetrics.filter((m) => m.statusCode >= 500).length;
    const cachedCount = consumerMetrics.filter((m) => m.cached).length;
    const rateLimitedCount = consumerMetrics.filter((m) => m.rateLimited).length;

    const durations = consumerMetrics.map((m) => m.duration).sort((a, b) => a - b);
    const avgLatency = durations.reduce((sum, d) => sum + d, 0) / durations.length;

    const timeRange = Date.now() - consumerMetrics[0]!.timestamp;
    const requestsPerSecond = consumerMetrics.length / (timeRange / 1000);

    return {
      totalRequests: consumerMetrics.length,
      successRate: successCount / consumerMetrics.length,
      averageLatency: avgLatency,
      p50Latency: this.percentile(durations, 0.5),
      p95Latency: this.percentile(durations, 0.95),
      p99Latency: this.percentile(durations, 0.99),
      requestsPerSecond,
      errorRate: errorCount / consumerMetrics.length,
      cacheHitRate: cachedCount / consumerMetrics.length,
      rateLimitRate: rateLimitedCount / consumerMetrics.length,
    };
  }

  /**
   * Get time series data
   */
  public getTimeSeries(interval: number = 60000): Array<{ timestamp: number; count: number; avgLatency: number }> {
    const buckets = new Map<number, RequestMetrics[]>();

    for (const metric of this.metrics) {
      const bucket = Math.floor(metric.timestamp / interval) * interval;
      const existing = buckets.get(bucket) || [];
      existing.push(metric);
      buckets.set(bucket, existing);
    }

    const result: Array<{ timestamp: number; count: number; avgLatency: number }> = [];

    for (const [timestamp, metrics] of buckets) {
      const avgLatency = metrics.reduce((sum, m) => sum + m.duration, 0) / metrics.length;

      result.push({
        timestamp,
        count: metrics.length,
        avgLatency,
      });
    }

    return result.sort((a, b) => a.timestamp - b.timestamp);
  }

  /**
   * Calculate percentile
   */
  private percentile(sorted: number[], p: number): number {
    if (sorted.length === 0) return 0;

    const index = Math.ceil(sorted.length * p) - 1;
    return sorted[Math.max(0, index)]!;
  }

  /**
   * Clean up old metrics
   */
  private cleanup(): void {
    const now = Date.now();

    // Remove metrics older than retention period
    this.metrics = this.metrics.filter((m) => now - m.timestamp < this.retentionMs);

    // Limit total number of metrics
    if (this.metrics.length > this.maxMetrics) {
      this.metrics = this.metrics.slice(-this.maxMetrics);
    }
  }

  /**
   * Clear all metrics
   */
  public clear(): void {
    this.metrics = [];
  }

  /**
   * Export metrics
   */
  public exportMetrics(): RequestMetrics[] {
    return [...this.metrics];
  }
}
