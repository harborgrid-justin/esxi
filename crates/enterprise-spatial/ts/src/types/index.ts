/**
 * Core GIS Spatial Types
 * Comprehensive type definitions for geospatial data structures
 */

// ============================================================================
// Coordinate and Position Types
// ============================================================================

export type Position = [number, number] | [number, number, number];
export type Position2D = [number, number];
export type Position3D = [number, number, number];

export interface Coordinates {
  x: number;
  y: number;
  z?: number;
  m?: number; // Measure value
}

// ============================================================================
// Spatial Reference and Projection Types
// ============================================================================

export interface SpatialReference {
  wkid?: number; // Well-Known ID (EPSG code)
  wkt?: string; // Well-Known Text
  proj4?: string; // Proj4 definition
  name?: string;
  type: 'geographic' | 'projected' | 'custom';
}

export interface Projection {
  code: string;
  name: string;
  proj4def: string;
  bounds?: Bounds;
  units: 'degrees' | 'meters' | 'feet' | 'us-feet';
}

export interface DatumTransform {
  from: string;
  to: string;
  definition: string;
  accuracy: number; // meters
}

// ============================================================================
// Bounds and Extent Types
// ============================================================================

export interface Bounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  minZ?: number;
  maxZ?: number;
  spatialReference?: SpatialReference;
}

export interface Extent extends Bounds {
  width: number;
  height: number;
  center: Position;
}

// ============================================================================
// Geometry Types
// ============================================================================

export type GeometryType =
  | 'Point'
  | 'LineString'
  | 'Polygon'
  | 'MultiPoint'
  | 'MultiLineString'
  | 'MultiPolygon'
  | 'GeometryCollection';

export interface Geometry {
  type: GeometryType;
  coordinates?: any;
  geometries?: Geometry[];
  spatialReference?: SpatialReference;
}

export interface Point extends Geometry {
  type: 'Point';
  coordinates: Position;
}

export interface LineString extends Geometry {
  type: 'LineString';
  coordinates: Position[];
}

export interface Polygon extends Geometry {
  type: 'Polygon';
  coordinates: Position[][];
}

export interface MultiPoint extends Geometry {
  type: 'MultiPoint';
  coordinates: Position[];
}

export interface MultiLineString extends Geometry {
  type: 'MultiLineString';
  coordinates: Position[][];
}

export interface MultiPolygon extends Geometry {
  type: 'MultiPolygon';
  coordinates: Position[][][];
}

export interface GeometryCollection extends Geometry {
  type: 'GeometryCollection';
  geometries: Geometry[];
}

// ============================================================================
// Feature Types
// ============================================================================

export interface Feature<G extends Geometry = Geometry, P = any> {
  type: 'Feature';
  id?: string | number;
  geometry: G;
  properties: P;
  bbox?: number[];
}

export interface FeatureCollection<G extends Geometry = Geometry, P = any> {
  type: 'FeatureCollection';
  features: Feature<G, P>[];
  bbox?: number[];
  crs?: SpatialReference;
}

// ============================================================================
// Layer Types
// ============================================================================

export type LayerType = 'vector' | 'raster' | 'tile' | 'group';

export interface Layer {
  id: string;
  name: string;
  type: LayerType;
  visible: boolean;
  opacity: number;
  minScale?: number;
  maxScale?: number;
  spatialReference?: SpatialReference;
  bounds?: Bounds;
  metadata?: Record<string, any>;
}

export interface VectorLayer extends Layer {
  type: 'vector';
  data: VectorData;
  style?: VectorStyle;
  interactive?: boolean;
}

export interface RasterLayer extends Layer {
  type: 'raster';
  data: RasterData;
  renderer?: RasterRenderer;
  noDataValue?: number;
}

export interface TileLayer extends Layer {
  type: 'tile';
  url: string;
  tileSize: number;
  minZoom: number;
  maxZoom: number;
  attribution?: string;
}

export interface GroupLayer extends Layer {
  type: 'group';
  layers: Layer[];
}

// ============================================================================
// Vector Data Types
// ============================================================================

export interface VectorData {
  type: 'FeatureCollection';
  features: Feature[];
  fields?: FieldDefinition[];
  spatialReference?: SpatialReference;
}

export interface FieldDefinition {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'date' | 'geometry';
  alias?: string;
  nullable?: boolean;
  editable?: boolean;
  domain?: FieldDomain;
}

export type FieldDomain = CodedValueDomain | RangeDomain;

export interface CodedValueDomain {
  type: 'coded-value';
  codedValues: Array<{ code: any; name: string }>;
}

export interface RangeDomain {
  type: 'range';
  min: number;
  max: number;
}

// ============================================================================
// Raster Data Types
// ============================================================================

export interface RasterData {
  width: number;
  height: number;
  bands: RasterBand[];
  bounds: Bounds;
  pixelSize: { x: number; y: number };
  spatialReference?: SpatialReference;
  noDataValue?: number;
}

export interface RasterBand {
  data: Float32Array | Uint8Array | Uint16Array;
  statistics?: BandStatistics;
  colorMap?: ColorMap;
  noDataValue?: number;
}

export interface BandStatistics {
  min: number;
  max: number;
  mean: number;
  stdDev: number;
  count: number;
}

export interface ColorMap {
  [value: number]: [number, number, number, number]; // RGBA
}

// ============================================================================
// Topology Types
// ============================================================================

export interface Topology {
  type: 'Topology';
  objects: { [key: string]: GeometryObject };
  arcs: Position[][];
  transform?: Transform;
}

export interface GeometryObject {
  type: string;
  arcs?: number[][];
  properties?: any;
}

