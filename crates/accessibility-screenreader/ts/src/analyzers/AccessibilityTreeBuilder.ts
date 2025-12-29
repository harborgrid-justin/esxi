/**
 * Builds an accessibility tree from the DOM
 * Mimics how screen readers construct their internal representation
 */

import type { AccessibilityNode, AriaRole } from '../types';

export class AccessibilityTreeBuilder {
  private nodeIdCounter = 0;
  private nodeCache = new Map<Element, AccessibilityNode>();

  /**
   * Build accessibility tree from a root element
   */
  public buildTree(root: Element = document.body): AccessibilityNode {
    this.nodeCache.clear();
    this.nodeIdCounter = 0;
    return this.buildNode(root, null);
  }

  /**
   * Build a single accessibility node
   */
  private buildNode(element: Element, parent: AccessibilityNode | null): AccessibilityNode {
    // Check cache
    if (this.nodeCache.has(element)) {
      return this.nodeCache.get(element)!;
    }

    const node: AccessibilityNode = {
      id: `a11y-node-${this.nodeIdCounter++}`,
      element,
      role: this.getRole(element),
      name: this.computeName(element),
      description: this.computeDescription(element),
      value: this.computeValue(element),
      level: this.getLevel(element),
      focusable: this.isFocusable(element),
      hidden: this.isHidden(element),
      disabled: this.isDisabled(element),
      readonly: this.isReadonly(element),
      required: this.isRequired(element),
      invalid: this.isInvalid(element),
      expanded: this.getExpanded(element),
      selected: this.getSelected(element),
      checked: this.getChecked(element),
      pressed: this.getPressed(element),
      current: this.getCurrent(element),
      live: this.getLive(element),
      atomic: this.getAtomic(element),
      relevant: this.getRelevant(element),
      busy: this.getBusy(element),
      controls: this.getRelatedIds(element, 'aria-controls'),
      describedBy: this.getRelatedIds(element, 'aria-describedby'),
      labelledBy: this.getRelatedIds(element, 'aria-labelledby'),
      owns: this.getRelatedIds(element, 'aria-owns'),
      flowTo: this.getRelatedIds(element, 'aria-flowto'),
      children: [],
      parent,
      boundingBox: element.getBoundingClientRect(),
      tabIndex: this.getTabIndex(element),
    };

    this.nodeCache.set(element, node);

    // Build children
    if (!this.isLeafNode(element)) {
      node.children = this.buildChildren(element, node);
    }

    return node;
  }

  /**
   * Build child nodes
   */
  private buildChildren(element: Element, parent: AccessibilityNode): AccessibilityNode[] {
    const children: AccessibilityNode[] = [];

    // Check for aria-owns relationships
    const owns = element.getAttribute('aria-owns');
    if (owns) {
      const ownedIds = owns.split(/\s+/);
      for (const id of ownedIds) {
        const ownedElement = document.getElementById(id);
        if (ownedElement) {
          children.push(this.buildNode(ownedElement, parent));
        }
      }
    }

    // Process natural children
    for (let i = 0; i < element.children.length; i++) {
      const child = element.children[i];

      // Skip if hidden from accessibility tree
      if (this.isHiddenFromTree(child)) {
        continue;
      }

      children.push(this.buildNode(child, parent));
    }

    return children;
  }

  /**
   * Get the accessible role of an element
   */
  private getRole(element: Element): AriaRole | string {
    // Explicit ARIA role
    const ariaRole = element.getAttribute('role');
    if (ariaRole) {
      return ariaRole as AriaRole;
    }

    // Implicit roles from HTML semantics
    const tagName = element.tagName.toLowerCase();
    const type = element.getAttribute('type');

    const implicitRoles: Record<string, string> = {
      a: element.hasAttribute('href') ? 'link' : 'generic',
      article: 'article',
      aside: 'complementary',
      button: 'button',
      dialog: 'dialog',
      footer: 'contentinfo',
      form: 'form',
      h1: 'heading',
      h2: 'heading',
      h3: 'heading',
      h4: 'heading',
      h5: 'heading',
      h6: 'heading',
      header: 'banner',
      img: 'img',
      input: this.getInputRole(type),
      li: 'listitem',
      main: 'main',
      nav: 'navigation',
      ol: 'list',
      section: 'region',
      select: 'combobox',
      table: 'table',
      textarea: 'textbox',
      ul: 'list',
    };

    return implicitRoles[tagName] || 'generic';
  }

