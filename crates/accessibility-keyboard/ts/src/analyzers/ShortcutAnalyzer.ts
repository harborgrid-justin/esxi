/**
 * Shortcut Analyzer
 * Analyzes and detects keyboard shortcuts and potential conflicts
 */

import { KeyboardShortcut } from '../types';

export class ShortcutAnalyzer {
  private browserShortcuts: Set<string> = new Set([
    'Ctrl+T',
    'Ctrl+W',
    'Ctrl+N',
    'Ctrl+Shift+N',
    'Ctrl+R',
    'Ctrl+F',
    'Ctrl+P',
    'Ctrl+S',
    'Ctrl+A',
    'Ctrl+C',
    'Ctrl+V',
    'Ctrl+X',
    'Ctrl+Z',
    'Ctrl+Y',
    'F5',
    'F11',
    'F12',
    'Alt+Left',
    'Alt+Right',
    'Ctrl+Tab',
    'Ctrl+Shift+Tab',
  ]);

  /**
   * Analyzes keyboard shortcuts in the container
   */
  async analyze(container: HTMLElement = document.body): Promise<KeyboardShortcut[]> {
    const shortcuts: KeyboardShortcut[] = [];

    // Find elements with accesskey attribute
    const elementsWithAccessKey = Array.from(
      container.querySelectorAll('[accesskey]')
    ) as HTMLElement[];

    for (const element of elementsWithAccessKey) {
      const accesskey = element.getAttribute('accesskey');
      if (accesskey) {
        const shortcut = this.createShortcutFromAccessKey(element, accesskey);
        shortcuts.push(shortcut);
      }
    }

    // Detect shortcuts from data attributes
    const elementsWithShortcut = Array.from(
      container.querySelectorAll('[data-shortcut], [data-keyboard-shortcut]')
    ) as HTMLElement[];

    for (const element of elementsWithShortcut) {
      const shortcutAttr =
        element.getAttribute('data-shortcut') ||
        element.getAttribute('data-keyboard-shortcut');

      if (shortcutAttr) {
        const shortcut = this.parseShortcut(element, shortcutAttr);
        if (shortcut) {
          shortcuts.push(shortcut);
        }
      }
    }

    // Detect shortcuts from aria-keyshortcuts
    const elementsWithAriaShortcut = Array.from(
      container.querySelectorAll('[aria-keyshortcuts]')
    ) as HTMLElement[];

    for (const element of elementsWithAriaShortcut) {
      const ariaShortcut = element.getAttribute('aria-keyshortcuts');
      if (ariaShortcut) {
        const shortcut = this.parseShortcut(element, ariaShortcut);
        if (shortcut) {
          shortcuts.push(shortcut);
        }
      }
    }

    // Detect conflicts
    this.detectConflicts(shortcuts);

    return shortcuts;
  }

  /**
   * Creates shortcut from accesskey attribute
   */
  private createShortcutFromAccessKey(
    element: HTMLElement,
    accesskey: string
  ): KeyboardShortcut {
    const action = this.getAction(element);
    const key = accesskey.toUpperCase();

    // AccessKey typically uses Alt modifier
    const shortcut: KeyboardShortcut = {
      key,
      modifiers: { alt: true },
      action,
      element,
      conflicts: [],
      isBrowserConflict: false,
      isDocumented: element.hasAttribute('title') || element.hasAttribute('aria-label'),
    };

    // Check for browser conflicts
    const combo = this.formatCombo(key, shortcut.modifiers);
    shortcut.isBrowserConflict = this.browserShortcuts.has(combo);

    return shortcut;
  }

  /**
   * Parses shortcut string
   */
  private parseShortcut(
    element: HTMLElement,
    shortcutStr: string
  ): KeyboardShortcut | null {
    const parts = shortcutStr.split('+').map((p) => p.trim().toLowerCase());
    if (parts.length === 0) return null;

    const modifiers = {
      ctrl: parts.includes('ctrl') || parts.includes('control'),
      alt: parts.includes('alt'),
      shift: parts.includes('shift'),
      meta: parts.includes('meta') || parts.includes('cmd') || parts.includes('command'),
    };

    const key = parts[parts.length - 1].toUpperCase();
    const action = this.getAction(element);

    const shortcut: KeyboardShortcut = {
      key,
      modifiers,
      action,
      element,
      conflicts: [],
      isBrowserConflict: false,
      isDocumented: element.hasAttribute('title') || element.hasAttribute('aria-label'),
    };

    // Check for browser conflicts
    const combo = this.formatCombo(key, modifiers);
    shortcut.isBrowserConflict = this.browserShortcuts.has(combo);

    return shortcut;
  }

  /**
   * Gets the action description for an element
   */
  private getAction(element: HTMLElement): string {
    // Check aria-label
    const ariaLabel = element.getAttribute('aria-label');
    if (ariaLabel) return ariaLabel;

    // Check title
    const title = element.getAttribute('title');
    if (title) return title;

    // Check text content
    const text = element.textContent?.trim();
    if (text && text.length > 0 && text.length < 50) {
      return text;
    }

    // Check data attributes
    const dataAction = element.getAttribute('data-action');
    if (dataAction) return dataAction;

    // Fallback to tag name
    return `${element.tagName.toLowerCase()} element`;
  }

  /**
   * Formats shortcut combination
   */
  private formatCombo(
    key: string,
    modifiers: {
      ctrl?: boolean;
      alt?: boolean;
      shift?: boolean;
      meta?: boolean;
    }
  ): string {
    const parts: string[] = [];
    if (modifiers.ctrl) parts.push('Ctrl');
    if (modifiers.alt) parts.push('Alt');
    if (modifiers.shift) parts.push('Shift');
    if (modifiers.meta) parts.push('Meta');
    parts.push(key);
    return parts.join('+');
  }

  /**
   * Detects conflicts between shortcuts
   */
  private detectConflicts(shortcuts: KeyboardShortcut[]): void {
    const shortcutMap = new Map<string, KeyboardShortcut[]>();

    // Group shortcuts by combination
    for (const shortcut of shortcuts) {
      const combo = this.formatCombo(shortcut.key, shortcut.modifiers);
      const existing = shortcutMap.get(combo) || [];
      existing.push(shortcut);
      shortcutMap.set(combo, existing);
    }

    // Identify conflicts
    for (const [combo, conflictingShortcuts] of shortcutMap.entries()) {
      if (conflictingShortcuts.length > 1) {
        for (const shortcut of conflictingShortcuts) {
          shortcut.conflicts = conflictingShortcuts.filter((s) => s !== shortcut);
        }
      }
    }
  }
}
