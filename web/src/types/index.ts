// Geometry Types
export interface Point {
  type: 'Point';
  coordinates: [number, number];
}

export interface LineString {
  type: 'LineString';
  coordinates: [number, number][];
}

export interface Polygon {
  type: 'Polygon';
  coordinates: [number, number][][];
}

export type Geometry = Point | LineString | Polygon;

// Feature Types
export interface Feature {
  id: string;
  type: 'Feature';
  geometry: Geometry;
  properties: Record<string, unknown>;
}

export interface FeatureCollection {
  type: 'FeatureCollection';
  features: Feature[];
}

// Layer Types
export interface Layer {
  id: string;
  name: string;
  type: 'vector' | 'raster' | 'tile';
  visible: boolean;
  opacity: number;
  source: LayerSource;
  style?: LayerStyle;
  metadata?: Record<string, unknown>;
}

export interface LayerSource {
  type: 'geojson' | 'vector' | 'raster' | 'wms' | 'wmts';
  url?: string;
  data?: FeatureCollection;
  tiles?: string[];
  bounds?: [number, number, number, number];
}

export interface LayerStyle {
  fillColor?: string;
  fillOpacity?: number;
  strokeColor?: string;
  strokeWidth?: number;
  strokeOpacity?: number;
  radius?: number;
  icon?: string;
}

// Map State Types
export interface MapState {
  center: [number, number];
  zoom: number;
  bearing: number;
  pitch: number;
  bounds?: [number, number, number, number];
}

export interface ViewState {
  longitude: number;
  latitude: number;
  zoom: number;
  bearing?: number;
  pitch?: number;
}

// Tool Types
export type MapTool =
  | 'pan'
  | 'select'
  | 'draw-point'
  | 'draw-line'
  | 'draw-polygon'
  | 'measure-distance'
  | 'measure-area'
  | 'identify';

export interface DrawingState {
  active: boolean;
  tool: MapTool | null;
  features: Feature[];
}

// Analysis Types
export interface AnalysisParams {
  type: 'buffer' | 'intersect' | 'union' | 'difference' | 'clip';
  inputLayers: string[];
  distance?: number;
  units?: 'meters' | 'kilometers' | 'miles' | 'feet';
  outputName?: string;
}

export interface AnalysisResult {
  id: string;
  type: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  result?: FeatureCollection;
  error?: string;
  createdAt: string;
  completedAt?: string;
}

// API Response Types
export interface ApiResponse<T> {
  data: T;
  message?: string;
  status: number;
}

export interface ApiError {
  message: string;
  code: string;
  details?: Record<string, unknown>;
}

// User & Auth Types
export interface User {
  id: string;
  username: string;
  email: string;
  role: 'admin' | 'editor' | 'viewer';
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
}

// Project Types
export interface Project {
  id: string;
  name: string;
  description?: string;
  bounds?: [number, number, number, number];
  crs: string;
  layers: Layer[];
  createdAt: string;
  updatedAt: string;
}

// WebSocket Event Types
export interface WebSocketEvent {
  type: 'layer-update' | 'feature-change' | 'analysis-complete' | 'notification';
  payload: unknown;
  timestamp: string;
}

// UI State Types
export interface UIState {
  sidebarOpen: boolean;
  sidebarTab: 'layers' | 'properties' | 'analysis';
  layerPanelOpen: boolean;
  selectedFeatures: Feature[];
  activeTool: MapTool | null;
  loading: boolean;
  error: string | null;
}
