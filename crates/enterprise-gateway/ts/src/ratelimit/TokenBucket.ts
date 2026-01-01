/**
 * Enterprise API Gateway - Token Bucket Algorithm
 *
 * Classic token bucket rate limiting with burst support
 */

export interface TokenBucketResult {
  allowed: boolean;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
}

export class TokenBucket {
  private tokens: number;
  private lastRefill: number;

  constructor(
    private readonly capacity: number,
    private readonly refillRate: number, // tokens per second
    private readonly burstSize: number = capacity
  ) {
    this.tokens = capacity;
    this.lastRefill = Date.now();
  }

  /**
   * Consume tokens from the bucket
   */
  public async consume(key: string, tokens: number = 1): Promise<TokenBucketResult> {
    this.refill();

    const allowed = this.tokens >= tokens;
    const resetAt = this.calculateResetTime();

    if (allowed) {
      this.tokens -= tokens;
      return {
        allowed: true,
        remaining: Math.floor(this.tokens),
        resetAt,
      };
    }

    // Calculate retry after duration
    const tokensNeeded = tokens - this.tokens;
    const retryAfter = Math.ceil((tokensNeeded / this.refillRate) * 1000);

    return {
      allowed: false,
      remaining: 0,
      resetAt,
      retryAfter,
    };
  }

  /**
   * Refill tokens based on elapsed time
   */
  private refill(): void {
    const now = Date.now();
    const elapsed = (now - this.lastRefill) / 1000; // seconds
    const tokensToAdd = elapsed * this.refillRate;

    this.tokens = Math.min(this.burstSize, this.tokens + tokensToAdd);
    this.lastRefill = now;
  }

  /**
   * Calculate when the bucket will be full
   */
  private calculateResetTime(): number {
    if (this.tokens >= this.capacity) {
      return Date.now();
    }

    const tokensNeeded = this.capacity - this.tokens;
    const timeToFull = (tokensNeeded / this.refillRate) * 1000; // milliseconds

    return Date.now() + timeToFull;
  }

  /**
   * Get current state
   */
  public getState(): {
    tokens: number;
    capacity: number;
    refillRate: number;
    lastRefill: number;
  } {
    this.refill();

    return {
      tokens: this.tokens,
      capacity: this.capacity,
      refillRate: this.refillRate,
      lastRefill: this.lastRefill,
    };
  }

  /**
   * Reset the bucket
   */
  public reset(): void {
    this.tokens = this.capacity;
    this.lastRefill = Date.now();
  }
}
