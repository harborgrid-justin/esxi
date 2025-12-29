/**
 * VoiceOver screen reader simulator (macOS/iOS)
 * Simulates VoiceOver behavior and announcements
 */

import type {
  AccessibilityNode,
  Announcement,
  ScreenReaderState,
} from '../types';
import { AnnouncementGenerator } from '../analyzers/AnnouncementGenerator';

export class VoiceOverSimulator {
  private announcementGenerator = new AnnouncementGenerator();
  private state: ScreenReaderState;
  private platform: 'macOS' | 'iOS';

  constructor(platform: 'macOS' | 'iOS' = 'macOS') {
    this.platform = platform;
    this.state = this.createInitialState();
  }

  /**
   * Create initial screen reader state
   */
  private createInitialState(): ScreenReaderState {
    return {
      currentNode: null,
      navigationMode: 'browse',
      verbosity: 'normal',
      readingMode: 'line',
      navigationPath: null,
      announcementQueue: [],
      history: [],
    };
  }

  /**
   * Navigate to element
   */
  public navigateTo(node: AccessibilityNode): Announcement {
    // Update state
    if (this.state.currentNode) {
      this.state.history.push(this.state.currentNode);
    }
    this.state.currentNode = node;

    // Generate announcement with VoiceOver-specific phrasing
    const announcement = this.announcementGenerator.generate(
      node,
      'VoiceOver',
      'Safari',
      this.state.verbosity
    );

    // VoiceOver uses different announcement order: Role, Name, State
    announcement.text = this.formatVoiceOverAnnouncement(node);

    this.queueAnnouncement(announcement);
    return announcement;
  }

  /**
   * Format announcement in VoiceOver style
   */
  private formatVoiceOverAnnouncement(node: AccessibilityNode): string {
    const parts: string[] = [];

    // VoiceOver announces role first
    const roleName = this.getVoiceOverRoleName(node.role);
    parts.push(roleName);

    // Then name
    if (node.name) {
      parts.push(node.name);
    }

    // Then state
    const state = this.getVoiceOverState(node);
    if (state.length > 0) {
      parts.push(...state);
    }

    // Context for certain elements
    const context = this.getVoiceOverContext(node);
    if (context) {
      parts.push(context);
    }

    // VoiceOver adds hints
    const hint = this.getVoiceOverHint(node);
    if (hint && this.state.verbosity !== 'minimal') {
      parts.push(hint);
    }

    return parts.join(', ');
  }

  /**
   * Get VoiceOver-specific role name
   */
  private getVoiceOverRoleName(role: string): string {
    const roleNames: Record<string, string> = {
      button: 'button',
      link: 'link',
      heading: 'heading',
      textbox: 'text field',
      searchbox: 'search field',
      checkbox: 'checkbox',
      radio: 'radio button',
      combobox: 'pop up button',
      navigation: 'navigation',
      main: 'main',
      banner: 'banner',
      contentinfo: 'content information',
      complementary: 'complementary',
      img: 'image',
      list: 'list',
      listitem: 'item',
    };

    return roleNames[role] || role;
  }

  /**
   * Get VoiceOver state announcements
   */
  private getVoiceOverState(node: AccessibilityNode): string[] {
    const state: string[] = [];

    // Checked state
    if (node.checked === true) {
      state.push('checked');
    } else if (node.checked === false) {
      state.push('unchecked');
    }

    // Selected state
    if (node.selected === true) {
      state.push('selected');
    }

    // Expanded state
    if (node.expanded === true) {
      state.push('expanded');
    } else if (node.expanded === false) {
      state.push('collapsed');
    }

    // Disabled (VoiceOver says "dimmed")
    if (node.disabled) {
      state.push('dimmed');
    }

    // Required
    if (node.required) {
      state.push('required');
    }

    // Invalid
    if (node.invalid) {
      state.push('invalid data');
    }

    // Value for inputs
    if (node.value) {
      state.push(`value ${node.value}`);
    }

    return state;
  }

