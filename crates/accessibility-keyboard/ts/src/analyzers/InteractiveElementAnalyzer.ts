/**
 * Interactive Element Analyzer
 * Analyzes interactive elements for keyboard accessibility
 */

import { InteractiveElement } from '../types';

export class InteractiveElementAnalyzer {
  /**
   * Analyzes interactive elements in the container
   */
  async analyze(container: HTMLElement = document.body): Promise<InteractiveElement[]> {
    const interactiveElements: InteractiveElement[] = [];

    // Find elements with click handlers
    const clickableElements = this.findClickableElements(container);

    for (const element of clickableElements) {
      const analysis = this.analyzeElement(element);
      interactiveElements.push(analysis);
    }

    return interactiveElements;
  }

  /**
   * Finds potentially interactive elements
   */
  private findClickableElements(container: HTMLElement): HTMLElement[] {
    const elements: Set<HTMLElement> = new Set();

    // Elements with onclick
    container.querySelectorAll('[onclick]').forEach((el) => {
      elements.add(el as HTMLElement);
    });

    // Elements with common interactive roles
    const interactiveRoles = [
      'button',
      'link',
      'menuitem',
      'tab',
      'checkbox',
      'radio',
      'switch',
      'slider',
    ];

    interactiveRoles.forEach((role) => {
      container.querySelectorAll(`[role="${role}"]`).forEach((el) => {
        elements.add(el as HTMLElement);
      });
    });

    // Common interactive elements
    ['button', 'a', 'input', 'select', 'textarea'].forEach((tag) => {
      container.querySelectorAll(tag).forEach((el) => {
        elements.add(el as HTMLElement);
      });
    });

    // Elements with cursor pointer (likely interactive)
    const allElements = container.querySelectorAll('*');
    allElements.forEach((el) => {
      const style = window.getComputedStyle(el as HTMLElement);
      if (style.cursor === 'pointer') {
        elements.add(el as HTMLElement);
      }
    });

    return Array.from(elements);
  }

  /**
   * Analyzes a single interactive element
   */
  private analyzeElement(element: HTMLElement): InteractiveElement {
    const isKeyboardAccessible = this.isKeyboardAccessible(element);
    const hasProperRole = this.hasProperRole(element);
    const hasKeyboardHandler = this.hasKeyboardHandler(element);
    const requiredKeys = this.getRequiredKeys(element);
    const implementedKeys = this.getImplementedKeys(element);
    const missingHandlers = requiredKeys.filter(
      (key) => !implementedKeys.includes(key)
    );

    return {
      element,
      isKeyboardAccessible,
      hasProperRole,
      hasKeyboardHandler,
      missingHandlers,
      requiredKeys,
      implementedKeys,
    };
  }

  /**
   * Checks if element is keyboard accessible
   */
  private isKeyboardAccessible(element: HTMLElement): boolean {
    // Native interactive elements are keyboard accessible
    const tag = element.tagName.toLowerCase();
    if (['button', 'a', 'input', 'select', 'textarea'].includes(tag)) {
      return !element.hasAttribute('disabled');
    }

    // Check tabindex
    const tabindex = element.getAttribute('tabindex');
    if (tabindex === null || parseInt(tabindex, 10) < 0) {
      return false;
    }

    // Check for keyboard event handlers
    return this.hasKeyboardHandler(element);
  }

  /**
   * Checks if element has proper ARIA role
   */
  private hasProperRole(element: HTMLElement): boolean {
    const tag = element.tagName.toLowerCase();

    // Native elements don't need roles
    if (['button', 'a', 'input', 'select', 'textarea'].includes(tag)) {
      return true;
    }

    // Check for appropriate role
    const role = element.getAttribute('role');
    if (!role) {
      return false;
    }

    const interactiveRoles = [
      'button',
      'link',
      'checkbox',
      'radio',
      'switch',
      'slider',
      'menuitem',
      'tab',
      'combobox',
      'textbox',
    ];

    return interactiveRoles.includes(role);
  }

  /**
   * Checks if element has keyboard event handlers
   */
  private hasKeyboardHandler(element: HTMLElement): boolean {
    return (
      element.hasAttribute('onkeydown') ||
      element.hasAttribute('onkeyup') ||
      element.hasAttribute('onkeypress')
    );
  }

  /**
   * Gets required keyboard handlers based on role
   */
  private getRequiredKeys(element: HTMLElement): string[] {
    const role = element.getAttribute('role') || element.tagName.toLowerCase();
    const tag = element.tagName.toLowerCase();

    // Native elements handle their own keyboard
    if (['button', 'a', 'input', 'select', 'textarea'].includes(tag)) {
      return [];
    }

    const keyMap: { [key: string]: string[] } = {
      button: ['Enter', 'Space'],
      link: ['Enter'],
      checkbox: ['Space'],
      radio: ['Space', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'],
      switch: ['Space'],
      slider: ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End'],
      menuitem: ['Enter'],
      tab: ['Enter', 'Space'],
      combobox: ['ArrowUp', 'ArrowDown', 'Enter', 'Escape'],
      textbox: [], // Handled natively if contenteditable
    };

    return keyMap[role] || ['Enter', 'Space'];
  }

  /**
   * Gets implemented keyboard handlers
   */
  private getImplementedKeys(element: HTMLElement): string[] {
    // This is a simplified check - in real implementation,
    // we would need to inspect actual event listeners
    const hasKeyHandler = this.hasKeyboardHandler(element);

    if (!hasKeyHandler) {
      return [];
    }

    // Assume common keys are implemented if handler exists
    // In practice, you'd need to analyze the actual handler code
    return ['Enter', 'Space'];
  }
}
