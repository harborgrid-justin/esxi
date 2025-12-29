/**
 * Shortcut Tester Component
 * Tests and validates keyboard shortcuts for conflicts and usability
 */

import React, { useState, useEffect, useCallback } from 'react';
import { KeyboardShortcut } from '../../types';
import { ShortcutAnalyzer } from '../../analyzers/ShortcutAnalyzer';

export interface ShortcutTesterProps {
  targetElement?: HTMLElement;
  onShortcutsAnalyzed?: (shortcuts: KeyboardShortcut[]) => void;
}

export const ShortcutTester: React.FC<ShortcutTesterProps> = ({
  targetElement,
  onShortcutsAnalyzed,
}) => {
  const [shortcuts, setShortcuts] = useState<KeyboardShortcut[]>([]);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [testMode, setTestMode] = useState(false);
  const [lastPressed, setLastPressed] = useState<string>('');
  const [matchedShortcut, setMatchedShortcut] = useState<KeyboardShortcut | null>(null);

  const analyzeShortcuts = async () => {
    setIsAnalyzing(true);
    const analyzer = new ShortcutAnalyzer();
    const target = targetElement || document.body;

    try {
      const detected = await analyzer.analyze(target);
      setShortcuts(detected);

      if (onShortcutsAnalyzed) {
        onShortcutsAnalyzed(detected);
      }
    } catch (error) {
      console.error('Shortcut analysis failed:', error);
    } finally {
      setIsAnalyzing(false);
    }
  };

  const handleTestKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (!testMode) return;

      const keyCombo = formatKeyCombo({
        key: event.key,
        ctrl: event.ctrlKey,
        alt: event.altKey,
        shift: event.shiftKey,
        meta: event.metaKey,
      });

      setLastPressed(keyCombo);

      // Check if this matches any registered shortcut
      const match = shortcuts.find((s) => {
        const sCombo = formatKeyCombo({
          key: s.key,
          ctrl: s.modifiers.ctrl || false,
          alt: s.modifiers.alt || false,
          shift: s.modifiers.shift || false,
          meta: s.modifiers.meta || false,
        });
        return sCombo === keyCombo;
      });

      setMatchedShortcut(match || null);
    },
    [testMode, shortcuts]
  );

  useEffect(() => {
    analyzeShortcuts();
  }, [targetElement]);

  useEffect(() => {
    if (testMode) {
      document.addEventListener('keydown', handleTestKeyPress);
      return () => {
        document.removeEventListener('keydown', handleTestKeyPress);
      };
    }
  }, [testMode, handleTestKeyPress]);

  const formatKeyCombo = ({
    key,
    ctrl,
    alt,
    shift,
    meta,
  }: {
    key: string;
    ctrl: boolean;
    alt: boolean;
    shift: boolean;
    meta: boolean;
  }): string => {
    const parts: string[] = [];
    if (ctrl) parts.push('Ctrl');
    if (alt) parts.push('Alt');
    if (shift) parts.push('Shift');
    if (meta) parts.push('Meta');
    parts.push(key);
    return parts.join('+');
  };

  const conflictingShortcuts = shortcuts.filter((s) => s.conflicts.length > 0);
  const browserConflicts = shortcuts.filter((s) => s.isBrowserConflict);
  const undocumented = shortcuts.filter((s) => !s.isDocumented);

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Keyboard Shortcut Tester</h3>
        <div style={styles.controls}>
          <button
            onClick={analyzeShortcuts}
            disabled={isAnalyzing}
            style={styles.button}
          >
            {isAnalyzing ? 'Analyzing...' : 'Re-analyze'}
          </button>
          <button
            onClick={() => setTestMode(!testMode)}
            style={{
              ...styles.button,
              ...(testMode ? styles.buttonActive : {}),
            }}
          >
            {testMode ? 'Stop Testing' : 'Test Mode'}
          </button>
        </div>
      </div>

      {testMode && (
        <div style={styles.testPanel}>
          <div style={styles.testHeader}>
            🎹 Press any key combination to test
          </div>
          {lastPressed && (
            <div style={styles.lastPressed}>
              <strong>Last Pressed:</strong> {lastPressed}
            </div>
          )}
          {matchedShortcut ? (
            <div style={styles.matchFound}>
              ✅ Matched: {matchedShortcut.action}
              {matchedShortcut.conflicts.length > 0 && (
                <div style={styles.conflictWarning}>
                  ⚠️ This shortcut has {matchedShortcut.conflicts.length} conflict(s)
                </div>
              )}
            </div>
          ) : lastPressed ? (
            <div style={styles.noMatch}>❌ No registered shortcut found</div>
          ) : null}
        </div>
      )}

      <div style={styles.summary}>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Total Shortcuts</div>
          <div style={styles.summaryValue}>{shortcuts.length}</div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Conflicts</div>
          <div style={{ ...styles.summaryValue, color: '#dc3545' }}>
            {conflictingShortcuts.length}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Browser Conflicts</div>
          <div style={{ ...styles.summaryValue, color: '#fd7e14' }}>
            {browserConflicts.length}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Undocumented</div>
          <div style={{ ...styles.summaryValue, color: '#ffc107' }}>
            {undocumented.length}
          </div>
        </div>
      </div>

      <div style={styles.shortcutList}>
        {shortcuts.map((shortcut, index) => {
          const keyCombo = formatKeyCombo({
            key: shortcut.key,
            ctrl: shortcut.modifiers.ctrl || false,
            alt: shortcut.modifiers.alt || false,
            shift: shortcut.modifiers.shift || false,
            meta: shortcut.modifiers.meta || false,
          });

          const hasIssues =
            shortcut.conflicts.length > 0 ||
            shortcut.isBrowserConflict ||
            !shortcut.isDocumented;

          return (
            <div
              key={index}
              style={{
                ...styles.shortcutItem,
                ...(hasIssues ? styles.shortcutWithIssues : {}),
              }}
            >
              <div style={styles.shortcutHeader}>
                <span style={styles.keyCombo}>{keyCombo}</span>
                <span style={styles.action}>{shortcut.action}</span>
              </div>

              <div style={styles.shortcutDetails}>
                {shortcut.element && (
                  <div style={styles.detailRow}>
                    <span style={styles.detailLabel}>Element:</span>
                    <span style={styles.detailValue}>
                      {shortcut.element.tagName.toLowerCase()}
                    </span>
                  </div>
                )}

                <div style={styles.badges}>
                  {shortcut.isBrowserConflict && (
                    <span style={styles.badgeBrowser}>Browser Conflict</span>
                  )}
                  {!shortcut.isDocumented && (
                    <span style={styles.badgeUndocumented}>Undocumented</span>
                  )}
                  {shortcut.conflicts.length > 0 && (
                    <span style={styles.badgeConflict}>
                      {shortcut.conflicts.length} Conflict(s)
                    </span>
                  )}
                </div>
              </div>

              {shortcut.conflicts.length > 0 && (
                <div style={styles.conflictList}>
                  <strong>Conflicts with:</strong>
                  <ul style={styles.conflicts}>
                    {shortcut.conflicts.map((conflict, cIdx) => (
                      <li key={cIdx}>
                        {formatKeyCombo({
                          key: conflict.key,
                          ctrl: conflict.modifiers.ctrl || false,
                          alt: conflict.modifiers.alt || false,
                          shift: conflict.modifiers.shift || false,
                          meta: conflict.modifiers.meta || false,
                        })}{' '}
                        - {conflict.action}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          );
        })}

        {shortcuts.length === 0 && !isAnalyzing && (
          <div style={styles.noShortcuts}>
            No keyboard shortcuts detected in the current scope
          </div>
        )}
      </div>
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
    padding: '8px 16px',
    fontSize: '14px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
  },
  buttonActive: {
    backgroundColor: '#28a745',
  },
  testPanel: {
    padding: '16px',
    backgroundColor: '#e7f3ff',
    border: '2px solid #007bff',
    borderRadius: '6px',
    marginBottom: '16px',
  },
  testHeader: {
    fontSize: '16px',
    fontWeight: 600,
    marginBottom: '12px',
  },
  lastPressed: {
    padding: '12px',
    backgroundColor: '#fff',
    borderRadius: '4px',
    marginBottom: '8px',
    fontSize: '14px',
  },
  matchFound: {
    padding: '12px',
    backgroundColor: '#d4edda',
    border: '1px solid #c3e6cb',
    borderRadius: '4px',
    color: '#155724',
    fontSize: '14px',
  },
  noMatch: {
    padding: '12px',
    backgroundColor: '#f8d7da',
    border: '1px solid #f5c6cb',
    borderRadius: '4px',
    color: '#721c24',
    fontSize: '14px',
  },
  conflictWarning: {
    marginTop: '8px',
    fontSize: '13px',
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
  shortcutList: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
    maxHeight: '500px',
    overflowY: 'auto' as const,
  },
  shortcutItem: {
    padding: '12px',
    backgroundColor: '#f8f9fa',
    borderRadius: '6px',
    border: '1px solid #dee2e6',
  },
  shortcutWithIssues: {
    borderColor: '#ffc107',
    backgroundColor: '#fff3cd',
  },
  shortcutHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '8px',
  },
  keyCombo: {
    padding: '4px 12px',
    backgroundColor: '#495057',
    color: 'white',
    borderRadius: '4px',
    fontFamily: 'monospace',
    fontSize: '13px',
    fontWeight: 600,
  },
  action: {
    fontSize: '14px',
    color: '#495057',
    flex: 1,
    marginLeft: '12px',
  },
  shortcutDetails: {
    marginBottom: '8px',
  },
  detailRow: {
    fontSize: '13px',
    marginBottom: '4px',
  },
  detailLabel: {
    fontWeight: 500,
    marginRight: '8px',
    color: '#6c757d',
  },
  detailValue: {
    fontFamily: 'monospace',
    color: '#495057',
  },
  badges: {
    display: 'flex',
    gap: '6px',
    flexWrap: 'wrap' as const,
    marginTop: '8px',
  },
  badgeBrowser: {
    padding: '3px 8px',
    backgroundColor: '#fd7e14',
    color: 'white',
    borderRadius: '3px',
    fontSize: '11px',
    fontWeight: 600,
  },
  badgeUndocumented: {
    padding: '3px 8px',
    backgroundColor: '#ffc107',
    color: '#000',
    borderRadius: '3px',
    fontSize: '11px',
    fontWeight: 600,
  },
  badgeConflict: {
    padding: '3px 8px',
    backgroundColor: '#dc3545',
    color: 'white',
    borderRadius: '3px',
    fontSize: '11px',
    fontWeight: 600,
  },
  conflictList: {
    marginTop: '12px',
    padding: '12px',
    backgroundColor: '#fff',
    borderRadius: '4px',
    fontSize: '13px',
  },
  conflicts: {
    marginTop: '8px',
    marginBottom: 0,
    paddingLeft: '20px',
  },
  noShortcuts: {
    padding: '32px',
    textAlign: 'center' as const,
    color: '#6c757d',
    fontSize: '14px',
  },
};
