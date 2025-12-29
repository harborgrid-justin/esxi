/**
 * Error recovery strategies
 * @module errors/ErrorRecovery
 */

import { AccessibilityError } from './AccessibilityError';
import type { RecoveryStrategy, RetryConfig } from '../types';

/**
 * Default retry configuration
 */
const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  initialDelay: 1000,
  maxDelay: 30000,
  backoffMultiplier: 2,
  exponentialBackoff: true,
};

/**
 * Error recovery manager
 */
export class ErrorRecovery {
  private strategies: Map<string, RecoveryStrategy> = new Map();
  private retryConfig: RetryConfig;

  constructor(retryConfig: Partial<RetryConfig> = {}) {
    this.retryConfig = {
      ...DEFAULT_RETRY_CONFIG,
      ...retryConfig,
    };
  }

  /**
   * Register a recovery strategy
   */
  public registerStrategy(errorCode: string, strategy: RecoveryStrategy): void {
    this.strategies.set(errorCode, strategy);
  }

  /**
   * Get recovery strategy for error
   */
  public getStrategy(error: AccessibilityError): RecoveryStrategy | undefined {
    return this.strategies.get(error.code);
  }

  /**
   * Attempt to recover from error
   */
  public async recover(error: AccessibilityError): Promise<boolean> {
    const strategy = this.getStrategy(error);

    if (!strategy) {
      return false;
    }

    try {
      await strategy.execute();
      return true;
    } catch (recoveryError) {
      console.error('Recovery strategy failed:', recoveryError);
      return false;
    }
  }

  /**
   * Retry operation with exponential backoff
   */
  public async retry<T>(
    operation: () => Promise<T>,
    config: Partial<RetryConfig> = {}
  ): Promise<T> {
    const retryConfig = {
      ...this.retryConfig,
      ...config,
    };

    let lastError: Error | undefined;
    let delay = retryConfig.initialDelay;

    for (let attempt = 0; attempt <= retryConfig.maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        // Check if error is retryable
        if (error instanceof AccessibilityError && !error.isRetryable()) {
          throw error;
        }

        // Don't delay on last attempt
        if (attempt < retryConfig.maxRetries) {
          await this.sleep(delay);

          // Calculate next delay with exponential backoff
          if (retryConfig.exponentialBackoff) {
            delay = Math.min(
              delay * retryConfig.backoffMultiplier,
              retryConfig.maxDelay
            );
          }
        }
      }
    }

