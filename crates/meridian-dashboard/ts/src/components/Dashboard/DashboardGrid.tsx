/**
 * DashboardGrid - Drag-and-drop dashboard grid layout
 */

import React, { useState, useCallback } from 'react';
import GridLayout, { Layout, Layouts } from 'react-grid-layout';
import 'react-grid-layout/css/styles.css';
import { Widget } from '../../types';
import { DashboardWidget } from './DashboardWidget';
import { useDashboardContext } from '../../context/DashboardContext';

export interface DashboardGridProps {
  widgets: Widget[];
  editable?: boolean;
  onLayoutChange?: (layout: Layout[]) => void;
  onWidgetRemove?: (widgetId: string) => void;
}

export const DashboardGrid: React.FC<DashboardGridProps> = ({
  widgets,
  editable = false,
  onLayoutChange,
  onWidgetRemove,
}) => {
  const { dashboard } = useDashboardContext();
  const [currentBreakpoint, setCurrentBreakpoint] = useState<string>('lg');

  // Convert widgets to grid layout format
  const layout: Layout[] = widgets.map((widget) => ({
    i: widget.id,
    x: widget.position.x,
    y: widget.position.y,
    w: widget.position.w,
    h: widget.position.h,
    minW: widget.position.min_w,
    minH: widget.position.min_h,
    maxW: widget.position.max_w,
    maxH: widget.position.max_h,
    static: !editable,
  }));

  const handleLayoutChange = useCallback(
    (newLayout: Layout[]) => {
      if (onLayoutChange && editable) {
        onLayoutChange(newLayout);
      }
    },
    [onLayoutChange, editable]
  );

  const handleBreakpointChange = useCallback((breakpoint: string) => {
    setCurrentBreakpoint(breakpoint);
  }, []);

  const breakpoints = dashboard?.layout.breakpoints || {
    lg: 1200,
    md: 996,
    sm: 768,
    xs: 480,
    xxs: 0,
  };

  const cols = dashboard?.layout.breakpoints || {
    lg: 12,
    md: 10,
    sm: 6,
    xs: 4,
    xxs: 2,
  };

  return (
    <div className="dashboard-grid">
      <GridLayout
        className="layout"
        layout={layout}
        cols={cols[currentBreakpoint as keyof typeof cols] || 12}
        rowHeight={60}
        width={1200}
        breakpoints={breakpoints}
        onLayoutChange={handleLayoutChange}
        onBreakpointChange={handleBreakpointChange}
        isDraggable={editable}
        isResizable={editable}
        compactType="vertical"
        preventCollision={false}
      >
        {widgets.map((widget) => (
          <div key={widget.id} className="grid-item">
            <DashboardWidget
              widget={widget}
              editable={editable}
              onRemove={onWidgetRemove}
            />
          </div>
        ))}
      </GridLayout>

      <style jsx>{`
        .dashboard-grid {
          width: 100%;
          height: 100%;
          padding: 20px;
          background: #f5f5f5;
        }

        .layout {
          position: relative;
        }

        .grid-item {
          background: white;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          overflow: hidden;
          transition: box-shadow 0.3s ease;
        }

        .grid-item:hover {
          box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
        }

        @media (max-width: 768px) {
          .dashboard-grid {
            padding: 10px;
          }
        }
      `}</style>
    </div>
  );
};
