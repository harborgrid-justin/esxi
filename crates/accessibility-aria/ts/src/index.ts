/**
 * Meridian Accessibility ARIA Validator
 * Enterprise-grade ARIA attribute validator with WAI-ARIA 1.2 compliance
 * @module @meridian/accessibility-aria
 */

// Type exports
export type {
  ARIARole,
  ARIARoleType,
  ARIAAttribute,
  ARIAState,
  ARIAAttributeValue,
  ARIAAutocomplete,
  ARIAChecked,
  ARIACurrent,
  ARIAHasPopup,
  ARIAInvalid,
  ARIALive,
  ARIAOrientation,
  ARIAPressed,
  ARIARelevant,
  ARIASort,
  ValidationResult,
  ValidationError,
  ValidationWarning,
  ValidationInfo,
  RoleDefinition,
  AttributeDefinition,
  ARIAPattern,
  KeyboardInteraction,
  StateManagement,
  SemanticAnalysis,
  ARIATreeNode,
  RelationshipValidation,
  ImplicitRoleMapping,
  AccessibilityContext,
  ValidationOptions,
  CustomRule,
  ARIAValidationHookResult,
} from './types';

// Rule exports
export {
  ARIA_ROLES,
  getRoleDefinition,
  isAbstractRole,
  isValidRole,
} from './rules/ARIARoles';

export {
  ARIA_ATTRIBUTES,
  getAttributeDefinition,
  isValidAttribute,
  isStateAttribute,
  isPropertyAttribute,
  getAllowedValues,
  getDefaultValue,
} from './rules/ARIAAttributes';

export {
  ARIA_PATTERNS,
  getPattern,
  getPatternsForRole,
  getAllPatternNames,
} from './rules/ARIAPatterns';

// Validator exports
export { RoleValidator, roleValidator } from './validators/RoleValidator';
export { AttributeValidator, attributeValidator } from './validators/AttributeValidator';
export { StateValidator, stateValidator } from './validators/StateValidator';
export {
  RequiredAttributeValidator,
  requiredAttributeValidator,
} from './validators/RequiredAttributeValidator';
export {
  AllowedAttributeValidator,
  allowedAttributeValidator,
} from './validators/AllowedAttributeValidator';
export { ValueValidator, valueValidator } from './validators/ValueValidator';
export {
  RelationshipValidator,
  relationshipValidator,
} from './validators/RelationshipValidator';

// Analyzer exports
export { SemanticAnalyzer, semanticAnalyzer } from './analyzers/SemanticAnalyzer';
export { ImplicitRoleAnalyzer, implicitRoleAnalyzer } from './analyzers/ImplicitRoleAnalyzer';

// Component exports
export { ARIAValidator } from './components/Validator/ARIAValidator';
export type { ARIAValidatorProps } from './components/Validator/ARIAValidator';

export { RoleChecker } from './components/Validator/RoleChecker';
export type { RoleCheckerProps } from './components/Validator/RoleChecker';

export { AttributeChecker } from './components/Validator/AttributeChecker';
export type { AttributeCheckerProps } from './components/Validator/AttributeChecker';

export { StateChecker } from './components/Validator/StateChecker';
export type { StateCheckerProps } from './components/Validator/StateChecker';

export { RoleAnalysis } from './components/Analysis/RoleAnalysis';
export type { RoleAnalysisProps } from './components/Analysis/RoleAnalysis';

export { PatternMatcher } from './components/Analysis/PatternMatcher';
export type { PatternMatcherProps } from './components/Analysis/PatternMatcher';

export { SemanticChecker } from './components/Analysis/SemanticChecker';
export type { SemanticCheckerProps } from './components/Analysis/SemanticChecker';

export { ARIATree } from './components/Visualization/ARIATree';
export type { ARIATreeProps } from './components/Visualization/ARIATree';

export { RoleHierarchy } from './components/Visualization/RoleHierarchy';
export type { RoleHierarchyProps } from './components/Visualization/RoleHierarchy';

export { RoleReference } from './components/Reference/RoleReference';
export { PatternLibrary } from './components/Reference/PatternLibrary';

// Hook exports
export { useARIAValidation } from './hooks/useARIAValidation';

