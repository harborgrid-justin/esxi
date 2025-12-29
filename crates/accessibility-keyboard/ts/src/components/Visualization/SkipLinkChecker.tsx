/**
 * Skip Link Checker Component
 * Validates and tests skip links for keyboard navigation
 */

import React, { useState, useEffect } from 'react';
import { SkipLink } from '../../types';
import { SkipLinkAnalyzer } from '../../analyzers/SkipLinkAnalyzer';

export interface SkipLinkCheckerProps {
  autoCheck?: boolean;
  highlightLinks?: boolean;
  onCheckComplete?: (links: SkipLink[]) => void;
}

export const SkipLinkChecker: React.FC<SkipLinkCheckerProps> = ({
  autoCheck = true,
  highlightLinks = true,
  onCheckComplete,
}) => {
  const [skipLinks, setSkipLinks] = useState<SkipLink[]>([]);
  const [isChecking, setIsChecking] = useState(false);
  const [selectedLink, setSelectedLink] = useState<SkipLink | null>(null);

  useEffect(() => {
    if (autoCheck) {
      checkSkipLinks();
    }
  }, [autoCheck]);

  const checkSkipLinks = async () => {
    setIsChecking(true);
    const analyzer = new SkipLinkAnalyzer();

    try {
      const results = await analyzer.analyze(document.body);
      setSkipLinks(results);

      if (onCheckComplete) {
        onCheckComplete(results);
      }
    } catch (error) {
      console.error('Skip link check failed:', error);
    } finally {
      setIsChecking(false);
    }
  };

  const testSkipLink = (link: SkipLink) => {
    // Highlight the skip link element
    link.element.scrollIntoView({ behavior: 'smooth', block: 'start' });
    link.element.focus();

    // After a delay, try to navigate to target
    setTimeout(() => {
      const targetElement = document.querySelector(link.target);
      if (targetElement) {
        (targetElement as HTMLElement).scrollIntoView({
          behavior: 'smooth',
          block: 'center',
        });
        (targetElement as HTMLElement).focus();
      }
    }, 1000);
  };

  const highlightElement = (element: HTMLElement, color: string) => {
    const rect = element.getBoundingClientRect();
    const existing = document.querySelector('.skip-link-highlight');
    if (existing) existing.remove();

    const highlight = document.createElement('div');
    highlight.className = 'skip-link-highlight';
    highlight.style.cssText = `
      position: fixed;
      top: ${rect.top}px;
      left: ${rect.left}px;
      width: ${rect.width}px;
      height: ${rect.height}px;
      border: 3px solid ${color};
      background-color: ${color}33;
      pointer-events: none;
      z-index: 999999;
      border-radius: 4px;
    `;
    document.body.appendChild(highlight);

    setTimeout(() => highlight.remove(), 3000);
  };

  const validLinks = skipLinks.filter((link) => link.worksCorrectly);
  const invalidLinks = skipLinks.filter((link) => !link.worksCorrectly);

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Skip Link Checker</h3>
        <button onClick={checkSkipLinks} disabled={isChecking} style={styles.button}>
          {isChecking ? 'Checking...' : 'Re-check'}
        </button>
      </div>

      <div style={styles.summary}>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Total Skip Links</div>
          <div style={styles.summaryValue}>{skipLinks.length}</div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Valid</div>
          <div style={{ ...styles.summaryValue, color: '#28a745' }}>
            {validLinks.length}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Invalid</div>
          <div style={{ ...styles.summaryValue, color: '#dc3545' }}>
            {invalidLinks.length}
          </div>
        </div>
      </div>

      {skipLinks.length === 0 && !isChecking && (
        <div style={styles.warning}>
          ⚠️ No skip links found! Consider adding skip navigation links for better accessibility.
        </div>
      )}

      <div style={styles.linkList}>
        {skipLinks.map((link, index) => (
          <div
            key={index}
            style={{
              ...styles.linkItem,
              ...(link.worksCorrectly ? styles.linkValid : styles.linkInvalid),
            }}
            onClick={() => setSelectedLink(link)}
          >
            <div style={styles.linkHeader}>
              <span
                style={{
                  ...styles.statusBadge,
                  ...(link.worksCorrectly ? styles.statusValid : styles.statusInvalid),
                }}
              >
                {link.worksCorrectly ? 'VALID' : 'INVALID'}
              </span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  testSkipLink(link);
                }}
                style={styles.testButton}
              >
                Test
              </button>
            </div>

            <div style={styles.linkTarget}>Target: {link.target}</div>

            <div style={styles.linkDetails}>
              <div style={styles.detailRow}>
                <span style={styles.checkIcon}>
                  {link.targetExists ? '✓' : '✗'}
                </span>
                <span>Target exists</span>
              </div>
              <div style={styles.detailRow}>
                <span style={styles.checkIcon}>
                  {link.isVisible ? '✓' : '✗'}
                </span>
                <span>Link is visible</span>
              </div>
              <div style={styles.detailRow}>
                <span style={styles.checkIcon}>
                  {link.isFirstFocusable ? '✓' : '✗'}
                </span>
                <span>First focusable element</span>
              </div>
            </div>

            {link.issues.length > 0 && (
              <div style={styles.issueList}>
                <strong>Issues:</strong>
                <ul style={styles.issues}>
                  {link.issues.map((issue, issueIndex) => (
                    <li key={issueIndex}>{issue}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ))}
      </div>

      {selectedLink && (
        <div style={styles.detailPanel}>
          <h4>Skip Link Details</h4>

          <div style={styles.detailSection}>
            <strong>Target Selector:</strong>
            <code style={styles.code}>{selectedLink.target}</code>
          </div>

          <div style={styles.detailSection}>
            <strong>Status:</strong>
            <div style={styles.statusGrid}>
              <div>
                <input
                  type="checkbox"
                  checked={selectedLink.targetExists}
                  readOnly
                  style={styles.checkbox}
                />
                <label>Target element exists</label>
              </div>
              <div>
                <input
                  type="checkbox"
                  checked={selectedLink.isVisible}
                  readOnly
                  style={styles.checkbox}
                />
                <label>Link is visible on focus</label>
              </div>
              <div>
                <input
                  type="checkbox"
                  checked={selectedLink.isFirstFocusable}
                  readOnly
                  style={styles.checkbox}
                />
                <label>First in tab order</label>
              </div>
              <div>
                <input
                  type="checkbox"
                  checked={selectedLink.worksCorrectly}
                  readOnly
                  style={styles.checkbox}
                />
                <label>Works correctly</label>
              </div>
            </div>
          </div>

          {selectedLink.issues.length > 0 && (
            <div style={styles.detailSection}>
              <strong>Issues:</strong>
              <ul style={styles.detailIssues}>
                {selectedLink.issues.map((issue, idx) => (
                  <li key={idx}>{issue}</li>
                ))}
              </ul>
            </div>
          )}

          <div style={styles.actions}>
            <button
              onClick={() => testSkipLink(selectedLink)}
              style={styles.actionButton}
            >
              Test This Link
            </button>
            <button
              onClick={() => {
                highlightElement(selectedLink.element, '#007bff');
              }}
              style={styles.actionButton}
            >
              Highlight Link
            </button>
          </div>
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
  warning: {
    padding: '16px',
    backgroundColor: '#fff3cd',
    border: '1px solid #ffc107',
    borderRadius: '6px',
    color: '#856404',
    fontSize: '14px',
    marginBottom: '20px',
  },
  linkList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
  },
  linkItem: {
    padding: '16px',
    borderRadius: '6px',
    border: '2px solid #dee2e6',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  linkValid: {
    borderColor: '#28a745',
    backgroundColor: '#f8fff8',
  },
  linkInvalid: {
    borderColor: '#dc3545',
    backgroundColor: '#fff5f5',
  },
  linkHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  statusBadge: {
    padding: '4px 12px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: 700,
  },
  statusValid: {
    backgroundColor: '#28a745',
    color: 'white',
  },
  statusInvalid: {
    backgroundColor: '#dc3545',
    color: 'white',
  },
  testButton: {
    padding: '6px 12px',
    fontSize: '12px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  linkTarget: {
    fontFamily: 'monospace',
    fontSize: '13px',
    color: '#495057',
    marginBottom: '12px',
    padding: '8px',
    backgroundColor: '#f8f9fa',
    borderRadius: '4px',
  },
  linkDetails: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
  },
  detailRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '13px',
  },
  checkIcon: {
    width: '20px',
    fontWeight: 700,
  },
  issueList: {
    marginTop: '12px',
    padding: '12px',
    backgroundColor: '#fff',
    borderRadius: '4px',
    fontSize: '13px',
  },
  issues: {
    marginTop: '8px',
    marginBottom: 0,
    paddingLeft: '20px',
  },
  detailPanel: {
    marginTop: '20px',
    padding: '16px',
    backgroundColor: '#e9ecef',
    borderRadius: '6px',
  },
  detailSection: {
    marginBottom: '16px',
  },
  code: {
    display: 'block',
    marginTop: '8px',
    padding: '8px',
    backgroundColor: '#f8f9fa',
    borderRadius: '4px',
    fontFamily: 'monospace',
    fontSize: '13px',
  },
  statusGrid: {
    marginTop: '12px',
    display: 'grid',
    gridTemplateColumns: 'repeat(2, 1fr)',
    gap: '8px',
  },
  checkbox: {
    marginRight: '8px',
  },
  detailIssues: {
    marginTop: '8px',
    paddingLeft: '20px',
    fontSize: '13px',
  },
  actions: {
    display: 'flex',
    gap: '8px',
  },
  actionButton: {
    flex: 1,
    padding: '8px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '13px',
  },
};
