/**
 * Keyboard Issues Component
 * Displays and manages keyboard accessibility issues
 */

import React, { useState } from 'react';
import { KeyboardViolation, KeyboardWarning } from '../../types';

export interface KeyboardIssuesProps {
  violations: KeyboardViolation[];
  warnings: KeyboardWarning[];
  onIssueClick?: (issue: KeyboardViolation | KeyboardWarning) => void;
  groupBy?: 'severity' | 'type' | 'wcag';
}

export const KeyboardIssues: React.FC<KeyboardIssuesProps> = ({
  violations,
  warnings,
  onIssueClick,
  groupBy = 'severity',
}) => {
  const [selectedIssue, setSelectedIssue] = useState<KeyboardViolation | KeyboardWarning | null>(
    null
  );
  const [filterSeverity, setFilterSeverity] = useState<string>('all');
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(new Set());

  const toggleGroup = (groupName: string) => {
    const newExpanded = new Set(expandedGroups);
    if (newExpanded.has(groupName)) {
      newExpanded.delete(groupName);
    } else {
      newExpanded.add(groupName);
    }
    setExpandedGroups(newExpanded);
  };

  const groupIssues = () => {
    const grouped: { [key: string]: (KeyboardViolation | KeyboardWarning)[] } = {};

    const allIssues = [...violations, ...warnings];

    allIssues.forEach((issue) => {
      let key: string;

      if (groupBy === 'severity') {
        key = 'severity' in issue ? issue.severity : 'warning';
      } else if (groupBy === 'type') {
        key = issue.type;
      } else {
        // WCAG criteria
        key =
          'wcagCriteria' in issue && issue.wcagCriteria.length > 0
            ? issue.wcagCriteria[0]
            : 'Other';
      }

      if (!grouped[key]) {
        grouped[key] = [];
      }
      grouped[key].push(issue);
    });

    return grouped;
  };

  const filteredViolations =
    filterSeverity === 'all'
      ? violations
      : violations.filter((v) => v.severity === filterSeverity);

  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'critical':
        return '#dc3545';
      case 'serious':
        return '#fd7e14';
      case 'moderate':
        return '#ffc107';
      case 'minor':
        return '#17a2b8';
      default:
        return '#6c757d';
    }
  };

  const highlightElement = (element: HTMLElement) => {
    element.scrollIntoView({ behavior: 'smooth', block: 'center' });

    const rect = element.getBoundingClientRect();
    const existing = document.querySelector('.issue-highlight');
    if (existing) existing.remove();

    const highlight = document.createElement('div');
    highlight.className = 'issue-highlight';
    highlight.style.cssText = `
      position: fixed;
      top: ${rect.top - 5}px;
      left: ${rect.left - 5}px;
      width: ${rect.width + 10}px;
      height: ${rect.height + 10}px;
      border: 3px solid #dc3545;
      background-color: rgba(220, 53, 69, 0.1);
      pointer-events: none;
      z-index: 999999;
      animation: pulse 2s ease-in-out infinite;
    `;
    document.body.appendChild(highlight);

    setTimeout(() => highlight.remove(), 5000);
  };

  const handleIssueClick = (issue: KeyboardViolation | KeyboardWarning) => {
    setSelectedIssue(issue);
    highlightElement(issue.element);

    if (onIssueClick) {
      onIssueClick(issue);
    }
  };

  const groupedIssues = groupIssues();

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Keyboard Issues</h3>
        <div style={styles.filters}>
          <select
            value={filterSeverity}
            onChange={(e) => setFilterSeverity(e.target.value)}
            style={styles.select}
          >
            <option value="all">All Severities</option>
            <option value="critical">Critical</option>
            <option value="serious">Serious</option>
            <option value="moderate">Moderate</option>
            <option value="minor">Minor</option>
          </select>
        </div>
      </div>

      <div style={styles.summary}>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Total Violations</div>
          <div style={{ ...styles.summaryValue, color: '#dc3545' }}>
            {violations.length}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Warnings</div>
          <div style={{ ...styles.summaryValue, color: '#ffc107' }}>
            {warnings.length}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Total Issues</div>
          <div style={styles.summaryValue}>{violations.length + warnings.length}</div>
        </div>
      </div>

      <div style={styles.issueList}>
        {Object.entries(groupedIssues).map(([groupName, issues]) => (
          <div key={groupName} style={styles.group}>
            <div
              style={styles.groupHeader}
              onClick={() => toggleGroup(groupName)}
            >
              <span style={styles.groupTitle}>
                {groupBy === 'severity' && (
                  <span
                    style={{
                      ...styles.severityDot,
                      backgroundColor: getSeverityColor(groupName),
                    }}
                  />
                )}
                {groupName} ({issues.length})
              </span>
              <span style={styles.groupToggle}>
                {expandedGroups.has(groupName) ? '▼' : '▶'}
              </span>
            </div>

            {expandedGroups.has(groupName) && (
              <div style={styles.groupContent}>
                {issues.map((issue, index) => (
                  <div
                    key={index}
                    style={styles.issueItem}
                    onClick={() => handleIssueClick(issue)}
                  >
                    <div style={styles.issueHeader}>
                      {'severity' in issue && (
                        <span
                          style={{
                            ...styles.severityBadge,
                            backgroundColor: getSeverityColor(issue.severity),
                          }}
                        >
                          {issue.severity}
                        </span>
                      )}
                      <span style={styles.issueType}>{issue.type}</span>
                    </div>

                    <div style={styles.issueDescription}>
                      {'description' in issue ? issue.description : issue.message}
                    </div>

                    <div style={styles.issueElement}>
                      Element: {issue.element.tagName.toLowerCase()}
                    </div>

                    {'wcagCriteria' in issue && issue.wcagCriteria.length > 0 && (
                      <div style={styles.wcagCriteria}>
                        WCAG: {issue.wcagCriteria.join(', ')}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {selectedIssue && (
        <div style={styles.detailPanel}>
          <h4>Issue Details</h4>

          <div style={styles.detailSection}>
            <strong>Type:</strong> {selectedIssue.type}
          </div>

          {'severity' in selectedIssue && (
            <div style={styles.detailSection}>
              <strong>Severity:</strong>{' '}
              <span
                style={{
                  color: getSeverityColor(selectedIssue.severity),
                  fontWeight: 600,
                }}
              >
                {selectedIssue.severity.toUpperCase()}
              </span>
            </div>
          )}

          <div style={styles.detailSection}>
            <strong>Description:</strong>
            <p>
              {'description' in selectedIssue
                ? selectedIssue.description
                : selectedIssue.message}
            </p>
          </div>

          {'wcagCriteria' in selectedIssue && selectedIssue.wcagCriteria.length > 0 && (
            <div style={styles.detailSection}>
              <strong>WCAG Criteria:</strong>
              <ul style={styles.criteriaList}>
                {selectedIssue.wcagCriteria.map((criteria, idx) => (
                  <li key={idx}>{criteria}</li>
                ))}
              </ul>
            </div>
          )}

          {'impact' in selectedIssue && (
            <div style={styles.detailSection}>
              <strong>Impact:</strong>
              <p>{selectedIssue.impact}</p>
            </div>
          )}

          {'howToFix' in selectedIssue && (
            <div style={styles.fixSection}>
              <strong>How to Fix:</strong>
              <p>{selectedIssue.howToFix}</p>
            </div>
          )}

          {'bestPractice' in selectedIssue && (
            <div style={styles.fixSection}>
              <strong>Best Practice:</strong>
              <p>{selectedIssue.bestPractice}</p>
            </div>
          )}

          <button
            onClick={() => highlightElement(selectedIssue.element)}
            style={styles.highlightButton}
          >
            Highlight Element
          </button>
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
  filters: {
    display: 'flex',
    gap: '8px',
  },
  select: {
    padding: '6px 12px',
    fontSize: '13px',
    border: '1px solid #dee2e6',
    borderRadius: '4px',
    backgroundColor: '#fff',
  },
  summary: {
    display: 'grid',
    gridTemplateColumns: 'repeat(3, 1fr)',
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
  issueList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
  },
  group: {
    border: '1px solid #dee2e6',
    borderRadius: '6px',
    overflow: 'hidden',
  },
  groupHeader: {
    padding: '12px 16px',
    backgroundColor: '#f8f9fa',
    cursor: 'pointer',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    fontWeight: 600,
    fontSize: '14px',
  },
  groupTitle: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  severityDot: {
    width: '12px',
    height: '12px',
    borderRadius: '50%',
  },
  groupToggle: {
    fontSize: '12px',
    color: '#6c757d',
  },
  groupContent: {
    padding: '12px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  issueItem: {
    padding: '12px',
    backgroundColor: '#fff',
    border: '1px solid #dee2e6',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  issueHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '8px',
  },
  severityBadge: {
    padding: '3px 8px',
    borderRadius: '3px',
    color: 'white',
    fontSize: '11px',
    fontWeight: 700,
    textTransform: 'uppercase' as const,
  },
  issueType: {
    fontSize: '13px',
    fontWeight: 600,
    color: '#495057',
  },
  issueDescription: {
    fontSize: '13px',
    color: '#495057',
    marginBottom: '8px',
    lineHeight: 1.4,
  },
  issueElement: {
    fontSize: '12px',
    fontFamily: 'monospace',
    color: '#6c757d',
  },
  wcagCriteria: {
    fontSize: '12px',
    color: '#007bff',
    marginTop: '4px',
  },
  detailPanel: {
    marginTop: '20px',
    padding: '16px',
    backgroundColor: '#e9ecef',
    borderRadius: '6px',
  },
  detailSection: {
    marginBottom: '16px',
    fontSize: '14px',
  },
  criteriaList: {
    marginTop: '8px',
    paddingLeft: '20px',
    fontSize: '13px',
  },
  fixSection: {
    marginTop: '16px',
    padding: '12px',
    backgroundColor: '#d1ecf1',
    border: '1px solid #bee5eb',
    borderRadius: '4px',
    fontSize: '13px',
  },
  highlightButton: {
    marginTop: '16px',
    padding: '8px 16px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '13px',
  },
};
