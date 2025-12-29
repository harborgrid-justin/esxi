/**
 * Keyboard Operable Validator
 * Validates that all functionality is keyboard accessible (WCAG 2.1)
 */

export class KeyboardOperableValidator {
  /**
   * Validates keyboard operability
   */
  validate(container: HTMLElement = document.body): KeyboardOperableResult {
    const issues: OperableIssue[] = [];

    // Check for click handlers without keyboard support
    this.checkClickHandlers(container, issues);

    // Check for mouse-only interactions
    this.checkMouseOnlyInteractions(container, issues);

    // Check for missing focus indicators
    this.checkFocusIndicators(container, issues);

    // Check for keyboard traps
    this.checkForTraps(container, issues);

    const critical = issues.filter((i) => i.severity === 'critical').length;
    const serious = issues.filter((i) => i.severity === 'serious').length;

    return {
      passed: critical === 0,
      issues,
      summary: {
        total: issues.length,
        critical,
        serious,
        moderate: issues.filter((i) => i.severity === 'moderate').length,
      },
    };
  }

  /**
   * Checks for click handlers without keyboard equivalents
   */
  private checkClickHandlers(container: HTMLElement, issues: OperableIssue[]): void {
    const elementsWithClick = Array.from(
      container.querySelectorAll('[onclick]')
    ) as HTMLElement[];

    for (const element of elementsWithClick) {
      const tag = element.tagName.toLowerCase();
      const hasKeyHandler =
        element.hasAttribute('onkeydown') ||
        element.hasAttribute('onkeyup') ||
        element.hasAttribute('onkeypress');

      const isInteractive = ['button', 'a', 'input', 'select', 'textarea'].includes(
        tag
      );

      if (!isInteractive && !hasKeyHandler) {
        const tabindex = element.getAttribute('tabindex');
        const isFocusable = tabindex !== null && parseInt(tabindex, 10) >= 0;

        if (!isFocusable) {
          issues.push({
            element,
            type: 'click-without-keyboard',
            severity: 'critical',
            message: `${tag} has onclick but no keyboard handler and is not focusable`,
            wcag: ['2.1.1', '2.1.3'],
            howToFix:
              'Add tabindex="0" and keyboard event handlers (Enter/Space keys)',
          });
        } else {
          issues.push({
            element,
            type: 'click-without-keyboard',
            severity: 'serious',
            message: `${tag} has onclick but no keyboard handler`,
            wcag: ['2.1.1'],
            howToFix: 'Add keyboard event handlers for Enter and Space keys',
          });
        }
      }
    }
  }

  /**
   * Checks for mouse-only interactions
   */
  private checkMouseOnlyInteractions(
    container: HTMLElement,
    issues: OperableIssue[]
  ): void {
    const mouseOnlyElements = Array.from(
      container.querySelectorAll('[onmouseover], [onmouseout], [onmouseenter], [onmouseleave]')
    ) as HTMLElement[];

    for (const element of mouseOnlyElements) {
      const hasFocusEquivalent =
        element.hasAttribute('onfocus') || element.hasAttribute('onblur');

      if (!hasFocusEquivalent) {
        issues.push({
          element,
          type: 'mouse-only-interaction',
          severity: 'serious',
          message: 'Element has mouse events but no focus equivalents',
          wcag: ['2.1.1'],
          howToFix: 'Add onfocus/onblur handlers to provide keyboard equivalent',
        });
      }
    }
  }

  /**
   * Checks for missing focus indicators
   */
  private checkFocusIndicators(
    container: HTMLElement,
    issues: OperableIssue[]
  ): void {
    const focusableSelector = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(',');

    const focusableElements = Array.from(
      container.querySelectorAll(focusableSelector)
    ) as HTMLElement[];

    for (const element of focusableElements) {
      const style = window.getComputedStyle(element);

      // Check if outline is disabled
      if (
        style.outline === 'none' ||
        style.outlineWidth === '0px' ||
        style.outlineStyle === 'none'
      ) {
        // Check for alternative focus indicator
        const hasAlternative =
          (style.boxShadow && style.boxShadow !== 'none') ||
          (style.border && style.border !== 'none');

        if (!hasAlternative) {
          issues.push({
            element,
            type: 'no-focus-indicator',
            severity: 'serious',
            message: 'Element has no visible focus indicator',
            wcag: ['2.4.7'],
            howToFix:
              'Add visible focus styles (outline, border, or box-shadow) with sufficient contrast',
          });
        }
      }
    }
  }

  /**
   * Checks for potential keyboard traps
   */
  private checkForTraps(container: HTMLElement, issues: OperableIssue[]): void {
    const modals = Array.from(
      container.querySelectorAll('[role="dialog"], [role="alertdialog"], [aria-modal="true"]')
    ) as HTMLElement[];

    for (const modal of modals) {
      const style = window.getComputedStyle(modal);
      if (style.display === 'none' || style.visibility === 'hidden') {
        continue;
      }

      const hasCloseButton = modal.querySelector(
        'button[aria-label*="close" i], button.close, [data-dismiss]'
      );

      if (!hasCloseButton) {
        issues.push({
          element: modal,
          type: 'potential-trap',
          severity: 'critical',
          message: 'Modal dialog without visible close button may trap keyboard focus',
          wcag: ['2.1.2'],
          howToFix:
            'Add a keyboard-accessible close button or ESC key handler to allow users to exit',
        });
      }
    }
  }
}

export interface KeyboardOperableResult {
  passed: boolean;
  issues: OperableIssue[];
  summary: {
    total: number;
    critical: number;
    serious: number;
    moderate: number;
  };
}

export interface OperableIssue {
  element: HTMLElement;
  type:
    | 'click-without-keyboard'
    | 'mouse-only-interaction'
    | 'no-focus-indicator'
    | 'potential-trap';
  severity: 'critical' | 'serious' | 'moderate';
  message: string;
  wcag: string[];
  howToFix: string;
}
