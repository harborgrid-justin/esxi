/**
 * Analyzes form accessibility
 */

import type {
  AccessibilityNode,
  FormStructure,
  FormFieldInfo,
  FormIssue,
  Announcement,
} from '../types';
import { AnnouncementGenerator } from './AnnouncementGenerator';

export class FormAnalyzer {
  private announcementGenerator = new AnnouncementGenerator();

  /**
   * Analyze form accessibility
   */
  public analyze(root: AccessibilityNode): FormStructure {
    const fields = this.extractFormFields(root);
    const issues = this.detectIssues(fields);
    const score = this.calculateScore(fields, issues);

    return { fields, issues, score };
  }

  /**
   * Extract form fields from tree
   */
  private extractFormFields(root: AccessibilityNode): FormFieldInfo[] {
    const fields: FormFieldInfo[] = [];

    const formRoles = [
      'textbox',
      'searchbox',
      'checkbox',
      'radio',
      'combobox',
      'listbox',
      'slider',
      'spinbutton',
      'switch',
    ];

    const traverse = (node: AccessibilityNode) => {
      if (formRoles.includes(node.role)) {
        const field = this.analyzeField(node);
        fields.push(field);
      }

      node.children.forEach(traverse);
    };

    traverse(root);

    return fields;
  }

  /**
   * Analyze a single form field
   */
  private analyzeField(node: AccessibilityNode): FormFieldInfo {
    const label = node.name;
    const labelMethod = this.determineLabelMethod(node);
    const instructions = this.getInstructions(node);
    const errorMessage = this.getErrorMessage(node);
    const groupLabel = this.getGroupLabel(node);

    // Generate announcement
    const announcement = this.announcementGenerator.generate(
      node,
      'NVDA',
      'Chrome',
      'normal'
    );

    return {
      node,
      label,
      labelMethod,
      instructions,
      errorMessage,
      groupLabel,
      announcement,
    };
  }

  /**
   * Determine how the field is labeled
   */
  private determineLabelMethod(
    node: AccessibilityNode
  ): 'label-element' | 'aria-label' | 'aria-labelledby' | 'title' | 'placeholder' | 'none' {
    const element = node.element;

    // aria-labelledby
    if (element.hasAttribute('aria-labelledby')) {
      return 'aria-labelledby';
    }

    // aria-label
    if (element.hasAttribute('aria-label')) {
      return 'aria-label';
    }

    // Label element
    if (element instanceof HTMLInputElement ||
        element instanceof HTMLSelectElement ||
        element instanceof HTMLTextAreaElement) {
      const labels = (element as HTMLInputElement).labels;
      if (labels && labels.length > 0) {
        return 'label-element';
      }

      const id = element.id;
      if (id) {
        const label = document.querySelector(`label[for="${id}"]`);
        if (label) {
          return 'label-element';
        }
      }
    }

    // Title
    if (element.hasAttribute('title')) {
      return 'title';
    }

    // Placeholder
    if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
      if (element.placeholder) {
        return 'placeholder';
      }
    }

