/**
 * Enterprise API Gateway - Adaptive Rate Limiting
 *
 * Dynamic rate limiting that adjusts based on system load and response times
 */

export interface AdaptiveLimitResult {
  allowed: boolean;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
  currentLimit?: number;
}

interface AdaptiveState {
  count: number;
  windowStart: number;
  currentLimit: number;
  avgResponseTime: number;
  errorRate: number;
  successCount: number;
  errorCount: number;
}

export class AdaptiveLimit {
  private states: Map<string, AdaptiveState> = new Map();
  private readonly minLimit: number;
  private readonly maxLimit: number;
  private readonly targetResponseTime = 200; // ms
  private readonly maxErrorRate = 0.1; // 10%

  constructor(
    private readonly baseLimit: number,
    private readonly windowMs: number
  ) {
    this.minLimit = Math.floor(baseLimit * 0.5);
    this.maxLimit = Math.floor(baseLimit * 2);
  }

  /**
   * Consume from adaptive limiter
   */
  public async consume(key: string, responseTime?: number, error?: boolean): Promise<AdaptiveLimitResult> {
    const now = Date.now();
    const windowStart = this.getWindowStart(now);

    // Get or create state
    let state = this.states.get(key);

    // Reset if we're in a new window
    if (!state || state.windowStart !== windowStart) {
      const previousLimit = state?.currentLimit || this.baseLimit;
      const newLimit = this.calculateNewLimit(state, previousLimit);

      state = {
        count: 0,
        windowStart,
        currentLimit: newLimit,
        avgResponseTime: 0,
        errorRate: 0,
        successCount: 0,
        errorCount: 0,
      };
      this.states.set(key, state);
    }

    // Update metrics if provided
    if (responseTime !== undefined) {
      this.updateMetrics(state, responseTime, error || false);
    }

    const allowed = state.count < state.currentLimit;
    const remaining = Math.max(0, state.currentLimit - state.count);
    const resetAt = windowStart + this.windowMs;

    if (allowed) {
      state.count++;
      this.states.set(key, state);

      return {
        allowed: true,
        remaining: remaining - 1,
        resetAt,
        currentLimit: state.currentLimit,
      };
    }

    const retryAfter = resetAt - now;

    return {
      allowed: false,
      remaining: 0,
      resetAt,
      retryAfter,
      currentLimit: state.currentLimit,
    };
  }

  /**
   * Update performance metrics
   */
  private updateMetrics(state: AdaptiveState, responseTime: number, error: boolean): void {
    const totalRequests = state.successCount + state.errorCount;

    // Update average response time
    state.avgResponseTime =
      (state.avgResponseTime * totalRequests + responseTime) / (totalRequests + 1);

    // Update error rate
    if (error) {
      state.errorCount++;
    } else {
      state.successCount++;
    }

    state.errorRate = state.errorCount / (state.successCount + state.errorCount);
  }

  /**
   * Calculate new limit based on performance metrics
   */
  private calculateNewLimit(state: AdaptiveState | undefined, currentLimit: number): number {
    if (!state) {
      return this.baseLimit;
    }

    let newLimit = currentLimit;

    // Decrease limit if response time is too high
    if (state.avgResponseTime > this.targetResponseTime) {
      const ratio = this.targetResponseTime / state.avgResponseTime;
      newLimit = Math.floor(newLimit * ratio);
    }

    // Decrease limit if error rate is too high
    if (state.errorRate > this.maxErrorRate) {
      const ratio = this.maxErrorRate / state.errorRate;
      newLimit = Math.floor(newLimit * ratio);
    }

    // Increase limit if everything is performing well
    if (
      state.avgResponseTime < this.targetResponseTime * 0.8 &&
      state.errorRate < this.maxErrorRate * 0.5
    ) {
      newLimit = Math.floor(newLimit * 1.1);
    }

    // Enforce min/max bounds
    return Math.max(this.minLimit, Math.min(this.maxLimit, newLimit));
  }

  /**
   * Get the start of the current window
   */
  private getWindowStart(timestamp: number): number {
    return Math.floor(timestamp / this.windowMs) * this.windowMs;
  }

  /**
   * Get current state for a key
   */
  public getState(key: string): AdaptiveState | undefined {
    return this.states.get(key);
  }

  /**
   * Reset for a specific key
   */
  public reset(key: string): void {
    this.states.delete(key);
  }

  /**
   * Clear all
   */
  public clearAll(): void {
    this.states.clear();
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalKeys: number;
    avgLimit: number;
    avgResponseTime: number;
    avgErrorRate: number;
  } {
    let totalLimit = 0;
    let totalResponseTime = 0;
    let totalErrorRate = 0;
    let count = 0;

    for (const state of this.states.values()) {
      totalLimit += state.currentLimit;
      totalResponseTime += state.avgResponseTime;
      totalErrorRate += state.errorRate;
      count++;
    }

    return {
      totalKeys: this.states.size,
      avgLimit: count > 0 ? totalLimit / count : this.baseLimit,
      avgResponseTime: count > 0 ? totalResponseTime / count : 0,
      avgErrorRate: count > 0 ? totalErrorRate / count : 0,
    };
  }
}
