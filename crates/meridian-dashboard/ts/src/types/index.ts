/**
 * TypeScript type definitions for Meridian Dashboard
 */

export interface Dashboard {
  id: string;
  name: string;
  description?: string;
  owner_id: string;
  layout: DashboardLayout;
  widgets: Widget[];
  filters: DashboardFilter[];
  refresh_interval?: number;
  is_public: boolean;
  created_at: string;
  updated_at: string;
}

export interface DashboardLayout {
  cols: number;
  rows: number;
  breakpoints: LayoutBreakpoints;
}

export interface LayoutBreakpoints {
  lg: number;
  md: number;
  sm: number;
  xs: number;
  xxs: number;
}

export interface Widget {
  id: string;
  dashboard_id: string;
  widget_type: WidgetType;
  title: string;
  description?: string;
  position: WidgetPosition;
  config: WidgetConfig;
  data_source: DataSourceConfig;
  created_at: string;
  updated_at: string;
}

export type WidgetType = 'map' | 'chart' | 'table' | 'kpi' | 'timeline' | 'filter';

export interface WidgetPosition {
  x: number;
  y: number;
  w: number;
  h: number;
  min_w?: number;
  min_h?: number;
  max_w?: number;
  max_h?: number;
}

export type WidgetConfig =
  | MapWidgetConfig
  | ChartWidgetConfig
  | TableWidgetConfig
  | KpiWidgetConfig
  | TimelineWidgetConfig
  | FilterWidgetConfig;

export interface MapWidgetConfig {
  type: 'Map';
  center: [number, number];
  zoom: number;
  layers: string[];
}

export interface ChartWidgetConfig {
  type: 'Chart';
  chart_type: ChartType;
  x_axis: string;
  y_axis: string[];
  options: ChartOptions;
}

export type ChartType = 'line' | 'bar' | 'pie' | 'scatter' | 'heatmap' | 'area' | 'column' | 'donut';

export interface ChartOptions {
  legend: boolean;
  grid: boolean;
  tooltip: boolean;
  animation: boolean;
  stacked: boolean;
  colors?: string[];
}

export interface TableWidgetConfig {
  type: 'Table';
  columns: TableColumn[];
  pagination: boolean;
  page_size: number;
}

export interface TableColumn {
  field: string;
  header: string;
  width?: number;
  sortable: boolean;
  filterable: boolean;
  format?: string;
}

export interface KpiWidgetConfig {
  type: 'Kpi';
  metric: string;
  aggregation: AggregationType;
  comparison?: KpiComparison;
  format: string;
}

export type AggregationType = 'sum' | 'avg' | 'min' | 'max' | 'count' | 'count_distinct';

export interface KpiComparison {
  period: ComparisonPeriod;
  show_change: boolean;
  show_percentage: boolean;
}

export type ComparisonPeriod =
  | { type: 'previous_day' }
  | { type: 'previous_week' }
  | { type: 'previous_month' }
  | { type: 'previous_year' }
  | { type: 'custom'; days: number };

export interface TimelineWidgetConfig {
  type: 'Timeline';
  date_field: string;
  title_field: string;
  description_field?: string;
}

export interface FilterWidgetConfig {
  type: 'Filter';
  field: string;
  filter_type: FilterType;
  default_value?: any;
}

export type FilterType = 'select' | 'multi_select' | 'date_range' | 'number_range' | 'text';

export type DataSourceConfig =
  | SqlDataSource
  | ApiDataSource
  | StaticDataSource;

export interface SqlDataSource {
  type: 'Sql';
  connection_id: string;
  query: string;
  parameters: QueryParameter[];
}

export interface ApiDataSource {
  type: 'Api';
  url: string;
  method: string;
  headers: Record<string, string>;
  body?: string;
}

export interface StaticDataSource {
  type: 'Static';
  data: any;
}

export interface QueryParameter {
  name: string;
  value: any;
  param_type: ParameterType;
}

export type ParameterType = 'string' | 'number' | 'boolean' | 'date' | 'array';

export interface DashboardFilter {
  id: string;
  field: string;
  operator: FilterOperator;
  value: any;
  applies_to: string[];
}

export type FilterOperator =
  | 'equals'
  | 'not_equals'
  | 'greater_than'
  | 'less_than'
  | 'greater_than_or_equal'
  | 'less_than_or_equal'
  | 'contains'
  | 'starts_with'
  | 'ends_with'
  | 'in'
  | 'not_in'
  | 'between';

export interface Report {
  id: string;
  name: string;
  description?: string;
  dashboard_id: string;
  format: ReportFormat;
  schedule?: ReportSchedule;
  recipients: string[];
  created_at: string;
  updated_at: string;
}

export type ReportFormat = 'pdf' | 'excel' | 'csv' | 'json';

export interface ReportSchedule {
  cron: string;
  timezone: string;
  enabled: boolean;
}

export interface WidgetDataResponse {
  widget_id: string;
  data: any;
  timestamp: string;
}

// Grid Layout Types (from react-grid-layout)
export interface Layout {
  i: string;
  x: number;
  y: number;
  w: number;
  h: number;
  minW?: number;
  minH?: number;
  maxW?: number;
  maxH?: number;
  static?: boolean;
  isDraggable?: boolean;
  isResizable?: boolean;
}

export interface Layouts {
  lg?: Layout[];
  md?: Layout[];
  sm?: Layout[];
  xs?: Layout[];
  xxs?: Layout[];
}

// API Response Types
export interface ApiResponse<T> {
  data?: T;
  error?: string;
  status: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  page_size: number;
}

// Export Types
export interface ExportOptions {
  format: ReportFormat;
  orientation?: 'portrait' | 'landscape';
  paper_size?: 'A4' | 'Letter' | 'Legal';
  include_filters?: boolean;
  include_data?: boolean;
  include_charts?: boolean;
}
