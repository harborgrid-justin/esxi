/**
 * Viewport Controls - Pan, Zoom, and Rotation Controls
 * Control viewport transformation
 */

import React from 'react';
import { Viewport } from '../types';

export interface ViewportControlsProps {
  viewport: Viewport;
  onViewportChange: (viewport: Viewport) => void;
}

export const ViewportControls: React.FC<ViewportControlsProps> = ({
  viewport,
  onViewportChange
}) => {
  const handleZoomIn = () => {
    const newViewport = { ...viewport, zoom: Math.min(viewport.zoom * 1.2, 10) };
    onViewportChange(newViewport);
  };

  const handleZoomOut = () => {
    const newViewport = { ...viewport, zoom: Math.max(viewport.zoom / 1.2, 0.1) };
    onViewportChange(newViewport);
  };

  const handleZoomReset = () => {
    const newViewport = { ...viewport, zoom: 1, center: { x: 0, y: 0 }, rotation: 0 };
    onViewportChange(newViewport);
  };

  const handleRotateLeft = () => {
    const newViewport = { ...viewport, rotation: viewport.rotation - Math.PI / 12 };
    onViewportChange(newViewport);
  };

  const handleRotateRight = () => {
    const newViewport = { ...viewport, rotation: viewport.rotation + Math.PI / 12 };
    onViewportChange(newViewport);
  };

  const handleRotateReset = () => {
    const newViewport = { ...viewport, rotation: 0 };
    onViewportChange(newViewport);
  };

  return (
    <div
      style={{
        position: 'absolute',
        bottom: 20,
        right: 20,
        background: 'rgba(255, 255, 255, 0.95)',
        border: '1px solid #ccc',
        borderRadius: 8,
        padding: 12,
        boxShadow: '0 2px 8px rgba(0,0,0,0.15)'
      }}
    >
      {/* Zoom Controls */}
      <div style={{ marginBottom: 12 }}>
        <div style={{ fontSize: 11, fontWeight: 600, marginBottom: 6, color: '#666' }}>
          ZOOM
        </div>
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={handleZoomIn}
            style={buttonStyle}
            title="Zoom In"
          >
            +
          </button>
          <button
            onClick={handleZoomOut}
            style={buttonStyle}
            title="Zoom Out"
          >
            −
          </button>
          <button
            onClick={handleZoomReset}
            style={buttonStyle}
            title="Reset Zoom"
          >
            1:1
          </button>
        </div>
        <div style={{ fontSize: 11, marginTop: 4, textAlign: 'center', color: '#666' }}>
          {(viewport.zoom * 100).toFixed(0)}%
        </div>
      </div>

      {/* Rotation Controls */}
      <div>
        <div style={{ fontSize: 11, fontWeight: 600, marginBottom: 6, color: '#666' }}>
          ROTATE
        </div>
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={handleRotateLeft}
            style={buttonStyle}
            title="Rotate Left"
          >
            ↶
          </button>
          <button
            onClick={handleRotateRight}
            style={buttonStyle}
            title="Rotate Right"
          >
            ↷
          </button>
          <button
            onClick={handleRotateReset}
            style={buttonStyle}
            title="Reset Rotation"
          >
            ⟲
          </button>
        </div>
        <div style={{ fontSize: 11, marginTop: 4, textAlign: 'center', color: '#666' }}>
          {((viewport.rotation * 180) / Math.PI).toFixed(0)}°
        </div>
      </div>
    </div>
  );
};

const buttonStyle: React.CSSProperties = {
  width: 32,
  height: 32,
  padding: 0,
  border: '1px solid #ccc',
  borderRadius: 4,
  background: 'white',
  cursor: 'pointer',
  fontSize: 14,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  transition: 'background 0.2s'
};
