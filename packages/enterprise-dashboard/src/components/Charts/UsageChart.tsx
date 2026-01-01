/**
 * Usage Chart Component
 * Platform usage metrics visualization
 */

import React, { useMemo } from 'react';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import { format } from 'date-fns';
import type { UsageMetrics } from '../../types';

export interface UsageChartProps {
  data: UsageMetrics[];
  metrics?: Array<keyof UsageMetrics>;
  type?: 'line' | 'area';
  height?: number;
  className?: string;
  showGrid?: boolean;
  theme?: 'light' | 'dark';
}

const METRIC_CONFIG: Record<
  keyof Omit<UsageMetrics, 'timestamp'>,
  { label: string; color: string; unit: string; format: (v: number) => string }
> = {
  activeUsers: {
    label: 'Active Users',
    color: '#3b82f6',
    unit: 'users',
    format: (v) => new Intl.NumberFormat('en-US').format(v),
  },
  apiCalls: {
    label: 'API Calls',
    color: '#10b981',
    unit: 'calls',
    format: (v) => (v >= 1000 ? `${(v / 1000).toFixed(1)}K` : v.toString()),
  },
  dataTransfer: {
    label: 'Data Transfer',
    color: '#f59e0b',
    unit: 'GB',
    format: (v) => {
      const gb = v / (1024 * 1024 * 1024);
      return gb >= 1 ? `${gb.toFixed(2)}GB` : `${(gb * 1024).toFixed(0)}MB`;
    },
  },
  storageUsed: {
    label: 'Storage Used',
    color: '#8b5cf6',
    unit: 'GB',
    format: (v) => {
      const gb = v / (1024 * 1024 * 1024);
      return `${gb.toFixed(2)}GB`;
    },
  },
  cpuUsage: {
    label: 'CPU Usage',
    color: '#ef4444',
    unit: '%',
    format: (v) => `${v.toFixed(1)}%`,
  },
  memoryUsage: {
    label: 'Memory Usage',
    color: '#ec4899',
    unit: '%',
    format: (v) => `${v.toFixed(1)}%`,
  },
  requestLatency: {
    label: 'Request Latency',
    color: '#06b6d4',
    unit: 'ms',
    format: (v) => `${v.toFixed(0)}ms`,
  },
  errorRate: {
    label: 'Error Rate',
    color: '#f43f5e',
    unit: '%',
    format: (v) => `${v.toFixed(2)}%`,
  },
  successRate: {
    label: 'Success Rate',
    color: '#22c55e',
    unit: '%',
    format: (v) => `${v.toFixed(2)}%`,
  },
  peakConcurrency: {
    label: 'Peak Concurrency',
    color: '#a855f7',
    unit: 'concurrent',
    format: (v) => new Intl.NumberFormat('en-US').format(v),
  },
};

