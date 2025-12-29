/**
 * MapLegend Component
 * Dynamic legend displaying layer styles and symbology
 * @module @meridian/ui-components/Map
 */

import React, { useState } from 'react';
import { useLayers } from '../../hooks/useLayers';
import type { LayerConfig } from '../../types';

export interface MapLegendProps {
  /** Position of legend */
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  /** Initial collapsed state */
  collapsed?: boolean;
  /** Show only visible layers */
  visibleOnly?: boolean;
  /** Custom CSS class */
  className?: string;
}

/**
 * Legend component showing layer symbology
 */
export const MapLegend: React.FC<MapLegendProps> = ({
  position = 'bottom-right',
  collapsed: initialCollapsed = false,
  visibleOnly = true,
  className = '',
}) => {
  const [collapsed, setCollapsed] = useState(initialCollapsed);
  const { layers, visibleLayers } = useLayers();

  const displayLayers = visibleOnly ? visibleLayers : layers;

  const positionClasses = {
    'top-left': 'top-4 left-4',
    'top-right': 'top-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'bottom-right': 'bottom-4 right-4',
  };

  if (displayLayers.length === 0) {
    return null;
  }

  return (
    <div
      className={`absolute ${positionClasses[position]} bg-white rounded-lg shadow-lg max-w-xs ${className}`}
      role="complementary"
      aria-label="Map legend"
    >
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-gray-200">
        <h3 className="font-semibold text-sm text-gray-900">Legend</h3>
        <button
          onClick={() => setCollapsed(!collapsed)}
          className="w-6 h-6 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
          aria-label={collapsed ? 'Expand legend' : 'Collapse legend'}
          aria-expanded={!collapsed}
        >
          <svg
            className={`w-4 h-4 transition-transform ${
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
      </div>

      {/* Content */}
      {!collapsed && (
        <div className="p-3 max-h-96 overflow-y-auto">
          <div className="space-y-3">
            {displayLayers.map((layer) => (
              <LegendItem key={layer.id} layer={layer} />
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * Individual legend item for a layer
 */
const LegendItem: React.FC<{ layer: LayerConfig }> = ({ layer }) => {
  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2">
        <div
          className="w-5 h-5 rounded border border-gray-300 flex-shrink-0"
          style={{
            backgroundColor: layer.style.fillColor || '#3b82f6',
            opacity: layer.opacity,
          }}
          aria-hidden="true"
        />
        <span className="text-sm text-gray-900 font-medium truncate">
          {layer.name}
        </span>
      </div>

      {/* Layer type indicator */}
      <div className="flex items-center gap-2 ml-7">
        <span className="text-xs text-gray-500 capitalize">{layer.type}</span>
        {layer.style.strokeColor && (
          <div className="flex items-center gap-1">
            <div
              className="w-6 h-0.5"
              style={{
                backgroundColor: layer.style.strokeColor,
              }}
              aria-hidden="true"
            />
            <span className="text-xs text-gray-500">
              {layer.style.strokeWidth}px
            </span>
          </div>
        )}
      </div>

      {/* Opacity indicator */}
      {layer.opacity < 1 && (
        <div className="ml-7 text-xs text-gray-500">
          Opacity: {Math.round(layer.opacity * 100)}%
        </div>
      )}

      {/* Zoom range */}
      {(layer.minZoom !== undefined || layer.maxZoom !== undefined) && (
        <div className="ml-7 text-xs text-gray-500">
          Zoom: {layer.minZoom ?? 0} - {layer.maxZoom ?? 22}
        </div>
      )}
    </div>
  );
};

export default MapLegend;
