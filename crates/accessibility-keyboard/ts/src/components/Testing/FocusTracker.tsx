/**
 * Focus Tracker Component
 * Real-time focus tracking and visualization
 */

import React, { useState, useEffect } from 'react';
import { FocusEvent, FocusPath } from '../../types';
import { useFocusTracking } from '../../hooks/useFocusTracking';

export interface FocusTrackerProps {
  showVisualIndicator?: boolean;
  trackHistory?: boolean;
  maxHistorySize?: number;
}

export const FocusTracker: React.FC<FocusTrackerProps> = ({
  showVisualIndicator = true,
  trackHistory = true,
  maxHistorySize = 50,
}) => {
  const {
    currentFocus,
    focusHistory,
    isTracking,
    startTracking,
    stopTracking,
    clearHistory,
  } = useFocusTracking(maxHistorySize);

  const [focusPaths, setFocusPaths] = useState<FocusPath[]>([]);
  const [showDetails, setShowDetails] = useState(true);

  useEffect(() => {
    if (!isTracking) {
      startTracking();
    }
  }, []);

  useEffect(() => {
    // Analyze focus paths
    if (focusHistory.length >= 2) {
      const recentHistory = focusHistory.slice(-10);
      const path: FocusPath = {
        sequence: recentHistory.map((f) => f.element),
        startTime: recentHistory[0].timestamp,
        endTime: recentHistory[recentHistory.length - 1].timestamp,
        method: 'tab',
        deviations: 0,
      };
      setFocusPaths((prev) => [...prev.slice(-5), path]);
    }
  }, [focusHistory]);

  const getFocusStats = () => {
    const keyboardFocus = focusHistory.filter((f) => f.triggeredBy === 'keyboard').length;
    const mouseFocus = focusHistory.filter((f) => f.triggeredBy === 'mouse').length;
    const scriptFocus = focusHistory.filter((f) => f.triggeredBy === 'script').length;

    const uniqueElements = new Set(focusHistory.map((f) => f.element)).size;

    return {
      total: focusHistory.length,
      keyboard: keyboardFocus,
      mouse: mouseFocus,
      script: scriptFocus,
      unique: uniqueElements,
    };
  };

  const stats = getFocusStats();

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Focus Tracker</h3>
        <div style={styles.controls}>
          <button
            onClick={isTracking ? stopTracking : startTracking}
            style={{
              ...styles.button,
              backgroundColor: isTracking ? '#dc3545' : '#28a745',
            }}
          >
            {isTracking ? 'Stop' : 'Start'}
          </button>
          <button onClick={clearHistory} style={styles.button}>
            Clear
          </button>
          <button
            onClick={() => setShowDetails(!showDetails)}
            style={styles.button}
          >
            {showDetails ? 'Hide' : 'Show'} Details
          </button>
        </div>
      </div>

      {isTracking && (
        <div style={styles.status}>
          <span style={styles.statusIndicator}>🔴 Tracking Active</span>
        </div>
      )}

      {currentFocus && (
        <div style={styles.currentFocus}>
          <h4>Currently Focused</h4>
          <div style={styles.focusInfo}>
            <div style={styles.infoRow}>
              <span style={styles.infoLabel}>Element:</span>
              <span style={styles.infoValue}>
                {currentFocus.element.tagName.toLowerCase()}
              </span>
            </div>
            <div style={styles.infoRow}>
              <span style={styles.infoLabel}>Triggered By:</span>
              <span
                style={{
                  ...styles.triggerBadge,
                  backgroundColor:
                    currentFocus.triggeredBy === 'keyboard'
                      ? '#007bff'
                      : currentFocus.triggeredBy === 'mouse'
                      ? '#28a745'
                      : '#6c757d',
                }}
              >
                {currentFocus.triggeredBy}
              </span>
            </div>
            <div style={styles.infoRow}>
              <span style={styles.infoLabel}>Timestamp:</span>
              <span style={styles.infoValue}>
                {new Date(currentFocus.timestamp).toLocaleTimeString()}
              </span>
            </div>
            {currentFocus.previousElement && (
              <div style={styles.infoRow}>
                <span style={styles.infoLabel}>Previous:</span>
                <span style={styles.infoValue}>
                  {currentFocus.previousElement.tagName.toLowerCase()}
                </span>
              </div>
            )}
          </div>
        </div>
      )}

      <div style={styles.stats}>
        <div style={styles.statItem}>
          <div style={styles.statLabel}>Total Events</div>
          <div style={styles.statValue}>{stats.total}</div>
        </div>
        <div style={styles.statItem}>
          <div style={styles.statLabel}>Keyboard</div>
          <div style={{ ...styles.statValue, color: '#007bff' }}>
            {stats.keyboard}
          </div>
        </div>
        <div style={styles.statItem}>
          <div style={styles.statLabel}>Mouse</div>
          <div style={{ ...styles.statValue, color: '#28a745' }}>
            {stats.mouse}
          </div>
        </div>
        <div style={styles.statItem}>
          <div style={styles.statLabel}>Script</div>
          <div style={{ ...styles.statValue, color: '#6c757d' }}>
            {stats.script}
          </div>
        </div>
        <div style={styles.statItem}>
          <div style={styles.statLabel}>Unique Elements</div>
          <div style={styles.statValue}>{stats.unique}</div>
        </div>
      </div>

      {showDetails && (
        <>
          <div style={styles.history}>
            <h4>Focus History</h4>
            <div style={styles.historyList}>
              {focusHistory
                .slice()
                .reverse()
                .slice(0, 20)
                .map((event, index) => (
                  <div key={index} style={styles.historyItem}>
                    <span
                      style={{
                        ...styles.eventType,
                        color: event.type === 'focus' ? '#28a745' : '#dc3545',
                      }}
                    >
                      {event.type}
                    </span>
                    <span style={styles.eventElement}>
                      {event.element.tagName.toLowerCase()}
                    </span>
                    <span
                      style={{
                        ...styles.triggerBadge,
                        backgroundColor:
                          event.triggeredBy === 'keyboard'
                            ? '#007bff'
                            : event.triggeredBy === 'mouse'
                            ? '#28a745'
                            : '#6c757d',
                      }}
                    >
                      {event.triggeredBy}
                    </span>
                    <span style={styles.eventTime}>
                      {new Date(event.timestamp).toLocaleTimeString()}
                    </span>
                  </div>
                ))}
            </div>
            {focusHistory.length > 20 && (
              <div style={styles.moreItems}>
                Showing last 20 of {focusHistory.length} events
              </div>
            )}
          </div>

          {focusPaths.length > 0 && (
            <div style={styles.paths}>
              <h4>Focus Paths</h4>
              <div style={styles.pathList}>
                {focusPaths.map((path, index) => (
                  <div key={index} style={styles.pathItem}>
                    <div style={styles.pathHeader}>
                      <span style={styles.pathMethod}>{path.method}</span>
                      <span style={styles.pathDuration}>
                        {((path.endTime - path.startTime) / 1000).toFixed(2)}s
                      </span>
                    </div>
                    <div style={styles.pathSequence}>
                      {path.sequence.map((el, idx) => (
                        <React.Fragment key={idx}>
                          <span style={styles.pathElement}>
                            {el.tagName.toLowerCase()}
                          </span>
                          {idx < path.sequence.length - 1 && (
                            <span style={styles.pathArrow}>→</span>
                          )}
                        </React.Fragment>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </>
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
  controls: {
    display: 'flex',
    gap: '8px',
  },
  button: {
    padding: '6px 12px',
    fontSize: '13px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  status: {
    padding: '12px',
    backgroundColor: '#fff3cd',
    borderRadius: '6px',
    marginBottom: '16px',
  },
  statusIndicator: {
    fontSize: '14px',
    fontWeight: 500,
  },
  currentFocus: {
    marginBottom: '20px',
    padding: '16px',
    backgroundColor: '#e7f3ff',
    borderRadius: '6px',
  },
  focusInfo: {
    marginTop: '12px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
  },
  infoRow: {
    display: 'flex',
    alignItems: 'center',
    fontSize: '14px',
  },
  infoLabel: {
    fontWeight: 500,
    marginRight: '12px',
    minWidth: '120px',
    color: '#6c757d',
  },
  infoValue: {
    fontFamily: 'monospace',
    color: '#495057',
  },
  triggerBadge: {
    padding: '2px 8px',
    borderRadius: '3px',
    color: 'white',
    fontSize: '12px',
    fontWeight: 600,
  },
  stats: {
    display: 'grid',
    gridTemplateColumns: 'repeat(5, 1fr)',
    gap: '12px',
    marginBottom: '20px',
  },
  statItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    textAlign: 'center' as const,
  },
  statLabel: {
    fontSize: '11px',
    color: '#6c757d',
    marginBottom: '4px',
  },
  statValue: {
    fontSize: '18px',
    fontWeight: 600,
  },
  history: {
    marginBottom: '20px',
  },
  historyList: {
    marginTop: '12px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    maxHeight: '300px',
    overflowY: 'auto' as const,
  },
  historyItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '8px 12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '4px',
    fontSize: '13px',
  },
  eventType: {
    fontWeight: 600,
    minWidth: '50px',
  },
  eventElement: {
    fontFamily: 'monospace',
    color: '#495057',
    flex: 1,
  },
  eventTime: {
    color: '#6c757d',
    fontSize: '12px',
  },
  moreItems: {
    marginTop: '8px',
    fontSize: '12px',
    color: '#6c757d',
    textAlign: 'center' as const,
  },
  paths: {
    marginBottom: '20px',
  },
  pathList: {
    marginTop: '12px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
  },
  pathItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
  },
  pathHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: '8px',
  },
  pathMethod: {
    padding: '3px 8px',
    backgroundColor: '#007bff',
    color: 'white',
    borderRadius: '3px',
    fontSize: '12px',
    fontWeight: 600,
  },
  pathDuration: {
    fontSize: '12px',
    color: '#6c757d',
  },
  pathSequence: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    flexWrap: 'wrap' as const,
  },
  pathElement: {
    padding: '4px 8px',
    backgroundColor: '#e9ecef',
    borderRadius: '3px',
    fontFamily: 'monospace',
    fontSize: '12px',
  },
  pathArrow: {
    color: '#6c757d',
    fontSize: '14px',
  },
};
