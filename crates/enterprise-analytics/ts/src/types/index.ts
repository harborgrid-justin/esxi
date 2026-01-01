/**
 * Enterprise Analytics & Business Intelligence Dashboard - Core Types
 * @module @harborgrid/enterprise-analytics/types
 */

import { z } from 'zod';

// ============================================================================
// Data Source Types
// ============================================================================

export enum DataSourceType {
  SQL = 'sql',
  REST_API = 'rest_api',
  GRAPHQL = 'graphql',
  CSV = 'csv',
  JSON = 'json',
  WEBSOCKET = 'websocket',
  ELASTICSEARCH = 'elasticsearch',
  MONGODB = 'mongodb',
  REDIS = 'redis',
  CUSTOM = 'custom',
}

export interface DataSourceConfig {
  id: string;
  name: string;
  type: DataSourceType;
  connectionString?: string;
  credentials?: {
    username?: string;
    password?: string;
    apiKey?: string;
    token?: string;
  };
  headers?: Record<string, string>;
  timeout?: number;
  retryAttempts?: number;
  cacheTTL?: number;
  metadata?: Record<string, unknown>;
}

export interface DataSource extends DataSourceConfig {
  createdAt: Date;
  updatedAt: Date;
  isActive: boolean;
  lastSync?: Date;
  schema?: DataSchema;
}

export interface DataSchema {
  tables?: Array<{
    name: string;
    columns: ColumnDefinition[];
    primaryKey?: string[];
    foreignKeys?: ForeignKey[];
  }>;
  fields?: ColumnDefinition[];
}

export interface ColumnDefinition {
  name: string;
  type: DataType;
  nullable?: boolean;
  unique?: boolean;
  description?: string;
  metadata?: Record<string, unknown>;
}

export interface ForeignKey {
  column: string;
  referencedTable: string;
  referencedColumn: string;
}

export enum DataType {
  STRING = 'string',
  NUMBER = 'number',
  INTEGER = 'integer',
  FLOAT = 'float',
  BOOLEAN = 'boolean',
  DATE = 'date',
  DATETIME = 'datetime',
  TIMESTAMP = 'timestamp',
  JSON = 'json',
  ARRAY = 'array',
  OBJECT = 'object',
  BINARY = 'binary',
  UNKNOWN = 'unknown',
}

// ============================================================================
// Query Types
// ============================================================================

export interface Query {
  id: string;
  name: string;
  dataSourceId: string;
  dimensions: Dimension[];
  metrics: Metric[];
  filters: Filter[];
  sort?: SortClause[];
  limit?: number;
  offset?: number;
  groupBy?: string[];
  having?: Filter[];
  timeRange?: TimeRange;
  refreshInterval?: number;
  cache?: boolean;
  cacheTTL?: number;
  metadata?: Record<string, unknown>;
}

export interface Dimension {
  id: string;
  field: string;
  alias?: string;
  type: DataType;
  format?: string;
  bucket?: BucketConfig;
}

export interface BucketConfig {
  type: 'date' | 'numeric' | 'text';
  interval?: string; // e.g., '1h', '1d', '1M'
  ranges?: Array<{ from: number; to: number; label: string }>;
  customBuckets?: string[];
}

export interface Metric {
  id: string;
  field: string;
  aggregation: Aggregation;
  alias?: string;
  format?: string;
  precision?: number;
  calculation?: string; // Formula for calculated metrics
}

export enum Aggregation {
  COUNT = 'count',
  COUNT_DISTINCT = 'count_distinct',
  SUM = 'sum',
  AVG = 'avg',
  MIN = 'min',
  MAX = 'max',
  MEDIAN = 'median',
  PERCENTILE = 'percentile',
  STDDEV = 'stddev',
  VARIANCE = 'variance',
  FIRST = 'first',
  LAST = 'last',
  CUSTOM = 'custom',
}

export interface Filter {
  id: string;
  field: string;
  operator: FilterOperator;
  value: unknown;
  values?: unknown[];
  caseSensitive?: boolean;
  negate?: boolean;
  condition?: 'and' | 'or';
}