  /**
   * Get role for input elements based on type
   */
  private getInputRole(type: string | null): string {
    const inputRoles: Record<string, string> = {
      button: 'button',
      checkbox: 'checkbox',
      email: 'textbox',
      number: 'spinbutton',
      radio: 'radio',
      range: 'slider',
      search: 'searchbox',
      submit: 'button',
      tel: 'textbox',
      text: 'textbox',
      url: 'textbox',
    };

    return inputRoles[type || 'text'] || 'textbox';
  }

  /**
   * Compute accessible name using the accessible name computation algorithm
   */
  private computeName(element: Element): string | null {
    // aria-labelledby
    const labelledBy = element.getAttribute('aria-labelledby');
    if (labelledBy) {
      const ids = labelledBy.split(/\s+/);
      const names = ids
        .map(id => document.getElementById(id)?.textContent?.trim())
        .filter(Boolean);
      if (names.length > 0) {
        return names.join(' ');
      }
    }

    // aria-label
    const ariaLabel = element.getAttribute('aria-label');
    if (ariaLabel) {
      return ariaLabel.trim();
    }

    // Label element
    if (element instanceof HTMLInputElement ||
        element instanceof HTMLSelectElement ||
        element instanceof HTMLTextAreaElement) {
      const labels = (element as HTMLInputElement).labels;
      if (labels && labels.length > 0) {
        return labels[0].textContent?.trim() || null;
      }

      // Label with for attribute
      const id = element.id;
      if (id) {
        const label = document.querySelector(`label[for="${id}"]`);
        if (label) {
          return label.textContent?.trim() || null;
        }
      }
    }

    // Alt text for images
    if (element instanceof HTMLImageElement) {
      return element.alt || null;
    }

    // Title attribute
    const title = element.getAttribute('title');
    if (title) {
      return title.trim();
    }

    // Text content for certain roles
    const role = this.getRole(element);
    if (['button', 'link', 'heading', 'cell', 'columnheader', 'rowheader'].includes(role)) {
      return element.textContent?.trim() || null;
    }

    // Placeholder (last resort)
    if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
      return element.placeholder || null;
    }

