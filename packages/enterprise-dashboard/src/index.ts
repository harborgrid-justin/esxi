/**
 * Enterprise Dashboard UI Package
 * $983M Enterprise SaaS Platform v0.5
 *
 * Production-ready React dashboard with real-time data,
 * advanced analytics, and enterprise-grade features.
 */

// ============================================================================
// Types
// ============================================================================
export type {
  TimeRange,
  TrendDirection,
  AlertSeverity,
  ChartType,
  WidgetSize,
  KPIMetric,
  DashboardWidget,
  DashboardLayout,
  DashboardPermissions,
  TimeSeriesDataPoint,
  ChartDataSeries,
  RevenueData,
  UsageMetrics,
  PerformanceMetrics,
  GeoDataPoint,
  Alert,
  AlertAction,
  ActivityLogEntry,
  QuotaUsage,
  DataSubscription,
  DashboardState,
  DashboardFilters,
  DashboardConfig,
  APIResponse,
  PaginationParams,
  PaginatedResponse,
  ExportConfig,
  Notification,
} from './types';

// ============================================================================
// Store
// ============================================================================
export {
  useDashboardStore,
  selectLayout,
  selectTimeRange,
  selectFilters,
  selectKPIs,
  selectAlerts,
  selectActiveAlerts,
  selectCriticalAlerts,
  selectActivities,
  selectQuotas,
  selectConfig,
  selectIsLoading,
  selectError,
} from './stores/dashboardStore';

// ============================================================================
// Services
// ============================================================================
export { DashboardService, dashboardService } from './services/DashboardService';

// ============================================================================
// Hooks
// ============================================================================
export { useDashboard } from './hooks/useDashboard';
export type { UseDashboardOptions } from './hooks/useDashboard';

export {
  useRealTimeData,
  useRealTimeKPIs,
  useRealTimeAlerts,
  useRealTimeActivity,
  useRealTimeMultiChannel,
} from './hooks/useRealTimeData';
export type { UseRealTimeDataOptions, RealTimeDataState } from './hooks/useRealTimeData';

// ============================================================================
// Components - Dashboard
// ============================================================================
export { ExecutiveDashboard } from './components/Dashboard/ExecutiveDashboard';
export type { ExecutiveDashboardProps } from './components/Dashboard/ExecutiveDashboard';

export {
  DashboardGrid,
  EmptyDashboard,
  WidgetCatalog,
} from './components/Dashboard/DashboardGrid';
export type { DashboardGridProps } from './components/Dashboard/DashboardGrid';

// ============================================================================
// Components - KPI
// ============================================================================
export { KPICard } from './components/KPI/KPICard';
export type { KPICardProps } from './components/KPI/KPICard';

export {
  KPITrend,
  CompactTrend,
  TrendBadge,
} from './components/KPI/KPITrend';
export type { KPITrendProps } from './components/KPI/KPITrend';

// ============================================================================
// Components - Charts
// ============================================================================
export { RevenueChart } from './components/Charts/RevenueChart';
export type { RevenueChartProps } from './components/Charts/RevenueChart';

export { UsageChart } from './components/Charts/UsageChart';
export type { UsageChartProps } from './components/Charts/UsageChart';

export { PerformanceChart } from './components/Charts/PerformanceChart';
export type { PerformanceChartProps } from './components/Charts/PerformanceChart';

export { GeoChart } from './components/Charts/GeoChart';
export type { GeoChartProps } from './components/Charts/GeoChart';

// ============================================================================
// Components - Widgets
// ============================================================================
export { AlertWidget } from './components/Widgets/AlertWidget';
export type { AlertWidgetProps } from './components/Widgets/AlertWidget';

export {
  ActivityWidget,
  ActivityTimeline,
} from './components/Widgets/ActivityWidget';
export type { ActivityWidgetProps } from './components/Widgets/ActivityWidget';

export {
  QuotaWidget,
  QuotaSummary,
} from './components/Widgets/QuotaWidget';
export type { QuotaWidgetProps } from './components/Widgets/QuotaWidget';

// ============================================================================
// Package Metadata
// ============================================================================
export const VERSION = '0.5.0';
export const PACKAGE_NAME = '@esxi/enterprise-dashboard';

/**
 * Package Information
 */
export const PACKAGE_INFO = {
  name: PACKAGE_NAME,
  version: VERSION,
  description: 'Enterprise Dashboard UI for $983M SaaS Platform',
  features: [
    'Real-time data updates via WebSocket',
    'Interactive KPI cards with trends',
    'Advanced analytics charts (Revenue, Usage, Performance, Geo)',
    'Draggable and resizable widget grid',
    'Alert management and monitoring',
    'Activity feed and audit logs',
    'Quota tracking and forecasting',
    'Responsive design',
    'Dark mode optimized',
    'TypeScript support',
    'Enterprise-grade performance',
  ],
  dependencies: {
    react: '^18.2.0',
    recharts: '^2.10.3',
    d3: '^7.8.5',
    zustand: '^4.4.7',
    'framer-motion': '^10.16.16',
  },
};

/**
 * Default Configuration
 */
export const DEFAULT_CONFIG = {
  timeRange: '24h' as const,
  refreshInterval: 30000, // 30 seconds
  autoRefresh: true,
  theme: 'dark' as const,
  animations: true,
  realTimeEnabled: true,
};

/**
 * Initialize Dashboard
 * Helper function to set up dashboard with default configuration
 */
export function initializeDashboard(config?: Partial<typeof DEFAULT_CONFIG>) {
  return {
    ...DEFAULT_CONFIG,
    ...config,
  };
}
