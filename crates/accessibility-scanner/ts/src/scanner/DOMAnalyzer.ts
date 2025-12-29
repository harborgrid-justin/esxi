/**
 * DOM Analyzer - Advanced DOM analysis utilities
 */

import * as cheerio from 'cheerio';
import { Issue, Severity, IssueContext } from '../types';
import { getRuleEngine } from './RuleEngine';

export interface ColorRGB {
  r: number;
  g: number;
  b: number;
}

/**
 * DOM Analyzer for accessibility checks
 */
export class DOMAnalyzer {
  private $: cheerio.CheerioAPI;
  private ruleEngine = getRuleEngine();

  constructor(html: string) {
    this.$ = cheerio.load(html);
  }

  /**
   * Get all elements matching a selector
   */
  select(selector: string): cheerio.Cheerio<cheerio.Element> {
    return this.$(selector);
  }

  /**
   * Get element text content
   */
  getText(element: cheerio.Cheerio<cheerio.Element>): string {
    return element.text().trim();
  }

  /**
   * Get element HTML
   */
  getHTML(element: cheerio.Cheerio<cheerio.Element>): string {
    return element.html() || '';
  }

  /**
   * Get element outer HTML
   */
  getOuterHTML(element: cheerio.Cheerio<cheerio.Element>): string {
    return cheerio.html(element);
  }

  /**
   * Get element attributes
   */
  getAttributes(element: cheerio.Cheerio<cheerio.Element>): Record<string, string> {
    const attrs: Record<string, string> = {};
    const attribs = element.attr();
    if (attribs) {
      Object.assign(attrs, attribs);
    }
    return attrs;
  }

  /**
   * Get element selector
   */
  getSelector(element: cheerio.Cheerio<cheerio.Element>): string {
    const tagName = element.prop('tagName')?.toLowerCase() || 'unknown';
    const id = element.attr('id');
    const className = element.attr('class')?.split(' ')[0];

    if (id) {
      return `${tagName}#${id}`;
    } else if (className) {
      return `${tagName}.${className}`;
    }
    return tagName;
  }

  /**
   * Check for missing image alt text
   */
  checkImageAlt(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('image-alt');
    if (!rule) return issues;

    this.$('img').each((_, elem) => {
      const $elem = this.$(elem);
      const hasAlt = $elem.attr('alt') !== undefined;
      const role = $elem.attr('role');
      const ariaHidden = $elem.attr('aria-hidden');

      if (!hasAlt && role !== 'presentation' && ariaHidden !== 'true') {
        const context = this.createContext($elem);
        issues.push(
          this.ruleEngine.createIssue(
            rule,
            Severity.Critical,
            'Image is missing alt attribute',
            context,
            [
              'Add an alt attribute describing the image',
              'If decorative, use alt="" or role="presentation"',
            ],
            'Images without alt text are inaccessible to screen reader users'
          )
        );
      }
    });

    return issues;
  }

  /**
   * Check for form labels
   */
  checkFormLabels(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('label');
    if (!rule) return issues;

    this.$('input, select, textarea').each((_, elem) => {
      const $elem = this.$(elem);
      const type = $elem.attr('type') || 'text';

      // Skip hidden and button inputs
      if (['hidden', 'submit', 'button'].includes(type)) {
        return;
      }

      const id = $elem.attr('id');
      const hasLabel = id ? this.$(`label[for="${id}"]`).length > 0 : false;
      const hasAriaLabel = $elem.attr('aria-label') || $elem.attr('aria-labelledby');

      if (!hasLabel && !hasAriaLabel) {
        const context = this.createContext($elem);
        issues.push(
          this.ruleEngine.createIssue(
            rule,
            Severity.Critical,
            'Form input is missing a label',
            context,
            [
              'Add a label element with for attribute matching input id',
              'Or add aria-label or aria-labelledby attribute',
            ],
            'Form inputs without labels are difficult for screen reader users to understand'
          )
        );
      }
    });

    return issues;
  }

  /**
   * Check document title
   */
  checkDocumentTitle(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('document-title');
    if (!rule) return issues;

    const $title = this.$('title');

    if ($title.length === 0) {
      const $html = this.$('html');
      const context = this.createContext($html);
      issues.push(
        this.ruleEngine.createIssue(
          rule,
          Severity.Serious,
          'Page is missing a title element',
          context,
          ['Add a <title> element in the <head> section'],
          'Pages without titles are difficult to identify in browser tabs and search results'
        )
      );
    } else {
      const titleText = this.getText($title);
      if (!titleText) {
        const context = this.createContext($title);
        issues.push(
          this.ruleEngine.createIssue(
            rule,
            Severity.Serious,
            'Page title is empty',
            context,
            ['Add descriptive text to the title element'],
            'Empty page titles provide no context about the page content'
          )
        );
      }
    }

    return issues;
  }

