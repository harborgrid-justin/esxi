/**
 * CAD Canvas - Main GPU-Accelerated Canvas Component
 * High-performance WebGL2 canvas for CAD operations
 */

import React, { useEffect, useRef, useState, useCallback } from 'react';
import { CADDocument, Point, ToolType } from '../types';
import { WebGLRenderer } from '../gpu/WebGLRenderer';
import { SnapEngine } from '../engine/SnapEngine';
import { SelectionEngine } from '../engine/SelectionEngine';

export interface CADCanvasProps {
  document: CADDocument;
  activeTool: ToolType;
  width?: number;
  height?: number;
  onDocumentChange?: (document: CADDocument) => void;
  onSelectionChange?: (selectedIds: string[]) => void;
  className?: string;
  style?: React.CSSProperties;
}

export const CADCanvas: React.FC<CADCanvasProps> = ({
  document,
  activeTool,
  width = 800,
  height = 600,
  onDocumentChange,
  onSelectionChange,
  className,
  style
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rendererRef = useRef<WebGLRenderer | null>(null);
  const snapEngineRef = useRef<SnapEngine | null>(null);
  const selectionEngineRef = useRef<SelectionEngine | null>(null);
  const animationFrameRef = useRef<number>(0);

  const [isInitialized, setIsInitialized] = useState(false);
  const [fps, setFps] = useState(0);
  const [mousePosition, setMousePosition] = useState<Point>({ x: 0, y: 0 });

  /**
   * Initialize renderer
   */
  useEffect(() => {
    if (!canvasRef.current) return;

    try {
      const renderer = new WebGLRenderer(canvasRef.current, {
        antialias: true,
        alpha: true,
        powerPreference: 'high-performance'
      });

      renderer.resize(width, height);
      rendererRef.current = renderer;

      // Initialize engines
      snapEngineRef.current = new SnapEngine();
      selectionEngineRef.current = new SelectionEngine();

      setIsInitialized(true);

      // Start render loop
      const renderLoop = () => {
        if (rendererRef.current && document) {
          rendererRef.current.render(document);
          setFps(rendererRef.current.getFPS());
        }
        animationFrameRef.current = requestAnimationFrame(renderLoop);
      };

      renderLoop();

      return () => {
        if (animationFrameRef.current) {
          cancelAnimationFrame(animationFrameRef.current);
        }
        renderer.dispose();
      };
    } catch (error) {
      console.error('Failed to initialize WebGL renderer:', error);
    }
  }, [width, height]);

  /**
   * Update document
   */
  useEffect(() => {
    if (!rendererRef.current || !document) return;

    // Update snap engine with shapes
    const shapes = Array.from(document.shapes.values());
    snapEngineRef.current?.registerShapes(shapes);
    selectionEngineRef.current?.registerShapes(shapes);
  }, [document]);

  /**
   * Handle mouse down
   */
  const handleMouseDown = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      if (!canvasRef.current || !document) return;

      const rect = canvasRef.current.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;

      const worldPoint = document.viewport.screenToWorld({ x, y });

      // Handle tool-specific logic
      switch (activeTool) {
        case ToolType.Select: {
          const shape = selectionEngineRef.current?.hitTest(worldPoint);
          if (shape) {
            if (event.shiftKey) {
              selectionEngineRef.current?.toggle(shape.id);
            } else {
              selectionEngineRef.current?.clear();
              selectionEngineRef.current?.select(shape.id);
            }

            const selected = Array.from(
              selectionEngineRef.current?.getState().selectedIds || []
            );
            onSelectionChange?.(selected);
          } else if (!event.shiftKey) {
            selectionEngineRef.current?.clear();
            onSelectionChange?.([]);
          }
          break;
        }
        // Other tools would be handled here
      }
    },
    [document, activeTool, onSelectionChange]
  );

  /**
   * Handle mouse move
   */
  const handleMouseMove = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      if (!canvasRef.current || !document) return;

      const rect = canvasRef.current.getBoundingClientRect();
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;

      const worldPoint = document.viewport.screenToWorld({ x, y });
      setMousePosition(worldPoint);

      // Update hover state
      const shape = selectionEngineRef.current?.hitTest(worldPoint);
      selectionEngineRef.current?.setHovered(shape?.id);
    },
    [document]
  );

  /**
   * Handle mouse up
   */
  const handleMouseUp = useCallback(
    (event: React.MouseEvent<HTMLCanvasElement>) => {
      // Handle mouse up logic
    },
    []
  );

  /**
   * Handle wheel (zoom)
   */
  const handleWheel = useCallback(
    (event: React.WheelEvent<HTMLCanvasElement>) => {
      if (!document) return;

      event.preventDefault();

      const delta = event.deltaY > 0 ? 0.9 : 1.1;
      const newZoom = document.viewport.zoom * delta;

      document.viewport.zoom = Math.max(0.1, Math.min(10, newZoom));
      onDocumentChange?.(document);
    },
    [document, onDocumentChange]
  );

  /**
   * Handle context menu
   */
  const handleContextMenu = useCallback((event: React.MouseEvent) => {
    event.preventDefault();
  }, []);

  return (
    <div
      className={className}
      style={{
        position: 'relative',
        width,
        height,
        overflow: 'hidden',
        ...style
      }}
    >
      <canvas
        ref={canvasRef}
        width={width}
        height={height}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onWheel={handleWheel}
        onContextMenu={handleContextMenu}
        style={{
          display: 'block',
          cursor: activeTool === ToolType.Pan ? 'grab' : 'crosshair'
        }}
      />

      {/* Debug overlay */}
      <div
        style={{
          position: 'absolute',
          top: 10,
          left: 10,
          background: 'rgba(0,0,0,0.7)',
          color: 'white',
          padding: '5px 10px',
          borderRadius: 4,
          fontSize: 12,
          fontFamily: 'monospace',
          pointerEvents: 'none'
        }}
      >
        <div>FPS: {fps.toFixed(1)}</div>
        <div>
          Position: {mousePosition.x.toFixed(1)}, {mousePosition.y.toFixed(1)}
        </div>
        <div>Zoom: {(document.viewport.zoom * 100).toFixed(0)}%</div>
        <div>Tool: {activeTool}</div>
      </div>
    </div>
  );
};
