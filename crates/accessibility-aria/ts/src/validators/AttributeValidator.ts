/**
 * ARIA Attribute Validator
 * Validates ARIA attributes and their values
 */

import { ARIAAttribute, ARIARole, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { getAttributeDefinition, isValidAttribute } from '../rules/ARIAAttributes';
import { getRoleDefinition } from '../rules/ARIARoles';

export class AttributeValidator {
  validate(element: HTMLElement, attribute: ARIAAttribute, value: string): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    // Check if attribute is valid
    if (!isValidAttribute(attribute)) {
      errors.push({
        type: 'attribute',
        severity: 'error',
        message: `Invalid ARIA attribute: "${attribute}"`,
        element: element.tagName.toLowerCase(),
        attribute,
        wcagCriterion: '4.1.2',
      });
      return { valid: false, errors, warnings, info };
    }

    const attrDefinition = getAttributeDefinition(attribute);
    if (!attrDefinition) {
      return { valid: false, errors, warnings, info };
    }

    // Validate attribute value
    const valueValidation = this.validateValue(attribute, value, attrDefinition.valueType, attrDefinition.allowedValues);
    errors.push(...valueValidation.errors);
    warnings.push(...valueValidation.warnings);

    // Check if attribute is supported by element's role
    const role = element.getAttribute('role') as ARIARole | null;
    if (role) {
      const roleDefinition = getRoleDefinition(role);
      if (roleDefinition) {
        // Check if attribute is prohibited
        if (roleDefinition.prohibitedAttributes?.includes(attribute)) {
          errors.push({
            type: 'attribute',
            severity: 'error',
            message: `Attribute "${attribute}" is prohibited on role="${role}"`,
            element: element.tagName.toLowerCase(),
            role,
            attribute,
            wcagCriterion: '4.1.2',
          });
        }
        // Check if attribute is supported
        else if (!roleDefinition.supportedAttributes.includes(attribute)) {
          warnings.push({
            type: 'attribute',
            severity: 'warning',
            message: `Attribute "${attribute}" is not supported on role="${role}"`,
            element: element.tagName.toLowerCase(),
            role,
            attribute,
            suggestion: `Remove this attribute or verify role is correct`,
          });
        }
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  private validateValue(
    attribute: ARIAAttribute,
    value: string,
    valueType: string,
    allowedValues?: string[]
  ): { errors: ValidationError[]; warnings: ValidationWarning[] } {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    if (!value || value.trim() === '') {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `Attribute "${attribute}" has empty value`,
        attribute,
        actualValue: value,
      });
      return { errors, warnings };
    }

    switch (valueType) {
      case 'true/false':
        if (value !== 'true' && value !== 'false') {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" must be "true" or "false"`,
            attribute,
            expectedValue: 'true or false',
            actualValue: value,
          });
        }
        break;

      case 'tristate':
        if (!['true', 'false', 'mixed'].includes(value)) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" must be "true", "false", or "mixed"`,
            attribute,
            expectedValue: 'true, false, or mixed',
            actualValue: value,
          });
        }
        break;

      case 'true/false/undefined':
        if (!['true', 'false', 'undefined'].includes(value)) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" must be "true", "false", or "undefined"`,
            attribute,
            expectedValue: 'true, false, or undefined',
            actualValue: value,
          });
        }
        break;

      case 'integer':
        if (!Number.isInteger(Number(value)) || Number(value) < 0) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" must be a non-negative integer`,
            attribute,
            expectedValue: 'non-negative integer',
            actualValue: value,
          });
        }
        break;

      case 'number':
        if (isNaN(Number(value))) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" must be a number`,
            attribute,
            expectedValue: 'number',
            actualValue: value,
          });
        }
        break;

      case 'token':
        if (allowedValues && !allowedValues.includes(value)) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" has invalid value`,
            attribute,
            expectedValue: allowedValues.join(', '),
            actualValue: value,
          });
        }
        break;

      case 'token list':
        if (allowedValues) {
          const tokens = value.split(/\s+/);
          const invalidTokens = tokens.filter(token => !allowedValues.includes(token));
          if (invalidTokens.length > 0) {
            errors.push({
              type: 'value',
              severity: 'error',
              message: `Attribute "${attribute}" contains invalid tokens: ${invalidTokens.join(', ')}`,
              attribute,
              expectedValue: allowedValues.join(', '),
              actualValue: value,
            });
          }
        }
        break;

      case 'ID reference':
        if (!this.isValidId(value)) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" must reference a valid ID`,
            attribute,
            expectedValue: 'valid ID',
            actualValue: value,
          });
        }
        break;

      case 'ID reference list':
        const ids = value.split(/\s+/);
        const invalidIds = ids.filter(id => !this.isValidId(id));
        if (invalidIds.length > 0) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: `Attribute "${attribute}" contains invalid IDs: ${invalidIds.join(', ')}`,
            attribute,
            expectedValue: 'space-separated list of valid IDs',
            actualValue: value,
          });
        }
        break;

      case 'string':
        // Strings are generally valid, but check for emptiness
        if (value.trim() === '') {
          warnings.push({
            type: 'value',
            severity: 'warning',
            message: `Attribute "${attribute}" has empty string value`,
            attribute,
            suggestion: 'Provide a meaningful value or remove the attribute',
          });
        }
        break;
    }

    return { errors, warnings };
  }

  private isValidId(id: string): boolean {
    // Basic ID validation - must not be empty and not contain whitespace
    return id.length > 0 && !/\s/.test(id);
  }

  validateDeprecated(attribute: ARIAAttribute): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const deprecated: ARIAAttribute[] = ['aria-dropeffect', 'aria-grabbed'];

    if (deprecated.includes(attribute)) {
      warnings.push({
        type: 'attribute',
        severity: 'warning',
        message: `Attribute "${attribute}" is deprecated in ARIA 1.1+`,
        attribute,
        suggestion: 'Remove this attribute and use alternative approach',
      });
    }

    return {
      valid: true,
      errors,
      warnings,
      info,
    };
  }
}

export const attributeValidator = new AttributeValidator();
