/**
 * ARIA Validation React Hook
 * Custom hook for ARIA validation in React components
 */

import { useState, useCallback, useEffect } from 'react';
import { ARIARole, ARIAAttribute, ValidationResult, ARIAValidationHookResult } from '../types';
import { roleValidator } from '../validators/RoleValidator';
import { attributeValidator } from '../validators/AttributeValidator';
import { stateValidator } from '../validators/StateValidator';
import { requiredAttributeValidator } from '../validators/RequiredAttributeValidator';
import { allowedAttributeValidator } from '../validators/AllowedAttributeValidator';
import { relationshipValidator } from '../validators/RelationshipValidator';
import { semanticAnalyzer } from '../analyzers/SemanticAnalyzer';
import { implicitRoleAnalyzer } from '../analyzers/ImplicitRoleAnalyzer';
import { getRoleDefinition } from '../rules/ARIARoles';
import { getAttributeDefinition } from '../rules/ARIAAttributes';

export function useARIAValidation(): ARIAValidationHookResult {
  const [isValidating, setIsValidating] = useState(false);
  const [lastResult, setLastResult] = useState<ValidationResult | null>(null);

  const validate = useCallback((element: HTMLElement | Document): ValidationResult => {
    setIsValidating(true);

    const errors: any[] = [];
    const warnings: any[] = [];
    const info: any[] = [];

    try {
      if (element instanceof Document) {
        // Validate entire document
        const semanticResult = semanticAnalyzer.validateSemanticStructure(element);
        errors.push(...semanticResult.errors);
        warnings.push(...semanticResult.warnings);
        info.push(...semanticResult.info);

        // Validate all elements with ARIA attributes or roles
        const allElements = element.querySelectorAll('[role], [class*="aria-"]');
        allElements.forEach(el => {
          if (el instanceof HTMLElement) {
            const elementResult = validateElement(el);
            errors.push(...elementResult.errors);
            warnings.push(...elementResult.warnings);
            info.push(...elementResult.info);
          }
        });
      } else {
        // Validate single element
        const elementResult = validateElement(element);
        errors.push(...elementResult.errors);
        warnings.push(...elementResult.warnings);
        info.push(...elementResult.info);
      }

      const result: ValidationResult = {
        valid: errors.length === 0,
        errors,
        warnings,
        info,
      };

      setLastResult(result);
      return result;
    } finally {
      setIsValidating(false);
    }
  }, []);

  const validateElement = (element: HTMLElement): ValidationResult => {
    const errors: any[] = [];
    const warnings: any[] = [];
    const info: any[] = [];

    const role = element.getAttribute('role') as ARIARole | null;

    // Validate role
    if (role) {
      const roleResult = roleValidator.validate(element, role);
      errors.push(...roleResult.errors);
      warnings.push(...roleResult.warnings);

      // Validate accessible name
      const nameResult = roleValidator.validateAccessibleName(element, role);
      errors.push(...nameResult.errors);
      warnings.push(...nameResult.warnings);

      // Validate children presentational
      const childrenResult = roleValidator.validateChildrenPresentational(element, role);
      errors.push(...childrenResult.errors);
      warnings.push(...childrenResult.warnings);

      // Validate required attributes
      const requiredResult = requiredAttributeValidator.validate(element, role);
      errors.push(...requiredResult.errors);
      warnings.push(...requiredResult.warnings);

      // Validate conditional attributes
      const conditionalResult = requiredAttributeValidator.validateConditionalAttributes(element, role);
      errors.push(...conditionalResult.errors);
      warnings.push(...conditionalResult.warnings);
    }

    // Validate allowed attributes
    const allowedResult = allowedAttributeValidator.validate(element);
    errors.push(...allowedResult.errors);
    warnings.push(...allowedResult.warnings);

    // Validate global attributes
    const globalResult = allowedAttributeValidator.validateGlobalAttributes(element);
    errors.push(...globalResult.errors);
    warnings.push(...globalResult.warnings);

    // Validate prohibited combinations
    const prohibitedResult = allowedAttributeValidator.validateProhibitedCombinations(element);
    errors.push(...prohibitedResult.errors);
    warnings.push(...prohibitedResult.warnings);

    // Validate states
    const stateResult = stateValidator.validate(element);
    errors.push(...stateResult.errors);
    warnings.push(...stateResult.warnings);

    // Validate relationships
    const relationshipResult = relationshipValidator.validate(element);
    errors.push(...relationshipResult.errors);
    warnings.push(...relationshipResult.warnings);

    // Validate all ARIA attribute values
    Array.from(element.attributes).forEach(attr => {
      if (attr.name.startsWith('aria-')) {
        const attrResult = attributeValidator.validate(
          element,
          attr.name as ARIAAttribute,
          attr.value
        );
        errors.push(...attrResult.errors);
        warnings.push(...attrResult.warnings);

        // Check for deprecated attributes
        const deprecatedResult = attributeValidator.validateDeprecated(attr.name as ARIAAttribute);
        warnings.push(...deprecatedResult.warnings);
      }
    });

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  };

  const validateRole = useCallback((element: HTMLElement, role: ARIARole): boolean => {
    const result = roleValidator.validate(element, role);
    return result.valid;
  }, []);

  const validateAttribute = useCallback((
    element: HTMLElement,
    attribute: ARIAAttribute,
    value: string
  ): boolean => {
    const result = attributeValidator.validate(element, attribute, value);
    return result.valid;
  }, []);

  const getImplicitRole = useCallback((element: HTMLElement): ARIARole | null => {
    return implicitRoleAnalyzer.getImplicitRole(element);
  }, []);

  const getRoleDefinitionFn = useCallback((role: ARIARole) => {
    return getRoleDefinition(role);
  }, []);

  const getAttributeDefinitionFn = useCallback((attribute: ARIAAttribute) => {
    return getAttributeDefinition(attribute);
  }, []);

  return {
    validate,
    validateRole,
    validateAttribute,
    getImplicitRole,
    getRoleDefinition: getRoleDefinitionFn,
    getAttributeDefinition: getAttributeDefinitionFn,
    isValidating,
    lastResult,
  };
}
