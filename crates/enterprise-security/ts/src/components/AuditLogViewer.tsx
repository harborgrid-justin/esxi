/**
 * Audit Log Viewer - Audit Trail UI Component
 * View and filter audit logs with real-time updates
 */

import React, { useState, useEffect } from 'react';
import { auditTrail } from '../compliance/AuditTrail';
import { AuditLog, AuditEventType, AuditSeverity } from '../types';

export interface AuditLogViewerProps {
  userId?: string;
  limit?: number;
}

export const AuditLogViewer: React.FC<AuditLogViewerProps> = ({ userId, limit = 50 }) => {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [filterSeverity, setFilterSeverity] = useState<AuditSeverity | 'ALL'>('ALL');
  const [filterEvent, setFilterEvent] = useState<AuditEventType | 'ALL'>('ALL');

  useEffect(() => {
    const fetchLogs = () => {
      const filters: any = { limit };
      if (userId) filters.userId = userId;
      if (filterSeverity !== 'ALL') filters.severity = filterSeverity;
      if (filterEvent !== 'ALL') filters.eventType = filterEvent;

      const results = auditTrail.query(filters);
      setLogs(results);
    };

    fetchLogs();
  }, [userId, limit, filterSeverity, filterEvent]);

  return (
    <div className="audit-log-viewer">
      <div className="viewer-header">
        <h2>Audit Logs</h2>
        <div className="filters">
          <select value={filterSeverity} onChange={(e) => setFilterSeverity(e.target.value as any)}>
            <option value="ALL">All Severities</option>
            <option value="CRITICAL">Critical</option>
            <option value="HIGH">High</option>
            <option value="MEDIUM">Medium</option>
            <option value="LOW">Low</option>
            <option value="INFO">Info</option>
          </select>
        </div>
      </div>

      <div className="logs-table">
        <table>
          <thead>
            <tr>
              <th>Timestamp</th>
              <th>Event</th>
              <th>Severity</th>
              <th>User</th>
              <th>Resource</th>
              <th>Result</th>
            </tr>
          </thead>
          <tbody>
            {logs.map((log) => (
              <tr key={log.id} className={`severity-${log.severity.toLowerCase()}`}>
                <td>{log.timestamp.toLocaleString()}</td>
                <td>{log.eventType}</td>
                <td><span className="severity-badge">{log.severity}</span></td>
                <td>{log.username || log.userId || 'System'}</td>
                <td>{log.resource}</td>
                <td><span className={`result-${log.result.toLowerCase()}`}>{log.result}</span></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <style>{`
        .audit-log-viewer {
          background: #fff;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        .viewer-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px 24px;
          border-bottom: 1px solid #e0e0e0;
        }
        .logs-table {
          overflow-x: auto;
        }
        table {
          width: 100%;
          border-collapse: collapse;
        }
        th, td {
          padding: 12px 16px;
          text-align: left;
          border-bottom: 1px solid #e0e0e0;
        }
        th {
          background: #f5f5f5;
          font-weight: 600;
        }
        .severity-badge {
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
        }
        .severity-critical { background-color: #ffebee; }
        .severity-high { background-color: #fff3e0; }
        .result-success { color: #4caf50; }
        .result-failure { color: #f44336; }
      `}</style>
    </div>
  );
};
