/**
 * Enterprise Visualization Engine
 * Advanced visualization library for $983M Enterprise SaaS Platform v0.5
 *
 * @packageDocumentation
 */

// ============================================================================
// Type Exports
// ============================================================================
export * from './types';

// ============================================================================
// 2D Chart Components
// ============================================================================
export { BarChart } from './charts/BarChart';
export { LineChart } from './charts/LineChart';
export { PieChart } from './charts/PieChart';
export { ScatterPlot } from './charts/ScatterPlot';
export { HeatMap } from './charts/HeatMap';
export { TreeMap } from './charts/TreeMap';
export { SankeyDiagram } from './charts/SankeyDiagram';
export { NetworkGraph } from './charts/NetworkGraph';

// ============================================================================
// 3D Visualization Components
// ============================================================================
export { Scene3D } from './3d/Scene3D';
export { DataVisualization3D } from './3d/DataVisualization3D';
export { GlobeVisualization } from './3d/GlobeVisualization';

// ============================================================================
// Animation
// ============================================================================
export {
  AnimationEngine,
  animationEngine,
  Easing,
} from './animation/AnimationEngine';

export {
  Interpolators,
  colorInterpolator,
  positionInterpolator,
  circularInterpolator,
  bezierInterpolator,
  springInterpolator,
  elasticInterpolator,
  arrayInterpolator,
  objectInterpolator,
  pathInterpolator,
  logInterpolator,
  angleInterpolator,
  steppedInterpolator,
  overshootInterpolator,
  anticipateInterpolator,
  wiggleInterpolator,
  smoothStepInterpolator,
  smootherStepInterpolator,
  noiseInterpolator,
} from './animation/Interpolators';

// ============================================================================
// Interaction
// ============================================================================
export { ZoomPan, zoomPan } from './interaction/ZoomPan';

export {
  Tooltip,
  useTooltip,
  withTooltip,
  createDataTooltip,
  createTimeSeriesTooltip,
  createNetworkTooltip,
} from './interaction/Tooltip';

// ============================================================================
// Themes
// ============================================================================
export {
  ThemeManager,
  themeManager,
  themes,
  lightTheme,
  darkTheme,
  highContrastTheme,
  pastelTheme,
  corporateTheme,
} from './themes/ChartTheme';

export type { ChartTheme } from './themes/ChartTheme';

// ============================================================================
// Version
// ============================================================================
export const VERSION = '0.5.0';

// ============================================================================
// Default Exports (for convenience)
// ============================================================================
import { BarChart } from './charts/BarChart';
import { LineChart } from './charts/LineChart';
import { PieChart } from './charts/PieChart';
import { ScatterPlot } from './charts/ScatterPlot';
import { HeatMap } from './charts/HeatMap';
import { TreeMap } from './charts/TreeMap';
import { SankeyDiagram } from './charts/SankeyDiagram';
import { NetworkGraph } from './charts/NetworkGraph';
import { Scene3D } from './3d/Scene3D';
import { DataVisualization3D } from './3d/DataVisualization3D';
import { GlobeVisualization } from './3d/GlobeVisualization';
import { AnimationEngine, animationEngine, Easing } from './animation/AnimationEngine';
import { Interpolators } from './animation/Interpolators';
import { ZoomPan, zoomPan } from './interaction/ZoomPan';
import { Tooltip, useTooltip } from './interaction/Tooltip';
import { themeManager, themes } from './themes/ChartTheme';

/**
 * Collection of all 2D chart components
 */
export const Charts = {
  BarChart,
  LineChart,
  PieChart,
  ScatterPlot,
  HeatMap,
  TreeMap,
  SankeyDiagram,
  NetworkGraph,
};

/**
 * Collection of all 3D visualization components
 */
export const Visualizations3D = {
  Scene3D,
  DataVisualization3D,
  GlobeVisualization,
};

/**
 * Collection of animation utilities
 */
export const Animation = {
  AnimationEngine,
  animationEngine,
  Easing,
  Interpolators,
};

/**
 * Collection of interaction utilities
 */
export const Interaction = {
  ZoomPan,
  zoomPan,
  Tooltip,
  useTooltip,
};

/**
 * Collection of theming utilities
 */
export const Theming = {
  themeManager,
  themes,
};

/**
 * Main visualization library object
 */
export const EnterpriseVisualization = {
  Charts,
  Visualizations3D,
  Animation,
  Interaction,
  Theming,
  VERSION,
};

// Default export
export default EnterpriseVisualization;