export enum FilterOperator {
  EQUALS = 'eq',
  NOT_EQUALS = 'ne',
  GREATER_THAN = 'gt',
  GREATER_THAN_OR_EQUAL = 'gte',
  LESS_THAN = 'lt',
  LESS_THAN_OR_EQUAL = 'lte',
  IN = 'in',
  NOT_IN = 'not_in',
  CONTAINS = 'contains',
  NOT_CONTAINS = 'not_contains',
  STARTS_WITH = 'starts_with',
  ENDS_WITH = 'ends_with',
  REGEX = 'regex',
  IS_NULL = 'is_null',
  IS_NOT_NULL = 'is_not_null',
  BETWEEN = 'between',
}

export interface SortClause {
  field: string;
  direction: 'asc' | 'desc';
  nullsFirst?: boolean;
}

export interface TimeRange {
  start: Date | string;
  end: Date | string;
  timezone?: string;
}

// ============================================================================
// Visualization Types
// ============================================================================

export enum VisualizationType {
  LINE_CHART = 'line_chart',
  BAR_CHART = 'bar_chart',
  PIE_CHART = 'pie_chart',
  DONUT_CHART = 'donut_chart',
  AREA_CHART = 'area_chart',
  SCATTER_PLOT = 'scatter_plot',
  HEAT_MAP = 'heat_map',
  TREE_MAP = 'tree_map',
  SANKEY_DIAGRAM = 'sankey_diagram',
  FUNNEL_CHART = 'funnel_chart',
  GEO_MAP = 'geo_map',
  TABLE = 'table',
  METRIC_CARD = 'metric_card',
  GAUGE = 'gauge',
  WATERFALL = 'waterfall',
  BULLET_CHART = 'bullet_chart',
  RADAR_CHART = 'radar_chart',
  CUSTOM = 'custom',
}

export interface Visualization {
  id: string;
  name: string;
  type: VisualizationType;
  queryId: string;
  config: VisualizationConfig;
  layout?: LayoutConfig;
  interactions?: InteractionConfig;
  createdAt: Date;
  updatedAt: Date;
}

export interface VisualizationConfig {
  // Chart-specific configuration
  xAxis?: AxisConfig;
  yAxis?: AxisConfig;
  zAxis?: AxisConfig;
  colorScale?: ColorScaleConfig;
  legend?: LegendConfig;
  tooltip?: TooltipConfig;
  annotations?: Annotation[];
  theme?: ThemeConfig;

  // Chart-type specific options
  stacked?: boolean;
  normalized?: boolean;
  showValues?: boolean;
  showGrid?: boolean;
  smooth?: boolean;
  fillOpacity?: number;

  // Table-specific
  columns?: TableColumn[];
  pagination?: PaginationConfig;

  // Map-specific
  mapType?: 'choropleth' | 'bubble' | 'heatmap' | 'marker';
  geoField?: string;
  projection?: string;

  // Custom configuration
  custom?: Record<string, unknown>;
}

export interface AxisConfig {
  label?: string;
  field: string;
  scale?: 'linear' | 'log' | 'time' | 'ordinal' | 'band';
  domain?: [number, number] | string[];
  range?: [number, number];
  format?: string;
  tickCount?: number;
  grid?: boolean;
  zero?: boolean;
}

export interface ColorScaleConfig {
  type: 'categorical' | 'sequential' | 'diverging';
  scheme?: string;
  domain?: unknown[];
  range?: string[];
  customColors?: Record<string, string>;
}

export interface LegendConfig {
  show: boolean;
  position: 'top' | 'right' | 'bottom' | 'left';
  orientation?: 'horizontal' | 'vertical';
  title?: string;
  interactive?: boolean;
}

export interface TooltipConfig {
  show: boolean;
  format?: string;
  fields?: string[];
  customTemplate?: string;
}

export interface Annotation {
  type: 'line' | 'band' | 'text' | 'point';
  value?: unknown;
  range?: [unknown, unknown];
  axis?: 'x' | 'y';
  label?: string;
  style?: {
    color?: string;
    strokeWidth?: number;
    strokeDash?: number[];
    opacity?: number;
  };
}

