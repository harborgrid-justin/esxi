/**
 * Performance Chart Component
 * System performance metrics visualization
 */

import React, { useMemo } from 'react';
import {
  ComposedChart,
  Line,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  Radar,
} from 'recharts';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import { format } from 'date-fns';
import type { PerformanceMetrics } from '../../types';

export interface PerformanceChartProps {
  data: PerformanceMetrics[];
  type?: 'timeline' | 'radar' | 'percentiles';
  height?: number;
  className?: string;
  theme?: 'light' | 'dark';
}

export const PerformanceChart: React.FC<PerformanceChartProps> = ({
  data,
  type = 'timeline',
  height = 400,
  className,
  theme = 'dark',
}) => {
  const isDark = theme === 'dark';

  // Format data for charts
  const chartData = useMemo(() => {
    return data.map((item) => ({
      ...item,
      time: format(new Date(item.timestamp), 'HH:mm'),
      fullTime: format(new Date(item.timestamp), 'MMM dd, HH:mm:ss'),
      p50: item.responseTime.p50,
      p95: item.responseTime.p95,
      p99: item.responseTime.p99,
      avgResponseTime: item.responseTime.avg,
      availabilityPct: item.availability * 100,
      errorRatePct: item.errorRate * 100,
      cacheHitRatePct: item.cacheHitRate * 100,
      apdexScore: item.apdex * 100,
    }));
  }, [data]);

  // Radar chart data (latest metrics)
  const radarData = useMemo(() => {
    if (!data || data.length === 0) return [];

    const latest = data[data.length - 1];
    if (!latest) return [];

    return [
      { metric: 'Availability', value: latest.availability * 100, fullMark: 100 },
      { metric: 'Cache Hit', value: latest.cacheHitRate * 100, fullMark: 100 },
      { metric: 'Apdex', value: latest.apdex * 100, fullMark: 100 },
      { metric: 'Throughput', value: Math.min((latest.throughput / 10000) * 100, 100), fullMark: 100 },
      { metric: 'Low Errors', value: 100 - (latest.errorRate * 100), fullMark: 100 },
    ];
  }, [data]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (!data || data.length === 0) return null;

    const latest = data[data.length - 1];
    if (!latest) return null;

    const avgAvailability = data.reduce((sum, d) => sum + d.availability, 0) / data.length;
    const avgThroughput = data.reduce((sum, d) => sum + d.throughput, 0) / data.length;
    const avgErrorRate = data.reduce((sum, d) => sum + d.errorRate, 0) / data.length;
    const avgP99 = data.reduce((sum, d) => sum + d.responseTime.p99, 0) / data.length;

    return {
      availability: latest.availability * 100,
      avgAvailability: avgAvailability * 100,
      throughput: latest.throughput,
      avgThroughput,
      errorRate: latest.errorRate * 100,
      avgErrorRate: avgErrorRate * 100,
      p99: latest.responseTime.p99,
      avgP99,
      apdex: latest.apdex,
      cacheHitRate: latest.cacheHitRate * 100,
    };
  }, [data]);

  const gridColor = isDark ? '#374151' : '#e5e7eb';
  const textColor = isDark ? '#9ca3af' : '#6b7280';

  // Custom tooltip
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (!active || !payload || !payload.length) return null;

    return (
      <div className="bg-gray-900/95 backdrop-blur-sm border border-gray-700 rounded-lg p-4 shadow-xl">
        <p className="text-sm font-semibold text-gray-300 mb-2">{payload[0]?.payload?.fullTime}</p>
        {payload.map((entry: any, index: number) => (
          <div key={index} className="flex items-center justify-between gap-4 mb-1">
            <span className="flex items-center gap-2 text-sm">
              <span
                className="w-3 h-3 rounded-full"
                style={{ backgroundColor: entry.color }}
              />
              <span className="text-gray-400">{entry.name}:</span>
            </span>
            <span className="font-semibold text-white">
              {typeof entry.value === 'number'
                ? entry.name.includes('Rate') || entry.name.includes('%')
                  ? `${entry.value.toFixed(2)}%`
                  : entry.name.includes('ms')
                  ? `${entry.value.toFixed(0)}ms`
                  : entry.value.toFixed(0)
                : entry.value}
            </span>
          </div>
        ))}
      </div>
    );
  };

  const renderChart = () => {
    switch (type) {
      case 'radar':
        return (
          <RadarChart data={radarData}>
            <PolarGrid stroke={gridColor} />
            <PolarAngleAxis dataKey="metric" stroke={textColor} fontSize={12} />
            <PolarRadiusAxis angle={90} domain={[0, 100]} stroke={textColor} fontSize={10} />
            <Radar
              name="Performance"
              dataKey="value"
              stroke="#3b82f6"
              fill="#3b82f6"
              fillOpacity={0.3}
              strokeWidth={2}
            />
            <Tooltip />
          </RadarChart>
        );

      case 'percentiles':
        return (
          <ComposedChart data={chartData} margin={{ top: 20, right: 30, left: 0, bottom: 0 }}>
            <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
            <XAxis dataKey="time" stroke={textColor} fontSize={12} />
            <YAxis stroke={textColor} fontSize={12} label={{ value: 'ms', angle: -90, position: 'insideLeft' }} />
            <Tooltip content={<CustomTooltip />} />
            <Legend wrapperStyle={{ color: textColor }} />
            <Line
              type="monotone"
              dataKey="p50"
              name="P50"
              stroke="#10b981"
              strokeWidth={2}
              dot={false}
            />
            <Line
              type="monotone"
              dataKey="p95"
              name="P95"
              stroke="#f59e0b"
              strokeWidth={2}
              dot={false}
            />
            <Line
              type="monotone"
              dataKey="p99"
              name="P99"
              stroke="#ef4444"
              strokeWidth={2}
              dot={false}
            />
            <Line
              type="monotone"
              dataKey="avgResponseTime"
              name="Avg"
              stroke="#3b82f6"
              strokeWidth={2}
              strokeDasharray="5 5"
              dot={false}
            />
          </ComposedChart>
        );

      case 'timeline':
      default:
        return (
          <ComposedChart data={chartData} margin={{ top: 20, right: 30, left: 0, bottom: 0 }}>
            <defs>
              <linearGradient id="throughputGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
            <XAxis dataKey="time" stroke={textColor} fontSize={12} />
            <YAxis yAxisId="left" stroke={textColor} fontSize={12} />
            <YAxis yAxisId="right" orientation="right" stroke={textColor} fontSize={12} />
            <Tooltip content={<CustomTooltip />} />
            <Legend wrapperStyle={{ color: textColor }} />
            <Bar
              yAxisId="left"
              dataKey="throughput"
              name="Throughput"
              fill="url(#throughputGradient)"
              radius={[4, 4, 0, 0]}
            />
            <Line
              yAxisId="right"
              type="monotone"
              dataKey="availabilityPct"
              name="Availability %"
              stroke="#10b981"
              strokeWidth={2}
              dot={{ r: 3 }}
            />
            <Line
              yAxisId="right"
              type="monotone"
              dataKey="errorRatePct"
              name="Error Rate %"
              stroke="#ef4444"
              strokeWidth={2}
              dot={{ r: 3 }}
            />
            <Line
              yAxisId="right"
              type="monotone"
              dataKey="cacheHitRatePct"
              name="Cache Hit %"
              stroke="#8b5cf6"
              strokeWidth={2}
              dot={false}
              strokeDasharray="5 5"
            />
          </ComposedChart>
        );
    }
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
        <h3 className="text-lg font-semibold text-white mb-4">Performance Metrics</h3>

        {/* Statistics Grid */}
        {stats && (
          <div className="grid grid-cols-5 gap-3 mb-4">
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Availability</div>
              <div className="text-xl font-bold text-green-400">
                {stats.availability.toFixed(2)}%
              </div>
              <div className="text-xs text-gray-500 mt-1">
                Avg: {stats.avgAvailability.toFixed(2)}%
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Throughput</div>
              <div className="text-xl font-bold text-blue-400">
                {(stats.throughput / 1000).toFixed(1)}K
              </div>
              <div className="text-xs text-gray-500 mt-1">
                Avg: {(stats.avgThroughput / 1000).toFixed(1)}K
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Error Rate</div>
              <div className="text-xl font-bold text-red-400">
                {stats.errorRate.toFixed(3)}%
              </div>
              <div className="text-xs text-gray-500 mt-1">
                Avg: {stats.avgErrorRate.toFixed(3)}%
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">P99 Latency</div>
              <div className="text-xl font-bold text-amber-400">
                {stats.p99.toFixed(0)}ms
              </div>
              <div className="text-xs text-gray-500 mt-1">
                Avg: {stats.avgP99.toFixed(0)}ms
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Apdex Score</div>
              <div className="text-xl font-bold text-violet-400">
                {stats.apdex.toFixed(2)}
              </div>
              <div className="text-xs text-gray-500 mt-1">
                Cache: {stats.cacheHitRate.toFixed(1)}%
              </div>
            </div>
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

export default PerformanceChart;
