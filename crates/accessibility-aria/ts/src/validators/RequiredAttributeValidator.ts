/**
 * Required ARIA Attribute Validator
 * Validates that required ARIA attributes are present for roles
 */

import { ARIARole, ARIAAttribute, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { getRoleDefinition } from '../rules/ARIARoles';

export class RequiredAttributeValidator {
  validate(element: HTMLElement, role: ARIARole): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const roleDefinition = getRoleDefinition(role);
    if (!roleDefinition) {
      return { valid: true, errors, warnings, info };
    }

    // Check for required attributes
    for (const requiredAttr of roleDefinition.requiredAttributes) {
      const hasAttribute = element.hasAttribute(requiredAttr);

      if (!hasAttribute) {
        // Check if there's an implicit value
        const implicitValue = roleDefinition.implicitValueForRole?.[requiredAttr];

        if (!implicitValue) {
          errors.push({
            type: 'attribute',
            severity: 'error',
            message: `Role "${role}" requires attribute "${requiredAttr}"`,
            element: element.tagName.toLowerCase(),
            role,
            attribute: requiredAttr,
            expectedValue: 'required',
            wcagCriterion: '4.1.2',
          });
        }
      } else {
        // Attribute exists, check if value is empty
        const value = element.getAttribute(requiredAttr);
        if (value === null || value.trim() === '') {
          errors.push({
            type: 'attribute',
            severity: 'error',
            message: `Required attribute "${requiredAttr}" has empty value`,
            element: element.tagName.toLowerCase(),
            role,
            attribute: requiredAttr,
            actualValue: value || '',
            wcagCriterion: '4.1.2',
          });
        }
      }
    }

    // Check for conditional required attributes
    this.checkConditionalRequirements(element, role, errors, warnings);

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  private checkConditionalRequirements(
    element: HTMLElement,
    role: ARIARole,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    // Combobox specific requirements (ARIA 1.2)
    if (role === 'combobox') {
      const hasControls = element.hasAttribute('aria-controls');
      const hasExpanded = element.hasAttribute('aria-expanded');

      if (!hasControls) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: 'Combobox requires aria-controls to reference listbox/grid/tree/dialog',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-controls',
          wcagCriterion: '4.1.2',
        });
      }

      if (!hasExpanded) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: 'Combobox requires aria-expanded',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-expanded',
          wcagCriterion: '4.1.2',
        });
      }
    }

    // Scrollbar specific requirements
    if (role === 'scrollbar') {
      const hasControls = element.hasAttribute('aria-controls');
      const hasValuenow = element.hasAttribute('aria-valuenow');

      if (!hasControls) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: 'Scrollbar requires aria-controls to reference scrolled element',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-controls',
          wcagCriterion: '4.1.2',
        });
      }

      if (!hasValuenow) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: 'Scrollbar requires aria-valuenow',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-valuenow',
          wcagCriterion: '4.1.2',
        });
      }
    }

    // Range widgets (slider, spinbutton, progressbar)
    if (['slider', 'spinbutton'].includes(role)) {
      const hasValuenow = element.hasAttribute('aria-valuenow');

      if (!hasValuenow) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: `${role} requires aria-valuenow`,
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-valuenow',
          wcagCriterion: '4.1.2',
        });
      }

      // Check for valuemin and valuemax
      const hasValuemin = element.hasAttribute('aria-valuemin');
      const hasValuemax = element.hasAttribute('aria-valuemax');

      if (!hasValuemin) {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: `${role} should have aria-valuemin defined`,
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-valuemin',
          suggestion: 'Add aria-valuemin to define minimum value',
        });
      }

      if (!hasValuemax) {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: `${role} should have aria-valuemax defined`,
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-valuemax',
          suggestion: 'Add aria-valuemax to define maximum value',
        });
      }
    }

    // Tab requires aria-controls pointing to tabpanel
    if (role === 'tab') {
      const hasControls = element.hasAttribute('aria-controls');

      if (!hasControls) {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: 'Tab should have aria-controls referencing its tabpanel',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-controls',
          suggestion: 'Add aria-controls attribute to associate tab with tabpanel',
        });
      }
    }

    // Grid, table require row children
    if (['grid', 'table', 'treegrid'].includes(role)) {
      const hasRowChild = Array.from(element.children).some(
        child => child.getAttribute('role') === 'row' || child.getAttribute('role') === 'rowgroup'
      );

      if (!hasRowChild) {
        errors.push({
          type: 'role',
          severity: 'error',
          message: `${role} must contain row or rowgroup elements`,
          element: element.tagName.toLowerCase(),
          role,
          wcagCriterion: '4.1.2',
        });
      }
    }

    // Heading requires aria-level
    if (role === 'heading') {
      const hasLevel = element.hasAttribute('aria-level');

      if (!hasLevel) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: 'Heading role requires aria-level',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-level',
          wcagCriterion: '4.1.2',
        });
      }
    }
  }

  validateConditionalAttributes(element: HTMLElement, role: ARIARole): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    // aria-errormessage requires aria-invalid
    if (element.hasAttribute('aria-errormessage')) {
      const invalid = element.getAttribute('aria-invalid');
      if (!invalid || invalid === 'false') {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: 'aria-errormessage should be used with aria-invalid="true"',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-errormessage',
          suggestion: 'Set aria-invalid to "true" when error message is present',
        });
      }
    }

    // aria-activedescendant requires element to be focused or have tabindex
    if (element.hasAttribute('aria-activedescendant')) {
      const tabindex = element.getAttribute('tabindex');
      if (!tabindex && element !== document.activeElement) {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: 'aria-activedescendant requires element to be focusable',
          element: element.tagName.toLowerCase(),
          role,
          attribute: 'aria-activedescendant',
          suggestion: 'Add tabindex="0" or ensure element is inherently focusable',
        });
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }
}

export const requiredAttributeValidator = new RequiredAttributeValidator();
