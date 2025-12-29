/**
 * Focus Trap Analyzer
 * Detects keyboard focus traps in the DOM
 */

import { FocusTrap } from '../types';

export class FocusTrapAnalyzer {
  private trapTimeout: number = 5000; // 5 seconds to detect trap

  /**
   * Detects focus traps in the container
   */
  async detectTraps(container: HTMLElement = document.body): Promise<FocusTrap[]> {
    const traps: FocusTrap[] = [];

    // Find modal/dialog elements
    const modalSelectors = [
      '[role="dialog"]',
      '[role="alertdialog"]',
      '.modal',
      '.dialog',
      '[aria-modal="true"]',
    ];

    const modals = Array.from(
      container.querySelectorAll(modalSelectors.join(','))
    ) as HTMLElement[];

    for (const modal of modals) {
      const trap = await this.analyzeModal(modal);
      if (trap) {
        traps.push(trap);
      }
    }

    // Check for other potential traps
    const otherTraps = await this.findUnintentionalTraps(container);
    traps.push(...otherTraps);

    return traps;
  }

  /**
   * Analyzes a modal for focus trap characteristics
   */
  private async analyzeModal(modal: HTMLElement): Promise<FocusTrap | null> {
    const isVisible = this.isVisible(modal);
    if (!isVisible) {
      return null;
    }

    const focusableElements = this.getFocusableElements(modal);
    const hasCloseButton = this.hasCloseButton(modal);
    const hasEscapeHandler = this.hasEscapeKeyHandler(modal);
    const isModal = modal.getAttribute('aria-modal') === 'true';

    if (focusableElements.length === 0) {
      return {
        detected: true,
        trapElement: modal,
        escapeMethod: hasEscapeHandler ? 'keyboard' : 'none',
        affectedElements: [],
        severity: 'critical',
        canEscape: hasEscapeHandler || hasCloseButton,
        description: 'Modal dialog with no focusable elements - users cannot interact',
      };
    }

    // Check if focus is trapped
    const canEscape = hasCloseButton || hasEscapeHandler;

    if (isModal && !canEscape) {
      return {
        detected: true,
        trapElement: modal,
        escapeMethod: 'mouse-only',
        affectedElements: focusableElements,
        severity: 'critical',
        canEscape: false,
        description:
          'Modal dialog without keyboard-accessible escape method (ESC key or close button)',
      };
    }

    return null;
  }

  /**
   * Finds unintentional focus traps
   */
  private async findUnintentionalTraps(container: HTMLElement): Promise<FocusTrap[]> {
    const traps: FocusTrap[] = [];

    // Find elements with tabindex that might create traps
    const elementsWithTabindex = Array.from(
      container.querySelectorAll('[tabindex]')
    ) as HTMLElement[];

    for (const element of elementsWithTabindex) {
      const tabindex = parseInt(element.getAttribute('tabindex') || '0', 10);

      // Negative tabindex on container might trap focus
      if (tabindex < 0 && element.children.length > 0) {
        const focusableChildren = this.getFocusableElements(element);

        if (focusableChildren.length > 0) {
          traps.push({
            detected: true,
            trapElement: element,
            escapeMethod: 'none',
            affectedElements: focusableChildren,
            severity: 'major',
            canEscape: false,
            description:
              'Container with tabindex="-1" contains focusable elements that may be unreachable',
          });
        }
      }
    }

    // Check for elements that might trap focus due to overflow
    const scrollContainers = Array.from(
      container.querySelectorAll('*')
    ).filter((el) => {
      const style = window.getComputedStyle(el);
      return (
        (style.overflow === 'auto' || style.overflow === 'scroll') &&
        el.scrollHeight > el.clientHeight
      );
    }) as HTMLElement[];

    for (const scrollContainer of scrollContainers) {
      const focusableElements = this.getFocusableElements(scrollContainer);

      if (focusableElements.length > 10) {
        // Large scrollable area with many focusable elements
        const hasScrollKeys = this.checkScrollKeySupport(scrollContainer);

        if (!hasScrollKeys) {
          traps.push({
            detected: true,
            trapElement: scrollContainer,
            escapeMethod: 'keyboard',
            affectedElements: focusableElements,
            severity: 'minor',
            canEscape: true,
            description:
              'Scrollable container with many focusable elements - users may have difficulty navigating',
          });
        }
      }
    }

    return traps;
  }

  /**
   * Gets focusable elements within a container
   */
  private getFocusableElements(container: HTMLElement): HTMLElement[] {
    const selector = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(',');

    return Array.from(container.querySelectorAll(selector)) as HTMLElement[];
  }

  /**
   * Checks if modal has a close button
   */
  private hasCloseButton(modal: HTMLElement): boolean {
    const closeSelectors = [
      'button.close',
      'button[aria-label*="close" i]',
      'button[aria-label*="dismiss" i]',
      '[role="button"][aria-label*="close" i]',
      '.modal-close',
      '[data-dismiss="modal"]',
    ];

    const closeButton = modal.querySelector(closeSelectors.join(','));
    return closeButton !== null;
  }

  /**
   * Checks if element has ESC key handler
   */
  private hasEscapeKeyHandler(element: HTMLElement): boolean {
    // Check for event listeners (limited detection)
    const hasKeydownAttr =
      element.hasAttribute('onkeydown') || element.hasAttribute('onkeyup');

    // Check for data attributes that might indicate key handling
    const hasKeyHandling =
      element.hasAttribute('data-keyboard') ||
      element.getAttribute('data-keyboard') !== 'false';

    return hasKeydownAttr || hasKeyHandling;
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
   * Checks if container supports arrow key scrolling
   */
  private checkScrollKeySupport(container: HTMLElement): boolean {
    // Check if container has keyboard event listeners
    const hasListeners =
      container.hasAttribute('onkeydown') || container.hasAttribute('onkeyup');

    // Check for role that implies keyboard support
    const role = container.getAttribute('role');
    const keyboardRoles = ['listbox', 'tree', 'grid', 'tablist'];

    return hasListeners || (role !== null && keyboardRoles.includes(role));
  }
}