  /**
   * Get VoiceOver context
   */
  private getVoiceOverContext(node: AccessibilityNode): string | null {
    // Heading level
    if (node.role === 'heading' && node.level) {
      return `level ${node.level}`;
    }

    // List position
    if (node.role === 'listitem' && node.parent) {
      const siblings = node.parent.children.filter(c => c.role === 'listitem');
      const index = siblings.indexOf(node);
      if (index !== -1 && this.state.verbosity === 'verbose') {
        return `${index + 1} of ${siblings.length}`;
      }
    }

    return null;
  }

  /**
   * Get VoiceOver hint (usage instructions)
   */
  private getVoiceOverHint(node: AccessibilityNode): string | null {
    if (this.platform === 'iOS') {
      // iOS VoiceOver hints
      if (node.role === 'button') {
        return 'Double tap to activate';
      }
      if (node.role === 'link') {
        return 'Double tap to follow link';
      }
      if (node.role === 'textbox') {
        return 'Double tap to edit';
      }
      if (node.role === 'checkbox') {
        return 'Double tap to toggle';
      }
    } else {
      // macOS VoiceOver hints
      if (node.role === 'button') {
        return 'Press Control-Option-Space to activate';
      }
      if (node.role === 'link') {
        return 'Press Control-Option-Space to follow link';
      }
      if (node.role === 'textbox') {
        return 'Press Control-Option-Shift-Down to interact';
      }
    }

    return null;
  }

  /**
   * Navigate forward (VO + Right Arrow on macOS, swipe right on iOS)
   */
  public navigateNext(root: AccessibilityNode): Announcement | null {
    if (!this.state.currentNode) {
      const first = this.getFirstNavigableNode(root);
      if (first) {
        return this.navigateTo(first);
      }
      return null;
    }

    const next = this.getNextNode(this.state.currentNode, root);
    if (next) {
      return this.navigateTo(next);
    }

    // VoiceOver plays a "bonk" sound at boundaries
    return {
      text: 'End of content',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'VoiceOver',
      browser: 'Safari',
      verbosity: 'minimal',
    };
  }

  /**
   * Navigate backward (VO + Left Arrow on macOS, swipe left on iOS)
   */
  public navigatePrevious(root: AccessibilityNode): Announcement | null {
    if (!this.state.currentNode) {
      return null;
    }

    const prev = this.getPreviousNode(this.state.currentNode, root);
    if (prev) {
      return this.navigateTo(prev);
    }

    return {
      text: 'Top of content',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'VoiceOver',
      browser: 'Safari',
      verbosity: 'minimal',
    };
  }

  /**
   * Navigate by heading (VO + Command + H)
   */
  public navigateNextHeading(root: AccessibilityNode, level?: number): Announcement | null {
    const headings = this.findHeadings(root, level);
    return this.navigateInList(headings, 'heading');
  }

  /**
   * Navigate by landmark (VO + Command + L on Web Rotor)
   */
  public navigateNextLandmark(root: AccessibilityNode): Announcement | null {
    const landmarks = this.findLandmarks(root);
    return this.navigateInList(landmarks, 'landmark');
  }

  /**
   * Navigate by link (VO + Command + L)
   */
  public navigateNextLink(root: AccessibilityNode): Announcement | null {
    const links = this.findLinks(root);
    return this.navigateInList(links, 'link');
  }

  /**
   * Navigate by form control (VO + Command + J)
   */
  public navigateNextFormControl(root: AccessibilityNode): Announcement | null {
    const controls = this.findFormControls(root);
    return this.navigateInList(controls, 'form control');
  }

  /**
   * Navigate by table (VO + Command + T)
   */
  public navigateNextTable(root: AccessibilityNode): Announcement | null {
    const tables = this.findTables(root);
    return this.navigateInList(tables, 'table');
  }

  /**
   * Navigate by list (VO + Command + X)
   */
  public navigateNextList(root: AccessibilityNode): Announcement | null {
    const lists = this.findLists(root);
    return this.navigateInList(lists, 'list');
  }

