/**
 * KPIWidget - Key Performance Indicator card widget
 */

import React from 'react';
import { Widget, KpiWidgetConfig } from '../../types';
import { useDataSource } from '../../hooks/useDataSource';

export interface KPIWidgetProps {
  widget: Widget;
}

export const KPIWidget: React.FC<KPIWidgetProps> = ({ widget }) => {
  const { data, loading, error } = useDataSource(widget.data_source);
  const config = widget.config as KpiWidgetConfig;

  const calculateMetric = (): number => {
    if (!Array.isArray(data) || data.length === 0) return 0;

    const values = data
      .map((item) => Number(item[config.metric]))
      .filter((val) => !isNaN(val));

    switch (config.aggregation) {
      case 'sum':
        return values.reduce((a, b) => a + b, 0);
      case 'avg':
        return values.reduce((a, b) => a + b, 0) / values.length;
      case 'min':
        return Math.min(...values);
      case 'max':
        return Math.max(...values);
      case 'count':
        return values.length;
      case 'count_distinct':
        return new Set(values).size;
      default:
        return 0;
    }
  };

  const formatMetric = (value: number): string => {
    if (config.format === 'currency') {
      return `$${value.toLocaleString()}`;
    }
    if (config.format === 'percent') {
      return `${(value * 100).toFixed(2)}%`;
    }
    if (config.format === 'decimal') {
      return value.toFixed(2);
    }
    return value.toLocaleString();
  };

  if (loading) {
    return (
      <div className="kpi-loading">
        <div className="spinner"></div>
      </div>
    );
  }

  if (error) {
    return <div className="kpi-error">Error: {error.message}</div>;
  }

  const currentValue = calculateMetric();
  const comparisonValue = config.comparison ? calculateMetric() * 0.9 : null; // Simplified
  const change = comparisonValue
    ? ((currentValue - comparisonValue) / comparisonValue) * 100
    : null;

  return (
    <div className="kpi-widget">
      <div className="kpi-label">{config.metric.toUpperCase()}</div>
      <div className="kpi-value">{formatMetric(currentValue)}</div>

      {config.comparison && change !== null && (
        <div className={`kpi-comparison ${change >= 0 ? 'positive' : 'negative'}`}>
          <span className="change-arrow">{change >= 0 ? '↑' : '↓'}</span>
          {config.comparison.show_percentage && (
            <span className="change-percent">{Math.abs(change).toFixed(2)}%</span>
          )}
          {config.comparison.show_change && (
            <span className="change-value">
              ({formatMetric(Math.abs(currentValue - comparisonValue))})
            </span>
          )}
        </div>
      )}

      <div className="kpi-footer">
        <span className="aggregation-type">{config.aggregation}</span>
      </div>

      <style jsx>{`
        .kpi-widget {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          padding: 20px;
          text-align: center;
        }

        .kpi-label {
          font-size: 12px;
          font-weight: 600;
          color: #666;
          letter-spacing: 1px;
          margin-bottom: 12px;
        }

        .kpi-value {
          font-size: 36px;
          font-weight: 700;
          color: #333;
          margin-bottom: 12px;
        }

        .kpi-comparison {
          font-size: 14px;
          font-weight: 600;
          display: flex;
          align-items: center;
          gap: 6px;
        }

        .kpi-comparison.positive {
          color: #4caf50;
        }

        .kpi-comparison.negative {
          color: #f44336;
        }

        .change-arrow {
          font-size: 18px;
        }

        .change-value {
          font-size: 12px;
          opacity: 0.8;
        }

        .kpi-footer {
          margin-top: auto;
          padding-top: 12px;
          border-top: 1px solid #e0e0e0;
          width: 100%;
        }

        .aggregation-type {
          font-size: 11px;
          color: #999;
          text-transform: uppercase;
        }

        .kpi-loading,
        .kpi-error {
          display: flex;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: #666;
        }

        .spinner {
          border: 3px solid #f3f3f3;
          border-top: 3px solid #1976d2;
          border-radius: 50%;
          width: 40px;
          height: 40px;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% {
            transform: rotate(0deg);
          }
          100% {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </div>
  );
};
