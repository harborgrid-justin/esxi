/**
 * Analyzes reading order and detects visual/logical mismatches
 */

import type {
  AccessibilityNode,
  ReadingOrder,
  ReadingOrderItem,
  ReadingOrderIssue,
  SeverityLevel,
} from '../types';

export class ReadingOrderAnalyzer {
  /**
   * Analyze reading order of accessibility tree
   */
  public analyze(root: AccessibilityNode): ReadingOrder {
    const items = this.buildReadingOrder(root);
    const issues = this.detectIssues(items);
    const score = this.calculateScore(items, issues);

    return { items, issues, score };
  }

  /**
   * Build reading order from accessibility tree
   */
  private buildReadingOrder(root: AccessibilityNode): ReadingOrderItem[] {
    const items: ReadingOrderItem[] = [];
    let order = 0;

    const traverse = (node: AccessibilityNode) => {
      // Skip hidden nodes
      if (node.hidden) {
        return;
      }

      // Add focusable and content nodes
      if (node.focusable || this.hasContent(node)) {
        const visualPosition = this.getVisualPosition(node);
        const logicalPosition = order;

        items.push({
          node,
          order: order++,
          visualPosition,
          logicalPosition,
          isOutOfOrder: false,
          deviation: 0,
        });
      }

      // Traverse children
      node.children.forEach(traverse);
    };

    traverse(root);

    // Calculate deviations
    this.calculateDeviations(items);

    return items;
  }

  /**
   * Check if node has meaningful content
   */
  private hasContent(node: AccessibilityNode): boolean {
    return Boolean(node.name && node.name.trim().length > 0);
  }

  /**
   * Get visual position (reading direction order)
   */
  private getVisualPosition(node: AccessibilityNode): { x: number; y: number } {
    const rect = node.boundingBox;
    return {
      x: rect.left + rect.width / 2,
      y: rect.top + rect.height / 2,
    };
  }

  /**
   * Calculate deviations between visual and logical order
   */
  private calculateDeviations(items: ReadingOrderItem[]): void {
    // Sort by visual position (top to bottom, left to right)
    const visualOrder = [...items].sort((a, b) => {
      const yDiff = a.visualPosition.y - b.visualPosition.y;
      if (Math.abs(yDiff) > 10) {
        return yDiff;
      }
      return a.visualPosition.x - b.visualPosition.x;
    });

    // Calculate expected order based on visual position
    visualOrder.forEach((item, index) => {
      const expectedOrder = index;
      const actualOrder = item.logicalPosition;
      const deviation = Math.abs(expectedOrder - actualOrder);

      item.deviation = deviation;
      item.isOutOfOrder = deviation > 2; // Allow small deviations
    });
  }

  /**
   * Detect reading order issues
   */
  private detectIssues(items: ReadingOrderItem[]): ReadingOrderIssue[] {
    const issues: ReadingOrderIssue[] = [];

    // Visual/logical mismatch
    const outOfOrderItems = items.filter(item => item.isOutOfOrder);
    if (outOfOrderItems.length > 0) {
      issues.push({
        type: 'visual-logical-mismatch',
        severity: this.calculateSeverity(outOfOrderItems.length, items.length),
        items: outOfOrderItems,
        description: `${outOfOrderItems.length} elements have mismatched visual and logical reading order`,
        remediation:
          'Ensure DOM order matches visual order, or use CSS Grid/Flexbox order properties sparingly with proper focus management',
      });
    }

    // Focus order mismatch
    const focusOrderIssue = this.detectFocusOrderMismatch(items);
    if (focusOrderIssue) {
      issues.push(focusOrderIssue);
    }

    // Tabindex abuse
    const tabindexIssue = this.detectTabindexAbuse(items);
    if (tabindexIssue) {
      issues.push(tabindexIssue);
    }

    return issues;
  }

  /**
   * Detect focus order mismatch
   */
  private detectFocusOrderMismatch(items: ReadingOrderItem[]): ReadingOrderIssue | null {
    const focusableItems = items.filter(item => item.node.focusable);

    if (focusableItems.length < 2) {
      return null;
    }

    const mismatched: ReadingOrderItem[] = [];

    for (let i = 1; i < focusableItems.length; i++) {
      const prev = focusableItems[i - 1];
      const curr = focusableItems[i];

      // Check if focus order jumps around visually
      const yDiff = Math.abs(curr.visualPosition.y - prev.visualPosition.y);
      const xDiff = curr.visualPosition.x - prev.visualPosition.x;

      // Moving backward or jumping between lines unexpectedly
      if (yDiff < 50 && xDiff < -100) {
        mismatched.push(curr);
      } else if (yDiff > 200) {
        // Large vertical jump
        mismatched.push(curr);
      }
    }

    if (mismatched.length === 0) {
      return null;
    }

    return {
      type: 'focus-order-mismatch',
      severity: this.calculateSeverity(mismatched.length, focusableItems.length),
      items: mismatched,
      description: `${mismatched.length} focusable elements have unexpected focus order`,
      remediation:
        'Ensure tab order follows a logical pattern. Avoid positive tabindex values and CSS properties that change visual order',
    };
  }

  /**
   * Detect tabindex abuse
   */
  private detectTabindexAbuse(items: ReadingOrderItem[]): ReadingOrderIssue | null {
    const positiveTabindex = items.filter(
      item => item.node.tabIndex > 0
    );

    if (positiveTabindex.length === 0) {
      return null;
    }

    return {
      type: 'tabindex-abuse',
      severity: 'serious',
      items: positiveTabindex,
      description: `${positiveTabindex.length} elements use positive tabindex values`,
      remediation:
        'Avoid positive tabindex values. Use tabindex="0" for custom interactive elements and tabindex="-1" to remove from tab order. Rely on DOM order for natural tab sequence',
    };
  }

  /**
   * Calculate severity based on percentage of issues
   */
  private calculateSeverity(issueCount: number, totalCount: number): SeverityLevel {
    const percentage = (issueCount / totalCount) * 100;

    if (percentage > 50) return 'critical';
    if (percentage > 25) return 'serious';
    if (percentage > 10) return 'moderate';
    return 'minor';
  }

  /**
   * Calculate overall score (0-100)
   */
  private calculateScore(items: ReadingOrderItem[], issues: ReadingOrderIssue[]): number {
    if (items.length === 0) {
      return 100;
    }

    let score = 100;

    // Deduct for issues
    issues.forEach(issue => {
      switch (issue.severity) {
        case 'critical':
          score -= 40;
          break;
        case 'serious':
          score -= 25;
          break;
        case 'moderate':
          score -= 15;
          break;
        case 'minor':
          score -= 5;
          break;
      }
    });

    // Deduct for average deviation
    const avgDeviation = items.reduce((sum, item) => sum + item.deviation, 0) / items.length;
    score -= avgDeviation * 2;

    return Math.max(0, Math.min(100, Math.round(score)));
  }

  /**
   * Get reading order sequence as text
   */
  public getReadingSequence(items: ReadingOrderItem[]): string[] {
    return items
      .sort((a, b) => a.order - b.order)
      .map(item => item.node.name || '[unnamed element]')
      .filter(name => name !== '[unnamed element]');
  }

  /**
   * Simulate screen reader reading the page
   */
  public simulateReading(items: ReadingOrderItem[]): string {
    const sequence = this.getReadingSequence(items);
    return sequence.join('. ');
  }
}
