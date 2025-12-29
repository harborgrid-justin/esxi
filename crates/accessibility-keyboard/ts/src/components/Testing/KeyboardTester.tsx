/**
 * Keyboard Tester Component
 * Interactive keyboard accessibility testing interface
 */

import React, { useState, useEffect, useCallback } from 'react';
import { KeyboardEventInfo, FocusEvent } from '../../types';
import { useFocusTracking } from '../../hooks/useFocusTracking';

export interface KeyboardTesterProps {
  onTestComplete?: (results: TestResults) => void;
}

interface TestResults {
  totalKeyPresses: number;
  tabPresses: number;
  enterPresses: number;
  spacePresses: number;
  escapePresses: number;
  arrowPresses: number;
  focusChanges: number;
  accessibleElements: number;
  inaccessibleElements: number;
  duration: number;
}

export const KeyboardTester: React.FC<KeyboardTesterProps> = ({ onTestComplete }) => {
  const [isRecording, setIsRecording] = useState(false);
  const [keyEvents, setKeyEvents] = useState<KeyboardEventInfo[]>([]);
  const { focusHistory, startTracking, stopTracking, clearHistory } = useFocusTracking();
  const [startTime, setStartTime] = useState<number>(0);
  const [testResults, setTestResults] = useState<TestResults | null>(null);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!isRecording) return;

      const keyInfo: KeyboardEventInfo = {
        key: event.key,
        code: event.code,
        keyCode: event.keyCode,
        ctrlKey: event.ctrlKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        metaKey: event.metaKey,
        target: event.target as HTMLElement,
        timestamp: Date.now(),
      };

      setKeyEvents((prev) => [...prev, keyInfo]);
    },
    [isRecording]
  );

  useEffect(() => {
    if (isRecording) {
      document.addEventListener('keydown', handleKeyDown);
      return () => {
        document.removeEventListener('keydown', handleKeyDown);
      };
    }
  }, [isRecording, handleKeyDown]);

  const startTest = () => {
    setIsRecording(true);
    setKeyEvents([]);
    clearHistory();
    startTracking();
    setStartTime(Date.now());
    setTestResults(null);
  };

  const stopTest = () => {
    setIsRecording(false);
    stopTracking();

    const duration = (Date.now() - startTime) / 1000;

    const results: TestResults = {
      totalKeyPresses: keyEvents.length,
      tabPresses: keyEvents.filter((e) => e.key === 'Tab').length,
      enterPresses: keyEvents.filter((e) => e.key === 'Enter').length,
      spacePresses: keyEvents.filter((e) => e.key === ' ').length,
      escapePresses: keyEvents.filter((e) => e.key === 'Escape').length,
      arrowPresses: keyEvents.filter((e) =>
        ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)
      ).length,
      focusChanges: focusHistory.length,
      accessibleElements: new Set(
        focusHistory.map((f) => f.element.tagName)
      ).size,
      inaccessibleElements: 0,
      duration,
    };

    setTestResults(results);

    if (onTestComplete) {
      onTestComplete(results);
    }
  };

  const getKeyColor = (key: string): string => {
    if (key === 'Tab') return '#007bff';
    if (key === 'Enter') return '#28a745';
    if (key === ' ') return '#17a2b8';
    if (key === 'Escape') return '#ffc107';
    if (['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(key)) return '#6f42c1';
    return '#6c757d';
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Keyboard Tester</h3>
        <div style={styles.controls}>
          {!isRecording ? (
            <button onClick={startTest} style={styles.buttonStart}>
              Start Test
            </button>
          ) : (
            <button onClick={stopTest} style={styles.buttonStop}>
              Stop Test
            </button>
          )}
        </div>
      </div>

      {isRecording && (
        <div style={styles.recording}>
          <div style={styles.recordingIndicator}>
            <span style={styles.recordingDot} />
            Recording keyboard input...
          </div>
          <div style={styles.stats}>
            <span>Key Presses: {keyEvents.length}</span>
            <span>Focus Changes: {focusHistory.length}</span>
            <span>Duration: {((Date.now() - startTime) / 1000).toFixed(1)}s</span>
          </div>
        </div>
      )}

      {testResults && (
        <div style={styles.results}>
          <h4>Test Results</h4>
          <div style={styles.resultsGrid}>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Total Key Presses</div>
              <div style={styles.resultValue}>{testResults.totalKeyPresses}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Tab</div>
              <div style={styles.resultValue}>{testResults.tabPresses}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Enter</div>
              <div style={styles.resultValue}>{testResults.enterPresses}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Space</div>
              <div style={styles.resultValue}>{testResults.spacePresses}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Escape</div>
              <div style={styles.resultValue}>{testResults.escapePresses}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Arrow Keys</div>
              <div style={styles.resultValue}>{testResults.arrowPresses}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Focus Changes</div>
              <div style={styles.resultValue}>{testResults.focusChanges}</div>
            </div>
            <div style={styles.resultItem}>
              <div style={styles.resultLabel}>Duration</div>
              <div style={styles.resultValue}>{testResults.duration.toFixed(1)}s</div>
            </div>
          </div>
        </div>
      )}

      {keyEvents.length > 0 && (
        <div style={styles.eventLog}>
          <h4>Key Event Log</h4>
          <div style={styles.eventList}>
            {keyEvents.slice(-20).reverse().map((event, index) => (
              <div key={index} style={styles.eventItem}>
                <span
                  style={{
                    ...styles.keyBadge,
                    backgroundColor: getKeyColor(event.key),
                  }}
                >
                  {event.key}
                </span>
                <span style={styles.eventTarget}>
                  {event.target.tagName.toLowerCase()}
                </span>
                <span style={styles.eventModifiers}>
                  {event.ctrlKey && <span style={styles.modifier}>Ctrl</span>}
                  {event.altKey && <span style={styles.modifier}>Alt</span>}
                  {event.shiftKey && <span style={styles.modifier}>Shift</span>}
                  {event.metaKey && <span style={styles.modifier}>Meta</span>}
                </span>
                <span style={styles.eventTime}>
                  {new Date(event.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
          {keyEvents.length > 20 && (
            <div style={styles.moreEvents}>
              Showing last 20 of {keyEvents.length} events
            </div>
          )}
        </div>
      )}

      {focusHistory.length > 0 && (
        <div style={styles.focusLog}>
          <h4>Focus History</h4>
          <div style={styles.focusList}>
            {focusHistory.slice(-10).reverse().map((focusEvent, index) => (
              <div key={index} style={styles.focusItem}>
                <span
                  style={{
                    ...styles.focusType,
                    color: focusEvent.type === 'focus' ? '#28a745' : '#6c757d',
                  }}
                >
                  {focusEvent.type}
                </span>
                <span style={styles.focusElement}>
                  {focusEvent.element.tagName.toLowerCase()}
                </span>
                <span style={styles.focusTrigger}>{focusEvent.triggeredBy}</span>
                <span style={styles.focusTime}>
                  {new Date(focusEvent.timestamp).toLocaleTimeString()}
                </span>
              </div>
            ))}
          </div>
          {focusHistory.length > 10 && (
            <div style={styles.moreEvents}>
              Showing last 10 of {focusHistory.length} focus events
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
  controls: {
    display: 'flex',
    gap: '8px',
  },
  buttonStart: {
    padding: '8px 20px',
    fontSize: '14px',
    fontWeight: 500,
    backgroundColor: '#28a745',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  buttonStop: {
    padding: '8px 20px',
    fontSize: '14px',
    fontWeight: 500,
    backgroundColor: '#dc3545',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  recording: {
    padding: '16px',
    backgroundColor: '#fff3cd',
    border: '2px solid #ffc107',
    borderRadius: '6px',
    marginBottom: '16px',
  },
  recordingIndicator: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '14px',
    fontWeight: 600,
    marginBottom: '12px',
  },
  recordingDot: {
    width: '12px',
    height: '12px',
    backgroundColor: '#dc3545',
    borderRadius: '50%',
    animation: 'pulse 1.5s ease-in-out infinite',
  },
  stats: {
    display: 'flex',
    gap: '20px',
    fontSize: '13px',
  },
  results: {
    marginBottom: '20px',
  },
  resultsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '12px',
    marginTop: '12px',
  },
  resultItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    textAlign: 'center' as const,
  },
  resultLabel: {
    fontSize: '12px',
    color: '#6c757d',
    marginBottom: '4px',
  },
  resultValue: {
    fontSize: '20px',
    fontWeight: 600,
  },
  eventLog: {
    marginBottom: '20px',
  },
  eventList: {
    marginTop: '12px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    maxHeight: '300px',
    overflowY: 'auto' as const,
  },
  eventItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '8px 12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '4px',
    fontSize: '13px',
  },
  keyBadge: {
    padding: '4px 10px',
    borderRadius: '4px',
    color: 'white',
    fontWeight: 600,
    fontFamily: 'monospace',
    minWidth: '60px',
    textAlign: 'center' as const,
  },
  eventTarget: {
    fontFamily: 'monospace',
    color: '#495057',
    flex: 1,
  },
  eventModifiers: {
    display: 'flex',
    gap: '4px',
  },
  modifier: {
    padding: '2px 6px',
    backgroundColor: '#6c757d',
    color: 'white',
    borderRadius: '3px',
    fontSize: '11px',
  },
  eventTime: {
    color: '#6c757d',
    fontSize: '12px',
  },
  focusLog: {
    marginBottom: '20px',
  },
  focusList: {
    marginTop: '12px',
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    maxHeight: '250px',
    overflowY: 'auto' as const,
  },
  focusItem: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '8px 12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '4px',
    fontSize: '13px',
  },
  focusType: {
    fontWeight: 600,
    minWidth: '60px',
  },
  focusElement: {
    fontFamily: 'monospace',
    color: '#495057',
    flex: 1,
  },
  focusTrigger: {
    padding: '2px 8px',
    backgroundColor: '#e9ecef',
    borderRadius: '3px',
    fontSize: '12px',
  },
  focusTime: {
    color: '#6c757d',
    fontSize: '12px',
  },
  moreEvents: {
    marginTop: '8px',
    fontSize: '12px',
    color: '#6c757d',
    textAlign: 'center' as const,
  },
};
