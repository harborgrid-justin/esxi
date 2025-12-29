/**
 * Focus Management Utilities
 * Helper functions for managing keyboard focus
 */

/**
 * Gets all focusable elements in a container
 */
export function getFocusableElements(
  container: HTMLElement = document.body
): HTMLElement[] {
  const selector = [
    'a[href]:not([disabled])',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
    '[contenteditable="true"]',
  ].join(',');

  const elements = Array.from(container.querySelectorAll(selector)) as HTMLElement[];

  // Filter out invisible elements
  return elements.filter((el) => isVisible(el) && !isDisabled(el));
}

/**
 * Checks if an element is visible
 */
export function isVisible(element: HTMLElement): boolean {
  const style = window.getComputedStyle(element);
  const rect = element.getBoundingClientRect();

  return (
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    parseFloat(style.opacity) > 0 &&
    rect.width > 0 &&
    rect.height > 0
  );
}

/**
 * Checks if an element is disabled
 */
export function isDisabled(element: HTMLElement): boolean {
  return (
    element.hasAttribute('disabled') ||
    element.getAttribute('aria-disabled') === 'true' ||
    (element as HTMLInputElement).disabled
  );
}

/**
 * Creates a focus trap within a container
 */
export function createFocusTrap(container: HTMLElement): () => void {
  const focusableElements = getFocusableElements(container);

  if (focusableElements.length === 0) {
    console.warn('No focusable elements found in container');
    return () => {};
  }

  const firstElement = focusableElements[0];
  const lastElement = focusableElements[focusableElements.length - 1];

  const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key !== 'Tab') return;

    if (event.shiftKey) {
      // Shift + Tab
      if (document.activeElement === firstElement) {
        event.preventDefault();
        lastElement.focus();
      }
    } else {
      // Tab
      if (document.activeElement === lastElement) {
        event.preventDefault();
        firstElement.focus();
      }
    }
  };

  container.addEventListener('keydown', handleKeyDown);

  // Focus first element
  firstElement.focus();

  // Return cleanup function
  return () => {
    container.removeEventListener('keydown', handleKeyDown);
  };
}

/**
 * Saves current focus to restore later
 */
export function saveFocus(): HTMLElement | null {
  return document.activeElement as HTMLElement;
}

/**
 * Restores previously saved focus
 */
export function restoreFocus(element: HTMLElement | null): void {
  if (element && element.focus) {
    element.focus();
  }
}

/**
 * Focuses the first focusable element in a container
 */
export function focusFirstElement(container: HTMLElement): boolean {
  const focusableElements = getFocusableElements(container);

  if (focusableElements.length > 0) {
    focusableElements[0].focus();
    return true;
  }

  return false;
}

/**
 * Focuses the last focusable element in a container
 */
export function focusLastElement(container: HTMLElement): boolean {
  const focusableElements = getFocusableElements(container);

  if (focusableElements.length > 0) {
    focusableElements[focusableElements.length - 1].focus();
    return true;
  }

  return false;
}

/**
 * Gets the next focusable element
 */
export function getNextFocusable(
  current: HTMLElement,
  container: HTMLElement = document.body
): HTMLElement | null {
  const focusableElements = getFocusableElements(container);
  const currentIndex = focusableElements.indexOf(current);

  if (currentIndex === -1 || currentIndex === focusableElements.length - 1) {
    return null;
  }

  return focusableElements[currentIndex + 1];
}

/**
 * Gets the previous focusable element
 */
export function getPreviousFocusable(
  current: HTMLElement,
  container: HTMLElement = document.body
): HTMLElement | null {
  const focusableElements = getFocusableElements(container);
  const currentIndex = focusableElements.indexOf(current);

  if (currentIndex <= 0) {
    return null;
  }

  return focusableElements[currentIndex - 1];
}

/**
 * Checks if an element is focusable
 */
export function isFocusable(element: HTMLElement): boolean {
  const tabindex = element.getAttribute('tabindex');

  if (tabindex !== null) {
    return parseInt(tabindex, 10) >= -1;
  }

  const tag = element.tagName.toLowerCase();
  const focusableTags = ['a', 'button', 'input', 'select', 'textarea'];

  if (focusableTags.includes(tag)) {
    return !isDisabled(element);
  }

  return element.hasAttribute('contenteditable');
}

/**
 * Makes an element focusable
 */
export function makeFocusable(element: HTMLElement): void {
  if (!isFocusable(element)) {
    element.setAttribute('tabindex', '0');
  }
}

/**
 * Makes an element unfocusable
 */
export function makeUnfocusable(element: HTMLElement): void {
  element.setAttribute('tabindex', '-1');
}

/**
 * Temporarily disables tab order for all elements except the specified container
 */
export function isolateTabOrder(container: HTMLElement): () => void {
  const allFocusable = getFocusableElements(document.body);
  const containerFocusable = getFocusableElements(container);

  const outsideElements = allFocusable.filter(
    (el) => !containerFocusable.includes(el)
  );

  const originalTabIndices = new Map<HTMLElement, string | null>();

  // Save original tabindex and disable
  outsideElements.forEach((el) => {
    originalTabIndices.set(el, el.getAttribute('tabindex'));
    el.setAttribute('tabindex', '-1');
  });

  // Return restore function
  return () => {
    outsideElements.forEach((el) => {
      const original = originalTabIndices.get(el);
      if (original === null) {
        el.removeAttribute('tabindex');
      } else {
        el.setAttribute('tabindex', original);
      }
    });
  };
}

/**
 * Announces text to screen readers
 */
export function announce(
  message: string,
  priority: 'polite' | 'assertive' = 'polite'
): void {
  const announcer = document.getElementById('a11y-announcer') || createAnnouncer();

  announcer.setAttribute('aria-live', priority);
  announcer.textContent = message;

  // Clear after announcement
  setTimeout(() => {
    announcer.textContent = '';
  }, 1000);
}

/**
 * Creates an announcer element for screen readers
 */
function createAnnouncer(): HTMLElement {
  const announcer = document.createElement('div');
  announcer.id = 'a11y-announcer';
  announcer.setAttribute('role', 'status');
  announcer.setAttribute('aria-live', 'polite');
  announcer.setAttribute('aria-atomic', 'true');
  announcer.style.cssText = `
    position: absolute;
    left: -10000px;
    width: 1px;
    height: 1px;
    overflow: hidden;
  `;
  document.body.appendChild(announcer);
  return announcer;
}
