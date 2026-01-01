/**
 * TreeMap Component - Hierarchical Data Visualization
 * @module @harborgrid/enterprise-analytics/components/charts
 */

import type { VisualizationConfig } from '../../types';

export interface TreeMapProps<T = Record<string, unknown>> {
  data: T[];
  config: VisualizationConfig;
  width?: number;
  height?: number;
}