  /**
   * Navigate by graphic (VO + Command + G)
   */
  public navigateNextGraphic(root: AccessibilityNode): Announcement | null {
    const graphics = this.findGraphics(root);
    return this.navigateInList(graphics, 'graphic');
  }

  /**
   * Read from current position (VO + A)
   */
  public readFromCurrent(root: AccessibilityNode): Announcement[] {
    const announcements: Announcement[] = [];
    let current = this.state.currentNode;

    if (!current) {
      current = this.getFirstNavigableNode(root);
    }

    while (current) {
      const announcement = this.announcementGenerator.generate(
        current,
        'VoiceOver',
        'Safari',
        this.state.verbosity
      );
      announcement.text = this.formatVoiceOverAnnouncement(current);
      announcements.push(announcement);

      current = this.getNextNode(current, root);
    }

    return announcements;
  }

  /**
   * Open rotor (VO + U) - returns navigation options
   */
  public openRotor(root: AccessibilityNode): Record<string, number> {
    return {
      headings: this.findHeadings(root).length,
      landmarks: this.findLandmarks(root).length,
      links: this.findLinks(root).length,
      'form controls': this.findFormControls(root).length,
      tables: this.findTables(root).length,
      lists: this.findLists(root).length,
      graphics: this.findGraphics(root).length,
    };
  }

  /**
   * Interact with element (VO + Shift + Down Arrow)
   */
  public interactWith(node: AccessibilityNode): Announcement {
    // Enter interaction mode for containers
    const interactableRoles = ['application', 'toolbar', 'menu', 'tree', 'grid', 'tablist'];

    if (interactableRoles.includes(node.role)) {
      this.state.navigationMode = 'focus';
      return {
        text: `In ${node.name || node.role}. To stop interacting press Control-Option-Shift-Up Arrow`,
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'VoiceOver',
        browser: 'Safari',
        verbosity: 'normal',
      };
    }

    return this.navigateTo(node);
  }

  /**
   * Stop interacting (VO + Shift + Up Arrow)
   */
  public stopInteracting(): Announcement {
    this.state.navigationMode = 'browse';
    return {
      text: 'Out of web content',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'VoiceOver',
      browser: 'Safari',
      verbosity: 'minimal',
    };
  }

  /**
   * Set verbosity
   */
  public setVerbosity(level: 'minimal' | 'normal' | 'verbose'): void {
    this.state.verbosity = level;
  }

  /**
   * Get current state
   */
  public getState(): ScreenReaderState {
    return { ...this.state };
  }

  // Helper methods

  private getFirstNavigableNode(root: AccessibilityNode): AccessibilityNode | null {
    if (this.isNavigable(root)) {
      return root;
    }

    for (const child of root.children) {
      const result = this.getFirstNavigableNode(child);
      if (result) {
        return result;
      }
    }

    return null;
  }

  private getNextNode(current: AccessibilityNode, root: AccessibilityNode): AccessibilityNode | null {
    if (current.children.length > 0) {
      const first = this.getFirstNavigableNode(current.children[0]);
      if (first) {
        return first;
      }
    }

    let node: AccessibilityNode | null = current;
    while (node) {
      if (node.parent) {
        const siblings = node.parent.children;
        const index = siblings.indexOf(node);
        for (let i = index + 1; i < siblings.length; i++) {
          const next = this.getFirstNavigableNode(siblings[i]);
          if (next) {
            return next;
          }
        }
      }
      node = node.parent;
    }

    return null;
  }

  private getPreviousNode(current: AccessibilityNode, root: AccessibilityNode): AccessibilityNode | null {
    if (!current.parent) {
      return null;
    }

    const siblings = current.parent.children;
    const index = siblings.indexOf(current);

    if (index > 0) {
      return this.getLastNavigableDescendant(siblings[index - 1]) || siblings[index - 1];
    }

    if (this.isNavigable(current.parent)) {
      return current.parent;
    }

    return this.getPreviousNode(current.parent, root);
  }

