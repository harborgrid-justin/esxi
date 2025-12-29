/**
 * SelectTool Component
 * Feature selection tool with multiple selection modes
 * @module @meridian/ui-components/Tools
 */

import React, { useState, useCallback } from 'react';
import { useMap } from '../../hooks/useMap';
import { useSelection } from '../../hooks/useSelection';
import type { SelectionMode, Feature } from '../../types';

export interface SelectToolProps {
  /** Initial selection mode */
  initialMode?: SelectionMode;
  /** Callback when selection changes */
  onSelectionChange?: (features: Feature[]) => void;
  /** Callback when tool is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Tool for selecting features on the map
 */
export const SelectTool: React.FC<SelectToolProps> = ({
  initialMode = 'single',
  onSelectionChange,
  onClose,
  className = '',
}) => {
  const { setInteractionMode, setCursor } = useMap();
  const {
    selectedFeatures,
    selectionCount,
    clearSelection,
    exportSelection,
  } = useSelection();
  const [selectionMode, setSelectionMode] = useState<SelectionMode>(initialMode);
  const [isActive, setIsActive] = useState(false);

  /**
   * Start selection
   */
  const startSelection = useCallback(() => {
    setIsActive(true);
    setInteractionMode('select');
    setCursor(selectionMode === 'box' ? 'crosshair' : 'pointer');
  }, [setInteractionMode, setCursor, selectionMode]);

  /**
   * Stop selection
   */
  const stopSelection = useCallback(() => {
    setIsActive(false);
    setInteractionMode('pan');
    setCursor('default');
  }, [setInteractionMode, setCursor]);

  /**
   * Handle mode change
   */
  const handleModeChange = useCallback(
    (mode: SelectionMode) => {
      setSelectionMode(mode);
      if (isActive) {
        setCursor(mode === 'box' ? 'crosshair' : 'pointer');
      }
    },
    [isActive, setCursor]
  );

  /**
   * Handle clear selection
   */
  const handleClearSelection = useCallback(() => {
    clearSelection();
    onSelectionChange?.([]);
  }, [clearSelection, onSelectionChange]);

  /**
   * Export selected features
   */
  const handleExport = useCallback(() => {
    const geojson = exportSelection();
    const blob = new Blob([JSON.stringify(geojson, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `selection-${Date.now()}.geojson`;
    a.click();
    URL.revokeObjectURL(url);
  }, [exportSelection]);

  return (
    <div
      className={`meridian-select-tool bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="region"
      aria-label="Selection tool"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Select Features</h3>
        {onClose && (
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
            aria-label="Close tool"
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

      {/* Selection mode */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Selection Mode
        </label>
        <div className="grid grid-cols-2 gap-2">
          {(['single', 'multiple', 'box', 'polygon'] as SelectionMode[]).map(
            (mode) => (
              <button
                key={mode}
                onClick={() => handleModeChange(mode)}
                disabled={isActive}
                className={`px-3 py-2 rounded-lg border text-sm transition-colors ${
                  selectionMode === mode
                    ? 'bg-blue-600 text-white border-blue-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50 disabled:opacity-50'
                }`}
                aria-pressed={selectionMode === mode}
              >
                {mode.charAt(0).toUpperCase() + mode.slice(1)}
              </button>
            )
          )}
        </div>
      </div>

      {/* Instructions */}
      <div className="mb-4 p-3 bg-blue-50 rounded-lg">
        <p className="text-sm text-blue-900">
          {getSelectionInstructions(selectionMode)}
        </p>
      </div>

      {/* Controls */}
      <div className="space-y-2">
        {!isActive ? (
          <button
            onClick={startSelection}
            className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Start Selection
          </button>
        ) : (
          <button
            onClick={stopSelection}
            className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
          >
            Stop Selection
          </button>
        )}

        {selectionCount > 0 && (
          <>
            <button
              onClick={handleClearSelection}
              className="w-full px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
            >
              Clear Selection
            </button>
            <button
              onClick={handleExport}
              className="w-full px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
            >
              Export Selection
            </button>
          </>
        )}
      </div>

      {/* Selection summary */}
      {selectionCount > 0 && (
        <div className="mt-4 p-4 bg-green-50 rounded-lg border border-green-200">
          <div className="flex items-center justify-between mb-2">
            <div className="text-sm font-medium text-green-900">
              Selected Features
            </div>
            <div className="text-2xl font-bold text-green-900">
              {selectionCount}
            </div>
          </div>

          {/* Feature list */}
          <div className="mt-3 max-h-48 overflow-y-auto space-y-2">
            {selectedFeatures.map((feature, index) => (
              <div
                key={feature.id || index}
                className="text-xs bg-white p-2 rounded border border-green-200"
              >
                <div className="font-medium text-gray-900">
                  {feature.properties.name || `Feature ${index + 1}`}
                </div>
                <div className="text-gray-600 capitalize">
                  Type: {feature.geometry.type}
                </div>
                {feature.id && (
                  <div className="text-gray-500">ID: {feature.id}</div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Selection options */}
      <div className="mt-4 space-y-3">
        <div>
          <label className="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              defaultChecked
              className="w-4 h-4 text-blue-600 border-gray-300 rounded"
            />
            <span>Highlight selected features</span>
          </label>
        </div>
        <div>
          <label className="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              defaultChecked
              className="w-4 h-4 text-blue-600 border-gray-300 rounded"
            />
            <span>Show selection count</span>
          </label>
        </div>
        <div>
          <label className="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
            <input
              type="checkbox"
              className="w-4 h-4 text-blue-600 border-gray-300 rounded"
            />
            <span>Zoom to selection</span>
          </label>
        </div>
      </div>
    </div>
  );
};

/**
 * Get instructions based on selection mode
 */
function getSelectionInstructions(mode: SelectionMode): string {
  switch (mode) {
    case 'single':
      return 'Click on features to select them one at a time.';
    case 'multiple':
      return 'Hold Ctrl/Cmd and click to select multiple features.';
    case 'box':
      return 'Click and drag to draw a selection box around features.';
    case 'polygon':
      return 'Click to draw a polygon around features you want to select.';
    default:
      return 'Click on features to select them.';
  }
}

export default SelectTool;
