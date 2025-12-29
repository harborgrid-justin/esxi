/**
 * LayerPanel Component
 * Layer management panel with drag-and-drop reordering
 * @module @meridian/ui-components/Layers
 */

import React, { useState } from 'react';
import { useLayers } from '../../hooks/useLayers';
import { LayerItem } from './LayerItem';
import type { PanelProps } from '../../types';

export interface LayerPanelProps extends PanelProps {
  /** Show add layer button */
  showAddButton?: boolean;
  /** Callback when add layer is clicked */
  onAddLayer?: () => void;
  /** Enable drag and drop reordering */
  enableReorder?: boolean;
}

/**
 * Layer management panel
 * Displays all layers with visibility toggles and management options
 */
export const LayerPanel: React.FC<LayerPanelProps> = ({
  title = 'Layers',
  collapsible = true,
  collapsed: initialCollapsed = false,
  onCollapse,
  showAddButton = true,
  onAddLayer,
  enableReorder = true,
  className = '',
  children,
}) => {
  const [collapsed, setCollapsed] = useState(initialCollapsed);
  const {
    sortedLayers,
    activeLayerId,
    setActiveLayer,
    layerCount,
    visibleLayerCount,
  } = useLayers();

  const handleCollapse = () => {
    const newCollapsed = !collapsed;
    setCollapsed(newCollapsed);
    onCollapse?.(newCollapsed);
  };

  return (
    <div
      className={`meridian-layer-panel bg-white rounded-lg shadow-lg ${className}`}
      role="region"
      aria-label="Layer panel"
    >
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div className="flex items-center gap-2">
          <h2 className="font-semibold text-gray-900">{title}</h2>
          <span className="text-xs text-gray-500">
            ({visibleLayerCount}/{layerCount})
          </span>
        </div>
        <div className="flex items-center gap-2">
          {showAddButton && (
            <button
              onClick={onAddLayer}
              className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
              aria-label="Add layer"
              title="Add new layer"
            >
              <svg
                className="w-5 h-5 text-gray-700"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 4v16m8-8H4"
                />
              </svg>
            </button>
          )}
          {collapsible && (
            <button
              onClick={handleCollapse}
              className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
              aria-label={collapsed ? 'Expand panel' : 'Collapse panel'}
              aria-expanded={!collapsed}
            >
              <svg
                className={`w-5 h-5 text-gray-700 transition-transform ${
                  collapsed ? 'rotate-180' : ''
                }`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 9l-7 7-7-7"
                />
              </svg>
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="p-4">
          {/* Layer list */}
          {sortedLayers.length === 0 ? (
            <div className="text-center py-8 text-gray-500">
              <svg
                className="w-12 h-12 mx-auto mb-3 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
                />
              </svg>
              <p className="text-sm">No layers added</p>
              {showAddButton && (
                <button
                  onClick={onAddLayer}
                  className="mt-3 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors text-sm"
                >
                  Add Your First Layer
                </button>
              )}
            </div>
          ) : (
            <div className="space-y-2" role="list">
              {sortedLayers.map((layer) => (
                <LayerItem
                  key={layer.id}
                  layer={layer}
                  active={layer.id === activeLayerId}
                  onSelect={() => setActiveLayer(layer.id)}
                  draggable={enableReorder}
                />
              ))}
            </div>
          )}

          {/* Custom children */}
          {children}
        </div>
      )}
    </div>
  );
};

export default LayerPanel;