  /**
   * Check HTML lang attribute
   */
  checkHtmlLang(): Issue[] {
    const issues: Issue[] = [];
    const $html = this.$('html');

    const hasLangRule = this.ruleEngine.getRuleById('html-has-lang');
    const validLangRule = this.ruleEngine.getRuleById('html-lang-valid');

    if (hasLangRule && !$html.attr('lang')) {
      const context = this.createContext($html);
      issues.push(
        this.ruleEngine.createIssue(
          hasLangRule,
          Severity.Serious,
          'HTML element is missing lang attribute',
          context,
          ['Add lang attribute to html element (e.g., lang="en")'],
          'Missing language declaration prevents screen readers from using correct pronunciation'
        )
      );
    } else if (validLangRule) {
      const lang = $html.attr('lang');
      if (lang && !this.isValidLangCode(lang)) {
        const context = this.createContext($html);
        issues.push(
          this.ruleEngine.createIssue(
            validLangRule,
            Severity.Serious,
            `HTML lang attribute has invalid value: ${lang}`,
            context,
            ['Use a valid ISO language code (e.g., "en", "es", "fr")'],
            'Invalid language codes prevent proper text-to-speech synthesis'
          )
        );
      }
    }

    return issues;
  }

  /**
   * Check heading structure
   */
  checkHeadingStructure(): Issue[] {
    const issues: Issue[] = [];
    const orderRule = this.ruleEngine.getRuleById('heading-order');
    const emptyRule = this.ruleEngine.getRuleById('empty-heading');

    let lastLevel: number | null = null;

    this.$('h1, h2, h3, h4, h5, h6').each((_, elem) => {
      const $elem = this.$(elem);
      const tagName = $elem.prop('tagName');
      const currentLevel = parseInt(tagName?.charAt(1) || '0', 10);

      // Check for empty headings
      if (emptyRule) {
        const text = this.getText($elem);
        if (!text) {
          const context = this.createContext($elem);
          issues.push(
            this.ruleEngine.createIssue(
              emptyRule,
              Severity.Serious,
              'Heading is empty',
              context,
              ['Add descriptive text to the heading'],
              'Empty headings provide no navigation context for screen reader users'
            )
          );
        }
      }

      // Check heading order
      if (orderRule && lastLevel !== null) {
        if (currentLevel > lastLevel + 1) {
          const context = this.createContext($elem);
          issues.push(
            this.ruleEngine.createIssue(
              orderRule,
              Severity.Moderate,
              `Heading levels skipped from h${lastLevel} to h${currentLevel}`,
              context,
              [
                `Use h${lastLevel + 1} instead of h${currentLevel} here`,
                'Maintain sequential heading hierarchy',
              ],
              'Skipped heading levels confuse screen reader users navigating by headings'
            )
          );
        }
      }

      lastLevel = currentLevel;
    });

    return issues;
  }

  /**
   * Check for duplicate IDs
   */
  checkDuplicateIds(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('duplicate-id');
    if (!rule) return issues;

    const idMap = new Map<string, cheerio.Cheerio<cheerio.Element>[]>();

    this.$('[id]').each((_, elem) => {
      const $elem = this.$(elem);
      const id = $elem.attr('id');
      if (id) {
        if (!idMap.has(id)) {
          idMap.set(id, []);
        }
        idMap.get(id)!.push($elem);
      }
    });

    idMap.forEach((elements, id) => {
      if (elements.length > 1) {
        elements.forEach($elem => {
          const context = this.createContext($elem);
          issues.push(
            this.ruleEngine.createIssue(
              rule,
              Severity.Critical,
              `Duplicate ID: ${id}`,
              context,
              ['Ensure each ID is unique within the page'],
              'Duplicate IDs break ARIA relationships and form label associations'
            )
          );
        });
      }
    });

    return issues;
  }

  /**
   * Check link text
   */
  checkLinkText(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('link-name');
    if (!rule) return issues;

    this.$('a[href]').each((_, elem) => {
      const $elem = this.$(elem);
      const text = this.getText($elem);
      const ariaLabel = $elem.attr('aria-label');
      const ariaLabelledby = $elem.attr('aria-labelledby');

      if (!text && !ariaLabel && !ariaLabelledby) {
        const context = this.createContext($elem);
        issues.push(
          this.ruleEngine.createIssue(
            rule,
            Severity.Serious,
            'Link has no accessible text',
            context,
            ['Add text content to the link', 'Or add aria-label attribute'],
            'Links without text cannot be understood by screen reader users'
          )
        );
      }
    });

    return issues;
  }

  /**
   * Check button text
   */
  checkButtonText(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('button-name');
    if (!rule) return issues;

    this.$('button').each((_, elem) => {
      const $elem = this.$(elem);
      const text = this.getText($elem);
      const ariaLabel = $elem.attr('aria-label');
      const ariaLabelledby = $elem.attr('aria-labelledby');

      if (!text && !ariaLabel && !ariaLabelledby) {
        const context = this.createContext($elem);
        issues.push(
          this.ruleEngine.createIssue(
            rule,
            Severity.Critical,
            'Button has no accessible text',
            context,
            ['Add text content to the button', 'Or add aria-label attribute'],
            'Buttons without text cannot be understood or activated by screen reader users'
          )
        );
      }
    });

    return issues;
  }

