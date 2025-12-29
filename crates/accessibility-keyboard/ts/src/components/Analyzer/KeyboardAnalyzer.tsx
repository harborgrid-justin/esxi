/**
 * Main Keyboard Analyzer Component
 * Comprehensive keyboard accessibility analysis and testing interface
 */

import React, { useState, useEffect, useCallback } from 'react';
import { TabOrderAnalyzer } from '../../analyzers/TabOrderAnalyzer';
import { FocusTrapAnalyzer } from '../../analyzers/FocusTrapAnalyzer';
import { FocusVisibilityAnalyzer } from '../../analyzers/FocusVisibilityAnalyzer';
import { ShortcutAnalyzer } from '../../analyzers/ShortcutAnalyzer';
import { SkipLinkAnalyzer } from '../../analyzers/SkipLinkAnalyzer';
import { InteractiveElementAnalyzer } from '../../analyzers/InteractiveElementAnalyzer';
import { KeyboardNavigationReport, KeyboardTestConfig } from '../../types';

export interface KeyboardAnalyzerProps {
  targetElement?: HTMLElement;
  config?: Partial<KeyboardTestConfig>;
  onReportGenerated?: (report: KeyboardNavigationReport) => void;
  autoStart?: boolean;
}

export const KeyboardAnalyzer: React.FC<KeyboardAnalyzerProps> = ({
  targetElement,
  config,
  onReportGenerated,
  autoStart = false,
}) => {
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [report, setReport] = useState<KeyboardNavigationReport | null>(null);
  const [progress, setProgress] = useState(0);
  const [currentPhase, setCurrentPhase] = useState('');

  const defaultConfig: KeyboardTestConfig = {
    checkTabOrder: true,
    checkFocusTraps: true,
    checkFocusIndicators: true,
    checkSkipLinks: true,
    checkShortcuts: true,
    checkInteractiveElements: true,
    contrastThreshold: 3.0,
    timeout: 30000,
    ...config,
  };

  const runAnalysis = useCallback(async () => {
    setIsAnalyzing(true);
    setProgress(0);

    const target = targetElement || document.body;
    const phases = [
      'Tab Order Analysis',
      'Focus Trap Detection',
      'Focus Indicator Validation',
      'Skip Link Verification',
      'Keyboard Shortcut Analysis',
      'Interactive Element Check',
    ];

    let phaseIndex = 0;

    try {
      // Phase 1: Tab Order
      setCurrentPhase(phases[phaseIndex++]);
      setProgress(15);
      const tabOrderAnalyzer = new TabOrderAnalyzer();
      const tabOrder = await tabOrderAnalyzer.analyze(target);

      // Phase 2: Focus Traps
      setCurrentPhase(phases[phaseIndex++]);
      setProgress(30);
      const focusTrapAnalyzer = new FocusTrapAnalyzer();
      const focusTraps = await focusTrapAnalyzer.detectTraps(target);

      // Phase 3: Focus Indicators
      setCurrentPhase(phases[phaseIndex++]);
      setProgress(50);
      const focusVisibilityAnalyzer = new FocusVisibilityAnalyzer(
        defaultConfig.contrastThreshold
      );
      const focusIndicators = await focusVisibilityAnalyzer.analyzeAll(target);

      // Phase 4: Skip Links
      setCurrentPhase(phases[phaseIndex++]);
      setProgress(65);
      const skipLinkAnalyzer = new SkipLinkAnalyzer();
      const skipLinks = await skipLinkAnalyzer.analyze(target);

      // Phase 5: Shortcuts
      setCurrentPhase(phases[phaseIndex++]);
      setProgress(80);
      const shortcutAnalyzer = new ShortcutAnalyzer();
      const shortcuts = await shortcutAnalyzer.analyze(target);

      // Phase 6: Interactive Elements
      setCurrentPhase(phases[phaseIndex++]);
      setProgress(95);
      const interactiveAnalyzer = new InteractiveElementAnalyzer();
      const interactiveElements = await interactiveAnalyzer.analyze(target);

      // Generate Report
      const totalIssues =
        tabOrder.issues.length +
        focusTraps.filter((t) => t.detected).length +
        focusIndicators.filter((i) => !i.meetsWCAG).length +
        skipLinks.filter((s) => s.issues.length > 0).length +
        shortcuts.filter((s) => s.conflicts.length > 0).length +
        interactiveElements.filter((e) => !e.isKeyboardAccessible).length;

      const criticalIssues =
        tabOrder.issues.filter((i) => i.severity === 'error').length +
        focusTraps.filter((t) => t.severity === 'critical').length;

      const generatedReport: KeyboardNavigationReport = {
        timestamp: new Date().toISOString(),
        url: window.location.href,
        tabOrder,
        focusTraps,
        focusIndicators,
        skipLinks,
        shortcuts,
        interactiveElements,
        summary: {
          totalFocusable: tabOrder.elements.length,
          totalIssues,
          criticalIssues,
          warnings: totalIssues - criticalIssues,
          passed: criticalIssues === 0,
          wcagLevel: criticalIssues === 0 ? 'AA' : 'fail',
        },
      };

      setReport(generatedReport);
      setProgress(100);

      if (onReportGenerated) {
        onReportGenerated(generatedReport);
      }
    } catch (error) {
      console.error('Analysis failed:', error);
    } finally {
      setIsAnalyzing(false);
      setCurrentPhase('');
    }
  }, [targetElement, defaultConfig, onReportGenerated]);

  useEffect(() => {
    if (autoStart) {
      runAnalysis();
    }
  }, [autoStart, runAnalysis]);

  return (
    <div className="keyboard-analyzer" style={styles.container}>
      <div style={styles.header}>
        <h2 style={styles.title}>Keyboard Navigation Analyzer</h2>
        <button
          onClick={runAnalysis}
          disabled={isAnalyzing}
          style={styles.button}
        >
          {isAnalyzing ? 'Analyzing...' : 'Run Analysis'}
        </button>
      </div>

      {isAnalyzing && (
        <div style={styles.progress}>
          <div style={styles.progressLabel}>
            {currentPhase} - {progress}%
          </div>
          <div style={styles.progressBar}>
            <div
              style={{
                ...styles.progressFill,
                width: `${progress}%`,
              }}
            />
          </div>
        </div>
      )}

      {report && (
        <div style={styles.report}>
          <div style={styles.summary}>
            <h3>Summary</h3>
            <div style={styles.summaryGrid}>
              <div style={styles.summaryItem}>
                <div style={styles.summaryLabel}>Focusable Elements</div>
                <div style={styles.summaryValue}>
                  {report.summary.totalFocusable}
                </div>
              </div>
              <div style={styles.summaryItem}>
                <div style={styles.summaryLabel}>Total Issues</div>
                <div style={styles.summaryValue}>
                  {report.summary.totalIssues}
                </div>
              </div>
              <div style={styles.summaryItem}>
                <div style={styles.summaryLabel}>Critical</div>
                <div style={{ ...styles.summaryValue, color: '#dc3545' }}>
                  {report.summary.criticalIssues}
                </div>
              </div>
              <div style={styles.summaryItem}>
                <div style={styles.summaryLabel}>WCAG Level</div>
                <div
                  style={{
                    ...styles.summaryValue,
                    color: report.summary.passed ? '#28a745' : '#dc3545',
                  }}
                >
                  {report.summary.wcagLevel}
                </div>
              </div>
            </div>
          </div>

          <div style={styles.details}>
            <h4>Tab Order Issues: {report.tabOrder.issues.length}</h4>
            <h4>Focus Traps: {report.focusTraps.filter((t) => t.detected).length}</h4>
            <h4>
              Focus Indicator Issues:{' '}
              {report.focusIndicators.filter((i) => !i.meetsWCAG).length}
            </h4>
            <h4>
              Skip Link Issues:{' '}
              {report.skipLinks.filter((s) => s.issues.length > 0).length}
            </h4>
            <h4>
              Shortcut Conflicts:{' '}
              {report.shortcuts.filter((s) => s.conflicts.length > 0).length}
            </h4>
            <h4>
              Non-Accessible Elements:{' '}
              {report.interactiveElements.filter((e) => !e.isKeyboardAccessible).length}
            </h4>
          </div>
        </div>
      )}
    </div>
  );
};

