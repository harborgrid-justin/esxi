/**
 * Enterprise API Gateway - Rate Limiter Engine
 *
 * Main rate limiting orchestrator supporting multiple algorithms
 */

import type { RateLimit, RateLimitResult, GatewayRequest } from '../types';
import { RateLimitError } from '../types';
import { TokenBucket } from './TokenBucket';
import { SlidingWindow } from './SlidingWindow';
import { FixedWindow } from './FixedWindow';
import { AdaptiveLimit } from './AdaptiveLimit';

export class RateLimiter {
  private limiters: Map<string, any> = new Map();

  /**
   * Check if a request is allowed under rate limits
   */
  public async checkLimit(
    rateLimit: RateLimit,
    request: GatewayRequest
  ): Promise<RateLimitResult> {
    const key = this.buildKey(rateLimit, request);
    let limiter = this.limiters.get(key);

    if (!limiter) {
      limiter = this.createLimiter(rateLimit);
      this.limiters.set(key, limiter);
    }

    const result = await limiter.consume(key);

    return {
      allowed: result.allowed,
      limit: rateLimit.limit,
      remaining: result.remaining,
      resetAt: result.resetAt,
      retryAfter: result.retryAfter,
    };
  }

  /**
   * Create appropriate limiter based on algorithm
   */
  private createLimiter(rateLimit: RateLimit): any {
    switch (rateLimit.algorithm) {
      case 'token-bucket':
        return new TokenBucket(
          rateLimit.limit,
          rateLimit.refillRate || rateLimit.limit / (rateLimit.window / 1000),
          rateLimit.burstSize || rateLimit.limit
        );

      case 'sliding-window':
        return new SlidingWindow(rateLimit.limit, rateLimit.window);

      case 'fixed-window':
        return new FixedWindow(rateLimit.limit, rateLimit.window);

      case 'adaptive':
        return new AdaptiveLimit(rateLimit.limit, rateLimit.window);

      default:
        return new TokenBucket(rateLimit.limit, rateLimit.limit, rateLimit.limit);
    }
  }

  /**
   * Build rate limit key based on scope
   */
  private buildKey(rateLimit: RateLimit, request: GatewayRequest): string {
    const parts = [rateLimit.id];

    switch (rateLimit.scope) {
      case 'global':
        parts.push('global');
        break;

      case 'consumer':
        parts.push('consumer', request.consumer?.id || 'anonymous');
        break;

      case 'route':
        parts.push('route', request.path);
        break;

      case 'ip':
        parts.push('ip', request.ip);
        break;
    }

    if (rateLimit.key) {
      parts.push(rateLimit.key);
    }

    return parts.join(':');
  }

  /**
   * Reset rate limit for a specific key
   */
  public async reset(rateLimit: RateLimit, request: GatewayRequest): Promise<void> {
    const key = this.buildKey(rateLimit, request);
    this.limiters.delete(key);
  }

  /**
   * Clear all rate limiters
   */
  public clearAll(): void {
    this.limiters.clear();
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalLimiters: number;
    limiters: Map<string, any>;
  } {
    return {
      totalLimiters: this.limiters.size,
      limiters: new Map(this.limiters),
    };
  }
}