    return 'none';
  }

  /**
   * Get field instructions
   */
  private getInstructions(node: AccessibilityNode): string | null {
    // Check aria-description
    const ariaDescription = node.element.getAttribute('aria-description');
    if (ariaDescription) {
      return ariaDescription;
    }

    // Check aria-describedby
    if (node.describedBy && node.describedBy.length > 0) {
      const descriptions = node.describedBy
        .map(id => document.getElementById(id)?.textContent?.trim())
        .filter(Boolean);
      if (descriptions.length > 0) {
        return descriptions.join(' ');
      }
    }

    return null;
  }

  /**
   * Get error message
   */
  private getErrorMessage(node: AccessibilityNode): string | null {
    if (!node.invalid) {
      return null;
    }

    // Check for error message in aria-describedby
    if (node.describedBy && node.describedBy.length > 0) {
      for (const id of node.describedBy) {
        const element = document.getElementById(id);
        if (element) {
          const role = element.getAttribute('role');
          const ariaLive = element.getAttribute('aria-live');

          // Error messages often have role="alert" or aria-live
          if (role === 'alert' || ariaLive) {
            return element.textContent?.trim() || null;
          }

          // Check for error classes
          if (element.className.match(/error|invalid/i)) {
            return element.textContent?.trim() || null;
          }
        }
      }
    }

    return null;
  }

  /**
   * Get group label for radio/checkbox groups
   */
  private getGroupLabel(node: AccessibilityNode): string | null {
    if (node.role !== 'radio' && node.role !== 'checkbox') {
      return null;
    }

    // Find parent fieldset or radiogroup
    let current = node.parent;
    while (current) {
      if (current.role === 'radiogroup' || current.role === 'group') {
        return current.name;
      }

      if (current.element instanceof HTMLFieldSetElement) {
        const legend = current.element.querySelector('legend');
        return legend?.textContent?.trim() || null;
      }

      current = current.parent;
    }

    return null;
  }

  /**
   * Detect form issues
   */
  private detectIssues(fields: FormFieldInfo[]): FormIssue[] {
    const issues: FormIssue[] = [];

    fields.forEach(field => {
      // Missing label
      if (!field.label || field.label.trim().length === 0) {
        issues.push({
          type: 'missing-label',
          severity: 'critical',
          field,
          description: `${field.node.role} field is missing a label`,
          remediation: 'Add a <label> element, aria-label, or aria-labelledby to identify the field',
        });
      }

      // Placeholder as label
      if (field.labelMethod === 'placeholder') {
        issues.push({
          type: 'placeholder-as-label',
          severity: 'serious',
          field,
          description: `${field.node.role} field uses placeholder as label`,
          remediation: 'Use a visible <label> element. Placeholders disappear when typing and are not reliable labels',
        });
      }

      // Title as label
      if (field.labelMethod === 'title') {
        issues.push({
          type: 'title-as-label',
          severity: 'serious',
          field,
          description: `${field.node.role} field uses title attribute as label`,
          remediation: 'Use a visible <label> element. Title attributes are not consistently announced',
        });
      }

      // Missing error message
      if (field.node.invalid && !field.errorMessage) {
        issues.push({
          type: 'missing-error-message',
          severity: 'serious',
          field,
          description: `Invalid ${field.node.role} field has no error message`,
          remediation: 'Add aria-describedby pointing to an error message element with role="alert"',
        });
      }

      // Generic error
      if (field.errorMessage && this.isGenericError(field.errorMessage)) {
        issues.push({
          type: 'generic-error',
          severity: 'moderate',
          field,
          description: 'Error message is too generic',
          remediation: 'Provide specific, actionable error messages that explain how to fix the error',
        });
      }

      // Missing required indicator
      if (field.node.required && !this.hasRequiredIndicator(field)) {
        issues.push({
          type: 'missing-required',
          severity: 'moderate',
          field,
          description: 'Required field not clearly marked',
          remediation: 'Add aria-required="true" and a visible indicator (* or "required")',
        });
      }

      // Missing instructions for complex fields
      if (this.needsInstructions(field) && !field.instructions) {
        issues.push({
          type: 'missing-instructions',
          severity: 'moderate',
          field,
          description: `${field.node.role} field may need instructions`,
          remediation: 'Add aria-describedby with helpful instructions for complex fields',
        });
      }

      // Unlabeled group
      if ((field.node.role === 'radio' || field.node.role === 'checkbox') && !field.groupLabel) {
        const hasMultipleInGroup = this.hasMultipleSimilarFields(fields, field);
        if (hasMultipleInGroup) {
          issues.push({
            type: 'unlabeled-group',
            severity: 'serious',
            field,
            description: `${field.node.role} group is missing a group label`,
            remediation: 'Wrap group in <fieldset> with <legend>, or use role="radiogroup/group" with aria-label',
          });
        }
      }
    });

    return issues;
  }

  /**
   * Check if error message is generic
   */
  private isGenericError(message: string): boolean {
    const genericPatterns = [
      /^error$/i,
      /^invalid$/i,
      /^required$/i,
      /^this field/i,
      /^please fix/i,
    ];

    return genericPatterns.some(pattern => pattern.test(message.trim()));
  }

  /**
   * Check if field has required indicator
   */
  private hasRequiredIndicator(field: FormFieldInfo): boolean {
    if (field.node.required) {
      return true;
    }

    if (field.label && field.label.includes('*')) {
      return true;
    }

    if (field.label && /required/i.test(field.label)) {
      return true;
    }

    return false;
  }

  /**
   * Check if field needs instructions
   */
  private needsInstructions(field: FormFieldInfo): boolean {
    // Password fields often need instructions
    const element = field.node.element;
    if (element instanceof HTMLInputElement && element.type === 'password') {
      return true;
    }

    // Fields with pattern validation
    if (element instanceof HTMLInputElement && element.pattern) {
      return true;
    }

    // Combobox, slider, spinbutton often need instructions
    if (['combobox', 'slider', 'spinbutton'].includes(field.node.role)) {
      return true;
    }

    return false;
  }

  /**
   * Check if there are multiple similar fields
   */
  private hasMultipleSimilarFields(fields: FormFieldInfo[], field: FormFieldInfo): boolean {
    const similarFields = fields.filter(f => f.node.role === field.node.role);
    return similarFields.length > 1;
  }

  /**
   * Calculate score
   */
  private calculateScore(fields: FormFieldInfo[], issues: FormIssue[]): number {
    if (fields.length === 0) {
      return 100;
    }

    let score = 100;

    // Deduct for issues
    issues.forEach(issue => {
      switch (issue.severity) {
        case 'critical':
          score -= 30;
          break;
        case 'serious':
          score -= 20;
          break;
        case 'moderate':
          score -= 10;
          break;
        case 'minor':
          score -= 5;
          break;
      }
    });

    // Bonus for good labeling
    const properlyLabeled = fields.filter(
      f => f.labelMethod === 'label-element' || f.labelMethod === 'aria-label' || f.labelMethod === 'aria-labelledby'
    ).length;
    const labelingScore = (properlyLabeled / fields.length) * 10;
    score += labelingScore;

    return Math.max(0, Math.min(100, Math.round(score)));
  }
}
