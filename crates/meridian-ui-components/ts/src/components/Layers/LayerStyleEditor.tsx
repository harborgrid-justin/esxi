/**
 * LayerStyleEditor Component
 * Advanced style editing interface for layers
 * @module @meridian/ui-components/Layers
 */

import React, { useState } from 'react';
import { useLayers } from '../../hooks/useLayers';
import type { LayerConfig, LayerStyle } from '../../types';

export interface LayerStyleEditorProps {
  /** Layer to edit */
  layer: LayerConfig;
  /** Callback when style is updated */
  onStyleUpdate?: (style: LayerStyle) => void;
  /** Callback when editor is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Style editor for customizing layer appearance
 */
export const LayerStyleEditor: React.FC<LayerStyleEditorProps> = ({
  layer,
  onStyleUpdate,
  onClose,
  className = '',
}) => {
  const { updateLayerStyle } = useLayers();
  const [style, setStyle] = useState<LayerStyle>(layer.style);

  const handleChange = (key: keyof LayerStyle, value: unknown) => {
    const newStyle = { ...style, [key]: value };
    setStyle(newStyle);
    updateLayerStyle(layer.id, newStyle);
    onStyleUpdate?.(newStyle);
  };

  return (
    <div
      className={`meridian-layer-style-editor bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="dialog"
      aria-label="Layer style editor"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Style Editor</h3>
        {onClose && (
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
            aria-label="Close editor"
          >
            <svg
              className="w-5 h-5"
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
      </div>

      <div className="space-y-4">
        {/* Layer name */}
        <div className="text-sm text-gray-600 mb-4">
          Editing: <strong>{layer.name}</strong>
        </div>

        {/* Fill color */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Fill Color
          </label>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={style.fillColor || '#3b82f6'}
              onChange={(e) => handleChange('fillColor', e.target.value)}
              className="w-12 h-10 rounded border border-gray-300 cursor-pointer"
              aria-label="Fill color"
            />
            <input
              type="text"
              value={style.fillColor || '#3b82f6'}
              onChange={(e) => handleChange('fillColor', e.target.value)}
              className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
              placeholder="#3b82f6"
              aria-label="Fill color hex value"
            />
          </div>
        </div>

        {/* Stroke color */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Stroke Color
          </label>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={style.strokeColor || '#1e40af'}
              onChange={(e) => handleChange('strokeColor', e.target.value)}
              className="w-12 h-10 rounded border border-gray-300 cursor-pointer"
              aria-label="Stroke color"
            />
            <input
              type="text"
              value={style.strokeColor || '#1e40af'}
              onChange={(e) => handleChange('strokeColor', e.target.value)}
              className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
              placeholder="#1e40af"
              aria-label="Stroke color hex value"
            />
          </div>
        </div>

        {/* Stroke width */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Stroke Width: {style.strokeWidth || 2}px
          </label>
          <input
            type="range"
            min="0"
            max="10"
            step="0.5"
            value={style.strokeWidth || 2}
            onChange={(e) =>
              handleChange('strokeWidth', parseFloat(e.target.value))
            }
            className="w-full"
            aria-label="Stroke width"
          />
        </div>

        {/* Opacity */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Opacity: {Math.round((style.opacity || 1) * 100)}%
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={style.opacity || 1}
            onChange={(e) =>
              handleChange('opacity', parseFloat(e.target.value))
            }
            className="w-full"
            aria-label="Opacity"
          />
        </div>

        {/* Z-index */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Z-Index
          </label>
          <input
            type="number"
            value={style.zIndex || 0}
            onChange={(e) => handleChange('zIndex', parseInt(e.target.value))}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            aria-label="Z-index"
          />
        </div>

        {/* Preview */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Preview
          </label>
          <div className="border border-gray-300 rounded-lg p-4 bg-gray-50">
            <div className="flex items-center justify-center">
              {/* Polygon preview */}
              <svg width="120" height="80" viewBox="0 0 120 80">
                <polygon
                  points="20,60 60,20 100,40 80,70"
                  fill={style.fillColor || '#3b82f6'}
                  stroke={style.strokeColor || '#1e40af'}
                  strokeWidth={style.strokeWidth || 2}
                  opacity={style.opacity || 1}
                />
              </svg>
            </div>
          </div>
        </div>

        {/* Preset styles */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Presets
          </label>
          <div className="grid grid-cols-3 gap-2">
            {STYLE_PRESETS.map((preset, index) => (
              <button
                key={index}
                onClick={() => {
                  Object.entries(preset.style).forEach(([key, value]) => {
                    handleChange(key as keyof LayerStyle, value);
                  });
                }}
                className="p-2 border border-gray-300 rounded-lg hover:border-blue-500 transition-colors"
                title={preset.name}
              >
                <div
                  className="w-full h-8 rounded"
                  style={{
                    backgroundColor: preset.style.fillColor,
                    border: `2px solid ${preset.style.strokeColor}`,
                    opacity: preset.style.opacity,
                  }}
                />
                <div className="text-xs text-gray-600 mt-1 truncate">
                  {preset.name}
                </div>
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

/**
 * Predefined style presets
 */
const STYLE_PRESETS = [
  {
    name: 'Blue',
    style: {
      fillColor: '#3b82f6',
      strokeColor: '#1e40af',
      strokeWidth: 2,
      opacity: 0.8,
    },
  },
  {
    name: 'Green',
    style: {
      fillColor: '#10b981',
      strokeColor: '#047857',
      strokeWidth: 2,
      opacity: 0.8,
    },
  },
  {
    name: 'Red',
    style: {
      fillColor: '#ef4444',
      strokeColor: '#b91c1c',
      strokeWidth: 2,
      opacity: 0.8,
    },
  },
  {
    name: 'Yellow',
    style: {
      fillColor: '#f59e0b',
      strokeColor: '#d97706',
      strokeWidth: 2,
      opacity: 0.8,
    },
  },
  {
    name: 'Purple',
    style: {
      fillColor: '#8b5cf6',
      strokeColor: '#6d28d9',
      strokeWidth: 2,
      opacity: 0.8,
    },
  },
  {
    name: 'Gray',
    style: {
      fillColor: '#6b7280',
      strokeColor: '#374151',
      strokeWidth: 2,
      opacity: 0.8,
    },
  },
];

export default LayerStyleEditor;