  /**
   * Analyze color contrast (simplified version)
   */
  analyzeColorContrast(): Issue[] {
    const issues: Issue[] = [];
    const rule = this.ruleEngine.getRuleById('color-contrast');
    if (!rule) return issues;

    // Note: Full color contrast checking requires computed styles
    // This is a simplified version that checks inline styles

    this.$('p, h1, h2, h3, h4, h5, h6, span, div, a, button, label, li, td, th').each((_, elem) => {
      const $elem = this.$(elem);
      const style = $elem.attr('style');

      if (style) {
        const colors = this.extractColorsFromStyle(style);
        if (colors.foreground && colors.background) {
          const ratio = this.calculateContrastRatio(colors.foreground, colors.background);
          const requiredRatio = 4.5; // WCAG AA for normal text

          if (ratio < requiredRatio) {
            const context = this.createContext($elem);
            issues.push(
              this.ruleEngine.createIssue(
                rule,
                Severity.Serious,
                `Insufficient color contrast: ${ratio.toFixed(2)}:1 (minimum ${requiredRatio}:1 required)`,
                context,
                [
                  `Increase contrast to at least ${requiredRatio}:1`,
                  'Use a color contrast checker tool to verify colors',
                ],
                'Low contrast makes text difficult to read for users with low vision'
              )
            );
          }
        }
      }
    });

    return issues;
  }

  /**
   * Create issue context from element
   */
  private createContext(element: cheerio.Cheerio<cheerio.Element>): IssueContext {
    return this.ruleEngine.createContext(
      this.getText(element),
      this.getOuterHTML(element),
      this.getSelector(element),
      this.getAttributes(element)
    );
  }

  /**
   * Validate language code
   */
  private isValidLangCode(lang: string): boolean {
    // Simplified language code validation
    return /^[a-z]{2,3}(-[A-Z]{2})?$/.test(lang);
  }

  /**
   * Extract colors from inline style
   */
  private extractColorsFromStyle(style: string): {
    foreground?: ColorRGB;
    background?: ColorRGB;
  } {
    const result: { foreground?: ColorRGB; background?: ColorRGB } = {};

    const declarations = style.split(';');
    for (const decl of declarations) {
      const [property, value] = decl.split(':').map(s => s.trim());
      if (property === 'color') {
        result.foreground = this.parseColor(value);
      } else if (property === 'background-color' || property === 'background') {
        result.background = this.parseColor(value);
      }
    }

    return result;
  }

  /**
   * Parse CSS color value
   */
  private parseColor(value: string): ColorRGB | undefined {
    // Handle hex colors
    if (value.startsWith('#')) {
      const hex = value.slice(1);
      if (hex.length === 6) {
        return {
          r: parseInt(hex.slice(0, 2), 16) / 255,
          g: parseInt(hex.slice(2, 4), 16) / 255,
          b: parseInt(hex.slice(4, 6), 16) / 255,
        };
      }
    }

    // Handle rgb() colors
    if (value.startsWith('rgb(')) {
      const rgb = value.slice(4, -1).split(',').map(s => parseInt(s.trim(), 10));
      if (rgb.length === 3) {
        return {
          r: rgb[0] / 255,
          g: rgb[1] / 255,
          b: rgb[2] / 255,
        };
      }
    }

    // Handle named colors (simplified)
    const namedColors: Record<string, ColorRGB> = {
      black: { r: 0, g: 0, b: 0 },
      white: { r: 1, g: 1, b: 1 },
      red: { r: 1, g: 0, b: 0 },
      green: { r: 0, g: 0.5, b: 0 },
      blue: { r: 0, g: 0, b: 1 },
    };

    return namedColors[value];
  }

  /**
   * Calculate relative luminance
   */
  private relativeLuminance(color: ColorRGB): number {
    const r = color.r <= 0.03928 ? color.r / 12.92 : Math.pow((color.r + 0.055) / 1.055, 2.4);
    const g = color.g <= 0.03928 ? color.g / 12.92 : Math.pow((color.g + 0.055) / 1.055, 2.4);
    const b = color.b <= 0.03928 ? color.b / 12.92 : Math.pow((color.b + 0.055) / 1.055, 2.4);

    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  }

  /**
   * Calculate contrast ratio
   */
  private calculateContrastRatio(color1: ColorRGB, color2: ColorRGB): number {
    const l1 = this.relativeLuminance(color1);
    const l2 = this.relativeLuminance(color2);

    const lighter = Math.max(l1, l2);
    const darker = Math.min(l1, l2);

    return (lighter + 0.05) / (darker + 0.05);
  }
}
