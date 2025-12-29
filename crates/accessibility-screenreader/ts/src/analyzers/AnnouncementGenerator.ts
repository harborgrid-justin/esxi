/**
 * Generates screen reader announcements for accessibility nodes
 */

import type {
  AccessibilityNode,
  Announcement,
  ScreenReaderType,
  BrowserType,
} from '../types';

export class AnnouncementGenerator {
  /**
   * Generate announcement for a node
   */
  public generate(
    node: AccessibilityNode,
    screenReader: ScreenReaderType,
    browser: BrowserType,
    verbosity: 'minimal' | 'normal' | 'verbose' = 'normal'
  ): Announcement {
    const parts: string[] = [];
    const state: string[] = [];
    const properties: string[] = [];
    const context: string[] = [];

    // Name (most important)
    if (node.name) {
      parts.push(node.name);
    }

    // Role
    const roleName = this.getRoleName(node.role, screenReader);
    parts.push(roleName);

    // State
    this.addState(node, state, screenReader);

    // Properties
    this.addProperties(node, properties, screenReader, verbosity);

    // Context
    this.addContext(node, context, screenReader, verbosity);

    // Build announcement text
    const text = this.buildAnnouncementText(
      parts,
      state,
      properties,
      context,
      screenReader,
      verbosity
    );

    return {
      text,
      role: node.role,
      name: node.name,
      state,
      properties,
      context,
      screenReader,
      browser,
      verbosity,
    };
  }

  /**
   * Get human-readable role name
   */
  private getRoleName(role: string, screenReader: ScreenReaderType): string {
    const roleNames: Record<string, Record<ScreenReaderType, string>> = {
      button: {
        NVDA: 'button',
        JAWS: 'button',
        VoiceOver: 'button',
        TalkBack: 'button',
        Narrator: 'button',
      },
      link: {
        NVDA: 'link',
        JAWS: 'link',
        VoiceOver: 'link',
        TalkBack: 'link',
        Narrator: 'link',
      },
      heading: {
        NVDA: 'heading',
        JAWS: 'heading',
        VoiceOver: 'heading',
        TalkBack: 'heading',
        Narrator: 'heading',
      },
      textbox: {
        NVDA: 'edit',
        JAWS: 'edit',
        VoiceOver: 'text field',
        TalkBack: 'edit box',
        Narrator: 'edit',
      },
      checkbox: {
        NVDA: 'checkbox',
        JAWS: 'check box',
        VoiceOver: 'checkbox',
        TalkBack: 'check box',
        Narrator: 'checkbox',
      },
      radio: {
        NVDA: 'radio button',
        JAWS: 'radio button',
        VoiceOver: 'radio button',
        TalkBack: 'radio button',
        Narrator: 'radio button',
      },
      combobox: {
        NVDA: 'combo box',
        JAWS: 'combo box',
        VoiceOver: 'pop up button',
        TalkBack: 'combo box',
        Narrator: 'combo box',
      },
      navigation: {
        NVDA: 'navigation',
        JAWS: 'navigation region',
        VoiceOver: 'navigation',
        TalkBack: 'navigation',
        Narrator: 'navigation landmark',
      },
      main: {
        NVDA: 'main',
        JAWS: 'main region',
        VoiceOver: 'main',
        TalkBack: 'main',
        Narrator: 'main landmark',
      },
      banner: {
        NVDA: 'banner',
        JAWS: 'banner region',
        VoiceOver: 'banner',
        TalkBack: 'banner',
        Narrator: 'banner landmark',
      },
      contentinfo: {
        NVDA: 'content information',
        JAWS: 'content information region',
        VoiceOver: 'content information',
        TalkBack: 'content information',
        Narrator: 'content info landmark',
      },
    };

    const mapping = roleNames[role];
    if (mapping) {
      return mapping[screenReader];
    }

    return role;
  }

  /**
   * Add state information
   */
  private addState(
    node: AccessibilityNode,
    state: string[],
    screenReader: ScreenReaderType
  ): void {
    // Checked
    if (node.checked === true) {
      state.push(screenReader === 'VoiceOver' ? 'checked' : 'checked');
    } else if (node.checked === false) {
      state.push(screenReader === 'VoiceOver' ? 'unchecked' : 'not checked');
    } else if (node.checked === 'mixed') {
      state.push('mixed');
    }

    // Selected
    if (node.selected === true) {
      state.push('selected');
    } else if (node.selected === false && screenReader !== 'VoiceOver') {
      state.push('not selected');
    }

    // Pressed
    if (node.pressed === true) {
      state.push('pressed');
    } else if (node.pressed === false) {
      state.push('not pressed');
    }

    // Expanded
    if (node.expanded === true) {
      state.push(screenReader === 'JAWS' ? 'expanded' : 'expanded');
    } else if (node.expanded === false) {
      state.push(screenReader === 'JAWS' ? 'collapsed' : 'collapsed');
    }

    // Disabled
    if (node.disabled) {
      state.push(screenReader === 'VoiceOver' ? 'dimmed' : 'unavailable');
    }

    // Required
    if (node.required) {
      state.push(screenReader === 'NVDA' ? 'required' : 'required');
    }

    // Invalid
    if (node.invalid) {
      state.push('invalid');
    }

    // Readonly
    if (node.readonly) {
      state.push('read only');
    }
  }

