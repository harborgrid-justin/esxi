/**
 * Enterprise API Gateway - Fixed Window Algorithm
 *
 * Simple and efficient fixed window rate limiting
 */

export interface FixedWindowResult {
  allowed: boolean;
  remaining: number;
  resetAt: number;
  retryAfter?: number;
}

interface WindowState {
  count: number;
  windowStart: number;
}

export class FixedWindow {
  private windows: Map<string, WindowState> = new Map();

  constructor(
    private readonly limit: number,
    private readonly windowMs: number
  ) {}

  /**
   * Consume from the fixed window
   */
  public async consume(key: string): Promise<FixedWindowResult> {
    const now = Date.now();
    const windowStart = this.getWindowStart(now);

    // Get or create window state
    let state = this.windows.get(key);

    // Reset if we're in a new window
    if (!state || state.windowStart !== windowStart) {
      state = {
        count: 0,
        windowStart,
      };
      this.windows.set(key, state);
    }

    const allowed = state.count < this.limit;
    const remaining = Math.max(0, this.limit - state.count);
    const resetAt = windowStart + this.windowMs;

    if (allowed) {
      state.count++;
      this.windows.set(key, state);

      return {
        allowed: true,
        remaining: remaining - 1,
        resetAt,
      };
    }

    const retryAfter = resetAt - now;

    return {
      allowed: false,
      remaining: 0,
      resetAt,
      retryAfter,
    };
  }

  /**
   * Get the start of the current window
   */
  private getWindowStart(timestamp: number): number {
    return Math.floor(timestamp / this.windowMs) * this.windowMs;
  }

  /**
   * Clean up old windows
   */
  public cleanup(): void {
    const now = Date.now();
    const currentWindowStart = this.getWindowStart(now);

    for (const [key, state] of this.windows.entries()) {
      if (state.windowStart < currentWindowStart) {
        this.windows.delete(key);
      }
    }
  }

  /**
   * Get current state for a key
   */
  public getState(key: string): {
    count: number;
    limit: number;
    windowStart: number;
    windowMs: number;
  } {
    const now = Date.now();
    const windowStart = this.getWindowStart(now);
    const state = this.windows.get(key);

    if (!state || state.windowStart !== windowStart) {
      return {
        count: 0,
        limit: this.limit,
        windowStart,
        windowMs: this.windowMs,
      };
    }

    return {
      count: state.count,
      limit: this.limit,
      windowStart: state.windowStart,
      windowMs: this.windowMs,
    };
  }

  /**
   * Reset for a specific key
   */
  public reset(key: string): void {
    this.windows.delete(key);
  }

  /**
   * Clear all
   */
  public clearAll(): void {
    this.windows.clear();
  }

  /**
   * Get statistics
   */
  public getStatistics(): {
    totalKeys: number;
    activeWindows: number;
  } {
    const now = Date.now();
    const currentWindowStart = this.getWindowStart(now);
    let activeWindows = 0;

    for (const state of this.windows.values()) {
      if (state.windowStart === currentWindowStart) {
        activeWindows++;
      }
    }

    return {
      totalKeys: this.windows.size,
      activeWindows,
    };
  }
}
