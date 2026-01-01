/**
 * Enterprise Spatial Analysis Tools
 * Advanced GIS spatial analysis and geospatial processing
 * @version 0.4.0
 * @author HarborGrid
 */

// Core Types
export * from './types';

// Geometry Operations
export { GeometryFactory } from './geometry/GeometryFactory';
export { TopologyEngine } from './geometry/TopologyEngine';
export { BufferAnalysis } from './geometry/BufferAnalysis';
export { OverlayAnalysis } from './geometry/OverlayAnalysis';
export { SimplificationEngine } from './geometry/SimplificationEngine';
export { ValidationEngine } from './geometry/ValidationEngine';

// Spatial Analysis
export { ProximityAnalysis } from './analysis/ProximityAnalysis';
export { DensityAnalysis } from './analysis/DensityAnalysis';
export { ClusterAnalysis } from './analysis/ClusterAnalysis';
export { NetworkAnalysis } from './analysis/NetworkAnalysis';
export { TerrainAnalysis } from './analysis/TerrainAnalysis';
export { ViewshedAnalysis } from './analysis/ViewshedAnalysis';

// Raster Processing
export { RasterCalculator } from './raster/RasterCalculator';
export { RasterInterpolation } from './raster/RasterInterpolation';
export { RasterClassification } from './raster/RasterClassification';
export { RasterMosaic } from './raster/RasterMosaic';
export { ContourGeneration } from './raster/ContourGeneration';

// Projection System
export { ProjectionEngine, projectionEngine } from './projection/ProjectionEngine';
export { DatumTransform, datumTransform } from './projection/DatumTransform';
export { ProjectionRegistry } from './projection/ProjectionRegistry';
export { CustomProjection } from './projection/CustomProjection';

// Services
export { GeocodingService } from './services/GeocodingService';
export { TileService } from './services/TileService';
export { FeatureService } from './services/FeatureService';
export { SpatialIndexService } from './services/SpatialIndexService';

// React Components
export { SpatialAnalyzer } from './components/SpatialAnalyzer';
export { LayerManager } from './components/LayerManager';
export { FeatureEditor } from './components/FeatureEditor';
export { QueryBuilder } from './components/QueryBuilder';
export { ResultsPanel } from './components/ResultsPanel';
export { ProjectionPicker } from './components/ProjectionPicker';
export { MeasurementTool } from './components/MeasurementTool';

// Re-export commonly used types for convenience
export type {
  Geometry,
  Feature,
  FeatureCollection,
  Position,
  Bounds,
  Layer,
  RasterData,
  SpatialQuery,
  BufferOptions,
  SimplificationOptions,
  ClusterOptions,
  InterpolationOptions,
  NetworkAnalysisOptions,
  Projection,
  SpatialReference,
} from './types';
