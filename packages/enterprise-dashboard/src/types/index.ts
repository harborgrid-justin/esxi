/**
 * Enterprise Dashboard Types
 * $983M SaaS Platform - TypeScript Type Definitions
 */

export type TimeRange = '1h' | '6h' | '24h' | '7d' | '30d' | '90d' | 'ytd' | 'all';

export type TrendDirection = 'up' | 'down' | 'stable';

export type AlertSeverity = 'critical' | 'high' | 'medium' | 'low' | 'info';

export type ChartType = 'line' | 'area' | 'bar' | 'pie' | 'donut' | 'scatter' | 'heatmap';

export type WidgetSize = 'small' | 'medium' | 'large' | 'xlarge';

/**
 * KPI Metric Interface
 */
export interface KPIMetric {
  id: string;
  label: string;
  value: number | string;
  previousValue?: number | string;
  unit?: string;
  format?: 'number' | 'currency' | 'percentage' | 'bytes' | 'duration';
  trend?: TrendDirection;
  trendValue?: number;
  sparklineData?: number[];
  target?: number;
  threshold?: {
    warning: number;
    critical: number;
  };
  status?: 'healthy' | 'warning' | 'critical';
  description?: string;
  icon?: string;
  color?: string;
}

/**
 * Dashboard Widget Configuration
 */
export interface DashboardWidget {
  id: string;
  type: 'kpi' | 'chart' | 'table' | 'alert' | 'activity' | 'quota' | 'custom';
  title: string;
  description?: string;
  position: {
    x: number;
    y: number;
    w: number;
    h: number;
  };
  config: Record<string, unknown>;
  refreshInterval?: number;
  dataSource?: string;
  permissions?: string[];
  visible?: boolean;
  locked?: boolean;
}

/**
 * Dashboard Layout Configuration
 */
export interface DashboardLayout {
  id: string;
  name: string;
  description?: string;
  widgets: DashboardWidget[];
  isDefault?: boolean;
  isShared?: boolean;
  owner?: string;
  permissions?: DashboardPermissions;
  createdAt: Date;
  updatedAt: Date;
  tags?: string[];
}

/**
 * Dashboard Permissions
 */
export interface DashboardPermissions {
  read: string[];
  write: string[];
  share: string[];
  delete: string[];
}

/**
 * Time Series Data Point
 */
export interface TimeSeriesDataPoint {
  timestamp: number | Date;
  value: number;
  label?: string;
  metadata?: Record<string, unknown>;
}

/**
 * Chart Data Series
 */
export interface ChartDataSeries {
  id: string;
  name: string;
  data: TimeSeriesDataPoint[];
  color?: string;
  type?: ChartType;
  visible?: boolean;
  yAxis?: 'left' | 'right';
  unit?: string;
  aggregation?: 'sum' | 'avg' | 'min' | 'max' | 'count';
}

/**
 * Revenue Analytics Data
 */
export interface RevenueData {
  period: string;
  revenue: number;
  cost: number;
  profit: number;
  margin: number;
  customers: number;
  arpu: number;
  churnRate: number;
  growthRate: number;
  forecast?: number;
  breakdown?: {
    category: string;
    amount: number;
    percentage: number;
  }[];
}

/**
 * Usage Metrics Data
 */
export interface UsageMetrics {
  timestamp: Date;
  activeUsers: number;
  apiCalls: number;
  dataTransfer: number;
  storageUsed: number;
  cpuUsage: number;
  memoryUsage: number;
  requestLatency: number;
  errorRate: number;
  successRate: number;
  peakConcurrency: number;
}

/**
 * Performance Metrics
 */
export interface PerformanceMetrics {
  timestamp: Date;
  responseTime: {
    p50: number;
    p95: number;
    p99: number;
    avg: number;
    max: number;
  };
  throughput: number;
  errorRate: number;
  availability: number;
  saturation: number;
  apdex: number;
  slowQueries: number;
  cacheHitRate: number;
}

/**
 * Geographic Distribution Data
 */
export interface GeoDataPoint {
  country: string;
  countryCode: string;
  region?: string;
  city?: string;
  coordinates: [number, number]; // [longitude, latitude]
  users: number;
  revenue: number;
  requests: number;
  latency: number;
  availability?: number;
}

