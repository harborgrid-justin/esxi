/**
 * Complete WAI-ARIA 1.2 Attribute Definitions
 * Comprehensive attribute specifications with value types and constraints
 */

import { AttributeDefinition, ARIAAttribute } from '../types';

export const ARIA_ATTRIBUTES: Record<ARIAAttribute, AttributeDefinition> = {
  'aria-autocomplete': {
    name: 'aria-autocomplete',
    type: 'property',
    valueType: 'token',
    allowedValues: ['inline', 'list', 'both', 'none'],
    defaultValue: 'none',
    description: 'Indicates whether inputting text could trigger display of one or more predictions of the user\'s intended value for a combobox, searchbox, or textbox.',
    relatedConcepts: ['HTML autocomplete attribute'],
  },
  'aria-checked': {
    name: 'aria-checked',
    type: 'state',
    valueType: 'tristate',
    allowedValues: ['true', 'false', 'mixed', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates the current "checked" state of checkboxes, radio buttons, and other widgets.',
    relatedConcepts: ['HTML checked attribute'],
  },
  'aria-disabled': {
    name: 'aria-disabled',
    type: 'state',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates that the element is perceivable but disabled, so it is not editable or otherwise operable.',
    relatedConcepts: ['HTML disabled attribute'],
  },
  'aria-errormessage': {
    name: 'aria-errormessage',
    type: 'property',
    valueType: 'ID reference',
    description: 'Identifies the element that provides an error message for the object.',
    relatedConcepts: ['aria-invalid', 'aria-describedby'],
  },
  'aria-expanded': {
    name: 'aria-expanded',
    type: 'state',
    valueType: 'true/false/undefined',
    allowedValues: ['true', 'false', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates whether the element, or another grouping element it controls, is currently expanded or collapsed.',
    relatedConcepts: ['HTML details element'],
  },
  'aria-haspopup': {
    name: 'aria-haspopup',
    type: 'property',
    valueType: 'token',
    allowedValues: ['false', 'true', 'menu', 'listbox', 'tree', 'grid', 'dialog'],
    defaultValue: 'false',
    description: 'Indicates the availability and type of interactive popup element, such as menu or dialog, that can be triggered by an element.',
  },
  'aria-hidden': {
    name: 'aria-hidden',
    type: 'state',
    valueType: 'true/false/undefined',
    allowedValues: ['true', 'false', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates whether the element is exposed to an accessibility API.',
    relatedConcepts: ['HTML hidden attribute'],
  },
  'aria-invalid': {
    name: 'aria-invalid',
    type: 'state',
    valueType: 'token',
    allowedValues: ['true', 'false', 'grammar', 'spelling'],
    defaultValue: 'false',
    description: 'Indicates the entered value does not conform to the format expected by the application.',
    relatedConcepts: ['HTML5 input validation'],
  },
  'aria-label': {
    name: 'aria-label',
    type: 'property',
    valueType: 'string',
    description: 'Defines a string value that labels the current element.',
    relatedConcepts: ['aria-labelledby', 'HTML label element'],
  },
  'aria-level': {
    name: 'aria-level',
    type: 'property',
    valueType: 'integer',
    description: 'Defines the hierarchical level of an element within a structure.',
    relatedConcepts: ['HTML heading elements h1-h6'],
  },
  'aria-modal': {
    name: 'aria-modal',
    type: 'property',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates whether an element is modal when displayed.',
    relatedConcepts: ['HTML dialog element'],
  },
  'aria-multiline': {
    name: 'aria-multiline',
    type: 'property',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates whether a text box accepts multiple lines of input or only a single line.',
    relatedConcepts: ['HTML textarea element'],
  },
  'aria-multiselectable': {
    name: 'aria-multiselectable',
    type: 'property',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates that the user may select more than one item from the current selectable descendants.',
    relatedConcepts: ['HTML select multiple attribute'],
  },
  'aria-orientation': {
    name: 'aria-orientation',
    type: 'property',
    valueType: 'token',
    allowedValues: ['horizontal', 'vertical', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates whether the element\'s orientation is horizontal, vertical, or unknown/ambiguous.',
  },
  'aria-placeholder': {
    name: 'aria-placeholder',
    type: 'property',
    valueType: 'string',
    description: 'Defines a short hint intended to aid the user with data entry when the control has no value.',
    relatedConcepts: ['HTML placeholder attribute'],
  },
  'aria-pressed': {
    name: 'aria-pressed',
    type: 'state',
    valueType: 'tristate',
    allowedValues: ['true', 'false', 'mixed', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates the current "pressed" state of toggle buttons.',
  },
  'aria-readonly': {
    name: 'aria-readonly',
    type: 'property',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates that the element is not editable, but is otherwise operable.',
    relatedConcepts: ['HTML readonly attribute'],
  },
  'aria-required': {
    name: 'aria-required',
    type: 'property',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates that user input is required on the element before a form may be submitted.',
    relatedConcepts: ['HTML required attribute'],
  },
  'aria-selected': {
    name: 'aria-selected',
    type: 'state',
    valueType: 'true/false/undefined',
    allowedValues: ['true', 'false', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates the current "selected" state of various widgets.',
    relatedConcepts: ['HTML option selected attribute'],
  },
  'aria-sort': {
    name: 'aria-sort',
    type: 'property',
    valueType: 'token',
    allowedValues: ['ascending', 'descending', 'none', 'other'],
    defaultValue: 'none',
    description: 'Indicates if items in a table or grid are sorted in ascending or descending order.',
  },
  'aria-valuemax': {
    name: 'aria-valuemax',
    type: 'property',
    valueType: 'number',
    description: 'Defines the maximum allowed value for a range widget.',
    relatedConcepts: ['HTML input max attribute'],
  },
  'aria-valuemin': {
    name: 'aria-valuemin',
    type: 'property',
    valueType: 'number',
    description: 'Defines the minimum allowed value for a range widget.',
    relatedConcepts: ['HTML input min attribute'],
  },
  'aria-valuenow': {
    name: 'aria-valuenow',
    type: 'property',
    valueType: 'number',
    description: 'Defines the current value for a range widget.',
    relatedConcepts: ['HTML input value attribute'],
  },
  'aria-valuetext': {
    name: 'aria-valuetext',
    type: 'property',
    valueType: 'string',
    description: 'Defines the human readable text alternative of aria-valuenow for a range widget.',
  },
  'aria-atomic': {
    name: 'aria-atomic',
    type: 'property',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates whether assistive technologies will present all, or only parts of, the changed region based on the change notifications defined by the aria-relevant attribute.',
  },
  'aria-busy': {
    name: 'aria-busy',
    type: 'state',
    valueType: 'true/false',
    allowedValues: ['true', 'false'],
    defaultValue: 'false',
    description: 'Indicates an element is being modified and that assistive technologies may want to wait until the modifications are complete before exposing them to the user.',
  },
  'aria-live': {
    name: 'aria-live',
    type: 'property',
    valueType: 'token',
    allowedValues: ['off', 'polite', 'assertive'],
    defaultValue: 'off',
    description: 'Indicates that an element will be updated, and describes the types of updates the user agents, assistive technologies, and user can expect from the live region.',
  },
  'aria-relevant': {
    name: 'aria-relevant',
    type: 'property',
    valueType: 'token list',
    allowedValues: ['additions', 'removals', 'text', 'all'],
    defaultValue: 'additions text',
    description: 'Indicates what notifications the user agent will trigger when the accessibility tree within a live region is modified.',
  },
  'aria-dropeffect': {
    name: 'aria-dropeffect',
    type: 'property',
    valueType: 'token list',
    allowedValues: ['copy', 'execute', 'link', 'move', 'none', 'popup'],
    defaultValue: 'none',
    description: 'Indicates what functions can be performed when a dragged object is released on the drop target. (Deprecated in ARIA 1.1)',
  },
  'aria-grabbed': {
    name: 'aria-grabbed',
    type: 'state',
    valueType: 'true/false/undefined',
    allowedValues: ['true', 'false', 'undefined'],
    defaultValue: 'undefined',
    description: 'Indicates an element\'s "grabbed" state in a drag-and-drop operation. (Deprecated in ARIA 1.1)',
  },
  'aria-activedescendant': {
    name: 'aria-activedescendant',
    type: 'property',
    valueType: 'ID reference',
    description: 'Identifies the currently active element when DOM focus is on a composite widget, textbox, group, or application.',
  },
  'aria-colcount': {
    name: 'aria-colcount',
    type: 'property',
    valueType: 'integer',
    description: 'Defines the total number of columns in a table, grid, or treegrid.',
  },
  'aria-colindex': {
    name: 'aria-colindex',
    type: 'property',
    valueType: 'integer',
    description: 'Defines an element\'s column index or position with respect to the total number of columns within a table, grid, or treegrid.',
  },
  'aria-colspan': {
    name: 'aria-colspan',
    type: 'property',
    valueType: 'integer',
    description: 'Defines the number of columns spanned by a cell or gridcell within a table, grid, or treegrid.',
    relatedConcepts: ['HTML colspan attribute'],
  },
  'aria-controls': {
    name: 'aria-controls',
    type: 'property',
    valueType: 'ID reference list',
    description: 'Identifies the element (or elements) whose contents or presence are controlled by the current element.',
  },
  'aria-describedby': {
    name: 'aria-describedby',
    type: 'property',
    valueType: 'ID reference list',
    description: 'Identifies the element (or elements) that describes the object.',
    relatedConcepts: ['aria-labelledby', 'HTML aria-describedby'],
  },
  'aria-details': {
    name: 'aria-details',
    type: 'property',
    valueType: 'ID reference',
    description: 'Identifies the element that provides a detailed, extended description for the object.',
  },
  'aria-flowto': {
    name: 'aria-flowto',
    type: 'property',
    valueType: 'ID reference list',
    description: 'Identifies the next element (or elements) in an alternate reading order of content.',
  },
  'aria-labelledby': {
    name: 'aria-labelledby',
    type: 'property',
    valueType: 'ID reference list',
    description: 'Identifies the element (or elements) that labels the current element.',
    relatedConcepts: ['aria-label', 'HTML label element'],
  },
  'aria-owns': {
    name: 'aria-owns',
    type: 'property',
    valueType: 'ID reference list',
    description: 'Identifies an element (or elements) in order to define a visual, functional, or contextual parent/child relationship between DOM elements where the DOM hierarchy cannot be used to represent the relationship.',
  },
  'aria-posinset': {
    name: 'aria-posinset',
    type: 'property',
    valueType: 'integer',
    description: 'Defines an element\'s number or position in the current set of listitems or treeitems.',
  },
  'aria-rowcount': {
    name: 'aria-rowcount',
    type: 'property',
    valueType: 'integer',
    description: 'Defines the total number of rows in a table, grid, or treegrid.',
  },
  'aria-rowindex': {
    name: 'aria-rowindex',
    type: 'property',
    valueType: 'integer',
    description: 'Defines an element\'s row index or position with respect to the total number of rows within a table, grid, or treegrid.',
  },
  'aria-rowspan': {
    name: 'aria-rowspan',
    type: 'property',
    valueType: 'integer',
    description: 'Defines the number of rows spanned by a cell or gridcell within a table, grid, or treegrid.',
    relatedConcepts: ['HTML rowspan attribute'],
  },
  'aria-setsize': {
    name: 'aria-setsize',
    type: 'property',
    valueType: 'integer',
    description: 'Defines the number of items in the current set of listitems or treeitems.',
  },
  'aria-current': {
    name: 'aria-current',
    type: 'state',
    valueType: 'token',
    allowedValues: ['page', 'step', 'location', 'date', 'time', 'true', 'false'],
    defaultValue: 'false',
    description: 'Indicates the element that represents the current item within a container or set of related elements.',
  },
  'aria-keyshortcuts': {
    name: 'aria-keyshortcuts',
    type: 'property',
    valueType: 'string',
    description: 'Indicates keyboard shortcuts that an author has implemented to activate or give focus to an element.',
  },
  'aria-roledescription': {
    name: 'aria-roledescription',
    type: 'property',
    valueType: 'string',
    description: 'Defines a human-readable, author-localized description for the role of an element.',
  },
};

export function getAttributeDefinition(attribute: ARIAAttribute): AttributeDefinition | null {
  return ARIA_ATTRIBUTES[attribute] || null;
}

export function isValidAttribute(attribute: string): attribute is ARIAAttribute {
  return attribute in ARIA_ATTRIBUTES;
}

export function isStateAttribute(attribute: ARIAAttribute): boolean {
  const definition = getAttributeDefinition(attribute);
  return definition?.type === 'state';
}

export function isPropertyAttribute(attribute: ARIAAttribute): boolean {
  const definition = getAttributeDefinition(attribute);
  return definition?.type === 'property';
}

export function getAllowedValues(attribute: ARIAAttribute): string[] | undefined {
  const definition = getAttributeDefinition(attribute);
  return definition?.allowedValues;
}

export function getDefaultValue(attribute: ARIAAttribute): string | number | boolean | undefined {
  const definition = getAttributeDefinition(attribute);
  return definition?.defaultValue;
}
