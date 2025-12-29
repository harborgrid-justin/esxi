/**
 * Render utility with built-in accessibility testing
 * Wraps Testing Library's render with automatic accessibility checks
 */

import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { runAxe } from './axeRunner';
import type { AccessibilityOptions, A11yViolation } from '../index';
import React from 'react';

export interface A11yRenderOptions extends RenderOptions {
  /**
   * Accessibility test options
   */
  a11yOptions?: AccessibilityOptions;

  /**
   * Whether to automatically run accessibility tests after render
   * @default true
   */
  runA11yTests?: boolean;

  /**
   * Whether to throw on accessibility violations
   * @default false
   */
  throwOnViolations?: boolean;

  /**
   * Custom callback for handling violations
   */
  onViolation?: (violations: A11yViolation[]) => void;
}

export interface A11yRenderResult extends RenderResult {
  /**
   * Run accessibility tests on the rendered component
   */
  checkA11y: (options?: AccessibilityOptions) => Promise<A11yViolation[]>;

  /**
   * Get the last accessibility test results
   */
  getA11yResults: () => A11yViolation[] | null;
}

let lastViolations: A11yViolation[] | null = null;

/**
 * Render a React component with accessibility testing capabilities
 *
 * @example
 * ```typescript
 * const { container, checkA11y } = renderWithA11y(<MyComponent />);
 * const violations = await checkA11y();
 * expect(violations).toHaveLength(0);
 * ```
 */
export function renderWithA11y(
  ui: React.ReactElement,
  options: A11yRenderOptions = {}
): A11yRenderResult {
  const {
    a11yOptions = {},
    runA11yTests = true,
    throwOnViolations = false,
    onViolation,
    ...renderOptions
  } = options;

  // Render the component
  const result = render(ui, renderOptions);

  // Function to check accessibility
  const checkA11y = async (
    customOptions?: AccessibilityOptions
  ): Promise<A11yViolation[]> => {
    const mergedOptions = { ...a11yOptions, ...customOptions };

    try {
      const axeResults = await runAxe(result.container, mergedOptions);
      const violations = axeResults.violations as A11yViolation[];

      lastViolations = violations;

      // Call custom violation handler if provided
      if (onViolation && violations.length > 0) {
        onViolation(violations);
      }

      // Throw if configured to do so
      if (throwOnViolations && violations.length > 0) {
        const summary = violations
          .map((v) => `  - [${v.impact}] ${v.id}: ${v.description}`)
          .join('\n');

        throw new Error(
          `Accessibility violations found:\n${summary}\n\nRun with throwOnViolations: false to see full details.`
        );
      }

      return violations;
    } catch (error) {
      if (throwOnViolations) {
        throw error;
      }
      console.error('Accessibility check failed:', error);
      return [];
    }
  };

  // Get last results
  const getA11yResults = () => lastViolations;

  // Automatically run tests if configured
  if (runA11yTests) {
    // Run async but don't block render
    checkA11y().catch((error) => {
      console.error('Auto accessibility check failed:', error);
    });
  }

  return {
    ...result,
    checkA11y,
    getA11yResults,
  };
}

/**
 * Render with strict accessibility checking
 * Automatically throws on any violations
 */
export function renderWithA11yStrict(
  ui: React.ReactElement,
  options: A11yRenderOptions = {}
): A11yRenderResult {
  return renderWithA11y(ui, {
    ...options,
    a11yOptions: {
      wcagLevel: 'AAA',
      ...options.a11yOptions,
    },
    throwOnViolations: true,
  });
}

/**
 * Render with permissive accessibility checking
 * Only checks for critical/serious violations
 */
export function renderWithA11yPermissive(
  ui: React.ReactElement,
  options: A11yRenderOptions = {}
): A11yRenderResult {
  return renderWithA11y(ui, {
    ...options,
    a11yOptions: {
      wcagLevel: 'A',
      ...options.a11yOptions,
    },
    onViolation: (violations) => {
      const criticalOrSerious = violations.filter(
        (v) => v.impact === 'critical' || v.impact === 'serious'
      );

      if (options.onViolation) {
        options.onViolation(criticalOrSerious);
      }

      if (criticalOrSerious.length > 0) {
        console.warn(
          `Found ${criticalOrSerious.length} critical/serious accessibility violations`
        );
      }
    },
  });
}

/**
 * Wrapper for Testing Library's render that includes accessibility provider
 * Useful for apps that need accessibility context
 */
export function renderWithA11yProvider(
  ui: React.ReactElement,
  options: A11yRenderOptions = {}
): A11yRenderResult {
  // You can wrap with custom providers here
  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    return <>{children}</>;
  };

  return renderWithA11y(ui, {
    ...options,
    wrapper: options.wrapper || Wrapper,
  });
}

/**
 * Helper to render multiple components and check them all
 */
export async function renderMultipleWithA11y(
  components: React.ReactElement[],
  options: A11yRenderOptions = {}
): Promise<Array<A11yRenderResult & { violations: A11yViolation[] }>> {
  const results = await Promise.all(
    components.map(async (component) => {
      const result = renderWithA11y(component, {
        ...options,
        runA11yTests: false, // We'll run manually
      });

      const violations = await result.checkA11y();

      return {
        ...result,
        violations,
      };
    })
  );

  return results;
}

/**
 * Create a custom render function with default options
 * Useful for creating project-specific render utilities
 */
export function createA11yRender(defaultOptions: A11yRenderOptions) {
  return (ui: React.ReactElement, options: A11yRenderOptions = {}) => {
    return renderWithA11y(ui, {
      ...defaultOptions,
      ...options,
    });
  };
}
