import React from 'react';
import type { ScanContext, MonitorMetrics, HealthStatus } from '../../types';
import { StatusIndicator } from './StatusIndicator';
import { ScanProgress } from './ScanProgress';

export interface LiveMonitorProps {
  activeScans: Map<string, ScanContext>;
  metrics: MonitorMetrics | null;
  health: HealthStatus;
  isConnected: boolean;
}

/**
 * Real-time monitoring dashboard
 */
export const LiveMonitor: React.FC<LiveMonitorProps> = ({
  activeScans,
  metrics,
  health,
  isConnected,
}) => {
  return (
    <div className="live-monitor">
      <div className="monitor-header">
        <h2>Real-time Accessibility Monitor</h2>
        <StatusIndicator health={health} isConnected={isConnected} />
      </div>

      <div className="monitor-stats">
        {metrics && (
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-label">Active Scans</div>
              <div className="stat-value">{metrics.active_scans}</div>
            </div>

            <div className="stat-card">
              <div className="stat-label">Completed Scans</div>
              <div className="stat-value">{metrics.completed_scans}</div>
            </div>

            <div className="stat-card">
              <div className="stat-label">Total Issues</div>
              <div className="stat-value">{metrics.total_issues}</div>
            </div>

            <div className="stat-card">
              <div className="stat-label">Avg Duration</div>
              <div className="stat-value">
                {(metrics.average_scan_duration_ms / 1000).toFixed(1)}s
              </div>
            </div>
          </div>
        )}
      </div>

      <div className="active-scans">
        <h3>Active Scans ({activeScans.size})</h3>
        {activeScans.size === 0 ? (
          <div className="empty-state">
            <p>No active scans</p>
          </div>
        ) : (
          <div className="scans-list">
            {Array.from(activeScans.entries()).map(([scanId, context]) => (
              <ScanProgress key={scanId} scanId={scanId} context={context} />
            ))}
          </div>
        )}
      </div>

      {metrics && metrics.issues_by_severity && (
        <div className="severity-breakdown">
          <h3>Issues by Severity</h3>
          <div className="severity-chart">
            {Object.entries(metrics.issues_by_severity).map(([severity, count]) => (
              <div key={severity} className="severity-bar">
                <div className="severity-label">
                  <span className={`severity-badge severity-${severity}`}>{severity}</span>
                  <span className="severity-count">{count}</span>
                </div>
                <div className="severity-progress">
                  <div
                    className={`severity-fill severity-${severity}`}
                    style={{
                      width: `${(count / Math.max(...Object.values(metrics.issues_by_severity))) * 100}%`,
                    }}
                  />
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <style>{`
        .live-monitor {
          padding: 24px;
          background: #ffffff;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .monitor-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .monitor-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #111827;
        }

        .stats-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 32px;
        }

        .stat-card {
          padding: 16px;
          background: #f9fafb;
          border-radius: 6px;
          border: 1px solid #e5e7eb;
        }

        .stat-label {
          font-size: 12px;
          font-weight: 500;
          color: #6b7280;
          text-transform: uppercase;
          letter-spacing: 0.5px;
          margin-bottom: 8px;
        }

        .stat-value {
          font-size: 28px;
          font-weight: 700;
          color: #111827;
        }

        .active-scans h3,
        .severity-breakdown h3 {
          font-size: 18px;
          font-weight: 600;
          color: #111827;
          margin: 0 0 16px 0;
        }

        .scans-list {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .empty-state {
          padding: 48px;
          text-align: center;
          color: #6b7280;
        }

        .severity-breakdown {
          margin-top: 32px;
        }

        .severity-chart {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .severity-bar {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .severity-label {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .severity-badge {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
          text-transform: capitalize;
          color: white;
        }

        .severity-badge.severity-critical {
          background: #dc2626;
        }

        .severity-badge.severity-high {
          background: #ea580c;
        }

        .severity-badge.severity-medium {
          background: #f59e0b;
        }

        .severity-badge.severity-low {
          background: #3b82f6;
        }

        .severity-badge.severity-info {
          background: #6b7280;
        }

        .severity-count {
          font-size: 16px;
          font-weight: 700;
          color: #111827;
        }

        .severity-progress {
          height: 8px;
          background: #e5e7eb;
          border-radius: 4px;
          overflow: hidden;
        }

        .severity-fill {
          height: 100%;
          transition: width 0.3s ease;
        }

        .severity-fill.severity-critical {
          background: #dc2626;
        }

        .severity-fill.severity-high {
          background: #ea580c;
        }

        .severity-fill.severity-medium {
          background: #f59e0b;
        }

        .severity-fill.severity-low {
          background: #3b82f6;
        }

        .severity-fill.severity-info {
          background: #6b7280;
        }
      `}</style>
    </div>
  );
};
