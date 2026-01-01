/**
 * Advanced Visualization Engine - TypeScript Type Definitions
 * Enterprise SaaS Platform v0.5
 */

import { ScaleLinear, ScaleTime, ScaleBand, ScaleOrdinal } from 'd3-scale';
import { Selection } from 'd3-selection';
import { Transition } from 'd3-transition';
import * as THREE from 'three';

// ============================================================================
// Base Types
// ============================================================================

export interface Dimensions {
  width: number;
  height: number;
  margin?: Margin;
}

export interface Margin {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

export interface Position {
  x: number;
  y: number;
}

export interface Position3D extends Position {
  z: number;
}

export interface Color {
  r: number;
  g: number;
  b: number;
  a?: number;
}

export type ColorValue = string | Color;

// ============================================================================
// Data Types
// ============================================================================

export interface DataPoint {
  id?: string | number;
  label?: string;
  value: number;
  category?: string;
  timestamp?: Date | string;
  metadata?: Record<string, unknown>;
}

export interface TimeSeriesData extends DataPoint {
  timestamp: Date | string;
}

export interface MultiSeriesData {
  series: string;
  data: DataPoint[];
}

export interface HierarchicalData {
  name: string;
  value?: number;
  children?: HierarchicalData[];
  metadata?: Record<string, unknown>;
}

export interface NetworkNode {
  id: string | number;
  label?: string;
  group?: string;
  value?: number;
  x?: number;
  y?: number;
  metadata?: Record<string, unknown>;
}

export interface NetworkLink {
  source: string | number | NetworkNode;
  target: string | number | NetworkNode;
  value?: number;
  label?: string;
  metadata?: Record<string, unknown>;
}

export interface NetworkData {
  nodes: NetworkNode[];
  links: NetworkLink[];
}

export interface FlowNode {
  id: string | number;
  label?: string;
  category?: string;
}

export interface FlowLink {
  source: string | number;
  target: string | number;
  value: number;
  label?: string;
}

export interface FlowData {
  nodes: FlowNode[];
  links: FlowLink[];
}

export interface GeoPoint {
  lat: number;
  lng: number;
  value?: number;
  label?: string;
  metadata?: Record<string, unknown>;
}

// ============================================================================
// Chart Configuration
// ============================================================================

export interface ChartConfig {
  dimensions: Dimensions;
  theme?: ThemeConfig;
  animation?: AnimationConfig;
  interaction?: InteractionConfig;
  accessibility?: AccessibilityConfig;
}

export interface ThemeConfig {
  colorScheme?: string[];
  backgroundColor?: ColorValue;
  textColor?: ColorValue;
  gridColor?: ColorValue;
  fontFamily?: string;
  fontSize?: number;
}

export interface AnimationConfig {
  duration?: number;
  delay?: number;
  easing?: string | ((t: number) => number);
  enabled?: boolean;
}

export interface InteractionConfig {
  zoom?: boolean;
  pan?: boolean;
  tooltip?: boolean;
  brush?: boolean;
  click?: boolean;
  hover?: boolean;
}

export interface AccessibilityConfig {
  ariaLabel?: string;
  ariaDescription?: string;
  keyboardNavigation?: boolean;
  screenReaderOptimized?: boolean;
}

// ============================================================================
// Chart-Specific Types
// ============================================================================

export interface BarChartConfig extends ChartConfig {
  orientation?: 'vertical' | 'horizontal';
  barPadding?: number;
  grouped?: boolean;
  stacked?: boolean;
}

export interface LineChartConfig extends ChartConfig {
  curve?: 'linear' | 'monotone' | 'step' | 'basis' | 'cardinal';
  showPoints?: boolean;
  showArea?: boolean;
  showGrid?: boolean;
  interpolation?: string;
}

export interface PieChartConfig extends ChartConfig {
  innerRadius?: number;
  outerRadius?: number;
  padAngle?: number;
  startAngle?: number;
  endAngle?: number;
  showLabels?: boolean;
}

export interface ScatterPlotConfig extends ChartConfig {
  pointRadius?: number;
  showRegression?: boolean;
  regressionType?: 'linear' | 'polynomial' | 'exponential';
  colorByCategory?: boolean;
}

export interface HeatMapConfig extends ChartConfig {
  colorScale?: string[];
  cellPadding?: number;
  showValues?: boolean;
  showLegend?: boolean;
}

export interface TreeMapConfig extends ChartConfig {
  paddingInner?: number;
  paddingOuter?: number;
  tile?: 'binary' | 'squarify' | 'slice' | 'dice';
  showLabels?: boolean;
}

export interface NetworkGraphConfig extends ChartConfig {
  linkDistance?: number;
  linkStrength?: number;
  chargeStrength?: number;
  showLabels?: boolean;
  showArrows?: boolean;
}

// ============================================================================
// 3D Visualization Types
// ============================================================================

export interface Scene3DConfig {
  camera?: CameraConfig;
  lighting?: LightingConfig;
  controls?: ControlsConfig;
  renderer?: RendererConfig;
}

export interface CameraConfig {
  type?: 'perspective' | 'orthographic';
  fov?: number;
  near?: number;
  far?: number;
  position?: Position3D;
  lookAt?: Position3D;
}

export interface LightingConfig {
  ambient?: {
    color: ColorValue;
    intensity: number;
  };
  directional?: Array<{
    color: ColorValue;
    intensity: number;
    position: Position3D;
  }>;
  point?: Array<{
    color: ColorValue;
    intensity: number;
    position: Position3D;
    distance?: number;
  }>;
}

export interface ControlsConfig {
  enableRotate?: boolean;
  enableZoom?: boolean;
  enablePan?: boolean;
  autoRotate?: boolean;
  autoRotateSpeed?: number;
}

export interface RendererConfig {
  antialias?: boolean;
  alpha?: boolean;
  shadowMap?: boolean;
  pixelRatio?: number;
}

export interface DataVisualization3DConfig extends Scene3DConfig {
  dataType?: 'bars' | 'scatter' | 'surface' | 'network';
  heightScale?: number;
  colorScale?: string[];
}

export interface GlobeConfig extends Scene3DConfig {
  radius?: number;
  segments?: number;
  textureUrl?: string;
  showAtmosphere?: boolean;
  rotationSpeed?: number;
}

// ============================================================================
// Animation Types
// ============================================================================

export type D3Selection = Selection<SVGElement, unknown, null, undefined>;
export type D3Transition = Transition<SVGElement, unknown, null, undefined>;

export interface AnimationOptions {
  duration: number;
  delay?: number;
  easing?: (t: number) => number;
  onStart?: () => void;
  onEnd?: () => void;
  onUpdate?: (progress: number) => void;
}

export interface InterpolatorFunction<T> {
  (t: number): T;
}

export interface CustomInterpolator<T> {
  interpolate: (start: T, end: T) => InterpolatorFunction<T>;
}

// ============================================================================
// Interaction Types
// ============================================================================

export interface ZoomPanConfig {
  minZoom?: number;
  maxZoom?: number;
  enableZoom?: boolean;
  enablePan?: boolean;
  wheelSensitivity?: number;
  panExtent?: [[number, number], [number, number]];
}

export interface ZoomPanState {
  scale: number;
  translateX: number;
  translateY: number;
}

export interface TooltipConfig {
  position?: 'mouse' | 'fixed';
  offset?: Position;
  showDelay?: number;
  hideDelay?: number;
  maxWidth?: number;
  className?: string;
}

export interface TooltipData {
  title?: string;
  content: React.ReactNode | string;
  position: Position;
  visible: boolean;
}

// ============================================================================
// Event Types
// ============================================================================

export interface ChartEvent<T = unknown> {
  type: 'click' | 'hover' | 'zoom' | 'pan' | 'brush';
  data?: T;
  position?: Position;
  target?: EventTarget;
  originalEvent?: Event;
}

export type EventHandler<T = unknown> = (event: ChartEvent<T>) => void;

// ============================================================================
// Scale Types
// ============================================================================

export type Scale =
  | ScaleLinear<number, number>
  | ScaleTime<number, number>
  | ScaleBand<string>
  | ScaleOrdinal<string, string>;

export interface ScaleConfig {
  type: 'linear' | 'time' | 'band' | 'ordinal' | 'log' | 'pow';
  domain?: unknown[];
  range?: unknown[];
  padding?: number;
}

// ============================================================================
// Export Types
// ============================================================================

export interface ExportConfig {
  format: 'png' | 'svg' | 'pdf' | 'json';
  quality?: number;
  scale?: number;
  filename?: string;
}

// ============================================================================
// Performance Types
// ============================================================================

export interface PerformanceMetrics {
  renderTime: number;
  dataPoints: number;
  fps?: number;
  memoryUsage?: number;
}

export interface OptimizationConfig {
  useWebGL?: boolean;
  enableCaching?: boolean;
  lazyLoading?: boolean;
  virtualScrolling?: boolean;
  decimation?: {
    enabled: boolean;
    algorithm: 'lttb' | 'minmax';
    threshold: number;
  };
}

// ============================================================================
// Utility Types
// ============================================================================

export type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export type RequireAtLeastOne<T, Keys extends keyof T = keyof T> = Pick<
  T,
  Exclude<keyof T, Keys>
> &
  {
    [K in Keys]-?: Required<Pick<T, K>> & Partial<Pick<T, Exclude<Keys, K>>>;
  }[Keys];

export type Nullable<T> = T | null;
export type Optional<T> = T | undefined;

// ============================================================================
// React Component Props
// ============================================================================

export interface BaseChartProps<T = unknown, C = ChartConfig> {
  data: T;
  config: C;
  className?: string;
  style?: React.CSSProperties;
  onEvent?: EventHandler;
  ref?: React.Ref<HTMLDivElement>;
}