export interface Transform {
  scale: [number, number];
  translate: [number, number];
}

// ============================================================================
// Style Types
// ============================================================================

export interface VectorStyle {
  point?: PointStyle;
  line?: LineStyle;
  polygon?: PolygonStyle;
}

export interface PointStyle {
  type: 'circle' | 'square' | 'triangle' | 'icon';
  size: number;
  color: string;
  opacity: number;
  strokeColor?: string;
  strokeWidth?: number;
  icon?: string;
}

export interface LineStyle {
  color: string;
  width: number;
  opacity: number;
  dashArray?: number[];
  lineCap?: 'butt' | 'round' | 'square';
  lineJoin?: 'miter' | 'round' | 'bevel';
}

export interface PolygonStyle {
  fillColor: string;
  fillOpacity: number;
  strokeColor: string;
  strokeWidth: number;
  strokeOpacity: number;
  dashArray?: number[];
}

export interface RasterRenderer {
  type: 'stretch' | 'classify' | 'unique-value' | 'colormap';
  colorRamp?: ColorRamp;
  classBreaks?: ClassBreak[];
}

export interface ColorRamp {
  type: 'algorithmic' | 'multipart';
  fromColor: [number, number, number, number];
  toColor: [number, number, number, number];
  algorithm?: 'hsv' | 'lab' | 'cie';
}

export interface ClassBreak {
  minValue: number;
  maxValue: number;
  label: string;
  color: [number, number, number, number];
}

// ============================================================================
// Spatial Analysis Types
// ============================================================================

export interface SpatialQuery {
  geometry?: Geometry;
  spatialRel?: SpatialRelationship;
  where?: string;
  fields?: string[];
  returnGeometry?: boolean;
  orderBy?: string[];
  limit?: number;
}

export type SpatialRelationship =
  | 'intersects'
  | 'contains'
  | 'within'
  | 'overlaps'
  | 'touches'
  | 'crosses'
  | 'disjoint'
  | 'equals';

export interface BufferOptions {
  distance: number;
  units: 'meters' | 'kilometers' | 'feet' | 'miles' | 'degrees';
  steps?: number;
  cap?: 'round' | 'flat' | 'square';
  endCap?: 'round' | 'flat' | 'square';
}

export interface SimplificationOptions {
  tolerance: number;
  highQuality?: boolean;
  preserveTopology?: boolean;
}

export interface ClusterOptions {
  algorithm: 'dbscan' | 'kmeans' | 'hierarchical';
  epsilon?: number;
  minPoints?: number;
  k?: number;
  distanceMetric?: 'euclidean' | 'manhattan' | 'haversine';
}

export interface InterpolationOptions {
  method: 'idw' | 'kriging' | 'spline' | 'tin';
  power?: number; // IDW power parameter
  cellSize?: number;
  searchRadius?: number;
  variogramModel?: 'spherical' | 'exponential' | 'gaussian';
}

export interface NetworkAnalysisOptions {
  algorithm: 'dijkstra' | 'astar' | 'bellman-ford';
  impedance?: string; // Field name for cost
  barriers?: Geometry[];
  restrictions?: string[];
}

// ============================================================================
// Measurement Types
// ============================================================================

export interface DistanceResult {
  distance: number;
  units: string;
  path?: Position[];
}

export interface AreaResult {
  area: number;
  units: string;
  perimeter?: number;
}

export interface ElevationProfile {
  distances: number[];
  elevations: number[];
  totalDistance: number;
  gain: number;
  loss: number;
  minElevation: number;
  maxElevation: number;
}

// ============================================================================
// Spatial Index Types
// ============================================================================

export interface SpatialIndex {
  insert(feature: Feature): void;
  remove(feature: Feature): void;
  search(bounds: Bounds): Feature[];
  clear(): void;
  all(): Feature[];
}

export interface IndexedFeature {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  feature: Feature;
}

// ============================================================================
// Geocoding Types
// ============================================================================

export interface GeocodeRequest {
  address?: string;
  location?: Position;
  maxResults?: number;
  bounds?: Bounds;
  countries?: string[];
}

export interface GeocodeResult {
  address: string;
  location: Position;
  confidence: number;
  type: string;
  bounds?: Bounds;
  attributes?: Record<string, any>;
}

// ============================================================================
// Tile Types
// ============================================================================

export interface TileCoordinate {
  x: number;
  y: number;
  z: number;
}

export interface TileRequest {
  coord: TileCoordinate;
  format: 'png' | 'jpg' | 'pbf' | 'mvt';
  layers?: string[];
}

export interface VectorTile {
  layers: { [name: string]: VectorTileLayer };
  extent: number;
}

export interface VectorTileLayer {
  name: string;
  features: VectorTileFeature[];
  extent: number;
  version: number;
}

export interface VectorTileFeature {
  id?: number;
  type: GeometryType;
  geometry: number[][];
  properties: Record<string, any>;
}

// ============================================================================
// Error Types
// ============================================================================

export class SpatialError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: any
  ) {
    super(message);
    this.name = 'SpatialError';
  }
}

export class ProjectionError extends SpatialError {
  constructor(message: string, details?: any) {
    super(message, 'PROJECTION_ERROR', details);
    this.name = 'ProjectionError';
  }
}

export class GeometryError extends SpatialError {
  constructor(message: string, details?: any) {
    super(message, 'GEOMETRY_ERROR', details);
    this.name = 'GeometryError';
  }
}

export class TopologyError extends SpatialError {
  constructor(message: string, details?: any) {
    super(message, 'TOPOLOGY_ERROR', details);
    this.name = 'TopologyError';
  }
}
