/**
 * Core types for screen reader compatibility analysis
 */

export type AriaRole =
  | 'alert'
  | 'alertdialog'
  | 'application'
  | 'article'
  | 'banner'
  | 'button'
  | 'cell'
  | 'checkbox'
  | 'columnheader'
  | 'combobox'
  | 'complementary'
  | 'contentinfo'
  | 'definition'
  | 'dialog'
  | 'directory'
  | 'document'
  | 'feed'
  | 'figure'
  | 'form'
  | 'grid'
  | 'gridcell'
  | 'group'
  | 'heading'
  | 'img'
  | 'link'
  | 'list'
  | 'listbox'
  | 'listitem'
  | 'log'
  | 'main'
  | 'marquee'
  | 'math'
  | 'menu'
  | 'menubar'
  | 'menuitem'
  | 'menuitemcheckbox'
  | 'menuitemradio'
  | 'navigation'
  | 'none'
  | 'note'
  | 'option'
  | 'presentation'
  | 'progressbar'
  | 'radio'
  | 'radiogroup'
  | 'region'
  | 'row'
  | 'rowgroup'
  | 'rowheader'
  | 'scrollbar'
  | 'search'
  | 'searchbox'
  | 'separator'
  | 'slider'
  | 'spinbutton'
  | 'status'
  | 'switch'
  | 'tab'
  | 'table'
  | 'tablist'
  | 'tabpanel'
  | 'term'
  | 'textbox'
  | 'timer'
  | 'toolbar'
  | 'tooltip'
  | 'tree'
  | 'treegrid'
  | 'treeitem';

export type ScreenReaderType = 'NVDA' | 'JAWS' | 'VoiceOver' | 'TalkBack' | 'Narrator';

export type BrowserType = 'Chrome' | 'Firefox' | 'Safari' | 'Edge';

export type SeverityLevel = 'critical' | 'serious' | 'moderate' | 'minor';

export interface AccessibilityNode {
  id: string;
  element: Element;
  role: AriaRole | string;
  name: string | null;
  description: string | null;
  value: string | null;
  level?: number;
  focusable: boolean;
  hidden: boolean;
  disabled: boolean;
  readonly: boolean;
  required: boolean;
  invalid: boolean;
  expanded?: boolean;
  selected?: boolean;
  checked?: boolean | 'mixed';
  pressed?: boolean | 'mixed';
  current?: string | boolean;
  live?: 'off' | 'polite' | 'assertive';
  atomic?: boolean;
  relevant?: string;
  busy?: boolean;
  controls?: string[];
  describedBy?: string[];
  labelledBy?: string[];
  owns?: string[];
  flowTo?: string[];
  children: AccessibilityNode[];
  parent: AccessibilityNode | null;
  boundingBox: DOMRect;
  tabIndex: number;
}

export interface ReadingOrderItem {
  node: AccessibilityNode;
  order: number;
  visualPosition: { x: number; y: number };
  logicalPosition: number;
  isOutOfOrder: boolean;
  deviation: number;
}

export interface ReadingOrder {
  items: ReadingOrderItem[];
  issues: ReadingOrderIssue[];
  score: number;
}

export interface ReadingOrderIssue {
  type: 'visual-logical-mismatch' | 'focus-order-mismatch' | 'tabindex-abuse';
  severity: SeverityLevel;
  items: ReadingOrderItem[];
  description: string;
  remediation: string;
}

export interface Announcement {
  text: string;
  role: AriaRole | string;
  name: string | null;
  state: string[];
  properties: string[];
  context: string[];
  screenReader: ScreenReaderType;
  browser: BrowserType;
  verbosity: 'minimal' | 'normal' | 'verbose';
}

export interface LandmarkInfo {
  node: AccessibilityNode;
  role: AriaRole | string;
  label: string | null;
  level: number;
  duplicateLabels: boolean;
  missingLabel: boolean;
  children: LandmarkInfo[];
}

export interface LandmarkStructure {
  landmarks: LandmarkInfo[];
  issues: LandmarkIssue[];
  score: number;
}

export interface LandmarkIssue {
  type:
    | 'missing-main'
    | 'multiple-main'
    | 'missing-label'
    | 'duplicate-label'
    | 'nested-incorrectly'
    | 'redundant-landmark';
  severity: SeverityLevel;
  node: AccessibilityNode;
  description: string;
  remediation: string;
}

