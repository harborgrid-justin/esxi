/**
 * useDashboard Hook
 * Main dashboard state management hook
 */

import { useEffect, useCallback, useRef } from 'react';
import { useDashboardStore } from '../stores/dashboardStore';
import { dashboardService } from '../services/DashboardService';
import type { TimeRange, DashboardFilters, DashboardLayout } from '../types';

export interface UseDashboardOptions {
  layoutId?: string;
  autoLoad?: boolean;
  autoRefresh?: boolean;
  refreshInterval?: number;
}

export function useDashboard(options: UseDashboardOptions = {}) {
  const {
    layoutId,
    autoLoad = true,
    autoRefresh: autoRefreshOption,
    refreshInterval: refreshIntervalOption,
  } = options;

  // Store state
  const {
    layout,
    timeRange,
    refreshInterval,
    autoRefresh,
    isLoading,
    error,
    filters,
    kpis,
    alerts,
    activities,
    quotas,
    setLayout,
    setTimeRange,
    setRefreshInterval,
    toggleAutoRefresh,
    setFilters,
    clearFilters,
    setKPIs,
    setAlerts,
    setActivities,
    setQuotas,
    setLoading,
    setError,
  } = useDashboardStore();

  const refreshIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // ============================================================================
  // Data Loading Functions
  // ============================================================================

  const loadLayout = useCallback(async (id: string) => {
    setLoading(true);
    setError(null);

    try {
      const response = await dashboardService.getLayout(id);
      if (response.success && response.data) {
        setLayout(response.data);
      } else {
        setError(response.error?.message || 'Failed to load layout');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [setLayout, setLoading, setError]);

  const loadKPIs = useCallback(async () => {
    try {
      const response = await dashboardService.getKPIs(timeRange, filters);
      if (response.success && response.data) {
        setKPIs(response.data);
      }
    } catch (err) {
      console.error('Failed to load KPIs:', err);
    }
  }, [timeRange, filters, setKPIs]);

  const loadAlerts = useCallback(async () => {
    try {
      const response = await dashboardService.getAlerts(
        { page: 1, pageSize: 50, sortBy: 'timestamp', sortOrder: 'desc' },
        filters
      );
      if (response.success && response.data) {
        setAlerts(response.data.items);
      }
    } catch (err) {
      console.error('Failed to load alerts:', err);
    }
  }, [filters, setAlerts]);

  const loadActivities = useCallback(async () => {
    try {
      const response = await dashboardService.getActivities(
        { page: 1, pageSize: 100, sortBy: 'timestamp', sortOrder: 'desc' },
        filters
      );
      if (response.success && response.data) {
        setActivities(response.data.items);
      }
    } catch (err) {
      console.error('Failed to load activities:', err);
    }
  }, [filters, setActivities]);

  const loadQuotas = useCallback(async () => {
    try {
      const response = await dashboardService.getQuotas(filters);
      if (response.success && response.data) {
        setQuotas(response.data);
      }
    } catch (err) {
      console.error('Failed to load quotas:', err);
    }
  }, [filters, setQuotas]);

  const loadAllData = useCallback(async () => {
    setLoading(true);
    try {
      await Promise.all([
        loadKPIs(),
        loadAlerts(),
        loadActivities(),
        loadQuotas(),
      ]);
    } finally {
      setLoading(false);
    }
  }, [loadKPIs, loadAlerts, loadActivities, loadQuotas, setLoading]);

  // ============================================================================
  // Dashboard Actions
  // ============================================================================

  const refresh = useCallback(() => {
    loadAllData();
  }, [loadAllData]);

  const updateTimeRange = useCallback((range: TimeRange) => {
    setTimeRange(range);
  }, [setTimeRange]);

  const updateFilters = useCallback((newFilters: Partial<DashboardFilters>) => {
    setFilters(newFilters);
  }, [setFilters]);

  const resetFilters = useCallback(() => {
    clearFilters();
  }, [clearFilters]);

  const updateRefreshInterval = useCallback((interval: number) => {
    setRefreshInterval(interval);
  }, [setRefreshInterval]);

  const toggleRefresh = useCallback(() => {
    toggleAutoRefresh();
  }, [toggleAutoRefresh]);

  const saveLayout = useCallback(async (updates: Partial<DashboardLayout>) => {
    if (!layout) return;

    setLoading(true);
    try {
      const response = await dashboardService.updateLayout(layout.id, updates);
      if (response.success && response.data) {
        setLayout(response.data);
      } else {
        setError(response.error?.message || 'Failed to save layout');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [layout, setLayout, setLoading, setError]);

  const acknowledgeAlert = useCallback(async (alertId: string) => {
    try {
      const response = await dashboardService.acknowledgeAlert(alertId, 'current-user');
      if (response.success) {
        await loadAlerts();
      }
    } catch (err) {
      console.error('Failed to acknowledge alert:', err);
    }
  }, [loadAlerts]);

  const resolveAlert = useCallback(async (alertId: string, resolution: string) => {
    try {
      const response = await dashboardService.resolveAlert(alertId, 'current-user', resolution);
      if (response.success) {
        await loadAlerts();
      }
    } catch (err) {
      console.error('Failed to resolve alert:', err);
    }
  }, [loadAlerts]);

  const exportDashboard = useCallback(async (format: 'csv' | 'xlsx' | 'pdf' | 'json') => {
    try {
      const response = await dashboardService.exportDashboard({
        format,
        includeCharts: true,
        filters,
      });

      if (response.success && response.data) {
        window.open(response.data.downloadUrl, '_blank');
      }
    } catch (err) {
      console.error('Failed to export dashboard:', err);
    }
  }, [filters]);

  // ============================================================================
  // Effects
  // ============================================================================

  // Load initial layout
  useEffect(() => {
    if (autoLoad && layoutId && !layout) {
      loadLayout(layoutId);
    }
  }, [autoLoad, layoutId, layout, loadLayout]);

  // Load initial data
  useEffect(() => {
    if (autoLoad && !isLoading) {
      loadAllData();
    }
  }, [autoLoad]); // Only run once on mount

  // Refresh data when time range or filters change
  useEffect(() => {
    if (layout) {
      loadAllData();
    }
  }, [timeRange, filters]);

  // Setup auto-refresh
  useEffect(() => {
    const shouldAutoRefresh = autoRefreshOption ?? autoRefresh;
    const interval = refreshIntervalOption ?? refreshInterval;

    if (shouldAutoRefresh && interval > 0) {
      refreshIntervalRef.current = setInterval(() => {
        loadAllData();
      }, interval);
    }

    return () => {
      if (refreshIntervalRef.current) {
        clearInterval(refreshIntervalRef.current);
      }
    };
  }, [autoRefresh, refreshInterval, autoRefreshOption, refreshIntervalOption, loadAllData]);

  // ============================================================================
  // Return Hook Interface
  // ============================================================================

  return {
    // State
    layout,
    timeRange,
    refreshInterval,
    autoRefresh,
    isLoading,
    error,
    filters,
    kpis,
    alerts,
    activities,
    quotas,

    // Actions
    refresh,
    loadLayout,
    saveLayout,
    updateTimeRange,
    updateFilters,
    resetFilters,
    updateRefreshInterval,
    toggleRefresh,
    acknowledgeAlert,
    resolveAlert,
    exportDashboard,

    // Computed
    criticalAlerts: alerts.filter((a) => a.severity === 'critical' && a.status === 'active'),
    activeAlerts: alerts.filter((a) => a.status === 'active'),
    criticalQuotas: quotas.filter((q) => q.percentage >= 90),
    warningQuotas: quotas.filter((q) => q.percentage >= 75 && q.percentage < 90),
  };
}

export default useDashboard;