export const UsageChart: React.FC<UsageChartProps> = ({
  data,
  metrics = ['activeUsers', 'apiCalls', 'requestLatency'],
  type = 'area',
  height = 350,
  className,
  showGrid = true,
  theme = 'dark',
}) => {
  const isDark = theme === 'dark';

  // Format chart data
  const chartData = useMemo(() => {
    return data.map((item) => ({
      ...item,
      time: format(new Date(item.timestamp), 'HH:mm'),
      fullTime: format(new Date(item.timestamp), 'MMM dd, HH:mm'),
    }));
  }, [data]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || !payload.length) return null;

    return (
      <div className="bg-gray-900/95 backdrop-blur-sm border border-gray-700 rounded-lg p-4 shadow-xl">
        <p className="text-sm font-semibold text-gray-300 mb-2">
          {payload[0]?.payload?.fullTime}
        </p>
        {payload.map((entry: any, index: number) => {
          const metricKey = entry.dataKey as keyof typeof METRIC_CONFIG;
          const config = METRIC_CONFIG[metricKey];
          if (!config) return null;

          return (
            <div key={index} className="flex items-center justify-between gap-4 mb-1">
              <span className="flex items-center gap-2 text-sm">
                <span
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: entry.color }}
                />
                <span className="text-gray-400">{config.label}:</span>
              </span>
              <span className="font-semibold text-white">
                {config.format(entry.value)}
              </span>
            </div>
          );
        })}
      </div>
    );
  };

  const gridColor = isDark ? '#374151' : '#e5e7eb';
  const textColor = isDark ? '#9ca3af' : '#6b7280';

  // Calculate statistics
  const stats = useMemo(() => {
    if (!data || data.length === 0) return null;

    return metrics.map((metric) => {
      const values = data.map((d) => d[metric] as number);
      const current = values[values.length - 1] || 0;
      const previous = values[values.length - 2] || 0;
      const avg = values.reduce((a, b) => a + b, 0) / values.length;
      const max = Math.max(...values);
      const min = Math.min(...values);
      const change = previous !== 0 ? ((current - previous) / previous) * 100 : 0;

      return {
        metric,
        current,
        avg,
        max,
        min,
        change,
        config: METRIC_CONFIG[metric as keyof typeof METRIC_CONFIG],
      };
    });
  }, [data, metrics]);

  const renderChart = () => {
    const chartProps = {
      data: chartData,
      margin: { top: 10, right: 30, left: 0, bottom: 0 },
    };

    if (type === 'area') {
      return (
        <AreaChart {...chartProps}>
          <defs>
            {metrics.map((metric) => {
              const config = METRIC_CONFIG[metric as keyof typeof METRIC_CONFIG];
              if (!config) return null;
              return (
                <linearGradient key={metric} id={`gradient-${metric}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%" stopColor={config.color} stopOpacity={0.3} />
                  <stop offset="95%" stopColor={config.color} stopOpacity={0} />
                </linearGradient>
              );
            })}
          </defs>
          {showGrid && <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />}
          <XAxis dataKey="time" stroke={textColor} fontSize={12} />
          <YAxis stroke={textColor} fontSize={12} />
          <Tooltip content={<CustomTooltip />} />
          <Legend wrapperStyle={{ color: textColor }} />
          {metrics.map((metric) => {
            const config = METRIC_CONFIG[metric as keyof typeof METRIC_CONFIG];
            if (!config) return null;
            return (
              <Area
                key={metric}
                type="monotone"
                dataKey={metric}
                name={config.label}
                stroke={config.color}
                fill={`url(#gradient-${metric})`}
                strokeWidth={2}
              />
            );
          })}
        </AreaChart>
      );
    }

    return (
      <LineChart {...chartProps}>
        {showGrid && <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />}
        <XAxis dataKey="time" stroke={textColor} fontSize={12} />
        <YAxis stroke={textColor} fontSize={12} />
        <Tooltip content={<CustomTooltip />} />
        <Legend wrapperStyle={{ color: textColor }} />
        {metrics.map((metric) => {
          const config = METRIC_CONFIG[metric as keyof typeof METRIC_CONFIG];
          if (!config) return null;
          return (
            <Line
              key={metric}
              type="monotone"
              dataKey={metric}
              name={config.label}
              stroke={config.color}
              strokeWidth={2}
              dot={{ r: 3 }}
            />
          );
        })}
      </LineChart>
    );
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className={clsx('rounded-xl border border-gray-800 bg-gray-900/50 p-6', className)}
    >
      {/* Header */}
      <div className="mb-6">
        <h3 className="text-lg font-semibold text-white mb-4">Usage Metrics</h3>

        {/* Statistics Grid */}
        {stats && (
          <div className="grid grid-cols-3 gap-3 mb-4">
            {stats.map(({ metric, current, avg, config, change }) => (
              <div key={metric} className="bg-gray-800/50 rounded-lg p-3">
                <div className="flex items-center justify-between mb-1">
                  <div className="text-xs text-gray-400">{config.label}</div>
                  <div
                    className={clsx('text-xs font-semibold', {
                      'text-green-400': change > 0,
                      'text-red-400': change < 0,
                      'text-gray-400': change === 0,
                    })}
                  >
                    {change > 0 ? '+' : ''}
                    {change.toFixed(1)}%
                  </div>
                </div>
                <div className="text-lg font-bold" style={{ color: config.color }}>
                  {config.format(current)}
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  Avg: {config.format(avg)}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Chart */}
      <ResponsiveContainer width="100%" height={height}>
        {renderChart()}
      </ResponsiveContainer>
    </motion.div>
  );
};

export default UsageChart;
