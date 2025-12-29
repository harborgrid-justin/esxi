/**
 * Enterprise Keyboard Navigation Validator
 * Main entry point for the accessibility keyboard navigation system
 *
 * @packageDocumentation
 */

// Type exports
export * from './types';

// Component exports
export { KeyboardAnalyzer } from './components/Analyzer/KeyboardAnalyzer';
export type { KeyboardAnalyzerProps } from './components/Analyzer/KeyboardAnalyzer';

export { TabOrderVisualizer } from './components/Analyzer/TabOrderVisualizer';
export type { TabOrderVisualizerProps } from './components/Analyzer/TabOrderVisualizer';

export { FocusIndicatorChecker } from './components/Analyzer/FocusIndicatorChecker';
export type { FocusIndicatorCheckerProps } from './components/Analyzer/FocusIndicatorChecker';

export { FocusTrapDetector } from './components/Analyzer/FocusTrapDetector';
export type { FocusTrapDetectorProps } from './components/Analyzer/FocusTrapDetector';

export { KeyboardTester } from './components/Testing/KeyboardTester';
export type { KeyboardTesterProps } from './components/Testing/KeyboardTester';

export { ShortcutTester } from './components/Testing/ShortcutTester';
export type { ShortcutTesterProps } from './components/Testing/ShortcutTester';

export { FocusTracker } from './components/Testing/FocusTracker';
export type { FocusTrackerProps } from './components/Testing/FocusTracker';

export { TabOrderOverlay } from './components/Visualization/TabOrderOverlay';
export type { TabOrderOverlayProps } from './components/Visualization/TabOrderOverlay';

export { FocusPathMap } from './components/Visualization/FocusPathMap';
export type { FocusPathMapProps } from './components/Visualization/FocusPathMap';

export { SkipLinkChecker } from './components/Visualization/SkipLinkChecker';
export type { SkipLinkCheckerProps } from './components/Visualization/SkipLinkChecker';

export { KeyboardIssues } from './components/Issues/KeyboardIssues';
export type { KeyboardIssuesProps } from './components/Issues/KeyboardIssues';

export { TrapWarning } from './components/Issues/TrapWarning';
export type { TrapWarningProps } from './components/Issues/TrapWarning';

// Analyzer exports
export { TabOrderAnalyzer } from './analyzers/TabOrderAnalyzer';
export { FocusVisibilityAnalyzer } from './analyzers/FocusVisibilityAnalyzer';
export { FocusTrapAnalyzer } from './analyzers/FocusTrapAnalyzer';
export { ShortcutAnalyzer } from './analyzers/ShortcutAnalyzer';
export { SkipLinkAnalyzer } from './analyzers/SkipLinkAnalyzer';
export { InteractiveElementAnalyzer } from './analyzers/InteractiveElementAnalyzer';

// Validator exports
export { TabIndexValidator } from './validators/TabIndexValidator';
export type {
  TabIndexValidationResult,
  TabIndexViolation,
} from './validators/TabIndexValidator';

export { KeyboardOperableValidator } from './validators/KeyboardOperableValidator';
export type {
  KeyboardOperableResult,
  OperableIssue,
} from './validators/KeyboardOperableValidator';

// Hook exports
export { useKeyboardNav } from './hooks/useKeyboardNav';
export type {
  UseKeyboardNavOptions,
  UseKeyboardNavReturn,
} from './hooks/useKeyboardNav';

export { useFocusTracking } from './hooks/useFocusTracking';
export type {
  UseFocusTrackingOptions,
  UseFocusTrackingReturn,
  FocusStats,
} from './hooks/useFocusTracking';

// Utility exports
export * from './utils/focusManagement';

/**
 * Main API for keyboard navigation validation
 */
export const KeyboardValidator = {
  /**
   * Analyzers for different aspects of keyboard navigation
   */
  analyzers: {
    TabOrder: TabOrderAnalyzer,
    FocusVisibility: FocusVisibilityAnalyzer,
    FocusTrap: FocusTrapAnalyzer,
    Shortcut: ShortcutAnalyzer,
    SkipLink: SkipLinkAnalyzer,
    InteractiveElement: InteractiveElementAnalyzer,
  },

  /**
   * Validators for WCAG compliance
   */
  validators: {
    TabIndex: TabIndexValidator,
    KeyboardOperable: KeyboardOperableValidator,
  },
};

/**
 * Quick validation function for common keyboard accessibility issues
 */
export async function validateKeyboardAccessibility(
  container: HTMLElement = document.body
): Promise<{
  passed: boolean;
  issues: number;
  report: any;
}> {
  const tabOrderAnalyzer = new TabOrderAnalyzer();
  const focusTrapAnalyzer = new FocusTrapAnalyzer();
  const focusVisibilityAnalyzer = new FocusVisibilityAnalyzer();
  const skipLinkAnalyzer = new SkipLinkAnalyzer();
  const shortcutAnalyzer = new ShortcutAnalyzer();
  const interactiveAnalyzer = new InteractiveElementAnalyzer();

  const [
    tabOrder,
    focusTraps,
    focusIndicators,
    skipLinks,
    shortcuts,
    interactiveElements,
  ] = await Promise.all([
    tabOrderAnalyzer.analyze(container),
    focusTrapAnalyzer.detectTraps(container),
    focusVisibilityAnalyzer.analyzeAll(container),
    skipLinkAnalyzer.analyze(container),
    shortcutAnalyzer.analyze(container),
    interactiveAnalyzer.analyze(container),
  ]);

  const totalIssues =
    tabOrder.issues.length +
    focusTraps.filter((t) => t.detected).length +
    focusIndicators.filter((i) => !i.meetsWCAG).length +
    skipLinks.filter((s) => !s.worksCorrectly).length +
    shortcuts.filter((s) => s.conflicts.length > 0).length +
    interactiveElements.filter((e) => !e.isKeyboardAccessible).length;

  const criticalIssues =
    tabOrder.issues.filter((i) => i.severity === 'error').length +
    focusTraps.filter((t) => t.severity === 'critical').length;

  return {
    passed: criticalIssues === 0,
    issues: totalIssues,
    report: {
      tabOrder,
      focusTraps,
      focusIndicators,
      skipLinks,
      shortcuts,
      interactiveElements,
      summary: {
        totalIssues,
        criticalIssues,
        warnings: totalIssues - criticalIssues,
      },
    },
  };
}

/**
 * Version information
 */
export const version = '1.0.0';
