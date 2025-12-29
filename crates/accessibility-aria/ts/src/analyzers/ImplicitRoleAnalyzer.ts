/**
 * Implicit ARIA Role Analyzer
 * Determines implicit ARIA roles for HTML elements
 */

import { ARIARole, ImplicitRoleMapping } from '../types';

export class ImplicitRoleAnalyzer {
  private implicitRoleMappings: ImplicitRoleMapping[] = [
    // Semantic sections
    { element: 'article', implicitRole: 'article' },
    { element: 'aside', implicitRole: 'complementary' },
    { element: 'footer', implicitRole: 'contentinfo', conditions: 'when not descendant of article, aside, main, nav, or section' },
    { element: 'header', implicitRole: 'banner', conditions: 'when not descendant of article, aside, main, nav, or section' },
    { element: 'main', implicitRole: 'main' },
    { element: 'nav', implicitRole: 'navigation' },
    { element: 'section', implicitRole: 'region', conditions: 'when it has an accessible name' },

    // Forms
    { element: 'form', implicitRole: 'form', conditions: 'when it has an accessible name' },
    { element: 'button', implicitRole: 'button' },
    { element: 'input', attributes: { type: 'button' }, implicitRole: 'button' },
    { element: 'input', attributes: { type: 'submit' }, implicitRole: 'button' },
    { element: 'input', attributes: { type: 'reset' }, implicitRole: 'button' },
    { element: 'input', attributes: { type: 'checkbox' }, implicitRole: 'checkbox' },
    { element: 'input', attributes: { type: 'radio' }, implicitRole: 'radio' },
    { element: 'input', attributes: { type: 'range' }, implicitRole: 'slider' },
    { element: 'input', attributes: { type: 'number' }, implicitRole: 'spinbutton' },
    { element: 'input', attributes: { type: 'search' }, implicitRole: 'searchbox' },
    { element: 'input', attributes: { type: 'text' }, implicitRole: 'textbox' },
    { element: 'input', attributes: { type: 'email' }, implicitRole: 'textbox' },
    { element: 'input', attributes: { type: 'tel' }, implicitRole: 'textbox' },
    { element: 'input', attributes: { type: 'url' }, implicitRole: 'textbox' },
    { element: 'textarea', implicitRole: 'textbox' },
    { element: 'select', implicitRole: 'listbox', conditions: 'when no multiple attribute and size <= 1' },
    { element: 'select', implicitRole: 'listbox', conditions: 'when multiple attribute or size > 1' },
    { element: 'option', implicitRole: 'option' },
    { element: 'fieldset', implicitRole: 'group' },

    // Links
    { element: 'a', implicitRole: 'link', conditions: 'when it has href attribute' },
    { element: 'area', implicitRole: 'link', conditions: 'when it has href attribute' },

    // Lists
    { element: 'ul', implicitRole: 'list' },
    { element: 'ol', implicitRole: 'list' },
    { element: 'li', implicitRole: 'listitem' },
    { element: 'dl', implicitRole: 'list' },
    { element: 'dt', implicitRole: 'term' },
    { element: 'dd', implicitRole: 'definition' },

    // Tables
    { element: 'table', implicitRole: 'table' },
    { element: 'thead', implicitRole: 'rowgroup' },
    { element: 'tbody', implicitRole: 'rowgroup' },
    { element: 'tfoot', implicitRole: 'rowgroup' },
    { element: 'tr', implicitRole: 'row' },
    { element: 'th', implicitRole: 'columnheader', conditions: 'when descendant of thead or scope="col"' },
    { element: 'th', implicitRole: 'rowheader', conditions: 'when scope="row"' },
    { element: 'td', implicitRole: 'cell' },

    // Text content
    { element: 'h1', implicitRole: 'heading' },
    { element: 'h2', implicitRole: 'heading' },
    { element: 'h3', implicitRole: 'heading' },
    { element: 'h4', implicitRole: 'heading' },
    { element: 'h5', implicitRole: 'heading' },
    { element: 'h6', implicitRole: 'heading' },
    { element: 'hr', implicitRole: 'separator' },
    { element: 'img', implicitRole: 'img', conditions: 'when alt attribute is present and not empty' },
    { element: 'figure', implicitRole: 'figure' },

    // Embedded content
    { element: 'dialog', implicitRole: 'dialog' },
    { element: 'output', implicitRole: 'status' },
    { element: 'progress', implicitRole: 'progressbar' },
    { element: 'meter', implicitRole: 'meter' },

    // Document structure
    { element: 'address', implicitRole: 'group' },
    { element: 'details', implicitRole: 'group' },
    { element: 'summary', implicitRole: 'button' },
  ];

