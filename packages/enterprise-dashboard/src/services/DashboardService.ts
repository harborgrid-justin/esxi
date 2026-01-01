/**
 * Enterprise Dashboard Service
 * API Integration Layer for Dashboard Data
 */

import type {
  DashboardLayout,
  KPIMetric,
  Alert,
  ActivityLogEntry,
  QuotaUsage,
  RevenueData,
  UsageMetrics,
  PerformanceMetrics,
  GeoDataPoint,
  TimeRange,
  DashboardFilters,
  APIResponse,
  PaginationParams,
  PaginatedResponse,
  ExportConfig,
} from '../types';

/**
 * Base API Configuration
 */
const API_BASE_URL = process.env.REACT_APP_API_URL || '/api/v1';
const API_TIMEOUT = 30000;

/**
 * HTTP Client with enterprise features
 */
class HTTPClient {
  private baseURL: string;
  private defaultHeaders: HeadersInit;

  constructor(baseURL: string) {
    this.baseURL = baseURL;
    this.defaultHeaders = {
      'Content-Type': 'application/json',
      'X-Client-Version': '0.5.0',
    };
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<APIResponse<T>> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT);

    try {
      const response = await fetch(`${this.baseURL}${endpoint}`, {
        ...options,
        headers: {
          ...this.defaultHeaders,
          ...options.headers,
        },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();
      return {
        success: true,
        data,
        metadata: {
          timestamp: new Date(),
          requestId: response.headers.get('X-Request-ID') || '',
          duration: 0,
        },
      };
    } catch (error) {
      clearTimeout(timeoutId);
      return {
        success: false,
        error: {
          code: error instanceof Error ? error.name : 'UNKNOWN_ERROR',
          message: error instanceof Error ? error.message : 'An error occurred',
        },
      };
    }
  }

  async get<T>(endpoint: string, params?: Record<string, string>): Promise<APIResponse<T>> {
    const url = params
      ? `${endpoint}?${new URLSearchParams(params).toString()}`
      : endpoint;
    return this.request<T>(url, { method: 'GET' });
  }

  async post<T>(endpoint: string, data: unknown): Promise<APIResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async put<T>(endpoint: string, data: unknown): Promise<APIResponse<T>> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async delete<T>(endpoint: string): Promise<APIResponse<T>> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }
}

/**
 * Dashboard Service Class
 */
export class DashboardService {
  private client: HTTPClient;

  constructor(baseURL: string = API_BASE_URL) {
    this.client = new HTTPClient(baseURL);
  }

  // ============================================================================
  // Dashboard Layout Management
  // ============================================================================

  async getLayouts(): Promise<APIResponse<DashboardLayout[]>> {
    return this.client.get<DashboardLayout[]>('/dashboards/layouts');
  }

  async getLayout(layoutId: string): Promise<APIResponse<DashboardLayout>> {
    return this.client.get<DashboardLayout>(`/dashboards/layouts/${layoutId}`);
  }

  async createLayout(layout: Partial<DashboardLayout>): Promise<APIResponse<DashboardLayout>> {
    return this.client.post<DashboardLayout>('/dashboards/layouts', layout);
  }

  async updateLayout(
    layoutId: string,
    updates: Partial<DashboardLayout>
  ): Promise<APIResponse<DashboardLayout>> {
    return this.client.put<DashboardLayout>(`/dashboards/layouts/${layoutId}`, updates);
  }

  async deleteLayout(layoutId: string): Promise<APIResponse<void>> {
    return this.client.delete<void>(`/dashboards/layouts/${layoutId}`);
  }

  // ============================================================================
  // KPI Metrics
  // ============================================================================

  async getKPIs(
    timeRange: TimeRange,
    filters?: DashboardFilters
  ): Promise<APIResponse<KPIMetric[]>> {
    return this.client.post<KPIMetric[]>('/dashboards/kpis', {
      timeRange,
      filters,
    });
  }

  async getKPIHistory(
    kpiId: string,
    timeRange: TimeRange
  ): Promise<APIResponse<number[]>> {
    return this.client.get<number[]>(`/dashboards/kpis/${kpiId}/history`, {
      timeRange,
    });
  }

  // ============================================================================
  // Alerts
  // ============================================================================

  async getAlerts(
    pagination?: PaginationParams,
    filters?: DashboardFilters
  ): Promise<APIResponse<PaginatedResponse<Alert>>> {
    return this.client.post<PaginatedResponse<Alert>>('/alerts', {
      pagination,
      filters,
    });
  }

  async acknowledgeAlert(alertId: string, userId: string): Promise<APIResponse<Alert>> {
    return this.client.post<Alert>(`/alerts/${alertId}/acknowledge`, {
      userId,
      timestamp: new Date().toISOString(),
    });
  }

  async resolveAlert(
    alertId: string,
    userId: string,
    resolution: string
  ): Promise<APIResponse<Alert>> {
    return this.client.post<Alert>(`/alerts/${alertId}/resolve`, {
      userId,
      resolution,
      timestamp: new Date().toISOString(),
    });
  }

  async escalateAlert(alertId: string, assignTo: string): Promise<APIResponse<Alert>> {
    return this.client.post<Alert>(`/alerts/${alertId}/escalate`, {
      assignTo,
      timestamp: new Date().toISOString(),
    });
  }

  // ============================================================================
  // Activity Logs
  // ============================================================================

