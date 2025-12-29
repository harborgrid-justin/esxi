/**
 * Enterprise Keyboard Navigation Validator Types
 * Comprehensive type definitions for keyboard accessibility testing
 */

export interface FocusableElement {
  element: HTMLElement;
  tabIndex: number;
  isVisible: boolean;
  isDisabled: boolean;
  hasValidRole: boolean;
  hasFocusIndicator: boolean;
  visualTabOrder: number;
  domOrder: number;
  selector: string;
  computedRole: string | null;
  ariaLabel?: string;
  ariaLabelledBy?: string;
  ariaDescribedBy?: string;
}

export interface TabOrder {
  elements: FocusableElement[];
  logicalOrder: number[];
  domOrder: number[];
  hasLogicalIssues: boolean;
  issues: TabOrderIssue[];
}

export interface TabOrderIssue {
  type: 'out-of-order' | 'skip' | 'positive-tabindex' | 'duplicate' | 'hidden-focusable';
  severity: 'error' | 'warning' | 'info';
  element: FocusableElement;
  message: string;
  wcagCriteria: string[];
  suggestion: string;
}

export interface FocusTrap {
  detected: boolean;
  trapElement: HTMLElement | null;
  escapeMethod: 'none' | 'keyboard' | 'mouse-only';
  affectedElements: HTMLElement[];
  severity: 'critical' | 'major' | 'minor';
  canEscape: boolean;
  description: string;
}

export interface KeyboardShortcut {
  key: string;
  modifiers: {
    ctrl?: boolean;
    alt?: boolean;
    shift?: boolean;
    meta?: boolean;
  };
  action: string;
  element: HTMLElement | null;
  conflicts: KeyboardShortcut[];
  isBrowserConflict: boolean;
  isDocumented: boolean;
}

export interface FocusIndicator {
  element: HTMLElement;
  hasOutline: boolean;
  hasCustomIndicator: boolean;
  contrastRatio: number | null;
  meetsWCAG: boolean;
  styles: {
    outline?: string;
    border?: string;
    boxShadow?: string;
    backgroundColor?: string;
  };
  issues: string[];
}

export interface SkipLink {
  element: HTMLElement;
  target: string;
  targetExists: boolean;
  isVisible: boolean;
  isFirstFocusable: boolean;
  worksCorrectly: boolean;
  issues: string[];
}

export interface InteractiveElement {
  element: HTMLElement;
  isKeyboardAccessible: boolean;
  hasProperRole: boolean;
  hasKeyboardHandler: boolean;
  missingHandlers: string[];
  requiredKeys: string[];
  implementedKeys: string[];
}

export interface KeyboardNavigationReport {
  timestamp: string;
  url: string;
  tabOrder: TabOrder;
  focusTraps: FocusTrap[];
  focusIndicators: FocusIndicator[];
  skipLinks: SkipLink[];
  shortcuts: KeyboardShortcut[];
  interactiveElements: InteractiveElement[];
  summary: {
    totalFocusable: number;
    totalIssues: number;
    criticalIssues: number;
    warnings: number;
    passed: boolean;
    wcagLevel: 'A' | 'AA' | 'AAA' | 'fail';
  };
}

export interface FocusPath {
  sequence: HTMLElement[];
  startTime: number;
  endTime: number;
  method: 'tab' | 'shift-tab' | 'arrow' | 'programmatic';
  expectedPath?: HTMLElement[];
  deviations: number;
}

export interface KeyboardTestConfig {
  checkTabOrder: boolean;
  checkFocusTraps: boolean;
  checkFocusIndicators: boolean;
  checkSkipLinks: boolean;
  checkShortcuts: boolean;
  checkInteractiveElements: boolean;
  contrastThreshold: number;
  timeout: number;
}

export interface KeyboardValidationResult {
  passed: boolean;
  violations: KeyboardViolation[];
  warnings: KeyboardWarning[];
  recommendations: string[];
  wcagCompliance: WCAGCompliance;
}

export interface KeyboardViolation {
  id: string;
  type: string;
  severity: 'critical' | 'serious' | 'moderate' | 'minor';
  element: HTMLElement;
  description: string;
  wcagCriteria: string[];
  impact: string;
  howToFix: string;
}

export interface KeyboardWarning {
  id: string;
  type: string;
  element: HTMLElement;
  message: string;
  bestPractice: string;
}

export interface WCAGCompliance {
  levelA: boolean;
  levelAA: boolean;
  levelAAA: boolean;
  criteria: {
    [criterion: string]: {
      passed: boolean;
      applicableTests: number;
      passedTests: number;
    };
  };
}

export type FocusEvent = {
  type: 'focus' | 'blur';
  element: HTMLElement;
  timestamp: number;
  triggeredBy: 'keyboard' | 'mouse' | 'script';
  previousElement: HTMLElement | null;
};

export type KeyboardEventInfo = {
  key: string;
  code: string;
  keyCode: number;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
  target: HTMLElement;
  timestamp: number;
};
