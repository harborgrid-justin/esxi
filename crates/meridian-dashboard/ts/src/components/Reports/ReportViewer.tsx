/**
 * ReportViewer - Report viewing and management interface
 */

import React, { useState } from 'react';
import { Report } from '../../types';
import { format } from 'date-fns';

export interface ReportViewerProps {
  reports: Report[];
  onGenerate?: (reportId: string) => void;
  onEdit?: (reportId: string) => void;
  onDelete?: (reportId: string) => void;
}

export const ReportViewer: React.FC<ReportViewerProps> = ({
  reports,
  onGenerate,
  onEdit,
  onDelete,
}) => {
  const [selectedReport, setSelectedReport] = useState<Report | null>(null);

  const handleSelectReport = (report: Report) => {
    setSelectedReport(selectedReport?.id === report.id ? null : report);
  };

  return (
    <div className="report-viewer">
      <div className="reports-list">
        <h3>Reports</h3>
        {reports.length === 0 ? (
          <div className="empty-state">No reports configured</div>
        ) : (
          <div className="reports-grid">
            {reports.map((report) => (
              <div
                key={report.id}
                className={`report-card ${
                  selectedReport?.id === report.id ? 'selected' : ''
                }`}
                onClick={() => handleSelectReport(report)}
              >
                <div className="report-header">
                  <h4>{report.name}</h4>
                  <span className={`format-badge ${report.format}`}>
                    {report.format.toUpperCase()}
                  </span>
                </div>

                {report.description && (
                  <p className="report-description">{report.description}</p>
                )}

                <div className="report-meta">
                  <div className="meta-item">
                    <span className="meta-label">Recipients:</span>
                    <span className="meta-value">{report.recipients.length}</span>
                  </div>

                  {report.schedule && (
                    <div className="meta-item">
                      <span className="meta-label">Schedule:</span>
                      <span className="meta-value">
                        {report.schedule.enabled ? '✓ Enabled' : '✗ Disabled'}
                      </span>
                    </div>
                  )}

                  <div className="meta-item">
                    <span className="meta-label">Created:</span>
                    <span className="meta-value">
                      {format(new Date(report.created_at), 'MMM dd, yyyy')}
                    </span>
                  </div>
                </div>

                <div className="report-actions">
                  {onGenerate && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onGenerate(report.id);
                      }}
                      className="btn-action primary"
                    >
                      Generate
                    </button>
                  )}
                  {onEdit && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onEdit(report.id);
                      }}
                      className="btn-action"
                    >
                      Edit
                    </button>
                  )}
                  {onDelete && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        if (confirm('Delete this report?')) {
                          onDelete(report.id);
                        }
                      }}
                      className="btn-action danger"
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {selectedReport && (
        <div className="report-details">
          <h3>Report Details</h3>
          <div className="details-grid">
            <div className="detail-item">
              <strong>Name:</strong>
              <span>{selectedReport.name}</span>
            </div>

            <div className="detail-item">
              <strong>Format:</strong>
              <span>{selectedReport.format.toUpperCase()}</span>
            </div>

            <div className="detail-item">
              <strong>Recipients:</strong>
              <ul>
                {selectedReport.recipients.map((email, index) => (
                  <li key={index}>{email}</li>
                ))}
              </ul>
            </div>

            {selectedReport.schedule && (
              <>
                <div className="detail-item">
                  <strong>Schedule:</strong>
                  <span>{selectedReport.schedule.cron}</span>
                </div>

                <div className="detail-item">
                  <strong>Timezone:</strong>
                  <span>{selectedReport.schedule.timezone}</span>
                </div>

                <div className="detail-item">
                  <strong>Status:</strong>
                  <span>
                    {selectedReport.schedule.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      <style jsx>{`
        .report-viewer {
          display: grid;
          grid-template-columns: 2fr 1fr;
          gap: 20px;
          padding: 20px;
        }

        .reports-list h3,
        .report-details h3 {
          margin: 0 0 16px 0;
          font-size: 18px;
          color: #333;
        }

        .empty-state {
          padding: 40px;
          text-align: center;
          color: #999;
        }

        .reports-grid {
          display: grid;
          gap: 16px;
        }

        .report-card {
          background: white;
          border: 2px solid #e0e0e0;
          border-radius: 8px;
          padding: 16px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .report-card:hover {
          border-color: #1976d2;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .report-card.selected {
          border-color: #1976d2;
          background: #f5f9ff;
        }

        .report-header {
          display: flex;
          justify-content: space-between;
          align-items: start;
          margin-bottom: 8px;
        }

        .report-header h4 {
          margin: 0;
          font-size: 16px;
          color: #333;
        }

        .format-badge {
          padding: 4px 8px;
          border-radius: 4px;
          font-size: 11px;
          font-weight: 600;
          color: white;
        }

        .format-badge.pdf {
          background: #f44336;
        }

        .format-badge.excel {
          background: #4caf50;
        }

        .format-badge.csv {
          background: #ff9800;
        }

        .format-badge.json {
          background: #2196f3;
        }

        .report-description {
          margin: 0 0 12px 0;
          font-size: 13px;
          color: #666;
        }

        .report-meta {
          display: flex;
          flex-wrap: wrap;
          gap: 16px;
          margin-bottom: 12px;
          font-size: 12px;
        }

        .meta-item {
          display: flex;
          gap: 4px;
        }

        .meta-label {
          color: #999;
        }

        .meta-value {
          color: #333;
          font-weight: 600;
        }

        .report-actions {
          display: flex;
          gap: 8px;
          padding-top: 12px;
          border-top: 1px solid #e0e0e0;
        }

        .btn-action {
          padding: 6px 12px;
          border: 1px solid #ddd;
          background: white;
          border-radius: 4px;
          font-size: 13px;
          cursor: pointer;
        }

        .btn-action.primary {
          background: #1976d2;
          color: white;
          border-color: #1976d2;
        }

        .btn-action.danger {
          color: #f44336;
          border-color: #f44336;
        }

        .report-details {
          background: white;
          border-radius: 8px;
          padding: 16px;
          height: fit-content;
        }

        .details-grid {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .detail-item {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .detail-item strong {
          font-size: 12px;
          color: #666;
          text-transform: uppercase;
        }

        .detail-item ul {
          margin: 0;
          padding-left: 20px;
        }

        .detail-item li {
          font-size: 13px;
          color: #333;
        }

        @media (max-width: 768px) {
          .report-viewer {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
