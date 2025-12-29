/**
 * MapContainer Component
 * Main map container with rendering and interaction handling
 * @module @meridian/ui-components/Map
 */

import React, { useEffect, useRef, useCallback } from 'react';
import { useMap } from '../../hooks/useMap';
import { useLayers } from '../../hooks/useLayers';
import type { MapEventHandlers } from '../../types';

export interface MapContainerProps {
  /** Custom CSS class */
  className?: string;
  /** Map width */
  width?: string | number;
  /** Map height */
  height?: string | number;
  /** Event handlers */
  eventHandlers?: MapEventHandlers;
  /** Show loading indicator */
  showLoader?: boolean;
  /** Children components (overlays, controls, etc.) */
  children?: React.ReactNode;
  /** ARIA label for accessibility */
  'aria-label'?: string;
}

/**
 * Main map container component
 * Handles map rendering, user interactions, and layer display
 */
export const MapContainer: React.FC<MapContainerProps> = ({
  className = '',
  width = '100%',
  height = '100vh',
  eventHandlers = {},
  showLoader = true,
  children,
  'aria-label': ariaLabel = 'Interactive map',
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const {
    mapRef,
    viewState,
    interactionMode,
    cursor,
    loading,
    error,
    setLoading,
  } = useMap();
  const { visibleLayers } = useLayers();

  /**
   * Initialize map rendering
   */
  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    setLoading(true);

    // Setup canvas size
    const resizeCanvas = () => {
      const rect = canvas.getBoundingClientRect();
      canvas.width = rect.width * window.devicePixelRatio;
      canvas.height = rect.height * window.devicePixelRatio;
      ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    setLoading(false);

    return () => {
      window.removeEventListener('resize', resizeCanvas);
    };
  }, [setLoading]);

  /**
   * Render map layers
   */
  useEffect(() => {
    if (!canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Render base layer (simplified)
    ctx.fillStyle = '#e0e7ff';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // Render grid
    ctx.strokeStyle = '#cbd5e1';
    ctx.lineWidth = 0.5;
    const gridSize = 50;
    for (let x = 0; x < canvas.width; x += gridSize) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvas.height);
      ctx.stroke();
    }
    for (let y = 0; y < canvas.height; y += gridSize) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvas.width, y);
      ctx.stroke();
    }

    // Render center crosshair
    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    ctx.strokeStyle = '#3b82f6';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(centerX - 10, centerY);
    ctx.lineTo(centerX + 10, centerY);
    ctx.moveTo(centerX, centerY - 10);
    ctx.lineTo(centerX, centerY + 10);
    ctx.stroke();

    // Display coordinates
    ctx.fillStyle = '#1f2937';
    ctx.font = '12px monospace';
    ctx.fillText(
      `Center: ${viewState.center.lon.toFixed(4)}, ${viewState.center.lat.toFixed(4)}`,
      10,
      20
    );
    ctx.fillText(`Zoom: ${viewState.zoom.toFixed(2)}`, 10, 40);
    ctx.fillText(`Mode: ${interactionMode}`, 10, 60);

    // Render visible layers (simplified)
    visibleLayers.forEach((layer, index) => {
      ctx.fillStyle = layer.style.fillColor || '#3b82f6';
      ctx.globalAlpha = layer.opacity;
      ctx.fillText(`Layer: ${layer.name}`, 10, 80 + index * 20);
      ctx.globalAlpha = 1.0;
    });
  }, [viewState, interactionMode, visibleLayers]);

  /**
   * Handle mouse events
   */
  const handleClick = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (eventHandlers.onClick) {
        const rect = canvasRef.current?.getBoundingClientRect();
        if (rect) {
          eventHandlers.onClick({
            type: 'click',
            pixel: [e.clientX - rect.left, e.clientY - rect.top],
            originalEvent: e.nativeEvent,
          });
        }
      }
    },
    [eventHandlers]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (eventHandlers.onMouseMove) {
        const rect = canvasRef.current?.getBoundingClientRect();
        if (rect) {
          eventHandlers.onMouseMove({
            type: 'mousemove',
            pixel: [e.clientX - rect.left, e.clientY - rect.top],
            originalEvent: e.nativeEvent,
          });
        }
      }
    },
    [eventHandlers]
  );

  return (
    <div
      ref={mapRef}
      className={`meridian-map-container relative ${className}`}
      style={{ width, height }}
      role="application"
      aria-label={ariaLabel}
    >
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full"
        style={{ cursor }}
        onClick={handleClick}
        onMouseMove={handleMouseMove}
        onDoubleClick={eventHandlers.onDoubleClick as any}
        onMouseEnter={eventHandlers.onMouseEnter as any}
        onMouseLeave={eventHandlers.onMouseLeave as any}
      />

      {/* Loading overlay */}
      {loading && showLoader && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/20">
          <div className="bg-white rounded-lg p-4 shadow-lg">
            <div className="animate-spin w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full" />
          </div>
        </div>
      )}

      {/* Error overlay */}
      {error && (
        <div className="absolute top-4 left-1/2 transform -translate-x-1/2 bg-red-500 text-white px-4 py-2 rounded-lg shadow-lg">
          {error}
        </div>
      )}

      {/* Children (controls, overlays, etc.) */}
      {children}
    </div>
  );
};

export default MapContainer;
