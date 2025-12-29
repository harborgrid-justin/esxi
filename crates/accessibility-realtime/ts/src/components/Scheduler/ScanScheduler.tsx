import React, { useState } from 'react';
import { format } from 'date-fns';
import type { ScanSchedule } from '../../types';

export interface ScanSchedulerProps {
  schedules: ScanSchedule[];
  onAdd?: (schedule: ScanSchedule) => void;
  onUpdate?: (schedule: ScanSchedule) => void;
  onDelete?: (scheduleId: string) => void;
  onToggle?: (scheduleId: string, enabled: boolean) => void;
  onTrigger?: (scheduleId: string) => void;
}

/**
 * Scan scheduler configuration interface
 */
export const ScanScheduler: React.FC<ScanSchedulerProps> = ({
  schedules,
  onAdd,
  onUpdate,
  onDelete,
  onToggle,
  onTrigger,
}) => {
  const [showForm, setShowForm] = useState(false);

  const getCronDescription = (cron: string): string => {
    const descriptions: Record<string, string> = {
      '0 0 * * *': 'Daily at midnight',
      '0 */6 * * *': 'Every 6 hours',
      '0 9 * * 1': 'Every Monday at 9 AM',
      '0 0 * * 0': 'Every Sunday at midnight',
      '*/30 * * * *': 'Every 30 minutes',
    };
    return descriptions[cron] || cron;
  };

  return (
    <div className="scan-scheduler">
      <div className="scheduler-header">
        <h2>Scan Schedules</h2>
        <button className="add-btn" onClick={() => setShowForm(true)}>
          + Add Schedule
        </button>
      </div>

      <div className="schedules-list">
        {schedules.length === 0 ? (
          <div className="empty-state">
            <p>No scan schedules configured</p>
            <button className="add-first-btn" onClick={() => setShowForm(true)}>
              Create your first schedule
            </button>
          </div>
        ) : (
          schedules.map((schedule) => (
            <div key={schedule.id} className="schedule-card">
              <div className="schedule-header">
                <div className="schedule-info">
                  <h3>{schedule.name}</h3>
                  <span className={`status-badge ${schedule.enabled ? 'enabled' : 'disabled'}`}>
                    {schedule.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
                <div className="schedule-actions">
                  <label className="toggle-switch">
                    <input
                      type="checkbox"
                      checked={schedule.enabled}
                      onChange={(e) => onToggle?.(schedule.id, e.target.checked)}
                    />
                    <span className="slider"></span>
                  </label>
                  <button
                    className="trigger-btn"
                    onClick={() => onTrigger?.(schedule.id)}
                    disabled={!schedule.enabled}
                  >
                    Run Now
                  </button>
                  <button className="delete-btn" onClick={() => onDelete?.(schedule.id)}>
                    Delete
                  </button>
                </div>
              </div>

              <div className="schedule-details">
                <div className="detail-row">
                  <strong>Schedule:</strong>
                  <span>{getCronDescription(schedule.cron)}</span>
                </div>
                <div className="detail-row">
                  <strong>Targets:</strong>
                  <span>{schedule.config.targets.length} URLs</span>
                </div>
                {schedule.next_run && (
                  <div className="detail-row">
                    <strong>Next Run:</strong>
                    <span>{format(new Date(schedule.next_run), 'MMM d, yyyy h:mm a')}</span>
                  </div>
                )}
                {schedule.last_run && (
                  <div className="detail-row">
                    <strong>Last Run:</strong>
                    <span>{format(new Date(schedule.last_run), 'MMM d, yyyy h:mm a')}</span>
                  </div>
                )}
              </div>

              <div className="targets-preview">
                <strong>Targets:</strong>
                {schedule.config.targets.slice(0, 3).map((target, idx) => (
                  <div key={idx} className="target-url">
                    {target}
                  </div>
                ))}
                {schedule.config.targets.length > 3 && (
                  <div className="targets-more">
                    +{schedule.config.targets.length - 3} more
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <style>{`
        .scan-scheduler {
          padding: 24px;
        }

        .scheduler-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .scheduler-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #111827;
        }

        .add-btn {
          padding: 10px 20px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .add-btn:hover {
          background: #2563eb;
        }

        .schedules-list {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .empty-state {
          padding: 64px;
          text-align: center;
          color: #6b7280;
        }

        .add-first-btn {
          margin-top: 16px;
          padding: 10px 24px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
        }

        .schedule-card {
          padding: 20px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .schedule-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .schedule-info {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .schedule-info h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .status-badge {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 11px;
          font-weight: 600;
          text-transform: uppercase;
        }

        .status-badge.enabled {
          background: #d1fae5;
          color: #065f46;
        }

        .status-badge.disabled {
          background: #f3f4f6;
          color: #6b7280;
        }

        .schedule-actions {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .toggle-switch {
          position: relative;
          display: inline-block;
          width: 48px;
          height: 24px;
        }

        .toggle-switch input {
          opacity: 0;
          width: 0;
          height: 0;
        }

        .slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: #cbd5e1;
          transition: 0.3s;
          border-radius: 24px;
        }

        .slider:before {
          position: absolute;
          content: "";
          height: 18px;
          width: 18px;
          left: 3px;
          bottom: 3px;
          background-color: white;
          transition: 0.3s;
          border-radius: 50%;
        }

        input:checked + .slider {
          background-color: #3b82f6;
        }

        input:checked + .slider:before {
          transform: translateX(24px);
        }

        .trigger-btn,
        .delete-btn {
          padding: 6px 14px;
          border: 1px solid #d1d5db;
          border-radius: 4px;
          font-size: 13px;
          font-weight: 500;
          cursor: pointer;
          background: white;
          transition: all 0.2s;
        }

        .trigger-btn:hover:not(:disabled) {
          background: #f3f4f6;
        }

        .trigger-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .delete-btn {
          color: #dc2626;
          border-color: #fecaca;
        }

        .delete-btn:hover {
          background: #fee2e2;
        }

        .schedule-details {
          display: flex;
          flex-direction: column;
          gap: 8px;
          margin-bottom: 16px;
          padding: 12px;
          background: #f9fafb;
          border-radius: 6px;
        }

        .detail-row {
          display: flex;
          gap: 8px;
          font-size: 14px;
        }

        .detail-row strong {
          color: #374151;
          min-width: 100px;
        }

        .detail-row span {
          color: #6b7280;
        }

        .targets-preview {
          font-size: 13px;
        }

        .targets-preview strong {
          display: block;
          margin-bottom: 8px;
          color: #374151;
        }

        .target-url {
          padding: 6px 12px;
          background: #f3f4f6;
          border-radius: 4px;
          color: #4b5563;
          font-family: monospace;
          margin-bottom: 4px;
        }

        .targets-more {
          margin-top: 8px;
          color: #9ca3af;
          font-style: italic;
        }
      `}</style>
    </div>
  );
};
