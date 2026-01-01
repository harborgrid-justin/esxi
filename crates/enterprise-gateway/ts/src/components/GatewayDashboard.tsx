/**
 * Enterprise API Gateway - Gateway Dashboard Component
 *
 * Main dashboard showing gateway overview and statistics
 */

import React, { useState, useEffect } from 'react';
import type { GatewayConfig, AggregatedMetrics } from '../types';

export interface GatewayDashboardProps {
  config: GatewayConfig;
  metrics?: AggregatedMetrics;
  onRefresh?: () => void;
}

export const GatewayDashboard: React.FC<GatewayDashboardProps> = ({
  config,
  metrics,
  onRefresh,
}) => {
  const [refreshInterval, setRefreshInterval] = useState(5000);
  const [autoRefresh, setAutoRefresh] = useState(true);

  useEffect(() => {
    if (autoRefresh && onRefresh) {
      const interval = setInterval(onRefresh, refreshInterval);
      return () => clearInterval(interval);
    }
  }, [autoRefresh, refreshInterval, onRefresh]);

  const formatNumber = (num: number): string => {
    if (num >= 1000000) return `${(num / 1000000).toFixed(2)}M`;
    if (num >= 1000) return `${(num / 1000).toFixed(2)}K`;
    return num.toFixed(0);
  };

  const formatPercentage = (num: number): string => {
    return `${(num * 100).toFixed(2)}%`;
  };

  const formatLatency = (ms: number): string => {
    return `${ms.toFixed(2)}ms`;
  };

  return (
    <div className="gateway-dashboard">
      <div className="dashboard-header">
        <h1>API Gateway Dashboard</h1>
        <div className="dashboard-controls">
          <label>
            <input
              type="checkbox"
              checked={autoRefresh}
              onChange={(e) => setAutoRefresh(e.target.checked)}
            />
            Auto-refresh
          </label>
          <select
            value={refreshInterval}
            onChange={(e) => setRefreshInterval(Number(e.target.value))}
            disabled={!autoRefresh}
          >
            <option value={1000}>1s</option>
            <option value={5000}>5s</option>
            <option value={10000}>10s</option>
            <option value={30000}>30s</option>
          </select>
          <button onClick={onRefresh}>Refresh Now</button>
        </div>
      </div>

      {/* Configuration Summary */}
      <div className="config-summary">
        <div className="config-card">
          <h3>Gateway Configuration</h3>
          <div className="config-details">
            <div className="config-item">
              <span className="label">Host:</span>
              <span className="value">{config.host}:{config.port}</span>
            </div>
            <div className="config-item">
              <span className="label">Workers:</span>
              <span className="value">{config.workers || 1}</span>
            </div>
            <div className="config-item">
              <span className="label">SSL:</span>
              <span className="value">{config.ssl?.enabled ? 'Enabled' : 'Disabled'}</span>
            </div>
            <div className="config-item">
              <span className="label">CORS:</span>
              <span className="value">{config.cors?.enabled ? 'Enabled' : 'Disabled'}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Metrics Overview */}
      {metrics && (
        <div className="metrics-overview">
          <div className="metric-card">
            <div className="metric-icon">📊</div>
            <div className="metric-content">
              <div className="metric-value">{formatNumber(metrics.totalRequests)}</div>
              <div className="metric-label">Total Requests</div>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">✅</div>
            <div className="metric-content">
              <div className="metric-value">{formatPercentage(metrics.successRate)}</div>
              <div className="metric-label">Success Rate</div>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">⚡</div>
            <div className="metric-content">
              <div className="metric-value">{formatLatency(metrics.averageLatency)}</div>
              <div className="metric-label">Avg Latency</div>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">🚀</div>
            <div className="metric-content">
              <div className="metric-value">{formatNumber(metrics.requestsPerSecond)}</div>
              <div className="metric-label">Requests/sec</div>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">❌</div>
            <div className="metric-content">
              <div className="metric-value">{formatPercentage(metrics.errorRate)}</div>
              <div className="metric-label">Error Rate</div>
            </div>
          </div>

          <div className="metric-card">
            <div className="metric-icon">💾</div>
            <div className="metric-content">
              <div className="metric-value">{formatPercentage(metrics.cacheHitRate)}</div>
              <div className="metric-label">Cache Hit Rate</div>
            </div>
          </div>
        </div>
      )}

      {/* Latency Breakdown */}
      {metrics && (
        <div className="latency-breakdown">
          <h3>Latency Percentiles</h3>
          <div className="latency-bars">
            <div className="latency-item">
              <span className="latency-label">P50</span>
              <div className="latency-bar">
                <div
                  className="latency-fill"
                  style={{ width: `${(metrics.p50Latency / metrics.p99Latency) * 100}%` }}
                />
              </div>
              <span className="latency-value">{formatLatency(metrics.p50Latency)}</span>
            </div>
            <div className="latency-item">
              <span className="latency-label">P95</span>
              <div className="latency-bar">
                <div
                  className="latency-fill"
                  style={{ width: `${(metrics.p95Latency / metrics.p99Latency) * 100}%` }}
                />
              </div>
              <span className="latency-value">{formatLatency(metrics.p95Latency)}</span>
            </div>
            <div className="latency-item">
              <span className="latency-label">P99</span>
              <div className="latency-bar">
                <div className="latency-fill" style={{ width: '100%' }} />
              </div>
              <span className="latency-value">{formatLatency(metrics.p99Latency)}</span>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .gateway-dashboard {
          padding: 20px;
          font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
        }

        .dashboard-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 30px;
        }

        .dashboard-header h1 {
          margin: 0;
          font-size: 28px;
          color: #1a1a1a;
        }

        .dashboard-controls {
          display: flex;
          gap: 10px;
          align-items: center;
        }

        .dashboard-controls button {
          padding: 8px 16px;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .dashboard-controls button:hover {
          background: #0056b3;
        }

        .config-summary {
          margin-bottom: 30px;
        }

        .config-card {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .config-card h3 {
          margin-top: 0;
          margin-bottom: 15px;
        }

        .config-details {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 15px;
        }

        .config-item {
          display: flex;
          justify-content: space-between;
        }

        .config-item .label {
          font-weight: 600;
          color: #666;
        }

        .metrics-overview {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 20px;
          margin-bottom: 30px;
        }

        .metric-card {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
          display: flex;
          align-items: center;
          gap: 15px;
        }

        .metric-icon {
          font-size: 36px;
        }

        .metric-value {
          font-size: 24px;
          font-weight: bold;
          color: #1a1a1a;
        }

        .metric-label {
          font-size: 14px;
          color: #666;
        }

        .latency-breakdown {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .latency-bars {
          margin-top: 15px;
        }

        .latency-item {
          display: grid;
          grid-template-columns: 50px 1fr 100px;
          align-items: center;
          gap: 15px;
          margin-bottom: 15px;
        }

        .latency-label {
          font-weight: 600;
          color: #666;
        }

        .latency-bar {
          height: 24px;
          background: #e9ecef;
          border-radius: 4px;
          overflow: hidden;
        }

        .latency-fill {
          height: 100%;
          background: linear-gradient(90deg, #28a745, #ffc107);
          transition: width 0.3s ease;
        }

        .latency-value {
          text-align: right;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
};
