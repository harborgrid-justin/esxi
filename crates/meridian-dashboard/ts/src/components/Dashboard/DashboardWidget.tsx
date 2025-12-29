/**
 * DashboardWidget - Base widget container component
 */

import React, { useState } from 'react';
import { Widget } from '../../types';
import { MapWidget } from '../Widgets/MapWidget';
import { ChartWidget } from '../Widgets/ChartWidget';
import { TableWidget } from '../Widgets/TableWidget';
import { KPIWidget } from '../Widgets/KPIWidget';
import { TimelineWidget } from '../Widgets/TimelineWidget';
import { FilterWidget } from '../Widgets/FilterWidget';

export interface DashboardWidgetProps {
  widget: Widget;
  editable?: boolean;
  onRemove?: (widgetId: string) => void;
  onEdit?: (widgetId: string) => void;
}

export const DashboardWidget: React.FC<DashboardWidgetProps> = ({
  widget,
  editable = false,
  onRemove,
  onEdit,
}) => {
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const handleRemove = () => {
    if (onRemove) {
      onRemove(widget.id);
    }
  };

  const handleEdit = () => {
    if (onEdit) {
      onEdit(widget.id);
    }
    setIsMenuOpen(false);
  };

  const renderWidget = () => {
    switch (widget.widget_type) {
      case 'map':
        return <MapWidget widget={widget} />;
      case 'chart':
        return <ChartWidget widget={widget} />;
      case 'table':
        return <TableWidget widget={widget} />;
      case 'kpi':
        return <KPIWidget widget={widget} />;
      case 'timeline':
        return <TimelineWidget widget={widget} />;
      case 'filter':
        return <FilterWidget widget={widget} />;
      default:
        return <div>Unknown widget type</div>;
    }
  };

  return (
    <div className="dashboard-widget">
      <div className="widget-header">
        <h3 className="widget-title">{widget.title}</h3>
        {editable && (
          <div className="widget-actions">
            <button
              className="action-btn"
              onClick={() => setIsMenuOpen(!isMenuOpen)}
              aria-label="Widget menu"
            >
              ⋮
            </button>
            {isMenuOpen && (
              <div className="action-menu">
                <button onClick={handleEdit}>Edit</button>
                <button onClick={handleRemove} className="danger">
                  Remove
                </button>
              </div>
            )}
          </div>
        )}
      </div>
      {widget.description && (
        <p className="widget-description">{widget.description}</p>
      )}
      <div className="widget-content">{renderWidget()}</div>

      <style jsx>{`
        .dashboard-widget {
          display: flex;
          flex-direction: column;
          height: 100%;
          width: 100%;
          overflow: hidden;
        }

        .widget-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid #e0e0e0;
          background: #fafafa;
        }

        .widget-title {
          margin: 0;
          font-size: 16px;
          font-weight: 600;
          color: #333;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .widget-actions {
          position: relative;
        }

        .action-btn {
          background: none;
          border: none;
          font-size: 20px;
          cursor: pointer;
          padding: 4px 8px;
          color: #666;
          transition: color 0.2s;
        }

        .action-btn:hover {
          color: #333;
        }

        .action-menu {
          position: absolute;
          right: 0;
          top: 100%;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
          z-index: 1000;
          min-width: 120px;
        }

        .action-menu button {
          display: block;
          width: 100%;
          padding: 8px 16px;
          border: none;
          background: none;
          text-align: left;
          cursor: pointer;
          transition: background 0.2s;
        }

        .action-menu button:hover {
          background: #f5f5f5;
        }

        .action-menu button.danger {
          color: #d32f2f;
        }

        .action-menu button.danger:hover {
          background: #ffebee;
        }

        .widget-description {
          padding: 8px 16px;
          margin: 0;
          font-size: 13px;
          color: #666;
          border-bottom: 1px solid #e0e0e0;
        }

        .widget-content {
          flex: 1;
          overflow: auto;
          padding: 16px;
        }
      `}</style>
    </div>
  );
};
