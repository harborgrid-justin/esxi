/**
 * Testing utilities for accessibility testing
 * Common helpers for writing accessible component tests
 */

import { screen, within, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

/**
 * Test if an element is keyboard accessible
 */
export async function testKeyboardAccessibility(
  element: HTMLElement,
  expectedRole?: string
): Promise<boolean> {
  // Check if element is focusable
  const isFocusable =
    element.tabIndex >= 0 ||
    ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName);

  if (!isFocusable) {
    return false;
  }

  // Check for role if specified
  if (expectedRole) {
    const role = element.getAttribute('role') || element.tagName.toLowerCase();
    if (role !== expectedRole && role !== expectedRole.toLowerCase()) {
      return false;
    }
  }

  return true;
}

/**
 * Simulate keyboard navigation through focusable elements
 */
export async function navigateWithKeyboard(
  container: HTMLElement,
  steps: number = 1
): Promise<HTMLElement[]> {
  const user = userEvent.setup();
  const focusedElements: HTMLElement[] = [];

  for (let i = 0; i < steps; i++) {
    await user.tab();
    const activeElement = document.activeElement as HTMLElement;
    if (activeElement && container.contains(activeElement)) {
      focusedElements.push(activeElement);
    }
  }

  return focusedElements;
}

/**
 * Test if element responds to Enter/Space key activation
 */
export async function testKeyboardActivation(
  element: HTMLElement,
  key: 'Enter' | ' ' = 'Enter'
): Promise<boolean> {
  const user = userEvent.setup();
  let activated = false;

  const handleClick = () => {
    activated = true;
  };

  element.addEventListener('click', handleClick);

  element.focus();
  await user.keyboard(`{${key}}`);

  element.removeEventListener('click', handleClick);

  return activated;
}

/**
 * Get all focusable elements within a container
 */
export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
    'audio[controls]',
    'video[controls]',
  ];

  const elements = container.querySelectorAll<HTMLElement>(
    focusableSelectors.join(',')
  );

  return Array.from(elements).filter((el) => {
    // Filter out hidden elements
    const style = window.getComputedStyle(el);
    return (
      style.display !== 'none' &&
      style.visibility !== 'hidden' &&
      !el.hasAttribute('hidden')
    );
  });
}

/**
 * Test focus trap (useful for modals)
 */
export async function testFocusTrap(
  container: HTMLElement,
  shouldTrap: boolean = true
): Promise<boolean> {
  const focusableElements = getFocusableElements(container);

  if (focusableElements.length === 0) {
    return false;
  }

  const user = userEvent.setup();

  // Focus first element
  focusableElements[0]?.focus();

  // Tab through all elements
  for (let i = 0; i < focusableElements.length + 2; i++) {
    await user.tab();
  }

  const activeElement = document.activeElement as HTMLElement;

  if (shouldTrap) {
    // Focus should stay within container
    return container.contains(activeElement);
  } else {
    // Focus should be able to leave container
    return !container.contains(activeElement);
  }
}

/**
 * Check if element has accessible name
 */