const styles = {
  container: {
    padding: '20px',
    fontFamily: 'system-ui, -apple-system, sans-serif',
    maxWidth: '1200px',
    margin: '0 auto',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '20px',
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: 600,
  },
  button: {
    padding: '10px 20px',
    fontSize: '14px',
    fontWeight: 500,
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  progress: {
    marginBottom: '20px',
  },
  progressLabel: {
    fontSize: '14px',
    marginBottom: '8px',
    fontWeight: 500,
  },
  progressBar: {
    width: '100%',
    height: '20px',
    backgroundColor: '#e9ecef',
    borderRadius: '4px',
    overflow: 'hidden',
  },
  progressFill: {
    height: '100%',
    backgroundColor: '#007bff',
    transition: 'width 0.3s ease',
  },
  report: {
    backgroundColor: '#f8f9fa',
    padding: '20px',
    borderRadius: '8px',
  },
  summary: {
    marginBottom: '20px',
  },
  summaryGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '16px',
    marginTop: '12px',
  },
  summaryItem: {
    backgroundColor: 'white',
    padding: '16px',
    borderRadius: '6px',
    textAlign: 'center' as const,
  },
  summaryLabel: {
    fontSize: '12px',
    color: '#6c757d',
    marginBottom: '8px',
  },
  summaryValue: {
    fontSize: '24px',
    fontWeight: 600,
  },
  details: {
    backgroundColor: 'white',
    padding: '16px',
    borderRadius: '6px',
  },
};
