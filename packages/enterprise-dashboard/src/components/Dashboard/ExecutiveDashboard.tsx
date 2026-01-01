/**
 * Executive Dashboard Component
 * Main dashboard view for enterprise SaaS platform
 */

import React, { useState, useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import clsx from 'clsx';
import { useDashboard } from '../../hooks/useDashboard';
import { useRealTimeData } from '../../hooks/useRealTimeData';
import { DashboardGrid, EmptyDashboard } from './DashboardGrid';
import { KPICard } from '../KPI/KPICard';
import { RevenueChart } from '../Charts/RevenueChart';
import { UsageChart } from '../Charts/UsageChart';
import { PerformanceChart } from '../Charts/PerformanceChart';
import { GeoChart } from '../Charts/GeoChart';
import { AlertWidget } from '../Widgets/AlertWidget';
import { ActivityWidget } from '../Widgets/ActivityWidget';
import { QuotaWidget } from '../Widgets/QuotaWidget';
import type { TimeRange, DashboardWidget, DashboardFilters } from '../../types';

export interface ExecutiveDashboardProps {
  layoutId?: string;
  defaultTimeRange?: TimeRange;
  enableRealTime?: boolean;
  className?: string;
}

export const ExecutiveDashboard: React.FC<ExecutiveDashboardProps> = ({
  layoutId,
  defaultTimeRange = '24h',
  enableRealTime = true,
  className,
}) => {
  const [isEditMode, setIsEditMode] = useState(false);
  const [selectedTimeRange, setSelectedTimeRange] = useState<TimeRange>(defaultTimeRange);
  const [showFilters, setShowFilters] = useState(false);

  // Dashboard state and actions
  const {
    layout,
    timeRange,
    isLoading,
    error,
    kpis,
    alerts,
    activities,
    quotas,
    criticalAlerts,
    activeAlerts,
    criticalQuotas,
    refresh,
    updateTimeRange,
    updateFilters,
    acknowledgeAlert,
    resolveAlert,
    saveLayout,
  } = useDashboard({
    layoutId,
    autoLoad: true,
    autoRefresh: true,
  });

  // Real-time data subscription
  const { connected: wsConnected } = useRealTimeData({
    channels: ['kpis', 'alerts', 'activities', 'metrics'],
    enabled: enableRealTime,
    onMessage: (data) => {
      console.log('Real-time update:', data);
      // Handle real-time updates
    },
  });

  // Time range options
  const timeRanges: Array<{ value: TimeRange; label: string }> = [
    { value: '1h', label: '1 Hour' },
    { value: '6h', label: '6 Hours' },
    { value: '24h', label: '24 Hours' },
    { value: '7d', label: '7 Days' },
    { value: '30d', label: '30 Days' },
    { value: '90d', label: '90 Days' },
    { value: 'ytd', label: 'Year to Date' },
  ];

  // Handle time range change
  const handleTimeRangeChange = (range: TimeRange) => {
    setSelectedTimeRange(range);
    updateTimeRange(range);
  };

  // Handle layout change
  const handleLayoutChange = (updatedWidgets: DashboardWidget[]) => {
    if (!layout) return;
    saveLayout({ widgets: updatedWidgets });
  };

  // Render widget content based on type
  const renderWidget = (widget: DashboardWidget): React.ReactNode => {
    switch (widget.type) {
      case 'kpi':
        // Find KPI data
        const kpi = kpis.find((k) => k.id === widget.config.kpiId as string);
        if (!kpi) return <div className="p-4 text-gray-500">KPI not found</div>;
        return <KPICard metric={kpi} />;

      case 'alert':
        return (
          <AlertWidget
            alerts={activeAlerts}
            onAcknowledge={acknowledgeAlert}
            onResolve={resolveAlert}
            maxDisplay={widget.config.maxDisplay as number | undefined}
          />
        );

      case 'activity':
        return (
          <ActivityWidget
            activities={activities}
            maxDisplay={widget.config.maxDisplay as number | undefined}
            compact={widget.config.compact as boolean | undefined}
          />
        );

      case 'quota':
        return (
          <QuotaWidget
            quotas={quotas}
            warningThreshold={widget.config.warningThreshold as number | undefined}
            criticalThreshold={widget.config.criticalThreshold as number | undefined}
          />
        );

      case 'chart':
        const chartType = widget.config.chartType as string;
        switch (chartType) {
          case 'revenue':
            return <RevenueChart data={[]} />;
          case 'usage':
            return <UsageChart data={[]} />;
          case 'performance':
            return <PerformanceChart data={[]} />;
          case 'geo':
            return <GeoChart data={[]} />;
          default:
            return <div className="p-4 text-gray-500">Unknown chart type</div>;
        }

      default:
        return (
          <div className="h-full flex items-center justify-center bg-gray-800/30 rounded-lg border border-gray-700">
            <div className="text-center text-gray-500">
              <div className="text-4xl mb-2">📊</div>
              <div className="text-sm">Widget: {widget.title}</div>
              <div className="text-xs mt-1">Type: {widget.type}</div>
            </div>
          </div>
        );
    }
  };

  // Summary stats
  const stats = useMemo(() => {
    return {
      totalKPIs: kpis.length,
      criticalAlerts: criticalAlerts.length,
      activeAlerts: activeAlerts.length,
      criticalQuotas: criticalQuotas.length,
      recentActivities: activities.length,
    };
  }, [kpis, criticalAlerts, activeAlerts, criticalQuotas, activities]);

  return (
    <div className={clsx('executive-dashboard', className)}>
      {/* Header */}
      <div className="mb-8">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-3xl font-bold text-white mb-2">
              Executive Dashboard
            </h1>
            <p className="text-gray-400">
              $983M Enterprise SaaS Platform - Real-time Monitoring & Analytics
            </p>
          </div>

          {/* Status Indicators */}
          <div className="flex items-center gap-4">
            {/* Real-time Status */}
            {enableRealTime && (
              <div className="flex items-center gap-2">
                <div
                  className={clsx(
                    'w-2 h-2 rounded-full',
                    wsConnected ? 'bg-green-400 animate-pulse' : 'bg-red-400'
                  )}
                />
                <span className="text-xs text-gray-500">
                  {wsConnected ? 'Live' : 'Disconnected'}
                </span>
              </div>
            )}

            {/* Last Updated */}
            <div className="text-xs text-gray-500">
              Updated: {new Date().toLocaleTimeString()}
            </div>
          </div>
        </div>

        {/* Controls */}
        <div className="flex items-center justify-between">
          {/* Time Range Selector */}
          <div className="flex items-center gap-2">
            <span className="text-sm text-gray-400">Time Range:</span>
            <div className="flex items-center gap-1 bg-gray-800/50 rounded-lg p-1">
              {timeRanges.map((range) => (
                <button
                  key={range.value}
                  onClick={() => handleTimeRangeChange(range.value)}
                  className={clsx(
                    'px-3 py-1.5 text-sm font-medium rounded transition-all',
                    selectedTimeRange === range.value
                      ? 'bg-blue-500 text-white'
                      : 'text-gray-400 hover:text-white hover:bg-gray-700/50'
                  )}
                >
                  {range.label}
                </button>
              ))}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex items-center gap-3">
            {/* Filters */}
            <button
              onClick={() => setShowFilters(!showFilters)}
              className="flex items-center gap-2 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-white rounded-lg transition-colors"
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                />
              </svg>
              <span className="text-sm">Filters</span>
            </button>

            {/* Edit Mode */}
            <button
              onClick={() => setIsEditMode(!isEditMode)}
              className={clsx(
                'flex items-center gap-2 px-4 py-2 rounded-lg transition-colors',
                isEditMode
                  ? 'bg-blue-500 hover:bg-blue-600 text-white'
                  : 'bg-gray-800 hover:bg-gray-700 text-white'
              )}
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                />
              </svg>
              <span className="text-sm">{isEditMode ? 'Done Editing' : 'Edit Layout'}</span>
            </button>

            {/* Refresh */}
            <button
              onClick={refresh}
              disabled={isLoading}
              className="flex items-center gap-2 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <svg
                className={clsx('w-4 h-4', isLoading && 'animate-spin')}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                />
              </svg>
              <span className="text-sm">Refresh</span>
            </button>
          </div>
        </div>
      </div>

      {/* Summary Stats Bar */}
      <div className="grid grid-cols-5 gap-4 mb-8">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="bg-gradient-to-br from-blue-500/10 to-blue-600/10 border border-blue-500/20 rounded-xl p-4"
        >
          <div className="text-sm text-gray-400 mb-1">Total KPIs</div>
          <div className="text-3xl font-bold text-blue-400">{stats.totalKPIs}</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="bg-gradient-to-br from-red-500/10 to-red-600/10 border border-red-500/20 rounded-xl p-4"
        >
          <div className="text-sm text-gray-400 mb-1">Critical Alerts</div>
          <div className="text-3xl font-bold text-red-400">{stats.criticalAlerts}</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
          className="bg-gradient-to-br from-amber-500/10 to-amber-600/10 border border-amber-500/20 rounded-xl p-4"
        >
          <div className="text-sm text-gray-400 mb-1">Active Alerts</div>
          <div className="text-3xl font-bold text-amber-400">{stats.activeAlerts}</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="bg-gradient-to-br from-purple-500/10 to-purple-600/10 border border-purple-500/20 rounded-xl p-4"
        >
          <div className="text-sm text-gray-400 mb-1">Critical Quotas</div>
          <div className="text-3xl font-bold text-purple-400">{stats.criticalQuotas}</div>
        </motion.div>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.4 }}
          className="bg-gradient-to-br from-green-500/10 to-green-600/10 border border-green-500/20 rounded-xl p-4"
        >
          <div className="text-sm text-gray-400 mb-1">Recent Activities</div>
          <div className="text-3xl font-bold text-green-400">{stats.recentActivities}</div>
        </motion.div>
      </div>

      {/* Error State */}
      {error && (
        <div className="mb-8 p-4 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400">
          <div className="flex items-center gap-2">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span className="font-semibold">Error loading dashboard:</span>
            <span>{error}</span>
          </div>
        </div>
      )}

      {/* Dashboard Content */}
      <AnimatePresence mode="wait">
        {isLoading && !layout ? (
          <div className="flex items-center justify-center min-h-[400px]">
            <div className="text-center">
              <div className="animate-spin w-12 h-12 border-4 border-blue-500 border-t-transparent rounded-full mx-auto mb-4" />
              <div className="text-gray-400">Loading dashboard...</div>
            </div>
          </div>
        ) : !layout || layout.widgets.length === 0 ? (
          <EmptyDashboard />
        ) : (
          <DashboardGrid
            widgets={layout.widgets}
            onLayoutChange={handleLayoutChange}
            editable={isEditMode}
            renderWidget={renderWidget}
            className="min-h-[600px]"
          />
        )}
      </AnimatePresence>
    </div>
  );
};

export default ExecutiveDashboard;
