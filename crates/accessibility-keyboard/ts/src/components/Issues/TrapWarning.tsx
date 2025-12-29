/**
 * Trap Warning Component
 * Displays warnings and alerts for detected focus traps
 */

import React from 'react';
import { FocusTrap } from '../../types';

export interface TrapWarningProps {
  traps: FocusTrap[];
  onDismiss?: (trap: FocusTrap) => void;
  onFix?: (trap: FocusTrap) => void;
  autoShow?: boolean;
}

export const TrapWarning: React.FC<TrapWarningProps> = ({
  traps,
  onDismiss,
  onFix,
  autoShow = true,
}) => {
  const activeTraps = traps.filter((t) => t.detected);
  const criticalTraps = activeTraps.filter((t) => t.severity === 'critical');

  if (!autoShow || activeTraps.length === 0) {
    return null;
  }

  const getSeverityIcon = (severity: string): string => {
    switch (severity) {
      case 'critical':
        return '🚨';
      case 'major':
        return '⚠️';
      case 'minor':
        return 'ℹ️';
      default:
        return '❗';
    }
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity) {
      case 'critical':
        return '#dc3545';
      case 'major':
        return '#fd7e14';
      case 'minor':
        return '#ffc107';
      default:
        return '#6c757d';
    }
  };

  const handleLocate = (trap: FocusTrap) => {
    if (trap.trapElement) {
      trap.trapElement.scrollIntoView({ behavior: 'smooth', block: 'center' });

      const rect = trap.trapElement.getBoundingClientRect();
      const highlight = document.createElement('div');
      highlight.style.cssText = `
        position: fixed;
        top: ${rect.top - 5}px;
        left: ${rect.left - 5}px;
        width: ${rect.width + 10}px;
        height: ${rect.height + 10}px;
        border: 3px dashed ${getSeverityColor(trap.severity)};
        background-color: ${getSeverityColor(trap.severity)}33;
        pointer-events: none;
        z-index: 999999;
        animation: pulse 2s ease-in-out 3;
      `;
      document.body.appendChild(highlight);

      setTimeout(() => highlight.remove(), 6000);
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <div style={styles.headerContent}>
          <span style={styles.icon}>🚨</span>
          <div>
            <h3 style={styles.title}>Focus Trap Warning</h3>
            <p style={styles.subtitle}>
              {activeTraps.length} focus trap{activeTraps.length > 1 ? 's' : ''} detected
              {criticalTraps.length > 0 && (
                <span style={styles.critical}>
                  {' '}
                  ({criticalTraps.length} critical)
                </span>
              )}
            </p>
          </div>
        </div>
      </div>

      <div style={styles.trapList}>
        {activeTraps.map((trap, index) => (
          <div
            key={index}
            style={{
              ...styles.trapCard,
              borderLeftColor: getSeverityColor(trap.severity),
            }}
          >
            <div style={styles.trapHeader}>
              <span style={styles.trapIcon}>{getSeverityIcon(trap.severity)}</span>
              <span
                style={{
                  ...styles.severityLabel,
                  color: getSeverityColor(trap.severity),
                }}
              >
                {trap.severity.toUpperCase()}
              </span>
            </div>

            <div style={styles.trapDescription}>{trap.description}</div>

            <div style={styles.trapDetails}>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Element:</span>
                <span style={styles.detailValue}>
                  {trap.trapElement?.tagName.toLowerCase() || 'Unknown'}
                </span>
              </div>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Escape Method:</span>
                <span
                  style={{
                    ...styles.detailValue,
                    color: trap.escapeMethod === 'none' ? '#dc3545' : '#28a745',
                    fontWeight: 600,
                  }}
                >
                  {trap.escapeMethod}
                </span>
              </div>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Can Escape:</span>
                <span
                  style={{
                    ...styles.detailValue,
                    color: trap.canEscape ? '#28a745' : '#dc3545',
                    fontWeight: 600,
                  }}
                >
                  {trap.canEscape ? 'Yes' : 'No'}
                </span>
              </div>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Affected Elements:</span>
                <span style={styles.detailValue}>{trap.affectedElements.length}</span>
              </div>
            </div>

            {!trap.canEscape && (
              <div style={styles.criticalWarning}>
                <strong>⚠️ Critical Issue:</strong> Users cannot escape this trap using
                keyboard alone. This violates WCAG 2.1.2 (No Keyboard Trap).
              </div>
            )}

            <div style={styles.actions}>
              <button
                onClick={() => handleLocate(trap)}
                style={styles.locateButton}
              >
                Locate Trap
              </button>
              {onFix && (
                <button onClick={() => onFix(trap)} style={styles.fixButton}>
                  Suggest Fix
                </button>
              )}
              {onDismiss && (
                <button onClick={() => onDismiss(trap)} style={styles.dismissButton}>
                  Dismiss
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      <div style={styles.footer}>
        <div style={styles.wcagInfo}>
          <strong>WCAG 2.1.2 (No Keyboard Trap):</strong> If keyboard focus can be moved
          to a component using a keyboard interface, then focus can be moved away from
          that component using only a keyboard interface.
        </div>

        <div style={styles.recommendations}>
          <strong>Recommendations:</strong>
          <ul style={styles.recommendationList}>
            <li>Ensure all modal dialogs have a keyboard-accessible close button</li>
            <li>Implement ESC key handler to close modal dialogs and overlays</li>
            <li>Provide clear instructions for exiting focus traps when necessary</li>
            <li>Test all interactive components with keyboard-only navigation</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

const styles = {
  container: {
    position: 'fixed' as const,
    bottom: '20px',
    right: '20px',
    maxWidth: '400px',
    backgroundColor: '#fff',
    borderRadius: '8px',
    boxShadow: '0 4px 20px rgba(0, 0, 0, 0.15)',
    zIndex: 1000000,
    border: '2px solid #dc3545',
    overflow: 'hidden',
    fontFamily: 'system-ui, -apple-system, sans-serif',
  },
  header: {
    backgroundColor: '#dc3545',
    color: 'white',
    padding: '16px',
  },
  headerContent: {
    display: 'flex',
    alignItems: 'flex-start',
    gap: '12px',
  },
  icon: {
    fontSize: '24px',
  },
  title: {
    margin: 0,
    fontSize: '16px',
    fontWeight: 600,
  },
  subtitle: {
    margin: '4px 0 0 0',
    fontSize: '13px',
    opacity: 0.95,
  },
  critical: {
    fontWeight: 700,
  },
  trapList: {
    maxHeight: '400px',
    overflowY: 'auto' as const,
    padding: '16px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
  },
  trapCard: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    borderLeft: '4px solid',
  },
  trapHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    marginBottom: '8px',
  },
  trapIcon: {
    fontSize: '18px',
  },
  severityLabel: {
    fontSize: '12px',
    fontWeight: 700,
  },
  trapDescription: {
    fontSize: '13px',
    color: '#495057',
    marginBottom: '12px',
    lineHeight: 1.4,
  },
  trapDetails: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '4px',
    marginBottom: '12px',
  },
  detailRow: {
    display: 'flex',
    fontSize: '12px',
  },
  detailLabel: {
    fontWeight: 500,
    marginRight: '8px',
    minWidth: '120px',
    color: '#6c757d',
  },
  detailValue: {
    fontFamily: 'monospace',
    color: '#495057',
  },
  criticalWarning: {
    padding: '10px',
    backgroundColor: '#fff3cd',
    border: '1px solid #ffc107',
    borderRadius: '4px',
    fontSize: '12px',
    marginBottom: '12px',
    lineHeight: 1.4,
  },
  actions: {
    display: 'flex',
    gap: '6px',
  },
  locateButton: {
    flex: 1,
    padding: '6px 12px',
    fontSize: '12px',
    fontWeight: 500,
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  fixButton: {
    flex: 1,
    padding: '6px 12px',
    fontSize: '12px',
    fontWeight: 500,
    backgroundColor: '#28a745',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  dismissButton: {
    flex: 1,
    padding: '6px 12px',
    fontSize: '12px',
    fontWeight: 500,
    backgroundColor: '#6c757d',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  footer: {
    padding: '16px',
    backgroundColor: '#e9ecef',
    borderTop: '1px solid #dee2e6',
  },
  wcagInfo: {
    fontSize: '12px',
    marginBottom: '12px',
    lineHeight: 1.4,
  },
  recommendations: {
    fontSize: '12px',
  },
  recommendationList: {
    marginTop: '8px',
    marginBottom: 0,
    paddingLeft: '20px',
    lineHeight: 1.6,
  },
};
