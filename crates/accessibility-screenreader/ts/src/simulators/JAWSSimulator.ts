/**
 * JAWS screen reader simulator
 * Simulates JAWS behavior and announcements
 */

import type {
  AccessibilityNode,
  Announcement,
  ScreenReaderState,
} from '../types';
import { AnnouncementGenerator } from '../analyzers/AnnouncementGenerator';

export class JAWSSimulator {
  private announcementGenerator = new AnnouncementGenerator();
  private state: ScreenReaderState;

  constructor() {
    this.state = this.createInitialState();
  }

  /**
   * Create initial screen reader state
   */
  private createInitialState(): ScreenReaderState {
    return {
      currentNode: null,
      navigationMode: 'browse', // Virtual PC cursor mode
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

    // Generate announcement with JAWS-specific phrasing
    const announcement = this.announcementGenerator.generate(
      node,
      'JAWS',
      'Chrome',
      this.state.verbosity
    );

    // JAWS adds specific context
    announcement.text = this.enhanceJAWSAnnouncement(announcement.text, node);

    this.queueAnnouncement(announcement);
    return announcement;
  }

  /**
   * Enhance announcement with JAWS-specific details
   */
  private enhanceJAWSAnnouncement(text: string, node: AccessibilityNode): string {
    const parts = [text];

    // JAWS announces form mode entry
    if (node.focusable && this.isFormControl(node)) {
      if (this.state.navigationMode === 'browse') {
        parts.push('To activate press Enter');
      }
    }

    // JAWS announces table navigation hints
    if (node.role === 'cell' || node.role === 'gridcell') {
      parts.push('Use Control + Alt + Arrow keys to navigate table');
    }

    // JAWS announces link activation hint
    if (node.role === 'link') {
      parts.push('To activate press Enter');
    }

    // JAWS announces clickable items
    if (node.role === 'button') {
      parts.push('To activate press Space bar');
    }

    return parts.join('. ');
  }

  /**
   * Navigate with arrow keys (JAWS uses different navigation than NVDA)
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

    return {
      text: 'End of document',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'JAWS',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Navigate to previous element
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
      text: 'Top of document',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'JAWS',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Navigate by heading (JAWS uses H key)
   */
  public navigateNextHeading(root: AccessibilityNode, level?: number): Announcement | null {
    const headings = this.findHeadings(root, level);

    if (!this.state.currentNode) {
      if (headings.length > 0) {
        const announcement = this.navigateTo(headings[0]);
        announcement.text = `Heading level ${headings[0].level}, ${announcement.text}`;
        return announcement;
      }
      return null;
    }

    const currentIndex = headings.indexOf(this.state.currentNode);
    if (currentIndex === -1) {
      const next = headings.find(h => this.isAfter(h, this.state.currentNode!));
      if (next) {
        const announcement = this.navigateTo(next);
        announcement.text = `Heading level ${next.level}, ${announcement.text}`;
        return announcement;
      }
    } else if (currentIndex < headings.length - 1) {
      const next = headings[currentIndex + 1];
      const announcement = this.navigateTo(next);
      announcement.text = `Heading level ${next.level}, ${announcement.text}`;
      return announcement;
    }

    const text = level ? `No more level ${level} headings` : 'No more headings';
    return {
      text,
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'JAWS',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Navigate by region/landmark (JAWS uses R key)
   */
  public navigateNextRegion(root: AccessibilityNode): Announcement | null {
    const landmarks = this.findLandmarks(root);
    return this.navigateInList(landmarks, 'region');
  }

  /**
   * Navigate by list (JAWS uses L key)
   */
  public navigateNextList(root: AccessibilityNode): Announcement | null {
    const lists = this.findLists(root);
    return this.navigateInList(lists, 'list');
  }

  /**
   * Navigate by form field (JAWS uses F key)
   */
  public navigateNextFormField(root: AccessibilityNode): Announcement | null {
    const fields = this.findFormFields(root);
    return this.navigateInList(fields, 'form field');
  }

  /**
   * Navigate by button (JAWS uses B key)
   */
  public navigateNextButton(root: AccessibilityNode): Announcement | null {
    const buttons = this.findButtons(root);
    return this.navigateInList(buttons, 'button');
  }

  /**
   * Navigate by link (JAWS uses Tab or K key)
   */
  public navigateNextLink(root: AccessibilityNode, visitedOnly: boolean = false): Announcement | null {
    const links = this.findLinks(root, visitedOnly);
    return this.navigateInList(links, visitedOnly ? 'visited link' : 'link');
  }

  /**
   * Navigate by graphic (JAWS uses G key)
   */
  public navigateNextGraphic(root: AccessibilityNode): Announcement | null {
    const graphics = this.findGraphics(root);
    return this.navigateInList(graphics, 'graphic');
  }

  /**
   * Navigate by table (JAWS uses T key)
   */
  public navigateNextTable(root: AccessibilityNode): Announcement | null {
    const tables = this.findTables(root);
    return this.navigateInList(tables, 'table');
  }

  /**
   * Read current line (JAWS uses Insert+Up Arrow)
   */
  public readCurrentLine(): Announcement | null {
    if (!this.state.currentNode) {
      return null;
    }

    return this.announcementGenerator.generate(
      this.state.currentNode,
      'JAWS',
      'Chrome',
      'normal'
    );
  }

  /**
   * Read to end of document (JAWS uses Insert+Page Down)
   */
  public readToEnd(root: AccessibilityNode): Announcement[] {
    const announcements: Announcement[] = [];
    let current = this.state.currentNode;

    if (!current) {
      current = this.getFirstNavigableNode(root);
    }

    while (current) {
      const announcement = this.announcementGenerator.generate(
        current,
        'JAWS',
        'Chrome',
        this.state.verbosity
      );
      announcements.push(announcement);

      current = this.getNextNode(current, root);
    }

    return announcements;
  }

  /**
   * List headings (JAWS uses Insert+F6)
   */
  public listHeadings(root: AccessibilityNode): string[] {
    const headings = this.findHeadings(root);
    return headings.map(h => `Level ${h.level}: ${h.name || 'Untitled'}`);
  }

  /**
   * List links (JAWS uses Insert+F7)
   */
  public listLinks(root: AccessibilityNode): string[] {
    const links = this.findLinks(root);
    return links.map(l => l.name || 'Untitled link');
  }

  /**
   * List form fields (JAWS uses Insert+F5)
   */
  public listFormFields(root: AccessibilityNode): string[] {
    const fields = this.findFormFields(root);
    return fields.map(f => {
      const type = f.role;
      const name = f.name || 'Unlabeled';
      return `${type}: ${name}`;
    });
  }

  /**
   * Toggle virtual cursor mode (Forms Mode off/on)
   */
  public toggleMode(): void {
    if (this.state.navigationMode === 'browse') {
      this.state.navigationMode = 'forms';
      this.queueAnnouncement({
        text: 'Forms mode on',
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'JAWS',
        browser: 'Chrome',
        verbosity: 'minimal',
      });
    } else {
      this.state.navigationMode = 'browse';
      this.queueAnnouncement({
        text: 'Forms mode off',
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'JAWS',
        browser: 'Chrome',
        verbosity: 'minimal',
      });
    }
  }

  /**
   * Get current state
   */
  public getState(): ScreenReaderState {
    return { ...this.state };
  }

  // Helper methods (similar to NVDA but with JAWS-specific behavior)

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
    return !node.hidden && (node.focusable || Boolean(node.name));
  }

  private isFormControl(node: AccessibilityNode): boolean {
    const formRoles = ['textbox', 'searchbox', 'checkbox', 'radio', 'combobox', 'slider', 'spinbutton', 'button'];
    return formRoles.includes(node.role);
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
    const landmarkRoles = ['banner', 'navigation', 'main', 'complementary', 'contentinfo', 'region', 'search', 'form'];

    const traverse = (node: AccessibilityNode) => {
      if (landmarkRoles.includes(node.role) && !node.hidden) {
        landmarks.push(node);
      }
      node.children.forEach(traverse);
    };

    traverse(root);
    return landmarks;
  }

  private findLinks(root: AccessibilityNode, visitedOnly: boolean = false): AccessibilityNode[] {
    const links: AccessibilityNode[] = [];

    const traverse = (node: AccessibilityNode) => {
      if (node.role === 'link' && !node.hidden) {
        // In a real implementation, would check if link was visited
        // For now, include all links
        if (!visitedOnly) {
          links.push(node);
        }
      }
      node.children.forEach(traverse);
    };

    traverse(root);
    return links;
  }

  private findFormFields(root: AccessibilityNode): AccessibilityNode[] {
    const formRoles = ['textbox', 'searchbox', 'checkbox', 'radio', 'combobox', 'slider', 'spinbutton'];
    return this.findByRoles(root, formRoles);
  }

  private findButtons(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'button');
  }

  private findGraphics(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'img');
  }

  private findTables(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'table');
  }

  private findLists(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'list');
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
        screenReader: 'JAWS',
        browser: 'Chrome',
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
      screenReader: 'JAWS',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  private queueAnnouncement(announcement: Announcement): void {
    this.state.announcementQueue.push(announcement);
  }
}
