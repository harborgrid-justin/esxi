/**
 * DashboardToolbar - Dashboard control toolbar
 */

import React, { useState } from 'react';
import { Dashboard } from '../../types';

export interface DashboardToolbarProps {
  dashboard: Dashboard;
  editable?: boolean;
  onToggleEdit?: () => void;
  onAddWidget?: () => void;
  onExport?: () => void;
  onShare?: () => void;
  onRefresh?: () => void;
  onSave?: () => void;
}

export const DashboardToolbar: React.FC<DashboardToolbarProps> = ({
  dashboard,
  editable = false,
  onToggleEdit,
  onAddWidget,
  onExport,
  onShare,
  onRefresh,
  onSave,
}) => {
  const [showFilters, setShowFilters] = useState(false);

  return (
    <div className="dashboard-toolbar">
      <div className="toolbar-left">
        <h1 className="dashboard-name">{dashboard.name}</h1>
        {dashboard.description && (
          <p className="dashboard-description">{dashboard.description}</p>
        )}
      </div>

      <div className="toolbar-right">
        {editable && onAddWidget && (
          <button className="toolbar-btn primary" onClick={onAddWidget}>
            + Add Widget
          </button>
        )}

        {onRefresh && (
          <button className="toolbar-btn" onClick={onRefresh} title="Refresh dashboard">
            ↻ Refresh
          </button>
        )}

        <button
          className="toolbar-btn"
          onClick={() => setShowFilters(!showFilters)}
          title="Toggle filters"
        >
          ⚙ Filters
        </button>

        {onExport && (
          <button className="toolbar-btn" onClick={onExport} title="Export dashboard">
            ↓ Export
          </button>
        )}

        {onShare && (
          <button className="toolbar-btn" onClick={onShare} title="Share dashboard">
            ⚑ Share
          </button>
        )}

        {onToggleEdit && (
          <button
            className={`toolbar-btn ${editable ? 'active' : ''}`}
            onClick={onToggleEdit}
            title={editable ? 'Exit edit mode' : 'Enter edit mode'}
          >
            {editable ? '✓ Done' : '✎ Edit'}
          </button>
        )}

        {editable && onSave && (
          <button className="toolbar-btn primary" onClick={onSave}>
            💾 Save
          </button>
        )}
      </div>

      {showFilters && dashboard.filters.length > 0 && (
        <div className="filters-panel">
          <h3>Active Filters</h3>
          <div className="filters-list">
            {dashboard.filters.map((filter) => (
              <div key={filter.id} className="filter-item">
                <span className="filter-field">{filter.field}</span>
                <span className="filter-operator">{filter.operator}</span>
                <span className="filter-value">
                  {JSON.stringify(filter.value)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      <style jsx>{`
        .dashboard-toolbar {
          background: white;
          border-bottom: 1px solid #e0e0e0;
          padding: 16px 24px;
        }

        .toolbar-left {
          display: inline-block;
          vertical-align: middle;
          margin-right: auto;
        }

        .dashboard-name {
          margin: 0;
          font-size: 24px;
          font-weight: 600;
          color: #333;
        }

        .dashboard-description {
          margin: 4px 0 0 0;
          font-size: 14px;
          color: #666;
        }

        .toolbar-right {
          display: inline-block;
          vertical-align: middle;
          float: right;
          gap: 8px;
        }

        .toolbar-btn {
          padding: 8px 16px;
          border: 1px solid #ddd;
          background: white;
          border-radius: 4px;
          font-size: 14px;
          cursor: pointer;
          transition: all 0.2s;
          margin-left: 8px;
        }

        .toolbar-btn:hover {
          background: #f5f5f5;
          border-color: #999;
        }

        .toolbar-btn.primary {
          background: #1976d2;
          color: white;
          border-color: #1976d2;
        }

        .toolbar-btn.primary:hover {
          background: #1565c0;
        }

        .toolbar-btn.active {
          background: #4caf50;
          color: white;
          border-color: #4caf50;
        }

        .toolbar-btn.active:hover {
          background: #45a049;
        }

        .filters-panel {
          margin-top: 16px;
          padding-top: 16px;
          border-top: 1px solid #e0e0e0;
        }

        .filters-panel h3 {
          margin: 0 0 12px 0;
          font-size: 16px;
          font-weight: 600;
          color: #333;
        }

        .filters-list {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }

        .filter-item {
          display: inline-flex;
          align-items: center;
          gap: 8px;
          padding: 6px 12px;
          background: #f5f5f5;
          border-radius: 4px;
          font-size: 13px;
        }

        .filter-field {
          font-weight: 600;
          color: #333;
        }

        .filter-operator {
          color: #666;
        }

        .filter-value {
          color: #1976d2;
        }

        @media (max-width: 768px) {
          .dashboard-toolbar {
            padding: 12px 16px;
          }

          .toolbar-left,
          .toolbar-right {
            display: block;
            float: none;
            margin-bottom: 12px;
          }

          .toolbar-btn {
            display: block;
            width: 100%;
            margin: 4px 0;
          }
        }
      `}</style>
    </div>
  );
};
