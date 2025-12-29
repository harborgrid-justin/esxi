/**
 * Focus Indicator Checker Component
 * Validates visibility and quality of focus indicators
 */

import React, { useState, useEffect } from 'react';
import { FocusIndicator } from '../../types';
import { FocusVisibilityAnalyzer } from '../../analyzers/FocusVisibilityAnalyzer';

export interface FocusIndicatorCheckerProps {
  targetElement?: HTMLElement;
  contrastThreshold?: number;
  onAnalysisComplete?: (indicators: FocusIndicator[]) => void;
}

export const FocusIndicatorChecker: React.FC<FocusIndicatorCheckerProps> = ({
  targetElement,
  contrastThreshold = 3.0,
  onAnalysisComplete,
}) => {
  const [indicators, setIndicators] = useState<FocusIndicator[]>([]);
  const [isChecking, setIsChecking] = useState(false);
  const [selectedIndicator, setSelectedIndicator] = useState<FocusIndicator | null>(null);

  const runCheck = async () => {
    setIsChecking(true);
    const analyzer = new FocusVisibilityAnalyzer(contrastThreshold);
    const target = targetElement || document.body;

    try {
      const results = await analyzer.analyzeAll(target);
      setIndicators(results);

      if (onAnalysisComplete) {
        onAnalysisComplete(results);
      }
    } catch (error) {
      console.error('Focus indicator check failed:', error);
    } finally {
      setIsChecking(false);
    }
  };

  useEffect(() => {
    runCheck();
  }, [targetElement, contrastThreshold]);

  const highlightElement = (indicator: FocusIndicator) => {
    indicator.element.scrollIntoView({ behavior: 'smooth', block: 'center' });
    indicator.element.focus();
  };

  const passingCount = indicators.filter((i) => i.meetsWCAG).length;
  const failingCount = indicators.length - passingCount;

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Focus Indicator Checker</h3>
        <button onClick={runCheck} disabled={isChecking} style={styles.button}>
          {isChecking ? 'Checking...' : 'Re-check'}
        </button>
      </div>

      <div style={styles.summary}>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Total Elements</div>
          <div style={styles.summaryValue}>{indicators.length}</div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Passing</div>
          <div style={{ ...styles.summaryValue, color: '#28a745' }}>
            {passingCount}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Failing</div>
          <div style={{ ...styles.summaryValue, color: '#dc3545' }}>
            {failingCount}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Pass Rate</div>
          <div style={styles.summaryValue}>
            {indicators.length > 0
              ? Math.round((passingCount / indicators.length) * 100)
              : 0}
            %
          </div>
        </div>
      </div>

      <div style={styles.indicatorList}>
        {indicators.map((indicator, index) => (
          <div
            key={index}
            style={{
              ...styles.indicatorItem,
              ...(indicator.meetsWCAG ? {} : styles.indicatorFailing),
            }}
            onClick={() => {
              setSelectedIndicator(indicator);
              highlightElement(indicator);
            }}
          >
            <div style={styles.indicatorHeader}>
              <span style={styles.indicatorTag}>
                {indicator.element.tagName.toLowerCase()}
              </span>
              <span
                style={{
                  ...styles.statusBadge,
                  ...(indicator.meetsWCAG
                    ? styles.statusPass
                    : styles.statusFail),
                }}
              >
                {indicator.meetsWCAG ? 'PASS' : 'FAIL'}
              </span>
            </div>

            <div style={styles.indicatorDetails}>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Outline:</span>
                <span style={styles.detailValue}>
                  {indicator.hasOutline ? 'Yes' : 'No'}
                </span>
              </div>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Custom Indicator:</span>
                <span style={styles.detailValue}>
                  {indicator.hasCustomIndicator ? 'Yes' : 'No'}
                </span>
              </div>
              {indicator.contrastRatio !== null && (
                <div style={styles.detailRow}>
                  <span style={styles.detailLabel}>Contrast Ratio:</span>
                  <span
                    style={{
                      ...styles.detailValue,
                      color:
                        indicator.contrastRatio >= contrastThreshold
                          ? '#28a745'
                          : '#dc3545',
                      fontWeight: 600,
                    }}
                  >
                    {indicator.contrastRatio.toFixed(2)}:1
                  </span>
                </div>
              )}
            </div>

            {indicator.styles.outline && (
              <div style={styles.styleInfo}>
                <code style={styles.code}>
                  outline: {indicator.styles.outline}
                </code>
              </div>
            )}

            {indicator.styles.boxShadow && (
              <div style={styles.styleInfo}>
                <code style={styles.code}>
                  box-shadow: {indicator.styles.boxShadow}
                </code>
              </div>
            )}

            {indicator.issues.length > 0 && (
              <div style={styles.issuesList}>
                {indicator.issues.map((issue, issueIndex) => (
                  <div key={issueIndex} style={styles.issue}>
                    ⚠ {issue}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {selectedIndicator && (
        <div style={styles.detailPanel}>
          <h4>Focus Indicator Details</h4>
          <div style={styles.detailGrid}>
            <div>
              <strong>Element:</strong> {selectedIndicator.element.tagName}
            </div>
            <div>
              <strong>Has Outline:</strong>{' '}
              {selectedIndicator.hasOutline ? 'Yes' : 'No'}
            </div>
            <div>
              <strong>Custom Indicator:</strong>{' '}
              {selectedIndicator.hasCustomIndicator ? 'Yes' : 'No'}
            </div>
            <div>
              <strong>WCAG Compliant:</strong>{' '}
              {selectedIndicator.meetsWCAG ? 'Yes' : 'No'}
            </div>
          </div>

          <div style={styles.stylesSection}>
            <h5>Applied Styles</h5>
            {selectedIndicator.styles.outline && (
              <div>
                <code>outline: {selectedIndicator.styles.outline}</code>
              </div>
            )}
            {selectedIndicator.styles.border && (
              <div>
                <code>border: {selectedIndicator.styles.border}</code>
              </div>
            )}
            {selectedIndicator.styles.boxShadow && (
              <div>
                <code>box-shadow: {selectedIndicator.styles.boxShadow}</code>
              </div>
            )}
            {selectedIndicator.styles.backgroundColor && (
              <div>
                <code>
                  background-color: {selectedIndicator.styles.backgroundColor}
                </code>
              </div>
            )}
          </div>

          {selectedIndicator.issues.length > 0 && (
            <div style={styles.issuesSection}>
              <h5>Issues</h5>
              <ul style={styles.issueList}>
                {selectedIndicator.issues.map((issue, idx) => (
                  <li key={idx}>{issue}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

const styles = {
  container: {
    padding: '16px',
    backgroundColor: '#fff',
    borderRadius: '8px',
    border: '1px solid #dee2e6',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '16px',
  },
  title: {
    margin: 0,
    fontSize: '18px',
    fontWeight: 600,
  },
  button: {
    padding: '8px 16px',
    fontSize: '14px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  summary: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '12px',
    marginBottom: '20px',
  },
  summaryItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    textAlign: 'center' as const,
  },
  summaryLabel: {
    fontSize: '12px',
    color: '#6c757d',
    marginBottom: '4px',
  },
  summaryValue: {
    fontSize: '20px',
    fontWeight: 600,
  },
  indicatorList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
    maxHeight: '500px',
    overflowY: 'auto' as const,
  },
  indicatorItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    border: '1px solid #dee2e6',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  indicatorFailing: {
    borderColor: '#dc3545',
    backgroundColor: '#fff5f5',
  },
  indicatorHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px',
  },
  indicatorTag: {
    fontFamily: 'monospace',
    fontSize: '14px',
    fontWeight: 600,
  },
  statusBadge: {
    padding: '4px 8px',
    borderRadius: '4px',
    fontSize: '11px',
    fontWeight: 600,
  },
  statusPass: {
    backgroundColor: '#d4edda',
    color: '#155724',
  },
  statusFail: {
    backgroundColor: '#f8d7da',
    color: '#721c24',
  },
  indicatorDetails: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
    marginBottom: '8px',
  },
  detailRow: {
    display: 'flex',
    fontSize: '13px',
  },
  detailLabel: {
    fontWeight: 500,
    marginRight: '8px',
    color: '#6c757d',
  },
  detailValue: {
    color: '#495057',
  },
  styleInfo: {
    marginTop: '8px',
    padding: '8px',
    backgroundColor: '#fff',
    borderRadius: '4px',
    fontSize: '12px',
  },
  code: {
    fontFamily: 'monospace',
    fontSize: '12px',
  },
  issuesList: {
    marginTop: '8px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
  },
  issue: {
    padding: '6px 8px',
    backgroundColor: '#fff3cd',
    border: '1px solid #ffc107',
    borderRadius: '4px',
    fontSize: '12px',
    color: '#856404',
  },
  detailPanel: {
    marginTop: '16px',
    padding: '16px',
    backgroundColor: '#e9ecef',
    borderRadius: '6px',
  },
  detailGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '8px',
    marginBottom: '12px',
    fontSize: '14px',
  },
  stylesSection: {
    marginTop: '12px',
    padding: '12px',
    backgroundColor: '#fff',
    borderRadius: '4px',
  },
  issuesSection: {
    marginTop: '12px',
  },
  issueList: {
    margin: '8px 0',
    paddingLeft: '20px',
    fontSize: '13px',
  },
};
