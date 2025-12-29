/**
 * Tab Order Visualizer Component
 * Visual representation of tab order and navigation sequence
 */

import React, { useEffect, useState } from 'react';
import { FocusableElement, TabOrder } from '../../types';

export interface TabOrderVisualizerProps {
  tabOrder: TabOrder;
  highlightIssues?: boolean;
  showLabels?: boolean;
}

export const TabOrderVisualizer: React.FC<TabOrderVisualizerProps> = ({
  tabOrder,
  highlightIssues = true,
  showLabels = true,
}) => {
  const [selectedElement, setSelectedElement] = useState<FocusableElement | null>(null);
  const [highlightedIndex, setHighlightedIndex] = useState<number>(-1);

  useEffect(() => {
    // Cleanup function to remove any highlights
    return () => {
      removeAllHighlights();
    };
  }, []);

  const removeAllHighlights = () => {
    document.querySelectorAll('.tab-order-highlight').forEach((el) => {
      el.remove();
    });
  };

  const highlightElement = (focusableEl: FocusableElement, index: number) => {
    removeAllHighlights();

    const rect = focusableEl.element.getBoundingClientRect();
    const highlight = document.createElement('div');
    highlight.className = 'tab-order-highlight';
    highlight.style.cssText = `
      position: fixed;
      top: ${rect.top}px;
      left: ${rect.left}px;
      width: ${rect.width}px;
      height: ${rect.height}px;
      border: 3px solid #007bff;
      background-color: rgba(0, 123, 255, 0.1);
      pointer-events: none;
      z-index: 999999;
      border-radius: 4px;
    `;

    if (showLabels) {
      const label = document.createElement('div');
      label.style.cssText = `
        position: absolute;
        top: -24px;
        left: 0;
        background: #007bff;
        color: white;
        padding: 2px 8px;
        border-radius: 3px;
        font-size: 12px;
        font-weight: bold;
        font-family: system-ui, -apple-system, sans-serif;
      `;
      label.textContent = `#${index + 1}`;
      highlight.appendChild(label);
    }

    document.body.appendChild(highlight);
  };

  const getElementIssues = (element: FocusableElement) => {
    return tabOrder.issues.filter((issue) => issue.element === element);
  };

  const handleElementHover = (element: FocusableElement, index: number) => {
    setSelectedElement(element);
    setHighlightedIndex(index);
    highlightElement(element, index);
  };

  const handleElementLeave = () => {
    setSelectedElement(null);
    setHighlightedIndex(-1);
    removeAllHighlights();
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Tab Order Visualization</h3>
        <div style={styles.stats}>
          <span style={styles.stat}>
            Total: {tabOrder.elements.length}
          </span>
          <span style={{ ...styles.stat, color: '#dc3545' }}>
            Issues: {tabOrder.issues.length}
          </span>
        </div>
      </div>

      <div style={styles.elementList}>
        {tabOrder.elements.map((element, index) => {
          const issues = getElementIssues(element);
          const hasIssues = issues.length > 0;
          const isHighlighted = highlightedIndex === index;

          return (
            <div
              key={index}
              style={{
                ...styles.elementItem,
                ...(hasIssues && highlightIssues ? styles.elementWithIssue : {}),
                ...(isHighlighted ? styles.elementHighlighted : {}),
              }}
              onMouseEnter={() => handleElementHover(element, index)}
              onMouseLeave={handleElementLeave}
            >
              <div style={styles.elementHeader}>
                <span style={styles.elementIndex}>#{index + 1}</span>
                <span style={styles.elementTag}>{element.element.tagName}</span>
                {element.tabIndex > 0 && (
                  <span style={styles.tabIndexBadge}>
                    tabindex={element.tabIndex}
                  </span>
                )}
              </div>

              <div style={styles.elementDetails}>
                <div style={styles.elementSelector}>{element.selector}</div>
                {element.ariaLabel && (
                  <div style={styles.elementLabel}>
                    Label: {element.ariaLabel}
                  </div>
                )}
              </div>

              <div style={styles.elementStatus}>
                <span
                  style={{
                    ...styles.statusBadge,
                    ...(element.isVisible ? styles.statusSuccess : styles.statusWarning),
                  }}
                >
                  {element.isVisible ? 'Visible' : 'Hidden'}
                </span>
                <span
                  style={{
                    ...styles.statusBadge,
                    ...(element.hasFocusIndicator
                      ? styles.statusSuccess
                      : styles.statusError),
                  }}
                >
                  {element.hasFocusIndicator ? 'Has Focus' : 'No Focus'}
                </span>
              </div>

              {hasIssues && highlightIssues && (
                <div style={styles.issueList}>
                  {issues.map((issue, issueIndex) => (
                    <div
                      key={issueIndex}
                      style={{
                        ...styles.issue,
                        ...(issue.severity === 'error'
                          ? styles.issueError
                          : styles.issueWarning),
                      }}
                    >
                      <strong>{issue.type}:</strong> {issue.message}
                    </div>
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {selectedElement && (
        <div style={styles.detailPanel}>
          <h4>Element Details</h4>
          <table style={styles.detailTable}>
            <tbody>
              <tr>
                <td style={styles.detailLabel}>Tag:</td>
                <td>{selectedElement.element.tagName}</td>
              </tr>
              <tr>
                <td style={styles.detailLabel}>Tab Index:</td>
                <td>{selectedElement.tabIndex}</td>
              </tr>
              <tr>
                <td style={styles.detailLabel}>Role:</td>
                <td>{selectedElement.computedRole || 'none'}</td>
              </tr>
              <tr>
                <td style={styles.detailLabel}>Selector:</td>
                <td style={styles.codeText}>{selectedElement.selector}</td>
              </tr>
              <tr>
                <td style={styles.detailLabel}>Visual Order:</td>
                <td>{selectedElement.visualTabOrder}</td>
              </tr>
              <tr>
                <td style={styles.detailLabel}>DOM Order:</td>
                <td>{selectedElement.domOrder}</td>
              </tr>
            </tbody>
          </table>
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
    paddingBottom: '12px',
    borderBottom: '2px solid #e9ecef',
  },
  title: {
    margin: 0,
    fontSize: '18px',
    fontWeight: 600,
  },
  stats: {
    display: 'flex',
    gap: '16px',
  },
  stat: {
    fontSize: '14px',
    fontWeight: 500,
  },
  elementList: {
    maxHeight: '500px',
    overflowY: 'auto' as const,
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  elementItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    border: '1px solid #dee2e6',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  elementWithIssue: {
    borderColor: '#dc3545',
    backgroundColor: '#fff5f5',
  },
  elementHighlighted: {
    borderColor: '#007bff',
    backgroundColor: '#e7f3ff',
    boxShadow: '0 2px 8px rgba(0, 123, 255, 0.3)',
  },
  elementHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '8px',
  },
  elementIndex: {
    fontWeight: 700,
    color: '#007bff',
    fontSize: '14px',
  },
  elementTag: {
    fontSize: '13px',
    fontFamily: 'monospace',
    color: '#495057',
  },
  tabIndexBadge: {
    fontSize: '11px',
    padding: '2px 6px',
    backgroundColor: '#ffc107',
    color: '#000',
    borderRadius: '3px',
    fontWeight: 600,
  },
  elementDetails: {
    marginBottom: '8px',
  },
  elementSelector: {
    fontSize: '12px',
    fontFamily: 'monospace',
    color: '#6c757d',
    wordBreak: 'break-all' as const,
  },
  elementLabel: {
    fontSize: '12px',
    color: '#495057',
    marginTop: '4px',
  },
  elementStatus: {
    display: 'flex',
    gap: '8px',
  },
  statusBadge: {
    fontSize: '11px',
    padding: '2px 8px',
    borderRadius: '3px',
    fontWeight: 500,
  },
  statusSuccess: {
    backgroundColor: '#d4edda',
    color: '#155724',
  },
  statusWarning: {
    backgroundColor: '#fff3cd',
    color: '#856404',
  },
  statusError: {
    backgroundColor: '#f8d7da',
    color: '#721c24',
  },
  issueList: {
    marginTop: '8px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
  },
  issue: {
    padding: '6px 8px',
    borderRadius: '4px',
    fontSize: '12px',
  },
  issueError: {
    backgroundColor: '#f8d7da',
    color: '#721c24',
    border: '1px solid #f5c6cb',
  },
  issueWarning: {
    backgroundColor: '#fff3cd',
    color: '#856404',
    border: '1px solid #ffeaa7',
  },
  detailPanel: {
    marginTop: '16px',
    padding: '12px',
    backgroundColor: '#e9ecef',
    borderRadius: '6px',
  },
  detailTable: {
    width: '100%',
    fontSize: '13px',
  },
  detailLabel: {
    fontWeight: 600,
    width: '120px',
    paddingRight: '12px',
    verticalAlign: 'top',
  },
  codeText: {
    fontFamily: 'monospace',
    fontSize: '12px',
  },
};
