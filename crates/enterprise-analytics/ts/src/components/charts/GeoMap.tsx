/**
 * Geographic Map Component - Spatial Data Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import type { VisualizationConfig } from '../../types';

export interface GeoMapProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
}
