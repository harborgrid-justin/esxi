/**
 * Metric Card Component - KPI Display
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

import React from 'react';

export interface MetricCardProps {
  title: string;
  value: number | string;
  format?: string;
  change?: number;
  changeLabel?: string;
  icon?: React.ReactNode;
  color?: string;
  trend?: 'up' | 'down' | 'neutral';
}

export function MetricCard({
  title,
  value,
  format,
  change,
  changeLabel = 'vs previous period',
  icon,
  color = '#1f77b4',
  trend = 'neutral',
}: MetricCardProps) {
  const formatValue = (val: number | string): string => {
    if (typeof val === 'number') {
      if (format === 'currency') {
        return new Intl.NumberFormat('en-US', {
          style: 'currency',
          currency: 'USD',
        }).format(val);
      } else if (format === 'percent') {
        return `${(val * 100).toFixed(1)}%`;
      } else if (format === 'decimal') {
        return val.toLocaleString(undefined, {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        });
      }
      return val.toLocaleString();
    }
    return String(val);
  };

  const getTrendColor = (): string => {
    if (trend === 'up') return '#27ae60';
    if (trend === 'down') return '#e74c3c';
    return '#95a5a6';
  };

  const getTrendIcon = (): string => {
    if (trend === 'up') return '↑';
    if (trend === 'down') return '↓';
    return '→';
  };

  return (
    <div
      style={{
        backgroundColor: 'white',
        borderRadius: '8px',
        padding: '20px',
        boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
        borderLeft: `4px solid ${color}`,
      }}
    >
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '16px' }}>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: '14px', color: '#666', marginBottom: '8px' }}>
            {title}
          </div>
          <div style={{ fontSize: '32px', fontWeight: 'bold', color: '#333' }}>
            {formatValue(value)}
          </div>
        </div>
        {icon && (
          <div style={{ fontSize: '24px', color, opacity: 0.7 }}>
            {icon}
          </div>
        )}
      </div>

      {/* Change Indicator */}
      {change !== undefined && (
        <div style={{ display: 'flex', alignItems: 'center', fontSize: '13px' }}>
          <span
            style={{
              color: getTrendColor(),
              fontWeight: '600',
              marginRight: '6px',
            }}
          >
            {getTrendIcon()} {Math.abs(change)}%
          </span>
          <span style={{ color: '#95a5a6' }}>{changeLabel}</span>
        </div>
      )}
    </div>
  );
}
