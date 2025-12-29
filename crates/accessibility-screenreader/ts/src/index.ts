/**
 * @harborgrid/accessibility-screenreader
 * Enterprise screen reader compatibility checker and simulator
 */

// Types
export * from './types';

// Components
export { ScreenReaderAnalyzer } from './components/Analyzer/ScreenReaderAnalyzer';
export { AccessibilityTree } from './components/Analyzer/AccessibilityTree';
export { ReadingOrderViewer } from './components/Analyzer/ReadingOrderViewer';
export { AnnouncementPreview } from './components/Analyzer/AnnouncementPreview';
export { VirtualScreenReader } from './components/Testing/VirtualScreenReader';
export { NavigationTester } from './components/Testing/NavigationTester';
export { FormTester } from './components/Testing/FormTester';
export { IssueHighlighter } from './components/Issues/IssueHighlighter';
export { IssueDetails } from './components/Issues/IssueDetails';
export { RemediationGuide } from './components/Issues/RemediationGuide';

// Analyzers
export { AccessibilityTreeBuilder } from './analyzers/AccessibilityTreeBuilder';
export { ReadingOrderAnalyzer } from './analyzers/ReadingOrderAnalyzer';
export { AnnouncementGenerator } from './analyzers/AnnouncementGenerator';
export { LandmarkAnalyzer } from './analyzers/LandmarkAnalyzer';
export { HeadingAnalyzer } from './analyzers/HeadingAnalyzer';
export { FormAnalyzer } from './analyzers/FormAnalyzer';
export { LiveRegionAnalyzer } from './analyzers/LiveRegionAnalyzer';

// Simulators
export { NVDASimulator } from './simulators/NVDASimulator';
export { JAWSSimulator } from './simulators/JAWSSimulator';
export { VoiceOverSimulator } from './simulators/VoiceOverSimulator';

// Hooks
export { useScreenReader } from './hooks/useScreenReader';
export { useAccessibilityTree } from './hooks/useAccessibilityTree';
