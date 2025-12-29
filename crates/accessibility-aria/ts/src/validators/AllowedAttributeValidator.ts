/**
 * Allowed ARIA Attribute Validator
 * Validates that ARIA attributes are allowed for roles
 */

import { ARIARole, ARIAAttribute, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { getRoleDefinition } from '../rules/ARIARoles';
import { isValidAttribute } from '../rules/ARIAAttributes';

export class AllowedAttributeValidator {
  validate(element: HTMLElement): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const role = element.getAttribute('role') as ARIARole | null;
    if (!role) {
      return { valid: true, errors, warnings, info };
    }

    const roleDefinition = getRoleDefinition(role);
    if (!roleDefinition) {
      return { valid: true, errors, warnings, info };
    }

    // Get all ARIA attributes on the element
    const ariaAttributes = this.getAriaAttributes(element);

    for (const attr of ariaAttributes) {
      const attrName = attr.name as ARIAAttribute;

      // Check if attribute is valid ARIA attribute
      if (!isValidAttribute(attrName)) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: `"${attrName}" is not a valid ARIA attribute`,
          element: element.tagName.toLowerCase(),
          role,
          attribute: attrName,
          wcagCriterion: '4.1.2',
        });
        continue;
      }

      // Check if attribute is prohibited for this role
      if (roleDefinition.prohibitedAttributes?.includes(attrName)) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: `Attribute "${attrName}" is prohibited on role="${role}"`,
          element: element.tagName.toLowerCase(),
          role,
          attribute: attrName,
          wcagCriterion: '4.1.2',
        });
        continue;
      }

      // Check if attribute is supported for this role
      if (!roleDefinition.supportedAttributes.includes(attrName)) {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: `Attribute "${attrName}" is not supported on role="${role}"`,
          element: element.tagName.toLowerCase(),
          role,
          attribute: attrName,
          suggestion: `Remove this attribute or verify the role is correct. Supported attributes: ${roleDefinition.supportedAttributes.slice(0, 10).join(', ')}...`,
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

  private getAriaAttributes(element: HTMLElement): Attr[] {
    const ariaAttrs: Attr[] = [];
    const attributes = element.attributes;

    for (let i = 0; i < attributes.length; i++) {
      const attr = attributes[i];
      if (attr.name.startsWith('aria-')) {
        ariaAttrs.push(attr);
      }
    }

    return ariaAttrs;
  }

  validateGlobalAttributes(element: HTMLElement): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    // Global ARIA attributes that can be used on any element
    const globalAttributes: ARIAAttribute[] = [
      'aria-atomic',
      'aria-busy',
      'aria-controls',
      'aria-current',
      'aria-describedby',
      'aria-details',
      'aria-disabled',
      'aria-dropeffect',
      'aria-errormessage',
      'aria-flowto',
      'aria-grabbed',
      'aria-haspopup',
      'aria-hidden',
      'aria-invalid',
      'aria-keyshortcuts',
      'aria-label',
      'aria-labelledby',
      'aria-live',
      'aria-owns',
      'aria-relevant',
      'aria-roledescription',
    ];

    const ariaAttributes = this.getAriaAttributes(element);
    const role = element.getAttribute('role') as ARIARole | null;

    if (!role) {
      // If no role, only global attributes should be used
      for (const attr of ariaAttributes) {
        const attrName = attr.name as ARIAAttribute;
        if (!globalAttributes.includes(attrName)) {
          warnings.push({
            type: 'attribute',
            severity: 'warning',
            message: `Attribute "${attrName}" is not a global ARIA attribute and element has no role`,
            element: element.tagName.toLowerCase(),
            attribute: attrName,
            suggestion: 'Add appropriate role or remove this attribute',
          });
        }
      }
    }

    return {
      valid: true,
      errors,
      warnings,
      info,
    };
  }

  validateProhibitedCombinations(element: HTMLElement): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    // Check for prohibited attribute combinations

    // aria-label and aria-labelledby together
    if (element.hasAttribute('aria-label') && element.hasAttribute('aria-labelledby')) {
      warnings.push({
        type: 'attribute',
        severity: 'warning',
        message: 'Both aria-label and aria-labelledby are present; aria-labelledby takes precedence',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-label',
        suggestion: 'Remove aria-label as aria-labelledby will be used',
      });
    }

    // role="none" or "presentation" with aria-label or aria-labelledby
    const role = element.getAttribute('role');
    if (role === 'none' || role === 'presentation') {
      if (element.hasAttribute('aria-label')) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: `role="${role}" prohibits use of aria-label`,
          element: element.tagName.toLowerCase(),
          role: role as ARIARole,
          attribute: 'aria-label',
          wcagCriterion: '4.1.2',
        });
      }

      if (element.hasAttribute('aria-labelledby')) {
        errors.push({
          type: 'attribute',
          severity: 'error',
          message: `role="${role}" prohibits use of aria-labelledby`,
          element: element.tagName.toLowerCase(),
          role: role as ARIARole,
          attribute: 'aria-labelledby',
          wcagCriterion: '4.1.2',
        });
      }
    }

    // aria-valuenow without aria-valuemin or aria-valuemax in range widgets
    if (element.hasAttribute('aria-valuenow')) {
      const hasValuemin = element.hasAttribute('aria-valuemin');
      const hasValuemax = element.hasAttribute('aria-valuemax');

      if (!hasValuemin || !hasValuemax) {
        warnings.push({
          type: 'attribute',
          severity: 'warning',
          message: 'aria-valuenow should be used with both aria-valuemin and aria-valuemax',
          element: element.tagName.toLowerCase(),
          attribute: 'aria-valuenow',
          suggestion: 'Add aria-valuemin and aria-valuemax to define range',
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

export const allowedAttributeValidator = new AllowedAttributeValidator();
