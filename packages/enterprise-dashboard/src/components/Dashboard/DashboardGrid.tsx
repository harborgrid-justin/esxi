/**
 * Dashboard Grid Component
 * Draggable and resizable grid layout for widgets
 */

import React, { useCallback, useMemo } from 'react';
import GridLayout, { Layout, Responsive, WidthProvider } from 'react-grid-layout';
import { motion, AnimatePresence } from 'framer-motion';
import clsx from 'clsx';
import type { DashboardWidget } from '../../types';
import 'react-grid-layout/css/styles.css';

const ResponsiveGridLayout = WidthProvider(Responsive);

export interface DashboardGridProps {
  widgets: DashboardWidget[];
  onLayoutChange?: (widgets: DashboardWidget[]) => void;
  onWidgetRemove?: (widgetId: string) => void;
  onWidgetEdit?: (widgetId: string) => void;
  editable?: boolean;
  cols?: { lg: number; md: number; sm: number; xs: number };
  rowHeight?: number;
  className?: string;
  renderWidget: (widget: DashboardWidget) => React.ReactNode;
}

export const DashboardGrid: React.FC<DashboardGridProps> = ({
  widgets,
  onLayoutChange,
  onWidgetRemove,
  onWidgetEdit,
  editable = false,
  cols = { lg: 12, md: 10, sm: 6, xs: 4 },
  rowHeight = 100,
  className,
  renderWidget,
}) => {
  // Convert widgets to layout format
  const layouts = useMemo(() => {
    const layout: Layout[] = widgets.map((widget) => ({
      i: widget.id,
      x: widget.position.x,
      y: widget.position.y,
      w: widget.position.w,
      h: widget.position.h,
      minW: 2,
      minH: 2,
      static: widget.locked || false,
    }));

    return {
      lg: layout,
      md: layout,
      sm: layout,
      xs: layout,
    };
  }, [widgets]);

  // Handle layout change
  const handleLayoutChange = useCallback(
    (currentLayout: Layout[], allLayouts: any) => {
      if (!onLayoutChange || !editable) return;

      const updatedWidgets = widgets.map((widget) => {
        const layoutItem = currentLayout.find((item) => item.i === widget.id);
        if (!layoutItem) return widget;

        return {
          ...widget,
          position: {
            x: layoutItem.x,
            y: layoutItem.y,
            w: layoutItem.w,
            h: layoutItem.h,
          },
        };
      });

      onLayoutChange(updatedWidgets);
    },
    [widgets, onLayoutChange, editable]
  );

  return (
    <div className={clsx('dashboard-grid', className)}>
      <ResponsiveGridLayout
        className="layout"
        layouts={layouts}
        onLayoutChange={handleLayoutChange}
        cols={cols}
        rowHeight={rowHeight}
        isDraggable={editable}
        isResizable={editable}
        margin={[16, 16]}
        containerPadding={[0, 0]}
        useCSSTransforms={true}
        preventCollision={false}
        compactType="vertical"
        draggableHandle=".widget-drag-handle"
      >
        {widgets.map((widget) => (
          <div
            key={widget.id}
            className={clsx(
              'dashboard-widget-container',
              !widget.visible && 'hidden'
            )}
          >
            <WidgetWrapper
              widget={widget}
              editable={editable}
              onRemove={onWidgetRemove}
              onEdit={onWidgetEdit}
            >
              {renderWidget(widget)}
            </WidgetWrapper>
          </div>
        ))}
      </ResponsiveGridLayout>
    </div>
  );
};

/**
 * Widget Wrapper with Controls
 */
