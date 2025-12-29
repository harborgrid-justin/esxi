/**
 * ARIA Relationship Validator
 * Validates ARIA relationship attributes and their references
 */

import { ARIAAttribute, ARIARole, RelationshipValidation, ValidationResult, ValidationError, ValidationWarning } from '../types';

export class RelationshipValidator {
  private relationshipAttributes: ARIAAttribute[] = [
    'aria-activedescendant',
    'aria-controls',
    'aria-describedby',
    'aria-details',
    'aria-errormessage',
    'aria-flowto',
    'aria-labelledby',
    'aria-owns',
  ];

  validate(element: HTMLElement): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    for (const attr of this.relationshipAttributes) {
      const value = element.getAttribute(attr);
      if (value) {
        const validation = this.validateRelationship(element, attr, value);
        errors.push(...validation.errors);
        warnings.push(...validation.warnings);
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  validateRelationship(element: HTMLElement, attribute: ARIAAttribute, value: string): {
    errors: ValidationError[];
    warnings: ValidationWarning[];
  } {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    const isList = ['aria-controls', 'aria-describedby', 'aria-flowto', 'aria-labelledby', 'aria-owns'].includes(attribute);
    const ids = isList ? value.trim().split(/\s+/) : [value];

    for (const id of ids) {
      if (!id) continue;

      const referencedElement = element.ownerDocument?.getElementById(id);

      if (!referencedElement) {
        errors.push({
          type: 'relationship',
          severity: 'error',
          message: `${attribute} references non-existent element with ID "${id}"`,
          element: element.tagName.toLowerCase(),
          attribute,
          actualValue: id,
          wcagCriterion: '4.1.2',
        });
        continue;
      }

      // Specific validations for each relationship type
      switch (attribute) {
        case 'aria-activedescendant':
          this.validateActiveDescendant(element, referencedElement, errors, warnings);
          break;

        case 'aria-controls':
          this.validateControls(element, referencedElement, errors, warnings);
          break;

        case 'aria-describedby':
        case 'aria-labelledby':
          this.validateLabelReference(element, referencedElement, attribute, errors, warnings);
          break;

        case 'aria-errormessage':
          this.validateErrorMessage(element, referencedElement, errors, warnings);
          break;

        case 'aria-owns':
          this.validateOwns(element, referencedElement, errors, warnings);
          break;
      }
    }

    return { errors, warnings };
  }

  private validateActiveDescendant(
    element: HTMLElement,
    descendant: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Check if descendant is actually a descendant
    if (!element.contains(descendant)) {
      errors.push({
        type: 'relationship',
        severity: 'error',
        message: 'aria-activedescendant must reference a descendant element',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-activedescendant',
        wcagCriterion: '4.1.2',
      });
    }

    // Check if element has DOM focus or tabindex
    const hasTabindex = element.hasAttribute('tabindex');
    const hasFocus = element === element.ownerDocument?.activeElement;

    if (!hasTabindex && !hasFocus) {
      warnings.push({
        type: 'relationship',
        severity: 'warning',
        message: 'aria-activedescendant requires element to be focusable (add tabindex)',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-activedescendant',
        suggestion: 'Add tabindex="0" to make element focusable',
      });
    }

    // Check if descendant has an ID
    if (!descendant.id) {
      errors.push({
        type: 'relationship',
        severity: 'error',
        message: 'aria-activedescendant references element without ID',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-activedescendant',
      });
    }
  }

  private validateControls(
    element: HTMLElement,
    controlled: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const role = element.getAttribute('role');

    // For combobox, controls should point to listbox, grid, tree, or dialog
    if (role === 'combobox') {
      const controlledRole = controlled.getAttribute('role');
      const validRoles = ['listbox', 'grid', 'tree', 'dialog'];

      if (controlledRole && !validRoles.includes(controlledRole)) {
        errors.push({
          type: 'relationship',
          severity: 'error',
          message: `Combobox aria-controls must reference ${validRoles.join(', ')}`,
          element: element.tagName.toLowerCase(),
          attribute: 'aria-controls',
          expectedValue: validRoles.join(' or '),
          actualValue: controlledRole,
        });
      }
    }

    // For scrollbar, controls should exist
    if (role === 'scrollbar' && !controlled) {
      errors.push({
        type: 'relationship',
        severity: 'error',
        message: 'Scrollbar must control a scrollable region',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-controls',
      });
    }
  }

  private validateLabelReference(
    element: HTMLElement,
    labelElement: HTMLElement,
    attribute: ARIAAttribute,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Check if label element is hidden with aria-hidden
    if (labelElement.getAttribute('aria-hidden') === 'true') {
      warnings.push({
        type: 'relationship',
        severity: 'warning',
        message: `${attribute} references element with aria-hidden="true"`,
        element: element.tagName.toLowerCase(),
        attribute,
        suggestion: 'Label elements should not be hidden from assistive technology',
      });
    }

    // Check if label element has content
    const hasContent = labelElement.textContent?.trim().length ?? 0 > 0;
    const hasAriaLabel = labelElement.hasAttribute('aria-label');

    if (!hasContent && !hasAriaLabel) {
      warnings.push({
        type: 'relationship',
        severity: 'warning',
        message: `${attribute} references element with no text content`,
        element: element.tagName.toLowerCase(),
        attribute,
        suggestion: 'Label element should have meaningful content',
      });
    }
  }

  private validateErrorMessage(
    element: HTMLElement,
    errorElement: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Check if aria-invalid is set
    const invalid = element.getAttribute('aria-invalid');

    if (!invalid || invalid === 'false') {
      warnings.push({
        type: 'relationship',
        severity: 'warning',
        message: 'aria-errormessage present but aria-invalid is not "true"',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-errormessage',
        suggestion: 'Set aria-invalid="true" when error message is shown',
      });
    }

    // Check if error element has role="alert" or is in a live region
    const errorRole = errorElement.getAttribute('role');
    const hasLiveRegion = errorElement.hasAttribute('aria-live');

    if (errorRole !== 'alert' && !hasLiveRegion) {
      warnings.push({
        type: 'relationship',
        severity: 'warning',
        message: 'Error message should have role="alert" or aria-live attribute',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-errormessage',
        suggestion: 'Add role="alert" to error message element',
      });
    }
  }

  private validateOwns(
    element: HTMLElement,
    ownedElement: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Check if owned element is already a DOM descendant
    if (element.contains(ownedElement)) {
      warnings.push({
        type: 'relationship',
        severity: 'warning',
        message: 'aria-owns references element that is already a DOM descendant',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-owns',
        suggestion: 'aria-owns is typically used for non-DOM parent-child relationships',
      });
    }

    // Check if owned element has another aria-owns parent
    const allOwners = element.ownerDocument?.querySelectorAll('[aria-owns]');
    if (allOwners) {
      for (const owner of Array.from(allOwners)) {
        if (owner === element) continue;

        const ownedIds = owner.getAttribute('aria-owns')?.split(/\s+/) || [];
        if (ownedIds.includes(ownedElement.id)) {
          errors.push({
            type: 'relationship',
            severity: 'error',
            message: 'Element is owned by multiple parents via aria-owns',
            element: element.tagName.toLowerCase(),
            attribute: 'aria-owns',
            wcagCriterion: '4.1.2',
          });
        }
      }
    }
  }

  getRelationships(element: HTMLElement): RelationshipValidation[] {
    const relationships: RelationshipValidation[] = [];

    for (const attr of this.relationshipAttributes) {
      const value = element.getAttribute(attr);
      if (!value) continue;

      const isList = ['aria-controls', 'aria-describedby', 'aria-flowto', 'aria-labelledby', 'aria-owns'].includes(attr);
      const ids = isList ? value.trim().split(/\s+/) : [value];

      for (const id of ids) {
        if (!id) continue;

        const referencedElement = element.ownerDocument?.getElementById(id);
        const exists = !!referencedElement;
        const targetRole = referencedElement?.getAttribute('role') as ARIARole | undefined;

        relationships.push({
          attribute: attr,
          referencedId: id,
          exists,
          targetElement: referencedElement?.tagName.toLowerCase(),
          targetRole,
          validReference: exists,
          errors: exists ? [] : [`Referenced element with ID "${id}" does not exist`],
        });
      }
    }

    return relationships;
  }
}

export const relationshipValidator = new RelationshipValidator();
