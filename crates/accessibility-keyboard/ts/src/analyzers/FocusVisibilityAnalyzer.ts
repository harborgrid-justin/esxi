/**
 * Focus Visibility Analyzer
 * Analyzes focus indicators for visibility and WCAG compliance
 */

import { FocusIndicator } from '../types';

export class FocusVisibilityAnalyzer {
  private contrastThreshold: number;

  constructor(contrastThreshold: number = 3.0) {
    this.contrastThreshold = contrastThreshold;
  }

  /**
   * Analyzes all focusable elements for focus indicators
   */
  async analyzeAll(container: HTMLElement = document.body): Promise<FocusIndicator[]> {
    const focusableSelector = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(',');

    const elements = Array.from(
      container.querySelectorAll(focusableSelector)
    ) as HTMLElement[];

    const indicators: FocusIndicator[] = [];

    for (const element of elements) {
      const indicator = await this.analyze(element);
      indicators.push(indicator);
    }

    return indicators;
  }

  /**
   * Analyzes focus indicator for a single element
   */
  async analyze(element: HTMLElement): Promise<FocusIndicator> {
    // Temporarily focus the element to check focus styles
    const originalFocus = document.activeElement;
    element.focus();

    const focusedStyle = window.getComputedStyle(element);
    const hasOutline = this.hasOutline(focusedStyle);
    const hasCustomIndicator = this.hasCustomFocusIndicator(focusedStyle);

    const styles = {
      outline: focusedStyle.outline,
      border: focusedStyle.border,
      boxShadow: focusedStyle.boxShadow,
      backgroundColor: focusedStyle.backgroundColor,
    };

    const contrastRatio = this.calculateContrast(element, focusedStyle);
    const issues = this.getIssues(hasOutline, hasCustomIndicator, contrastRatio);
    const meetsWCAG = this.checkWCAGCompliance(
      hasOutline,
      hasCustomIndicator,
      contrastRatio
    );

    // Restore original focus
    if (originalFocus instanceof HTMLElement) {
      originalFocus.focus();
    } else {
      element.blur();
    }

    return {
      element,
      hasOutline,
      hasCustomIndicator,
      contrastRatio,
      meetsWCAG,
      styles,
      issues,
    };
  }

  /**
   * Checks if element has an outline
   */
  private hasOutline(style: CSSStyleDeclaration): boolean {
    return (
      style.outline !== 'none' &&
      style.outlineStyle !== 'none' &&
      style.outlineWidth !== '0px'
    );
  }

  /**
   * Checks if element has custom focus indicator
   */
  private hasCustomFocusIndicator(style: CSSStyleDeclaration): boolean {
    // Check for box-shadow
    if (style.boxShadow && style.boxShadow !== 'none') {
      return true;
    }

    // Check for border changes
    const borderWidth = parseInt(style.borderWidth, 10);
    if (borderWidth > 0) {
      return true;
    }

    // Check for background color
    if (style.backgroundColor && style.backgroundColor !== 'transparent') {
      return true;
    }

    return false;
  }

  /**
   * Calculates contrast ratio for focus indicator
   */
  private calculateContrast(
    element: HTMLElement,
    focusedStyle: CSSStyleDeclaration
  ): number | null {
    try {
      const backgroundColor = this.getBackgroundColor(element);
      const indicatorColor = this.getIndicatorColor(focusedStyle);

      if (!backgroundColor || !indicatorColor) {
        return null;
      }

      return this.getContrastRatio(backgroundColor, indicatorColor);
    } catch (error) {
      return null;
    }
  }

  /**
   * Gets the effective background color
   */
  private getBackgroundColor(element: HTMLElement): string | null {
    let current: HTMLElement | null = element;

    while (current) {
      const style = window.getComputedStyle(current);
      const bgColor = style.backgroundColor;

      if (bgColor && bgColor !== 'transparent' && bgColor !== 'rgba(0, 0, 0, 0)') {
        return bgColor;
      }

      current = current.parentElement;
    }

    return 'rgb(255, 255, 255)'; // Default to white
  }

  /**
   * Gets the indicator color (outline or box-shadow)
   */
  private getIndicatorColor(style: CSSStyleDeclaration): string | null {
    if (style.outlineColor && style.outlineColor !== 'transparent') {
      return style.outlineColor;
    }

    if (style.boxShadow && style.boxShadow !== 'none') {
      // Extract color from box-shadow
      const shadowMatch = style.boxShadow.match(/rgba?\([^)]+\)/);
      if (shadowMatch) {
        return shadowMatch[0];
      }
    }

    if (style.borderColor && style.borderColor !== 'transparent') {
      return style.borderColor;
    }

    return null;
  }

  /**
   * Calculates WCAG contrast ratio
   */
  private getContrastRatio(color1: string, color2: string): number {
    const lum1 = this.getLuminance(this.parseColor(color1));
    const lum2 = this.getLuminance(this.parseColor(color2));

    const lighter = Math.max(lum1, lum2);
    const darker = Math.min(lum1, lum2);

    return (lighter + 0.05) / (darker + 0.05);
  }

  /**
   * Parses CSS color to RGB values
   */
  private parseColor(color: string): [number, number, number] {
    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
    if (match) {
      return [parseInt(match[1]), parseInt(match[2]), parseInt(match[3])];
    }
    return [0, 0, 0];
  }

  /**
   * Calculates relative luminance
   */
  private getLuminance([r, g, b]: [number, number, number]): number {
    const [rs, gs, bs] = [r, g, b].map((val) => {
      const sRGB = val / 255;
      return sRGB <= 0.03928 ? sRGB / 12.92 : Math.pow((sRGB + 0.055) / 1.055, 2.4);
    });

    return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
  }

  /**
   * Gets list of issues
   */
  private getIssues(
    hasOutline: boolean,
    hasCustomIndicator: boolean,
    contrastRatio: number | null
  ): string[] {
    const issues: string[] = [];

    if (!hasOutline && !hasCustomIndicator) {
      issues.push('No visible focus indicator detected');
    }

    if (contrastRatio !== null && contrastRatio < this.contrastThreshold) {
      issues.push(
        `Contrast ratio ${contrastRatio.toFixed(2)}:1 is below recommended ${this.contrastThreshold}:1`
      );
    }

    if (hasOutline) {
      const userAgent = navigator.userAgent.toLowerCase();
      if (userAgent.includes('chrome') || userAgent.includes('safari')) {
        // Chrome/Safari default outline might be removed
        issues.push('Using browser default outline - consider custom focus indicator');
      }
    }

    return issues;
  }

  /**
   * Checks WCAG compliance
   */
  private checkWCAGCompliance(
    hasOutline: boolean,
    hasCustomIndicator: boolean,
    contrastRatio: number | null
  ): boolean {
    // Must have some visible indicator
    if (!hasOutline && !hasCustomIndicator) {
      return false;
    }

    // If we can calculate contrast, it must meet threshold
    if (contrastRatio !== null && contrastRatio < this.contrastThreshold) {
      return false;
    }

    return true;
  }
}
