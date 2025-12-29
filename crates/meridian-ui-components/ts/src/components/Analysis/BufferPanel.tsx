/**
 * BufferPanel Component
 * Buffer analysis interface
 * @module @meridian/ui-components/Analysis
 */

import React, { useState } from 'react';
import { useSelection } from '../../hooks/useSelection';
import type { BufferParams, Feature } from '../../types';

export interface BufferPanelProps {
  /** Callback when buffer is created */
  onBufferCreate?: (features: Feature[], params: BufferParams) => void;
  /** Callback when panel is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Panel for creating buffer zones around features
 */
export const BufferPanel: React.FC<BufferPanelProps> = ({
  onBufferCreate,
  onClose,
  className = '',
}) => {
  const { selectedFeatures, selectionCount } = useSelection();
  const [distance, setDistance] = useState(100);
  const [unit, setUnit] = useState<BufferParams['unit']>('meters');
  const [segments, setSegments] = useState(16);
  const [dissolve, setDissolve] = useState(false);

  const handleCreateBuffer = () => {
    if (selectionCount === 0) {
      alert('Please select features to buffer');
      return;
    }

    const params: BufferParams = {
      distance,
      unit,
      segments,
    };

    onBufferCreate?.(selectedFeatures, params);
  };

  return (
    <div
      className={`meridian-buffer-panel bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="region"
      aria-label="Buffer analysis"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Buffer Analysis</h3>
        {onClose && (
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
            aria-label="Close panel"
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

      {/* Description */}
      <p className="text-sm text-gray-600 mb-4">
        Create buffer zones around selected features at a specified distance.
      </p>

      {/* Selection status */}
      <div
        className={`mb-4 p-3 rounded-lg ${
          selectionCount > 0
            ? 'bg-green-50 border border-green-200'
            : 'bg-yellow-50 border border-yellow-200'
        }`}
      >
        <div className="flex items-center gap-2">
          <svg
            className={`w-5 h-5 ${
              selectionCount > 0 ? 'text-green-600' : 'text-yellow-600'
            }`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d={
                selectionCount > 0
                  ? 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z'
                  : 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
              }
            />
          </svg>
          <span
            className={`text-sm font-medium ${
              selectionCount > 0 ? 'text-green-900' : 'text-yellow-900'
            }`}
          >
            {selectionCount > 0
              ? `${selectionCount} feature${selectionCount !== 1 ? 's' : ''} selected`
              : 'No features selected'}
          </span>
        </div>
      </div>

      {/* Buffer distance */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Buffer Distance
        </label>
        <div className="flex gap-2">
          <input
            type="number"
            value={distance}
            onChange={(e) => setDistance(parseFloat(e.target.value))}
            min="0"
            step="10"
            className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
            aria-label="Buffer distance"
          />
          <select
            value={unit}
            onChange={(e) => setUnit(e.target.value as BufferParams['unit'])}
            className="px-3 py-2 border border-gray-300 rounded-lg text-sm"
            aria-label="Distance unit"
          >
            <option value="meters">Meters</option>
            <option value="kilometers">Kilometers</option>
            <option value="feet">Feet</option>
            <option value="miles">Miles</option>
          </select>
        </div>
      </div>

      {/* Segments */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Segments: {segments}
        </label>
        <input
          type="range"
          value={segments}
          onChange={(e) => setSegments(parseInt(e.target.value))}
          min="8"
          max="64"
          step="8"
          className="w-full"
          aria-label="Number of segments"
        />
        <div className="flex justify-between text-xs text-gray-500 mt-1">
          <span>Low quality (8)</span>
          <span>High quality (64)</span>
        </div>
      </div>

      {/* Options */}
      <div className="mb-4 space-y-2">
        <label className="flex items-center gap-2 text-sm text-gray-700 cursor-pointer">
          <input
            type="checkbox"
            checked={dissolve}
            onChange={(e) => setDissolve(e.target.checked)}
            className="w-4 h-4 text-blue-600 border-gray-300 rounded"
          />
          <span>Dissolve overlapping buffers</span>
        </label>
      </div>

      {/* Preview */}
      {distance > 0 && (
        <div className="mb-4 p-3 bg-gray-50 rounded-lg border border-gray-200">
          <div className="text-sm font-medium text-gray-700 mb-2">Preview</div>
          <div className="flex items-center justify-center p-4">
            <svg width="120" height="120" viewBox="0 0 120 120">
              {/* Original feature (point) */}
              <circle cx="60" cy="60" r="3" fill="#3b82f6" />
              {/* Buffer */}
              <circle
                cx="60"
                cy="60"
                r="30"
                fill="none"
                stroke="#3b82f6"
                strokeWidth="2"
                strokeDasharray="4 2"
                opacity="0.5"
              />
              <circle
                cx="60"
                cy="60"
                r="30"
                fill="#3b82f6"
                opacity="0.2"
              />
            </svg>
          </div>
          <div className="text-xs text-gray-600 text-center">
            {distance} {unit} buffer
          </div>
        </div>
      )}

      {/* Actions */}
      <div className="space-y-2">
        <button
          onClick={handleCreateBuffer}
          disabled={selectionCount === 0}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Create Buffer
        </button>
      </div>

      {/* Help text */}
      <div className="mt-4 p-3 bg-blue-50 rounded-lg">
        <p className="text-xs text-blue-900">
          <strong>Tip:</strong> Select one or more features on the map, then
          specify the buffer distance to create zones around them.
        </p>
      </div>
    </div>
  );
};

export default BufferPanel;