const WidgetWrapper: React.FC<{
  widget: DashboardWidget;
  editable: boolean;
  onRemove?: (widgetId: string) => void;
  onEdit?: (widgetId: string) => void;
  children: React.ReactNode;
}> = ({ widget, editable, onRemove, onEdit, children }) => {
  const [isHovered, setIsHovered] = React.useState(false);

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.2 }}
      className="relative h-full w-full"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Widget Controls */}
      <AnimatePresence>
        {editable && isHovered && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            className="absolute top-2 right-2 z-50 flex items-center gap-2"
          >
            {/* Drag Handle */}
            <button
              className="widget-drag-handle p-2 bg-gray-800/90 hover:bg-gray-700 rounded-lg cursor-move transition-colors backdrop-blur-sm border border-gray-700"
              title="Drag to move"
            >
              <svg
                className="w-4 h-4 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 8h16M4 16h16"
                />
              </svg>
            </button>

            {/* Edit Button */}
            {onEdit && (
              <button
                onClick={() => onEdit(widget.id)}
                className="p-2 bg-blue-500/20 hover:bg-blue-500/30 text-blue-400 rounded-lg transition-colors backdrop-blur-sm border border-blue-500/30"
                title="Edit widget"
              >
                <svg
                  className="w-4 h-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                  />
                </svg>
              </button>
            )}

            {/* Lock/Unlock Button */}
            <button
              onClick={() => {
                // Toggle lock state
                // This would need to be implemented in parent
              }}
              className={clsx(
                'p-2 rounded-lg transition-colors backdrop-blur-sm border',
                widget.locked
                  ? 'bg-yellow-500/20 hover:bg-yellow-500/30 text-yellow-400 border-yellow-500/30'
                  : 'bg-gray-800/90 hover:bg-gray-700 text-gray-400 border-gray-700'
              )}
              title={widget.locked ? 'Unlock widget' : 'Lock widget'}
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                {widget.locked ? (
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                  />
                ) : (
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z"
                  />
                )}
              </svg>
            </button>

            {/* Remove Button */}
            {onRemove && (
              <button
                onClick={() => onRemove(widget.id)}
                className="p-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 rounded-lg transition-colors backdrop-blur-sm border border-red-500/30"
                title="Remove widget"
              >
                <svg
                  className="w-4 h-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            )}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Widget Content */}
      <div className="h-full w-full overflow-hidden">
        {children}
      </div>

      {/* Locked Indicator */}
      {widget.locked && editable && (
        <div className="absolute bottom-2 left-2 z-40 px-2 py-1 bg-yellow-500/20 text-yellow-400 text-xs font-semibold rounded backdrop-blur-sm border border-yellow-500/30">
          🔒 Locked
        </div>
      )}
    </motion.div>
  );
};

/**
 * Empty State for Grid
 */
export const EmptyDashboard: React.FC<{
  onAddWidget?: () => void;
  className?: string;
}> = ({ onAddWidget, className }) => {
  return (
    <div
      className={clsx(
        'flex flex-col items-center justify-center min-h-[400px] rounded-xl border-2 border-dashed border-gray-700 bg-gray-800/20',
        className
      )}
    >
      <div className="text-6xl mb-4">📊</div>
      <h3 className="text-xl font-semibold text-white mb-2">No widgets yet</h3>
      <p className="text-gray-400 mb-6 text-center max-w-md">
        Start building your dashboard by adding widgets. Customize your view with KPIs, charts, and real-time data.
      </p>
      {onAddWidget && (
        <button
          onClick={onAddWidget}
          className="px-6 py-3 bg-blue-500 hover:bg-blue-600 text-white font-semibold rounded-lg transition-colors"
        >
          Add Your First Widget
        </button>
      )}
    </div>
  );
};

/**
 * Widget Catalog/Gallery
 */
export const WidgetCatalog: React.FC<{
  onAddWidget: (type: DashboardWidget['type']) => void;
  className?: string;
}> = ({ onAddWidget, className }) => {
  const widgetTypes: Array<{
    type: DashboardWidget['type'];
    icon: string;
    label: string;
    description: string;
  }> = [
    {
      type: 'kpi',
      icon: '📈',
      label: 'KPI Card',
      description: 'Display key performance indicators with trends',
    },
    {
      type: 'chart',
      icon: '📊',
      label: 'Chart',
      description: 'Visualize data with various chart types',
    },
    {
      type: 'table',
      icon: '📋',
      label: 'Table',
      description: 'Show data in tabular format',
    },
    {
      type: 'alert',
      icon: '🔔',
      label: 'Alerts',
      description: 'Monitor and manage active alerts',
    },
    {
      type: 'activity',
      icon: '📝',
      label: 'Activity Feed',
      description: 'Track recent system activities',
    },
    {
      type: 'quota',
      icon: '📊',
      label: 'Quotas',
      description: 'Monitor resource usage and limits',
    },
  ];

  return (
    <div className={clsx('grid grid-cols-3 gap-4', className)}>
      {widgetTypes.map((widget) => (
        <button
          key={widget.type}
          onClick={() => onAddWidget(widget.type)}
          className="flex flex-col items-center gap-3 p-6 rounded-xl border border-gray-700 bg-gray-800/30 hover:bg-gray-800/50 hover:border-blue-500/50 transition-all group"
        >
          <span className="text-4xl">{widget.icon}</span>
          <div className="text-center">
            <div className="font-semibold text-white mb-1 group-hover:text-blue-400 transition-colors">
              {widget.label}
            </div>
            <div className="text-xs text-gray-500">{widget.description}</div>
          </div>
        </button>
      ))}
    </div>
  );
};

export default DashboardGrid;
