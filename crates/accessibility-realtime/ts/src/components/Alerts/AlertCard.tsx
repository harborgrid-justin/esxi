import React from 'react';
import { formatDistanceToNow, format } from 'date-fns';
import type { Alert } from '../../types';
import { SEVERITY_COLORS } from '../../types';

export interface AlertCardProps {
  alert: Alert;
  onAcknowledge?: (alertId: string) => void;
  onDismiss?: (alertId: string) => void;
  expanded?: boolean;
}

/**
 * Alert notification card component
 */
export const AlertCard: React.FC<AlertCardProps> = ({
  alert,
  onAcknowledge,
  onDismiss,
  expanded = false,
}) => {
  const [isExpanded, setIsExpanded] = React.useState(expanded);

  return (
    <div className={`alert-card severity-${alert.severity} ${alert.acknowledged ? 'acknowledged' : ''}`}>
      <div className="card-header">
        <div
          className="severity-badge"
          style={{ backgroundColor: SEVERITY_COLORS[alert.severity] }}
        >
          {alert.severity}
        </div>
        <div className="timestamp">
          {formatDistanceToNow(new Date(alert.created_at), { addSuffix: true })}
        </div>
      </div>

      <h3 className="alert-title">{alert.title}</h3>

      <p className="alert-message">{alert.message}</p>

      {alert.issues.length > 0 && (
        <div className="issue-summary">
          <strong>{alert.issues.length}</strong> accessibility{' '}
          {alert.issues.length === 1 ? 'issue' : 'issues'} detected
        </div>
      )}

      {isExpanded && alert.issues.length > 0 && (
        <div className="issues-list">
          <h4>Issues:</h4>
          <ul>
            {alert.issues.slice(0, 5).map((issue) => (
              <li key={issue.id}>
                <strong>{issue.rule_name}</strong>
                <span className="issue-page">{issue.page_url}</span>
              </li>
            ))}
            {alert.issues.length > 5 && (
              <li className="more-issues">+{alert.issues.length - 5} more issues</li>
            )}
          </ul>
        </div>
      )}

      <div className="card-actions">
        {alert.issues.length > 0 && (
          <button
            className="expand-btn"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            {isExpanded ? 'Show Less' : 'Show Details'}
          </button>
        )}

        {!alert.acknowledged && onAcknowledge && (
          <button
            className="acknowledge-btn"
            onClick={() => onAcknowledge(alert.id)}
          >
            Acknowledge
          </button>
        )}

        {onDismiss && (
          <button
            className="dismiss-btn"
            onClick={() => onDismiss(alert.id)}
          >
            Dismiss
          </button>
        )}
      </div>

      {alert.acknowledged && (
        <div className="acknowledged-footer">
          <span>
            Acknowledged by {alert.acknowledged_by} •{' '}
            {format(new Date(alert.acknowledged_at!), 'MMM d, yyyy h:mm a')}
          </span>
        </div>
      )}

      <style>{`
        .alert-card {
          background: white;
          border-radius: 8px;
          padding: 20px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          border-left: 4px solid;
          transition: all 0.2s;
        }

        .alert-card.severity-critical {
          border-left-color: #dc2626;
        }

        .alert-card.severity-high {
          border-left-color: #ea580c;
        }

        .alert-card.severity-medium {
          border-left-color: #f59e0b;
        }

        .alert-card.severity-low {
          border-left-color: #3b82f6;
        }

        .alert-card.severity-info {
          border-left-color: #6b7280;
        }

        .alert-card.acknowledged {
          opacity: 0.7;
          background: #f9fafb;
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .severity-badge {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 11px;
          font-weight: 700;
          text-transform: uppercase;
          color: white;
          letter-spacing: 0.5px;
        }

        .timestamp {
          font-size: 12px;
          color: #6b7280;
        }

        .alert-title {
          margin: 0 0 12px 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .alert-message {
          margin: 0 0 16px 0;
          font-size: 14px;
          line-height: 1.6;
          color: #4b5563;
          white-space: pre-wrap;
        }

        .issue-summary {
          padding: 12px;
          background: #f3f4f6;
          border-radius: 4px;
          font-size: 14px;
          color: #374151;
          margin-bottom: 16px;
        }

        .issues-list {
          margin: 16px 0;
          padding: 16px;
          background: #f9fafb;
          border-radius: 6px;
        }

        .issues-list h4 {
          margin: 0 0 12px 0;
          font-size: 14px;
          font-weight: 600;
          color: #111827;
        }

        .issues-list ul {
          margin: 0;
          padding: 0;
          list-style: none;
        }

        .issues-list li {
          padding: 8px 0;
          border-bottom: 1px solid #e5e7eb;
          font-size: 13px;
        }

        .issues-list li:last-child {
          border-bottom: none;
        }

        .issue-page {
          display: block;
          color: #6b7280;
          font-size: 12px;
          margin-top: 4px;
        }

        .more-issues {
          color: #6b7280;
          font-style: italic;
        }

        .card-actions {
          display: flex;
          gap: 8px;
          margin-top: 16px;
        }

        .expand-btn,
        .acknowledge-btn,
        .dismiss-btn {
          padding: 8px 16px;
          border: none;
          border-radius: 4px;
          font-size: 13px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .expand-btn {
          background: #f3f4f6;
          color: #374151;
        }

        .expand-btn:hover {
          background: #e5e7eb;
        }

        .acknowledge-btn {
          background: #3b82f6;
          color: white;
        }

        .acknowledge-btn:hover {
          background: #2563eb;
        }

        .dismiss-btn {
          background: transparent;
          color: #6b7280;
        }

        .dismiss-btn:hover {
          background: #f3f4f6;
        }

        .acknowledged-footer {
          margin-top: 16px;
          padding-top: 16px;
          border-top: 1px solid #e5e7eb;
          font-size: 12px;
          color: #6b7280;
          font-style: italic;
        }
      `}</style>
    </div>
  );
};
