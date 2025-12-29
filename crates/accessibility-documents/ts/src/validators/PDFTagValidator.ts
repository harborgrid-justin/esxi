/**
 * PDF Tag Structure Validator
 * Validates PDF tag structure according to PDF/UA-1 requirements
 */

import type {
  TagStructure,
  ValidationResult,
  ValidationError,
  ValidationWarning
} from '../types/index.js';

export class PDFTagValidator {
  private errors: ValidationError[] = [];
  private warnings: ValidationWarning[] = [];

  /**
   * Validate PDF tag structure
   */
  validate(structure: TagStructure): ValidationResult {
    this.errors = [];
    this.warnings = [];

    // Validate root structure
    this.validateRoot(structure);

    // Recursively validate children
    this.validateStructure(structure, 0);

    return {
      valid: this.errors.length === 0,
      errors: this.errors,
      warnings: this.warnings
    };
  }

  /**
   * Validate root element
   */
  private validateRoot(structure: TagStructure): void {
    // Root should be Document, Part, or Art
    const validRootTypes = ['Document', 'Part', 'Art'];

    if (!validRootTypes.includes(structure.type)) {
      this.addError('INVALID_ROOT', `Root element should be Document, Part, or Art. Found: ${structure.type}`);
    }
  }

  /**
   * Validate structure recursively
   */
  private validateStructure(node: TagStructure, depth: number): void {
    // Check for required attributes based on tag type
    this.validateTagType(node);
    this.validateAttributes(node);

    // Validate children
    if (node.children) {
      node.children.forEach(child => {
        this.validateParentChildRelationship(node, child);
        this.validateStructure(child, depth + 1);
      });
    }

    // Check for content
    if (!node.children || node.children.length === 0) {
      this.validateLeafNode(node);
    }
  }

  /**
   * Validate tag type
   */
  private validateTagType(node: TagStructure): void {
    const validStructureTags = [
      // Grouping elements
      'Document', 'Part', 'Art', 'Sect', 'Div',
      // Block-level structure elements
      'P', 'H', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6',
      'L', 'LI', 'Lbl', 'LBody',
      'Table', 'TR', 'TH', 'TD', 'THead', 'TBody', 'TFoot',
      'Caption', 'Figure', 'Formula', 'Form',
      // Inline-level structure elements
      'Span', 'Quote', 'Note', 'Reference',
      'BibEntry', 'Code', 'Link', 'Annot',
      // Special structure elements
      'Ruby', 'RB', 'RT', 'RP', 'Warichu', 'WT', 'WP'
    ];

    if (!validStructureTags.includes(node.type)) {
      this.addWarning('UNKNOWN_TAG', `Unknown or non-standard tag type: ${node.type}`, node.type);
    }
  }

  /**
   * Validate tag attributes
   */
  private validateAttributes(node: TagStructure): void {
    // Figure elements must have Alt or ActualText
    if (node.type === 'Figure') {
      if (!node.alt && !node.actualText) {
        this.addError('FIGURE_NO_ALT', 'Figure element must have Alt or ActualText attribute', node.type);
      }
    }

    // Link elements should have Contents or Alt
    if (node.type === 'Link') {
      if (!node.alt && !node.actualText) {
        this.addWarning('LINK_NO_DESCRIPTION', 'Link should have descriptive text', node.type);
      }
    }

    // Table elements must have proper structure
    if (node.type === 'Table') {
      this.validateTableStructure(node);
    }

    // List elements must have proper structure
    if (node.type === 'L') {
      this.validateListStructure(node);
    }

    // Heading elements should have Title or content
    if (node.type.match(/^H[1-6]?$/)) {
      if (!node.title && (!node.children || node.children.length === 0)) {
        this.addWarning('HEADING_NO_CONTENT', 'Heading element should have content', node.type);
      }
    }
  }

  /**
   * Validate table structure
   */
  private validateTableStructure(table: TagStructure): void {
    if (!table.children || table.children.length === 0) {
      this.addError('TABLE_NO_ROWS', 'Table has no rows', 'Table');
      return;
    }

    // Check for TR children
    const hasTR = table.children.some(child => child.type === 'TR');
    if (!hasTR) {
      this.addError('TABLE_NO_TR', 'Table must contain TR elements', 'Table');
    }

    // Check for TH elements (header cells)
    const hasTH = this.containsDescendant(table, 'TH');
    if (!hasTH) {
      this.addWarning('TABLE_NO_HEADERS', 'Table should have TH (header) elements', 'Table');
    }

    // Validate each row
    table.children.forEach(row => {
      if (row.type === 'TR') {
        this.validateTableRow(row);
      }
    });
  }

