/**
 * List Structure Validator
 * Validates list accessibility and proper structure
 */

import type { ValidationResult, ValidationError, ValidationWarning, ListInfo } from '../types/index.js';

export class ListValidator {
  private errors: ValidationError[] = [];
  private warnings: ValidationWarning[] = [];

  /**
   * Validate list structure
   */
  validate(list: ListInfo): ValidationResult {
    this.errors = [];
    this.warnings = [];

    // Validate basic structure
    this.validateBasicStructure(list);

    // Validate nesting
    this.validateNesting(list);

    // Validate list type
    this.validateListType(list);

    return {
      valid: this.errors.length === 0,
      errors: this.errors,
      warnings: this.warnings
    };
  }

  /**
   * Validate multiple lists
   */
  validateMultiple(lists: ListInfo[]): ValidationResult {
    this.errors = [];
    this.warnings = [];

    if (lists.length === 0) {
      return {
        valid: true,
        errors: [],
        warnings: []
      };
    }

    lists.forEach((list, index) => {
      const result = this.validate(list);

      // Prefix errors and warnings with list number
      result.errors.forEach(error => {
        this.addError(error.code, `List ${index + 1}: ${error.message}`);
      });

      result.warnings.forEach(warning => {
        this.addWarning(warning.code, `List ${index + 1}: ${warning.message}`);
      });
    });

    return {
      valid: this.errors.length === 0,
      errors: this.errors,
      warnings: this.warnings
    };
  }

  /**
   * Validate basic structure
   */
  private validateBasicStructure(list: ListInfo): void {
    // Check for empty list
    if (list.itemCount === 0) {
      this.addError('LIST_EMPTY', 'List has no items');
    }

    // Check for single item list
    if (list.itemCount === 1) {
      this.addWarning('LIST_SINGLE_ITEM', 'List has only one item. Consider if a list is appropriate.');
    }

    // Check for very long lists
    if (list.itemCount > 50) {
      this.addWarning('LIST_VERY_LONG', `List has ${list.itemCount} items. Consider breaking into multiple lists or using subheadings.`);
    }

    // Check accessibility flag
    if (!list.isAccessible) {
      this.addError('LIST_NOT_ACCESSIBLE', 'List structure has accessibility issues');
    }
  }

  /**
   * Validate nesting
   */
  private validateNesting(list: ListInfo): void {
    if (list.nested) {
      // Nested lists should be properly structured
      // In HTML: nested lists should be children of <li> elements
      // In PDF: nested L elements should be properly tagged
      this.addWarning('LIST_NESTED', 'List contains nested lists. Ensure proper nesting structure.');
    }
  }

  /**
   * Validate list type
   */
  private validateListType(list: ListInfo): void {
    // Check if ordered/unordered is appropriate
    if (list.type === 'ordered' && list.itemCount > 20) {
      this.addWarning('ORDERED_LIST_LONG', 'Very long ordered list. Consider if numbered items are necessary.');
    }

    // Suggest ordered list for sequential items
    // This would require content analysis
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
   * Determine if content should be a list
   */
  static shouldBeList(content: string): boolean {
    // Check for common list patterns in plain text
    const listPatterns = [
      /^[-*•]\s+.+$/m, // Bullet points
      /^\d+\.\s+.+$/m, // Numbered items
      /^[a-z]\.\s+.+$/mi, // Lettered items
      /^[ivx]+\.\s+.+$/mi // Roman numerals
    ];

    return listPatterns.some(pattern => pattern.test(content));
  }

  /**
   * Suggest list improvements
   */
  static suggestImprovements(list: ListInfo): string[] {
    const suggestions: string[] = [];

    if (list.itemCount === 0) {
      suggestions.push('Remove empty list or add list items');
    }

    if (list.itemCount === 1) {
      suggestions.push('Consider if a single-item list is necessary');
    }

    if (list.itemCount > 50) {
      suggestions.push('Break long list into multiple shorter lists');
      suggestions.push('Add subheadings to organize list items');
      suggestions.push('Consider using a table if items have multiple attributes');
    }

    if (list.nested) {
      suggestions.push('Ensure nested lists are properly structured as children of list items');
      suggestions.push('Keep nesting to 2-3 levels maximum for readability');
    }

    if (list.type === 'unordered') {
      suggestions.push('Use ordered lists if sequence or priority matters');
    }

    if (list.type === 'ordered' && list.itemCount > 20) {
      suggestions.push('Consider if numbered sequence is meaningful for this many items');
    }

    if (!list.isAccessible) {
      suggestions.push('Use proper list markup (<ul>, <ol>, <li> in HTML; L, LI in PDF)');
      suggestions.push('Ensure list items are direct children of list elements');
      suggestions.push('Avoid manual numbering or bullets with plain text');
    }

    return suggestions;
  }

  /**
   * Generate list accessibility report
   */
  static generateReport(list: ListInfo): string {
    let report = 'List Accessibility Report\n';
    report += '========================\n\n';

    report += `Type: ${list.type === 'ordered' ? 'Ordered (numbered)' : 'Unordered (bulleted)'}\n`;
    report += `Items: ${list.itemCount}\n`;
    report += `Nested: ${list.nested ? 'Yes' : 'No'}\n`;
    report += `Accessible: ${list.isAccessible ? 'Yes' : 'No'}\n`;

    if (list.page) {
      report += `Page: ${list.page}\n`;
    }

    return report;
  }
}
