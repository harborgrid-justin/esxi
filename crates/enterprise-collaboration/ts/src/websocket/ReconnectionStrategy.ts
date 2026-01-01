/**
 * Reconnection Strategy
 * Implements exponential backoff and connection retry logic
 */

export interface ReconnectionConfig {
  maxAttempts: number;
  baseInterval: number;
  backoff: boolean;
  maxInterval?: number;
  jitter?: boolean;
}

export class ReconnectionStrategy {
  private attempts: number = 0;
  private config: Required<ReconnectionConfig>;

  constructor(config: ReconnectionConfig) {
    this.config = {
      maxAttempts: config.maxAttempts,
      baseInterval: config.baseInterval,
      backoff: config.backoff,
      maxInterval: config.maxInterval ?? 30000,
      jitter: config.jitter ?? true,
    };
  }

  /**
   * Check if should attempt reconnection
   */
  async shouldReconnect(): Promise<boolean> {
    if (this.attempts >= this.config.maxAttempts) {
      return false;
    }

    this.attempts++;
    return true;
  }

  /**
   * Get next reconnection delay
   */
  getNextDelay(): number {
    let delay = this.config.baseInterval;

    if (this.config.backoff) {
      // Exponential backoff: baseInterval * 2^(attempts - 1)
      delay = this.config.baseInterval * Math.pow(2, this.attempts - 1);

      // Cap at max interval
      delay = Math.min(delay, this.config.maxInterval);
    }

    // Add jitter to prevent thundering herd
    if (this.config.jitter) {
      const jitter = Math.random() * 0.3 * delay; // ±30% jitter
      delay = delay + (Math.random() > 0.5 ? jitter : -jitter);
    }

    return Math.max(0, delay);
  }

  /**
   * Reset reconnection state
   */
  reset(): void {
    this.attempts = 0;
  }

  /**
   * Get current attempt count
   */
  getAttempts(): number {
    return this.attempts;
  }

  /**
   * Get remaining attempts
   */
  getRemainingAttempts(): number {
    return Math.max(0, this.config.maxAttempts - this.attempts);
  }

  /**
   * Check if max attempts reached
   */
  isMaxAttemptsReached(): boolean {
    return this.attempts >= this.config.maxAttempts;
  }

  /**
   * Set max attempts
   */
  setMaxAttempts(max: number): void {
    this.config.maxAttempts = max;
  }

  /**
   * Calculate all delays up to max attempts
   */
  calculateDelays(): number[] {
    const delays: number[] = [];
    const originalAttempts = this.attempts;

    this.reset();

    for (let i = 0; i < this.config.maxAttempts; i++) {
      this.attempts = i + 1;
      delays.push(this.getNextDelay());
    }

    this.attempts = originalAttempts;

    return delays;
  }

  /**
   * Get total time for all reconnection attempts
   */
  getTotalReconnectionTime(): number {
    return this.calculateDelays().reduce((sum, delay) => sum + delay, 0);
  }
}
