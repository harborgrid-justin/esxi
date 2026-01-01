/**
 * AlertDashboard - Alert overview and management dashboard
 */

import React, { useState } from 'react';
import { Alert, AlertStatus, AlertSeverity } from '../types';

export interface AlertDashboardProps {
  alerts: Alert[];
  onAcknowledge?: (alertId: string) => void;
  onResolve?: (alertId: string) => void;
  onAssign?: (alertId: string, userId: string) => void;
  className?: string;
}

export const AlertDashboard: React.FC<AlertDashboardProps> = ({
  alerts,
  onAcknowledge,
  onResolve,
  onAssign,
  className = '',
}) => {
  const [filter, setFilter] = useState<AlertStatus | 'all'>('all');
  const [severityFilter, setSeverityFilter] = useState<AlertSeverity | 'all'>('all');

  const filtered = alerts.filter(alert => {
    if (filter !== 'all' && alert.status !== filter) return false;
    if (severityFilter !== 'all' && alert.severity !== severityFilter) return false;
    return true;
  });

  const stats = {
    total: alerts.length,
    open: alerts.filter(a => a.status === AlertStatus.OPEN).length,
    acknowledged: alerts.filter(a => a.status === AlertStatus.ACKNOWLEDGED).length,
    resolved: alerts.filter(a => a.status === AlertStatus.RESOLVED).length,
    critical: alerts.filter(a => a.severity === AlertSeverity.CRITICAL).length,
  };

  return (
    <div className={`alert-dashboard ${className}`}>
      {/* Stats Cards */}
      <div className="alert-stats">
        <div className="stat-card">
          <h3>{stats.total}</h3>
          <p>Total Alerts</p>
        </div>
        <div className="stat-card critical">
          <h3>{stats.critical}</h3>
          <p>Critical</p>
        </div>
        <div className="stat-card">
          <h3>{stats.open}</h3>
          <p>Open</p>
        </div>
        <div className="stat-card">
          <h3>{stats.acknowledged}</h3>
          <p>Acknowledged</p>
        </div>
      </div>

      {/* Filters */}
      <div className="alert-filters">
        <select value={filter} onChange={e => setFilter(e.target.value as AlertStatus | 'all')}>
          <option value="all">All Status</option>
          <option value={AlertStatus.OPEN}>Open</option>
          <option value={AlertStatus.ACKNOWLEDGED}>Acknowledged</option>
          <option value={AlertStatus.IN_PROGRESS}>In Progress</option>
          <option value={AlertStatus.RESOLVED}>Resolved</option>
        </select>

        <select
          value={severityFilter}
          onChange={e => setSeverityFilter(e.target.value as AlertSeverity | 'all')}
        >
          <option value="all">All Severities</option>
          <option value={AlertSeverity.CRITICAL}>Critical</option>
          <option value={AlertSeverity.ERROR}>Error</option>
          <option value={AlertSeverity.WARNING}>Warning</option>
          <option value={AlertSeverity.INFO}>Info</option>
        </select>
      </div>

      {/* Alert List */}
      <div className="alert-list">
        {filtered.map(alert => (
          <div key={alert.id} className={`alert-item severity-${alert.severity}`}>
            <div className="alert-header">
              <h4>{alert.name}</h4>
              <span className={`status-badge ${alert.status}`}>{alert.status}</span>
            </div>

            <p className="alert-message">{alert.message}</p>

            <div className="alert-metadata">
              <span>Source: {alert.source}</span>
              <span>Count: {alert.count}</span>
              <span>First seen: {new Date(alert.firstOccurrenceAt).toLocaleString()}</span>
            </div>

            <div className="alert-actions">
              {alert.status === AlertStatus.OPEN && onAcknowledge && (
                <button onClick={() => onAcknowledge(alert.id)}>Acknowledge</button>
              )}
              {alert.status !== AlertStatus.RESOLVED && onResolve && (
                <button onClick={() => onResolve(alert.id)}>Resolve</button>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default AlertDashboard;
