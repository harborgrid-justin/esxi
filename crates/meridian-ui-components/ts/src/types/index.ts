/**
 * Type definitions for Meridian UI Components
 * @module @meridian/ui-components/types
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * Geographic coordinate in WGS84
 */
export interface Coordinate {
  lon: number;
  lat: number;
}

/**
 * Web Mercator coordinate (EPSG:3857)
 */
export interface WebMercatorCoordinate {
  x: number;
  y: number;
}

/**
 * Bounding box for spatial extents
 */
export interface BoundingBox {
  minLon: number;
  minLat: number;
  maxLon: number;
  maxLat: number;
}

/**
 * Geometry types supported
 */
export type GeometryType =
  | 'Point'
  | 'LineString'
  | 'Polygon'
  | 'MultiPoint'
  | 'MultiLineString'
  | 'MultiPolygon';

/**
 * Generic geometry structure
 */
export interface Geometry<T extends GeometryType = GeometryType> {
  type: T;
  coordinates: number[] | number[][] | number[][][];
}

// ============================================================================
// Map Types
// ============================================================================

/**
 * Map view state
 */
export interface MapViewState {
  center: Coordinate;
  zoom: number;
  rotation: number;
  pitch: number;
  bearing?: number;
}

/**
 * Map control types
 */
export type MapControlType =
  | 'zoom'
  | 'rotate'
  | 'pitch'
  | 'fullscreen'
  | 'geolocation'
  | 'scale';

/**
 * Map interaction mode
 */
export type MapInteractionMode =
  | 'pan'
  | 'select'
  | 'draw'
  | 'measure'
  | 'edit';

/**
 * Map event data
 */
export interface MapEvent {
  type: string;
  coordinate?: Coordinate;
  pixel?: [number, number];
  features?: Feature[];
  originalEvent?: MouseEvent | TouchEvent;
}

// ============================================================================
// Layer Types
// ============================================================================

/**
 * Layer types
 */
export type LayerType = 'vector' | 'raster' | 'tile' | 'heatmap' | 'cluster';

/**
 * Layer style configuration
 */
export interface LayerStyle {
  fillColor?: string;
  strokeColor?: string;
  strokeWidth?: number;
  opacity?: number;
  zIndex?: number;
  iconUrl?: string;
  iconSize?: [number, number];
  textField?: string;
  textFont?: string;
  textSize?: number;
  textColor?: string;
}

/**
 * Layer configuration
 */
export interface LayerConfig {
  id: string;
  name: string;
  type: LayerType;
  sourceUrl?: string;
  style: LayerStyle;
  visible: boolean;
  opacity: number;
  minZoom?: number;
  maxZoom?: number;
  metadata?: Record<string, unknown>;
}

/**
 * Layer group
 */
export interface LayerGroup {
  id: string;
  name: string;
  layers: LayerConfig[];
  visible: boolean;
  expanded?: boolean;
}

// ============================================================================
// Feature Types
// ============================================================================

/**
 * Feature properties
 */
export interface FeatureProperties {
  id: string | number;
  name?: string;
  [key: string]: unknown;
}

/**
 * GeoJSON Feature
 */
export interface Feature<
  G extends Geometry = Geometry,
  P extends FeatureProperties = FeatureProperties
> {
  type: 'Feature';
  id?: string | number;
  geometry: G;
  properties: P;
}

/**
 * GeoJSON Feature Collection
 */
export interface FeatureCollection<
  G extends Geometry = Geometry,
  P extends FeatureProperties = FeatureProperties
> {
  type: 'FeatureCollection';
  features: Feature<G, P>[];
}

// ============================================================================
// Tool Types
// ============================================================================

/**
 * Drawing tool types
 */
export type DrawingToolType =
  | 'point'
  | 'line'
  | 'polygon'
  | 'circle'
  | 'rectangle';

/**
 * Measurement types
 */
export type MeasurementType = 'distance' | 'area' | 'perimeter';

/**
 * Measurement result
 */
export interface MeasurementResult {
  type: MeasurementType;
  value: number;
  unit: string;
  geometry?: Geometry;
}

/**
 * Selection mode
 */
export type SelectionMode = 'single' | 'multiple' | 'box' | 'polygon';

/**
 * Selection result
 */
export interface SelectionResult {
  features: Feature[];
  mode: SelectionMode;
  bounds?: BoundingBox;
}

// ============================================================================
// Analysis Types
// ============================================================================

/**
 * Buffer parameters
 */
