/**
 * Log Viewer - View execution logs
 */

import React, { useState } from 'react';
import { ExecutionLog } from '../../types';

export interface LogViewerProps {
  logs: ExecutionLog[];
}

export const LogViewer: React.FC<LogViewerProps> = ({ logs }) => {
  const [filter, setFilter] = useState<'all' | 'debug' | 'info' | 'warn' | 'error'>('all');

  const filteredLogs = logs.filter(
    log => filter === 'all' || log.level === filter
  );

  const getLevelColor = (level: string): string => {
    switch (level) {
      case 'error': return '#ef4444';
      case 'warn': return '#f59e0b';
      case 'info': return '#3b82f6';
      case 'debug': return '#6b7280';
      default: return '#000';
    }
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Execution Logs</h3>
        <select
          value={filter}
          onChange={(e) => setFilter(e.target.value as any)}
          style={styles.select}
        >
          <option value="all">All Levels</option>
          <option value="debug">Debug</option>
          <option value="info">Info</option>
          <option value="warn">Warn</option>
          <option value="error">Error</option>
        </select>
      </div>

      <div style={styles.logContainer}>
        {filteredLogs.map(log => (
          <div key={log.id} style={styles.logEntry}>
            <span style={styles.timestamp}>
              {log.timestamp.toLocaleTimeString()}
            </span>
            <span style={{ ...styles.level, color: getLevelColor(log.level) }}>
              [{log.level.toUpperCase()}]
            </span>
            {log.stepId && (
              <span style={styles.stepId}>[{log.stepId}]</span>
            )}
            <span style={styles.message}>{log.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: { display: 'flex', flexDirection: 'column', height: '100%', backgroundColor: '#1f2937' },
  header: {
    padding: '12px 16px',
    borderBottom: '1px solid #374151',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center'
  },
  title: { margin: 0, fontSize: '14px', fontWeight: 600, color: '#f9fafb' },
  select: { padding: '6px 10px', fontSize: '12px', borderRadius: '4px' },
  logContainer: {
    flex: 1,
    overflowY: 'auto',
    padding: '8px',
    fontFamily: 'monospace',
    fontSize: '12px'
  },
  logEntry: {
    padding: '4px 8px',
    marginBottom: '2px',
    color: '#f9fafb',
    display: 'flex',
    gap: '8px'
  },
  timestamp: { color: '#9ca3af', whiteSpace: 'nowrap' },
  level: { fontWeight: 600, whiteSpace: 'nowrap' },
  stepId: { color: '#60a5fa', whiteSpace: 'nowrap' },
  message: { flex: 1 }
};
