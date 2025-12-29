/**
 * Table Structure Validator
 * Validates table accessibility and structure
 */

import type { ValidationResult, ValidationError, ValidationWarning, TableInfo } from '../types/index.js';

export class TableValidator {
  private errors: ValidationError[] = [];
  private warnings: ValidationWarning[] = [];

  /**
   * Validate table structure
   */
  validate(table: TableInfo): ValidationResult {
    this.errors = [];
    this.warnings = [];

    // Validate basic structure
    this.validateBasicStructure(table);

    // Validate headers
    this.validateHeaders(table);

    // Validate caption
    this.validateCaption(table);

    // Validate complexity
    this.validateComplexity(table);

    return {
      valid: this.errors.length === 0,
      errors: this.errors,
      warnings: this.warnings
    };
  }

  /**
   * Validate multiple tables
   */
  validateMultiple(tables: TableInfo[]): ValidationResult {
    this.errors = [];
    this.warnings = [];

    if (tables.length === 0) {
      return {
        valid: true,
        errors: [],
        warnings: []
      };
    }

    tables.forEach((table, index) => {
      const result = this.validate(table);

      // Prefix errors and warnings with table number
      result.errors.forEach(error => {
        this.addError(error.code, `Table ${index + 1}: ${error.message}`);
      });

      result.warnings.forEach(warning => {
        this.addWarning(warning.code, `Table ${index + 1}: ${warning.message}`);
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
  private validateBasicStructure(table: TableInfo): void {
    // Check for empty table
    if (table.rowCount === 0) {
      this.addError('TABLE_NO_ROWS', 'Table has no rows');
    }

    if (table.columnCount === 0) {
      this.addError('TABLE_NO_COLUMNS', 'Table has no columns');
    }

    // Check for minimum size
    if (table.rowCount === 1 && table.columnCount === 1) {
      this.addWarning('TABLE_SINGLE_CELL', 'Table has only one cell. Consider using a different element.');
    }

    // Check for very large tables
    if (table.rowCount > 100) {
      this.addWarning('TABLE_VERY_LARGE', `Table has ${table.rowCount} rows. Consider breaking into smaller tables.`);
    }

    if (table.columnCount > 20) {
      this.addWarning('TABLE_VERY_WIDE', `Table has ${table.columnCount} columns. Consider restructuring.`);
    }
  }

  /**
   * Validate table headers
   */
  private validateHeaders(table: TableInfo): void {
    if (!table.hasHeaders) {
      this.addError('TABLE_NO_HEADERS', 'Table has no header row. Data tables should have headers.');
    }

    // Check header cells
    if (table.hasHeaders && (!table.headerCells || table.headerCells.length === 0)) {
      this.addError('TABLE_HEADERS_EMPTY', 'Table header row appears to be empty');
    }

    // Validate header text
    if (table.headerCells) {
      table.headerCells.forEach((headerText, index) => {
        if (!headerText || headerText.trim().length === 0) {
          this.addWarning('TABLE_HEADER_EMPTY', `Header cell ${index + 1} is empty`);
        }

        // Check for generic headers
        if (headerText && /^(column|col|header|head)\s*\d*$/i.test(headerText.trim())) {
          this.addWarning('TABLE_HEADER_GENERIC', `Header "${headerText}" is generic. Use descriptive text.`);
        }
      });

      // Check for duplicate headers
      const duplicates = this.findDuplicates(table.headerCells.filter(h => h && h.trim().length > 0));
      if (duplicates.length > 0) {
        this.addWarning('TABLE_HEADERS_DUPLICATE', `Table has duplicate headers: ${duplicates.join(', ')}`);
      }
    }
  }

  /**
   * Validate table caption
   */
  private validateCaption(table: TableInfo): void {
    if (!table.hasCaption) {
      this.addWarning('TABLE_NO_CAPTION', 'Table should have a caption describing its contents');
    }

    if (table.hasCaption && table.caption) {
      if (table.caption.trim().length === 0) {
        this.addError('TABLE_CAPTION_EMPTY', 'Table caption is empty');
      }

      if (table.caption.length < 5) {
        this.addWarning('TABLE_CAPTION_TOO_SHORT', 'Table caption is very short');
      }

      // Check for generic captions
      const genericCaptions = ['table', 'data', 'information', 'results'];
      if (genericCaptions.includes(table.caption.toLowerCase().trim())) {
        this.addWarning('TABLE_CAPTION_GENERIC', `Caption "${table.caption}" is too generic. Be more specific.`);
      }
    }
  }

  /**
   * Validate table complexity
   */
  private validateComplexity(table: TableInfo): void {
    const isComplex = table.rowCount * table.columnCount > 100;

    if (isComplex) {
      this.addWarning(
        'TABLE_COMPLEX',
        'Complex table detected. Ensure proper header associations (scope or headers/id attributes).'
      );
    }

    // Check for nested tables (if applicable)
    // This would require analyzing actual table content
  }

  /**
   * Find duplicate values in array
   */
  private findDuplicates(arr: string[]): string[] {
    const seen = new Set<string>();
    const duplicates = new Set<string>();

    arr.forEach(item => {
      const normalized = item.toLowerCase().trim();
      if (seen.has(normalized)) {
        duplicates.add(item);
      }
      seen.add(normalized);
    });

    return Array.from(duplicates);
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
   * Generate table accessibility report
   */
  static generateReport(table: TableInfo): string {
    let report = 'Table Accessibility Report\n';
    report += '=========================\n\n';

    report += `Dimensions: ${table.rowCount} rows × ${table.columnCount} columns\n`;
    report += `Has Headers: ${table.hasHeaders ? 'Yes' : 'No'}\n`;
    report += `Has Caption: ${table.hasCaption ? 'Yes' : 'No'}\n`;

    if (table.caption) {
      report += `Caption: "${table.caption}"\n`;
    }

    if (table.headerCells && table.headerCells.length > 0) {
      report += `\nHeader Cells:\n`;
      table.headerCells.forEach((header, index) => {
        report += `  ${index + 1}. ${header || '(empty)'}\n`;
      });
    }

    report += `\nAccessibility Status: ${table.isAccessible ? 'Accessible' : 'Needs Improvement'}\n`;

    return report;
  }

  /**
   * Suggest table improvements
   */
  static suggestImprovements(table: TableInfo): string[] {
    const suggestions: string[] = [];

    if (!table.hasHeaders) {
      suggestions.push('Add a header row to identify column contents');
    }

    if (!table.hasCaption) {
      suggestions.push('Add a caption to describe the table\'s purpose');
    }

    if (table.rowCount > 50) {
      suggestions.push('Consider breaking this large table into smaller, topic-specific tables');
    }

    if (table.columnCount > 10) {
      suggestions.push('Consider restructuring this wide table or using a different layout');
    }

    if (!table.isAccessible) {
      suggestions.push('Review table structure to ensure proper header associations');
      suggestions.push('Use scope attribute (scope="col" or scope="row") for simple tables');
      suggestions.push('Use headers and id attributes for complex tables');
    }

    return suggestions;
  }
}
