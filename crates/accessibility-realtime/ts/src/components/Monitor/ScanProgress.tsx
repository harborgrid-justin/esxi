import React from 'react';
import { formatDistanceToNow } from 'date-fns';
import type { ScanContext } from '../../types';

export interface ScanProgressProps {
  scanId: string;
  context: ScanContext;
}

/**
 * Display progress of an active scan
 */
export const ScanProgress: React.FC<ScanProgressProps> = ({ scanId, context }) => {
  const percentage =
    context.total_pages > 0 ? (context.pages_scanned / context.total_pages) * 100 : 0;

  const timeElapsed = formatDistanceToNow(new Date(context.started_at), {
    addSuffix: false,
  });

  return (
    <div className="scan-progress">
      <div className="scan-header">
        <div className="scan-info">
          <h4 className="scan-name">{context.config.name}</h4>
          <span className="scan-meta">
            {context.pages_scanned} / {context.total_pages} pages • {timeElapsed}
          </span>
        </div>
        <div className="scan-stats">
          <span className="issues-count">{context.issues.length} issues</span>
          <span className="scan-percentage">{percentage.toFixed(0)}%</span>
        </div>
      </div>

      <div className="progress-bar">
        <div
          className="progress-fill"
          style={{
            width: `${percentage}%`,
          }}
        />
      </div>

      <div className="scan-targets">
        <span className="targets-label">Targets:</span>
        <span className="targets-list">{context.config.targets.slice(0, 3).join(', ')}</span>
        {context.config.targets.length > 3 && (
          <span className="targets-more">+{context.config.targets.length - 3} more</span>
        )}
      </div>

      <style>{`
        .scan-progress {
          padding: 16px;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
        }

        .scan-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 12px;
        }

        .scan-info {
          flex: 1;
        }

        .scan-name {
          margin: 0 0 4px 0;
          font-size: 15px;
          font-weight: 600;
          color: #111827;
        }

        .scan-meta {
          font-size: 13px;
          color: #6b7280;
        }

        .scan-stats {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .issues-count {
          padding: 4px 10px;
          background: #fef3c7;
          color: #92400e;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
        }

        .scan-percentage {
          font-size: 16px;
          font-weight: 700;
          color: #3b82f6;
        }

        .progress-bar {
          height: 6px;
          background: #e5e7eb;
          border-radius: 3px;
          overflow: hidden;
          margin-bottom: 12px;
        }

        .progress-fill {
          height: 100%;
          background: linear-gradient(90deg, #3b82f6, #2563eb);
          transition: width 0.3s ease;
          border-radius: 3px;
        }

        .scan-targets {
          font-size: 12px;
          color: #6b7280;
        }

        .targets-label {
          font-weight: 600;
          margin-right: 6px;
        }

        .targets-list {
          margin-right: 6px;
        }

        .targets-more {
          color: #9ca3af;
          font-style: italic;
        }
      `}</style>
    </div>
  );
};
