import React, { useState } from 'react';
import { format } from 'date-fns';
import type { Alert, Severity } from '../../types';
import { AlertCard } from './AlertCard';

export interface AlertHistoryProps {
  alerts: Alert[];
  onAcknowledge?: (alertId: string) => void;
  onDismiss?: (alertId: string) => void;
}

type FilterType = 'all' | 'acknowledged' | 'unacknowledged';

/**
 * Alert history with filtering and search
 */
export const AlertHistory: React.FC<AlertHistoryProps> = ({
  alerts,
  onAcknowledge,
  onDismiss,
}) => {
  const [filter, setFilter] = useState<FilterType>('all');
  const [severityFilter, setSeverityFilter] = useState<Severity | 'all'>('all');
  const [searchTerm, setSearchTerm] = useState('');

  const filteredAlerts = alerts.filter((alert) => {
    // Filter by acknowledgement status
    if (filter === 'acknowledged' && !alert.acknowledged) return false;
    if (filter === 'unacknowledged' && alert.acknowledged) return false;

    // Filter by severity
    if (severityFilter !== 'all' && alert.severity !== severityFilter) return false;

    // Filter by search term
    if (searchTerm) {
      const search = searchTerm.toLowerCase();
      return (
        alert.title.toLowerCase().includes(search) ||
        alert.message.toLowerCase().includes(search)
      );
    }

    return true;
  });

  const stats = {
    total: alerts.length,
    acknowledged: alerts.filter((a) => a.acknowledged).length,
    unacknowledged: alerts.filter((a) => !a.acknowledged).length,
  };

  return (
    <div className="alert-history">
      <div className="history-header">
        <h2>Alert History</h2>
        <div className="stats-badges">
          <span className="stat-badge">Total: {stats.total}</span>
          <span className="stat-badge unack">Unacknowledged: {stats.unacknowledged}</span>
          <span className="stat-badge ack">Acknowledged: {stats.acknowledged}</span>
        </div>
      </div>

      <div className="filters">
        <div className="filter-group">
          <label>Status:</label>
          <select value={filter} onChange={(e) => setFilter(e.target.value as FilterType)}>
            <option value="all">All</option>
            <option value="unacknowledged">Unacknowledged</option>
            <option value="acknowledged">Acknowledged</option>
          </select>
        </div>

        <div className="filter-group">
          <label>Severity:</label>
          <select
            value={severityFilter}
            onChange={(e) => setSeverityFilter(e.target.value as Severity | 'all')}
          >
            <option value="all">All</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
            <option value="info">Info</option>
          </select>
        </div>

        <div className="search-group">
          <input
            type="text"
            placeholder="Search alerts..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>

      <div className="alerts-list">
        {filteredAlerts.length === 0 ? (
          <div className="empty-state">
            <p>No alerts found matching your filters</p>
          </div>
        ) : (
          filteredAlerts.map((alert) => (
            <AlertCard
              key={alert.id}
              alert={alert}
              onAcknowledge={onAcknowledge}
              onDismiss={onDismiss}
            />
          ))
        )}
      </div>

      <style>{`
        .alert-history {
          padding: 24px;
        }

        .history-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .history-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #111827;
        }

        .stats-badges {
          display: flex;
          gap: 12px;
        }

        .stat-badge {
          padding: 6px 14px;
          background: #f3f4f6;
          border-radius: 16px;
          font-size: 13px;
          font-weight: 500;
          color: #374151;
        }

        .stat-badge.unack {
          background: #fee2e2;
          color: #991b1b;
        }

        .stat-badge.ack {
          background: #d1fae5;
          color: #065f46;
        }

        .filters {
          display: flex;
          gap: 16px;
          margin-bottom: 24px;
          padding: 16px;
          background: #f9fafb;
          border-radius: 8px;
        }

        .filter-group {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .filter-group label {
          font-size: 14px;
          font-weight: 500;
          color: #374151;
        }

        .filter-group select {
          padding: 6px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
          background: white;
          cursor: pointer;
        }

        .search-group {
          flex: 1;
          display: flex;
          justify-content: flex-end;
        }

        .search-group input {
          width: 100%;
          max-width: 300px;
          padding: 6px 12px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 14px;
        }

        .alerts-list {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .empty-state {
          padding: 64px;
          text-align: center;
          color: #6b7280;
        }
      `}</style>
    </div>
  );
};
