/**
 * Custom Jest/Vitest matcher: toBeAccessible
 * Validates that an element meets accessibility standards
 */

import { runAxe } from '../utils/axeRunner';
import type { AccessibilityOptions, A11yViolation } from '../index';

/**
 * Format violation messages for readable output
 */
function formatViolations(violations: A11yViolation[]): string {
  if (violations.length === 0) {
    return 'No accessibility violations found.';
  }

  const formatted = violations.map((violation, index) => {
    const impactBadge = `[${violation.impact.toUpperCase()}]`;
    const nodes = violation.nodes
      .map((node) => {
        const target = node.target.join(' > ');
        return `    Target: ${target}\n    HTML: ${node.html}\n    Issue: ${node.failureSummary}`;
      })
      .join('\n\n');

    return `
${index + 1}. ${impactBadge} ${violation.id}
   ${violation.description}
   Help: ${violation.help}
   More info: ${violation.helpUrl}

   Affected nodes:
${nodes}
`;
  });

  return formatted.join('\n');
}

/**
 * Categorize violations by impact level
 */
function categorizeViolations(violations: A11yViolation[]): {
  critical: A11yViolation[];
  serious: A11yViolation[];
  moderate: A11yViolation[];
  minor: A11yViolation[];
} {
  return {
    critical: violations.filter((v) => v.impact === 'critical'),
    serious: violations.filter((v) => v.impact === 'serious'),
    moderate: violations.filter((v) => v.impact === 'moderate'),
    minor: violations.filter((v) => v.impact === 'minor'),
  };
}

/**
 * Custom matcher implementation
 */
export async function toBeAccessible(
  this: jest.MatcherContext,
  received: HTMLElement | Document,
  options: AccessibilityOptions = {}
): Promise<jest.CustomMatcherResult> {
  const {
    wcagLevel = 'AA',
    disabledRules = [],
    includedRules,
    exclude = [],
    checkColorContrast = true,
    checkKeyboardNav = true,
    checkAria = true,
  } = options;

  try {
    // Run axe-core accessibility tests
    const results = await runAxe(received, {
      wcagLevel,
      disabledRules,
      includedRules,
      exclude,
      checkColorContrast,
      checkKeyboardNav,
      checkAria,
    });

    const violations = results.violations as A11yViolation[];
    const categorized = categorizeViolations(violations);

    const pass = violations.length === 0;

    if (pass) {
      return {
        pass: true,
        message: () =>
          `Expected element to have accessibility violations, but none were found.\n` +
          `Tested against WCAG ${wcagLevel} standards.\n` +
          `Total violations: 0`,
      };
    }

    // Build detailed failure message
    const summary = `
Accessibility Violations Summary:
  Critical: ${categorized.critical.length}
  Serious: ${categorized.serious.length}
  Moderate: ${categorized.moderate.length}
  Minor: ${categorized.minor.length}
  Total: ${violations.length}

WCAG Level: ${wcagLevel}
`;

    const detailedViolations = formatViolations(violations);

    return {
      pass: false,
      message: () =>
        `Expected element to be accessible, but found ${violations.length} violation(s):\n\n` +
        summary +
        '\nDetailed Violations:\n' +
        detailedViolations +
        '\n\nRecommendations:\n' +
        '  1. Review the violations listed above\n' +
        '  2. Fix critical and serious issues first\n' +
        '  3. Consult WCAG guidelines at https://www.w3.org/WAI/WCAG21/quickref/\n' +
        '  4. Use accessibility testing tools during development',
    };
  } catch (error) {
    return {
      pass: false,
      message: () =>
        `Failed to run accessibility tests: ${error instanceof Error ? error.message : String(error)}`,
    };
  }
}

/**
 * Matcher with strict mode - fails on any violation
 */
export async function toBeAccessibleStrict(
  this: jest.MatcherContext,
  received: HTMLElement | Document,
  options: AccessibilityOptions = {}
): Promise<jest.CustomMatcherResult> {
  const result = await toBeAccessible.call(this, received, {
    ...options,
    wcagLevel: options.wcagLevel || 'AAA',
  });

  return result;
}

/**
 * Matcher with permissive mode - only fails on critical/serious violations
 */
export async function toBeAccessiblePermissive(
  this: jest.MatcherContext,
  received: HTMLElement | Document,
  options: AccessibilityOptions = {}
): Promise<jest.CustomMatcherResult> {
  try {
    const results = await runAxe(received, {
      wcagLevel: options.wcagLevel || 'A',
      disabledRules: options.disabledRules,
      includedRules: options.includedRules,
      exclude: options.exclude,
      checkColorContrast: options.checkColorContrast ?? true,
      checkKeyboardNav: options.checkKeyboardNav ?? true,
      checkAria: options.checkAria ?? true,
    });

    const violations = results.violations as A11yViolation[];
    const criticalOrSerious = violations.filter(
      (v) => v.impact === 'critical' || v.impact === 'serious'
    );

    const pass = criticalOrSerious.length === 0;

    if (pass) {
      return {
        pass: true,
        message: () =>
          `Expected element to have critical/serious violations, but none were found.\n` +
          `Note: ${violations.length - criticalOrSerious.length} minor/moderate violations were ignored.`,
      };
    }

    return {
      pass: false,
      message: () =>
        `Expected element to be accessible (permissive mode), but found ${criticalOrSerious.length} critical/serious violation(s):\n\n` +
        formatViolations(criticalOrSerious),
    };
  } catch (error) {
    return {
      pass: false,
      message: () =>
        `Failed to run accessibility tests: ${error instanceof Error ? error.message : String(error)}`,
    };
  }
}
