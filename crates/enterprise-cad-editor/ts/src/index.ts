/**
 * Enterprise CAD/Vector Editor
 * GPU-accelerated CAD system with parametric constraints
 *
 * @packageDocumentation
 */

// Core Types
export * from './types';

// GPU Rendering
export { WebGLRenderer } from './gpu/WebGLRenderer';
export { ShaderManager } from './gpu/ShaderManager';
export { BufferManager } from './gpu/BufferManager';
export { TextureAtlas } from './gpu/TextureAtlas';
export { BatchRenderer } from './gpu/BatchRenderer';

// CAD Engine
export { GeometryEngine } from './engine/GeometryEngine';
export { ConstraintSolver } from './engine/ConstraintSolver';
export { SnapEngine } from './engine/SnapEngine';
export { SelectionEngine } from './engine/SelectionEngine';
export { TransformEngine } from './engine/TransformEngine';

// Tools
export { PenTool } from './tools/PenTool';
export { ShapeTool } from './tools/ShapeTool';
export { MeasureTool } from './tools/MeasureTool';
export { DimensionTool } from './tools/DimensionTool';
export { BooleanTool } from './tools/BooleanTool';

// React Components
export { CADCanvas } from './components/CADCanvas';
export { LayerPanel } from './components/LayerPanel';
export { ToolPalette } from './components/ToolPalette';
export { PropertyInspector } from './components/PropertyInspector';
export { ViewportControls } from './components/ViewportControls';
export { GridOverlay } from './components/GridOverlay';

// Algorithms
export { PathSimplification } from './algorithms/PathSimplification';
export { BezierMath } from './algorithms/BezierMath';
export { ConvexHull } from './algorithms/ConvexHull';
export { Tessellation } from './algorithms/Tessellation';

// Version
export const VERSION = '1.0.0';

/**
 * Initialize the CAD editor with default configuration
 */
export function createCADEditor(canvas: HTMLCanvasElement, options?: {
  width?: number;
  height?: number;
  antialias?: boolean;
  gridSize?: number;
}) {
  const {
    width = 800,
    height = 600,
    antialias = true,
    gridSize = 10
  } = options || {};

  const renderer = new WebGLRenderer(canvas, {
    antialias,
    alpha: true,
    powerPreference: 'high-performance'
  });

  renderer.resize(width, height);

  const snapEngine = new SnapEngine({
    enabled: true,
    threshold: 10,
    gridSize
  });

  const selectionEngine = new SelectionEngine();
  const constraintSolver = new ConstraintSolver();

  return {
    renderer,
    snapEngine,
    selectionEngine,
    constraintSolver,
    dispose: () => {
      renderer.dispose();
    }
  };
}

/**
 * Utility function to create a basic CAD document
 */
export function createDocument(options?: {
  name?: string;
  width?: number;
  height?: number;
  units?: 'px' | 'mm' | 'cm' | 'in' | 'pt';
}) {
  const {
    name = 'Untitled',
    width = 1920,
    height = 1080,
    units = 'px'
  } = options || {};

  const document: import('./types').CADDocument = {
    id: `doc_${Date.now()}`,
    name,
    width,
    height,
    units,
    layers: new Map(),
    shapes: new Map(),
    viewport: {
      x: 0,
      y: 0,
      width,
      height,
      zoom: 1,
      rotation: 0,
      center: { x: 0, y: 0 },
      screenToWorld: function(point) {
        return {
          x: (point.x - this.width / 2) / this.zoom + this.center.x,
          y: (point.y - this.height / 2) / this.zoom + this.center.y
        };
      },
      worldToScreen: function(point) {
        return {
          x: (point.x - this.center.x) * this.zoom + this.width / 2,
          y: (point.y - this.center.y) * this.zoom + this.height / 2
        };
      },
      getViewBounds: function() {
        const halfWidth = this.width / (2 * this.zoom);
        const halfHeight = this.height / (2 * this.zoom);
        return {
          minX: this.center.x - halfWidth,
          minY: this.center.y - halfHeight,
          maxX: this.center.x + halfWidth,
          maxY: this.center.y + halfHeight,
          width: halfWidth * 2,
          height: halfHeight * 2,
          center: this.center
        };
      },
      fitBounds: function(bounds, padding = 0.1) {
        const boundsWidth = bounds.width * (1 + padding);
        const boundsHeight = bounds.height * (1 + padding);

        const scaleX = this.width / boundsWidth;
        const scaleY = this.height / boundsHeight;
        this.zoom = Math.min(scaleX, scaleY);

        this.center = bounds.center;
      }
    },
    selectedShapeIds: new Set(),
    addLayer: function(layer) {
      this.layers.set(layer.id, layer);
    },
    removeLayer: function(layerId) {
      this.layers.delete(layerId);
    },
    addShape: function(shape, layerId) {
      this.shapes.set(shape.id, shape);
      if (layerId) {
        shape.layerId = layerId;
      }
    },
    removeShape: function(shapeId) {
      this.shapes.delete(shapeId);
    },
    getShape: function(shapeId) {
      return this.shapes.get(shapeId);
    },
    getLayer: function(layerId) {
      return this.layers.get(layerId);
    },
    getShapesInLayer: function(layerId) {
      return Array.from(this.shapes.values()).filter(s => s.layerId === layerId);
    },
    export: function(format) {
      // Simplified export
      return JSON.stringify({
        id: this.id,
        name: this.name,
        width: this.width,
        height: this.height,
        units: this.units
      });
    }
  };

  // Add default layer
  document.addLayer({
    id: 'default',
    name: 'Layer 1',
    visible: true,
    locked: false,
    opacity: 1,
    blendMode: 'source-over',
    shapes: [],
    order: 0
  });

  return document;
}