  private getLastNavigableDescendant(node: AccessibilityNode): AccessibilityNode | null {
    if (node.children.length === 0) {
      return this.isNavigable(node) ? node : null;
    }

    for (let i = node.children.length - 1; i >= 0; i--) {
      const result = this.getLastNavigableDescendant(node.children[i]);
      if (result) {
        return result;
      }
    }

    return this.isNavigable(node) ? node : null;
  }

  private isNavigable(node: AccessibilityNode): boolean {
    // VoiceOver navigates slightly differently - it includes more elements
    return !node.hidden && (node.focusable || Boolean(node.name) || this.isStructuralElement(node));
  }

  private isStructuralElement(node: AccessibilityNode): boolean {
    const structuralRoles = ['list', 'listitem', 'table', 'row', 'cell', 'heading', 'navigation', 'main', 'banner'];
    return structuralRoles.includes(node.role);
  }

  private isAfter(node: AccessibilityNode, reference: AccessibilityNode): boolean {
    return node.boundingBox.top > reference.boundingBox.top ||
           (node.boundingBox.top === reference.boundingBox.top &&
            node.boundingBox.left > reference.boundingBox.left);
  }

  private findHeadings(root: AccessibilityNode, level?: number): AccessibilityNode[] {
    const headings: AccessibilityNode[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (node.role === 'heading' && !node.hidden) {
        if (level === undefined || node.level === level) {
          headings.push(node);
        }
      }
      node.children.forEach(traverse);
    };

    traverse(root);
    return headings;
  }

  private findLandmarks(root: AccessibilityNode): AccessibilityNode[] {
    const landmarks: AccessibilityNode[] = [];
    const landmarkRoles = ['banner', 'navigation', 'main', 'complementary', 'contentinfo', 'region', 'search'];

    const traverse = (node: AccessibilityNode) => {
      if (landmarkRoles.includes(node.role) && !node.hidden) {
        landmarks.push(node);
      }
      node.children.forEach(traverse);
    };

    traverse(root);
    return landmarks;
  }

  private findLinks(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'link');
  }

  private findFormControls(root: AccessibilityNode): AccessibilityNode[] {
    const formRoles = ['textbox', 'searchbox', 'checkbox', 'radio', 'combobox', 'button', 'slider', 'spinbutton'];
    return this.findByRoles(root, formRoles);
  }

  private findTables(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'table');
  }

  private findLists(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'list');
  }

  private findGraphics(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'img');
  }

  private findByRole(root: AccessibilityNode, role: string): AccessibilityNode[] {
    const results: AccessibilityNode[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (node.role === role && !node.hidden) {
        results.push(node);
      }
      node.children.forEach(traverse);
    };

    traverse(root);
    return results;
  }

  private findByRoles(root: AccessibilityNode, roles: string[]): AccessibilityNode[] {
    const results: AccessibilityNode[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (roles.includes(node.role) && !node.hidden) {
        results.push(node);
      }
      node.children.forEach(traverse);
    };

    traverse(root);
    return results;
  }

  private navigateInList(nodes: AccessibilityNode[], typeName: string): Announcement | null {
    if (nodes.length === 0) {
      return {
        text: `No ${typeName}s`,
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'VoiceOver',
        browser: 'Safari',
        verbosity: 'minimal',
      };
    }

    if (!this.state.currentNode) {
      return this.navigateTo(nodes[0]);
    }

    const currentIndex = nodes.indexOf(this.state.currentNode);
    if (currentIndex === -1) {
      const next = nodes.find(n => this.isAfter(n, this.state.currentNode!));
      if (next) {
        return this.navigateTo(next);
      }
    } else if (currentIndex < nodes.length - 1) {
      return this.navigateTo(nodes[currentIndex + 1]);
    }

    return {
      text: `No more ${typeName}s`,
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'VoiceOver',
      browser: 'Safari',
      verbosity: 'minimal',
    };
  }

  private queueAnnouncement(announcement: Announcement): void {
    this.state.announcementQueue.push(announcement);
  }
}
