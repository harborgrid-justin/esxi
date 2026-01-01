/**
 * Enterprise API Gateway - Analytics Dashboard Component
 */

import React from 'react';
import type { TrafficMetrics, AggregatedMetrics } from '../types';

export interface AnalyticsDashboardProps {
  metrics: AggregatedMetrics;
  trafficByRoute: TrafficMetrics[];
  statusCodes: Record<number, number>;
}

export const AnalyticsDashboard: React.FC<AnalyticsDashboardProps> = ({
  metrics,
  trafficByRoute,
  statusCodes,
}) => {
  const getStatusClass = (code: number): string => {
    if (code >= 200 && code < 300) return 'success';
    if (code >= 400 && code < 500) return 'client-error';
    if (code >= 500) return 'server-error';
    return 'other';
  };

  return (
    <div className="analytics-dashboard">
      <h2>Analytics Dashboard</h2>

      <div className="charts-grid">
        {/* Traffic by Route */}
        <div className="chart-card">
          <h3>Top Routes by Traffic</h3>
          {trafficByRoute.slice(0, 10).map((route) => (
            <div key={route.route} className="route-bar">
              <span className="route-name">{route.route}</span>
              <div className="bar">
                <div
                  className="fill"
                  style={{
                    width: `${(route.requests / trafficByRoute[0]!.requests) * 100}%`,
                  }}
                />
              </div>
              <span className="route-count">{route.requests}</span>
            </div>
          ))}
        </div>

        {/* Status Code Distribution */}
        <div className="chart-card">
          <h3>Status Code Distribution</h3>
          {Object.entries(statusCodes).map(([code, count]) => (
            <div key={code} className={`status-item ${getStatusClass(Number(code))}`}>
              <span className="code">{code}</span>
              <span className="count">{count}</span>
            </div>
          ))}
        </div>
      </div>

      <style>{`
        .analytics-dashboard {
          padding: 20px;
        }

        .charts-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
          gap: 20px;
        }

        .chart-card {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .route-bar {
          display: grid;
          grid-template-columns: 150px 1fr 60px;
          align-items: center;
          gap: 10px;
          margin-bottom: 10px;
        }

        .bar {
          height: 24px;
          background: #e9ecef;
          border-radius: 4px;
          overflow: hidden;
        }

        .fill {
          height: 100%;
          background: #007bff;
        }

        .status-item {
          display: flex;
          justify-content: space-between;
          padding: 8px;
          margin-bottom: 5px;
          border-radius: 4px;
        }

        .status-item.success {
          background: #d4edda;
        }

        .status-item.client-error {
          background: #fff3cd;
        }

        .status-item.server-error {
          background: #f8d7da;
        }
      `}</style>
    </div>
  );
};
