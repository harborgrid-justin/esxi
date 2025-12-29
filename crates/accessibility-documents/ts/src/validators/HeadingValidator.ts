/**
 * Heading Structure Validator
 * Validates heading hierarchy and accessibility
 */

import type { ValidationResult, ValidationError, ValidationWarning, HeadingInfo } from '../types/index.js';

export class HeadingValidator {
  private errors: ValidationError[] = [];
  private warnings: ValidationWarning[] = [];

  /**
   * Validate heading structure
   */
  validate(headings: HeadingInfo[]): ValidationResult {
    this.errors = [];
    this.warnings = [];

    if (headings.length === 0) {
      this.addError('NO_HEADINGS', 'Document has no headings');
      return {
        valid: false,
        errors: this.errors,
        warnings: this.warnings
      };
    }

    // Validate first heading
    this.validateFirstHeading(headings[0]);

    // Validate hierarchy
    this.validateHierarchy(headings);

    // Validate individual headings
    headings.forEach((heading, index) => {
      this.validateHeading(heading, index);
    });

    // Check for multiple H1s
    this.validateSingleH1(headings);

    // Check for empty headings
    this.validateNoEmptyHeadings(headings);

    return {
      valid: this.errors.length === 0,
      errors: this.errors,
      warnings: this.warnings
    };
  }

  /**
   * Validate first heading
   */
  private validateFirstHeading(heading: HeadingInfo): void {
    if (heading.level !== 1) {
      this.addError('FIRST_HEADING_NOT_H1', `Document should start with H1, found H${heading.level}`);
    }
  }

  /**
   * Validate heading hierarchy
   */
  private validateHierarchy(headings: HeadingInfo[]): void {
    for (let i = 1; i < headings.length; i++) {
      const previous = headings[i - 1];
      const current = headings[i];

      // Check for skipped levels
      if (current.level > previous.level + 1) {
        this.addError(
          'HEADING_LEVEL_SKIPPED',
          `Heading level skipped from H${previous.level} to H${current.level} at "${current.text.substring(0, 50)}"`
        );
      }

      // Warn if jumping back more than one level
      if (current.level < previous.level - 1) {
        this.addWarning(
          'HEADING_JUMP_BACK',
          `Heading jumps back multiple levels from H${previous.level} to H${current.level} at "${current.text.substring(0, 50)}"`
        );
      }
    }
  }

  /**
   * Validate individual heading
   */
  private validateHeading(heading: HeadingInfo, index: number): void {
    // Check for empty heading
    if (!heading.text || heading.text.trim().length === 0) {
      this.addError('HEADING_EMPTY', `Heading ${index + 1} (H${heading.level}) is empty`);
    }

    // Check heading length
    if (heading.text && heading.text.length > 150) {
      this.addWarning(
        'HEADING_TOO_LONG',
        `Heading ${index + 1} is very long (${heading.text.length} characters). Consider shortening.`
      );
    }

    // Check for generic headings
    const genericHeadings = [
      'untitled',
      'heading',
      'section',
      'chapter',
      'part',
      'introduction',
      'conclusion',
      'more',
      'click here'
    ];

    const lower = heading.text?.toLowerCase().trim();
    if (lower && genericHeadings.includes(lower)) {
      this.addWarning(
        'HEADING_GENERIC',
        `Heading "${heading.text}" is generic. Use more descriptive text.`
      );
    }

    // Check for all caps
    if (heading.text && heading.text === heading.text.toUpperCase() && heading.text.length > 5) {
      this.addWarning(
        'HEADING_ALL_CAPS',
        `Heading "${heading.text.substring(0, 50)}" is in all caps. Use sentence or title case.`
      );
    }

    // Check for numbers only
    if (heading.text && /^\d+\.?\s*$/.test(heading.text)) {
      this.addWarning(
        'HEADING_NUMBERS_ONLY',
        `Heading "${heading.text}" contains only numbers. Add descriptive text.`
      );
    }

    // Check for proper punctuation
    if (heading.text && heading.text.endsWith('.') && heading.level <= 2) {
      this.addWarning(
        'HEADING_ENDS_PERIOD',
        'High-level headings (H1-H2) typically should not end with a period.'
      );
    }
  }

  /**
   * Validate single H1
   */
  private validateSingleH1(headings: HeadingInfo[]): void {
    const h1Count = headings.filter(h => h.level === 1).length;

    if (h1Count === 0) {
      this.addError('NO_H1', 'Document has no H1 heading');
    } else if (h1Count > 1) {
      this.addWarning('MULTIPLE_H1', `Document has ${h1Count} H1 headings. Best practice is to have only one.`);
    }
  }

  /**
   * Validate no empty headings
   */
  private validateNoEmptyHeadings(headings: HeadingInfo[]): void {
    const emptyCount = headings.filter(h => !h.text || h.text.trim().length === 0).length;

    if (emptyCount > 0) {
      this.addError('HAS_EMPTY_HEADINGS', `Document has ${emptyCount} empty heading(s)`);
    }
  }

  /**
   * Add validation error
   */
  private addError(code: string, message: string): void {
    this.errors.push({ code, message });
  }

  /**
   * Add validation warning
   */
  private addWarning(code: string, message: string): void {
    this.warnings.push({ code, message });
  }

  /**
   * Generate heading outline
   */
  static generateOutline(headings: HeadingInfo[]): string {
    let outline = '';

    headings.forEach(heading => {
      const indent = '  '.repeat(heading.level - 1);
      const text = heading.text.substring(0, 80);
      outline += `${indent}H${heading.level}: ${text}\n`;
    });

    return outline;
  }

  /**
   * Suggest heading improvements
   */
  static suggestImprovements(headings: HeadingInfo[]): string[] {
    const suggestions: string[] = [];

    if (headings.length === 0) {
      suggestions.push('Add headings to structure your document');
      return suggestions;
    }

    const h1Count = headings.filter(h => h.level === 1).length;
    if (h1Count === 0) {
      suggestions.push('Add an H1 heading at the beginning of the document');
    } else if (h1Count > 1) {
      suggestions.push('Consider using only one H1 heading for the main document title');
    }

    // Check for heading gaps
    const levels = [...new Set(headings.map(h => h.level))].sort();
    for (let i = 1; i < levels.length; i++) {
      if (levels[i] > levels[i - 1] + 1) {
        suggestions.push(`Heading levels skip from H${levels[i - 1]} to H${levels[i]}. Fill in the gap.`);
      }
    }

    // Check for very long sections without subheadings
    let currentLevel = 1;
    let countAtLevel = 0;

    headings.forEach(heading => {
      if (heading.level === currentLevel) {
        countAtLevel++;
      } else if (heading.level > currentLevel) {
        currentLevel = heading.level;
        countAtLevel = 1;
      } else {
        currentLevel = heading.level;
        countAtLevel = 1;
      }
    });

    return suggestions;
  }
}