  getImplicitRole(element: HTMLElement): ARIARole | null {
    const tagName = element.tagName.toLowerCase();

    // Check for explicit role first
    const explicitRole = element.getAttribute('role');
    if (explicitRole) {
      return null; // Has explicit role, so no implicit role applies
    }

    // Special cases with conditions
    if (tagName === 'header' || tagName === 'footer') {
      if (this.isInSectioningContent(element)) {
        return null; // No implicit role when in sectioning content
      }
      return tagName === 'header' ? 'banner' : 'contentinfo';
    }

    if (tagName === 'section') {
      if (this.hasAccessibleName(element)) {
        return 'region';
      }
      return null;
    }

    if (tagName === 'form') {
      if (this.hasAccessibleName(element)) {
        return 'form';
      }
      return null;
    }

    if (tagName === 'a' || tagName === 'area') {
      if (element.hasAttribute('href')) {
        return 'link';
      }
      return null;
    }

    if (tagName === 'img') {
      const alt = element.getAttribute('alt');
      if (alt !== null && alt.trim() !== '') {
        return 'img';
      }
      if (alt === '') {
        return 'presentation';
      }
      return 'img';
    }

    if (tagName === 'input') {
      return this.getInputImplicitRole(element as HTMLInputElement);
    }

    if (tagName === 'select') {
      return 'listbox';
    }

    if (tagName === 'th') {
      const scope = element.getAttribute('scope');
      if (scope === 'row') {
        return 'rowheader';
      }
      // Check if in thead or scope="col"
      if (element.closest('thead') || scope === 'col') {
        return 'columnheader';
      }
      return 'cell';
    }

    // Simple mappings
    const mapping = this.implicitRoleMappings.find(m => {
      if (m.element !== tagName) return false;
      if (!m.attributes) return true;

      return Object.entries(m.attributes).every(([attr, value]) =>
        element.getAttribute(attr) === value
      );
    });

    return mapping?.implicitRole || null;
  }

  private getInputImplicitRole(input: HTMLInputElement): ARIARole | null {
    const type = input.type?.toLowerCase() || 'text';

    const typeRoleMap: Record<string, ARIARole> = {
      'button': 'button',
      'submit': 'button',
      'reset': 'button',
      'checkbox': 'checkbox',
      'radio': 'radio',
      'range': 'slider',
      'number': 'spinbutton',
      'search': 'searchbox',
      'text': 'textbox',
      'email': 'textbox',
      'tel': 'textbox',
      'url': 'textbox',
      'password': 'textbox',
    };

    return typeRoleMap[type] || null;
  }

  private isInSectioningContent(element: HTMLElement): boolean {
    const sectioningElements = ['article', 'aside', 'nav', 'section'];
    let parent = element.parentElement;

    while (parent) {
      if (sectioningElements.includes(parent.tagName.toLowerCase())) {
        return true;
      }
      parent = parent.parentElement;
    }

    return false;
  }

  private hasAccessibleName(element: HTMLElement): boolean {
    return !!(
      element.hasAttribute('aria-label') ||
      element.hasAttribute('aria-labelledby') ||
      element.hasAttribute('title')
    );
  }

  getImplicitAriaLevel(element: HTMLElement): number | null {
    const tagName = element.tagName.toLowerCase();

    if (tagName.match(/^h[1-6]$/)) {
      return parseInt(tagName.substring(1), 10);
    }

    return null;
  }

  getImplicitOrientation(element: HTMLElement): 'horizontal' | 'vertical' | null {
    const role = element.getAttribute('role') as ARIARole | null;

    const horizontalDefaults: ARIARole[] = ['menubar', 'slider', 'tablist', 'toolbar'];
    const verticalDefaults: ARIARole[] = ['menu', 'scrollbar'];

    if (role) {
      if (horizontalDefaults.includes(role)) return 'horizontal';
      if (verticalDefaults.includes(role)) return 'vertical';
    }

    return null;
  }

  getAllImplicitRoleMappings(): ImplicitRoleMapping[] {
    return [...this.implicitRoleMappings];
  }
}

export const implicitRoleAnalyzer = new ImplicitRoleAnalyzer();
