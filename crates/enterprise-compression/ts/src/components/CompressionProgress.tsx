/**
 * Compression Progress Component
 * Real-time progress display for compression operations
 */

import React from 'react';
import { ProgressInfo } from '../types';

interface CompressionProgressProps {
  progress: ProgressInfo;
  showDetails?: boolean;
}

export const CompressionProgress: React.FC<CompressionProgressProps> = ({
  progress,
  showDetails = true,
}) => {
  const formatBytes = (bytes: number): string => {
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const formatTime = (seconds: number): string => {
    if (seconds < 60) return `${Math.round(seconds)}s`;
    const mins = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${mins}m ${secs}s`;
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <div style={styles.title}>Processing...</div>
        <div style={styles.percentage}>{progress.percentage.toFixed(1)}%</div>
      </div>

      <div style={styles.progressBar}>
        <div
          style={{
            ...styles.progressFill,
            width: `${progress.percentage}%`,
          }}
        />
      </div>

      {showDetails && (
        <div style={styles.details}>
          <div style={styles.detail}>
            <span style={styles.detailLabel}>Processed:</span>
            <span style={styles.detailValue}>
              {formatBytes(progress.processedBytes)} / {formatBytes(progress.totalBytes)}
            </span>
          </div>
          <div style={styles.detail}>
            <span style={styles.detailLabel}>Chunks:</span>
            <span style={styles.detailValue}>
              {progress.currentChunk} / {progress.totalChunks}
            </span>
          </div>
          <div style={styles.detail}>
            <span style={styles.detailLabel}>Throughput:</span>
            <span style={styles.detailValue}>
              {formatBytes(progress.throughput)}/s
            </span>
          </div>
          <div style={styles.detail}>
            <span style={styles.detailLabel}>Time Remaining:</span>
            <span style={styles.detailValue}>
              {formatTime(progress.estimatedTimeRemaining)}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

const styles = {
  container: {
    padding: '20px',
    backgroundColor: 'white',
    borderRadius: '8px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '12px',
  },
  title: {
    fontSize: '16px',
    fontWeight: '500',
    color: '#333',
  },
  percentage: {
    fontSize: '18px',
    fontWeight: '600',
    color: '#007bff',
  },
  progressBar: {
    height: '8px',
    backgroundColor: '#e0e0e0',
    borderRadius: '4px',
    overflow: 'hidden',
    marginBottom: '16px',
  },
  progressFill: {
    height: '100%',
    backgroundColor: '#007bff',
    transition: 'width 0.3s ease',
  },
  details: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
    gap: '12px',
  },
  detail: {
    display: 'flex',
    justifyContent: 'space-between',
    fontSize: '14px',
  },
  detailLabel: {
    color: '#666',
  },
  detailValue: {
    color: '#333',
    fontWeight: '500',
  },
};