/**
 * Alert Definition
 */
export interface Alert {
  id: string;
  severity: AlertSeverity;
  title: string;
  message: string;
  source: string;
  timestamp: Date;
  status: 'active' | 'acknowledged' | 'resolved';
  assignedTo?: string;
  metadata?: Record<string, unknown>;
  actions?: AlertAction[];
  impact?: {
    services: string[];
    users: number;
    revenue: number;
  };
  relatedAlerts?: string[];
}

/**
 * Alert Action
 */
export interface AlertAction {
  id: string;
  label: string;
  type: 'acknowledge' | 'resolve' | 'escalate' | 'assign' | 'custom';
  handler: string;
  confirmRequired?: boolean;
}

/**
 * Activity Log Entry
 */
export interface ActivityLogEntry {
  id: string;
  timestamp: Date;
  type: 'user' | 'system' | 'security' | 'deployment' | 'configuration';
  action: string;
  actor: {
    id: string;
    name: string;
    type: 'user' | 'service' | 'system';
  };
  resource?: {
    type: string;
    id: string;
    name: string;
  };
  description: string;
  metadata?: Record<string, unknown>;
  severity?: 'info' | 'warning' | 'error';
  ipAddress?: string;
  location?: string;
}

/**
 * Quota Usage
 */
export interface QuotaUsage {
  id: string;
  name: string;
  category: 'compute' | 'storage' | 'network' | 'api' | 'users' | 'custom';
  current: number;
  limit: number;
  unit: string;
  percentage: number;
  trend: TrendDirection;
  resetDate?: Date;
  overage?: number;
  overageCost?: number;
  warnings?: {
    level: number;
    message: string;
  }[];
  forecast?: {
    exhaustionDate: Date;
    confidence: number;
  };
}

/**
 * Real-time Data Subscription
 */
export interface DataSubscription {
  id: string;
  channel: string;
  filters?: Record<string, unknown>;
  onData: (data: unknown) => void;
  onError?: (error: Error) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

/**
 * Dashboard State
 */
export interface DashboardState {
  layout: DashboardLayout | null;
  timeRange: TimeRange;
  refreshInterval: number;
  autoRefresh: boolean;
  isLoading: boolean;
  error: string | null;
  filters: DashboardFilters;
  kpis: KPIMetric[];
  alerts: Alert[];
  activities: ActivityLogEntry[];
  quotas: QuotaUsage[];
}

/**
 * Dashboard Filters
 */
export interface DashboardFilters {
  tenants?: string[];
  regions?: string[];
  services?: string[];
  environments?: string[];
  search?: string;
  customFilters?: Record<string, unknown>;
}

/**
 * Dashboard Configuration
 */
export interface DashboardConfig {
  theme: 'light' | 'dark' | 'auto';
  density: 'comfortable' | 'compact' | 'spacious';
  animations: boolean;
  soundEffects: boolean;
  notifications: {
    desktop: boolean;
    sound: boolean;
    alertSeverities: AlertSeverity[];
  };
  defaultTimeRange: TimeRange;
  defaultRefreshInterval: number;
  chartDefaults: {
    type: ChartType;
    colors: string[];
    showLegend: boolean;
    showGrid: boolean;
    animation: boolean;
  };
}

/**
 * API Response Wrapper
 */
export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
  metadata?: {
    timestamp: Date;
    requestId: string;
    duration: number;
  };
}

/**
 * Pagination Parameters
 */
export interface PaginationParams {
  page: number;
  pageSize: number;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

/**
 * Paginated Response
 */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
}

/**
 * Export Configuration
 */
export interface ExportConfig {
  format: 'csv' | 'xlsx' | 'pdf' | 'json';
  fileName?: string;
  includeCharts?: boolean;
  dateRange?: {
    start: Date;
    end: Date;
  };
  filters?: DashboardFilters;
}

/**
 * Notification
 */
export interface Notification {
  id: string;
  type: 'info' | 'success' | 'warning' | 'error';
  title: string;
  message: string;
  timestamp: Date;
  read: boolean;
  actions?: {
    label: string;
    handler: () => void;
  }[];
  autoClose?: boolean;
  duration?: number;
}
