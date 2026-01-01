/**
 * Step Debugger - Debug individual workflow steps
 */

import React from 'react';
import { StepExecution } from '../../types';

export interface StepDebuggerProps {
  stepExecution: StepExecution;
}

export const StepDebugger: React.FC<StepDebuggerProps> = ({ stepExecution }) => {
  return (
    <div style={styles.container}>
      <h3 style={styles.title}>Step Debugger: {stepExecution.stepId}</h3>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Execution Info</h4>
        <div style={styles.grid}>
          <div style={styles.field}>
            <span style={styles.label}>Status:</span>
            <span style={styles.value}>{stepExecution.status}</span>
          </div>
          <div style={styles.field}>
            <span style={styles.label}>Attempt:</span>
            <span style={styles.value}>{stepExecution.attempt}</span>
          </div>
          <div style={styles.field}>
            <span style={styles.label}>Duration:</span>
            <span style={styles.value}>
              {stepExecution.duration ? `${stepExecution.duration}ms` : '-'}
            </span>
          </div>
        </div>
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Input</h4>
        <pre style={styles.code}>
          {JSON.stringify(stepExecution.input, null, 2)}
        </pre>
      </div>

      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Output</h4>
        <pre style={styles.code}>
          {JSON.stringify(stepExecution.output, null, 2)}
        </pre>
      </div>

      {stepExecution.error && (
        <div style={styles.section}>
          <h4 style={styles.sectionTitle}>Error</h4>
          <div style={styles.error}>
            <div>{stepExecution.error.message}</div>
            {stepExecution.error.stackTrace && (
              <pre style={styles.stackTrace}>
                {stepExecution.error.stackTrace}
              </pre>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: { padding: '20px', backgroundColor: '#fff' },
  title: { margin: '0 0 24px 0', fontSize: '18px', fontWeight: 600 },
  section: { marginBottom: '24px' },
  sectionTitle: { margin: '0 0 12px 0', fontSize: '14px', fontWeight: 600, color: '#666' },
  grid: { display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '16px' },
  field: { display: 'flex', flexDirection: 'column', gap: '4px' },
  label: { fontSize: '12px', color: '#666' },
  value: { fontSize: '14px', fontWeight: 500 },
  code: {
    padding: '12px',
    backgroundColor: '#1f2937',
    color: '#10b981',
    borderRadius: '4px',
    fontSize: '12px',
    overflowX: 'auto'
  },
  error: {
    padding: '12px',
    backgroundColor: '#fee2e2',
    border: '1px solid #fecaca',
    borderRadius: '4px',
    color: '#991b1b'
  },
  stackTrace: {
    marginTop: '8px',
    padding: '8px',
    backgroundColor: '#1f2937',
    color: '#ef4444',
    borderRadius: '4px',
    fontSize: '11px'
  }
};
