/**
 * WAI-ARIA Design Patterns
 * Common ARIA patterns with keyboard interaction and state management requirements
 */

import { ARIAPattern } from '../types';

export const ARIA_PATTERNS: ARIAPattern[] = [
  {
    name: 'Accordion',
    description: 'A vertically stacked set of interactive headings that each contain a title, content snippet, or thumbnail representing a section of content.',
    roles: ['button', 'heading', 'region'],
    keyboardInteraction: [
      { key: 'Enter or Space', action: 'Expand/collapse the accordion panel', context: 'button' },
      { key: 'Tab', action: 'Move focus to next focusable element' },
      { key: 'Shift + Tab', action: 'Move focus to previous focusable element' },
    ],
    requiredAttributes: {
      button: ['aria-expanded', 'aria-controls'],
      region: ['aria-labelledby'],
    },
    stateManagement: [
      { attribute: 'aria-expanded', role: 'button', condition: 'Panel visible', expectedValue: 'true' },
      { attribute: 'aria-expanded', role: 'button', condition: 'Panel hidden', expectedValue: 'false' },
    ],
    focusManagement: 'Focus remains on the button when activating it',
    exampleHTML: '<h3><button aria-expanded="false" aria-controls="panel1">Accordion Header</button></h3><div id="panel1" role="region" aria-labelledby="header1">Panel content</div>',
  },
  {
    name: 'Alert',
    description: 'A message with important, time-sensitive information.',
    roles: ['alert'],
    keyboardInteraction: [],
    requiredAttributes: {},
    stateManagement: [],
    focusManagement: 'Alert does not receive focus',
    exampleHTML: '<div role="alert">Your session will expire in 5 minutes</div>',
  },
  {
    name: 'Alert Dialog',
    description: 'A modal dialog that interrupts the user\'s workflow to communicate an important message and acquire a response.',
    roles: ['alertdialog'],
    keyboardInteraction: [
      { key: 'Tab', action: 'Move focus to next focusable element inside dialog' },
      { key: 'Shift + Tab', action: 'Move focus to previous focusable element inside dialog' },
      { key: 'Escape', action: 'Close dialog (if allowed)' },
    ],
    requiredAttributes: {
      alertdialog: ['aria-labelledby', 'aria-describedby', 'aria-modal'],
    },
    stateManagement: [
      { attribute: 'aria-modal', role: 'alertdialog', condition: 'Dialog is modal', expectedValue: 'true' },
    ],
    focusManagement: 'Focus moves to an element inside the dialog when opened, typically the first focusable element or close button',
    exampleHTML: '<div role="alertdialog" aria-modal="true" aria-labelledby="dialog-title" aria-describedby="dialog-desc"><h2 id="dialog-title">Confirm Delete</h2><p id="dialog-desc">Are you sure?</p></div>',
  },
  {
    name: 'Breadcrumb',
    description: 'Provides a trail for users to keep track of their location within a website or application.',
    roles: ['navigation', 'list', 'listitem', 'link'],
    keyboardInteraction: [
      { key: 'Tab', action: 'Move to next link' },
      { key: 'Shift + Tab', action: 'Move to previous link' },
    ],
    requiredAttributes: {
      navigation: ['aria-label'],
      link: ['aria-current'],
    },
    stateManagement: [
      { attribute: 'aria-current', role: 'link', condition: 'Current page', expectedValue: 'page' },
    ],
    focusManagement: 'Standard link focus behavior',
    exampleHTML: '<nav aria-label="Breadcrumb"><ol role="list"><li role="listitem"><a href="/">Home</a></li><li role="listitem"><a href="/products" aria-current="page">Products</a></li></ol></nav>',
  },
  {
    name: 'Button',
    description: 'An input that allows for user-triggered actions.',
    roles: ['button'],
    keyboardInteraction: [
      { key: 'Enter or Space', action: 'Activate the button', context: 'button' },
    ],
    requiredAttributes: {},
    stateManagement: [
      { attribute: 'aria-pressed', role: 'button', condition: 'Toggle button pressed', expectedValue: 'true' },
      { attribute: 'aria-pressed', role: 'button', condition: 'Toggle button not pressed', expectedValue: 'false' },
    ],
    focusManagement: 'Button receives focus',
    exampleHTML: '<button type="button">Click Me</button>',
  },
  {
    name: 'Checkbox',
    description: 'A checkable input that has three possible values: true, false, or mixed.',
    roles: ['checkbox'],
    keyboardInteraction: [
      { key: 'Space', action: 'Toggle checkbox', context: 'checkbox' },
    ],
    requiredAttributes: {
      checkbox: ['aria-checked'],
    },
    stateManagement: [
      { attribute: 'aria-checked', role: 'checkbox', condition: 'Checked', expectedValue: 'true' },
      { attribute: 'aria-checked', role: 'checkbox', condition: 'Unchecked', expectedValue: 'false' },
      { attribute: 'aria-checked', role: 'checkbox', condition: 'Mixed/indeterminate', expectedValue: 'mixed' },
    ],
    focusManagement: 'Checkbox receives focus',
    exampleHTML: '<div role="checkbox" aria-checked="false" tabindex="0">Subscribe to newsletter</div>',
  },
  {
    name: 'Combobox',
    description: 'A composite widget containing a single-line textbox and a listbox popup.',
    roles: ['combobox', 'listbox', 'option'],
    keyboardInteraction: [
      { key: 'Down Arrow', action: 'Open listbox if closed, or move focus to next option', context: 'combobox' },
      { key: 'Up Arrow', action: 'Move focus to previous option', context: 'combobox' },
      { key: 'Enter', action: 'Accept current option and close listbox', context: 'combobox' },
      { key: 'Escape', action: 'Close listbox if open', context: 'combobox' },
    ],
    requiredAttributes: {
      combobox: ['aria-expanded', 'aria-controls', 'aria-haspopup'],
      option: ['aria-selected'],
    },
    stateManagement: [
      { attribute: 'aria-expanded', role: 'combobox', condition: 'Listbox visible', expectedValue: 'true' },
      { attribute: 'aria-expanded', role: 'combobox', condition: 'Listbox hidden', expectedValue: 'false' },
      { attribute: 'aria-selected', role: 'option', condition: 'Option selected', expectedValue: 'true' },
    ],
    focusManagement: 'Focus remains on the combobox',
    exampleHTML: '<input type="text" role="combobox" aria-expanded="false" aria-controls="listbox1" aria-haspopup="listbox"><ul id="listbox1" role="listbox"><li role="option">Option 1</li></ul>',
  },
  {
    name: 'Dialog (Modal)',
    description: 'A window overlaid on either the primary window or another dialog window.',
    roles: ['dialog'],
    keyboardInteraction: [
      { key: 'Tab', action: 'Move focus to next element in dialog' },
      { key: 'Shift + Tab', action: 'Move focus to previous element in dialog' },
      { key: 'Escape', action: 'Close dialog' },
    ],
    requiredAttributes: {
      dialog: ['aria-labelledby', 'aria-modal'],
    },
    stateManagement: [
      { attribute: 'aria-modal', role: 'dialog', condition: 'Dialog is modal', expectedValue: 'true' },
    ],
    focusManagement: 'Focus moves into dialog when opened and is trapped within dialog until closed',
    exampleHTML: '<div role="dialog" aria-modal="true" aria-labelledby="dialog-title"><h2 id="dialog-title">Dialog Title</h2></div>',
  },
  {
    name: 'Disclosure (Show/Hide)',
    description: 'A button that controls visibility of a section of content.',
    roles: ['button'],
    keyboardInteraction: [
      { key: 'Enter or Space', action: 'Toggle content visibility', context: 'button' },
    ],
    requiredAttributes: {
      button: ['aria-expanded', 'aria-controls'],
    },
    stateManagement: [
      { attribute: 'aria-expanded', role: 'button', condition: 'Content visible', expectedValue: 'true' },
      { attribute: 'aria-expanded', role: 'button', condition: 'Content hidden', expectedValue: 'false' },
    ],
    focusManagement: 'Focus remains on button',
    exampleHTML: '<button aria-expanded="false" aria-controls="content1">Show Details</button><div id="content1">Hidden content</div>',
  },
  {
    name: 'Listbox',
    description: 'A widget that allows the user to select one or more items from a list of choices.',
    roles: ['listbox', 'option'],
    keyboardInteraction: [
      { key: 'Down Arrow', action: 'Move focus to next option', context: 'listbox' },
      { key: 'Up Arrow', action: 'Move focus to previous option', context: 'listbox' },
      { key: 'Home', action: 'Move focus to first option', context: 'listbox' },
      { key: 'End', action: 'Move focus to last option', context: 'listbox' },
      { key: 'Space', action: 'Select/deselect focused option (multi-select)', context: 'listbox' },
    ],
    requiredAttributes: {
      option: ['aria-selected'],
    },
    stateManagement: [
      { attribute: 'aria-selected', role: 'option', condition: 'Option selected', expectedValue: 'true' },
      { attribute: 'aria-multiselectable', role: 'listbox', condition: 'Multiple selection allowed', expectedValue: 'true' },
    ],
    focusManagement: 'Listbox manages focus using aria-activedescendant or by moving DOM focus',
    exampleHTML: '<ul role="listbox" aria-label="Choose a color"><li role="option" aria-selected="true">Red</li><li role="option" aria-selected="false">Blue</li></ul>',
  },
  {
    name: 'Menu',
    description: 'A widget that offers a list of choices to the user.',
    roles: ['menu', 'menuitem', 'menuitemcheckbox', 'menuitemradio'],
    keyboardInteraction: [
      { key: 'Enter or Space', action: 'Activate menuitem', context: 'menuitem' },
      { key: 'Down Arrow', action: 'Move focus to next menuitem', context: 'menu' },
      { key: 'Up Arrow', action: 'Move focus to previous menuitem', context: 'menu' },
      { key: 'Home', action: 'Move focus to first menuitem', context: 'menu' },
      { key: 'End', action: 'Move focus to last menuitem', context: 'menu' },
      { key: 'Escape', action: 'Close menu and return focus to menu button', context: 'menu' },
    ],
    requiredAttributes: {
      menuitemcheckbox: ['aria-checked'],
      menuitemradio: ['aria-checked'],
    },
    stateManagement: [
      { attribute: 'aria-checked', role: 'menuitemcheckbox', condition: 'Checked', expectedValue: 'true' },
      { attribute: 'aria-checked', role: 'menuitemradio', condition: 'Selected', expectedValue: 'true' },
    ],
    focusManagement: 'Focus moves into menu when opened, typically to first menuitem',
    exampleHTML: '<ul role="menu"><li role="menuitem">New File</li><li role="menuitemcheckbox" aria-checked="true">Show Toolbar</li></ul>',
  },
  {
    name: 'Radio Group',
    description: 'A group of radio buttons.',
    roles: ['radiogroup', 'radio'],
    keyboardInteraction: [
      { key: 'Tab', action: 'Move focus into/out of radio group', context: 'radiogroup' },
      { key: 'Arrow Keys', action: 'Move focus and selection between radio buttons', context: 'radio' },
      { key: 'Space', action: 'Select focused radio button (if not using arrow keys to select)', context: 'radio' },
    ],
    requiredAttributes: {
      radio: ['aria-checked'],
    },
    stateManagement: [
      { attribute: 'aria-checked', role: 'radio', condition: 'Selected', expectedValue: 'true' },
      { attribute: 'aria-checked', role: 'radio', condition: 'Not selected', expectedValue: 'false' },
    ],
    focusManagement: 'Only one radio button in the group is in the tab sequence (roving tabindex)',
    exampleHTML: '<div role="radiogroup" aria-labelledby="group-label"><div role="radio" aria-checked="true" tabindex="0">Option 1</div><div role="radio" aria-checked="false" tabindex="-1">Option 2</div></div>',
  },
  {
    name: 'Slider',
    description: 'An input where the user selects a value from within a given range.',
    roles: ['slider'],
    keyboardInteraction: [
      { key: 'Right Arrow', action: 'Increase slider value', context: 'slider' },
      { key: 'Left Arrow', action: 'Decrease slider value', context: 'slider' },
      { key: 'Up Arrow', action: 'Increase slider value', context: 'slider' },
      { key: 'Down Arrow', action: 'Decrease slider value', context: 'slider' },
      { key: 'Home', action: 'Set slider to minimum value', context: 'slider' },
      { key: 'End', action: 'Set slider to maximum value', context: 'slider' },
    ],
    requiredAttributes: {
      slider: ['aria-valuenow', 'aria-valuemin', 'aria-valuemax'],
    },
    stateManagement: [
      { attribute: 'aria-valuenow', role: 'slider', condition: 'Current value', expectedValue: 'Current numeric value' },
    ],
    focusManagement: 'Slider receives focus',
    exampleHTML: '<div role="slider" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100" aria-label="Volume" tabindex="0"></div>',
  },
  {
    name: 'Tabs',
    description: 'A set of layered sections of content (tab panels) with tabs for switching between them.',
    roles: ['tablist', 'tab', 'tabpanel'],
    keyboardInteraction: [
      { key: 'Tab', action: 'Move focus from tablist to tab panel', context: 'tab' },
      { key: 'Arrow Keys', action: 'Move between tabs (horizontal tablist uses Left/Right, vertical uses Up/Down)', context: 'tablist' },
      { key: 'Home', action: 'Move focus to first tab', context: 'tablist' },
      { key: 'End', action: 'Move focus to last tab', context: 'tablist' },
    ],
    requiredAttributes: {
      tab: ['aria-selected', 'aria-controls'],
      tabpanel: ['aria-labelledby'],
    },
    stateManagement: [
      { attribute: 'aria-selected', role: 'tab', condition: 'Tab active', expectedValue: 'true' },
      { attribute: 'aria-selected', role: 'tab', condition: 'Tab inactive', expectedValue: 'false' },
    ],
    focusManagement: 'Only the active tab is in the tab sequence (roving tabindex)',
    exampleHTML: '<div role="tablist"><button role="tab" aria-selected="true" aria-controls="panel1" tabindex="0">Tab 1</button></div><div role="tabpanel" id="panel1" aria-labelledby="tab1">Panel content</div>',
  },
  {
    name: 'Tree View',
    description: 'A hierarchical list with parent and child nodes that can be expanded and collapsed.',
    roles: ['tree', 'treeitem', 'group'],
    keyboardInteraction: [
      { key: 'Down Arrow', action: 'Move to next visible node', context: 'tree' },
      { key: 'Up Arrow', action: 'Move to previous visible node', context: 'tree' },
      { key: 'Right Arrow', action: 'Expand node (if collapsed) or move to first child', context: 'treeitem' },
      { key: 'Left Arrow', action: 'Collapse node (if expanded) or move to parent', context: 'treeitem' },
      { key: 'Enter', action: 'Activate treeitem', context: 'treeitem' },
      { key: 'Home', action: 'Move to first node', context: 'tree' },
      { key: 'End', action: 'Move to last visible node', context: 'tree' },
    ],
    requiredAttributes: {
      treeitem: ['aria-expanded'],
    },
    stateManagement: [
      { attribute: 'aria-expanded', role: 'treeitem', condition: 'Node expanded', expectedValue: 'true' },
      { attribute: 'aria-expanded', role: 'treeitem', condition: 'Node collapsed', expectedValue: 'false' },
      { attribute: 'aria-selected', role: 'treeitem', condition: 'Node selected', expectedValue: 'true' },
    ],
    focusManagement: 'Only one treeitem is in the tab sequence (roving tabindex)',
    exampleHTML: '<ul role="tree"><li role="treeitem" aria-expanded="true">Parent<ul role="group"><li role="treeitem">Child</li></ul></li></ul>',
  },
];

export function getPattern(name: string): ARIAPattern | undefined {
  return ARIA_PATTERNS.find(pattern => pattern.name.toLowerCase() === name.toLowerCase());
}

export function getPatternsForRole(roleName: string): ARIAPattern[] {
  return ARIA_PATTERNS.filter(pattern =>
    pattern.roles.some(role => role === roleName)
  );
}

export function getAllPatternNames(): string[] {
  return ARIA_PATTERNS.map(pattern => pattern.name);
}
