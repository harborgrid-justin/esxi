/**
 * Enterprise API Gateway - Sliding Window Algorithm
 *
 * Sliding window rate limiting with precise tracking
 */

export interface SlidingWindowResult {
  allowed: boolean;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
}

interface RequestRecord {
  timestamp: number;
}

export class SlidingWindow {
  private requests: Map<string, RequestRecord[]> = new Map();

  constructor(
    private readonly limit: number,
    private readonly windowMs: number
  ) {}

  /**
   * Consume from the sliding window
   */
  public async consume(key: string): Promise<SlidingWindowResult> {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    // Get or create request records for this key
    let records = this.requests.get(key) || [];

    // Remove expired requests (outside the window)
    records = records.filter((r) => r.timestamp > windowStart);

    // Check if we're under the limit
    const allowed = records.length < this.limit;
    const remaining = Math.max(0, this.limit - records.length);

    if (allowed) {
      records.push({ timestamp: now });
      this.requests.set(key, records);

      return {
        allowed: true,
        remaining: remaining - 1,
        resetAt: now + this.windowMs,
      };
    }

    // Calculate retry after based on oldest request
    const oldestRequest = records[0];
    const retryAfter = oldestRequest
      ? oldestRequest.timestamp + this.windowMs - now
      : this.windowMs;

    return {
      allowed: false,
      remaining: 0,
      resetAt: now + this.windowMs,
      retryAfter,
    };
  }

  /**
   * Clean up expired records
   */
  public cleanup(): void {
    const now = Date.now();
    const windowStart = now - this.windowMs;

    for (const [key, records] of this.requests.entries()) {
      const filtered = records.filter((r) => r.timestamp > windowStart);

      if (filtered.length === 0) {
        this.requests.delete(key);
      } else {
        this.requests.set(key, filtered);
      }
    }
  }

  /**
   * Get current state for a key
   */
  public getState(key: string): {
    requestCount: number;
    limit: number;
    windowMs: number;
    oldestRequest?: number;
  } {
    const now = Date.now();
    const windowStart = now - this.windowMs;
    const records = (this.requests.get(key) || []).filter((r) => r.timestamp > windowStart);

    return {
      requestCount: records.length,
      limit: this.limit,
      windowMs: this.windowMs,
      oldestRequest: records[0]?.timestamp,
    };
  }

  /**
   * Reset for a specific key
   */
  public reset(key: string): void {
    this.requests.delete(key);
  }

  /**
   * Clear all
   */
  public clearAll(): void {
    this.requests.clear();
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalKeys: number;
    totalRequests: number;
  } {
    let totalRequests = 0;

    for (const records of this.requests.values()) {
      totalRequests += records.length;
    }

    return {
      totalKeys: this.requests.size,
      totalRequests,
    };
  }
}
