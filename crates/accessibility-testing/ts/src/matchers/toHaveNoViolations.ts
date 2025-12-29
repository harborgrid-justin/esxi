/**
 * Custom Jest/Vitest matcher: toHaveNoViolations
 * Validates that axe-core found no accessibility violations
 * This is an alias/wrapper for jest-axe's toHaveNoViolations with enhanced reporting
 */

import type { A11yViolation } from '../index';

/**
 * Format a single violation for output
 */
function formatViolation(violation: A11yViolation, index: number): string {
  const impactColor = getImpactColor(violation.impact);
  const impactBadge = `[${violation.impact.toUpperCase()}]`;

  const affectedNodes = violation.nodes.map((node, nodeIndex) => {
    const selector = Array.isArray(node.target) ? node.target.join(' > ') : node.target;
    return `
      Node ${nodeIndex + 1}:
        Selector: ${selector}
        HTML: ${truncateHTML(node.html)}
        ${node.failureSummary ? `Issue: ${node.failureSummary}` : ''}`;
  }).join('\n');

  return `
  ${index + 1}. ${impactBadge} ${violation.id}
     Impact: ${impactColor}${violation.impact}${impactColor}
     Description: ${violation.description}
     Help: ${violation.help}
     Learn more: ${violation.helpUrl}

     Affected elements (${violation.nodes.length}):${affectedNodes}
`;
}

/**
 * Get color code for impact level (for terminal output)
 */
function getImpactColor(impact: string): string {
  switch (impact) {
    case 'critical':
      return '\x1b[91m'; // Bright red
    case 'serious':
      return '\x1b[31m'; // Red
    case 'moderate':
      return '\x1b[33m'; // Yellow
    case 'minor':
      return '\x1b[36m'; // Cyan
    default:
      return '\x1b[0m'; // Reset
  }
}

/**
 * Truncate HTML string for readability
 */
function truncateHTML(html: string, maxLength: number = 150): string {
  if (html.length <= maxLength) {
    return html;
  }
  return html.substring(0, maxLength) + '...';
}

/**
 * Generate summary statistics
 */
function generateSummary(violations: A11yViolation[]): string {
  const byImpact = {
    critical: violations.filter((v) => v.impact === 'critical').length,
    serious: violations.filter((v) => v.impact === 'serious').length,
    moderate: violations.filter((v) => v.impact === 'moderate').length,
    minor: violations.filter((v) => v.impact === 'minor').length,
  };

  const totalNodes = violations.reduce((sum, v) => sum + v.nodes.length, 0);

  return `
Summary:
  Total Violations: ${violations.length}
  Total Affected Elements: ${totalNodes}

  By Impact Level:
    Critical: ${byImpact.critical}
    Serious: ${byImpact.serious}
    Moderate: ${byImpact.moderate}
    Minor: ${byImpact.minor}
`;
}

/**
 * Custom matcher implementation
 */
export async function toHaveNoViolations(
  this: jest.MatcherContext,
  received: { violations: A11yViolation[] }
): Promise<jest.CustomMatcherResult> {
  if (!received || typeof received !== 'object') {
    return {
      pass: false,
      message: () =>
        'toHaveNoViolations requires an axe results object.\n' +
        'Usage: expect(await axe(container)).toHaveNoViolations()',
    };
  }

  if (!Array.isArray(received.violations)) {
    return {
      pass: false,
      message: () =>
        'Invalid axe results object. Expected an object with a violations array.\n' +
        `Received: ${JSON.stringify(received)}`,
    };
  }

  const violations = received.violations;
  const pass = violations.length === 0;

  if (pass) {
    return {
      pass: true,
      message: () =>
        'Expected to find accessibility violations, but none were found.\n' +
        'The element passes all accessibility checks.',
    };
  }

  // Sort violations by impact (critical first)
  const impactOrder = { critical: 0, serious: 1, moderate: 2, minor: 3 };
  const sortedViolations = [...violations].sort((a, b) => {
    const aOrder = impactOrder[a.impact] ?? 4;
    const bOrder = impactOrder[b.impact] ?? 4;
    return aOrder - bOrder;
  });

  const summary = generateSummary(violations);
  const formattedViolations = sortedViolations
    .map((violation, index) => formatViolation(violation, index))
    .join('\n');

  const helpfulTips = `
Quick Fixes:
  • Critical/Serious: Fix these immediately - they prevent users from accessing content
  • Moderate: Address these soon - they create significant barriers
  • Minor: Improve when possible - they affect user experience

Resources:
  • WCAG Quick Reference: https://www.w3.org/WAI/WCAG21/quickref/
  • Axe Rules: https://dequeuniversity.com/rules/axe/
  • WebAIM: https://webaim.org/
`;

  return {
    pass: false,
    message: () =>
      `Expected no accessibility violations, but found ${violations.length}:\n` +
      summary +
      '\nDetailed Violations:\n' +
      formattedViolations +
      '\n' +
      helpfulTips,
  };
}

/**
 * Variant that only checks for critical violations
 */
export async function toHaveNoCriticalViolations(
  this: jest.MatcherContext,
  received: { violations: A11yViolation[] }
): Promise<jest.CustomMatcherResult> {
  if (!received || !Array.isArray(received.violations)) {
    return {
      pass: false,
      message: () => 'Invalid axe results object.',
    };
  }

  const criticalViolations = received.violations.filter(
    (v) => v.impact === 'critical'
  );

  const pass = criticalViolations.length === 0;

  if (pass) {
    const otherViolations = received.violations.length - criticalViolations.length;
    return {
      pass: true,
      message: () =>
        `No critical violations found.\n` +
        (otherViolations > 0
          ? `Note: ${otherViolations} non-critical violation(s) exist but were not checked.`
          : 'No violations of any severity found.'),
    };
  }

  const formattedViolations = criticalViolations
    .map((violation, index) => formatViolation(violation, index))
    .join('\n');

  return {
    pass: false,
    message: () =>
      `Expected no critical violations, but found ${criticalViolations.length}:\n\n` +
      formattedViolations,
  };
}

/**
 * Variant that checks for specific rule violations
 */
export async function toHaveNoViolationsForRules(
  this: jest.MatcherContext,
  received: { violations: A11yViolation[] },
  ruleIds: string[]
): Promise<jest.CustomMatcherResult> {
  if (!received || !Array.isArray(received.violations)) {
    return {
      pass: false,
      message: () => 'Invalid axe results object.',
    };
  }

  const relevantViolations = received.violations.filter((v) =>
    ruleIds.includes(v.id)
  );

  const pass = relevantViolations.length === 0;

  if (pass) {
    return {
      pass: true,
      message: () =>
        `No violations found for rules: ${ruleIds.join(', ')}\n` +
        `Total violations: ${received.violations.length}`,
    };
  }

  const formattedViolations = relevantViolations
    .map((violation, index) => formatViolation(violation, index))
    .join('\n');

  return {
    pass: false,
    message: () =>
      `Expected no violations for rules [${ruleIds.join(', ')}], but found ${relevantViolations.length}:\n\n` +
      formattedViolations,
  };
}
