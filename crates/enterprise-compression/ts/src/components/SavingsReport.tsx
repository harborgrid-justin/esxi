/**
 * Savings Report Component
 * Display bandwidth and storage savings
 */

import React from 'react';
import { BandwidthMetrics, StorageMetrics } from '../types';

interface SavingsReportProps {
  bandwidth: BandwidthMetrics;
  storage: StorageMetrics;
  period?: string;
}

export const SavingsReport: React.FC<SavingsReportProps> = ({
  bandwidth,
  storage,
  period = 'This Month',
}) => {
  const formatBytes = (bytes: number): string => {
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const totalSaved = bandwidth.savedBytes + (storage.totalSize - storage.compressedSize);

  return (
    <div style={styles.container}>
      <h2 style={styles.title}>Savings Report - {period}</h2>

      <div style={styles.highlight}>
        <div style={styles.highlightLabel}>Total Savings</div>
        <div style={styles.highlightValue}>{formatBytes(totalSaved)}</div>
      </div>

      <div style={styles.grid}>
        <div style={styles.card}>
          <h3 style={styles.cardTitle}>Bandwidth Savings</h3>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Data Saved</div>
            <div style={styles.statValue}>{formatBytes(bandwidth.savedBytes)}</div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Reduction</div>
            <div style={styles.statValue}>{bandwidth.savingsPercent.toFixed(1)}%</div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Requests</div>
            <div style={styles.statValue}>{bandwidth.requestCount.toLocaleString()}</div>
          </div>
        </div>

        <div style={styles.card}>
          <h3 style={styles.cardTitle}>Storage Savings</h3>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Space Saved</div>
            <div style={styles.statValue}>
              {formatBytes(storage.totalSize - storage.compressedSize)}
            </div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Reduction</div>
            <div style={styles.statValue}>{storage.savingsPercent.toFixed(1)}%</div>
          </div>
          <div style={styles.stat}>
            <div style={styles.statLabel}>Files</div>
            <div style={styles.statValue}>{storage.fileCount.toLocaleString()}</div>
          </div>
        </div>
      </div>
    </div>
  );
};

const styles = {
  container: {
    padding: '24px',
    backgroundColor: '#f5f5f5',
    borderRadius: '8px',
  },
  title: {
    margin: '0 0 24px 0',
    fontSize: '24px',
    fontWeight: '600',
    color: '#333',
  },
  highlight: {
    backgroundColor: '#007bff',
    color: 'white',
    padding: '24px',
    borderRadius: '6px',
    textAlign: 'center' as const,
    marginBottom: '24px',
  },
  highlightLabel: {
    fontSize: '14px',
    opacity: 0.9,
    marginBottom: '8px',
  },
  highlightValue: {
    fontSize: '36px',
    fontWeight: '600',
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
    fontSize: '20px',
    fontWeight: '600',
    color: '#333',
  },
};
