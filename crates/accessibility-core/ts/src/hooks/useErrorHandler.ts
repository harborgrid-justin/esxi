/**
 * Error handling React hook
 * @module hooks/useErrorHandler
 */

import { useState, useCallback, useEffect, useRef } from 'react';
import { AccessibilityError } from '../errors/AccessibilityError';
import { ErrorHandler } from '../errors/ErrorHandler';
import { SystemErrorCodes } from '../constants/errorCodes';

/**
 * Error handler hook options
 */
export interface UseErrorHandlerOptions {
  /** Whether to automatically log errors */
  logErrors?: boolean;
  /** Whether to automatically report errors */
  reportErrors?: boolean;
  /** Callback when error occurs */
  onError?: (error: AccessibilityError) => void;
  /** Whether to clear error after a timeout */
  autoClear?: boolean;
  /** Auto-clear timeout in milliseconds */
  autoClearTimeout?: number;
}

/**
 * Error handler hook return type
 */
export interface UseErrorHandlerReturn {
  /** Current error */
  error: AccessibilityError | null;
  /** Whether there is an error */
  hasError: boolean;
  /** Handle an error */
  handleError: (error: unknown) => void;
  /** Clear the current error */
  clearError: () => void;
  /** Wrap a function with error handling */
  wrapAsync: <T extends (...args: any[]) => Promise<any>>(
    fn: T
  ) => (...args: Parameters<T>) => Promise<ReturnType<T>>;
  /** Wrap a synchronous function with error handling */
  wrapSync: <T extends (...args: any[]) => any>(
    fn: T
  ) => (...args: Parameters<T>) => ReturnType<T>;
}

/**
 * Hook for handling errors in React components
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { error, handleError, clearError, wrapAsync } = useErrorHandler({
 *     onError: (err) => console.error('Error occurred:', err),
 *   });
 *
 *   const fetchData = wrapAsync(async () => {
 *     const response = await fetch('/api/data');
 *     if (!response.ok) throw new Error('Failed to fetch');
 *     return response.json();
 *   });
 *
 *   if (error) {
 *     return (
 *       <div>
 *         <p>Error: {error.message}</p>
 *         <button onClick={clearError}>Dismiss</button>
 *       </div>
 *     );
 *   }
 *
 *   return <button onClick={fetchData}>Fetch Data</button>;
 * }
 * ```
 */
