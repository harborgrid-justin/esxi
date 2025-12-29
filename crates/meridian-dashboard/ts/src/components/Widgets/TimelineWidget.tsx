/**
 * TimelineWidget - Timeline visualization widget
 */

import React from 'react';
import { Widget, TimelineWidgetConfig } from '../../types';
import { useDataSource } from '../../hooks/useDataSource';
import { format } from 'date-fns';

export interface TimelineWidgetProps {
  widget: Widget;
}

export const TimelineWidget: React.FC<TimelineWidgetProps> = ({ widget }) => {
  const { data, loading, error } = useDataSource(widget.data_source);
  const config = widget.config as TimelineWidgetConfig;

  const sortedData = React.useMemo(() => {
    if (!Array.isArray(data)) return [];
    return [...data].sort((a, b) => {
      const dateA = new Date(a[config.date_field]);
      const dateB = new Date(b[config.date_field]);
      return dateB.getTime() - dateA.getTime();
    });
  }, [data, config.date_field]);

  if (loading) {
    return (
      <div className="timeline-loading">
        <div className="spinner"></div>
        <p>Loading timeline...</p>
      </div>
    );
  }

  if (error) {
    return <div className="timeline-error">Error: {error.message}</div>;
  }

  return (
    <div className="timeline-widget">
      <div className="timeline-container">
        {sortedData.map((item, index) => {
          const date = new Date(item[config.date_field]);
          const title = item[config.title_field];
          const description = config.description_field
            ? item[config.description_field]
            : null;

          return (
            <div key={index} className="timeline-item">
              <div className="timeline-marker">
                <div className="timeline-dot"></div>
                {index < sortedData.length - 1 && (
                  <div className="timeline-line"></div>
                )}
              </div>
              <div className="timeline-content">
                <div className="timeline-date">
                  {format(date, 'MMM dd, yyyy HH:mm')}
                </div>
                <div className="timeline-title">{title}</div>
                {description && (
                  <div className="timeline-description">{description}</div>
                )}
              </div>
            </div>
          );
        })}
      </div>

      <style jsx>{`
        .timeline-widget {
          width: 100%;
          height: 100%;
          overflow: auto;
        }

        .timeline-container {
          padding: 8px 0;
        }

        .timeline-item {
          display: flex;
          gap: 16px;
          position: relative;
        }

        .timeline-marker {
          display: flex;
          flex-direction: column;
          align-items: center;
          position: relative;
        }

        .timeline-dot {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          background: #1976d2;
          border: 3px solid white;
          box-shadow: 0 0 0 2px #1976d2;
          position: relative;
          z-index: 2;
        }

        .timeline-line {
          width: 2px;
          flex: 1;
          min-height: 40px;
          background: #e0e0e0;
          margin-top: 4px;
        }

        .timeline-content {
          flex: 1;
          padding-bottom: 24px;
        }

        .timeline-date {
          font-size: 12px;
          color: #999;
          margin-bottom: 4px;
        }

        .timeline-title {
          font-size: 14px;
          font-weight: 600;
          color: #333;
          margin-bottom: 4px;
        }

        .timeline-description {
          font-size: 13px;
          color: #666;
          line-height: 1.5;
        }

        .timeline-loading,
        .timeline-error {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: #666;
        }

        .spinner {
          border: 3px solid #f3f3f3;
          border-top: 3px solid #1976d2;
          border-radius: 50%;
          width: 40px;
          height: 40px;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% {
            transform: rotate(0deg);
          }
          100% {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </div>
  );
};
