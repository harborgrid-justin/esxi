/**
 * Enterprise API Gateway - Gateway Engine
 *
 * Core request routing and processing engine
 */

import type {
  GatewayRequest,
  GatewayResponse,
  Route,
  Plugin,
  PluginContext,
  Upstream,
  GatewayConfig,
  MiddlewareHandler,
} from '../types';
import { GatewayError } from '../types';
import { RouteResolver } from './RouteResolver';
import { LoadBalancer } from './LoadBalancer';
import { CircuitBreaker } from './CircuitBreaker';
import { HealthChecker } from './HealthChecker';

export class GatewayEngine {
  private routes: Map<string, Route> = new Map();
  private plugins: Map<string, Plugin[]> = new Map();
  private middleware: MiddlewareHandler[] = [];
  private routeResolver: RouteResolver;
  private loadBalancer: LoadBalancer;
  private circuitBreaker: CircuitBreaker;
  private healthChecker: HealthChecker;

  constructor(private config: GatewayConfig) {
    this.routeResolver = new RouteResolver();
    this.loadBalancer = new LoadBalancer();
    this.circuitBreaker = new CircuitBreaker({
      failureThreshold: 5,
      successThreshold: 2,
      timeout: 60000,
      monitoringPeriod: 10000,
      volumeThreshold: 10,
    });
    this.healthChecker = new HealthChecker();
  }

  /**
   * Register a route with the gateway
   */
  public registerRoute(route: Route): void {
    this.routes.set(route.id, route);
    this.routeResolver.addRoute(route);

    // Register plugins for this route
    if (route.plugins && route.plugins.length > 0) {
      this.plugins.set(route.id, route.plugins);
    }

    // Start health checking for upstream targets
    if (route.upstream.healthChecks) {
      this.healthChecker.addUpstream(route.upstream);
    }
  }

  /**
   * Unregister a route
   */
  public unregisterRoute(routeId: string): void {
    const route = this.routes.get(routeId);
    if (route) {
      this.routes.delete(routeId);
      this.routeResolver.removeRoute(routeId);
      this.plugins.delete(routeId);
      this.healthChecker.removeUpstream(route.upstream.id);
    }
  }

  /**
   * Add global middleware
   */
  public use(handler: MiddlewareHandler): void {
    this.middleware.push(handler);
  }

  /**
   * Process an incoming request
   */
  public async processRequest(request: GatewayRequest): Promise<GatewayResponse> {
    const startTime = Date.now();

    try {
      // Execute middleware chain
      let middlewareResponse: GatewayResponse | undefined;
      for (const handler of this.middleware) {
        const result = await handler(request, middlewareResponse);
        if (result) {
          middlewareResponse = result;
          break; // Middleware returned a response, short-circuit
        }
      }

      if (middlewareResponse) {
        return middlewareResponse;
      }

      // Resolve route
      const route = this.routeResolver.resolve(request);
      if (!route) {
        throw new GatewayError('No matching route found', 404, 'ROUTE_NOT_FOUND');
      }

      if (!route.enabled) {
        throw new GatewayError('Route is disabled', 503, 'ROUTE_DISABLED');
      }

      // Execute pre-route plugins
      const context: PluginContext = {
        request,
        route,
        consumer: request.consumer,
        state: new Map(),
      };

      const preRouteResponse = await this.executePlugins(route, 'pre-route', context);
      if (preRouteResponse) {
        return preRouteResponse;
      }

      // Execute route plugins
      const routeResponse = await this.executePlugins(route, 'route', context);
      if (routeResponse) {
        context.response = routeResponse;
      } else {
        // Forward request to upstream
        context.response = await this.forwardToUpstream(request, route.upstream);
      }

      // Execute post-route plugins
      const postRouteResponse = await this.executePlugins(route, 'post-route', context);
      if (postRouteResponse) {
        context.response = postRouteResponse;
      }

      const duration = Date.now() - startTime;
      return {
        ...context.response!,
        duration,
      };
    } catch (error) {
      // Execute error plugins
      const errorContext: PluginContext = {
        request,
        state: new Map(),
      };

      try {
        const route = this.routeResolver.resolve(request);
        if (route) {
          const errorResponse = await this.executePlugins(route, 'error', errorContext);
          if (errorResponse) {
            return errorResponse;
          }
        }
      } catch {
        // Ignore plugin errors during error handling
      }

      // Default error response
      const duration = Date.now() - startTime;
      if (error instanceof GatewayError) {
        return {
          statusCode: error.statusCode,
          headers: { 'Content-Type': 'application/json' },
          body: {
            error: error.message,
            code: error.code,
            metadata: error.metadata,
          },
          duration,
        };
      }

      return {
        statusCode: 500,
        headers: { 'Content-Type': 'application/json' },
        body: {
          error: 'Internal server error',
          message: error instanceof Error ? error.message : 'Unknown error',
        },
        duration,
      };
    }
  }