    throw lastError ?? new Error('Retry failed');
  }

  /**
   * Retry operation with linear backoff
   */
  public async retryLinear<T>(
    operation: () => Promise<T>,
    maxRetries: number = 3,
    delay: number = 1000
  ): Promise<T> {
    return this.retry(operation, {
      maxRetries,
      initialDelay: delay,
      exponentialBackoff: false,
      backoffMultiplier: 1,
    });
  }

  /**
   * Execute operation with fallback
   */
  public async withFallback<T>(
    operation: () => Promise<T>,
    fallback: () => Promise<T> | T
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      console.warn('Operation failed, using fallback:', error);
      return await fallback();
    }
  }

  /**
   * Execute operation with timeout
   */
  public async withTimeout<T>(
    operation: () => Promise<T>,
    timeoutMs: number,
    timeoutMessage?: string
  ): Promise<T> {
    return Promise.race([
      operation(),
      this.createTimeout<T>(timeoutMs, timeoutMessage),
    ]);
  }

  /**
   * Execute multiple operations with circuit breaker pattern
   */
  public createCircuitBreaker<T>(
    operation: () => Promise<T>,
    options: {
      failureThreshold?: number;
      resetTimeout?: number;
      onStateChange?: (state: 'open' | 'closed' | 'half-open') => void;
    } = {}
  ): () => Promise<T> {
    const failureThreshold = options.failureThreshold ?? 5;
    const resetTimeout = options.resetTimeout ?? 60000;

    let failureCount = 0;
    let lastFailureTime = 0;
    let state: 'open' | 'closed' | 'half-open' = 'closed';

    return async () => {
      // Check if circuit should be reset
      if (state === 'open' && Date.now() - lastFailureTime >= resetTimeout) {
        state = 'half-open';
        options.onStateChange?.(state);
      }

      // Reject if circuit is open
      if (state === 'open') {
        throw new Error('Circuit breaker is open');
      }

      try {
        const result = await operation();

        // Reset on success
        if (state === 'half-open') {
          state = 'closed';
          failureCount = 0;
          options.onStateChange?.(state);
        }

        return result;
      } catch (error) {
        failureCount++;
        lastFailureTime = Date.now();

        // Open circuit if threshold reached
        if (failureCount >= failureThreshold) {
          state = 'open';
          options.onStateChange?.(state);
        }

        throw error;
      }
    };
  }

  /**
   * Debounce operation
   */
  public debounce<T extends (...args: any[]) => any>(
    operation: T,
    delayMs: number
  ): (...args: Parameters<T>) => Promise<ReturnType<T>> {
    let timeoutId: NodeJS.Timeout | null = null;

    return (...args: Parameters<T>): Promise<ReturnType<T>> => {
      return new Promise((resolve, reject) => {
        if (timeoutId) {
          clearTimeout(timeoutId);
        }

        timeoutId = setTimeout(async () => {
          try {
            const result = await operation(...args);
            resolve(result);
          } catch (error) {
            reject(error);
          }
        }, delayMs);
      });
    };
  }

  /**
   * Throttle operation
   */
  public throttle<T extends (...args: any[]) => any>(
    operation: T,
    delayMs: number
  ): (...args: Parameters<T>) => Promise<ReturnType<T>> | undefined {
    let lastCall = 0;
    let timeoutId: NodeJS.Timeout | null = null;

    return (...args: Parameters<T>): Promise<ReturnType<T>> | undefined => {
      const now = Date.now();

      if (now - lastCall >= delayMs) {
        lastCall = now;
        return Promise.resolve(operation(...args));
      }

      if (!timeoutId) {
        timeoutId = setTimeout(() => {
          lastCall = Date.now();
          timeoutId = null;
          operation(...args);
        }, delayMs - (now - lastCall));
      }

      return undefined;
    };
  }

  /**
   * Sleep for specified duration
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Create timeout promise
   */
  private createTimeout<T>(timeoutMs: number, message?: string): Promise<T> {
    return new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error(message ?? `Operation timed out after ${timeoutMs}ms`));
      }, timeoutMs);
    });
  }

  /**
   * Batch operations
   */
  public async batch<T, R>(
    items: T[],
    operation: (item: T) => Promise<R>,
    batchSize: number = 10
  ): Promise<R[]> {
    const results: R[] = [];

    for (let i = 0; i < items.length; i += batchSize) {
      const batch = items.slice(i, i + batchSize);
      const batchResults = await Promise.all(batch.map(operation));
      results.push(...batchResults);
    }

    return results;
  }

  /**
   * Execute operations in parallel with concurrency limit
   */
  public async parallel<T, R>(
    items: T[],
    operation: (item: T) => Promise<R>,
    concurrency: number = 5
  ): Promise<R[]> {
    const results: R[] = [];
    const executing: Promise<void>[] = [];

    for (const item of items) {
      const promise = operation(item).then((result) => {
        results.push(result);
        executing.splice(executing.indexOf(promise), 1);
      });

      executing.push(promise);

      if (executing.length >= concurrency) {
        await Promise.race(executing);
      }
    }

    await Promise.all(executing);
    return results;
  }

  /**
   * Update retry configuration
   */
  public updateRetryConfig(config: Partial<RetryConfig>): void {
    this.retryConfig = {
      ...this.retryConfig,
      ...config,
    };
  }
}

/**
 * Create error recovery instance
 */
export function createErrorRecovery(config?: Partial<RetryConfig>): ErrorRecovery {
  return new ErrorRecovery(config);
}

/**
 * Retry operation helper
 */
export async function retry<T>(
  operation: () => Promise<T>,
  config?: Partial<RetryConfig>
): Promise<T> {
  const recovery = new ErrorRecovery(config);
  return recovery.retry(operation);
}

/**
 * Retry with fallback helper
 */
export async function retryWithFallback<T>(
  operation: () => Promise<T>,
  fallback: () => Promise<T> | T,
  config?: Partial<RetryConfig>
): Promise<T> {
  const recovery = new ErrorRecovery(config);
  return recovery.withFallback(() => recovery.retry(operation, config), fallback);
}