export interface ThemeConfig {
  colors?: string[];
  fontFamily?: string;
  fontSize?: number;
  backgroundColor?: string;
  gridColor?: string;
  textColor?: string;
}

export interface TableColumn {
  field: string;
  header: string;
  width?: number;
  sortable?: boolean;
  filterable?: boolean;
  format?: string;
  align?: 'left' | 'center' | 'right';
  render?: (value: unknown, row: Record<string, unknown>) => string;
}

export interface PaginationConfig {
  pageSize: number;
  pageSizeOptions?: number[];
  showTotal?: boolean;
}

export interface LayoutConfig {
  x: number;
  y: number;
  width: number;
  height: number;
  minWidth?: number;
  minHeight?: number;
  maxWidth?: number;
  maxHeight?: number;
}

export interface InteractionConfig {
  drillDown?: DrillDownConfig;
  crossFilter?: boolean;
  zoom?: boolean;
  pan?: boolean;
  brush?: boolean;
  onClick?: string; // Event handler ID
  onHover?: string; // Event handler ID
}

// ============================================================================
// Dashboard Types
// ============================================================================

export interface Dashboard {
  id: string;
  name: string;
  description?: string;
  widgets: Widget[];
  filters: DashboardFilter[];
  layout: DashboardLayout;
  theme?: ThemeConfig;
  refreshInterval?: number;
  permissions?: PermissionConfig[];
  createdAt: Date;
  updatedAt: Date;
  createdBy: string;
  tags?: string[];
  isPublic?: boolean;
}

export interface Widget {
  id: string;
  visualizationId: string;
  layout: LayoutConfig;
  title?: string;
  description?: string;
  showTitle?: boolean;
  showDescription?: boolean;
}

export interface DashboardFilter {
  id: string;
  field: string;
  type: FilterType;
  label: string;
  defaultValue?: unknown;
  options?: FilterOption[];
  multiple?: boolean;
  required?: boolean;
  dependsOn?: string; // Filter ID that this filter depends on
}

export enum FilterType {
  SELECT = 'select',
  MULTI_SELECT = 'multi_select',
  DATE_RANGE = 'date_range',
  DATE = 'date',
  NUMERIC_RANGE = 'numeric_range',
  TEXT_INPUT = 'text_input',
  CHECKBOX = 'checkbox',
  RADIO = 'radio',
}

export interface FilterOption {
  label: string;
  value: unknown;
}

export interface DashboardLayout {
  type: 'grid' | 'flex' | 'fixed';
  columns?: number;
  rowHeight?: number;
  gap?: number;
  responsive?: boolean;
  breakpoints?: Record<string, LayoutBreakpoint>;
}

export interface LayoutBreakpoint {
  columns: number;
  rowHeight?: number;
}

export interface PermissionConfig {
  userId?: string;
  roleId?: string;
  permissions: Permission[];
}

export enum Permission {
  VIEW = 'view',
  EDIT = 'edit',
  DELETE = 'delete',
  SHARE = 'share',
  EXPORT = 'export',
}

// ============================================================================
// Advanced Analytics Types
// ============================================================================

export interface DrillDownConfig {
  enabled: boolean;
  levels: DrillDownLevel[];
  breadcrumb?: boolean;
}

export interface DrillDownLevel {
  dimension: string;
  label: string;
  queryId?: string;
}

export interface CrossFilterConfig {
  enabled: boolean;
  targetWidgets?: string[];
  filterField: string;
}

export interface PivotConfig {
  rows: string[];
  columns: string[];
  values: PivotValue[];
  aggregations?: Record<string, Aggregation>;
}

export interface PivotValue {
  field: string;
  aggregation: Aggregation;
  format?: string;
}

// ============================================================================
// Data Cube / OLAP Types
// ============================================================================

export interface DataCube {
  id: string;
  name: string;
  dataSourceId: string;
  dimensions: CubeDimension[];
  measures: CubeMeasure[];
  hierarchies?: Hierarchy[];
  calculated?: CalculatedMember[];
}

