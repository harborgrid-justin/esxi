/**
 * ARIA Role Validator
 * Validates ARIA roles according to WAI-ARIA 1.2 specification
 */

import { ARIARole, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { getRoleDefinition, isAbstractRole, isValidRole } from '../rules/ARIARoles';

export class RoleValidator {
  validate(element: HTMLElement, role: ARIARole): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    // Check if role is valid
    if (!isValidRole(role)) {
      errors.push({
        type: 'role',
        severity: 'error',
        message: `Invalid ARIA role: "${role}"`,
        element: element.tagName.toLowerCase(),
        role,
        wcagCriterion: '4.1.2',
      });
      return { valid: false, errors, warnings, info };
    }

    // Check if role is abstract
    if (isAbstractRole(role)) {
      errors.push({
        type: 'role',
        severity: 'error',
        message: `Abstract role "${role}" cannot be used in HTML`,
        element: element.tagName.toLowerCase(),
        role,
        wcagCriterion: '4.1.2',
      });
    }

    const roleDefinition = getRoleDefinition(role);
    if (!roleDefinition) {
      return { valid: false, errors, warnings, info };
    }

    // Check required context
    if (roleDefinition.requiredContextRole.length > 0) {
      const parent = element.parentElement;
      const parentRole = parent?.getAttribute('role') as ARIARole | null;

      if (!parent || !parentRole || !roleDefinition.requiredContextRole.includes(parentRole)) {
        errors.push({
          type: 'role',
          severity: 'error',
          message: `Role "${role}" must be contained in one of: ${roleDefinition.requiredContextRole.join(', ')}`,
          element: element.tagName.toLowerCase(),
          role,
          expectedValue: roleDefinition.requiredContextRole.join(' or '),
          actualValue: parentRole || 'none',
          wcagCriterion: '4.1.2',
        });
      }
    }

    // Check required owned elements
    if (roleDefinition.requiredOwnedElements.length > 0) {
      const childRoles = Array.from(element.children)
        .map(child => child.getAttribute('role'))
        .filter(Boolean);

      const hasRequiredChild = roleDefinition.requiredOwnedElements.some(required =>
        childRoles.includes(required)
      );

      if (!hasRequiredChild) {
        errors.push({
          type: 'role',
          severity: 'error',
          message: `Role "${role}" must contain at least one of: ${roleDefinition.requiredOwnedElements.join(', ')}`,
          element: element.tagName.toLowerCase(),
          role,
          expectedValue: roleDefinition.requiredOwnedElements.join(' or '),
          wcagCriterion: '4.1.2',
        });
      }
    }

    // Check if role conflicts with native semantics
    const tagName = element.tagName.toLowerCase();
    if (this.hasSemanticConflict(tagName, role)) {
      warnings.push({
        type: 'semantic',
        severity: 'warning',
        message: `Role "${role}" conflicts with native semantics of <${tagName}>`,
        element: tagName,
        role,
        suggestion: `Consider using native HTML element instead of ARIA role`,
        wcagCriterion: '4.1.2',
      });
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  private hasSemanticConflict(tagName: string, role: ARIARole): boolean {
    const conflicts: Record<string, ARIARole[]> = {
      'button': ['link'],
      'a': ['button'],
      'input': ['button', 'link'],
      'nav': ['main', 'search', 'banner'],
      'header': ['navigation', 'main'],
      'footer': ['navigation', 'main'],
      'main': ['navigation', 'banner', 'contentinfo'],
      'aside': ['main', 'banner', 'contentinfo'],
    };

    return conflicts[tagName]?.includes(role) || false;
  }

  validateAccessibleName(element: HTMLElement, role: ARIARole): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const roleDefinition = getRoleDefinition(role);
    if (!roleDefinition || !roleDefinition.accessibleNameRequired) {
      return { valid: true, errors, warnings, info };
    }

    const hasAriaLabel = element.hasAttribute('aria-label');
    const hasAriaLabelledby = element.hasAttribute('aria-labelledby');
    const hasTextContent = element.textContent?.trim().length ?? 0 > 0;
    const hasAltText = element.hasAttribute('alt');
    const hasTitle = element.hasAttribute('title');

    const hasAccessibleName =
      hasAriaLabel ||
      hasAriaLabelledby ||
      (roleDefinition.accessibleNameFromContent && hasTextContent) ||
      hasAltText ||
      hasTitle;

    if (!hasAccessibleName) {
      errors.push({
        type: 'role',
        severity: 'error',
        message: `Role "${role}" requires an accessible name`,
        element: element.tagName.toLowerCase(),
        role,
        expectedValue: 'aria-label, aria-labelledby, or appropriate content',
        wcagCriterion: '4.1.2',
      });
    }

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  validateChildrenPresentational(element: HTMLElement, role: ARIARole): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const roleDefinition = getRoleDefinition(role);
    if (!roleDefinition || !roleDefinition.childrenPresentational) {
      return { valid: true, errors, warnings, info };
    }

    // Check if children have semantic roles
    const children = Array.from(element.children);
    for (const child of children) {
      const childRole = child.getAttribute('role');
      if (childRole && childRole !== 'none' && childRole !== 'presentation') {
        warnings.push({
          type: 'role',
          severity: 'warning',
          message: `Children of role="${role}" are presentational and should not have semantic roles`,
          element: element.tagName.toLowerCase(),
          role,
          suggestion: `Remove role="${childRole}" from child element or use role="none"`,
        });
      }
    }

    return {
      valid: true,
      errors,
      warnings,
      info,
    };
  }
}

export const roleValidator = new RoleValidator();
