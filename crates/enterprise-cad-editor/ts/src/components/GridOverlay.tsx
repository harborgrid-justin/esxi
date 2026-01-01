/**
 * Grid Overlay - Dynamic Grid Rendering
 * Renders grid with adaptive spacing based on zoom level
 */

import React, { useEffect, useRef } from 'react';
import { Viewport } from '../types';

export interface GridOverlayProps {
  viewport: Viewport;
  gridSize: number;
  majorGridInterval?: number;
  gridColor?: string;
  majorGridColor?: string;
  enabled?: boolean;
}

export const GridOverlay: React.FC<GridOverlayProps> = ({
  viewport,
  gridSize,
  majorGridInterval = 5,
  gridColor = 'rgba(0, 0, 0, 0.1)',
  majorGridColor = 'rgba(0, 0, 0, 0.2)',
  enabled = true
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!enabled || !canvasRef.current) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    // Clear canvas
    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Calculate grid spacing based on zoom
    const effectiveGridSize = gridSize * viewport.zoom;

    // Skip rendering if grid is too small or too large
    if (effectiveGridSize < 5 || effectiveGridSize > 200) return;

    const bounds = viewport.getViewBounds();
    const startX = Math.floor(bounds.minX / gridSize) * gridSize;
    const startY = Math.floor(bounds.minY / gridSize) * gridSize;
    const endX = Math.ceil(bounds.maxX / gridSize) * gridSize;
    const endY = Math.ceil(bounds.maxY / gridSize) * gridSize;

    // Draw vertical lines
    for (let x = startX; x <= endX; x += gridSize) {
      const screenX = viewport.worldToScreen({ x, y: 0 }).x;
      const isMajor = Math.abs(x % (gridSize * majorGridInterval)) < 0.01;

      ctx.strokeStyle = isMajor ? majorGridColor : gridColor;
      ctx.lineWidth = isMajor ? 1 : 0.5;

      ctx.beginPath();
      ctx.moveTo(screenX, 0);
      ctx.lineTo(screenX, canvas.height);
      ctx.stroke();
    }

    // Draw horizontal lines
    for (let y = startY; y <= endY; y += gridSize) {
      const screenY = viewport.worldToScreen({ x: 0, y }).y;
      const isMajor = Math.abs(y % (gridSize * majorGridInterval)) < 0.01;

      ctx.strokeStyle = isMajor ? majorGridColor : gridColor;
      ctx.lineWidth = isMajor ? 1 : 0.5;

      ctx.beginPath();
      ctx.moveTo(0, screenY);
      ctx.lineTo(canvas.width, screenY);
      ctx.stroke();
    }

    // Draw origin axes
    const originX = viewport.worldToScreen({ x: 0, y: 0 }).x;
    const originY = viewport.worldToScreen({ x: 0, y: 0 }).y;

    // X-axis
    ctx.strokeStyle = 'rgba(255, 0, 0, 0.5)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(0, originY);
    ctx.lineTo(canvas.width, originY);
    ctx.stroke();

    // Y-axis
    ctx.strokeStyle = 'rgba(0, 255, 0, 0.5)';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(originX, 0);
    ctx.lineTo(originX, canvas.height);
    ctx.stroke();

  }, [viewport, gridSize, majorGridInterval, gridColor, majorGridColor, enabled]);

  if (!enabled) return null;

  return (
    <canvas
      ref={canvasRef}
      width={viewport.width}
      height={viewport.height}
      style={{
        position: 'absolute',
        top: 0,
        left: 0,
        pointerEvents: 'none',
        opacity: 0.5
      }}
    />
  );
};