  /**
   * Add property information
   */
  private addProperties(
    node: AccessibilityNode,
    properties: string[],
    screenReader: ScreenReaderType,
    verbosity: 'minimal' | 'normal' | 'verbose'
  ): void {
    // Level (for headings)
    if (node.level && node.role === 'heading') {
      properties.push(`level ${node.level}`);
    }

    // Value
    if (node.value && verbosity !== 'minimal') {
      properties.push(`value: ${node.value}`);
    }

    // Description
    if (node.description && verbosity !== 'minimal') {
      properties.push(node.description);
    }

    // Current
    if (node.current) {
      const currentValue = typeof node.current === 'string' ? node.current : 'true';
      properties.push(`current ${currentValue}`);
    }
  }

  /**
   * Add context information
   */
  private addContext(
    node: AccessibilityNode,
    context: string[],
    screenReader: ScreenReaderType,
    verbosity: 'minimal' | 'normal' | 'verbose'
  ): void {
    if (verbosity === 'minimal') {
      return;
    }

    // List item position
    if (node.role === 'listitem' && node.parent) {
      const siblings = node.parent.children.filter(child => child.role === 'listitem');
      const index = siblings.indexOf(node);
      if (index !== -1 && verbosity === 'verbose') {
        context.push(`${index + 1} of ${siblings.length}`);
      }
    }

    // Group membership
    if (node.parent && ['list', 'menu', 'menubar', 'tablist', 'tree', 'grid'].includes(node.parent.role)) {
      if (verbosity === 'verbose') {
        context.push(`in ${this.getRoleName(node.parent.role, screenReader)}`);
      }
    }

    // Landmark
    const landmarkParent = this.findLandmarkParent(node);
    if (landmarkParent && verbosity === 'verbose') {
      const landmarkName = landmarkParent.name
        ? `${landmarkParent.name} ${this.getRoleName(landmarkParent.role, screenReader)}`
        : this.getRoleName(landmarkParent.role, screenReader);
      context.push(`in ${landmarkName}`);
    }
  }

  /**
   * Find nearest landmark parent
   */
  private findLandmarkParent(node: AccessibilityNode): AccessibilityNode | null {
    const landmarks = ['banner', 'navigation', 'main', 'complementary', 'contentinfo', 'region', 'search', 'form'];

    let current = node.parent;
    while (current) {
      if (landmarks.includes(current.role)) {
        return current;
      }
      current = current.parent;
    }

    return null;
  }

  /**
   * Build announcement text from parts
   */
  private buildAnnouncementText(
    parts: string[],
    state: string[],
    properties: string[],
    context: string[],
    screenReader: ScreenReaderType,
    verbosity: 'minimal' | 'normal' | 'verbose'
  ): string {
    const segments: string[] = [];

    // NVDA/JAWS: Name, Role, State, Properties, Context
    if (screenReader === 'NVDA' || screenReader === 'JAWS' || screenReader === 'Narrator') {
      segments.push(...parts);
      if (state.length > 0) {
        segments.push(state.join(', '));
      }
      if (properties.length > 0 && verbosity !== 'minimal') {
        segments.push(properties.join(', '));
      }
      if (context.length > 0 && verbosity === 'verbose') {
        segments.push(context.join(', '));
      }
    }

    // VoiceOver: Role, Name, State, Properties, Context
    if (screenReader === 'VoiceOver') {
      if (parts.length > 1) {
        segments.push(parts[1]); // Role first
        segments.push(parts[0]); // Then name
      } else {
        segments.push(...parts);
      }
      if (state.length > 0) {
        segments.push(state.join(', '));
      }
      if (properties.length > 0 && verbosity !== 'minimal') {
        segments.push(properties.join(', '));
      }
      if (context.length > 0 && verbosity === 'verbose') {
        segments.push(context.join(', '));
      }
    }

    return segments.filter(Boolean).join(', ');
  }

  /**
   * Generate announcement for focus event
   */
  public generateFocusAnnouncement(
    node: AccessibilityNode,
    screenReader: ScreenReaderType,
    browser: BrowserType
  ): Announcement {
    return this.generate(node, screenReader, browser, 'normal');
  }

  /**
   * Generate announcement for live region update
   */
  public generateLiveRegionAnnouncement(
    node: AccessibilityNode,
    screenReader: ScreenReaderType,
    browser: BrowserType
  ): Announcement {
    const announcement = this.generate(node, screenReader, browser, 'minimal');

    // Live regions typically only announce the change, not the full role/state
    announcement.text = node.name || '';

    return announcement;
  }

  /**
   * Generate announcement for virtual cursor navigation
   */
  public generateNavigationAnnouncement(
    node: AccessibilityNode,
    screenReader: ScreenReaderType,
    browser: BrowserType,
    navigationMode: 'next' | 'previous' | 'heading' | 'landmark' | 'link'
  ): Announcement {
    const verbosity = navigationMode === 'next' || navigationMode === 'previous' ? 'normal' : 'verbose';
    return this.generate(node, screenReader, browser, verbosity);
  }
}
