/**
 * LayerItem Component
 * Individual layer item with controls
 * @module @meridian/ui-components/Layers
 */

import React, { useState } from 'react';
import { useLayers } from '../../hooks/useLayers';
import type { LayerConfig } from '../../types';

export interface LayerItemProps {
  /** Layer configuration */
  layer: LayerConfig;
  /** Whether layer is active */
  active?: boolean;
  /** Callback when layer is selected */
  onSelect?: () => void;
  /** Enable dragging */
  draggable?: boolean;
  /** Custom CSS class */
  className?: string;
}

/**
 * Individual layer item component
 */
export const LayerItem: React.FC<LayerItemProps> = ({
  layer,
  active = false,
  onSelect,
  draggable = false,
  className = '',
}) => {
  const [showMenu, setShowMenu] = useState(false);
  const {
    toggleLayerVisibility,
    removeLayer,
    updateLayerOpacity,
    duplicateLayer,
  } = useLayers();

  const handleVisibilityToggle = (e: React.MouseEvent) => {
    e.stopPropagation();
    toggleLayerVisibility(layer.id);
  };

  const handleDelete = () => {
    if (confirm(`Delete layer "${layer.name}"?`)) {
      removeLayer(layer.id);
    }
    setShowMenu(false);
  };

  const handleDuplicate = () => {
    duplicateLayer(layer.id);
    setShowMenu(false);
  };

  const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    updateLayerOpacity(layer.id, parseFloat(e.target.value));
  };

  return (
    <div
      className={`meridian-layer-item relative rounded-lg border ${
        active
          ? 'border-blue-500 bg-blue-50'
          : 'border-gray-200 bg-white hover:bg-gray-50'
      } transition-colors ${className}`}
      onClick={onSelect}
      role="listitem"
      aria-selected={active}
      tabIndex={0}
      onKeyPress={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          onSelect?.();
        }
      }}
    >
      <div className="p-3">
        {/* Header */}
        <div className="flex items-center gap-3">
          {/* Drag handle */}
          {draggable && (
            <button
              className="cursor-move text-gray-400 hover:text-gray-600"
              aria-label="Drag to reorder"
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
                  d="M4 8h16M4 16h16"
                />
              </svg>
            </button>
          )}

          {/* Visibility toggle */}
          <button
            onClick={handleVisibilityToggle}
            className="w-6 h-6 flex items-center justify-center text-gray-600 hover:text-gray-900"
            aria-label={layer.visible ? 'Hide layer' : 'Show layer'}
            title={layer.visible ? 'Hide layer' : 'Show layer'}
          >
            {layer.visible ? (
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
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                />
              </svg>
            ) : (
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
                  d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                />
              </svg>
            )}
          </button>

          {/* Color indicator */}
          <div
            className="w-4 h-4 rounded border border-gray-300"
            style={{
              backgroundColor: layer.style.fillColor || '#3b82f6',
            }}
            aria-hidden="true"
          />

          {/* Layer name */}
          <div className="flex-1 min-w-0">
            <div className="font-medium text-sm text-gray-900 truncate">
              {layer.name}
            </div>
            <div className="text-xs text-gray-500 capitalize">
              {layer.type}
            </div>
          </div>

          {/* Menu button */}
          <button
            onClick={(e) => {
              e.stopPropagation();
              setShowMenu(!showMenu);
            }}
            className="w-6 h-6 flex items-center justify-center text-gray-600 hover:text-gray-900"
            aria-label="Layer options"
            aria-expanded={showMenu}
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
                d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z"
              />
            </svg>
          </button>
        </div>

        {/* Opacity slider */}
        {active && (
          <div className="mt-3 pt-3 border-t border-gray-200">
            <label className="flex items-center gap-2 text-xs text-gray-600">
              <span>Opacity:</span>
              <input
                type="range"
                min="0"
                max="1"
                step="0.1"
                value={layer.opacity}
                onChange={handleOpacityChange}
                className="flex-1"
                aria-label="Layer opacity"
              />
              <span className="w-10 text-right">
                {Math.round(layer.opacity * 100)}%
              </span>
            </label>
          </div>
        )}
      </div>

      {/* Context menu */}
      {showMenu && (
        <>
          <div
            className="fixed inset-0"
            onClick={() => setShowMenu(false)}
            aria-hidden="true"
          />
          <div className="absolute right-2 top-12 bg-white rounded-lg shadow-xl border border-gray-200 py-1 z-10 min-w-[150px]">
            <button
              onClick={handleDuplicate}
              className="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 transition-colors"
            >
              Duplicate
            </button>
            <button
              onClick={() => {
                /* TODO: Open style editor */
                setShowMenu(false);
              }}
              className="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 transition-colors"
            >
              Edit Style
            </button>
            <button
              onClick={() => {
                /* TODO: Export layer */
                setShowMenu(false);
              }}
              className="w-full px-4 py-2 text-left text-sm hover:bg-gray-100 transition-colors"
            >
              Export
            </button>
            <hr className="my-1" />
            <button
              onClick={handleDelete}
              className="w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50 transition-colors"
            >
              Delete
            </button>
          </div>
        </>
      )}
    </div>
  );
};

export default LayerItem;