export function hasAccessibleName(element: HTMLElement): boolean {
  const ariaLabel = element.getAttribute('aria-label');
  const ariaLabelledBy = element.getAttribute('aria-labelledby');
  const title = element.getAttribute('title');

  // For form elements, check for associated label
  if (['INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName)) {
    const id = element.id;
    if (id) {
      const label = document.querySelector(`label[for="${id}"]`);
      if (label) {
        return true;
      }
    }
  }

  // For images, check alt text
  if (element.tagName === 'IMG') {
    return element.hasAttribute('alt');
  }

  // Check ARIA attributes or title
  return !!(ariaLabel || ariaLabelledBy || title || element.textContent?.trim());
}

/**
 * Get accessible name of an element
 */
export function getAccessibleName(element: HTMLElement): string {
  const ariaLabel = element.getAttribute('aria-label');
  if (ariaLabel) {
    return ariaLabel;
  }

  const ariaLabelledBy = element.getAttribute('aria-labelledby');
  if (ariaLabelledBy) {
    const labelElement = document.getElementById(ariaLabelledBy);
    if (labelElement) {
      return labelElement.textContent?.trim() || '';
    }
  }

  // For form elements, check for associated label
  if (['INPUT', 'SELECT', 'TEXTAREA'].includes(element.tagName)) {
    const id = element.id;
    if (id) {
      const label = document.querySelector(`label[for="${id}"]`);
      if (label) {
        return label.textContent?.trim() || '';
      }
    }
  }

  // For images, return alt text
  if (element.tagName === 'IMG') {
    return (element as HTMLImageElement).alt;
  }

  return element.textContent?.trim() || '';
}

/**
 * Test screen reader announcements (via live regions)
 */
export async function waitForAnnouncement(
  text: string,
  options?: { timeout?: number }
): Promise<boolean> {
  try {
    await waitFor(
      () => {
        const liveRegions = document.querySelectorAll('[aria-live]');
        for (const region of liveRegions) {
          if (region.textContent?.includes(text)) {
            return true;
          }
        }
        throw new Error('Announcement not found');
      },
      { timeout: options?.timeout || 3000 }
    );
    return true;
  } catch {
    return false;
  }
}

/**
 * Check color contrast ratio
 * Note: This is a simplified version. For production, use axe-core's color-contrast rule
 */
export function checkColorContrast(
  foreground: string,
  background: string,
  minimumRatio: number = 4.5
): boolean {
  // This would require a full color contrast calculation
  // For now, return true as a placeholder
  // In practice, use axe-core's color-contrast rule
  console.warn('checkColorContrast is a placeholder. Use axe-core for accurate testing.');
  return true;
}

/**
 * Test if element is hidden from screen readers
 */
export function isHiddenFromScreenReader(element: HTMLElement): boolean {
  const ariaHidden = element.getAttribute('aria-hidden') === 'true';
  const role = element.getAttribute('role');
  const rolePresentation = role === 'presentation' || role === 'none';

  return ariaHidden || rolePresentation;
}

/**
 * Find element by accessible name
 */
export function findByAccessibleName(
  container: HTMLElement,
  name: string
): HTMLElement | null {
  const elements = container.querySelectorAll<HTMLElement>('*');

  for (const element of elements) {
    if (getAccessibleName(element) === name) {
      return element;
    }
  }

  return null;
}

/**
 * Assert element has proper ARIA attributes for its role
 */
export function validateAriaForRole(
  element: HTMLElement,
  expectedRole: string
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  const role = element.getAttribute('role') || element.tagName.toLowerCase();

  if (role !== expectedRole && role !== expectedRole.toLowerCase()) {
    errors.push(`Expected role "${expectedRole}" but got "${role}"`);
  }

  // Check role-specific required attributes
  const roleRequirements: Record<string, string[]> = {
    button: [],
    checkbox: ['aria-checked'],
    radio: ['aria-checked'],
    textbox: [],
    combobox: ['aria-expanded'],
    tab: ['aria-selected'],
    tabpanel: ['aria-labelledby'],
    dialog: ['aria-labelledby', 'aria-modal'],
    alertdialog: ['aria-labelledby', 'aria-describedby'],
  };

  const required = roleRequirements[expectedRole] || [];
  for (const attr of required) {
    if (!element.hasAttribute(attr)) {
      errors.push(`Missing required attribute "${attr}" for role "${expectedRole}"`);
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Test landmark regions
 */
export function getLandmarkRegions(container: HTMLElement): {
  main: HTMLElement[];
  navigation: HTMLElement[];
  banner: HTMLElement[];
  contentinfo: HTMLElement[];
  complementary: HTMLElement[];
  search: HTMLElement[];
} {
  return {
    main: Array.from(container.querySelectorAll('main, [role="main"]')),
    navigation: Array.from(container.querySelectorAll('nav, [role="navigation"]')),
    banner: Array.from(container.querySelectorAll('header, [role="banner"]')),
    contentinfo: Array.from(container.querySelectorAll('footer, [role="contentinfo"]')),
    complementary: Array.from(container.querySelectorAll('aside, [role="complementary"]')),
    search: Array.from(container.querySelectorAll('[role="search"]')),
  };
}

/**
 * Validate heading hierarchy
 */
export function validateHeadingHierarchy(container: HTMLElement): {
  valid: boolean;
  errors: string[];
} {
  const headings = container.querySelectorAll<HTMLElement>('h1, h2, h3, h4, h5, h6, [role="heading"]');
  const errors: string[] = [];

  let previousLevel = 0;

  headings.forEach((heading, index) => {
    let level: number;

    if (heading.hasAttribute('role') && heading.getAttribute('role') === 'heading') {
      const ariaLevel = heading.getAttribute('aria-level');
      level = ariaLevel ? parseInt(ariaLevel, 10) : 2;
    } else {
      level = parseInt(heading.tagName.charAt(1), 10);
    }

    if (index === 0 && level !== 1) {
      errors.push('First heading should be h1');
    }

    if (level - previousLevel > 1) {
      errors.push(`Heading level skipped from h${previousLevel} to h${level}`);
    }

    previousLevel = level;
  });

  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Check if form has proper labels
 */
export function validateFormLabels(form: HTMLFormElement): {
  valid: boolean;
  unlabeledInputs: HTMLElement[];
} {
  const inputs = form.querySelectorAll<HTMLElement>('input, select, textarea');
  const unlabeledInputs: HTMLElement[] = [];

  inputs.forEach((input) => {
    if (!hasAccessibleName(input)) {
      unlabeledInputs.push(input);
    }
  });

  return {
    valid: unlabeledInputs.length === 0,
    unlabeledInputs,
  };
}

/**
 * Simulate screen reader navigation
 */
export async function simulateScreenReaderNavigation(
  container: HTMLElement,
  steps: number = 5
): Promise<string[]> {
  const announcements: string[] = [];
  const focusableElements = getFocusableElements(container);

  for (let i = 0; i < Math.min(steps, focusableElements.length); i++) {
    const element = focusableElements[i];
    if (element) {
      const name = getAccessibleName(element);
      const role = element.getAttribute('role') || element.tagName.toLowerCase();
      announcements.push(`${name} (${role})`);
    }
  }

  return announcements;
}
