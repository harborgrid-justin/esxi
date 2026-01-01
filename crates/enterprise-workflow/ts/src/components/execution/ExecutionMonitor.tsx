/**
 * Execution Monitor - Live workflow execution monitoring
 */

import React, { useEffect, useState } from 'react';
import { Execution, ExecutionStatus, StepExecution, StepStatus } from '../../types';

export interface ExecutionMonitorProps {
  execution: Execution;
  onRefresh?: () => void;
}

export const ExecutionMonitor: React.FC<ExecutionMonitorProps> = ({
  execution,
  onRefresh
}) => {
  const [autoRefresh, setAutoRefresh] = useState(true);

  useEffect(() => {
    if (autoRefresh && execution.status === ExecutionStatus.RUNNING) {
      const interval = setInterval(() => {
        onRefresh?.();
      }, 2000);

      return () => clearInterval(interval);
    }
  }, [autoRefresh, execution.status, onRefresh]);

  const getStatusColor = (status: ExecutionStatus | StepStatus): string => {
    switch (status) {
      case ExecutionStatus.SUCCESS:
      case StepStatus.SUCCESS:
        return '#10b981';
      case ExecutionStatus.FAILED:
      case StepStatus.FAILED:
        return '#ef4444';
      case ExecutionStatus.RUNNING:
      case StepStatus.RUNNING:
        return '#3b82f6';
      case ExecutionStatus.WAITING:
        return '#f59e0b';
      default:
        return '#6b7280';
    }
  };

  const formatDuration = (ms?: number): string => {
    if (!ms) return '-';
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    }
    return `${seconds}s`;
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <div style={styles.headerContent}>
          <h2 style={styles.title}>Execution Monitor</h2>
          <div
            style={{
              ...styles.statusBadge,
              backgroundColor: getStatusColor(execution.status)
            }}
          >
            {execution.status}
          </div>
        </div>

        <div style={styles.controls}>
          <label style={styles.checkbox}>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto Refresh
          </label>
          <button onClick={onRefresh} style={styles.refreshButton}>
            Refresh
          </button>
        </div>
      </div>

      <div style={styles.summary}>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Workflow</div>
          <div style={styles.summaryValue}>{execution.workflowId}</div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Started</div>
          <div style={styles.summaryValue}>
            {execution.startedAt.toLocaleString()}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Duration</div>
          <div style={styles.summaryValue}>
            {formatDuration(execution.duration)}
          </div>
        </div>
        <div style={styles.summaryItem}>
          <div style={styles.summaryLabel}>Progress</div>
          <div style={styles.summaryValue}>
            {execution.metrics.completedSteps} / {execution.metrics.totalSteps} steps
          </div>
        </div>
      </div>

      <div style={styles.metricsGrid}>
        <div style={styles.metricCard}>
          <div style={styles.metricValue}>{execution.metrics.completedSteps}</div>
          <div style={styles.metricLabel}>Completed</div>
        </div>
        <div style={styles.metricCard}>
          <div style={styles.metricValue}>{execution.metrics.failedSteps}</div>
          <div style={styles.metricLabel}>Failed</div>
        </div>
        <div style={styles.metricCard}>
          <div style={styles.metricValue}>{execution.metrics.skippedSteps}</div>
          <div style={styles.metricLabel}>Skipped</div>
        </div>
        <div style={styles.metricCard}>
          <div style={styles.metricValue}>{execution.metrics.retryCount}</div>
          <div style={styles.metricLabel}>Retries</div>
        </div>
      </div>

      <div style={styles.stepsContainer}>
        <h3 style={styles.sectionTitle}>Step Executions</h3>
        <div style={styles.stepsList}>
          {execution.stepExecutions.map((stepExec, index) => (
            <StepExecutionCard
              key={`${stepExec.stepId}-${index}`}
              stepExecution={stepExec}
              getStatusColor={getStatusColor}
              formatDuration={formatDuration}
            />
          ))}
        </div>
      </div>

      {execution.error && (
        <div style={styles.errorContainer}>
          <h3 style={styles.errorTitle}>Error</h3>
          <div style={styles.errorMessage}>{execution.error.message}</div>
          {execution.error.stackTrace && (
            <pre style={styles.stackTrace}>{execution.error.stackTrace}</pre>
          )}
        </div>
      )}
    </div>
  );
};

