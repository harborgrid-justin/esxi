/**
 * Skip Link Analyzer
 * Analyzes skip navigation links for accessibility
 */

import { SkipLink } from '../types';

export class SkipLinkAnalyzer {
  /**
   * Analyzes skip links in the container
   */
  async analyze(container: HTMLElement = document.body): Promise<SkipLink[]> {
    const skipLinks: SkipLink[] = [];

    // Find potential skip links
    const skipLinkSelectors = [
      'a[href^="#"]:first-of-type',
      'a[href^="#main"]',
      'a[href^="#content"]',
      'a[href^="#skip"]',
      '.skip-link',
      '.skip-to-content',
      '[class*="skip"]',
    ];

    const potentialSkipLinks = new Set<HTMLElement>();

    for (const selector of skipLinkSelectors) {
      const elements = Array.from(container.querySelectorAll(selector)) as HTMLElement[];
      elements.forEach((el) => potentialSkipLinks.add(el));
    }

    // Also check first few links in document
    const firstLinks = Array.from(container.querySelectorAll('a[href^="#"]'))
      .slice(0, 3)
      .filter((link) => {
        const text = link.textContent?.toLowerCase() || '';
        return (
          text.includes('skip') ||
          text.includes('jump') ||
          text.includes('main') ||
          text.includes('content')
        );
      }) as HTMLElement[];

    firstLinks.forEach((el) => potentialSkipLinks.add(el));

    // Analyze each potential skip link
    for (const element of potentialSkipLinks) {
      const skipLink = this.analyzeSkipLink(element);
      if (skipLink) {
        skipLinks.push(skipLink);
      }
    }

    return skipLinks;
  }

  /**
   * Analyzes a single skip link
   */
  private analyzeSkipLink(element: HTMLElement): SkipLink | null {
    if (element.tagName.toLowerCase() !== 'a') {
      return null;
    }

    const href = element.getAttribute('href');
    if (!href || !href.startsWith('#')) {
      return null;
    }

    const target = href;
    const targetElement = document.querySelector(target);
    const targetExists = targetElement !== null;

    const isVisible = this.checkVisibility(element);
    const isFirstFocusable = this.isFirstFocusableElement(element);
    const issues: string[] = [];

    // Check if target exists
    if (!targetExists) {
      issues.push(`Target element "${target}" does not exist in the document`);
    }

    // Check if link is visible or becomes visible on focus
    if (!isVisible) {
      const visibleOnFocus = this.checkVisibilityOnFocus(element);
      if (!visibleOnFocus) {
        issues.push('Skip link is not visible and does not become visible on focus');
      }
    }

    // Check if it's the first focusable element
    if (!isFirstFocusable) {
      issues.push(
        'Skip link is not the first focusable element - users must tab past other elements'
      );
    }

    // Check if target is focusable
    if (targetExists && targetElement) {
      const targetIsFocusable = this.isElementFocusable(targetElement as HTMLElement);
      if (!targetIsFocusable) {
        issues.push('Target element is not focusable - add tabindex="-1" to target');
      }
    }

    // Check link text
    const linkText = element.textContent?.trim() || '';
    if (linkText.length === 0) {
      issues.push('Skip link has no text content');
    } else if (linkText.length > 100) {
      issues.push('Skip link text is too long');
    }

    const worksCorrectly = issues.length === 0;

    return {
      element,
      target,
      targetExists,
      isVisible: isVisible || this.checkVisibilityOnFocus(element),
      isFirstFocusable,
      worksCorrectly,
      issues,
    };
  }

  /**
   * Checks if element is visible
   */
  private checkVisibility(element: HTMLElement): boolean {
    const style = window.getComputedStyle(element);
    const rect = element.getBoundingClientRect();

    return (
      style.display !== 'none' &&
      style.visibility !== 'hidden' &&
      style.opacity !== '0' &&
      rect.width > 0 &&
      rect.height > 0 &&
      !(rect.left < 0 && rect.top < 0) // Not positioned off-screen
    );
  }

  /**
   * Checks if element becomes visible on focus
   */
  private checkVisibilityOnFocus(element: HTMLElement): boolean {
    // Check for common CSS patterns that show on focus
    const classes = element.className.split(' ');
    const hasSkipClass = classes.some(
      (cls) =>
        cls.includes('skip') ||
        cls.includes('sr-only') ||
        cls.includes('visually-hidden')
    );

    if (hasSkipClass) {
      // Likely has focus styles that make it visible
      return true;
    }

    // Check computed styles
    const style = window.getComputedStyle(element);

    // Common pattern: positioned off-screen
    if (
      style.position === 'absolute' &&
      (parseInt(style.left, 10) < 0 || parseInt(style.top, 10) < 0)
    ) {
      return true; // Assume it becomes visible on focus
    }

    return false;
  }

  /**
   * Checks if element is the first focusable element
   */
  private isFirstFocusableElement(element: HTMLElement): boolean {
    const focusableSelector = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(',');

    const allFocusable = Array.from(
      document.querySelectorAll(focusableSelector)
    ) as HTMLElement[];

    // Filter to only visible or skip-link-style elements
    const effectivelyFocusable = allFocusable.filter(
      (el) => this.checkVisibility(el) || this.checkVisibilityOnFocus(el)
    );

    return effectivelyFocusable.length > 0 && effectivelyFocusable[0] === element;
  }

  /**
   * Checks if element is focusable
   */
  private isElementFocusable(element: HTMLElement): boolean {
    const tabindex = element.getAttribute('tabindex');
    if (tabindex !== null) {
      return parseInt(tabindex, 10) >= -1;
    }

    const tag = element.tagName.toLowerCase();
    const focusableTags = ['a', 'button', 'input', 'select', 'textarea'];

    return focusableTags.includes(tag);
  }
}