export interface CubeDimension {
  id: string;
  name: string;
  field: string;
  type: DataType;
  hierarchy?: string;
  level?: number;
}

export interface CubeMeasure {
  id: string;
  name: string;
  field: string;
  aggregation: Aggregation;
  format?: string;
}

export interface Hierarchy {
  id: string;
  name: string;
  levels: HierarchyLevel[];
}

export interface HierarchyLevel {
  id: string;
  name: string;
  field: string;
  order: number;
}

export interface CalculatedMember {
  id: string;
  name: string;
  formula: string;
  format?: string;
  type: 'dimension' | 'measure';
}

// ============================================================================
// Query Result Types
// ============================================================================

export interface QueryResult<T = Record<string, unknown>> {
  data: T[];
  metadata: QueryMetadata;
  cached?: boolean;
  executionTime?: number;
}

export interface QueryMetadata {
  rowCount: number;
  columnCount: number;
  columns: ColumnMetadata[];
  totalRows?: number; // For pagination
  hasMore?: boolean;
  queryId?: string;
  executedAt: Date;
}

export interface ColumnMetadata {
  name: string;
  type: DataType;
  nullable?: boolean;
  stats?: ColumnStats;
}

export interface ColumnStats {
  min?: number;
  max?: number;
  avg?: number;
  sum?: number;
  distinct?: number;
  nullCount?: number;
}

// ============================================================================
// Export Types
// ============================================================================

export enum ExportFormat {
  PDF = 'pdf',
  EXCEL = 'excel',
  CSV = 'csv',
  JSON = 'json',
  PNG = 'png',
  SVG = 'svg',
}

export interface ExportConfig {
  format: ExportFormat;
  filename?: string;
  includeMetadata?: boolean;
  orientation?: 'portrait' | 'landscape';
  pageSize?: 'A4' | 'Letter' | 'Legal';
  compression?: boolean;
}

// ============================================================================
// Scheduling Types
// ============================================================================

export interface ScheduleConfig {
  id: string;
  name: string;
  dashboardId?: string;
  queryId?: string;
  cron: string;
  timezone?: string;
  recipients: string[];
  format: ExportFormat;
  enabled: boolean;
  lastRun?: Date;
  nextRun?: Date;
  filters?: Record<string, unknown>;
}

// ============================================================================
// Validation Schemas
// ============================================================================

export const dataSourceSchema = z.object({
  id: z.string(),
  name: z.string().min(1),
  type: z.nativeEnum(DataSourceType),
  connectionString: z.string().optional(),
  credentials: z.object({
    username: z.string().optional(),
    password: z.string().optional(),
    apiKey: z.string().optional(),
    token: z.string().optional(),
  }).optional(),
  headers: z.record(z.string()).optional(),
  timeout: z.number().positive().optional(),
  retryAttempts: z.number().int().nonnegative().optional(),
  cacheTTL: z.number().positive().optional(),
  metadata: z.record(z.unknown()).optional(),
});

export const querySchema = z.object({
  id: z.string(),
  name: z.string().min(1),
  dataSourceId: z.string(),
  dimensions: z.array(z.any()),
  metrics: z.array(z.any()),
  filters: z.array(z.any()),
  sort: z.array(z.any()).optional(),
  limit: z.number().int().positive().optional(),
  offset: z.number().int().nonnegative().optional(),
  groupBy: z.array(z.string()).optional(),
  cache: z.boolean().optional(),
  cacheTTL: z.number().positive().optional(),
});

export const visualizationSchema = z.object({
  id: z.string(),
  name: z.string().min(1),
  type: z.nativeEnum(VisualizationType),
  queryId: z.string(),
  config: z.record(z.unknown()),
});

export const dashboardSchema = z.object({
  id: z.string(),
  name: z.string().min(1),
  description: z.string().optional(),
  widgets: z.array(z.any()),
  filters: z.array(z.any()),
  layout: z.record(z.unknown()),
  refreshInterval: z.number().positive().optional(),
  isPublic: z.boolean().optional(),
});
