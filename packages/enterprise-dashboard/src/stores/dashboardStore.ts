/**
 * Enterprise Dashboard Store
 * Zustand State Management for Dashboard
 */

import { create } from 'zustand';
import { persist, subscribeWithSelector } from 'zustand/middleware';
import type {
  DashboardState,
  DashboardLayout,
  DashboardWidget,
  TimeRange,
  DashboardFilters,
  KPIMetric,
  Alert,
  ActivityLogEntry,
  QuotaUsage,
  DashboardConfig,
} from '../types';

interface DashboardStore extends DashboardState {
  // Layout Management
  setLayout: (layout: DashboardLayout) => void;
  updateWidget: (widgetId: string, updates: Partial<DashboardWidget>) => void;
  addWidget: (widget: DashboardWidget) => void;
  removeWidget: (widgetId: string) => void;
  reorderWidgets: (widgets: DashboardWidget[]) => void;
  resetLayout: () => void;

  // Time Range & Refresh
  setTimeRange: (range: TimeRange) => void;
  setRefreshInterval: (interval: number) => void;
  toggleAutoRefresh: () => void;
  refresh: () => void;

  // Filters
  setFilters: (filters: Partial<DashboardFilters>) => void;
  clearFilters: () => void;

  // Data Management
  setKPIs: (kpis: KPIMetric[]) => void;
  updateKPI: (kpiId: string, updates: Partial<KPIMetric>) => void;
  setAlerts: (alerts: Alert[]) => void;
  acknowledgeAlert: (alertId: string) => void;
  resolveAlert: (alertId: string) => void;
  setActivities: (activities: ActivityLogEntry[]) => void;
  setQuotas: (quotas: QuotaUsage[]) => void;

  // Loading & Error States
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  // Configuration
  config: DashboardConfig;
  updateConfig: (updates: Partial<DashboardConfig>) => void;

  // Utilities
  reset: () => void;
}

const defaultConfig: DashboardConfig = {
  theme: 'dark',
  density: 'comfortable',
  animations: true,
  soundEffects: false,
  notifications: {
    desktop: true,
    sound: false,
    alertSeverities: ['critical', 'high'],
  },
  defaultTimeRange: '24h',
  defaultRefreshInterval: 30000,
  chartDefaults: {
    type: 'line',
    colors: [
      '#3b82f6', // blue
      '#10b981', // green
      '#f59e0b', // amber
      '#ef4444', // red
      '#8b5cf6', // violet
      '#ec4899', // pink
      '#14b8a6', // teal
      '#f97316', // orange
    ],
    showLegend: true,
    showGrid: true,
    animation: true,
  },
};

const initialState: Omit<DashboardState, 'config'> = {
  layout: null,
  timeRange: '24h',
  refreshInterval: 30000,
  autoRefresh: true,
  isLoading: false,
  error: null,
  filters: {},
  kpis: [],
  alerts: [],
  activities: [],
  quotas: [],
};

export const useDashboardStore = create<DashboardStore>()(
  subscribeWithSelector(
    persist(
      (set, get) => ({
        ...initialState,
        config: defaultConfig,

        // Layout Management
        setLayout: (layout) => set({ layout }),

        updateWidget: (widgetId, updates) =>
          set((state) => {
            if (!state.layout) return state;
            return {
              layout: {
                ...state.layout,
                widgets: state.layout.widgets.map((w) =>
                  w.id === widgetId ? { ...w, ...updates } : w
                ),
                updatedAt: new Date(),
              },
            };
          }),

        addWidget: (widget) =>
          set((state) => {
            if (!state.layout) return state;
            return {
              layout: {
                ...state.layout,
                widgets: [...state.layout.widgets, widget],
                updatedAt: new Date(),
              },
            };
          }),

        removeWidget: (widgetId) =>
          set((state) => {
            if (!state.layout) return state;
            return {
              layout: {
                ...state.layout,
                widgets: state.layout.widgets.filter((w) => w.id !== widgetId),
                updatedAt: new Date(),
              },
            };
          }),

        reorderWidgets: (widgets) =>
          set((state) => {
            if (!state.layout) return state;
            return {
              layout: {
                ...state.layout,
                widgets,
                updatedAt: new Date(),
              },
            };
          }),

        resetLayout: () =>
          set((state) => ({
            layout: state.layout
              ? { ...state.layout, widgets: [], updatedAt: new Date() }
              : null,
          })),

        // Time Range & Refresh
        setTimeRange: (range) => set({ timeRange: range }),

        setRefreshInterval: (interval) => set({ refreshInterval: interval }),

        toggleAutoRefresh: () =>
          set((state) => ({ autoRefresh: !state.autoRefresh })),

        refresh: () => {
          // Trigger refresh by updating a timestamp or similar
          set({ isLoading: true });
        },

        // Filters
        setFilters: (filters) =>
          set((state) => ({
            filters: { ...state.filters, ...filters },
          })),

        clearFilters: () => set({ filters: {} }),

        // Data Management
        setKPIs: (kpis) => set({ kpis }),

        updateKPI: (kpiId, updates) =>
          set((state) => ({
            kpis: state.kpis.map((kpi) =>
              kpi.id === kpiId ? { ...kpi, ...updates } : kpi
            ),
          })),

        setAlerts: (alerts) => set({ alerts }),

        acknowledgeAlert: (alertId) =>
          set((state) => ({
            alerts: state.alerts.map((alert) =>
              alert.id === alertId
                ? { ...alert, status: 'acknowledged' as const }
                : alert
            ),
          })),

        resolveAlert: (alertId) =>
          set((state) => ({
            alerts: state.alerts.map((alert) =>
              alert.id === alertId
                ? { ...alert, status: 'resolved' as const }
                : alert
            ),
          })),

        setActivities: (activities) => set({ activities }),

        setQuotas: (quotas) => set({ quotas }),

        // Loading & Error States
        setLoading: (loading) => set({ isLoading: loading }),

        setError: (error) => set({ error }),

        // Configuration
        updateConfig: (updates) =>
          set((state) => ({
            config: { ...state.config, ...updates },
          })),

        // Utilities
        reset: () => set({ ...initialState, config: defaultConfig }),
      }),
      {
        name: 'enterprise-dashboard-storage',
        partialize: (state) => ({
          layout: state.layout,
          timeRange: state.timeRange,
          refreshInterval: state.refreshInterval,
          autoRefresh: state.autoRefresh,
          filters: state.filters,
          config: state.config,
        }),
      }
    )
  )
);

// Selectors for optimized re-renders
export const selectLayout = (state: DashboardStore) => state.layout;
export const selectTimeRange = (state: DashboardStore) => state.timeRange;
export const selectFilters = (state: DashboardStore) => state.filters;
export const selectKPIs = (state: DashboardStore) => state.kpis;
export const selectAlerts = (state: DashboardStore) => state.alerts;
export const selectActiveAlerts = (state: DashboardStore) =>
  state.alerts.filter((a) => a.status === 'active');
export const selectCriticalAlerts = (state: DashboardStore) =>
  state.alerts.filter((a) => a.severity === 'critical' && a.status === 'active');
export const selectActivities = (state: DashboardStore) => state.activities;
export const selectQuotas = (state: DashboardStore) => state.quotas;
export const selectConfig = (state: DashboardStore) => state.config;
export const selectIsLoading = (state: DashboardStore) => state.isLoading;
export const selectError = (state: DashboardStore) => state.error;