    return null;
  }

  /**
   * Compute accessible description
   */
  private computeDescription(element: Element): string | null {
    // aria-describedby
    const describedBy = element.getAttribute('aria-describedby');
    if (describedBy) {
      const ids = describedBy.split(/\s+/);
      const descriptions = ids
        .map(id => document.getElementById(id)?.textContent?.trim())
        .filter(Boolean);
      if (descriptions.length > 0) {
        return descriptions.join(' ');
      }
    }

    // aria-description
    const ariaDescription = element.getAttribute('aria-description');
    if (ariaDescription) {
      return ariaDescription.trim();
    }

    return null;
  }

  /**
   * Compute accessible value
   */
  private computeValue(element: Element): string | null {
    if (element instanceof HTMLInputElement ||
        element instanceof HTMLSelectElement ||
        element instanceof HTMLTextAreaElement) {
      return element.value || null;
    }

    const ariaValueNow = element.getAttribute('aria-valuenow');
    if (ariaValueNow) {
      const ariaValueText = element.getAttribute('aria-valuetext');
      return ariaValueText || ariaValueNow;
    }

    return null;
  }

  /**
   * Get heading level
   */
  private getLevel(element: Element): number | undefined {
    const role = this.getRole(element);

    if (role === 'heading') {
      const ariaLevel = element.getAttribute('aria-level');
      if (ariaLevel) {
        return parseInt(ariaLevel, 10);
      }

      const tagMatch = element.tagName.match(/^H([1-6])$/i);
      if (tagMatch) {
        return parseInt(tagMatch[1], 10);
      }

      return 2; // Default heading level
    }

    return undefined;
  }

  /**
   * Check if element is focusable
   */
  private isFocusable(element: Element): boolean {
    const tabIndex = this.getTabIndex(element);
    if (tabIndex !== null && tabIndex >= 0) {
      return true;
    }

    const focusableTags = ['a', 'button', 'input', 'select', 'textarea'];
    const tagName = element.tagName.toLowerCase();

    if (focusableTags.includes(tagName)) {
      if (element instanceof HTMLInputElement ||
          element instanceof HTMLButtonElement ||
          element instanceof HTMLSelectElement ||
          element instanceof HTMLTextAreaElement) {
        return !element.disabled;
      }
      if (tagName === 'a') {
        return element.hasAttribute('href');
      }
      return true;
    }

    return false;
  }

  /**
   * Check if element is hidden from screen readers
   */
  private isHidden(element: Element): boolean {
    // aria-hidden
    if (element.getAttribute('aria-hidden') === 'true') {
      return true;
    }

    // CSS visibility
    const computed = window.getComputedStyle(element);
    if (computed.display === 'none' || computed.visibility === 'hidden') {
      return true;
    }

    // Hidden attribute
    if (element.hasAttribute('hidden')) {
      return true;
    }

    return false;
  }

  /**
   * Check if element is hidden from accessibility tree
   */
  private isHiddenFromTree(element: Element): boolean {
    // role="none" or role="presentation"
    const role = element.getAttribute('role');
    if (role === 'none' || role === 'presentation') {
      return true;
    }

    return this.isHidden(element);
  }

  /**
   * Check if element is disabled
   */
  private isDisabled(element: Element): boolean {
    const ariaDisabled = element.getAttribute('aria-disabled');
    if (ariaDisabled === 'true') {
      return true;
    }

    if (element instanceof HTMLButtonElement ||
        element instanceof HTMLInputElement ||
        element instanceof HTMLSelectElement ||
        element instanceof HTMLTextAreaElement ||
        element instanceof HTMLFieldSetElement) {
      return element.disabled;
    }

    return false;
  }

  /**
   * Check if element is readonly
   */
  private isReadonly(element: Element): boolean {
    const ariaReadonly = element.getAttribute('aria-readonly');
    if (ariaReadonly === 'true') {
      return true;
    }

    if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
      return element.readOnly;
    }

    return false;
  }

  /**
   * Check if element is required
   */
  private isRequired(element: Element): boolean {
    const ariaRequired = element.getAttribute('aria-required');
    if (ariaRequired === 'true') {
      return true;
    }

    if (element instanceof HTMLInputElement ||
        element instanceof HTMLSelectElement ||
        element instanceof HTMLTextAreaElement) {
      return element.required;
    }

    return false;
  }

  /**
   * Check if element is invalid
   */
  private isInvalid(element: Element): boolean {
    const ariaInvalid = element.getAttribute('aria-invalid');
    return ariaInvalid === 'true' || ariaInvalid === 'grammar' || ariaInvalid === 'spelling';
  }

  /**
   * Get expanded state
   */
  private getExpanded(element: Element): boolean | undefined {
    const ariaExpanded = element.getAttribute('aria-expanded');
    if (ariaExpanded === 'true') return true;
    if (ariaExpanded === 'false') return false;
    return undefined;
  }

  /**
   * Get selected state
   */
  private getSelected(element: Element): boolean | undefined {
    const ariaSelected = element.getAttribute('aria-selected');
    if (ariaSelected === 'true') return true;
    if (ariaSelected === 'false') return false;
    return undefined;
  }

  /**
   * Get checked state
   */
  private getChecked(element: Element): boolean | 'mixed' | undefined {
    const ariaChecked = element.getAttribute('aria-checked');
    if (ariaChecked === 'true') return true;
    if (ariaChecked === 'false') return false;
    if (ariaChecked === 'mixed') return 'mixed';

    if (element instanceof HTMLInputElement &&
        (element.type === 'checkbox' || element.type === 'radio')) {
      return element.checked;
    }

    return undefined;
  }

  /**
   * Get pressed state
   */
  private getPressed(element: Element): boolean | 'mixed' | undefined {
    const ariaPressed = element.getAttribute('aria-pressed');
    if (ariaPressed === 'true') return true;
    if (ariaPressed === 'false') return false;
    if (ariaPressed === 'mixed') return 'mixed';
    return undefined;
  }

  /**
   * Get current state
   */
  private getCurrent(element: Element): string | boolean | undefined {
    const ariaCurrent = element.getAttribute('aria-current');
    if (ariaCurrent) {
      return ariaCurrent === 'true' ? true : ariaCurrent;
    }
    return undefined;
  }

  /**
   * Get live region politeness
   */
  private getLive(element: Element): 'off' | 'polite' | 'assertive' | undefined {
    const ariaLive = element.getAttribute('aria-live');
    if (ariaLive === 'polite' || ariaLive === 'assertive' || ariaLive === 'off') {
      return ariaLive;
    }

    // Implicit live regions
    const role = this.getRole(element);
    if (role === 'alert') return 'assertive';
    if (role === 'status' || role === 'log') return 'polite';

    return undefined;
  }

  /**
   * Get atomic property
   */
  private getAtomic(element: Element): boolean | undefined {
    const ariaAtomic = element.getAttribute('aria-atomic');
    if (ariaAtomic === 'true') return true;
    if (ariaAtomic === 'false') return false;
    return undefined;
  }

  /**
   * Get relevant property
   */
  private getRelevant(element: Element): string | undefined {
    return element.getAttribute('aria-relevant') || undefined;
  }

  /**
   * Get busy state
   */
  private getBusy(element: Element): boolean | undefined {
    const ariaBusy = element.getAttribute('aria-busy');
    if (ariaBusy === 'true') return true;
    if (ariaBusy === 'false') return false;
    return undefined;
  }

  /**
   * Get related element IDs
   */
  private getRelatedIds(element: Element, attribute: string): string[] | undefined {
    const value = element.getAttribute(attribute);
    if (value) {
      return value.split(/\s+/).filter(Boolean);
    }
    return undefined;
  }

  /**
   * Get tab index
   */
  private getTabIndex(element: Element): number {
    const tabIndex = element.getAttribute('tabindex');
    if (tabIndex !== null) {
      return parseInt(tabIndex, 10);
    }

    // Naturally focusable elements have implicit tabindex of 0
    if (this.isNaturallyFocusable(element)) {
      return 0;
    }

    return -1;
  }

  /**
   * Check if element is naturally focusable
   */
  private isNaturallyFocusable(element: Element): boolean {
    const tagName = element.tagName.toLowerCase();

    if (tagName === 'a' || tagName === 'area') {
      return element.hasAttribute('href');
    }

    if (tagName === 'button' || tagName === 'input' ||
        tagName === 'select' || tagName === 'textarea') {
      if (element instanceof HTMLButtonElement ||
          element instanceof HTMLInputElement ||
          element instanceof HTMLSelectElement ||
          element instanceof HTMLTextAreaElement) {
        return !element.disabled;
      }
    }

    return false;
  }

  /**
   * Check if node should be a leaf (no children in a11y tree)
   */
  private isLeafNode(element: Element): boolean {
    const role = this.getRole(element);

    // Certain roles are always leaves
    const leafRoles = ['img', 'button', 'checkbox', 'radio', 'textbox', 'searchbox', 'slider', 'spinbutton'];
    if (leafRoles.includes(role)) {
      return true;
    }

    // Inputs are always leaves
    if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
      return true;
    }

    return false;
  }

  /**
   * Find node by element
   */
  public findNode(element: Element): AccessibilityNode | null {
    return this.nodeCache.get(element) || null;
  }

  /**
   * Get all focusable nodes in tab order
   */
  public getFocusableNodes(root: AccessibilityNode): AccessibilityNode[] {
    const nodes: AccessibilityNode[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (node.focusable && !node.hidden && node.tabIndex >= 0) {
        nodes.push(node);
      }
      node.children.forEach(traverse);
    };

    traverse(root);

    // Sort by tabindex, then document order
    return nodes.sort((a, b) => {
      if (a.tabIndex !== b.tabIndex) {
        return a.tabIndex - b.tabIndex;
      }
      // Document order is maintained by tree traversal order
      return 0;
    });
  }
}
