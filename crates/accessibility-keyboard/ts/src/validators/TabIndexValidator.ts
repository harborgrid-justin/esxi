/**
 * Tab Index Validator
 * Validates proper usage of tabindex attributes
 */

export class TabIndexValidator {
  /**
   * Validates tabindex usage in container
   */
  validate(container: HTMLElement = document.body): TabIndexValidationResult {
    const elements = Array.from(
      container.querySelectorAll('[tabindex]')
    ) as HTMLElement[];

    const violations: TabIndexViolation[] = [];
    const warnings: string[] = [];

    for (const element of elements) {
      const tabindex = element.getAttribute('tabindex');
      if (tabindex === null) continue;

      const tabindexValue = parseInt(tabindex, 10);

      // Check for positive tabindex
      if (tabindexValue > 0) {
        violations.push({
          element,
          value: tabindexValue,
          issue: 'positive-tabindex',
          message: `Positive tabindex (${tabindexValue}) can cause confusing navigation order`,
          severity: 'warning',
          recommendation: 'Use tabindex="0" and reorder elements in DOM instead',
        });
      }

      // Check for very negative tabindex
      if (tabindexValue < -1) {
        violations.push({
          element,
          value: tabindexValue,
          issue: 'invalid-tabindex',
          message: `Invalid tabindex value (${tabindexValue}). Use 0, -1, or positive integers`,
          severity: 'error',
          recommendation: 'Change to tabindex="-1" to remove from tab order',
        });
      }

      // Check if non-interactive element has tabindex="0"
      if (tabindexValue === 0 && !this.isInteractive(element)) {
        violations.push({
          element,
          value: tabindexValue,
          issue: 'non-interactive-focusable',
          message: 'Non-interactive element is made focusable with tabindex="0"',
          severity: 'warning',
          recommendation:
            'Add appropriate role and keyboard handlers, or remove tabindex',
        });
      }

      // Check if interactive element has tabindex="-1"
      if (tabindexValue === -1 && this.isInteractive(element)) {
        warnings.push(
          `Interactive ${element.tagName.toLowerCase()} is removed from tab order with tabindex="-1"`
        );
      }
    }

    return {
      passed: violations.filter((v) => v.severity === 'error').length === 0,
      violations,
      warnings,
      totalElements: elements.length,
      positiveTabindexCount: violations.filter((v) => v.issue === 'positive-tabindex')
        .length,
    };
  }

  /**
   * Checks if element is naturally interactive
   */
  private isInteractive(element: HTMLElement): boolean {
    const tag = element.tagName.toLowerCase();
    const interactiveTags = ['button', 'a', 'input', 'select', 'textarea'];

    if (interactiveTags.includes(tag)) {
      return true;
    }

    const role = element.getAttribute('role');
    const interactiveRoles = [
      'button',
      'link',
      'checkbox',
      'radio',
      'slider',
      'spinbutton',
      'switch',
      'tab',
      'menuitem',
    ];

    return role !== null && interactiveRoles.includes(role);
  }
}

export interface TabIndexValidationResult {
  passed: boolean;
  violations: TabIndexViolation[];
  warnings: string[];
  totalElements: number;
  positiveTabindexCount: number;
}

export interface TabIndexViolation {
  element: HTMLElement;
  value: number;
  issue: 'positive-tabindex' | 'invalid-tabindex' | 'non-interactive-focusable';
  message: string;
  severity: 'error' | 'warning';
  recommendation: string;
}
