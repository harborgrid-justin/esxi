/**
 * React Error Boundary component
 * @module errors/ErrorBoundary
 */

import React, { Component, ReactNode, ErrorInfo } from 'react';
import { AccessibilityError } from './AccessibilityError';
import { SystemErrorCodes } from '../constants/errorCodes';

/**
 * Props for ErrorBoundary component
 */
export interface ErrorBoundaryProps {
  /** Child components to wrap */
  children: ReactNode;
  /** Fallback UI to show when error occurs */
  fallback?: ReactNode | ((error: Error, reset: () => void) => ReactNode);
  /** Callback when error is caught */
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  /** Callback to reset error state */
  onReset?: () => void;
  /** Whether to log errors to console */
  logErrors?: boolean;
}

/**
 * State for ErrorBoundary component
 */
interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * React Error Boundary for catching and handling errors
 *
 * @example
 * ```tsx
 * <ErrorBoundary
 *   fallback={(error, reset) => (
 *     <div>
 *       <h1>Something went wrong</h1>
 *       <p>{error.message}</p>
 *       <button onClick={reset}>Try again</button>
 *     </div>
 *   )}
 *   onError={(error, errorInfo) => {
 *     console.error('Error caught:', error, errorInfo);
 *   }}
 * >
 *   <YourApp />
 * </ErrorBoundary>
 * ```
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
    };
  }

  /**
   * Update state when error is caught
   */
  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error,
    };
  }

  /**
   * Handle error caught by boundary
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    const { onError, logErrors = true } = this.props;

    // Convert to AccessibilityError if not already
    const accessibilityError =
      error instanceof AccessibilityError
        ? error
        : AccessibilityError.fromUnknown(error, SystemErrorCodes.INTERNAL_ERROR);

    // Log error if enabled
    if (logErrors) {
      console.error('Error caught by ErrorBoundary:', accessibilityError);
      console.error('Component stack:', errorInfo.componentStack);
    }

    // Call error callback
    if (onError) {
      onError(accessibilityError, errorInfo);
    }
  }

  /**
   * Reset error state
   */
  private resetError = (): void => {
    const { onReset } = this.props;
    this.setState({
      hasError: false,
      error: null,
    });
    if (onReset) {
      onReset();
    }
  };

  render(): ReactNode {
    const { hasError, error } = this.state;
    const { children, fallback } = this.props;

    if (hasError && error) {
      // Use custom fallback if provided
      if (fallback) {
        if (typeof fallback === 'function') {
          return fallback(error, this.resetError);
        }
        return fallback;
      }

      // Default fallback UI
      return (
        <div
          role="alert"
          style={{
            padding: '20px',
            margin: '20px',
            border: '1px solid #DC2626',
            borderRadius: '8px',
            backgroundColor: '#FEE2E2',
            color: '#991B1B',
          }}
        >
          <h2 style={{ margin: '0 0 10px 0', fontSize: '18px', fontWeight: 'bold' }}>
            Something went wrong
          </h2>
          <p style={{ margin: '0 0 15px 0' }}>{error.message}</p>
          <button
            onClick={this.resetError}
            style={{
              padding: '8px 16px',
              backgroundColor: '#DC2626',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '14px',
            }}
          >
            Try again
          </button>
        </div>
      );
    }

    return children;
  }
}

/**
 * Higher-order component to wrap a component with ErrorBoundary
 *
 * @example
 * ```tsx
 * const SafeComponent = withErrorBoundary(MyComponent, {
 *   fallback: <div>Error loading component</div>,
 *   onError: (error) => console.error(error),
 * });
 * ```
 */
export function withErrorBoundary<P extends object>(
  Component: React.ComponentType<P>,
  errorBoundaryProps?: Omit<ErrorBoundaryProps, 'children'>
): React.ComponentType<P> {
  const WrappedComponent = (props: P) => (
    <ErrorBoundary {...errorBoundaryProps}>
      <Component {...props} />
    </ErrorBoundary>
  );

  WrappedComponent.displayName = `withErrorBoundary(${Component.displayName || Component.name || 'Component'})`;

  return WrappedComponent;
}
