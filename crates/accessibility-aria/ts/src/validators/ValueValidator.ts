/**
 * ARIA Value Validator
 * Validates ARIA attribute values for correctness and constraints
 */

import { ARIAAttribute, ARIARole, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { getAttributeDefinition } from '../rules/ARIAAttributes';

export class ValueValidator {
  validate(attribute: ARIAAttribute, value: string, element?: HTMLElement): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const definition = getAttributeDefinition(attribute);
    if (!definition) {
      return { valid: false, errors, warnings, info };
    }

    // Validate based on value type
    switch (definition.valueType) {
      case 'true/false':
        this.validateBoolean(attribute, value, errors);
        break;

      case 'tristate':
        this.validateTristate(attribute, value, errors);
        break;

      case 'true/false/undefined':
        this.validateBooleanOrUndefined(attribute, value, errors);
        break;

      case 'integer':
        this.validateInteger(attribute, value, errors, element);
        break;

      case 'number':
        this.validateNumber(attribute, value, errors, element);
        break;

      case 'token':
        this.validateToken(attribute, value, definition.allowedValues || [], errors);
        break;

      case 'token list':
        this.validateTokenList(attribute, value, definition.allowedValues || [], errors);
        break;

      case 'ID reference':
        this.validateIdReference(attribute, value, errors, element);
        break;

      case 'ID reference list':
        this.validateIdReferenceList(attribute, value, errors, element);
        break;

      case 'string':
        this.validateString(attribute, value, errors, warnings);
        break;
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  private validateBoolean(attribute: ARIAAttribute, value: string, errors: ValidationError[]): void {
    if (value !== 'true' && value !== 'false') {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" must be "true" or "false"`,
        attribute,
        expectedValue: 'true or false',
        actualValue: value,
      });
    }
  }

  private validateTristate(attribute: ARIAAttribute, value: string, errors: ValidationError[]): void {
    if (!['true', 'false', 'mixed'].includes(value)) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" must be "true", "false", or "mixed"`,
        attribute,
        expectedValue: 'true, false, or mixed',
        actualValue: value,
      });
    }
  }

  private validateBooleanOrUndefined(attribute: ARIAAttribute, value: string, errors: ValidationError[]): void {
    if (!['true', 'false', 'undefined'].includes(value)) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" must be "true", "false", or "undefined"`,
        attribute,
        expectedValue: 'true, false, or undefined',
        actualValue: value,
      });
    }
  }

  private validateInteger(attribute: ARIAAttribute, value: string, errors: ValidationError[], element?: HTMLElement): void {
    const num = Number(value);

    if (!Number.isInteger(num)) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" must be an integer`,
        attribute,
        expectedValue: 'integer',
        actualValue: value,
      });
      return;
    }

    // Special validations for specific attributes
    if (['aria-level', 'aria-posinset', 'aria-setsize', 'aria-colcount', 'aria-rowcount', 'aria-colindex', 'aria-rowindex'].includes(attribute)) {
      if (num < 1) {
        errors.push({
          type: 'value',
          severity: 'error',
          message: `"${attribute}" must be at least 1`,
          attribute,
          expectedValue: '>= 1',
          actualValue: value,
        });
      }
    }

    if (['aria-colspan', 'aria-rowspan'].includes(attribute)) {
      if (num < 1) {
        errors.push({
          type: 'value',
          severity: 'error',
          message: `"${attribute}" must be at least 1`,
          attribute,
          expectedValue: '>= 1',
          actualValue: value,
        });
      }
    }

    // Validate posinset vs setsize
    if (attribute === 'aria-posinset' && element) {
      const setsize = element.getAttribute('aria-setsize');
      if (setsize) {
        const setsizeNum = Number(setsize);
        if (num > setsizeNum) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: 'aria-posinset cannot be greater than aria-setsize',
            attribute,
            expectedValue: `<= ${setsize}`,
            actualValue: value,
          });
        }
      }
    }
  }

  private validateNumber(attribute: ARIAAttribute, value: string, errors: ValidationError[], element?: HTMLElement): void {
    const num = Number(value);

    if (isNaN(num)) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" must be a number`,
        attribute,
        expectedValue: 'number',
        actualValue: value,
      });
      return;
    }

    // Validate range constraints
    if (attribute === 'aria-valuenow' && element) {
      const valuemin = element.getAttribute('aria-valuemin');
      const valuemax = element.getAttribute('aria-valuemax');

      if (valuemin !== null) {
        const min = Number(valuemin);
        if (!isNaN(min) && num < min) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: 'aria-valuenow is less than aria-valuemin',
            attribute,
            expectedValue: `>= ${valuemin}`,
            actualValue: value,
          });
        }
      }

      if (valuemax !== null) {
        const max = Number(valuemax);
        if (!isNaN(max) && num > max) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: 'aria-valuenow is greater than aria-valuemax',
            attribute,
            expectedValue: `<= ${valuemax}`,
            actualValue: value,
          });
        }
      }
    }

    // Validate valuemin < valuemax
    if (attribute === 'aria-valuemin' && element) {
      const valuemax = element.getAttribute('aria-valuemax');
      if (valuemax !== null) {
        const max = Number(valuemax);
        if (!isNaN(max) && num > max) {
          errors.push({
            type: 'value',
            severity: 'error',
            message: 'aria-valuemin cannot be greater than aria-valuemax',
            attribute,
            expectedValue: `<= ${valuemax}`,
            actualValue: value,
          });
        }
      }
    }
  }

  private validateToken(attribute: ARIAAttribute, value: string, allowedValues: string[], errors: ValidationError[]): void {
    if (allowedValues.length > 0 && !allowedValues.includes(value)) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" has invalid value "${value}"`,
        attribute,
        expectedValue: allowedValues.join(', '),
        actualValue: value,
      });
    }
  }

  private validateTokenList(attribute: ARIAAttribute, value: string, allowedValues: string[], errors: ValidationError[]): void {
    const tokens = value.trim().split(/\s+/);

    if (allowedValues.length > 0) {
      const invalidTokens = tokens.filter(token => !allowedValues.includes(token));

      if (invalidTokens.length > 0) {
        errors.push({
          type: 'value',
          severity: 'error',
          message: `"${attribute}" contains invalid tokens: ${invalidTokens.join(', ')}`,
          attribute,
          expectedValue: allowedValues.join(', '),
          actualValue: value,
        });
      }
    }

    // Check for duplicate tokens
    const uniqueTokens = new Set(tokens);
    if (uniqueTokens.size !== tokens.length) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" contains duplicate tokens`,
        attribute,
        actualValue: value,
      });
    }
  }

  private validateIdReference(attribute: ARIAAttribute, value: string, errors: ValidationError[], element?: HTMLElement): void {
    if (!value || value.trim() === '') {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" cannot be empty`,
        attribute,
        actualValue: value,
      });
      return;
    }

    // Check for whitespace
    if (/\s/.test(value)) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" ID reference cannot contain whitespace`,
        attribute,
        actualValue: value,
      });
      return;
    }

    // Check if referenced element exists (if we have access to document)
    if (element && element.ownerDocument) {
      const referencedElement = element.ownerDocument.getElementById(value);
      if (!referencedElement) {
        errors.push({
          type: 'relationship',
          severity: 'error',
          message: `"${attribute}" references non-existent ID "${value}"`,
          attribute,
          actualValue: value,
          wcagCriterion: '4.1.2',
        });
      }
    }
  }

  private validateIdReferenceList(attribute: ARIAAttribute, value: string, errors: ValidationError[], element?: HTMLElement): void {
    if (!value || value.trim() === '') {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" cannot be empty`,
        attribute,
        actualValue: value,
      });
      return;
    }

    const ids = value.trim().split(/\s+/);

    // Check for duplicates
    const uniqueIds = new Set(ids);
    if (uniqueIds.size !== ids.length) {
      errors.push({
        type: 'value',
        severity: 'error',
        message: `"${attribute}" contains duplicate ID references`,
        attribute,
        actualValue: value,
      });
    }

    // Check if referenced elements exist
    if (element && element.ownerDocument) {
      for (const id of ids) {
        if (!id) continue;

        const referencedElement = element.ownerDocument.getElementById(id);
        if (!referencedElement) {
          errors.push({
            type: 'relationship',
            severity: 'error',
            message: `"${attribute}" references non-existent ID "${id}"`,
            attribute,
            actualValue: value,
            wcagCriterion: '4.1.2',
          });
        }
      }
    }
  }

  private validateString(attribute: ARIAAttribute, value: string, errors: ValidationError[], warnings: ValidationWarning[]): void {
    if (value.trim() === '') {
      warnings.push({
        type: 'value',
        severity: 'warning',
        message: `"${attribute}" has empty or whitespace-only value`,
        attribute,
        suggestion: 'Provide meaningful text or remove the attribute',
      });
    }

    // Check for excessively long strings
    if (value.length > 1000) {
      warnings.push({
        type: 'value',
        severity: 'warning',
        message: `"${attribute}" has very long value (${value.length} characters)`,
        attribute,
        suggestion: 'Consider using more concise text',
      });
    }
  }
}

export const valueValidator = new ValueValidator();
