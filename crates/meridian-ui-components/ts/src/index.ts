/**
 * Meridian UI Components
 * Enterprise React/TypeScript UI component library for GIS applications
 * @module @meridian/ui-components
 */

// Context & Providers
export { MapProvider, useMapContext, useMapStore, useLayerStore } from './context/MapContext';

// Hooks
export { useMap } from './hooks/useMap';
export { useLayers } from './hooks/useLayers';
export { useSelection } from './hooks/useSelection';

// Map Components
export { MapContainer } from './components/Map/MapContainer';
export type { MapContainerProps } from './components/Map/MapContainer';

export { MapControls } from './components/Map/MapControls';
export type { MapControlsProps } from './components/Map/MapControls';

export { MapLegend } from './components/Map/MapLegend';
export type { MapLegendProps } from './components/Map/MapLegend';

// Layer Components
export { LayerPanel } from './components/Layers/LayerPanel';
export type { LayerPanelProps } from './components/Layers/LayerPanel';

export { LayerItem } from './components/Layers/LayerItem';
export type { LayerItemProps } from './components/Layers/LayerItem';

export { LayerStyleEditor } from './components/Layers/LayerStyleEditor';
export type { LayerStyleEditorProps } from './components/Layers/LayerStyleEditor';

// Tool Components
export { MeasureTool } from './components/Tools/MeasureTool';
export type { MeasureToolProps } from './components/Tools/MeasureTool';

export { DrawTool } from './components/Tools/DrawTool';
export type { DrawToolProps } from './components/Tools/DrawTool';

export { SelectTool } from './components/Tools/SelectTool';
export type { SelectToolProps } from './components/Tools/SelectTool';

// Analysis Components
export { BufferPanel } from './components/Analysis/BufferPanel';
export type { BufferPanelProps } from './components/Analysis/BufferPanel';

export { QueryBuilder } from './components/Analysis/QueryBuilder';
export type { QueryBuilderProps } from './components/Analysis/QueryBuilder';

// Data Components
export { AttributeTable } from './components/Data/AttributeTable';
export type { AttributeTableProps } from './components/Data/AttributeTable';

export { FeatureEditor } from './components/Data/FeatureEditor';
export type { FeatureEditorProps } from './components/Data/FeatureEditor';

// Navigation Components
export { Sidebar } from './components/Navigation/Sidebar';
export type { SidebarProps } from './components/Navigation/Sidebar';

export { Toolbar } from './components/Navigation/Toolbar';
export type { ToolbarProps } from './components/Navigation/Toolbar';

// Types
export type {
  // Core types
  Coordinate,
  WebMercatorCoordinate,
  BoundingBox,
  GeometryType,
  Geometry,

  // Map types
  MapViewState,
  MapControlType,
  MapInteractionMode,
  MapEvent,
  MapEventHandlers,

  // Layer types
  LayerType,
  LayerStyle,
  LayerConfig,
  LayerGroup,

  // Feature types
  FeatureProperties,
  Feature,
  FeatureCollection,

  // Tool types
  DrawingToolType,
  MeasurementType,
  MeasurementResult,
  SelectionMode,
  SelectionResult,

  // Analysis types
  BufferParams,
  SpatialRelation,
  QueryFilter,
  QueryResult,

  // UI types
  BaseComponentProps,
  PanelProps,
  ToolbarItem,
  ThemeConfig,

  // Export types
  ExportFormat,
  ExportOptions,

  // Utility types
  AsyncResult,
  ValidationResult,
  PerformanceMetrics,
  EventHandler,

  // Store types
  MapStore,
  LayerStore,
} from './types';

/**
 * Version information
 */
export const VERSION = '0.2.5';

/**
 * Library metadata
 */
export const LIBRARY_INFO = {
  name: '@meridian/ui-components',
  version: VERSION,
  description: 'Enterprise React/TypeScript UI component library for Meridian GIS Platform',
  author: 'Meridian GIS Platform Team',
  license: 'MIT',
} as const;
