/**
 * Global error handler
 * @module errors/ErrorHandler
 */

import { AccessibilityError } from './AccessibilityError';
import { SystemErrorCodes } from '../constants/errorCodes';
import type { ErrorHandlerConfig, Logger } from '../types';

/**
 * Global error handler for the application
 */
export class ErrorHandler {
  private static instance: ErrorHandler | null = null;
  private config: ErrorHandlerConfig;
  private errorListeners: Array<(error: AccessibilityError) => void> = [];

  private constructor(config: Partial<ErrorHandlerConfig> = {}) {
    this.config = {
      logErrors: config.logErrors ?? true,
      reportErrors: config.reportErrors ?? false,
      showUserMessages: config.showUserMessages ?? true,
      logger: config.logger ?? console,
      reportingEndpoint: config.reportingEndpoint,
      context: config.context ?? {},
    };

    this.setupGlobalHandlers();
  }

  /**
   * Get singleton instance
   */
  public static getInstance(config?: Partial<ErrorHandlerConfig>): ErrorHandler {
    if (!ErrorHandler.instance) {
      ErrorHandler.instance = new ErrorHandler(config);
    } else if (config) {
      ErrorHandler.instance.updateConfig(config);
    }
    return ErrorHandler.instance;
  }

  /**
   * Initialize error handler
   */
  public static initialize(config?: Partial<ErrorHandlerConfig>): ErrorHandler {
    return ErrorHandler.getInstance(config);
  }

  /**
   * Update configuration
   */
  public updateConfig(config: Partial<ErrorHandlerConfig>): void {
    this.config = {
      ...this.config,
      ...config,
    };
  }

  /**
   * Setup global error handlers
   */
  private setupGlobalHandlers(): void {
    if (typeof window === 'undefined') {
      return;
    }

    // Handle uncaught errors
    window.addEventListener('error', (event: ErrorEvent) => {
      event.preventDefault();
      const error = AccessibilityError.fromUnknown(
        event.error,
        SystemErrorCodes.INTERNAL_ERROR
      );
      this.handleError(error);
    });

    // Handle unhandled promise rejections
    window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
      event.preventDefault();
      const error = AccessibilityError.fromUnknown(
        event.reason,
        SystemErrorCodes.INTERNAL_ERROR
      );
      this.handleError(error);
    });
  }

  /**
   * Handle an error
   */
  public handleError(error: unknown): void {
    const accessibilityError =
      error instanceof AccessibilityError
        ? error
        : AccessibilityError.fromUnknown(error, SystemErrorCodes.INTERNAL_ERROR);

    // Log error
    if (this.config.logErrors) {
      this.logError(accessibilityError);
    }

    // Report error
    if (this.config.reportErrors) {
      this.reportError(accessibilityError).catch((reportError) => {
        this.config.logger?.error('Failed to report error', reportError);
      });
    }

    // Notify listeners
    this.notifyListeners(accessibilityError);
  }

  /**
   * Log error
   */
  private logError(error: AccessibilityError): void {
    const logger = this.config.logger ?? console;

    logger.error(
      `[${error.category.toUpperCase()}] ${error.code}: ${error.message}`,
      error,
      {
        metadata: error.metadata,
        stack: error.stack,
      }
    );
  }

  /**
   * Report error to external service
   */
  private async reportError(error: AccessibilityError): Promise<void> {
    if (!this.config.reportingEndpoint) {
      return;
    }

    try {
      const payload = {
        ...error.toJSON(),
        context: this.config.context,
        timestamp: new Date().toISOString(),
      };

      const response = await fetch(this.config.reportingEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        throw new Error(`Failed to report error: ${response.statusText}`);
      }
    } catch (reportError) {
      this.config.logger?.error('Error reporting failed', reportError as Error);
    }
  }

  /**
   * Add error listener
   */
  public addListener(listener: (error: AccessibilityError) => void): () => void {
    this.errorListeners.push(listener);

    // Return unsubscribe function
    return () => {
      const index = this.errorListeners.indexOf(listener);
      if (index > -1) {
        this.errorListeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners
   */
  private notifyListeners(error: AccessibilityError): void {
    for (const listener of this.errorListeners) {
      try {
        listener(error);
      } catch (listenerError) {
        this.config.logger?.error('Error in error listener', listenerError as Error);
      }
    }
  }

  /**
   * Clear all listeners
   */
  public clearListeners(): void {
    this.errorListeners = [];
  }

  /**
   * Handle error with recovery
   */
  public async handleErrorWithRecovery(
    error: unknown,
    recovery?: () => Promise<void> | void
  ): Promise<void> {
    this.handleError(error);

    if (recovery) {
      try {
        await recovery();
      } catch (recoveryError) {
        this.handleError(recoveryError);
      }
    }
  }

  /**
   * Wrap function with error handling
   */
  public wrap<T extends (...args: any[]) => any>(
    fn: T,
    onError?: (error: AccessibilityError) => void
  ): T {
    return ((...args: Parameters<T>) => {
      try {
        const result = fn(...args);

        // Handle async functions
        if (result instanceof Promise) {
          return result.catch((error) => {
            const accessibilityError =
              error instanceof AccessibilityError
                ? error
                : AccessibilityError.fromUnknown(error, SystemErrorCodes.INTERNAL_ERROR);

            this.handleError(accessibilityError);
            if (onError) {
              onError(accessibilityError);
            }
            throw accessibilityError;
          });
        }

        return result;
      } catch (error) {
        const accessibilityError =
          error instanceof AccessibilityError
            ? error
            : AccessibilityError.fromUnknown(error, SystemErrorCodes.INTERNAL_ERROR);

        this.handleError(accessibilityError);
        if (onError) {
          onError(accessibilityError);
        }
        throw accessibilityError;
      }
    }) as T;
  }

  /**
   * Create error handler instance with custom config
   */
  public static create(config?: Partial<ErrorHandlerConfig>): ErrorHandler {
    return new ErrorHandler(config);
  }
}

/**
 * Get global error handler instance
 */
export function getErrorHandler(): ErrorHandler {
  return ErrorHandler.getInstance();
}

/**
 * Initialize global error handler
 */
export function initializeErrorHandler(config?: Partial<ErrorHandlerConfig>): ErrorHandler {
  return ErrorHandler.initialize(config);
}
