/**
 * Tab Order Analyzer
 * Analyzes and validates tab order of focusable elements
 */

import { FocusableElement, TabOrder, TabOrderIssue } from '../types';

export class TabOrderAnalyzer {
  /**
   * Analyzes tab order in a given container
   */
  async analyze(container: HTMLElement = document.body): Promise<TabOrder> {
    const focusableElements = this.getFocusableElements(container);
    const issues = this.detectIssues(focusableElements);

    const logicalOrder = focusableElements.map((_, index) => index);
    const domOrder = focusableElements.map((el) => el.domOrder);

    return {
      elements: focusableElements,
      logicalOrder,
      domOrder,
      hasLogicalIssues: issues.length > 0,
      issues,
    };
  }

  /**
   * Gets all focusable elements in tab order
   */
  private getFocusableElements(container: HTMLElement): FocusableElement[] {
    const selector = [
      'a[href]',
      'area[href]',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'button:not([disabled])',
      'iframe',
      '[tabindex]',
      '[contenteditable="true"]',
    ].join(',');

    const elements = Array.from(container.querySelectorAll(selector)) as HTMLElement[];
    const focusableElements: FocusableElement[] = [];

    elements.forEach((element, index) => {
      const tabIndex = this.getTabIndex(element);
      const isVisible = this.isVisible(element);
      const isDisabled = this.isDisabled(element);
      const hasValidRole = this.hasValidRole(element);
      const hasFocusIndicator = this.hasFocusIndicator(element);

      // Only include elements that are part of tab sequence
      if (tabIndex >= 0 || element.getAttribute('tabindex') !== null) {
        focusableElements.push({
          element,
          tabIndex,
          isVisible,
          isDisabled,
          hasValidRole,
          hasFocusIndicator,
          visualTabOrder: index,
          domOrder: index,
          selector: this.getSelector(element),
          computedRole: this.getComputedRole(element),
          ariaLabel: element.getAttribute('aria-label') || undefined,
          ariaLabelledBy: element.getAttribute('aria-labelledby') || undefined,
          ariaDescribedBy: element.getAttribute('aria-describedby') || undefined,
        });
      }
    });

    // Sort by tab index (positive first, then 0/-1 in DOM order)
    return this.sortByTabOrder(focusableElements);
  }

  /**
   * Gets the effective tab index
   */
  private getTabIndex(element: HTMLElement): number {
    const tabindexAttr = element.getAttribute('tabindex');
    if (tabindexAttr !== null) {
      return parseInt(tabindexAttr, 10);
    }

    // Default tab indices for naturally focusable elements
    const tag = element.tagName.toLowerCase();
    if (['a', 'button', 'input', 'select', 'textarea'].includes(tag)) {
      return 0;
    }

    return -1;
  }

  /**
   * Checks if element is visible
   */
  private isVisible(element: HTMLElement): boolean {
    const style = window.getComputedStyle(element);
    const rect = element.getBoundingClientRect();

    return (
      style.display !== 'none' &&
      style.visibility !== 'hidden' &&
      style.opacity !== '0' &&
      rect.width > 0 &&
      rect.height > 0
    );
  }

  /**
   * Checks if element is disabled
   */
  private isDisabled(element: HTMLElement): boolean {
    return (
      element.hasAttribute('disabled') ||
      element.getAttribute('aria-disabled') === 'true'
    );
  }

  /**
   * Checks if element has a valid ARIA role
   */
  private hasValidRole(element: HTMLElement): boolean {
    const role = element.getAttribute('role');
    if (!role) return true; // No role is fine

    // Valid interactive roles
    const validRoles = [
      'button',
      'link',
      'checkbox',
      'radio',
      'textbox',
      'combobox',
      'slider',
      'spinbutton',
      'searchbox',
      'switch',
      'tab',
      'menuitem',
      'menuitemcheckbox',
      'menuitemradio',
      'option',
      'gridcell',
      'treeitem',
    ];

    return validRoles.includes(role);
  }

  /**
   * Checks if element has a visible focus indicator
   */
  private hasFocusIndicator(element: HTMLElement): boolean {
    const style = window.getComputedStyle(element);

    // Check outline
    if (style.outline && style.outline !== 'none' && style.outlineWidth !== '0px') {
      return true;
    }

    // Check for custom focus styles (box-shadow, border)
    if (style.boxShadow && style.boxShadow !== 'none') {
      return true;
    }

    return false;
  }

  /**
   * Gets computed ARIA role
   */
  private getComputedRole(element: HTMLElement): string | null {
    return element.getAttribute('role') || element.tagName.toLowerCase();
  }

  /**
   * Generates a CSS selector for the element
   */
  private getSelector(element: HTMLElement): string {
    if (element.id) {
      return `#${element.id}`;
    }

    const classes = element.className
      ? `.${element.className.split(' ').filter(Boolean).join('.')}`
      : '';
    return `${element.tagName.toLowerCase()}${classes}`;
  }

  /**
   * Sorts elements by tab order
   */
  private sortByTabOrder(elements: FocusableElement[]): FocusableElement[] {
    return elements.sort((a, b) => {
      // Positive tab indices come first, in order
      if (a.tabIndex > 0 && b.tabIndex > 0) {
        return a.tabIndex - b.tabIndex;
      }
      if (a.tabIndex > 0) return -1;
      if (b.tabIndex > 0) return 1;

      // Then 0/-1 in DOM order
      return a.domOrder - b.domOrder;
    });
  }

  /**
   * Detects tab order issues
   */
  private detectIssues(elements: FocusableElement[]): TabOrderIssue[] {
    const issues: TabOrderIssue[] = [];

    elements.forEach((element, index) => {
      // Check for positive tabindex
      if (element.tabIndex > 0) {
        issues.push({
          type: 'positive-tabindex',
          severity: 'warning',
          element,
          message: `Element has positive tabindex (${element.tabIndex}). This can cause confusing tab order.`,
          wcagCriteria: ['2.4.3'],
          suggestion: 'Use tabindex="0" and reorder elements in DOM instead.',
        });
      }

      // Check for hidden focusable elements
      if (!element.isVisible && element.tabIndex >= 0) {
        issues.push({
          type: 'hidden-focusable',
          severity: 'error',
          element,
          message: 'Element is focusable but not visible.',
          wcagCriteria: ['2.4.3', '2.4.7'],
          suggestion: 'Either make the element visible or set tabindex="-1".',
        });
      }

      // Check for out-of-order elements
      if (index > 0) {
        const prev = elements[index - 1];
        if (element.domOrder < prev.domOrder && element.tabIndex === 0 && prev.tabIndex === 0) {
          issues.push({
            type: 'out-of-order',
            severity: 'warning',
            element,
            message: 'Element appears before previous element in tab order but after in DOM.',
            wcagCriteria: ['2.4.3'],
            suggestion: 'Reorder elements in DOM to match visual tab order.',
          });
        }
      }

      // Check for missing focus indicator
      if (!element.hasFocusIndicator) {
        issues.push({
          type: 'no-focus-indicator',
          severity: 'error',
          element,
          message: 'Element lacks a visible focus indicator.',
          wcagCriteria: ['2.4.7'],
          suggestion: 'Add visible focus styles (outline, border, box-shadow).',
        });
      }
    });

    return issues;
  }
}