  /**
   * Execute plugins for a specific phase
   */
  private async executePlugins(
    route: Route,
    phase: Plugin['phase'],
    context: PluginContext
  ): Promise<GatewayResponse | void> {
    const routePlugins = this.plugins.get(route.id) || [];
    const phasePlugins = routePlugins
      .filter((p) => p.phase === phase && p.enabled)
      .sort((a, b) => b.priority - a.priority);

    for (const plugin of phasePlugins) {
      // Plugin execution would be handled by a plugin manager
      // For now, we'll skip actual plugin execution
      // In a real implementation, you'd have a plugin registry and handlers
    }
  }

  /**
   * Forward request to upstream backend
   */
  private async forwardToUpstream(
    request: GatewayRequest,
    upstream: Upstream
  ): Promise<GatewayResponse> {
    // Check circuit breaker
    if (!this.circuitBreaker.canExecute(upstream.id)) {
      throw new GatewayError('Circuit breaker is open', 503, 'CIRCUIT_BREAKER_OPEN');
    }

    const startTime = Date.now();
    let attempts = 0;
    const maxAttempts = upstream.retries + 1;

    while (attempts < maxAttempts) {
      attempts++;

      try {
        // Select target using load balancer
        const target = this.loadBalancer.selectTarget(upstream, request);
        if (!target) {
          throw new GatewayError('No healthy targets available', 503, 'NO_HEALTHY_TARGETS');
        }

        // Make request to upstream (simplified - in production use axios/fetch)
        const response = await this.makeUpstreamRequest(target.url, request, upstream);

        // Record success
        this.circuitBreaker.recordSuccess(upstream.id);

        return response;
      } catch (error) {
        // Record failure
        this.circuitBreaker.recordFailure(upstream.id);

        if (attempts >= maxAttempts) {
          throw new GatewayError(
            `Upstream request failed after ${attempts} attempts`,
            502,
            'UPSTREAM_FAILED',
            { error: error instanceof Error ? error.message : 'Unknown error' }
          );
        }

        // Wait before retry (exponential backoff)
        await new Promise((resolve) => setTimeout(resolve, Math.pow(2, attempts - 1) * 100));
      }
    }

    throw new GatewayError('Upstream request failed', 502, 'UPSTREAM_FAILED');
  }

  /**
   * Make actual HTTP request to upstream
   */
  private async makeUpstreamRequest(
    targetUrl: string,
    request: GatewayRequest,
    upstream: Upstream
  ): Promise<GatewayResponse> {
    // This is a simplified version - in production, use axios or node-fetch
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve({
          statusCode: 200,
          headers: { 'Content-Type': 'application/json' },
          body: { success: true, data: 'Mock response from upstream' },
          duration: 0,
          upstream: targetUrl,
        });
      }, 50);
    });
  }

  /**
   * Get all registered routes
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
   * Get health status of all upstreams
   */
  public getHealthStatus(): Map<string, boolean> {
    return this.healthChecker.getHealthStatus();
  }

  /**
   * Get circuit breaker states
   */
  public getCircuitBreakerStates(): Map<string, any> {
    return this.circuitBreaker.getStates();
  }
}
