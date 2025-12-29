/**
 * Axe-core runner utility
 * Provides a wrapper around axe-core for accessibility testing
 */

import { configureAxe, type JestAxeConfigureOptions } from 'jest-axe';
import type { AccessibilityOptions } from '../index';

/**
 * Axe-core result structure
 */
export interface AxeResults {
  violations: Array<{
    id: string;
    impact: 'minor' | 'moderate' | 'serious' | 'critical';
    description: string;
    help: string;
    helpUrl: string;
    nodes: Array<{
      html: string;
      target: string[];
      failureSummary: string;
    }>;
  }>;
  passes: Array<{
    id: string;
    description: string;
  }>;
  incomplete: Array<{
    id: string;
    description: string;
  }>;
  inapplicable: Array<{
    id: string;
    description: string;
  }>;
}

/**
 * WCAG level to axe-core tags mapping
 */
const WCAG_TAGS: Record<string, string[]> = {
  A: ['wcag2a', 'wcag21a'],
  AA: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'],
  AAA: ['wcag2a', 'wcag2aa', 'wcag2aaa', 'wcag21a', 'wcag21aa', 'wcag21aaa'],
};

/**
 * Build axe-core configuration from options
 */
function buildAxeConfig(options: AccessibilityOptions): JestAxeConfigureOptions {
  const {
    wcagLevel = 'AA',
    disabledRules = [],
    includedRules,
    exclude = [],
    checkColorContrast = true,
    checkKeyboardNav = true,
    checkAria = true,
  } = options;

  const runOnly = {
    type: 'tag' as const,
    values: [...WCAG_TAGS[wcagLevel], 'best-practice'],
  };

  const rules: Record<string, { enabled: boolean }> = {};

  // Disable rules as requested
  disabledRules.forEach((ruleId) => {
    rules[ruleId] = { enabled: false };
  });

  // Disable specific rule categories if requested
  if (!checkColorContrast) {
    rules['color-contrast'] = { enabled: false };
  }

  if (!checkAria) {
    rules['aria-allowed-attr'] = { enabled: false };
    rules['aria-required-attr'] = { enabled: false };
    rules['aria-required-children'] = { enabled: false };
    rules['aria-required-parent'] = { enabled: false };
    rules['aria-roles'] = { enabled: false };
    rules['aria-valid-attr'] = { enabled: false };
    rules['aria-valid-attr-value'] = { enabled: false };
  }

  // If specific rules are included, only run those
  if (includedRules && includedRules.length > 0) {
    return {
      rules: includedRules.reduce(
        (acc, ruleId) => {
          acc[ruleId] = { enabled: true };
          return acc;
        },
        {} as Record<string, { enabled: boolean }>
      ),
    };
  }

  return {
    runOnly,
    rules,
    ...(exclude.length > 0 && {
      exclude: exclude.map((selector) => [selector]),
    }),
  };
}

/**
 * Run axe-core accessibility tests on an element
 *
 * @param element - The element to test
 * @param options - Accessibility test options
 * @returns Axe results including violations, passes, incomplete, and inapplicable
 *
 * @example
 * ```typescript
 * const results = await runAxe(container);
 * expect(results.violations).toHaveLength(0);
 * ```
 */
export async function runAxe(
  element: HTMLElement | Document,
  options: AccessibilityOptions = {}
): Promise<AxeResults> {
  const config = buildAxeConfig(options);
  const axe = configureAxe(config);

  try {
    const results = await axe(element);
    return results as unknown as AxeResults;
  } catch (error) {
    console.error('Axe-core test failed:', error);
    throw new Error(
      `Failed to run accessibility tests: ${error instanceof Error ? error.message : String(error)}`
    );
  }
}

/**
 * Run axe-core tests and return only violations
 */
export async function getAxeViolations(
  element: HTMLElement | Document,
  options: AccessibilityOptions = {}
): Promise<AxeResults['violations']> {
  const results = await runAxe(element, options);
  return results.violations;
}

/**
 * Run axe-core tests and check if element is accessible
 * Returns true if no violations found
 */
export async function isAccessible(
  element: HTMLElement | Document,
  options: AccessibilityOptions = {}
): Promise<boolean> {
  const results = await runAxe(element, options);
  return results.violations.length === 0;
}

/**
 * Run axe-core tests for specific rules only
 */
export async function runAxeForRules(
  element: HTMLElement | Document,
  ruleIds: string[],
  options: Omit<AccessibilityOptions, 'includedRules'> = {}
): Promise<AxeResults> {
  return runAxe(element, {
    ...options,
    includedRules: ruleIds,
  });
}

