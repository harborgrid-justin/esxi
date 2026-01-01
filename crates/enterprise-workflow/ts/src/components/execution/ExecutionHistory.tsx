/**
 * Execution History - Past workflow executions
 */

import React, { useState } from 'react';
import { Execution, ExecutionStatus } from '../../types';

export interface ExecutionHistoryProps {
  executions: Execution[];
  onSelectExecution?: (execution: Execution) => void;
}

export const ExecutionHistory: React.FC<ExecutionHistoryProps> = ({
  executions,
  onSelectExecution
}) => {
  const [filter, setFilter] = useState<ExecutionStatus | 'all'>('all');
  const [sortBy, setSortBy] = useState<'date' | 'duration'>('date');

  const filteredExecutions = executions.filter(
    exec => filter === 'all' || exec.status === filter
  );

  const sortedExecutions = [...filteredExecutions].sort((a, b) => {
    if (sortBy === 'date') {
      return b.startedAt.getTime() - a.startedAt.getTime();
    }
    return (b.duration || 0) - (a.duration || 0);
  });

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h2 style={styles.title}>Execution History</h2>
        <div style={styles.controls}>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as any)}
            style={styles.select}
          >
            <option value="all">All Status</option>
            <option value={ExecutionStatus.SUCCESS}>Success</option>
            <option value={ExecutionStatus.FAILED}>Failed</option>
            <option value={ExecutionStatus.RUNNING}>Running</option>
            <option value={ExecutionStatus.CANCELLED}>Cancelled</option>
          </select>
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            style={styles.select}
          >
            <option value="date">Sort by Date</option>
            <option value="duration">Sort by Duration</option>
          </select>
        </div>
      </div>

      <div style={styles.list}>
        {sortedExecutions.map(execution => (
          <div
            key={execution.id}
            style={styles.executionCard}
            onClick={() => onSelectExecution?.(execution)}
          >
            <div style={styles.cardHeader}>
              <div style={styles.executionId}>{execution.id}</div>
              <div style={{
                ...styles.status,
                backgroundColor: getStatusColor(execution.status)
              }}>
                {execution.status}
              </div>
            </div>
            <div style={styles.cardBody}>
              <div style={styles.info}>
                <span style={styles.infoLabel}>Started:</span>
                {execution.startedAt.toLocaleString()}
              </div>
              <div style={styles.info}>
                <span style={styles.infoLabel}>Duration:</span>
                {formatDuration(execution.duration)}
              </div>
              <div style={styles.info}>
                <span style={styles.infoLabel}>Progress:</span>
                {execution.metrics.completedSteps}/{execution.metrics.totalSteps}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

const getStatusColor = (status: ExecutionStatus): string => {
  switch (status) {
    case ExecutionStatus.SUCCESS: return '#10b981';
    case ExecutionStatus.FAILED: return '#ef4444';
    case ExecutionStatus.RUNNING: return '#3b82f6';
    default: return '#6b7280';
  }
};

const formatDuration = (ms?: number): string => {
  if (!ms) return '-';
  return `${Math.floor(ms / 1000)}s`;
};

const styles: Record<string, React.CSSProperties> = {
  container: { display: 'flex', flexDirection: 'column', height: '100%' },
  header: { padding: '20px', borderBottom: '1px solid #e5e7eb' },
  title: { margin: '0 0 16px 0', fontSize: '20px', fontWeight: 600 },
  controls: { display: 'flex', gap: '12px' },
  select: { padding: '8px 12px', border: '1px solid #ddd', borderRadius: '6px', fontSize: '14px' },
  list: { flex: 1, overflowY: 'auto', padding: '20px' },
  executionCard: {
    padding: '16px',
    marginBottom: '12px',
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    cursor: 'pointer',
    transition: 'all 0.2s'
  },
  cardHeader: { display: 'flex', justifyContent: 'space-between', marginBottom: '12px' },
  executionId: { fontSize: '14px', fontWeight: 600, fontFamily: 'monospace' },
  status: {
    padding: '4px 8px',
    borderRadius: '4px',
    color: 'white',
    fontSize: '12px',
    fontWeight: 600
  },
  cardBody: { display: 'flex', flexDirection: 'column', gap: '6px' },
  info: { fontSize: '14px', color: '#6b7280' },
  infoLabel: { fontWeight: 500, marginRight: '8px' }
};