export interface BufferParams {
  distance: number;
  unit: 'meters' | 'kilometers' | 'miles' | 'feet';
  segments?: number;
}

/**
 * Spatial relation types
 */
export type SpatialRelation =
  | 'intersects'
  | 'contains'
  | 'within'
  | 'touches'
  | 'crosses'
  | 'overlaps'
  | 'disjoint';

/**
 * Query filter
 */
export interface QueryFilter {
  layerId?: string;
  geometryType?: GeometryType;
  spatialRelation?: SpatialRelation;
  geometry?: Geometry;
  attributes?: Record<string, unknown>;
  bounds?: BoundingBox;
}

/**
 * Query result
 */
export interface QueryResult {
  features: Feature[];
  count: number;
  executionTime?: number;
}

// ============================================================================
// UI Component Props
// ============================================================================

/**
 * Base component props
 */
export interface BaseComponentProps {
  className?: string;
  style?: React.CSSProperties;
  id?: string;
  'aria-label'?: string;
}

/**
 * Panel props
 */
export interface PanelProps extends BaseComponentProps {
  title?: string;
  collapsible?: boolean;
  collapsed?: boolean;
  onCollapse?: (collapsed: boolean) => void;
  children?: React.ReactNode;
}

/**
 * Toolbar item
 */
export interface ToolbarItem {
  id: string;
  label: string;
  icon?: React.ReactNode;
  onClick?: () => void;
  active?: boolean;
  disabled?: boolean;
  tooltip?: string;
}

/**
 * Theme configuration
 */
export interface ThemeConfig {
  primaryColor: string;
  secondaryColor: string;
  backgroundColor: string;
  textColor: string;
  borderColor: string;
  accentColor: string;
  darkMode: boolean;
}

// ============================================================================
// Data Export Types
// ============================================================================

/**
 * Export format
 */
export type ExportFormat = 'geojson' | 'csv' | 'kml' | 'shapefile' | 'gpx';

/**
 * Export options
 */
export interface ExportOptions {
  format: ExportFormat;
  crs?: string;
  includeAttributes?: boolean;
  filename?: string;
  features?: Feature[];
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Async result type
 */
export type AsyncResult<T, E = Error> =
  | { success: true; data: T }
  | { success: false; error: E };

/**
 * Validation result
 */
export interface ValidationResult {
  valid: boolean;
  errors?: string[];
  warnings?: string[];
}

/**
 * Performance metrics
 */
export interface PerformanceMetrics {
  framesRendered: number;
  averageFrameTime: number;
  featuresRendered: number;
  tilesLoaded: number;
  memoryUsage?: number;
}

// ============================================================================
// Event Handler Types
// ============================================================================

/**
 * Generic event handler
 */
export type EventHandler<T = void> = (event: T) => void;

/**
 * Map event handlers
 */
export interface MapEventHandlers {
  onClick?: EventHandler<MapEvent>;
  onDoubleClick?: EventHandler<MapEvent>;
  onMouseMove?: EventHandler<MapEvent>;
  onMouseEnter?: EventHandler<MapEvent>;
  onMouseLeave?: EventHandler<MapEvent>;
  onDragStart?: EventHandler<MapEvent>;
  onDrag?: EventHandler<MapEvent>;
  onDragEnd?: EventHandler<MapEvent>;
  onZoomChange?: EventHandler<number>;
  onViewChange?: EventHandler<MapViewState>;
}

// ============================================================================
// State Management Types
// ============================================================================

/**
 * Map store state
 */
export interface MapStore {
  viewState: MapViewState;
  interactionMode: MapInteractionMode;
  selectedFeatures: Feature[];
  hoveredFeature: Feature | null;
  setViewState: (viewState: Partial<MapViewState>) => void;
  setInteractionMode: (mode: MapInteractionMode) => void;
  selectFeature: (feature: Feature) => void;
  deselectFeature: (featureId: string | number) => void;
  clearSelection: () => void;
  setHoveredFeature: (feature: Feature | null) => void;
}

/**
 * Layer store state
 */
export interface LayerStore {
  layers: LayerConfig[];
  activeLayerId: string | null;
  addLayer: (layer: LayerConfig) => void;
  removeLayer: (layerId: string) => void;
  updateLayer: (layerId: string, updates: Partial<LayerConfig>) => void;
  toggleLayerVisibility: (layerId: string) => void;
  setActiveLayer: (layerId: string | null) => void;
  reorderLayers: (layerIds: string[]) => void;
}
