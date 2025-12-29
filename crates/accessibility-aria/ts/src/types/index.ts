/**
 * ARIA Attribute Validator - Type Definitions
 * Comprehensive types for WAI-ARIA 1.2 specification
 */

export type ARIARoleType =
  | 'abstract'
  | 'widget'
  | 'composite'
  | 'document'
  | 'landmark'
  | 'live'
  | 'window';

export type ARIARole =
  // Abstract roles (not to be used directly)
  | 'command'
  | 'composite'
  | 'input'
  | 'landmark'
  | 'range'
  | 'roletype'
  | 'section'
  | 'sectionhead'
  | 'select'
  | 'structure'
  | 'widget'
  | 'window'
  // Widget roles
  | 'button'
  | 'checkbox'
  | 'gridcell'
  | 'link'
  | 'menuitem'
  | 'menuitemcheckbox'
  | 'menuitemradio'
  | 'option'
  | 'progressbar'
  | 'radio'
  | 'scrollbar'
  | 'searchbox'
  | 'separator'
  | 'slider'
  | 'spinbutton'
  | 'switch'
  | 'tab'
  | 'tabpanel'
  | 'textbox'
  | 'treeitem'
  // Composite widget roles
  | 'combobox'
  | 'grid'
  | 'listbox'
  | 'menu'
  | 'menubar'
  | 'radiogroup'
  | 'tablist'
  | 'tree'
  | 'treegrid'
  // Document structure roles
  | 'article'
  | 'cell'
  | 'columnheader'
  | 'definition'
  | 'directory'
  | 'document'
  | 'feed'
  | 'figure'
  | 'group'
  | 'heading'
  | 'img'
  | 'list'
  | 'listitem'
  | 'math'
  | 'none'
  | 'note'
  | 'presentation'
  | 'row'
  | 'rowgroup'
  | 'rowheader'
  | 'table'
  | 'term'
  | 'toolbar'
  | 'tooltip'
  // Landmark roles
  | 'banner'
  | 'complementary'
  | 'contentinfo'
  | 'form'
  | 'main'
  | 'navigation'
  | 'region'
  | 'search'
  // Live region roles
  | 'alert'
  | 'log'
  | 'marquee'
  | 'status'
  | 'timer'
  // Window roles
  | 'alertdialog'
  | 'dialog';

export type ARIAAttribute =
  // Widget attributes
  | 'aria-autocomplete'
  | 'aria-checked'
  | 'aria-disabled'
  | 'aria-errormessage'
  | 'aria-expanded'
  | 'aria-haspopup'
  | 'aria-hidden'
  | 'aria-invalid'
  | 'aria-label'
  | 'aria-level'
  | 'aria-modal'
  | 'aria-multiline'
  | 'aria-multiselectable'
  | 'aria-orientation'
  | 'aria-placeholder'
  | 'aria-pressed'
  | 'aria-readonly'
  | 'aria-required'
  | 'aria-selected'
  | 'aria-sort'
  | 'aria-valuemax'
  | 'aria-valuemin'
  | 'aria-valuenow'
  | 'aria-valuetext'
  // Live region attributes
  | 'aria-atomic'
  | 'aria-busy'
  | 'aria-live'
  | 'aria-relevant'
  // Drag-and-drop attributes
  | 'aria-dropeffect'
  | 'aria-grabbed'
  // Relationship attributes
  | 'aria-activedescendant'
  | 'aria-colcount'
  | 'aria-colindex'
  | 'aria-colspan'
  | 'aria-controls'
  | 'aria-describedby'
  | 'aria-details'
  | 'aria-flowto'
  | 'aria-labelledby'
  | 'aria-owns'
  | 'aria-posinset'
  | 'aria-rowcount'
  | 'aria-rowindex'
  | 'aria-rowspan'
  | 'aria-setsize'
  // ARIA 1.1+ attributes
  | 'aria-current'
  | 'aria-keyshortcuts'
  | 'aria-roledescription';

export type ARIAState =
  | 'aria-busy'
  | 'aria-checked'
  | 'aria-disabled'
  | 'aria-expanded'
  | 'aria-grabbed'
  | 'aria-hidden'
  | 'aria-invalid'
  | 'aria-pressed'
  | 'aria-selected';

export type ARIAAttributeValue =
  | string
  | number
  | boolean
  | 'true'
  | 'false'
  | 'mixed'
  | 'undefined';

export type ARIAAutocomplete = 'inline' | 'list' | 'both' | 'none';
export type ARIAChecked = 'true' | 'false' | 'mixed' | 'undefined';
export type ARIACurrent = 'page' | 'step' | 'location' | 'date' | 'time' | 'true' | 'false';
export type ARIAHasPopup = 'true' | 'false' | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog';
export type ARIAInvalid = 'true' | 'false' | 'grammar' | 'spelling';
export type ARIALive = 'off' | 'polite' | 'assertive';
export type ARIAOrientation = 'horizontal' | 'vertical' | 'undefined';
export type ARIAPressed = 'true' | 'false' | 'mixed' | 'undefined';
export type ARIARelevant = 'additions' | 'removals' | 'text' | 'all' | 'additions text';
export type ARIASort = 'ascending' | 'descending' | 'none' | 'other';

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  info: ValidationInfo[];
}

