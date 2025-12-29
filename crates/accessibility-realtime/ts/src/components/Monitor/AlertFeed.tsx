import React from 'react';
import { formatDistanceToNow } from 'date-fns';
import type { Alert } from '../../types';
import { SEVERITY_COLORS } from '../../types';

export interface AlertFeedProps {
  alerts: Alert[];
  onAcknowledge?: (alertId: string) => void;
  maxVisible?: number;
}

/**
 * Live feed of accessibility alerts
 */
export const AlertFeed: React.FC<AlertFeedProps> = ({
  alerts,
  onAcknowledge,
  maxVisible = 10,
}) => {
  const visibleAlerts = alerts.slice(0, maxVisible);
  const unacknowledgedCount = alerts.filter((a) => !a.acknowledged).length;

  return (
    <div className="alert-feed">
      <div className="feed-header">
        <h3>Alert Feed</h3>
        {unacknowledgedCount > 0 && (
          <span className="unacknowledged-badge">{unacknowledgedCount} new</span>
        )}
      </div>

      <div className="alerts-container">
        {visibleAlerts.length === 0 ? (
          <div className="empty-state">
            <p>No alerts</p>
          </div>
        ) : (
          visibleAlerts.map((alert) => (
            <div
              key={alert.id}
              className={`alert-item ${alert.acknowledged ? 'acknowledged' : 'unacknowledged'}`}
            >
              <div className="alert-header">
                <div
                  className="severity-indicator"
                  style={{
                    backgroundColor: SEVERITY_COLORS[alert.severity],
                  }}
                />
                <div className="alert-title-section">
                  <h4 className="alert-title">{alert.title}</h4>
                  <span className="alert-time">
                    {formatDistanceToNow(new Date(alert.created_at), { addSuffix: true })}
                  </span>
                </div>
              </div>

              <p className="alert-message">{alert.message}</p>

              {alert.issues.length > 0 && (
                <div className="alert-stats">
                  <span>{alert.issues.length} issues detected</span>
                </div>
              )}

              <div className="alert-actions">
                {!alert.acknowledged && onAcknowledge && (
                  <button
                    className="acknowledge-btn"
                    onClick={() => onAcknowledge(alert.id)}
                  >
                    Acknowledge
                  </button>
                )}
                {alert.acknowledged && (
                  <span className="acknowledged-label">
                    Acknowledged by {alert.acknowledged_by}
                  </span>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <style>{`
        .alert-feed {
          background: #ffffff;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          overflow: hidden;
        }

        .feed-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 20px;
          border-bottom: 1px solid #e5e7eb;
          background: #f9fafb;
        }

        .feed-header h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .unacknowledged-badge {
          padding: 4px 12px;
          background: #dc2626;
          color: white;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
        }

        .alerts-container {
          max-height: 600px;
          overflow-y: auto;
        }

        .empty-state {
          padding: 48px;
          text-align: center;
          color: #6b7280;
        }

        .alert-item {
          padding: 16px 20px;
          border-bottom: 1px solid #e5e7eb;
          transition: background-color 0.2s;
        }

        .alert-item:hover {
          background: #f9fafb;
        }

        .alert-item.unacknowledged {
          background: #fef3c7;
        }

        .alert-item.acknowledged {
          opacity: 0.7;
        }

        .alert-header {
          display: flex;
          align-items: flex-start;
          gap: 12px;
          margin-bottom: 8px;
        }

        .severity-indicator {
          width: 4px;
          height: 40px;
          border-radius: 2px;
          flex-shrink: 0;
        }

        .alert-title-section {
          flex: 1;
        }

        .alert-title {
          margin: 0 0 4px 0;
          font-size: 15px;
          font-weight: 600;
          color: #111827;
        }

        .alert-time {
          font-size: 12px;
          color: #6b7280;
        }

        .alert-message {
          margin: 0 0 12px 16px;
          font-size: 14px;
          line-height: 1.5;
          color: #4b5563;
          white-space: pre-wrap;
        }

        .alert-stats {
          margin: 12px 0 12px 16px;
          font-size: 13px;
          color: #6b7280;
        }

        .alert-actions {
          margin-left: 16px;
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .acknowledge-btn {
          padding: 6px 16px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 4px;
          font-size: 13px;
          font-weight: 500;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .acknowledge-btn:hover {
          background: #2563eb;
        }

        .acknowledged-label {
          font-size: 12px;
          color: #6b7280;
          font-style: italic;
        }
      `}</style>
    </div>
  );
};
