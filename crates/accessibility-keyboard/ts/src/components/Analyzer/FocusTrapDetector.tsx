/**
 * Focus Trap Detector Component
 * Detects and reports keyboard focus traps
 */

import React, { useState, useEffect } from 'react';
import { FocusTrap } from '../../types';
import { FocusTrapAnalyzer } from '../../analyzers/FocusTrapAnalyzer';

export interface FocusTrapDetectorProps {
  targetElement?: HTMLElement;
  onTrapsDetected?: (traps: FocusTrap[]) => void;
  autoDetect?: boolean;
}

export const FocusTrapDetector: React.FC<FocusTrapDetectorProps> = ({
  targetElement,
  onTrapsDetected,
  autoDetect = true,
}) => {
  const [traps, setTraps] = useState<FocusTrap[]>([]);
  const [isDetecting, setIsDetecting] = useState(false);
  const [selectedTrap, setSelectedTrap] = useState<FocusTrap | null>(null);

  const detectTraps = async () => {
    setIsDetecting(true);
    const analyzer = new FocusTrapAnalyzer();
    const target = targetElement || document.body;

    try {
      const detectedTraps = await analyzer.detectTraps(target);
      setTraps(detectedTraps);

      if (onTrapsDetected) {
        onTrapsDetected(detectedTraps);
      }
    } catch (error) {
      console.error('Focus trap detection failed:', error);
    } finally {
      setIsDetecting(false);
    }
  };

  useEffect(() => {
    if (autoDetect) {
      detectTraps();
    }
  }, [autoDetect, targetElement]);

  const highlightTrap = (trap: FocusTrap) => {
    if (!trap.trapElement) return;

    // Remove existing highlights
    document.querySelectorAll('.focus-trap-highlight').forEach((el) => el.remove());

    const rect = trap.trapElement.getBoundingClientRect();
    const highlight = document.createElement('div');
    highlight.className = 'focus-trap-highlight';
    highlight.style.cssText = `
      position: fixed;
      top: ${rect.top - 5}px;
      left: ${rect.left - 5}px;
      width: ${rect.width + 10}px;
      height: ${rect.height + 10}px;
      border: 3px dashed #dc3545;
      background-color: rgba(220, 53, 69, 0.1);
      pointer-events: none;
      z-index: 999999;
      animation: pulse 2s ease-in-out infinite;
    `;

    document.body.appendChild(highlight);

    // Scroll into view
    trap.trapElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
  };

  const activeTrap s = traps.filter((t) => t.detected);
  const criticalTraps = activeTraps.filter((t) => t.severity === 'critical');

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Focus Trap Detector</h3>
        <button onClick={detectTraps} disabled={isDetecting} style={styles.button}>
          {isDetecting ? 'Detecting...' : 'Scan for Traps'}
        </button>
      </div>

      {activeTraps.length > 0 && (
        <div style={styles.alert}>
          ⚠️ {activeTraps.length} focus trap{activeTraps.length > 1 ? 's' : ''}{' '}
          detected! ({criticalTraps.length} critical)
        </div>
      )}

      <div style={styles.summary}>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Total Scanned</div>
          <div style={styles.summaryValue}>{traps.length}</div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Active Traps</div>
          <div style={{ ...styles.summaryValue, color: '#dc3545' }}>
            {activeTraps.length}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Critical</div>
          <div style={{ ...styles.summaryValue, color: '#dc3545', fontWeight: 700 }}>
            {criticalTraps.length}
          </div>
        </div>
      </div>

      <div style={styles.trapList}>
        {activeTraps.map((trap, index) => (
          <div
            key={index}
            style={{
              ...styles.trapItem,
              ...(trap.severity === 'critical' ? styles.trapCritical : {}),
            }}
            onClick={() => {
              setSelectedTrap(trap);
              highlightTrap(trap);
            }}
          >
            <div style={styles.trapHeader}>
              <span
                style={{
                  ...styles.severityBadge,
                  ...(trap.severity === 'critical'
                    ? styles.severityCritical
                    : trap.severity === 'major'
                    ? styles.severityMajor
                    : styles.severityMinor),
                }}
              >
                {trap.severity.toUpperCase()}
              </span>
              <span style={styles.trapElement}>
                {trap.trapElement?.tagName.toLowerCase() || 'Unknown'}
              </span>
            </div>

            <div style={styles.trapDescription}>{trap.description}</div>

            <div style={styles.trapDetails}>
              <div style={styles.detailRow}>
                <span style={styles.detailLabel}>Escape Method:</span>
                <span
                  style={{
                    ...styles.detailValue,
                    color: trap.escapeMethod === 'none' ? '#dc3545' : '#28a745',
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
              <div style={styles.warning}>
                🚨 Users cannot escape this trap using keyboard alone!
              </div>
            )}
          </div>
        ))}

        {activeTraps.length === 0 && !isDetecting && (
          <div style={styles.noTraps}>
            ✅ No focus traps detected. Keyboard navigation is free!
          </div>
        )}
      </div>

      {selectedTrap && (
        <div style={styles.detailPanel}>
          <h4>Focus Trap Details</h4>

          <div style={styles.detailSection}>
            <strong>Severity:</strong>{' '}
            <span
              style={{
                color:
                  selectedTrap.severity === 'critical'
                    ? '#dc3545'
                    : selectedTrap.severity === 'major'
                    ? '#fd7e14'
                    : '#ffc107',
              }}
            >
              {selectedTrap.severity.toUpperCase()}
            </span>
          </div>

          <div style={styles.detailSection}>
            <strong>Description:</strong>
            <p>{selectedTrap.description}</p>
          </div>

          <div style={styles.detailSection}>
            <strong>Escape Method:</strong> {selectedTrap.escapeMethod}
          </div>

          <div style={styles.detailSection}>
            <strong>Can User Escape:</strong>{' '}
            <span
              style={{
                color: selectedTrap.canEscape ? '#28a745' : '#dc3545',
                fontWeight: 600,
              }}
            >
              {selectedTrap.canEscape ? 'Yes' : 'No'}
            </span>
          </div>

          {selectedTrap.affectedElements.length > 0 && (
            <div style={styles.detailSection}>
              <strong>Affected Elements:</strong>
              <ul style={styles.elementList}>
                {selectedTrap.affectedElements.slice(0, 5).map((el, idx) => (
                  <li key={idx}>
                    <code>{el.tagName.toLowerCase()}</code>
                  </li>
                ))}
                {selectedTrap.affectedElements.length > 5 && (
                  <li>... and {selectedTrap.affectedElements.length - 5} more</li>
                )}
              </ul>
            </div>
          )}

          <div style={styles.recommendation}>
            <strong>Recommendation:</strong>
            <p>
              {!selectedTrap.canEscape
                ? 'Add keyboard-accessible close button or ESC key handler to allow users to exit this trap.'
                : 'Consider improving the escape method for better user experience.'}
            </p>
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
  alert: {
    padding: '12px',
    marginBottom: '16px',
    backgroundColor: '#fff3cd',
    border: '1px solid #ffc107',
    borderRadius: '6px',
    color: '#856404',
    fontWeight: 500,
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
  trapList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
  },
  trapItem: {
    padding: '16px',
    backgroundColor: '#fff5f5',
    border: '2px solid #f8d7da',
    borderRadius: '8px',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
  },
  trapCritical: {
    borderColor: '#dc3545',
    backgroundColor: '#ffe5e7',
  },
  trapHeader: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    marginBottom: '12px',
  },
  severityBadge: {
    padding: '4px 10px',
    borderRadius: '4px',
    fontSize: '11px',
    fontWeight: 700,
  },
  severityCritical: {
    backgroundColor: '#dc3545',
    color: 'white',
  },
  severityMajor: {
    backgroundColor: '#fd7e14',
    color: 'white',
  },
  severityMinor: {
    backgroundColor: '#ffc107',
    color: '#000',
  },
  trapElement: {
    fontFamily: 'monospace',
    fontSize: '14px',
    fontWeight: 600,
  },
  trapDescription: {
    marginBottom: '12px',
    fontSize: '14px',
    lineHeight: 1.5,
    color: '#495057',
  },
  trapDetails: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '6px',
  },
  detailRow: {
    display: 'flex',
    fontSize: '13px',
  },
  detailLabel: {
    fontWeight: 500,
    marginRight: '8px',
    color: '#6c757d',
    minWidth: '140px',
  },
  detailValue: {
    color: '#495057',
  },
  warning: {
    marginTop: '12px',
    padding: '10px',
    backgroundColor: '#dc3545',
    color: 'white',
    borderRadius: '4px',
    fontSize: '13px',
    fontWeight: 600,
  },
  noTraps: {
    padding: '32px',
    textAlign: 'center' as const,
    fontSize: '16px',
    color: '#28a745',
    fontWeight: 500,
    backgroundColor: '#d4edda',
    borderRadius: '8px',
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
  elementList: {
    marginTop: '8px',
    paddingLeft: '20px',
    fontSize: '13px',
  },
  recommendation: {
    marginTop: '16px',
    padding: '12px',
    backgroundColor: '#d1ecf1',
    border: '1px solid #bee5eb',
    borderRadius: '4px',
    fontSize: '13px',
  },
};
