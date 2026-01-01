/**
 * Dashboard Builder Component - Drag & Drop Dashboard Builder
 * @module @harborgrid/enterprise-analytics/components/dashboard
 */

import React, { useState, useCallback } from 'react';
import type { Dashboard, Widget, DashboardLayout } from '../../types';

export interface DashboardBuilderProps {
  dashboard: Dashboard;
  onDashboardUpdate: (dashboard: Dashboard) => void;
  editMode?: boolean;
}

export function DashboardBuilder({
  dashboard,
  onDashboardUpdate,
  editMode = false,
}: DashboardBuilderProps) {
  const [selectedWidget, setSelectedWidget] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  const handleWidgetMove = useCallback(
    (widgetId: string, x: number, y: number) => {
      const updatedWidgets = dashboard.widgets.map((widget) =>
        widget.id === widgetId
          ? { ...widget, layout: { ...widget.layout, x, y } }
          : widget
      );

      onDashboardUpdate({
        ...dashboard,
        widgets: updatedWidgets,
      });
    },
    [dashboard, onDashboardUpdate]
  );

  const handleWidgetResize = useCallback(
    (widgetId: string, width: number, height: number) => {
      const updatedWidgets = dashboard.widgets.map((widget) =>
        widget.id === widgetId
          ? { ...widget, layout: { ...widget.layout, width, height } }
          : widget
      );

      onDashboardUpdate({
        ...dashboard,
        widgets: updatedWidgets,
      });
    },
    [dashboard, onDashboardUpdate]
  );

  const handleWidgetDelete = useCallback(
    (widgetId: string) => {
      const updatedWidgets = dashboard.widgets.filter((w) => w.id !== widgetId);
      onDashboardUpdate({
        ...dashboard,
        widgets: updatedWidgets,
      });
    },
    [dashboard, onDashboardUpdate]
  );

  const handleAddWidget = useCallback(
    (widget: Widget) => {
      onDashboardUpdate({
        ...dashboard,
        widgets: [...dashboard.widgets, widget],
      });
    },
    [dashboard, onDashboardUpdate]
  );

  return (
    <div
      style={{
        width: '100%',
        minHeight: '600px',
        backgroundColor: dashboard.theme?.backgroundColor || '#f5f5f5',
        padding: '20px',
        position: 'relative',
      }}
    >
      {/* Dashboard Header */}
      <div style={{ marginBottom: '20px' }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 'bold' }}>
          {dashboard.name}
        </h1>
        {dashboard.description && (
          <p style={{ margin: '8px 0 0 0', color: '#666' }}>{dashboard.description}</p>
        )}
      </div>

      {/* Widgets Grid */}
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: `repeat(${dashboard.layout.columns || 12}, 1fr)`,
          gap: `${dashboard.layout.gap || 16}px`,
          position: 'relative',
        }}
      >
        {dashboard.widgets.map((widget) => (
          <WidgetContainer
            key={widget.id}
            widget={widget}
            editMode={editMode}
            selected={selectedWidget === widget.id}
            onSelect={() => setSelectedWidget(widget.id)}
            onMove={handleWidgetMove}
            onResize={handleWidgetResize}
            onDelete={handleWidgetDelete}
          />
        ))}
      </div>

      {/* Add Widget Button (Edit Mode) */}
      {editMode && (
        <button
          onClick={() => {
            const newWidget: Widget = {
              id: `widget_${Date.now()}`,
              visualizationId: '',
              layout: { x: 0, y: 0, width: 4, height: 3 },
              title: 'New Widget',
            };
            handleAddWidget(newWidget);
          }}
          style={{
            position: 'fixed',
            bottom: '20px',
            right: '20px',
            padding: '12px 24px',
            backgroundColor: '#1f77b4',
            color: 'white',
            border: 'none',
            borderRadius: '6px',
            cursor: 'pointer',
            fontSize: '14px',
            fontWeight: 'bold',
            boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
          }}
        >
          + Add Widget
        </button>
      )}
    </div>
  );
}

interface WidgetContainerProps {
  widget: Widget;
  editMode: boolean;
  selected: boolean;
  onSelect: () => void;
  onMove: (widgetId: string, x: number, y: number) => void;
  onResize: (widgetId: string, width: number, height: number) => void;
  onDelete: (widgetId: string) => void;
}

function WidgetContainer({
  widget,
  editMode,
  selected,
  onSelect,
  onMove,
  onResize,
  onDelete,
}: WidgetContainerProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [isResizing, setIsResizing] = useState(false);

  return (
    <div
      style={{
        gridColumn: `span ${widget.layout.width}`,
        gridRow: `span ${widget.layout.height}`,
        backgroundColor: 'white',
        borderRadius: '8px',
        boxShadow: selected
          ? '0 4px 12px rgba(31, 119, 180, 0.3)'
          : '0 2px 4px rgba(0,0,0,0.1)',
        padding: '16px',
        position: 'relative',
        cursor: editMode ? 'move' : 'default',
        border: selected ? '2px solid #1f77b4' : '2px solid transparent',
        transition: 'all 0.2s ease',
      }}
      onClick={editMode ? onSelect : undefined}
    >
      {/* Widget Header */}
      {widget.showTitle !== false && widget.title && (
        <div style={{ marginBottom: '12px', paddingBottom: '12px', borderBottom: '1px solid #e0e0e0' }}>
          <h3 style={{ margin: 0, fontSize: '16px', fontWeight: '600' }}>
            {widget.title}
          </h3>
          {widget.showDescription !== false && widget.description && (
            <p style={{ margin: '4px 0 0 0', fontSize: '12px', color: '#666' }}>
              {widget.description}
            </p>
          )}
        </div>
      )}

      {/* Widget Content */}
      <div style={{ minHeight: '200px' }}>
        {/* Visualization would be rendered here */}
        <div style={{ textAlign: 'center', padding: '40px', color: '#999' }}>
          Visualization: {widget.visualizationId}
        </div>
      </div>

      {/* Edit Mode Controls */}
      {editMode && selected && (
        <div
          style={{
            position: 'absolute',
            top: '8px',
            right: '8px',
            display: 'flex',
            gap: '4px',
          }}
        >
          <button
            onClick={(e) => {
              e.stopPropagation();
              onDelete(widget.id);
            }}
            style={{
              padding: '4px 8px',
              backgroundColor: '#e74c3c',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '12px',
            }}
          >
            Delete
          </button>
        </div>
      )}

      {/* Resize Handle */}
      {editMode && selected && (
        <div
          style={{
            position: 'absolute',
            bottom: '4px',
            right: '4px',
            width: '16px',
            height: '16px',
            cursor: 'nwse-resize',
            backgroundColor: '#1f77b4',
            borderRadius: '0 0 4px 0',
          }}
          onMouseDown={(e) => {
            e.stopPropagation();
            setIsResizing(true);
          }}
        />
      )}
    </div>
  );
}
