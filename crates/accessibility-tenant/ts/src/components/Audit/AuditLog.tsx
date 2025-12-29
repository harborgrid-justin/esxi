/**
 * Audit Log Component
 * Display and filter audit log entries
 */

import React, { useState } from 'react';
import { AuditLog as AuditLogType, AuditEventType, ResourceType } from '../../types';
import { usePermissions } from '../../hooks/usePermissions';

interface AuditLogProps {
  logs: AuditLogType[];
  onRefresh?: () => void;
  className?: string;
}

export const AuditLog: React.FC<AuditLogProps> = ({ logs, onRefresh, className }) => {
  const [eventTypeFilter, setEventTypeFilter] = useState<string>('');
  const [resourceFilter, setResourceFilter] = useState<string>('');
  const [userFilter, setUserFilter] = useState<string>('');
  const { can } = usePermissions();

  const canViewAudit = can.viewAuditLogs();

  if (!canViewAudit) {
    return (
      <div className={className} role="alert">
        <p>You do not have permission to view audit logs</p>
      </div>
    );
  }

  const filteredLogs = logs.filter((log) => {
    const matchesEvent = !eventTypeFilter || log.eventType === eventTypeFilter;
    const matchesResource = !resourceFilter || log.resource === resourceFilter;
    const matchesUser =
      !userFilter || log.userEmail.toLowerCase().includes(userFilter.toLowerCase());
    return matchesEvent && matchesResource && matchesUser;
  });

  const getEventIcon = (eventType: AuditEventType): string => {
    if (eventType.includes('CREATED')) return '➕';
    if (eventType.includes('UPDATED')) return '✏️';
    if (eventType.includes('DELETED')) return '🗑️';
    if (eventType.includes('LOGIN')) return '🔐';
    if (eventType.includes('LOGOUT')) return '🚪';
    return '📝';
  };

  return (
    <div className={className}>
      <header className="audit-header">
        <div>
          <h1>Audit Log</h1>
          <p className="subtitle">
            {filteredLogs.length} event{filteredLogs.length !== 1 ? 's' : ''}
          </p>
        </div>
        {onRefresh && (
          <button type="button" onClick={onRefresh} className="btn btn-secondary">
            Refresh
          </button>
        )}
      </header>

      <div className="filters">
        <input
          type="search"
          value={userFilter}
          onChange={(e) => setUserFilter(e.target.value)}
          placeholder="Search by user..."
          className="filter-input"
          aria-label="Search by user"
        />
        <select
          value={eventTypeFilter}
          onChange={(e) => setEventTypeFilter(e.target.value)}
          className="filter-select"
          aria-label="Filter by event type"
        >
          <option value="">All Events</option>
          {Object.values(AuditEventType).map((type) => (
            <option key={type} value={type}>
              {type.replace(/_/g, ' ')}
            </option>
          ))}
        </select>
        <select
          value={resourceFilter}
          onChange={(e) => setResourceFilter(e.target.value)}
          className="filter-select"
          aria-label="Filter by resource"
        >
          <option value="">All Resources</option>
          {Object.values(ResourceType).map((type) => (
            <option key={type} value={type}>
              {type.replace(/_/g, ' ')}
            </option>
          ))}
        </select>
      </div>

      <div className="audit-list">
        {filteredLogs.map((log) => (
          <article key={log.id} className={`audit-entry ${log.success ? 'success' : 'failure'}`}>
            <div className="entry-icon" aria-hidden="true">
              {getEventIcon(log.eventType)}
            </div>
            <div className="entry-content">
              <div className="entry-header">
                <strong>{log.eventType.replace(/_/g, ' ')}</strong>
                <span className="entry-timestamp">
                  {new Date(log.timestamp).toLocaleString()}
                </span>
              </div>
              <div className="entry-details">
                <span className="user-info">
                  {log.userEmail} ({log.resource})
                </span>
                {log.resourceId && <span className="resource-id">ID: {log.resourceId}</span>}
              </div>
              {log.changes && Object.keys(log.changes).length > 0 && (
                <details className="entry-changes">
                  <summary>View Changes</summary>
                  <pre>{JSON.stringify(log.changes, null, 2)}</pre>
                </details>
              )}
              {!log.success && log.errorMessage && (
                <div className="error-info" role="alert">
                  Error: {log.errorMessage}
                </div>
              )}
              {log.ipAddress && (
                <div className="meta-info">
                  <span>IP: {log.ipAddress}</span>
                </div>
              )}
            </div>
          </article>
        ))}

        {filteredLogs.length === 0 && (
          <div className="empty-state">
            <p>No audit log entries found</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default AuditLog;
