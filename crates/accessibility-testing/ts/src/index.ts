/**
 * @meridian/accessibility-testing
 * Enterprise-grade accessibility testing utilities for React applications
 *
 * @module accessibility-testing
 * @version 0.3.0
 */

// Custom matchers
import { toBeAccessible } from './matchers/toBeAccessible';
import { toHaveNoViolations } from './matchers/toHaveNoViolations';
import { toHaveValidARIA } from './matchers/toHaveValidARIA';

// Utilities
import { renderWithA11y } from './utils/renderWithA11y';
import { runAxe } from './utils/axeRunner';
import * as testUtils from './utils/testUtils';

// Fixtures
export * from './fixtures/accessibleComponents';
export * from './fixtures/inaccessibleComponents';

// Export matchers
export const matchers = {
  toBeAccessible,
  toHaveNoViolations,
  toHaveValidARIA,
};

// Export utilities
export { renderWithA11y, runAxe, testUtils };

// Setup functions for Jest
export { setupJest } from './setup/jest.setup';

// Setup functions for Vitest
export { setupVitest } from './setup/vitest.setup';

// Type augmentation for Jest matchers
declare global {
  namespace jest {
    interface Matchers<R> {
      toBeAccessible(options?: AccessibilityOptions): Promise<R>;
      toHaveNoViolations(): Promise<R>;
      toHaveValidARIA(): R;
    }
  }
}

// Type augmentation for Vitest matchers
declare module 'vitest' {
  interface Assertion<T = any> {
    toBeAccessible(options?: AccessibilityOptions): Promise<void>;
    toHaveNoViolations(): Promise<void>;
    toHaveValidARIA(): void;
  }
}

export interface AccessibilityOptions {
  /**
   * WCAG level to test against
   * @default 'AA'
   */
  wcagLevel?: 'A' | 'AA' | 'AAA';

  /**
   * Rules to disable during testing
   */
  disabledRules?: string[];

  /**
   * Specific rules to run
   */
  includedRules?: string[];

  /**
   * Elements to exclude from testing
   */
  exclude?: string[];

  /**
   * Whether to test color contrast
   * @default true
   */
  checkColorContrast?: boolean;

  /**
   * Whether to test keyboard navigation
   * @default true
   */
  checkKeyboardNav?: boolean;

  /**
   * Whether to test ARIA attributes
   * @default true
   */
  checkAria?: boolean;
}

export interface A11yViolation {
  id: string;
  impact: 'minor' | 'moderate' | 'serious' | 'critical';
  description: string;
  help: string;
  helpUrl: string;
  nodes: A11yViolationNode[];
}

export interface A11yViolationNode {
  html: string;
  target: string[];
  failureSummary: string;
}

// Default export
export default {
  matchers,
  renderWithA11y,
  runAxe,
  testUtils,
};