export function useErrorHandler(
  options: UseErrorHandlerOptions = {}
): UseErrorHandlerReturn {
  const {
    logErrors = true,
    reportErrors = false,
    onError,
    autoClear = false,
    autoClearTimeout = 5000,
  } = options;

  const [error, setError] = useState<AccessibilityError | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * Handle an error
   */
  const handleError = useCallback(
    (err: unknown) => {
      const accessibilityError =
        err instanceof AccessibilityError
          ? err
          : AccessibilityError.fromUnknown(err, SystemErrorCodes.INTERNAL_ERROR);

      setError(accessibilityError);

      // Log error if enabled
      if (logErrors) {
        console.error('[useErrorHandler]', accessibilityError);
      }

      // Report error if enabled
      if (reportErrors) {
        const errorHandler = ErrorHandler.getInstance();
        errorHandler.handleError(accessibilityError);
      }

      // Call error callback
      if (onError) {
        onError(accessibilityError);
      }

      // Auto-clear error
      if (autoClear) {
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
        }
        timeoutRef.current = setTimeout(() => {
          setError(null);
        }, autoClearTimeout);
      }
    },
    [logErrors, reportErrors, onError, autoClear, autoClearTimeout]
  );

  /**
   * Clear the current error
   */
  const clearError = useCallback(() => {
    setError(null);
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  /**
   * Wrap an async function with error handling
   */
  const wrapAsync = useCallback(
    <T extends (...args: any[]) => Promise<any>>(fn: T) => {
      return async (...args: Parameters<T>): Promise<ReturnType<T>> => {
        try {
          return await fn(...args);
        } catch (err) {
          handleError(err);
          throw err;
        }
      };
    },
    [handleError]
  );

  /**
   * Wrap a synchronous function with error handling
   */
  const wrapSync = useCallback(
    <T extends (...args: any[]) => any>(fn: T) => {
      return (...args: Parameters<T>): ReturnType<T> => {
        try {
          return fn(...args);
        } catch (err) {
          handleError(err);
          throw err;
        }
      };
    },
    [handleError]
  );

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return {
    error,
    hasError: error !== null,
    handleError,
    clearError,
    wrapAsync,
    wrapSync,
  };
}

/**
 * Hook for handling async operations with loading and error states
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { execute, loading, error, data } = useAsyncError(async () => {
 *     return fetch('/api/data').then(res => res.json());
 *   });
 *
 *   return (
 *     <div>
 *       {loading && <p>Loading...</p>}
 *       {error && <p>Error: {error.message}</p>}
 *       {data && <pre>{JSON.stringify(data, null, 2)}</pre>}
 *       <button onClick={execute}>Fetch Data</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function useAsyncError<T, Args extends any[] = []>(
  asyncFn: (...args: Args) => Promise<T>,
  options: UseErrorHandlerOptions = {}
): {
  execute: (...args: Args) => Promise<T | undefined>;
  loading: boolean;
  error: AccessibilityError | null;
  data: T | null;
  reset: () => void;
} {
  const [loading, setLoading] = useState(false);
  const [data, setData] = useState<T | null>(null);
  const { error, handleError, clearError } = useErrorHandler(options);

  const execute = useCallback(
    async (...args: Args): Promise<T | undefined> => {
      setLoading(true);
      clearError();

      try {
        const result = await asyncFn(...args);
        setData(result);
        return result;
      } catch (err) {
        handleError(err);
        return undefined;
      } finally {
        setLoading(false);
      }
    },
    [asyncFn, handleError, clearError]
  );

  const reset = useCallback(() => {
    setLoading(false);
    setData(null);
    clearError();
  }, [clearError]);

  return {
    execute,
    loading,
    error,
    data,
    reset,
  };
}

/**
 * Hook for retrying failed operations
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { execute, retrying, retriesLeft } = useRetry(
 *     async () => {
 *       const res = await fetch('/api/data');
 *       if (!res.ok) throw new Error('Failed');
 *       return res.json();
 *     },
 *     { maxRetries: 3, retryDelay: 1000 }
 *   );
 *
 *   return (
 *     <div>
 *       <button onClick={execute}>Fetch Data</button>
 *       {retrying && <p>Retrying... ({retriesLeft} attempts left)</p>}
 *     </div>
 *   );
 * }
 * ```
 */
export function useRetry<T, Args extends any[] = []>(
  asyncFn: (...args: Args) => Promise<T>,
  options: {
    maxRetries?: number;
    retryDelay?: number;
    onRetry?: (attempt: number) => void;
  } & UseErrorHandlerOptions = {}
): {
  execute: (...args: Args) => Promise<T | undefined>;
  retrying: boolean;
  retriesLeft: number;
  reset: () => void;
} {
  const { maxRetries = 3, retryDelay = 1000, onRetry, ...errorOptions } = options;

  const [retrying, setRetrying] = useState(false);
  const [retriesLeft, setRetriesLeft] = useState(maxRetries);
  const { handleError } = useErrorHandler(errorOptions);

  const execute = useCallback(
    async (...args: Args): Promise<T | undefined> => {
      setRetriesLeft(maxRetries);

      for (let attempt = 0; attempt <= maxRetries; attempt++) {
        try {
          const result = await asyncFn(...args);
          setRetrying(false);
          return result;
        } catch (err) {
          const accessibilityError =
            err instanceof AccessibilityError
              ? err
              : AccessibilityError.fromUnknown(err, SystemErrorCodes.INTERNAL_ERROR);

          // Don't retry if error is not retryable
          if (!accessibilityError.isRetryable() || attempt === maxRetries) {
            handleError(err);
            setRetrying(false);
            return undefined;
          }

          // Set retrying state
          setRetrying(true);
          setRetriesLeft(maxRetries - attempt);

          // Call retry callback
          if (onRetry) {
            onRetry(attempt + 1);
          }

          // Wait before retry
          await new Promise((resolve) => setTimeout(resolve, retryDelay));
        }
      }

      setRetrying(false);
      return undefined;
    },
    [asyncFn, maxRetries, retryDelay, handleError, onRetry]
  );

  const reset = useCallback(() => {
    setRetrying(false);
    setRetriesLeft(maxRetries);
  }, [maxRetries]);

  return {
    execute,
    retrying,
    retriesLeft,
    reset,
  };
}