export interface ValidationError {
  type: 'role' | 'attribute' | 'state' | 'relationship' | 'value' | 'semantic';
  severity: 'error';
  message: string;
  element?: string;
  role?: ARIARole;
  attribute?: ARIAAttribute;
  expectedValue?: string;
  actualValue?: string;
  wcagCriterion?: string;
  line?: number;
  column?: number;
}

export interface ValidationWarning {
  type: 'role' | 'attribute' | 'state' | 'relationship' | 'value' | 'semantic';
  severity: 'warning';
  message: string;
  element?: string;
  role?: ARIARole;
  attribute?: ARIAAttribute;
  suggestion?: string;
  wcagCriterion?: string;
  line?: number;
  column?: number;
}

export interface ValidationInfo {
  type: 'role' | 'attribute' | 'state' | 'relationship' | 'value' | 'semantic';
  severity: 'info';
  message: string;
  element?: string;
  role?: ARIARole;
  attribute?: ARIAAttribute;
  line?: number;
  column?: number;
}

export interface RoleDefinition {
  name: ARIARole;
  type: ARIARoleType;
  abstract: boolean;
  superclassRoles: ARIARole[];
  subclassRoles: ARIARole[];
  requiredAttributes: ARIAAttribute[];
  supportedAttributes: ARIAAttribute[];
  prohibitedAttributes: ARIAAttribute[];
  requiredOwnedElements: string[];
  requiredContextRole: ARIARole[];
  accessibleNameRequired: boolean;
  accessibleNameFromAuthor: boolean;
  accessibleNameFromContent: boolean;
  childrenPresentational: boolean;
  implicitValueForRole?: Record<ARIAAttribute, string>;
}

export interface AttributeDefinition {
  name: ARIAAttribute;
  type: 'state' | 'property';
  valueType: 'true/false' | 'tristate' | 'true/false/undefined' | 'ID reference' | 'ID reference list' | 'integer' | 'number' | 'string' | 'token' | 'token list';
  allowedValues?: string[];
  defaultValue?: ARIAAttributeValue;
  description: string;
  relatedConcepts?: string[];
}

export interface ARIAPattern {
  name: string;
  description: string;
  roles: ARIARole[];
  keyboardInteraction: KeyboardInteraction[];
  requiredAttributes: Partial<Record<ARIARole, ARIAAttribute[]>>;
  stateManagement: StateManagement[];
  focusManagement: string;
  exampleHTML: string;
}

export interface KeyboardInteraction {
  key: string;
  action: string;
  context?: ARIARole;
}

export interface StateManagement {
  attribute: ARIAAttribute;
  role: ARIARole;
  condition: string;
  expectedValue: string;
}

export interface SemanticAnalysis {
  element: string;
  implicitRole?: ARIARole;
  explicitRole?: ARIARole;
  roleConflict: boolean;
  semanticIssues: string[];
  recommendations: string[];
}

export interface ARIATreeNode {
  element: string;
  role: ARIARole | null;
  attributes: Partial<Record<ARIAAttribute, string>>;
  children: ARIATreeNode[];
  parent?: ARIATreeNode;
  accessibleName?: string;
  accessibleDescription?: string;
}

export interface RelationshipValidation {
  attribute: ARIAAttribute;
  referencedId: string;
  exists: boolean;
  targetElement?: string;
  targetRole?: ARIARole;
  validReference: boolean;
  errors: string[];
}

export interface ImplicitRoleMapping {
  element: string;
  attributes?: Record<string, string>;
  implicitRole: ARIARole;
  conditions?: string;
}

export interface AccessibilityContext {
  document: Document | null;
  rootElement: HTMLElement | null;
  validationOptions: ValidationOptions;
}

export interface ValidationOptions {
  checkSemanticHTML: boolean;
  checkImplicitRoles: boolean;
  checkRelationships: boolean;
  checkKeyboardInteraction: boolean;
  strictMode: boolean;
  wcagLevel: 'A' | 'AA' | 'AAA';
  customRules?: CustomRule[];
}

export interface CustomRule {
  id: string;
  name: string;
  description: string;
  validate: (element: HTMLElement) => ValidationResult;
}

export interface ARIAValidationHookResult {
  validate: (element: HTMLElement | Document) => ValidationResult;
  validateRole: (element: HTMLElement, role: ARIARole) => boolean;
  validateAttribute: (element: HTMLElement, attribute: ARIAAttribute, value: string) => boolean;
  getImplicitRole: (element: HTMLElement) => ARIARole | null;
  getRoleDefinition: (role: ARIARole) => RoleDefinition | null;
  getAttributeDefinition: (attribute: ARIAAttribute) => AttributeDefinition | null;
  isValidating: boolean;
  lastResult: ValidationResult | null;
}
