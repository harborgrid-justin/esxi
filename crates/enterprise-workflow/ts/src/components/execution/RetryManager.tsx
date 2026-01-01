/**
 * Retry Manager - Manage and retry failed steps
 */

import React from 'react';
import { StepExecution, StepStatus } from '../../types';

export interface RetryManagerProps {
  stepExecutions: StepExecution[];
  onRetry?: (stepId: string) => void;
}

export const RetryManager: React.FC<RetryManagerProps> = ({
  stepExecutions,
  onRetry
}) => {
  const failedSteps = stepExecutions.filter(
    step => step.status === StepStatus.FAILED
  );

  return (
    <div style={styles.container}>
      <h3 style={styles.title}>Failed Steps - Retry Manager</h3>

      {failedSteps.length === 0 ? (
        <div style={styles.emptyState}>
          No failed steps to retry
        </div>
      ) : (
        <div style={styles.list}>
          {failedSteps.map(step => (
            <div key={step.stepId} style={styles.stepCard}>
              <div style={styles.stepHeader}>
                <div style={styles.stepId}>{step.stepId}</div>
                <button
                  onClick={() => onRetry?.(step.stepId)}
                  style={styles.retryButton}
                >
                  Retry
                </button>
              </div>

              <div style={styles.stepBody}>
                <div style={styles.info}>
                  <span style={styles.label}>Attempts:</span> {step.attempt}
                </div>
                <div style={styles.info}>
                  <span style={styles.label}>Failed At:</span>{' '}
                  {step.completedAt?.toLocaleString() || '-'}
                </div>
                {step.error && (
                  <div style={styles.error}>
                    <strong>Error:</strong> {step.error.message}
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: { padding: '20px', backgroundColor: '#fff' },
  title: { margin: '0 0 20px 0', fontSize: '18px', fontWeight: 600 },
  emptyState: {
    padding: '40px',
    textAlign: 'center',
    color: '#10b981',
    fontSize: '14px'
  },
  list: { display: 'flex', flexDirection: 'column', gap: '12px' },
  stepCard: {
    padding: '16px',
    border: '1px solid #fecaca',
    borderRadius: '8px',
    backgroundColor: '#fef2f2'
  },
  stepHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px'
  },
  stepId: { fontSize: '14px', fontWeight: 600, fontFamily: 'monospace' },
  retryButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: 'white',
    border: 'none',
    borderRadius: '6px',
    cursor: 'pointer',
    fontSize: '14px'
  },
  stepBody: { display: 'flex', flexDirection: 'column', gap: '8px' },
  info: { fontSize: '14px', color: '#6b7280' },
  label: { fontWeight: 500 },
  error: {
    marginTop: '8px',
    padding: '8px',
    backgroundColor: '#fee2e2',
    borderRadius: '4px',
    fontSize: '13px',
    color: '#991b1b'
  }
};
