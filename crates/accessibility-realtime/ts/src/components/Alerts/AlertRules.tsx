import React, { useState } from 'react';
import type { AlertConfig, Severity, AlertChannel } from '../../types';

export interface AlertRulesProps {
  configs: AlertConfig[];
  onAdd?: (config: AlertConfig) => void;
  onUpdate?: (config: AlertConfig) => void;
  onDelete?: (configId: string) => void;
  onToggle?: (configId: string, enabled: boolean) => void;
}

/**
 * Alert rule configuration interface
 */
export const AlertRules: React.FC<AlertRulesProps> = ({
  configs,
  onAdd,
  onUpdate,
  onDelete,
  onToggle,
}) => {
  const [editing, setEditing] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);

  return (
    <div className="alert-rules">
      <div className="rules-header">
        <h2>Alert Rules</h2>
        <button className="add-btn" onClick={() => setShowForm(true)}>
          + Add Rule
        </button>
      </div>

      <div className="rules-list">
        {configs.length === 0 ? (
          <div className="empty-state">
            <p>No alert rules configured</p>
            <button className="add-first-btn" onClick={() => setShowForm(true)}>
              Create your first rule
            </button>
          </div>
        ) : (
          configs.map((config) => (
            <div key={config.id} className="rule-card">
              <div className="rule-header">
                <div className="rule-info">
                  <h3>{config.name}</h3>
                  <span className={`status-badge ${config.enabled ? 'enabled' : 'disabled'}`}>
                    {config.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
                <div className="rule-actions">
                  <label className="toggle-switch">
                    <input
                      type="checkbox"
                      checked={config.enabled}
                      onChange={(e) => onToggle?.(config.id, e.target.checked)}
                    />
                    <span className="slider"></span>
                  </label>
                  <button
                    className="edit-btn"
                    onClick={() => setEditing(config.id)}
                  >
                    Edit
                  </button>
                  <button
                    className="delete-btn"
                    onClick={() => onDelete?.(config.id)}
                  >
                    Delete
                  </button>
                </div>
              </div>

              <div className="rule-details">
                <div className="detail-item">
                  <strong>Min Severity:</strong> {config.conditions.min_severity}
                </div>
                {config.conditions.issue_threshold && (
                  <div className="detail-item">
                    <strong>Issue Threshold:</strong> {config.conditions.issue_threshold}
                  </div>
                )}
                {config.throttle_minutes && (
                  <div className="detail-item">
                    <strong>Throttle:</strong> {config.throttle_minutes} minutes
                  </div>
                )}
                <div className="detail-item">
                  <strong>Channels:</strong> {config.channels.length} configured
                </div>
              </div>

              <div className="channels-list">
                {config.channels.map((channel, idx) => (
                  <span key={idx} className="channel-badge">
                    {channel.type}
                  </span>
                ))}
              </div>
            </div>
          ))
        )}
      </div>

      <style>{`
        .alert-rules {
          padding: 24px;
        }

        .rules-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .rules-header h2 {
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

        .rules-list {
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

        .rule-card {
          padding: 20px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .rule-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .rule-info {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .rule-info h3 {
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

        .rule-actions {
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

        .edit-btn,
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

        .edit-btn:hover {
          background: #f3f4f6;
        }

        .delete-btn {
          color: #dc2626;
          border-color: #fecaca;
        }

        .delete-btn:hover {
          background: #fee2e2;
        }

        .rule-details {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 12px;
          margin-bottom: 16px;
        }

        .detail-item {
          font-size: 14px;
          color: #4b5563;
        }

        .detail-item strong {
          color: #111827;
          margin-right: 4px;
        }

        .channels-list {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }

        .channel-badge {
          padding: 4px 10px;
          background: #eff6ff;
          color: #1e40af;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 500;
          text-transform: capitalize;
        }
      `}</style>
    </div>
  );
};
