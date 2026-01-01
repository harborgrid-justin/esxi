/**
 * Compression Dashboard Component
 * Real-time compression statistics and monitoring
 */

import React, { useState, useEffect } from 'react';
import {
  BandwidthMetrics,
  StorageMetrics,
  CompressionAlgorithm,
  AlgorithmMetrics,
} from '../types';

interface CompressionDashboardProps {
  bandwidthMetrics: BandwidthMetrics;
  storageMetrics: StorageMetrics;
  onRefresh?: () => void;
  refreshInterval?: number;
}

export const CompressionDashboard: React.FC<CompressionDashboardProps> = ({
  bandwidthMetrics,
  storageMetrics,
  onRefresh,
  refreshInterval = 5000,
}) => {
  const [autoRefresh, setAutoRefresh] = useState(true);

  useEffect(() => {
    if (!autoRefresh || !onRefresh) return;

    const interval = setInterval(onRefresh, refreshInterval);
    return () => clearInterval(interval);
  }, [autoRefresh, onRefresh, refreshInterval]);

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
  };

  const formatPercent = (value: number): string => {
    return `${value.toFixed(2)}%`;
  };

  return (
    <div style={styles.dashboard}>
      <div style={styles.header}>
        <h2 style={styles.title}>Compression Dashboard</h2>
        <div style={styles.controls}>
          <label style={styles.label}>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto-refresh
          </label>
          {onRefresh && (
            <button onClick={onRefresh} style={styles.button}>
              Refresh Now
            </button>
          )}
        </div>
      </div>

      <div style={styles.grid}>
        {/* Bandwidth Stats */}
        <div style={styles.card}>
          <h3 style={styles.cardTitle}>Bandwidth Savings</h3>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Total Saved</div>
            <div style={styles.statValue}>
              {formatBytes(bandwidthMetrics.savedBytes)}
            </div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Savings Percent</div>
            <div style={styles.statValue}>
              {formatPercent(bandwidthMetrics.savingsPercent)}
            </div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Requests</div>
            <div style={styles.statValue}>{bandwidthMetrics.requestCount}</div>
          </div>
        </div>

        {/* Storage Stats */}
        <div style={styles.card}>
          <h3 style={styles.cardTitle}>Storage Optimization</h3>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Total Size</div>
            <div style={styles.statValue}>
              {formatBytes(storageMetrics.totalSize)}
            </div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Compressed Size</div>
            <div style={styles.statValue}>
              {formatBytes(storageMetrics.compressedSize)}
            </div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Files</div>
            <div style={styles.statValue}>{storageMetrics.fileCount}</div>
          </div>
        </div>

        {/* Algorithm Performance */}
        <div style={{ ...styles.card, gridColumn: '1 / -1' }}>
          <h3 style={styles.cardTitle}>Algorithm Performance</h3>
          <table style={styles.table}>
            <thead>
              <tr>
                <th style={styles.th}>Algorithm</th>
                <th style={styles.th}>Usage Count</th>
                <th style={styles.th}>Avg Ratio</th>
                <th style={styles.th}>Avg Duration</th>
                <th style={styles.th}>Throughput</th>
              </tr>
            </thead>
            <tbody>
              {Array.from(bandwidthMetrics.byAlgorithm.entries()).map(
                ([algo, metrics]) => (
                  <tr key={algo}>
                    <td style={styles.td}>{algo}</td>
                    <td style={styles.td}>{metrics.usageCount}</td>
                    <td style={styles.td}>{metrics.averageRatio.toFixed(2)}x</td>
                    <td style={styles.td}>{metrics.averageDuration.toFixed(2)}ms</td>
                    <td style={styles.td}>
                      {formatBytes(metrics.averageThroughput)}/s
                    </td>
                  </tr>
                )
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

const styles = {
  dashboard: {
    padding: '24px',
    backgroundColor: '#f5f5f5',
    borderRadius: '8px',
    fontFamily: 'system-ui, -apple-system, sans-serif',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
  },
  title: {
    margin: 0,
    fontSize: '24px',
    fontWeight: '600',
    color: '#333',
  },
  controls: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center',
  },
  label: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
    fontSize: '14px',
    color: '#666',
  },
  button: {
    padding: '8px 16px',
    backgroundColor: '#007bff',
    color: 'white',
    border: 'none',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '14px',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))',
    gap: '16px',
  },
  card: {
    backgroundColor: 'white',
    padding: '20px',
    borderRadius: '6px',
    boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
  },
  cardTitle: {
    margin: '0 0 16px 0',
    fontSize: '18px',
    fontWeight: '500',
    color: '#333',
  },
  stat: {
    marginBottom: '12px',
  },
  statLabel: {
    fontSize: '12px',
    color: '#666',
    marginBottom: '4px',
  },
  statValue: {
    fontSize: '24px',
    fontWeight: '600',
    color: '#007bff',
  },
  table: {
    width: '100%',
    borderCollapse: 'collapse' as const,
  },
  th: {
    textAlign: 'left' as const,
    padding: '12px',
    borderBottom: '2px solid #e0e0e0',
    fontSize: '14px',
    fontWeight: '600',
    color: '#333',
  },
  td: {
    padding: '12px',
    borderBottom: '1px solid #e0e0e0',
    fontSize: '14px',
    color: '#666',
  },
};
