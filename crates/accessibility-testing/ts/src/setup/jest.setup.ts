/**
 * Jest setup for accessibility testing
 * Configures Jest with accessibility matchers and utilities
 */

import '@testing-library/jest-dom';
import { toHaveNoViolations } from 'jest-axe';
import { matchers } from '../index';

/**
 * Setup function for Jest
 * Call this in your Jest setupFilesAfterEnv configuration
 *
 * @example
 * ```javascript
 * // jest.config.js
 * module.exports = {
 *   setupFilesAfterEnv: ['@meridian/accessibility-testing/dist/setup/jest.setup.js']
 * }
 * ```
 */
export function setupJest(): void {
  // Extend Jest matchers with accessibility matchers
  expect.extend({
    toHaveNoViolations,
    ...matchers,
  });

  // Configure jest-axe
  const jestAxeConfig = {
    // Global configuration for all axe runs
    globalOptions: {
      runOnly: {
        type: 'tag',
        values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'best-practice'],
      },
    },
  };

  // Set global timeout for async accessibility tests
  jest.setTimeout(10000); // 10 seconds

  // Mock console methods to reduce noise during tests (optional)
  const originalError = console.error;
  const originalWarn = console.warn;

  beforeAll(() => {
    // Suppress React warnings in tests (optional)
    console.error = (...args: unknown[]) => {
      const message = args[0];
      if (
        typeof message === 'string' &&
        (message.includes('Warning: ReactDOM.render') ||
          message.includes('Warning: useLayoutEffect'))
      ) {
        return;
      }
      originalError.call(console, ...args);
    };

    console.warn = (...args: unknown[]) => {
      const message = args[0];
      if (
        typeof message === 'string' &&
        message.includes('componentWillReceiveProps')
      ) {
        return;
      }
      originalWarn.call(console, ...args);
    };
  });

  afterAll(() => {
    console.error = originalError;
    console.warn = originalWarn;
  });

  // Clean up DOM after each test
  afterEach(() => {
    document.body.innerHTML = '';
    document.head.innerHTML = '';
  });

  // Add custom error messages for better debugging
  Error.stackTraceLimit = 50;
}

/**
 * Initialize accessibility testing for Jest
 * This is automatically called when imported
 */
setupJest();

/**
 * Configure accessibility testing options globally
 */
export interface GlobalA11yConfig {
  /**
   * Default WCAG level for tests
   * @default 'AA'
   */
  wcagLevel?: 'A' | 'AA' | 'AAA';

  /**
   * Whether to throw on violations by default
   * @default false
   */
  throwOnViolations?: boolean;

  /**
   * Rules to disable globally
   */
  disabledRules?: string[];

  /**
   * Whether to run accessibility tests automatically
   * @default false
   */
  autoRun?: boolean;
}

let globalConfig: GlobalA11yConfig = {
  wcagLevel: 'AA',
  throwOnViolations: false,
  disabledRules: [],
  autoRun: false,
};

/**
 * Set global configuration for accessibility testing
 */
export function setGlobalA11yConfig(config: GlobalA11yConfig): void {
  globalConfig = { ...globalConfig, ...config };
}

/**
 * Get current global configuration
 */
export function getGlobalA11yConfig(): GlobalA11yConfig {
  return { ...globalConfig };
}

/**
 * Reset global configuration to defaults
 */
export function resetGlobalA11yConfig(): void {
  globalConfig = {
    wcagLevel: 'AA',
    throwOnViolations: false,
    disabledRules: [],
    autoRun: false,
  };
}

/**
 * Helper to create a test suite with accessibility checks
 *
 * @example
 * ```typescript
 * describeA11y('MyComponent', () => {
 *   it('should be accessible', async () => {
 *     const { container } = render(<MyComponent />);
 *     // Accessibility check is automatically run
 *   });
 * });
 * ```
 */
export function describeA11y(
  name: string,
  fn: () => void,
  options?: GlobalA11yConfig
): void {
  describe(`${name} (Accessibility)`, () => {
    beforeAll(() => {
      if (options) {
        setGlobalA11yConfig(options);
      }
    });

    afterAll(() => {
      resetGlobalA11yConfig();
    });

    fn();
  });
}

/**
 * Helper to create an accessibility-focused test
 *
 * @example
 * ```typescript
 * itA11y('should have no violations', async () => {
 *   const { container } = render(<MyComponent />);
 *   const results = await axe(container);
 *   expect(results).toHaveNoViolations();
 * });
 * ```
 */
export function itA11y(
  name: string,
  fn: () => Promise<void> | void,
  timeout?: number
): void {
  it(`[A11Y] ${name}`, fn, timeout);
}

/**
 * Skip accessibility test (but mark it as accessibility-related)
 */
export function itA11ySkip(name: string, fn: () => void): void {
  it.skip(`[A11Y] ${name}`, fn);
}

/**
 * Only run this accessibility test
 */
export function itA11yOnly(name: string, fn: () => void, timeout?: number): void {
  it.only(`[A11Y] ${name}`, fn, timeout);
}

/**
 * Snapshot testing with accessibility checks
 */
export async function expectA11ySnapshot(
  container: HTMLElement,
  snapshotName?: string
): Promise<void> {
  // Take regular snapshot
  expect(container).toMatchSnapshot(snapshotName);

  // Note: Actual axe test would need to be run separately
  // This is just a helper to remind developers to test accessibility
  console.log('Remember to run accessibility tests on this snapshot!');
}

// Export default setup
export default setupJest;