export interface HeadingInfo {
  node: AccessibilityNode;
  level: number;
  text: string;
  order: number;
  skipped: boolean;
  empty: boolean;
}

export interface HeadingStructure {
  headings: HeadingInfo[];
  issues: HeadingIssue[];
  score: number;
}

export interface HeadingIssue {
  type: 'skipped-level' | 'empty-heading' | 'missing-h1' | 'multiple-h1' | 'improper-nesting';
  severity: SeverityLevel;
  heading: HeadingInfo;
  description: string;
  remediation: string;
}

export interface FormFieldInfo {
  node: AccessibilityNode;
  label: string | null;
  labelMethod: 'label-element' | 'aria-label' | 'aria-labelledby' | 'title' | 'placeholder' | 'none';
  instructions: string | null;
  errorMessage: string | null;
  groupLabel: string | null;
  announcement: Announcement;
}

export interface FormStructure {
  fields: FormFieldInfo[];
  issues: FormIssue[];
  score: number;
}

export interface FormIssue {
  type:
    | 'missing-label'
    | 'placeholder-as-label'
    | 'title-as-label'
    | 'missing-error-message'
    | 'generic-error'
    | 'missing-required'
    | 'missing-instructions'
    | 'unlabeled-group';
  severity: SeverityLevel;
  field: FormFieldInfo;
  description: string;
  remediation: string;
}

export interface LiveRegionInfo {
  node: AccessibilityNode;
  live: 'polite' | 'assertive';
  atomic: boolean;
  relevant: string[];
  busy: boolean;
  label: string | null;
}

export interface LiveRegionStructure {
  regions: LiveRegionInfo[];
  issues: LiveRegionIssue[];
  score: number;
}

export interface LiveRegionIssue {
  type: 'missing-role' | 'improper-politeness' | 'missing-label' | 'too-frequent-updates';
  severity: SeverityLevel;
  region: LiveRegionInfo;
  description: string;
  remediation: string;
}

export interface NavigationPath {
  type: 'landmark' | 'heading' | 'link' | 'form' | 'table' | 'list';
  items: AccessibilityNode[];
  currentIndex: number;
}

export interface ScreenReaderState {
  currentNode: AccessibilityNode | null;
  navigationMode: 'browse' | 'focus' | 'forms';
  verbosity: 'minimal' | 'normal' | 'verbose';
  readingMode: 'character' | 'word' | 'line' | 'sentence' | 'paragraph';
  navigationPath: NavigationPath | null;
  announcementQueue: Announcement[];
  history: AccessibilityNode[];
}

export interface AccessibilityIssue {
  id: string;
  type: string;
  severity: SeverityLevel;
  node: AccessibilityNode;
  description: string;
  remediation: string;
  wcagCriteria: string[];
  screenReadersAffected: ScreenReaderType[];
  codeExample?: string;
  fixedExample?: string;
}

export interface AccessibilityReport {
  timestamp: Date;
  url: string;
  tree: AccessibilityNode;
  readingOrder: ReadingOrder;
  landmarks: LandmarkStructure;
  headings: HeadingStructure;
  forms: FormStructure;
  liveRegions: LiveRegionStructure;
  issues: AccessibilityIssue[];
  score: number;
  summary: {
    totalNodes: number;
    focusableNodes: number;
    hiddenNodes: number;
    criticalIssues: number;
    seriousIssues: number;
    moderateIssues: number;
    minorIssues: number;
  };
}

export interface ScreenReaderConfig {
  type: ScreenReaderType;
  browser: BrowserType;
  verbosity: 'minimal' | 'normal' | 'verbose';
  announceAriaDescriptions: boolean;
  announceLists: boolean;
  announceTableHeaders: boolean;
  announceLineNumbers: boolean;
  announceIndentation: boolean;
  speechRate: number;
}

export interface TestResult {
  passed: boolean;
  testName: string;
  description: string;
  expected: string;
  actual: string;
  node?: AccessibilityNode;
  recommendation?: string;
}

export interface TestSuite {
  name: string;
  description: string;
  tests: TestResult[];
  passRate: number;
  totalTests: number;
  passedTests: number;
  failedTests: number;
}
