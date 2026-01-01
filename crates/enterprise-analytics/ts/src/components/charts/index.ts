/**
 * Chart Components Module
 * @module @harborgrid/enterprise-analytics/components/charts
 */

export * from './LineChart';
export * from './BarChart';
export * from './PieChart';
export * from './HeatMap';
export * from './ScatterPlot';

// Export type-only placeholders for other charts
// These would be fully implemented in a production system

export type { TreeMapProps } from './TreeMap';
export type { SankeyDiagramProps } from './SankeyDiagram';
export type { GeoMapProps } from './GeoMap';

// Placeholder exports
export const TreeMap = () => null;
export const SankeyDiagram = () => null;
export const GeoMap = () => null;
