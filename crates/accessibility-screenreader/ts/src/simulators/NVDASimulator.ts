/**
 * NVDA screen reader simulator
 * Simulates NVDA behavior and announcements
 */

import type {
  AccessibilityNode,
  Announcement,
  ScreenReaderState,
  NavigationPath,
} from '../types';
import { AnnouncementGenerator } from '../analyzers/AnnouncementGenerator';

export class NVDASimulator {
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

    // Generate announcement
    const announcement = this.announcementGenerator.generate(
      node,
      'NVDA',
      'Chrome',
      this.state.verbosity
    );

    this.queueAnnouncement(announcement);
    return announcement;
  }

  /**
   * Navigate to next element
   */
  public navigateNext(root: AccessibilityNode): Announcement | null {
    if (!this.state.currentNode) {
      // Start at first element
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

    // Announce end of document
    return this.announceEndOfDocument();
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

    // Announce start of document
    return this.announceStartOfDocument();
  }

  /**
   * Navigate by heading
   */
  public navigateNextHeading(root: AccessibilityNode, level?: number): Announcement | null {
    const headings = this.findHeadings(root, level);

    if (!this.state.currentNode) {
      // Go to first heading
      if (headings.length > 0) {
        return this.navigateTo(headings[0]);
      }
      return null;
    }

    const currentIndex = headings.indexOf(this.state.currentNode);
    if (currentIndex === -1) {
      // Not on a heading, go to first heading after current position
      const next = headings.find(h => this.isAfter(h, this.state.currentNode!));
      if (next) {
        return this.navigateTo(next);
      }
    } else if (currentIndex < headings.length - 1) {
      return this.navigateTo(headings[currentIndex + 1]);
    }

    // No more headings
    return this.announceNoMoreHeadings(level);
  }

  /**
   * Navigate by landmark
   */
  public navigateNextLandmark(root: AccessibilityNode): Announcement | null {
    const landmarks = this.findLandmarks(root);

    if (!this.state.currentNode) {
      if (landmarks.length > 0) {
        return this.navigateTo(landmarks[0]);
      }
      return null;
    }

    const currentIndex = landmarks.indexOf(this.state.currentNode);
    if (currentIndex === -1) {
      const next = landmarks.find(l => this.isAfter(l, this.state.currentNode!));
      if (next) {
        return this.navigateTo(next);
      }
    } else if (currentIndex < landmarks.length - 1) {
      return this.navigateTo(landmarks[currentIndex + 1]);
    }

    return this.announceNoMoreLandmarks();
  }

  /**
   * Navigate by link
   */
  public navigateNextLink(root: AccessibilityNode): Announcement | null {
    const links = this.findLinks(root);
    return this.navigateInList(links, 'link');
  }

  /**
   * Navigate by form field
   */
  public navigateNextFormField(root: AccessibilityNode): Announcement | null {
    const fields = this.findFormFields(root);
    return this.navigateInList(fields, 'form field');
  }

  /**
   * Navigate by button
   */
  public navigateNextButton(root: AccessibilityNode): Announcement | null {
    const buttons = this.findButtons(root);
    return this.navigateInList(buttons, 'button');
  }

  /**
   * Navigate by table
   */
  public navigateNextTable(root: AccessibilityNode): Announcement | null {
    const tables = this.findTables(root);
    return this.navigateInList(tables, 'table');
  }

  /**
   * Navigate by list
   */
  public navigateNextList(root: AccessibilityNode): Announcement | null {
    const lists = this.findLists(root);
    return this.navigateInList(lists, 'list');
  }

  /**
   * Read current line
   */
  public readCurrentLine(): Announcement | null {
    if (!this.state.currentNode) {
      return null;
    }

    // NVDA reads the current node's text
    return this.announcementGenerator.generate(
      this.state.currentNode,
      'NVDA',
      'Chrome',
      'normal'
    );
  }

  /**
   * Read from current position to end
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
        'NVDA',
        'Chrome',
        this.state.verbosity
      );
      announcements.push(announcement);

      current = this.getNextNode(current, root);
    }

    return announcements;
  }

  /**
   * Toggle browse/focus mode
   */
  public toggleMode(): void {
    if (this.state.navigationMode === 'browse') {
      this.state.navigationMode = 'focus';
      this.queueAnnouncement({
        text: 'Focus mode',
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'NVDA',
        browser: 'Chrome',
        verbosity: 'minimal',
      });
    } else {
      this.state.navigationMode = 'browse';
      this.queueAnnouncement({
        text: 'Browse mode',
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'NVDA',
        browser: 'Chrome',
        verbosity: 'minimal',
      });
    }
  }

  /**
   * Set verbosity level
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

  /**
   * Helper: Get first navigable node
   */
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

  /**
   * Helper: Get next node in document order
   */
  private getNextNode(current: AccessibilityNode, root: AccessibilityNode): AccessibilityNode | null {
    // Try children first
    if (current.children.length > 0) {
      const first = this.getFirstNavigableNode(current.children[0]);
      if (first) {
        return first;
      }
    }

    // Try siblings
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

  /**
   * Helper: Get previous node in document order
   */
  private getPreviousNode(current: AccessibilityNode, root: AccessibilityNode): AccessibilityNode | null {
    if (!current.parent) {
      return null;
    }

    const siblings = current.parent.children;
    const index = siblings.indexOf(current);

    if (index > 0) {
      // Get last descendant of previous sibling
      return this.getLastNavigableDescendant(siblings[index - 1]) || siblings[index - 1];
    }

    // Go to parent
    if (this.isNavigable(current.parent)) {
      return current.parent;
    }

    return this.getPreviousNode(current.parent, root);
  }

  /**
   * Helper: Get last navigable descendant
   */
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

  /**
   * Helper: Check if node is navigable
   */
  private isNavigable(node: AccessibilityNode): boolean {
    return !node.hidden && (node.focusable || Boolean(node.name));
  }

  /**
   * Helper: Check if node comes after another
   */
  private isAfter(node: AccessibilityNode, reference: AccessibilityNode): boolean {
    // Simplified comparison - in reality would need full document order comparison
    return node.boundingBox.top > reference.boundingBox.top ||
           (node.boundingBox.top === reference.boundingBox.top &&
            node.boundingBox.left > reference.boundingBox.left);
  }

  /**
   * Helper: Find headings
   */
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

  /**
   * Helper: Find landmarks
   */
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

  /**
   * Helper: Find links
   */
  private findLinks(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'link');
  }

  /**
   * Helper: Find form fields
   */
  private findFormFields(root: AccessibilityNode): AccessibilityNode[] {
    const formRoles = ['textbox', 'searchbox', 'checkbox', 'radio', 'combobox', 'slider', 'spinbutton'];
    return this.findByRoles(root, formRoles);
  }

  /**
   * Helper: Find buttons
   */
  private findButtons(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'button');
  }

  /**
   * Helper: Find tables
   */
  private findTables(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'table');
  }

  /**
   * Helper: Find lists
   */
  private findLists(root: AccessibilityNode): AccessibilityNode[] {
    return this.findByRole(root, 'list');
  }

  /**
   * Helper: Find nodes by role
   */
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

  /**
   * Helper: Find nodes by multiple roles
   */
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

  /**
   * Helper: Navigate in list of nodes
   */
  private navigateInList(nodes: AccessibilityNode[], typeName: string): Announcement | null {
    if (nodes.length === 0) {
      return {
        text: `No ${typeName}s`,
        role: 'status',
        name: null,
        state: [],
        properties: [],
        context: [],
        screenReader: 'NVDA',
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
      screenReader: 'NVDA',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Queue announcement
   */
  private queueAnnouncement(announcement: Announcement): void {
    this.state.announcementQueue.push(announcement);
  }

  /**
   * Announce end of document
   */
  private announceEndOfDocument(): Announcement {
    return {
      text: 'End of document',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'NVDA',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Announce start of document
   */
  private announceStartOfDocument(): Announcement {
    return {
      text: 'Start of document',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'NVDA',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Announce no more headings
   */
  private announceNoMoreHeadings(level?: number): Announcement {
    const text = level ? `No more level ${level} headings` : 'No more headings';
    return {
      text,
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'NVDA',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }

  /**
   * Announce no more landmarks
   */
  private announceNoMoreLandmarks(): Announcement {
    return {
      text: 'No more landmarks',
      role: 'status',
      name: null,
      state: [],
      properties: [],
      context: [],
      screenReader: 'NVDA',
      browser: 'Chrome',
      verbosity: 'minimal',
    };
  }
}
