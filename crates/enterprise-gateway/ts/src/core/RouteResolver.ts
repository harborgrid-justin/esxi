/**
 * Enterprise API Gateway - Route Resolver
 *
 * Advanced route matching with multiple strategies
 */

import type { Route, GatewayRequest, HTTPMethod } from '../types';

export class RouteResolver {
  private routes: Map<string, Route> = new Map();
  private exactMatches: Map<string, Route> = new Map();
  private prefixMatches: Route[] = [];
  private regexMatches: Route[] = [];

  /**
   * Add a route to the resolver
   */
  public addRoute(route: Route): void {
    this.routes.set(route.id, route);

    // Index routes by match type for faster lookup
    if (route.matchType === 'exact') {
      route.paths.forEach((path) => {
        const key = this.makeRouteKey(route.methods, path);
        this.exactMatches.set(key, route);
      });
    } else if (route.matchType === 'prefix') {
      this.prefixMatches.push(route);
      // Sort by path length (longest first) for better matching
      this.prefixMatches.sort((a, b) => {
        const maxLenA = Math.max(...a.paths.map((p) => p.length));
        const maxLenB = Math.max(...b.paths.map((p) => p.length));
        return maxLenB - maxLenA;
      });
    } else if (route.matchType === 'regex') {
      this.regexMatches.push(route);
    }
  }

  /**
   * Remove a route from the resolver
   */
  public removeRoute(routeId: string): void {
    const route = this.routes.get(routeId);
    if (!route) return;

    this.routes.delete(routeId);

    if (route.matchType === 'exact') {
      route.paths.forEach((path) => {
        route.methods.forEach((method) => {
          const key = this.makeRouteKey([method], path);
          this.exactMatches.delete(key);
        });
      });
    } else if (route.matchType === 'prefix') {
      this.prefixMatches = this.prefixMatches.filter((r) => r.id !== routeId);
    } else if (route.matchType === 'regex') {
      this.regexMatches = this.regexMatches.filter((r) => r.id !== routeId);
    }
  }

  /**
   * Resolve a request to a matching route
   */
  public resolve(request: GatewayRequest): Route | null {
    // Try exact match first (fastest)
    const exactMatch = this.findExactMatch(request.method, request.path);
    if (exactMatch) {
      return exactMatch;
    }

    // Try prefix matches
    const prefixMatch = this.findPrefixMatch(request.method, request.path);
    if (prefixMatch) {
      return prefixMatch;
    }

    // Try regex matches (slowest)
    const regexMatch = this.findRegexMatch(request.method, request.path);
    if (regexMatch) {
      return regexMatch;
    }

    return null;
  }

  /**
   * Find exact match
   */
  private findExactMatch(method: HTTPMethod, path: string): Route | null {
    const key = this.makeRouteKey([method], path);
    return this.exactMatches.get(key) || null;
  }

  /**
   * Find prefix match
   */
  private findPrefixMatch(method: HTTPMethod, path: string): Route | null {
    for (const route of this.prefixMatches) {
      if (!route.methods.includes(method)) {
        continue;
      }

      for (const routePath of route.paths) {
        if (path.startsWith(routePath)) {
          return route;
        }

        // Handle trailing slashes
        const normalizedPath = routePath.endsWith('/') ? routePath : routePath + '/';
        const normalizedRequestPath = path.endsWith('/') ? path : path + '/';
        if (normalizedRequestPath.startsWith(normalizedPath)) {
          return route;
        }
      }
    }

    return null;
  }

  /**
   * Find regex match
   */
  private findRegexMatch(method: HTTPMethod, path: string): Route | null {
    for (const route of this.regexMatches) {
      if (!route.methods.includes(method)) {
        continue;
      }

      for (const pattern of route.paths) {
        try {
          const regex = new RegExp(pattern);
          if (regex.test(path)) {
            return route;
          }
        } catch (error) {
          console.error(`Invalid regex pattern: ${pattern}`, error);
        }
      }
    }

    return null;
  }

  /**
   * Create a unique key for route lookup
   */
  private makeRouteKey(methods: HTTPMethod[], path: string): string {
    return `${methods.join(',').toLowerCase()}:${path}`;
  }

  /**
   * Extract path parameters from a path pattern
   */
  public extractPathParams(pattern: string, path: string): Record<string, string> {
    const params: Record<string, string> = {};

    // Handle simple path parameters like /users/:id
    const patternParts = pattern.split('/');
    const pathParts = path.split('/');

    if (patternParts.length !== pathParts.length) {
      return params;
    }

    for (let i = 0; i < patternParts.length; i++) {
      const patternPart = patternParts[i];
      const pathPart = pathParts[i];

      if (patternPart && patternPart.startsWith(':')) {
        const paramName = patternPart.slice(1);
        if (pathPart) {
          params[paramName] = decodeURIComponent(pathPart);
        }
      }
    }

    return params;
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
   * Clear all routes
   */
  public clear(): void {
    this.routes.clear();
    this.exactMatches.clear();
    this.prefixMatches = [];
    this.regexMatches = [];
  }

  /**
   * Get route statistics
   */
  public getStatistics(): {
    total: number;
    exact: number;
    prefix: number;
    regex: number;
  } {
    return {
      total: this.routes.size,
      exact: this.exactMatches.size,
      prefix: this.prefixMatches.length,
      regex: this.regexMatches.length,
    };
  }
}
