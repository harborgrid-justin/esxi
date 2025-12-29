/**
 * ARIA Utility Functions
 * Helper functions for ARIA validation and analysis
 */

import { ARIARole, ARIAAttribute, ARIATreeNode } from '../types';
import { getRoleDefinition } from '../rules/ARIARoles';
import { getAttributeDefinition } from '../rules/ARIAAttributes';
import { implicitRoleAnalyzer } from '../analyzers/ImplicitRoleAnalyzer';

export function buildARIATree(root: HTMLElement | Document): ARIATreeNode {
  const rootElement = root instanceof Document ? root.documentElement : root;

  function buildNode(element: HTMLElement, parent?: ARIATreeNode): ARIATreeNode {
    const role = element.getAttribute('role') as ARIARole | null;
    const implicitRole = implicitRoleAnalyzer.getImplicitRole(element);

    const attributes: Partial<Record<ARIAAttribute, string>> = {};
    Array.from(element.attributes).forEach(attr => {
      if (attr.name.startsWith('aria-')) {
        attributes[attr.name as ARIAAttribute] = attr.value;
      }
    });

    const node: ARIATreeNode = {
      element: element.tagName.toLowerCase(),
      role: role || implicitRole,
      attributes,
      children: [],
      parent,
      accessibleName: getAccessibleName(element),
      accessibleDescription: getAccessibleDescription(element),
    };

    Array.from(element.children).forEach(child => {
      if (child instanceof HTMLElement) {
        const childNode = buildNode(child, node);
        node.children.push(childNode);
      }
    });

    return node;
  }

  return buildNode(rootElement);
}

export function getAccessibleName(element: HTMLElement): string {
  // Check aria-labelledby (highest priority)
  const labelledby = element.getAttribute('aria-labelledby');
  if (labelledby) {
    const ids = labelledby.split(/\s+/);
    const texts = ids
      .map(id => element.ownerDocument?.getElementById(id)?.textContent?.trim())
      .filter(Boolean);
    if (texts.length > 0) {
      return texts.join(' ');
    }
  }

  // Check aria-label
  const ariaLabel = element.getAttribute('aria-label');
  if (ariaLabel) {
    return ariaLabel.trim();
  }

  // Check alt attribute (for images)
  const alt = element.getAttribute('alt');
  if (alt !== null) {
    return alt.trim();
  }

  // Check title
  const title = element.getAttribute('title');
  if (title) {
    return title.trim();
  }

  // Check content (for elements that allow name from content)
  const role = element.getAttribute('role') as ARIARole | null;
  if (role) {
    const roleDefinition = getRoleDefinition(role);
    if (roleDefinition?.accessibleNameFromContent) {
      return element.textContent?.trim() || '';
    }
  }

  return '';
}

export function getAccessibleDescription(element: HTMLElement): string {
  // Check aria-describedby
  const describedby = element.getAttribute('aria-describedby');
  if (describedby) {
    const ids = describedby.split(/\s+/);
    const texts = ids
      .map(id => element.ownerDocument?.getElementById(id)?.textContent?.trim())
      .filter(Boolean);
    if (texts.length > 0) {
      return texts.join(' ');
    }
  }

  // Check aria-description (ARIA 1.3)
  const ariaDescription = element.getAttribute('aria-description');
  if (ariaDescription) {
    return ariaDescription.trim();
  }

  return '';
}

export function isInteractive(element: HTMLElement): boolean {
  const interactiveTags = ['a', 'button', 'input', 'select', 'textarea', 'details', 'summary'];
  const tagName = element.tagName.toLowerCase();

  if (interactiveTags.includes(tagName)) {
    return true;
  }

  const role = element.getAttribute('role') as ARIARole | null;
  const interactiveRoles: ARIARole[] = [
    'button',
    'link',
    'checkbox',
    'radio',
    'textbox',
    'searchbox',
    'switch',
    'tab',
    'menuitem',
    'menuitemcheckbox',
    'menuitemradio',
    'option',
    'slider',
    'spinbutton',
    'combobox',
  ];

  if (role && interactiveRoles.includes(role)) {
    return true;
  }

  // Check for tabindex
  const tabindex = element.getAttribute('tabindex');
  if (tabindex !== null && tabindex !== '-1') {
    return true;
  }

  return false;
}

export function isFocusable(element: HTMLElement): boolean {
  // Check if element is disabled
  const disabled = element.hasAttribute('disabled') ||
    element.getAttribute('aria-disabled') === 'true';
  if (disabled) {
    return false;
  }

  // Check if element is hidden
  if (element.getAttribute('aria-hidden') === 'true') {
    return false;
  }

  // Check inherently focusable elements
  const focusableTags = ['a', 'button', 'input', 'select', 'textarea'];
  const tagName = element.tagName.toLowerCase();

  if (focusableTags.includes(tagName)) {
    if (tagName === 'a') {
      return element.hasAttribute('href');
    }
    return true;
  }

  // Check tabindex
  const tabindex = element.getAttribute('tabindex');
  if (tabindex !== null) {
    const tabindexValue = parseInt(tabindex, 10);
    return !isNaN(tabindexValue) && tabindexValue >= 0;
  }

  return false;
}

