/**
 * Enterprise Dashboard - Basic Usage Example
 * $983M SaaS Platform
 */

import React from 'react';
import {
  ExecutiveDashboard,
  useDashboard,
  KPICard,
  AlertWidget,
  ActivityWidget,
  QuotaWidget,
  RevenueChart,
  UsageChart,
  PerformanceChart,
  GeoChart,
  type KPIMetric,
  type Alert,
  type ActivityLogEntry,
  type QuotaUsage,
} from '@esxi/enterprise-dashboard';

// ============================================================================
// Example 1: Simple Dashboard
// ============================================================================
export function SimpleDashboard() {
  return (
    <div className="min-h-screen bg-gray-950 p-8">
      <ExecutiveDashboard
        layoutId="exec-dashboard"
        defaultTimeRange="24h"
        enableRealTime={true}
      />
    </div>
  );
}

// ============================================================================
// Example 2: Custom KPI Dashboard
// ============================================================================
export function CustomKPIDashboard() {
  const kpis: KPIMetric[] = [
    {
      id: 'revenue',
      label: 'Monthly Revenue',
      value: 1250000,
      previousValue: 1100000,
      unit: 'USD',
      format: 'currency',
      trend: 'up',
      trendValue: 13.6,
      sparklineData: [1000000, 1050000, 1100000, 1150000, 1250000],
      target: 1500000,
      status: 'healthy',
      description: 'Total monthly recurring revenue',
      icon: '💰',
      color: '#10b981',
    },
    {
      id: 'users',
      label: 'Active Users',
      value: 15234,
      previousValue: 14100,
      format: 'number',
      trend: 'up',
      trendValue: 8.0,
      sparklineData: [14000, 14200, 14500, 14800, 15234],
      status: 'healthy',
      icon: '👥',
      color: '#3b82f6',
    },
    {
      id: 'latency',
      label: 'Avg Response Time',
      value: 145,
      previousValue: 180,
      unit: 'ms',
      format: 'duration',
      trend: 'down',
      trendValue: 19.4,
      sparklineData: [180, 175, 165, 155, 145],
      threshold: {
        warning: 200,
        critical: 300,
      },
      status: 'healthy',
      icon: '⚡',
      color: '#f59e0b',
    },
    {
      id: 'availability',
      label: 'System Availability',
      value: 99.98,
      unit: '%',
      format: 'percentage',
      trend: 'stable',
      trendValue: 0.01,
      sparklineData: [99.95, 99.96, 99.97, 99.98, 99.98],
      target: 99.99,
      status: 'healthy',
      icon: '🎯',
      color: '#8b5cf6',
    },
  ];

  return (
    <div className="min-h-screen bg-gray-950 p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-3xl font-bold text-white mb-8">KPI Dashboard</h1>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {kpis.map((kpi) => (
            <KPICard key={kpi.id} metric={kpi} showSparkline={true} />
          ))}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Example 3: Monitoring Dashboard
// ============================================================================
export function MonitoringDashboard() {
  const alerts: Alert[] = [
    {
      id: 'alert-1',
      severity: 'critical',
      title: 'High CPU Usage Detected',
      message: 'CPU usage exceeded 90% threshold on production servers',
      source: 'Infrastructure Monitor',
      timestamp: new Date(Date.now() - 300000),
      status: 'active',
      impact: {
        services: ['api', 'web'],
        users: 1500,
        revenue: 5000,
      },
    },
    {
      id: 'alert-2',
      severity: 'high',
      title: 'Database Connection Pool Exhaustion',
      message: 'Connection pool utilization at 95%',
      source: 'Database Monitor',
      timestamp: new Date(Date.now() - 600000),
      status: 'active',
      impact: {
        services: ['database'],
        users: 500,
        revenue: 2000,
      },
    },
  ];

  const activities: ActivityLogEntry[] = [
    {
      id: 'activity-1',
      timestamp: new Date(Date.now() - 120000),
      type: 'deployment',
      action: 'Deployed new version',
      actor: {
        id: 'user-1',
        name: 'John Doe',
        type: 'user',
      },
      description: 'Deployed v2.5.0 to production environment',
      severity: 'info',
    },
    {
      id: 'activity-2',
      timestamp: new Date(Date.now() - 240000),
      type: 'security',
      action: 'Failed login attempt',
      actor: {
        id: 'unknown',
        name: 'Unknown User',
        type: 'user',
      },
      description: 'Multiple failed login attempts detected',
      severity: 'warning',
      ipAddress: '192.168.1.100',
    },
  ];

  const quotas: QuotaUsage[] = [
    {
      id: 'quota-1',
      name: 'API Requests',
      category: 'api',
      current: 4500000,
      limit: 5000000,
      unit: 'requests',
      percentage: 90,
      trend: 'up',
      resetDate: new Date(Date.now() + 86400000 * 7),
      warnings: [
        {
          level: 90,
          message: 'Approaching API quota limit',
        },
      ],
      forecast: {
        exhaustionDate: new Date(Date.now() + 86400000 * 3),
        confidence: 0.85,
      },
    },
    {
      id: 'quota-2',
      name: 'Storage',
      category: 'storage',
      current: 750 * 1024 * 1024 * 1024,
      limit: 1000 * 1024 * 1024 * 1024,
      unit: 'bytes',
      percentage: 75,
      trend: 'stable',
      resetDate: new Date(Date.now() + 86400000 * 30),
    },
  ];

  return (
    <div className="min-h-screen bg-gray-950 p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-3xl font-bold text-white mb-8">Monitoring Dashboard</h1>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          <AlertWidget
            alerts={alerts}
            maxDisplay={10}
            onAcknowledge={(id) => console.log('Acknowledged:', id)}
            onResolve={(id) => console.log('Resolved:', id)}
          />

          <ActivityWidget
            activities={activities}
            maxDisplay={20}
            showFilters={true}
          />
        </div>

        <QuotaWidget
          quotas={quotas}
          warningThreshold={75}
          criticalThreshold={90}
        />
      </div>
    </div>
  );
}

// ============================================================================
// Example 4: Analytics Dashboard
// ============================================================================
export function AnalyticsDashboard() {
  // This would typically come from your API
  const revenueData = Array.from({ length: 12 }, (_, i) => ({
    period: `Month ${i + 1}`,
    revenue: Math.random() * 1000000 + 500000,
    cost: Math.random() * 400000 + 200000,
    profit: 0,
    margin: 0,
    customers: Math.floor(Math.random() * 1000 + 500),
    arpu: 0,
    churnRate: Math.random() * 5,
    growthRate: Math.random() * 20 - 5,
  })).map(item => ({
    ...item,
    profit: item.revenue - item.cost,
    margin: ((item.revenue - item.cost) / item.revenue) * 100,
    arpu: item.revenue / item.customers,
  }));

  const usageData = Array.from({ length: 24 }, (_, i) => ({
    timestamp: new Date(Date.now() - (23 - i) * 3600000),
    activeUsers: Math.floor(Math.random() * 5000 + 10000),
    apiCalls: Math.floor(Math.random() * 100000 + 50000),
    dataTransfer: Math.random() * 1024 * 1024 * 1024 * 10,
    storageUsed: Math.random() * 1024 * 1024 * 1024 * 100,
    cpuUsage: Math.random() * 40 + 30,
    memoryUsage: Math.random() * 30 + 50,
    requestLatency: Math.random() * 100 + 50,
    errorRate: Math.random() * 2,
    successRate: 100 - Math.random() * 2,
    peakConcurrency: Math.floor(Math.random() * 1000 + 500),
  }));

  return (
    <div className="min-h-screen bg-gray-950 p-8">
      <div className="max-w-7xl mx-auto">
        <h1 className="text-3xl font-bold text-white mb-8">Analytics Dashboard</h1>

        <div className="space-y-6">
          <RevenueChart
            data={revenueData}
            type="composed"
            showForecast={true}
            showProfit={true}
            showMargin={true}
            height={400}
          />

          <UsageChart
            data={usageData}
            metrics={['activeUsers', 'apiCalls', 'requestLatency']}
            type="area"
            height={350}
          />
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Example 5: Custom Hook Usage
// ============================================================================
export function CustomHookExample() {
  const {
    kpis,
    alerts,
    activities,
    quotas,
    isLoading,
    error,
    refresh,
    criticalAlerts,
    activeAlerts,
  } = useDashboard({
    layoutId: 'custom-dashboard',
    autoLoad: true,
    autoRefresh: true,
    refreshInterval: 30000,
  });

  if (isLoading) {
    return (
      <div className="min-h-screen bg-gray-950 flex items-center justify-center">
        <div className="text-white">Loading dashboard...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-950 flex items-center justify-center">
        <div className="text-red-400">Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-950 p-8">
      <div className="max-w-7xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-3xl font-bold text-white">Custom Dashboard</h1>
          <button
            onClick={refresh}
            className="px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg"
          >
            Refresh
          </button>
        </div>

        <div className="grid grid-cols-3 gap-4 mb-8">
          <div className="bg-gray-800 rounded-lg p-4">
            <div className="text-gray-400 text-sm">Total KPIs</div>
            <div className="text-3xl font-bold text-white">{kpis.length}</div>
          </div>
          <div className="bg-red-900/20 border border-red-500/30 rounded-lg p-4">
            <div className="text-gray-400 text-sm">Critical Alerts</div>
            <div className="text-3xl font-bold text-red-400">{criticalAlerts.length}</div>
          </div>
          <div className="bg-blue-900/20 border border-blue-500/30 rounded-lg p-4">
            <div className="text-gray-400 text-sm">Active Alerts</div>
            <div className="text-3xl font-bold text-blue-400">{activeAlerts.length}</div>
          </div>
        </div>

        {/* Your custom dashboard UI */}
      </div>
    </div>
  );
}

export default SimpleDashboard;
