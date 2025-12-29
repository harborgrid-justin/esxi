/**
 * Analyzes heading structure and hierarchy
 */

import type {
  AccessibilityNode,
  HeadingStructure,
  HeadingInfo,
  HeadingIssue,
} from '../types';

export class HeadingAnalyzer {
  /**
   * Analyze heading structure
   */
  public analyze(root: AccessibilityNode): HeadingStructure {
    const headings = this.extractHeadings(root);
    const issues = this.detectIssues(headings, root);
    const score = this.calculateScore(headings, issues);

    return { headings, issues, score };
  }

  /**
   * Extract headings from tree
   */
  private extractHeadings(root: AccessibilityNode): HeadingInfo[] {
    const headings: HeadingInfo[] = [];
    let order = 0;

    const traverse = (node: AccessibilityNode) => {
      if (node.role === 'heading' && node.level) {
        const text = node.name || '';
        const empty = text.trim().length === 0;

        headings.push({
          node,
          level: node.level,
          text,
          order: order++,
          skipped: false,
          empty,
        });
      }

      node.children.forEach(traverse);
    };

    traverse(root);

    // Mark skipped levels
    this.markSkippedLevels(headings);

    return headings;
  }

  /**
   * Mark headings with skipped levels
   */
  private markSkippedLevels(headings: HeadingInfo[]): void {
    for (let i = 1; i < headings.length; i++) {
      const prev = headings[i - 1];
      const curr = headings[i];

      // Check if level jumped by more than 1
      if (curr.level > prev.level + 1) {
        curr.skipped = true;
      }
    }
  }

  /**
   * Detect heading issues
   */
  private detectIssues(headings: HeadingInfo[], root: AccessibilityNode): HeadingIssue[] {
    const issues: HeadingIssue[] = [];

    // Missing H1
    const h1Headings = headings.filter(h => h.level === 1);
    if (h1Headings.length === 0) {
      issues.push({
        type: 'missing-h1',
        severity: 'serious',
        heading: { node: root, level: 1, text: '', order: -1, skipped: false, empty: false },
        description: 'Page is missing an h1 heading',
        remediation: 'Add an h1 heading that describes the main purpose of the page',
      });
    }

    // Multiple H1s
    if (h1Headings.length > 1) {
      h1Headings.slice(1).forEach(heading => {
        issues.push({
          type: 'multiple-h1',
          severity: 'moderate',
          heading,
          description: 'Multiple h1 headings found on page',
          remediation: 'Use only one h1 heading per page to identify the main topic',
        });
      });
    }

    // Skipped levels
    const skippedHeadings = headings.filter(h => h.skipped);
    skippedHeadings.forEach(heading => {
      issues.push({
        type: 'skipped-level',
        severity: 'moderate',
        heading,
        description: `Heading level ${heading.level} skips previous levels`,
        remediation: 'Ensure heading levels increase by one at most. Do not skip levels (e.g., h2 to h4)',
      });
    });

    // Empty headings
    const emptyHeadings = headings.filter(h => h.empty);
    emptyHeadings.forEach(heading => {
      issues.push({
        type: 'empty-heading',
        severity: 'serious',
        heading,
        description: `h${heading.level} heading is empty`,
        remediation: 'Add descriptive text to the heading or remove it if not needed',
      });
    });

    // Improper nesting
    const nestingIssues = this.detectImproperNesting(headings);
    issues.push(...nestingIssues);

    return issues;
  }

  /**
   * Detect improper nesting
   */
  private detectImproperNesting(headings: HeadingInfo[]): HeadingIssue[] {
    const issues: HeadingIssue[] = [];
    const stack: HeadingInfo[] = [];

    for (const heading of headings) {
      // Pop headings of equal or higher level
      while (stack.length > 0 && stack[stack.length - 1].level >= heading.level) {
        stack.pop();
      }

      // Check if we're jumping levels
      if (stack.length > 0) {
        const parent = stack[stack.length - 1];
        if (heading.level > parent.level + 1) {
          issues.push({
            type: 'improper-nesting',
            severity: 'moderate',
            heading,
            description: `h${heading.level} follows h${parent.level}, skipping levels`,
            remediation: 'Maintain a logical heading hierarchy by not skipping levels',
          });
        }
      } else if (heading.level > 1) {
        // First heading but not h1
        issues.push({
          type: 'improper-nesting',
          severity: 'moderate',
          heading,
          description: `First heading is h${heading.level}, should start with h1`,
          remediation: 'Start the heading structure with h1',
        });
      }

      stack.push(heading);
    }

    return issues;
  }

  /**
   * Calculate score
   */
  private calculateScore(headings: HeadingInfo[], issues: HeadingIssue[]): number {
    if (headings.length === 0) {
      return 50; // No headings is not great but not terrible
    }

    let score = 100;

    // Deduct for issues
    issues.forEach(issue => {
      switch (issue.severity) {
        case 'critical':
          score -= 30;
          break;
        case 'serious':
          score -= 20;
          break;
        case 'moderate':
          score -= 10;
          break;
        case 'minor':
          score -= 5;
          break;
      }
    });

    // Bonus for good structure
    const h1Count = headings.filter(h => h.level === 1).length;
    if (h1Count === 1) score += 5;

    const hasNoSkipped = headings.every(h => !h.skipped);
    if (hasNoSkipped) score += 5;

    const hasNoEmpty = headings.every(h => !h.empty);
    if (hasNoEmpty) score += 5;

    return Math.max(0, Math.min(100, Math.round(score)));
  }

  /**
   * Get heading outline
   */
  public getHeadingOutline(headings: HeadingInfo[]): string[] {
    return headings.map(heading => {
      const indent = '  '.repeat(heading.level - 1);
      const text = heading.text || '[empty]';
      return `${indent}h${heading.level}: ${text}`;
    });
  }

  /**
   * Get heading navigation list
   */
  public getHeadingNavigationList(headings: HeadingInfo[]): AccessibilityNode[] {
    return headings.map(h => h.node);
  }
}