/**
 * Get detailed violation report as formatted string
 */
export function formatViolationReport(results: AxeResults): string {
  if (results.violations.length === 0) {
    return 'No accessibility violations found! ✓';
  }

  const lines: string[] = [
    `Found ${results.violations.length} accessibility violation(s):`,
    '',
  ];

  results.violations.forEach((violation, index) => {
    lines.push(`${index + 1}. [${violation.impact.toUpperCase()}] ${violation.id}`);
    lines.push(`   ${violation.description}`);
    lines.push(`   Help: ${violation.help}`);
    lines.push(`   More info: ${violation.helpUrl}`);
    lines.push(`   Affected elements: ${violation.nodes.length}`);

    violation.nodes.forEach((node, nodeIndex) => {
      lines.push(`     ${nodeIndex + 1}. ${node.target.join(' > ')}`);
      lines.push(`        ${node.html.substring(0, 80)}${node.html.length > 80 ? '...' : ''}`);
    });

    lines.push('');
  });

  return lines.join('\n');
}

/**
 * Get summary statistics for axe results
 */
export function getAxeSummary(results: AxeResults): {
  violations: number;
  passes: number;
  incomplete: number;
  inapplicable: number;
  byImpact: {
    critical: number;
    serious: number;
    moderate: number;
    minor: number;
  };
} {
  const byImpact = {
    critical: 0,
    serious: 0,
    moderate: 0,
    minor: 0,
  };

  results.violations.forEach((violation) => {
    byImpact[violation.impact]++;
  });

  return {
    violations: results.violations.length,
    passes: results.passes.length,
    incomplete: results.incomplete.length,
    inapplicable: results.inapplicable.length,
    byImpact,
  };
}

/**
 * Filter violations by impact level
 */
export function filterViolationsByImpact(
  results: AxeResults,
  impacts: Array<'critical' | 'serious' | 'moderate' | 'minor'>
): AxeResults['violations'] {
  return results.violations.filter((violation) =>
    impacts.includes(violation.impact)
  );
}

/**
 * Get only critical and serious violations
 */
export function getCriticalViolations(results: AxeResults): AxeResults['violations'] {
  return filterViolationsByImpact(results, ['critical', 'serious']);
}

/**
 * Check if results contain critical violations
 */
export function hasCriticalViolations(results: AxeResults): boolean {
  return results.violations.some(
    (v) => v.impact === 'critical' || v.impact === 'serious'
  );
}

/**
 * Common axe rule sets for quick testing
 */
export const RULE_SETS = {
  // Color contrast rules
  colorContrast: ['color-contrast'],

  // ARIA rules
  aria: [
    'aria-allowed-attr',
    'aria-command-name',
    'aria-hidden-body',
    'aria-hidden-focus',
    'aria-input-field-name',
    'aria-meter-name',
    'aria-progressbar-name',
    'aria-required-attr',
    'aria-required-children',
    'aria-required-parent',
    'aria-roledescription',
    'aria-roles',
    'aria-toggle-field-name',
    'aria-tooltip-name',
    'aria-valid-attr',
    'aria-valid-attr-value',
  ],

  // Keyboard navigation
  keyboard: [
    'accesskeys',
    'focus-order-semantics',
    'tabindex',
  ],

  // Forms
  forms: [
    'label',
    'label-content-name-mismatch',
    'label-title-only',
    'form-field-multiple-labels',
  ],

  // Images
  images: [
    'image-alt',
    'image-redundant-alt',
    'input-image-alt',
    'object-alt',
    'role-img-alt',
  ],

  // Headings and landmarks
  structure: [
    'bypass',
    'heading-order',
    'landmark-banner-is-top-level',
    'landmark-complementary-is-top-level',
    'landmark-contentinfo-is-top-level',
    'landmark-main-is-top-level',
    'landmark-no-duplicate-banner',
    'landmark-no-duplicate-contentinfo',
    'landmark-one-main',
    'landmark-unique',
    'page-has-heading-one',
    'region',
  ],
};

/**
 * Run tests for a specific category
 */
export async function runAxeCategory(
  element: HTMLElement | Document,
  category: keyof typeof RULE_SETS,
  options: Omit<AccessibilityOptions, 'includedRules'> = {}
): Promise<AxeResults> {
  return runAxeForRules(element, RULE_SETS[category], options);
}