export function getLandmarkLabel(element: HTMLElement, role: ARIARole): string {
  const accessibleName = getAccessibleName(element);

  if (accessibleName) {
    return `${role} "${accessibleName}"`;
  }

  return role;
}

export function findLandmarks(root: HTMLElement | Document): HTMLElement[] {
  const landmarks: HTMLElement[] = [];
  const landmarkRoles: ARIARole[] = [
    'banner',
    'complementary',
    'contentinfo',
    'form',
    'main',
    'navigation',
    'region',
    'search',
  ];

  const landmarkSelectors = [
    'header',
    'footer',
    'nav',
    'main',
    'aside',
    'section',
    'form',
    '[role="banner"]',
    '[role="complementary"]',
    '[role="contentinfo"]',
    '[role="form"]',
    '[role="main"]',
    '[role="navigation"]',
    '[role="region"]',
    '[role="search"]',
  ];

  const elements = (root instanceof Document ? root : root.ownerDocument || root)
    .querySelectorAll(landmarkSelectors.join(', '));

  elements.forEach(el => {
    if (el instanceof HTMLElement) {
      const role = el.getAttribute('role') as ARIARole | null;
      const implicitRole = implicitRoleAnalyzer.getImplicitRole(el);
      const effectiveRole = role || implicitRole;

      if (effectiveRole && landmarkRoles.includes(effectiveRole)) {
        landmarks.push(el);
      }
    }
  });

  return landmarks;
}

export function findHeadings(root: HTMLElement | Document): HTMLElement[] {
  const headings: HTMLElement[] = [];

  const headingSelectors = ['h1', 'h2', 'h3', 'h4', 'h5', 'h6', '[role="heading"]'];
  const elements = (root instanceof Document ? root : root.ownerDocument || root)
    .querySelectorAll(headingSelectors.join(', '));

  elements.forEach(el => {
    if (el instanceof HTMLElement) {
      headings.push(el);
    }
  });

  return headings;
}

export function getHeadingLevel(element: HTMLElement): number {
  const ariaLevel = element.getAttribute('aria-level');
  if (ariaLevel) {
    return parseInt(ariaLevel, 10);
  }

  const tagName = element.tagName.toLowerCase();
  if (tagName.match(/^h[1-6]$/)) {
    return parseInt(tagName.substring(1), 10);
  }

  return 2; // Default for role="heading" without aria-level
}

export function findInteractiveElements(root: HTMLElement | Document): HTMLElement[] {
  const interactive: HTMLElement[] = [];

  const elements = (root instanceof Document ? root : root.ownerDocument || root)
    .querySelectorAll('*');

  elements.forEach(el => {
    if (el instanceof HTMLElement && isInteractive(el)) {
      interactive.push(el);
    }
  });

  return interactive;
}

export function hasAccessibleName(element: HTMLElement): boolean {
  return getAccessibleName(element).length > 0;
}

export function computeAccessibleDescription(element: HTMLElement): string {
  // Compute full accessible description including all sources
  const parts: string[] = [];

  const description = getAccessibleDescription(element);
  if (description) {
    parts.push(description);
  }

  // Add title if not used for accessible name
  const hasNameFromTitle = !element.hasAttribute('aria-label') &&
    !element.hasAttribute('aria-labelledby') &&
    element.hasAttribute('title');

  if (!hasNameFromTitle) {
    const title = element.getAttribute('title');
    if (title && title.trim()) {
      parts.push(title.trim());
    }
  }

  return parts.join(' ');
}

export function formatRoleName(role: ARIARole): string {
  return role
    .split(/(?=[A-Z])/)
    .join(' ')
    .toLowerCase()
    .replace(/^./, str => str.toUpperCase());
}

export function formatAttributeName(attribute: ARIAAttribute): string {
  return attribute
    .replace('aria-', '')
    .split('-')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

export function getElementSelector(element: HTMLElement): string {
  if (element.id) {
    return `#${element.id}`;
  }

  const tagName = element.tagName.toLowerCase();
  const classes = Array.from(element.classList);

  if (classes.length > 0) {
    return `${tagName}.${classes.join('.')}`;
  }

  return tagName;
}

export function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

export function truncate(text: string, maxLength: number = 50): string {
  if (text.length <= maxLength) {
    return text;
  }
  return text.substring(0, maxLength) + '...';
}
