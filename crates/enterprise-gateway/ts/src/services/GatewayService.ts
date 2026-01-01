/**
 * Enterprise API Gateway - Gateway Service
 *
 * Core gateway operations and management
 */

import type {
  Route,
  Upstream,
  Consumer,
  GatewayConfig,
  GatewayRequest,
  GatewayResponse,
} from '../types';
import { GatewayEngine } from '../core/GatewayEngine';
import { MetricsService } from './MetricsService';
import { LoggingService } from './LoggingService';
import { CacheService } from './CacheService';

export class GatewayService {
  private engine: GatewayEngine;
  private metrics: MetricsService;
  private logging: LoggingService;
  private cache: CacheService;
  private routes: Map<string, Route> = new Map();
  private upstreams: Map<string, Upstream> = new Map();
  private consumers: Map<string, Consumer> = new Map();

  constructor(config: GatewayConfig) {
    this.engine = new GatewayEngine(config);
    this.metrics = new MetricsService();
    this.logging = new LoggingService();
    this.cache = new CacheService({
      enabled: config.compression?.enabled || false,
      strategy: 'lru',
      ttl: 300000, // 5 minutes
      maxSize: 100 * 1024 * 1024, // 100MB
      varyHeaders: ['accept-encoding'],
      cacheableStatusCodes: [200, 301, 404],
      cacheMethods: ['GET', 'HEAD'],
    });
  }

  /**
   * Handle incoming request
   */
  public async handleRequest(request: GatewayRequest): Promise<GatewayResponse> {
    const startTime = Date.now();

    try {
      // Log incoming request
      this.logging.logRequest(request);

      // Check cache
      const cached = this.cache.get(request);
      if (cached) {
        this.metrics.recordRequest(request, cached, true);
        return cached;
      }

      // Process request through gateway engine
      const response = await this.engine.processRequest(request);

      // Cache response if applicable
      this.cache.set(request, response);

      // Record metrics
      this.metrics.recordRequest(request, response, false);

      // Log response
      this.logging.logResponse(request, response);

      return response;
    } catch (error) {
      // Log error
      this.logging.logError(request, error instanceof Error ? error : new Error('Unknown error'));

      // Record error metrics
      this.metrics.recordError(request);

      throw error;
    }
  }

  /**
   * Register a new route
   */
  public registerRoute(route: Route): void {
    this.routes.set(route.id, route);
    this.engine.registerRoute(route);
    this.logging.log('info', `Route registered: ${route.name}`);
  }

  /**
   * Unregister a route
   */
  public unregisterRoute(routeId: string): void {
    const route = this.routes.get(routeId);
    if (route) {
      this.routes.delete(routeId);
      this.engine.unregisterRoute(routeId);
      this.logging.log('info', `Route unregistered: ${route.name}`);
    }
  }

  /**
   * Update a route
   */
  public updateRoute(routeId: string, updates: Partial<Route>): void {
    const route = this.routes.get(routeId);
    if (route) {
      const updatedRoute = { ...route, ...updates, updatedAt: Date.now() };
      this.routes.set(routeId, updatedRoute);
      this.engine.unregisterRoute(routeId);
      this.engine.registerRoute(updatedRoute);
      this.logging.log('info', `Route updated: ${route.name}`);
    }
  }

  /**
   * Get all routes
   */
  public getRoutes(): Route[] {
    return Array.from(this.routes.values());
  }

  /**
   * Get route by ID
   */
  public getRoute(id: string): Route | undefined {
    return this.routes.get(id);
  }

  /**
   * Register upstream
   */
  public registerUpstream(upstream: Upstream): void {
    this.upstreams.set(upstream.id, upstream);
    this.logging.log('info', `Upstream registered: ${upstream.name}`);
  }

  /**
   * Get all upstreams
   */
  public getUpstreams(): Upstream[] {
    return Array.from(this.upstreams.values());
  }

  /**
   * Register consumer
   */
  public registerConsumer(consumer: Consumer): void {
    this.consumers.set(consumer.id, consumer);
    this.logging.log('info', `Consumer registered: ${consumer.username}`);
  }

  /**
   * Get all consumers
   */
  public getConsumers(): Consumer[] {
    return Array.from(this.consumers.values());
  }

  /**
   * Get consumer by ID
   */
  public getConsumer(id: string): Consumer | undefined {
    return this.consumers.get(id);
  }

  /**
   * Get gateway statistics
   */
  public getStatistics(): {
    routes: number;
    upstreams: number;
    consumers: number;
    metrics: any;
    cache: any;
  } {
    return {
      routes: this.routes.size,
      upstreams: this.upstreams.size,
      consumers: this.consumers.size,
      metrics: this.metrics.getAggregatedMetrics(),
      cache: this.cache.getStatistics(),
    };
  }

  /**
   * Get health status
   */
  public getHealthStatus(): {
    healthy: boolean;
    upstreams: Map<string, boolean>;
    circuits: Map<string, any>;
  } {
    return {
      healthy: true,
      upstreams: this.engine.getHealthStatus(),
      circuits: this.engine.getCircuitBreakerStates(),
    };
  }

  /**
   * Clear cache
   */
  public clearCache(): void {
    this.cache.clear();
    this.logging.log('info', 'Cache cleared');
  }

  /**
   * Get metrics service
   */
  public getMetrics(): MetricsService {
    return this.metrics;
  }

  /**
   * Get logging service
   */
  public getLogging(): LoggingService {
    return this.logging;
  }
}