  async getActivities(
    pagination?: PaginationParams,
    filters?: DashboardFilters
  ): Promise<APIResponse<PaginatedResponse<ActivityLogEntry>>> {
    return this.client.post<PaginatedResponse<ActivityLogEntry>>('/activities', {
      pagination,
      filters,
    });
  }

  async getActivityById(activityId: string): Promise<APIResponse<ActivityLogEntry>> {
    return this.client.get<ActivityLogEntry>(`/activities/${activityId}`);
  }

  // ============================================================================
  // Quota Management
  // ============================================================================

  async getQuotas(filters?: DashboardFilters): Promise<APIResponse<QuotaUsage[]>> {
    return this.client.post<QuotaUsage[]>('/quotas', { filters });
  }

  async getQuotaForecast(quotaId: string): Promise<APIResponse<{
    exhaustionDate: Date;
    confidence: number;
    recommendations: string[];
  }>> {
    return this.client.get(`/quotas/${quotaId}/forecast`);
  }

  // ============================================================================
  // Revenue Analytics
  // ============================================================================

  async getRevenueData(
    timeRange: TimeRange,
    granularity: 'hour' | 'day' | 'week' | 'month' = 'day',
    filters?: DashboardFilters
  ): Promise<APIResponse<RevenueData[]>> {
    return this.client.post<RevenueData[]>('/analytics/revenue', {
      timeRange,
      granularity,
      filters,
    });
  }

  async getRevenueForecast(
    period: number,
    unit: 'days' | 'weeks' | 'months'
  ): Promise<APIResponse<RevenueData[]>> {
    return this.client.get<RevenueData[]>('/analytics/revenue/forecast', {
      period: String(period),
      unit,
    });
  }

  // ============================================================================
  // Usage Metrics
  // ============================================================================

  async getUsageMetrics(
    timeRange: TimeRange,
    granularity: 'minute' | 'hour' | 'day' = 'hour',
    filters?: DashboardFilters
  ): Promise<APIResponse<UsageMetrics[]>> {
    return this.client.post<UsageMetrics[]>('/metrics/usage', {
      timeRange,
      granularity,
      filters,
    });
  }

  async getUsageTrends(
    metric: keyof UsageMetrics,
    timeRange: TimeRange
  ): Promise<APIResponse<number[]>> {
    return this.client.get<number[]>('/metrics/usage/trends', {
      metric,
      timeRange,
    });
  }

  // ============================================================================
  // Performance Metrics
  // ============================================================================

  async getPerformanceMetrics(
    timeRange: TimeRange,
    services?: string[],
    regions?: string[]
  ): Promise<APIResponse<PerformanceMetrics[]>> {
    return this.client.post<PerformanceMetrics[]>('/metrics/performance', {
      timeRange,
      services,
      regions,
    });
  }

  async getServiceHealth(serviceId: string): Promise<APIResponse<{
    status: 'healthy' | 'degraded' | 'down';
    availability: number;
    incidents: number;
    lastIncident?: Date;
  }>> {
    return this.client.get(`/metrics/performance/services/${serviceId}/health`);
  }

  // ============================================================================
  // Geographic Distribution
  // ============================================================================

  async getGeoDistribution(
    timeRange: TimeRange,
    filters?: DashboardFilters
  ): Promise<APIResponse<GeoDataPoint[]>> {
    return this.client.post<GeoDataPoint[]>('/analytics/geo', {
      timeRange,
      filters,
    });
  }

  async getRegionMetrics(
    region: string,
    timeRange: TimeRange
  ): Promise<APIResponse<{
    users: number;
    revenue: number;
    latency: number;
    availability: number;
    topCities: Array<{ city: string; users: number }>;
  }>> {
    return this.client.get(`/analytics/geo/regions/${region}`, {
      timeRange,
    });
  }

  // ============================================================================
  // Export & Reporting
  // ============================================================================

  async exportDashboard(config: ExportConfig): Promise<APIResponse<{
    downloadUrl: string;
    expiresAt: Date;
  }>> {
    return this.client.post('/dashboards/export', config);
  }

  async scheduleReport(
    layoutId: string,
    schedule: {
      frequency: 'daily' | 'weekly' | 'monthly';
      recipients: string[];
      format: 'pdf' | 'xlsx';
    }
  ): Promise<APIResponse<{ reportId: string }>> {
    return this.client.post('/dashboards/reports/schedule', {
      layoutId,
      ...schedule,
    });
  }

  // ============================================================================
  // Real-time Data Streaming
  // ============================================================================

  createWebSocket(channels: string[]): WebSocket {
    const wsUrl = API_BASE_URL.replace(/^http/, 'ws');
    const ws = new WebSocket(`${wsUrl}/stream?channels=${channels.join(',')}`);

    ws.addEventListener('error', (error) => {
      console.error('WebSocket error:', error);
    });

    return ws;
  }

  // ============================================================================
  // Utilities
  // ============================================================================

  async healthCheck(): Promise<APIResponse<{
    status: 'healthy' | 'degraded' | 'down';
    version: string;
    uptime: number;
  }>> {
    return this.client.get('/health');
  }

  async validateAccess(resourceId: string): Promise<APIResponse<{
    allowed: boolean;
    permissions: string[];
  }>> {
    return this.client.get(`/access/validate/${resourceId}`);
  }
}

// Singleton instance
export const dashboardService = new DashboardService();

export default dashboardService;