const StepExecutionCard: React.FC<{
  stepExecution: StepExecution;
  getStatusColor: (status: StepStatus) => string;
  formatDuration: (ms?: number) => string;
}> = ({ stepExecution, getStatusColor, formatDuration }) => {
  const [expanded, setExpanded] = useState(false);

  return (
    <div style={styles.stepCard}>
      <div
        style={styles.stepHeader}
        onClick={() => setExpanded(!expanded)}
      >
        <div style={styles.stepHeaderLeft}>
          <div
            style={{
              ...styles.stepStatus,
              backgroundColor: getStatusColor(stepExecution.status)
            }}
          />
          <div style={styles.stepId}>{stepExecution.stepId}</div>
        </div>
        <div style={styles.stepHeaderRight}>
          <div style={styles.stepDuration}>
            {formatDuration(stepExecution.duration)}
          </div>
          <div style={styles.stepAttempt}>
            Attempt {stepExecution.attempt}
          </div>
        </div>
      </div>

      {expanded && (
        <div style={styles.stepDetails}>
          <div style={styles.detailRow}>
            <strong>Started:</strong> {stepExecution.startedAt.toLocaleString()}
          </div>
          {stepExecution.completedAt && (
            <div style={styles.detailRow}>
              <strong>Completed:</strong> {stepExecution.completedAt.toLocaleString()}
            </div>
          )}
          {stepExecution.output && (
            <div style={styles.detailRow}>
              <strong>Output:</strong>
              <pre style={styles.outputPre}>
                {JSON.stringify(stepExecution.output, null, 2)}
              </pre>
            </div>
          )}
          {stepExecution.error && (
            <div style={styles.stepError}>
              <strong>Error:</strong> {stepExecution.error.message}
            </div>
          )}
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#f9fafb'
  },
  header: {
    padding: '20px',
    backgroundColor: '#fff',
    borderBottom: '1px solid #e5e7eb'
  },
  headerContent: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    marginBottom: '16px'
  },
  title: {
    margin: 0,
    fontSize: '20px',
    fontWeight: 600
  },
  statusBadge: {
    padding: '4px 12px',
    borderRadius: '12px',
    color: 'white',
    fontSize: '12px',
    fontWeight: 600,
    textTransform: 'uppercase'
  },
  controls: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center'
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '14px',
    cursor: 'pointer'
  },
  refreshButton: {
    padding: '8px 16px',
    backgroundColor: '#3b82f6',
    color: 'white',
    border: 'none',
    borderRadius: '6px',
    cursor: 'pointer',
    fontSize: '14px'
  },
  summary: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '16px',
    padding: '20px',
    backgroundColor: '#fff',
    borderBottom: '1px solid #e5e7eb'
  },
  summaryItem: {},
  summaryLabel: {
    fontSize: '12px',
    color: '#6b7280',
    marginBottom: '4px'
  },
  summaryValue: {
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827'
  },
  metricsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(4, 1fr)',
    gap: '16px',
    padding: '20px',
    backgroundColor: '#fff',
    borderBottom: '1px solid #e5e7eb'
  },
  metricCard: {
    textAlign: 'center'
  },
  metricValue: {
    fontSize: '32px',
    fontWeight: 700,
    color: '#111827',
    marginBottom: '4px'
  },
  metricLabel: {
    fontSize: '14px',
    color: '#6b7280'
  },
  stepsContainer: {
    flex: 1,
    overflowY: 'auto',
    padding: '20px'
  },
  sectionTitle: {
    margin: '0 0 16px 0',
    fontSize: '16px',
    fontWeight: 600,
    color: '#111827'
  },
  stepsList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px'
  },
  stepCard: {
    backgroundColor: '#fff',
    border: '1px solid #e5e7eb',
    borderRadius: '8px',
    overflow: 'hidden'
  },
  stepHeader: {
    padding: '12px 16px',
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    cursor: 'pointer',
    userSelect: 'none'
  },
  stepHeaderLeft: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px'
  },
  stepStatus: {
    width: '12px',
    height: '12px',
    borderRadius: '50%'
  },
  stepId: {
    fontSize: '14px',
    fontWeight: 500,
    color: '#111827'
  },
  stepHeaderRight: {
    display: 'flex',
    gap: '16px',
    alignItems: 'center'
  },
  stepDuration: {
    fontSize: '14px',
    color: '#6b7280'
  },
  stepAttempt: {
    fontSize: '12px',
    color: '#6b7280'
  },
  stepDetails: {
    padding: '16px',
    backgroundColor: '#f9fafb',
    borderTop: '1px solid #e5e7eb'
  },
  detailRow: {
    marginBottom: '8px',
    fontSize: '14px',
    color: '#111827'
  },
  outputPre: {
    marginTop: '8px',
    padding: '12px',
    backgroundColor: '#1f2937',
    color: '#10b981',
    borderRadius: '4px',
    fontSize: '12px',
    overflowX: 'auto'
  },
  stepError: {
    padding: '12px',
    backgroundColor: '#fee2e2',
    border: '1px solid #fecaca',
    borderRadius: '4px',
    color: '#991b1b',
    fontSize: '14px'
  },
  errorContainer: {
    padding: '20px',
    backgroundColor: '#fee2e2',
    borderTop: '1px solid #fecaca'
  },
  errorTitle: {
    margin: '0 0 12px 0',
    fontSize: '16px',
    fontWeight: 600,
    color: '#991b1b'
  },
  errorMessage: {
    fontSize: '14px',
    color: '#7f1d1d',
    marginBottom: '12px'
  },
  stackTrace: {
    padding: '12px',
    backgroundColor: '#1f2937',
    color: '#ef4444',
    borderRadius: '4px',
    fontSize: '12px',
    overflowX: 'auto'
  }
};
