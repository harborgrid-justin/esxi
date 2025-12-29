/**
 * React hook for screen reader analysis
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  AccessibilityNode,
  AccessibilityReport,
  ScreenReaderType,
  BrowserType,
  ScreenReaderConfig,
  Announcement,
} from '../types';
import { AccessibilityTreeBuilder } from '../analyzers/AccessibilityTreeBuilder';
import { ReadingOrderAnalyzer } from '../analyzers/ReadingOrderAnalyzer';
import { LandmarkAnalyzer } from '../analyzers/LandmarkAnalyzer';
import { HeadingAnalyzer } from '../analyzers/HeadingAnalyzer';
import { FormAnalyzer } from '../analyzers/FormAnalyzer';
import { LiveRegionAnalyzer } from '../analyzers/LiveRegionAnalyzer';
import { NVDASimulator } from '../simulators/NVDASimulator';
import { JAWSSimulator } from '../simulators/JAWSSimulator';
import { VoiceOverSimulator } from '../simulators/VoiceOverSimulator';

export interface UseScreenReaderOptions {
  root?: Element;
  autoAnalyze?: boolean;
  screenReader?: ScreenReaderType;
  browser?: BrowserType;
  config?: Partial<ScreenReaderConfig>;
}

export interface UseScreenReaderReturn {
  report: AccessibilityReport | null;
  tree: AccessibilityNode | null;
  currentNode: AccessibilityNode | null;
  announcement: Announcement | null;
  isAnalyzing: boolean;
  analyze: () => void;
  navigateTo: (node: AccessibilityNode) => void;
  navigateNext: () => void;
  navigatePrevious: () => void;
  navigateNextHeading: (level?: number) => void;
  navigateNextLandmark: () => void;
  navigateNextLink: () => void;
  navigateNextFormField: () => void;
  setScreenReader: (type: ScreenReaderType) => void;
  setVerbosity: (level: 'minimal' | 'normal' | 'verbose') => void;
  getSimulator: () => NVDASimulator | JAWSSimulator | VoiceOverSimulator;
}

export function useScreenReader(options: UseScreenReaderOptions = {}): UseScreenReaderReturn {
  const {
    root = document.body,
    autoAnalyze = true,
    screenReader: initialScreenReader = 'NVDA',
    browser = 'Chrome',
    config = {},
  } = options;

  const [report, setReport] = useState<AccessibilityReport | null>(null);
  const [tree, setTree] = useState<AccessibilityNode | null>(null);
  const [currentNode, setCurrentNode] = useState<AccessibilityNode | null>(null);
  const [announcement, setAnnouncement] = useState<Announcement | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [screenReader, setScreenReaderType] = useState<ScreenReaderType>(initialScreenReader);

  const treeBuilder = useRef(new AccessibilityTreeBuilder());
  const readingOrderAnalyzer = useRef(new ReadingOrderAnalyzer());
  const landmarkAnalyzer = useRef(new LandmarkAnalyzer());
  const headingAnalyzer = useRef(new HeadingAnalyzer());
  const formAnalyzer = useRef(new FormAnalyzer());
  const liveRegionAnalyzer = useRef(new LiveRegionAnalyzer());

  const nvdaSimulator = useRef(new NVDASimulator());
  const jawsSimulator = useRef(new JAWSSimulator());
  const voiceOverSimulator = useRef(new VoiceOverSimulator());

  /**
   * Get current simulator
   */
  const getSimulator = useCallback(() => {
    switch (screenReader) {
      case 'NVDA':
        return nvdaSimulator.current;
      case 'JAWS':
        return jawsSimulator.current;
      case 'VoiceOver':
        return voiceOverSimulator.current;
      default:
        return nvdaSimulator.current;
    }
  }, [screenReader]);

  /**
   * Analyze accessibility
   */
  const analyze = useCallback(() => {
    setIsAnalyzing(true);

    try {
      // Build accessibility tree
      const accessibilityTree = treeBuilder.current.buildTree(root);
      setTree(accessibilityTree);

      // Analyze different aspects
      const readingOrder = readingOrderAnalyzer.current.analyze(accessibilityTree);
      const landmarks = landmarkAnalyzer.current.analyze(accessibilityTree);
      const headings = headingAnalyzer.current.analyze(accessibilityTree);
      const forms = formAnalyzer.current.analyze(accessibilityTree);
      const liveRegions = liveRegionAnalyzer.current.analyze(accessibilityTree);

      // Collect all issues
      const issues = [
        ...readingOrder.issues.map(issue => ({
          id: `reading-order-${Math.random()}`,
          type: issue.type,
          severity: issue.severity,
          node: issue.items[0]?.node || accessibilityTree,
          description: issue.description,
          remediation: issue.remediation,
          wcagCriteria: ['2.4.3'],
          screenReadersAffected: ['NVDA', 'JAWS', 'VoiceOver'] as ScreenReaderType[],
        })),
        ...landmarks.issues.map(issue => ({
          id: `landmark-${Math.random()}`,
          type: issue.type,
          severity: issue.severity,
          node: issue.node,
          description: issue.description,
          remediation: issue.remediation,
          wcagCriteria: ['1.3.1', '2.4.1'],
          screenReadersAffected: ['NVDA', 'JAWS', 'VoiceOver'] as ScreenReaderType[],
        })),
        ...headings.issues.map(issue => ({
          id: `heading-${Math.random()}`,
          type: issue.type,
          severity: issue.severity,
          node: issue.heading.node,
          description: issue.description,
          remediation: issue.remediation,
          wcagCriteria: ['1.3.1', '2.4.6'],
          screenReadersAffected: ['NVDA', 'JAWS', 'VoiceOver'] as ScreenReaderType[],
        })),
        ...forms.issues.map(issue => ({
          id: `form-${Math.random()}`,
          type: issue.type,
          severity: issue.severity,
          node: issue.field.node,
          description: issue.description,
          remediation: issue.remediation,
          wcagCriteria: ['1.3.1', '3.3.1', '3.3.2', '4.1.2'],
          screenReadersAffected: ['NVDA', 'JAWS', 'VoiceOver'] as ScreenReaderType[],
        })),
        ...liveRegions.issues.map(issue => ({
          id: `live-region-${Math.random()}`,
          type: issue.type,
          severity: issue.severity,
          node: issue.region.node,
          description: issue.description,
          remediation: issue.remediation,
          wcagCriteria: ['4.1.2', '4.1.3'],
          screenReadersAffected: ['NVDA', 'JAWS', 'VoiceOver'] as ScreenReaderType[],
        })),
      ];

      // Count nodes
      const countNodes = (node: AccessibilityNode): number => {
        return 1 + node.children.reduce((sum, child) => sum + countNodes(child), 0);
      };

      const countFocusable = (node: AccessibilityNode): number => {
        const focusable = node.focusable ? 1 : 0;
        return focusable + node.children.reduce((sum, child) => sum + countFocusable(child), 0);
      };

      const countHidden = (node: AccessibilityNode): number => {
        const hidden = node.hidden ? 1 : 0;
        return hidden + node.children.reduce((sum, child) => sum + countHidden(child), 0);
      };

      // Calculate overall score
      const scores = [
        readingOrder.score,
        landmarks.score,
        headings.score,
        forms.score,
        liveRegions.score,
      ];
      const overallScore = Math.round(scores.reduce((sum, score) => sum + score, 0) / scores.length);

      // Count issues by severity
      const criticalIssues = issues.filter(i => i.severity === 'critical').length;
      const seriousIssues = issues.filter(i => i.severity === 'serious').length;
      const moderateIssues = issues.filter(i => i.severity === 'moderate').length;
      const minorIssues = issues.filter(i => i.severity === 'minor').length;

      // Create report
      const accessibilityReport: AccessibilityReport = {
        timestamp: new Date(),
        url: window.location.href,
        tree: accessibilityTree,
        readingOrder,
        landmarks,
        headings,
        forms,
        liveRegions,
        issues,
        score: overallScore,
        summary: {
          totalNodes: countNodes(accessibilityTree),
          focusableNodes: countFocusable(accessibilityTree),
          hiddenNodes: countHidden(accessibilityTree),
          criticalIssues,
          seriousIssues,
          moderateIssues,
          minorIssues,
        },
      };

      setReport(accessibilityReport);
    } catch (error) {
      console.error('Error analyzing accessibility:', error);
    } finally {
      setIsAnalyzing(false);
    }
  }, [root]);

  /**
   * Navigate to specific node
   */
  const navigateTo = useCallback((node: AccessibilityNode) => {
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigateTo(node);
    setCurrentNode(node);
    setAnnouncement(newAnnouncement);
  }, [getSimulator]);

  /**
   * Navigate to next element
   */
  const navigateNext = useCallback(() => {
    if (!tree) return;
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigateNext(tree);
    if (newAnnouncement) {
      setCurrentNode(simulator.getState().currentNode);
      setAnnouncement(newAnnouncement);
    }
  }, [tree, getSimulator]);

  /**
   * Navigate to previous element
   */
  const navigatePrevious = useCallback(() => {
    if (!tree) return;
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigatePrevious(tree);
    if (newAnnouncement) {
      setCurrentNode(simulator.getState().currentNode);
      setAnnouncement(newAnnouncement);
    }
  }, [tree, getSimulator]);

  /**
   * Navigate to next heading
   */
  const navigateNextHeading = useCallback((level?: number) => {
    if (!tree) return;
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigateNextHeading(tree, level);
    if (newAnnouncement) {
      setCurrentNode(simulator.getState().currentNode);
      setAnnouncement(newAnnouncement);
    }
  }, [tree, getSimulator]);

  /**
   * Navigate to next landmark
   */
  const navigateNextLandmark = useCallback(() => {
    if (!tree) return;
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigateNextLandmark(tree);
    if (newAnnouncement) {
      setCurrentNode(simulator.getState().currentNode);
      setAnnouncement(newAnnouncement);
    }
  }, [tree, getSimulator]);

  /**
   * Navigate to next link
   */
  const navigateNextLink = useCallback(() => {
    if (!tree) return;
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigateNextLink(tree);
    if (newAnnouncement) {
      setCurrentNode(simulator.getState().currentNode);
      setAnnouncement(newAnnouncement);
    }
  }, [tree, getSimulator]);

  /**
   * Navigate to next form field
   */
  const navigateNextFormField = useCallback(() => {
    if (!tree) return;
    const simulator = getSimulator();
    const newAnnouncement = simulator.navigateNextFormField(tree);
    if (newAnnouncement) {
      setCurrentNode(simulator.getState().currentNode);
      setAnnouncement(newAnnouncement);
    }
  }, [tree, getSimulator]);

  /**
   * Set screen reader type
   */
  const setScreenReader = useCallback((type: ScreenReaderType) => {
    setScreenReaderType(type);
  }, []);

  /**
   * Set verbosity level
   */
  const setVerbosity = useCallback((level: 'minimal' | 'normal' | 'verbose') => {
    const simulator = getSimulator();
    simulator.setVerbosity(level);
  }, [getSimulator]);

  /**
   * Auto-analyze on mount or when root changes
   */
  useEffect(() => {
    if (autoAnalyze) {
      analyze();
    }
  }, [autoAnalyze, analyze]);

  /**
   * Watch for DOM changes
   */
  useEffect(() => {
    if (!autoAnalyze) return;

    const observer = new MutationObserver(() => {
      // Debounce re-analysis
      const timeoutId = setTimeout(analyze, 500);
      return () => clearTimeout(timeoutId);
    });

    observer.observe(root, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['aria-*', 'role', 'alt', 'title', 'tabindex'],
    });

    return () => observer.disconnect();
  }, [root, autoAnalyze, analyze]);

  return {
    report,
    tree,
    currentNode,
    announcement,
    isAnalyzing,
    analyze,
    navigateTo,
    navigateNext,
    navigatePrevious,
    navigateNextHeading,
    navigateNextLandmark,
    navigateNextLink,
    navigateNextFormField,
    setScreenReader,
    setVerbosity,
    getSimulator,
  };
}