// Utility exports
export {
  buildARIATree,
  getAccessibleName,
  getAccessibleDescription,
  isInteractive,
  isFocusable,
  getLandmarkLabel,
  findLandmarks,
  findHeadings,
  getHeadingLevel,
  findInteractiveElements,
  hasAccessibleName,
  computeAccessibleDescription,
  formatRoleName,
  formatAttributeName,
  getElementSelector,
  escapeHtml,
  truncate,
} from './utils/ariaUtils';

/**
 * Validate an HTML element or document for ARIA compliance
 * @param target - HTMLElement or Document to validate
 * @returns ValidationResult with errors, warnings, and info
 */
export function validateARIA(target: HTMLElement | Document): ValidationResult {
  const errors: any[] = [];
  const warnings: any[] = [];
  const info: any[] = [];

  // Import validators
  const { roleValidator } = require('./validators/RoleValidator');
  const { attributeValidator } = require('./validators/AttributeValidator');
  const { stateValidator } = require('./validators/StateValidator');
  const { requiredAttributeValidator } = require('./validators/RequiredAttributeValidator');
  const { allowedAttributeValidator } = require('./validators/AllowedAttributeValidator');
  const { relationshipValidator } = require('./validators/RelationshipValidator');
  const { semanticAnalyzer } = require('./analyzers/SemanticAnalyzer');

  if (target instanceof Document) {
    // Validate document structure
    const semanticResult = semanticAnalyzer.validateSemanticStructure(target);
    errors.push(...semanticResult.errors);
    warnings.push(...semanticResult.warnings);
    info.push(...semanticResult.info);

    // Validate all elements with ARIA
    const elements = target.querySelectorAll('[role], [class*="aria-"]');
    elements.forEach((el: Element) => {
      if (el instanceof HTMLElement) {
        const elementResult = validateElement(el);
        errors.push(...elementResult.errors);
        warnings.push(...elementResult.warnings);
        info.push(...elementResult.info);
      }
    });
  } else {
    const elementResult = validateElement(target);
    errors.push(...elementResult.errors);
    warnings.push(...elementResult.warnings);
    info.push(...elementResult.info);
  }

  return {
    valid: errors.length === 0,
    errors,
    warnings,
    info,
  };
}

function validateElement(element: HTMLElement): ValidationResult {
  const errors: any[] = [];
  const warnings: any[] = [];
  const info: any[] = [];

  const { roleValidator } = require('./validators/RoleValidator');
  const { attributeValidator } = require('./validators/AttributeValidator');
  const { stateValidator } = require('./validators/StateValidator');
  const { requiredAttributeValidator } = require('./validators/RequiredAttributeValidator');
  const { allowedAttributeValidator } = require('./validators/AllowedAttributeValidator');
  const { relationshipValidator } = require('./validators/RelationshipValidator');

  const role = element.getAttribute('role') as any;

  if (role) {
    const roleResult = roleValidator.validate(element, role);
    errors.push(...roleResult.errors);
    warnings.push(...roleResult.warnings);

    const nameResult = roleValidator.validateAccessibleName(element, role);
    errors.push(...nameResult.errors);
    warnings.push(...nameResult.warnings);

    const requiredResult = requiredAttributeValidator.validate(element, role);
    errors.push(...requiredResult.errors);
    warnings.push(...requiredResult.warnings);
  }

  const allowedResult = allowedAttributeValidator.validate(element);
  errors.push(...allowedResult.errors);
  warnings.push(...allowedResult.warnings);

  const stateResult = stateValidator.validate(element);
  errors.push(...stateResult.errors);
  warnings.push(...stateResult.warnings);

  const relationshipResult = relationshipValidator.validate(element);
  errors.push(...relationshipResult.errors);
  warnings.push(...relationshipResult.warnings);

  Array.from(element.attributes).forEach(attr => {
    if (attr.name.startsWith('aria-')) {
      const attrResult = attributeValidator.validate(element, attr.name as any, attr.value);
      errors.push(...attrResult.errors);
      warnings.push(...attrResult.warnings);
    }
  });

  return { valid: errors.length === 0, errors, warnings, info };
}

/**
 * Default export for the ARIA validator package
 */
export default {
  validateARIA,
  useARIAValidation,
  ARIA_ROLES,
  ARIA_ATTRIBUTES,
  ARIA_PATTERNS,
  roleValidator,
  attributeValidator,
  stateValidator,
  semanticAnalyzer,
  implicitRoleAnalyzer,
};