  /**
   * Validate table row
   */
  private validateTableRow(row: TagStructure): void {
    if (!row.children || row.children.length === 0) {
      this.addError('TR_NO_CELLS', 'Table row has no cells', 'TR');
      return;
    }

    // All children should be TH or TD
    row.children.forEach(cell => {
      if (cell.type !== 'TH' && cell.type !== 'TD') {
        this.addError('TR_INVALID_CHILD', `TR should only contain TH or TD elements, found: ${cell.type}`, 'TR');
      }
    });
  }

  /**
   * Validate list structure
   */
  private validateListStructure(list: TagStructure): void {
    if (!list.children || list.children.length === 0) {
      this.addError('LIST_NO_ITEMS', 'List has no items', 'L');
      return;
    }

    // All children should be LI
    list.children.forEach(item => {
      if (item.type !== 'LI') {
        this.addError('LIST_INVALID_CHILD', `List should only contain LI elements, found: ${item.type}`, 'L');
      } else {
        this.validateListItem(item);
      }
    });
  }

  /**
   * Validate list item structure
   */
  private validateListItem(item: TagStructure): void {
    if (!item.children || item.children.length === 0) {
      this.addWarning('LI_NO_CONTENT', 'List item has no content', 'LI');
      return;
    }

    // LI should contain Lbl (optional) and LBody (required)
    const hasLBody = item.children.some(child => child.type === 'LBody');
    if (!hasLBody) {
      this.addError('LI_NO_LBODY', 'List item must contain LBody element', 'LI');
    }
  }

  /**
   * Validate parent-child relationship
   */
  private validateParentChildRelationship(parent: TagStructure, child: TagStructure): void {
    // Specific rules for parent-child relationships

    // TR can only be child of Table, THead, TBody, or TFoot
    if (child.type === 'TR') {
      const validParents = ['Table', 'THead', 'TBody', 'TFoot'];
      if (!validParents.includes(parent.type)) {
        this.addError('INVALID_PARENT', `TR element has invalid parent: ${parent.type}`, child.type);
      }
    }

    // TH and TD can only be children of TR
    if (child.type === 'TH' || child.type === 'TD') {
      if (parent.type !== 'TR') {
        this.addError('INVALID_PARENT', `${child.type} element must be child of TR, found: ${parent.type}`, child.type);
      }
    }

    // LI must be child of L
    if (child.type === 'LI' && parent.type !== 'L') {
      this.addError('INVALID_PARENT', `LI element must be child of L, found: ${parent.type}`, child.type);
    }

    // Lbl and LBody must be children of LI
    if ((child.type === 'Lbl' || child.type === 'LBody') && parent.type !== 'LI') {
      this.addError('INVALID_PARENT', `${child.type} must be child of LI, found: ${parent.type}`, child.type);
    }
  }

  /**
   * Validate leaf node
   */
  private validateLeafNode(node: TagStructure): void {
    // Leaf nodes (no children) should have content or be marked as artifacts
    const contentTags = ['P', 'Span', 'Quote', 'Code', 'LBody'];

    if (contentTags.includes(node.type)) {
      if (!node.actualText && !node.alt && !node.isArtifact) {
        this.addWarning('LEAF_NO_CONTENT', `${node.type} element appears to have no content`, node.type);
      }
    }
  }

  /**
   * Check if structure contains a descendant of given type
   */
  private containsDescendant(node: TagStructure, type: string): boolean {
    if (node.type === type) {
      return true;
    }

    if (!node.children) {
      return false;
    }

    return node.children.some(child => this.containsDescendant(child, type));
  }

  /**
   * Add validation error
   */
  private addError(code: string, message: string, element?: string): void {
    this.errors.push({ code, message, element });
  }

  /**
   * Add validation warning
   */
  private addWarning(code: string, message: string, element?: string): void {
    this.warnings.push({ code, message, element });
  }

  /**
   * Get all standard PDF structure types
   */
  static getStandardTypes(): string[] {
    return [
      'Document', 'Part', 'Art', 'Sect', 'Div',
      'P', 'H', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6',
      'L', 'LI', 'Lbl', 'LBody',
      'Table', 'TR', 'TH', 'TD', 'THead', 'TBody', 'TFoot',
      'Caption', 'Figure', 'Formula', 'Form',
      'Span', 'Quote', 'Note', 'Reference',
      'BibEntry', 'Code', 'Link', 'Annot'
    ];
  }
}
