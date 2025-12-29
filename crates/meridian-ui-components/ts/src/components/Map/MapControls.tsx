/**
 * MapControls Component
 * Zoom, pan, rotate, and other map controls
 * @module @meridian/ui-components/Map
 */

import React from 'react';
import { useMap } from '../../hooks/useMap';

export interface MapControlsProps {
  /** Position of controls */
  position?: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right';
  /** Show zoom controls */
  showZoom?: boolean;
  /** Show rotation controls */
  showRotation?: boolean;
  /** Show pitch controls */
  showPitch?: boolean;
  /** Show fullscreen control */
  showFullscreen?: boolean;
  /** Custom CSS class */
  className?: string;
}

/**
 * Map control buttons for zoom, rotation, and other interactions
 */
export const MapControls: React.FC<MapControlsProps> = ({
  position = 'top-right',
  showZoom = true,
  showRotation = true,
  showPitch = true,
  showFullscreen = false,
  className = '',
}) => {
  const {
    zoomIn,
    zoomOut,
    resetRotation,
    setPitch,
    viewState,
  } = useMap();

  const positionClasses = {
    'top-left': 'top-4 left-4',
    'top-right': 'top-4 right-4',
    'bottom-left': 'bottom-4 left-4',
    'bottom-right': 'bottom-4 right-4',
  };

  const handleFullscreen = () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen();
    } else {
      document.exitFullscreen();
    }
  };

  const handlePitchChange = () => {
    setPitch(viewState.pitch > 0 ? 0 : 45);
  };

  return (
    <div
      className={`absolute ${positionClasses[position]} flex flex-col gap-2 ${className}`}
      role="group"
      aria-label="Map controls"
    >
      {/* Zoom controls */}
      {showZoom && (
        <div className="bg-white rounded-lg shadow-lg overflow-hidden">
          <button
            onClick={zoomIn}
            className="w-10 h-10 flex items-center justify-center hover:bg-gray-100 border-b border-gray-200 transition-colors"
            aria-label="Zoom in"
            title="Zoom in"
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
                d="M12 4v16m8-8H4"
              />
            </svg>
          </button>
          <button
            onClick={zoomOut}
            className="w-10 h-10 flex items-center justify-center hover:bg-gray-100 transition-colors"
            aria-label="Zoom out"
            title="Zoom out"
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
                d="M20 12H4"
              />
            </svg>
          </button>
        </div>
      )}

      {/* Rotation control */}
      {showRotation && (
        <button
          onClick={resetRotation}
          className={`w-10 h-10 bg-white rounded-lg shadow-lg flex items-center justify-center hover:bg-gray-100 transition-colors ${
            viewState.rotation !== 0 ? 'text-blue-600' : ''
          }`}
          aria-label="Reset rotation"
          title="Reset rotation to north"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
            style={{ transform: `rotate(${viewState.rotation}deg)` }}
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M5 10l7-7m0 0l7 7m-7-7v18"
            />
          </svg>
        </button>
      )}

      {/* Pitch control */}
      {showPitch && (
        <button
          onClick={handlePitchChange}
          className={`w-10 h-10 bg-white rounded-lg shadow-lg flex items-center justify-center hover:bg-gray-100 transition-colors ${
            viewState.pitch > 0 ? 'text-blue-600' : ''
          }`}
          aria-label="Toggle 3D view"
          title="Toggle 3D perspective"
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
              d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"
            />
          </svg>
        </button>
      )}

      {/* Fullscreen control */}
      {showFullscreen && (
        <button
          onClick={handleFullscreen}
          className="w-10 h-10 bg-white rounded-lg shadow-lg flex items-center justify-center hover:bg-gray-100 transition-colors"
          aria-label="Toggle fullscreen"
          title="Toggle fullscreen mode"
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
              d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4"
            />
          </svg>
        </button>
      )}
    </div>
  );
};

export default MapControls;
