/**
 * ARIA State Validator
 * Validates ARIA state attributes and their consistency
 */

import { ARIAState, ARIARole, ValidationResult, ValidationError, ValidationWarning } from '../types';
import { isStateAttribute } from '../rules/ARIAAttributes';

export class StateValidator {
  validate(element: HTMLElement): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];
    const info: any[] = [];

    const role = element.getAttribute('role') as ARIARole | null;

    // Validate aria-checked consistency
    this.validateCheckedState(element, role, errors, warnings);

    // Validate aria-expanded consistency
    this.validateExpandedState(element, role, errors, warnings);

    // Validate aria-selected consistency
    this.validateSelectedState(element, role, errors, warnings);

    // Validate aria-pressed consistency
    this.validatePressedState(element, role, errors, warnings);

    // Validate aria-hidden implications
    this.validateHiddenState(element, errors, warnings);

    // Validate aria-disabled implications
    this.validateDisabledState(element, errors, warnings);

    // Validate aria-invalid with aria-errormessage
    this.validateInvalidState(element, errors, warnings);

    return {
      valid: errors.length === 0,
      errors,
      warnings,
      info,
    };
  }

  private validateCheckedState(
    element: HTMLElement,
    role: ARIARole | null,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const checked = element.getAttribute('aria-checked');
    if (!checked) return;

    const validRoles: ARIARole[] = ['checkbox', 'radio', 'menuitemcheckbox', 'menuitemradio', 'switch'];

    if (role && !validRoles.includes(role)) {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: `aria-checked is not typically used with role="${role}"`,
        element: element.tagName.toLowerCase(),
        role,
        attribute: 'aria-checked',
        suggestion: `aria-checked is typically used with: ${validRoles.join(', ')}`,
      });
    }

    // For radio buttons in a group, check that only one is checked
    if (role === 'radio' && checked === 'true') {
      const radioGroup = this.findRadioGroup(element);
      if (radioGroup) {
        const checkedRadios = radioGroup.querySelectorAll('[role="radio"][aria-checked="true"]');
        if (checkedRadios.length > 1) {
          errors.push({
            type: 'state',
            severity: 'error',
            message: 'Only one radio button in a group can be checked',
            element: element.tagName.toLowerCase(),
            role,
            attribute: 'aria-checked',
            wcagCriterion: '4.1.2',
          });
        }
      }
    }
  }

  private validateExpandedState(
    element: HTMLElement,
    role: ARIARole | null,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const expanded = element.getAttribute('aria-expanded');
    if (!expanded) return;

    // Check for aria-controls when aria-expanded is present
    const controls = element.getAttribute('aria-controls');
    if (!controls) {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: 'aria-expanded should be used with aria-controls to identify controlled element',
        element: element.tagName.toLowerCase(),
        role,
        attribute: 'aria-expanded',
        suggestion: 'Add aria-controls attribute referencing the controlled element',
      });
    }
  }

  private validateSelectedState(
    element: HTMLElement,
    role: ARIARole | null,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const selected = element.getAttribute('aria-selected');
    if (!selected) return;

    const validRoles: ARIARole[] = ['option', 'tab', 'treeitem', 'gridcell', 'row', 'columnheader', 'rowheader'];

    if (role && !validRoles.includes(role)) {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: `aria-selected is not typically used with role="${role}"`,
        element: element.tagName.toLowerCase(),
        role,
        attribute: 'aria-selected',
        suggestion: `aria-selected is typically used with: ${validRoles.join(', ')}`,
      });
    }
  }

  private validatePressedState(
    element: HTMLElement,
    role: ARIARole | null,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const pressed = element.getAttribute('aria-pressed');
    if (!pressed) return;

    if (role !== 'button') {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: `aria-pressed should only be used on button elements`,
        element: element.tagName.toLowerCase(),
        role,
        attribute: 'aria-pressed',
        suggestion: 'Remove aria-pressed or change role to "button"',
      });
    }
  }

  private validateHiddenState(
    element: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const hidden = element.getAttribute('aria-hidden');
    if (hidden !== 'true') return;

    // Check if element or descendants can receive focus
    const focusableSelectors = [
      'a[href]',
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ];

    const hasFocusableContent = focusableSelectors.some(selector =>
      element.matches(selector) || element.querySelector(selector)
    );

    if (hasFocusableContent) {
      errors.push({
        type: 'state',
        severity: 'error',
        message: 'aria-hidden="true" should not be used on focusable elements or their containers',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-hidden',
        wcagCriterion: '4.1.2',
      });
    }

    // Check for aria-labelledby or aria-describedby pointing to hidden element
    const elementId = element.id;
    if (elementId) {
      const referencingElements = document.querySelectorAll(
        `[aria-labelledby~="${elementId}"], [aria-describedby~="${elementId}"]`
      );
      if (referencingElements.length > 0) {
        warnings.push({
          type: 'state',
          severity: 'warning',
          message: 'Element with aria-hidden="true" is referenced by aria-labelledby or aria-describedby',
          element: element.tagName.toLowerCase(),
          attribute: 'aria-hidden',
          suggestion: 'Hidden elements should not be used for accessible names or descriptions',
        });
      }
    }
  }

  private validateDisabledState(
    element: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const disabled = element.getAttribute('aria-disabled');
    const nativeDisabled = (element as HTMLInputElement).disabled;

    if (disabled === 'true' && !nativeDisabled && element.matches('button, input, select, textarea')) {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: 'Use native disabled attribute instead of aria-disabled when possible',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-disabled',
        suggestion: 'Add disabled attribute to native form control',
      });
    }
  }

  private validateInvalidState(
    element: HTMLElement,
    errors: ValidationError[],
    warnings: ValidationWarning[]
  ): void {
    const invalid = element.getAttribute('aria-invalid');
    const errormessage = element.getAttribute('aria-errormessage');

    if ((invalid === 'true' || invalid === 'grammar' || invalid === 'spelling') && !errormessage) {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: 'aria-invalid should be paired with aria-errormessage',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-invalid',
        suggestion: 'Add aria-errormessage attribute to reference error description',
      });
    }

    if (errormessage && invalid === 'false') {
      warnings.push({
        type: 'state',
        severity: 'warning',
        message: 'aria-errormessage present but aria-invalid is "false"',
        element: element.tagName.toLowerCase(),
        attribute: 'aria-errormessage',
        suggestion: 'Remove aria-errormessage or set aria-invalid to "true"',
      });
    }
  }

  private findRadioGroup(radio: HTMLElement): HTMLElement | null {
    let current: HTMLElement | null = radio.parentElement;
    while (current) {
      if (current.getAttribute('role') === 'radiogroup') {
        return current;
      }
      current = current.parentElement;
    }
    return null;
  }
}

export const stateValidator = new StateValidator();
