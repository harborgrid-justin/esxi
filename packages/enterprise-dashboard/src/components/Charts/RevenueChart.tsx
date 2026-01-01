/**
 * Revenue Chart Component
 * Advanced revenue analytics visualization
 */

import React, { useMemo } from 'react';
import {
  AreaChart,
  Area,
  BarChart,
  Bar,
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ComposedChart,
} from 'recharts';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import type { RevenueData } from '../../types';

export interface RevenueChartProps {
  data: RevenueData[];
  type?: 'area' | 'bar' | 'line' | 'composed';
  showForecast?: boolean;
  showProfit?: boolean;
  showMargin?: boolean;
  height?: number;
  className?: string;
  theme?: 'light' | 'dark';
}

export const RevenueChart: React.FC<RevenueChartProps> = ({
  data,
  type = 'composed',
  showForecast = true,
  showProfit = true,
  showMargin = false,
  height = 400,
  className,
  theme = 'dark',
}) => {
  const colors = {
    revenue: '#3b82f6',
    cost: '#ef4444',
    profit: '#10b981',
    margin: '#f59e0b',
    forecast: '#8b5cf6',
  };

  const isDark = theme === 'dark';

  // Custom tooltip
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (!active || !payload || !payload.length) return null;

    const formatCurrency = (value: number) =>
      new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        minimumFractionDigits: 0,
        maximumFractionDigits: 0,
      }).format(value);

    return (
      <div className="bg-gray-900/95 backdrop-blur-sm border border-gray-700 rounded-lg p-4 shadow-xl">
        <p className="text-sm font-semibold text-gray-300 mb-2">{label}</p>
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
              {entry.name === 'Margin'
                ? `${entry.value.toFixed(1)}%`
                : formatCurrency(entry.value)}
            </span>
          </div>
        ))}
      </div>
    );
  };

  // Format axis
  const formatYAxis = (value: number) => {
    if (value >= 1000000) return `$${(value / 1000000).toFixed(1)}M`;
    if (value >= 1000) return `$${(value / 1000).toFixed(0)}K`;
    return `$${value}`;
  };

  const chartProps = {
    data,
    margin: { top: 20, right: 30, left: 20, bottom: 20 },
  };

  const gridColor = isDark ? '#374151' : '#e5e7eb';
  const textColor = isDark ? '#9ca3af' : '#6b7280';

  const renderChart = () => {
    switch (type) {
      case 'area':
        return (
          <AreaChart {...chartProps}>
            <defs>
              <linearGradient id="revenueGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={colors.revenue} stopOpacity={0.3} />
                <stop offset="95%" stopColor={colors.revenue} stopOpacity={0} />
              </linearGradient>
              <linearGradient id="profitGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={colors.profit} stopOpacity={0.3} />
                <stop offset="95%" stopColor={colors.profit} stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
            <XAxis dataKey="period" stroke={textColor} fontSize={12} />
            <YAxis stroke={textColor} fontSize={12} tickFormatter={formatYAxis} />
            <Tooltip content={<CustomTooltip />} />
            <Legend wrapperStyle={{ color: textColor }} />
            <Area
              type="monotone"
              dataKey="revenue"
              name="Revenue"
              stroke={colors.revenue}
              fill="url(#revenueGradient)"
              strokeWidth={2}
            />
            {showProfit && (
              <Area
                type="monotone"
                dataKey="profit"
                name="Profit"
                stroke={colors.profit}
                fill="url(#profitGradient)"
                strokeWidth={2}
              />
            )}
            {showForecast && (
              <Area
                type="monotone"
                dataKey="forecast"
                name="Forecast"
                stroke={colors.forecast}
                fill="none"
                strokeDasharray="5 5"
                strokeWidth={2}
              />
            )}
          </AreaChart>
        );

      case 'bar':
        return (
          <BarChart {...chartProps}>
            <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
            <XAxis dataKey="period" stroke={textColor} fontSize={12} />
            <YAxis stroke={textColor} fontSize={12} tickFormatter={formatYAxis} />
            <Tooltip content={<CustomTooltip />} />
            <Legend wrapperStyle={{ color: textColor }} />
            <Bar dataKey="revenue" name="Revenue" fill={colors.revenue} radius={[4, 4, 0, 0]} />
            <Bar dataKey="cost" name="Cost" fill={colors.cost} radius={[4, 4, 0, 0]} />
            {showProfit && (
              <Bar dataKey="profit" name="Profit" fill={colors.profit} radius={[4, 4, 0, 0]} />
            )}
          </BarChart>
        );

      case 'line':
        return (
          <LineChart {...chartProps}>
            <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
            <XAxis dataKey="period" stroke={textColor} fontSize={12} />
            <YAxis stroke={textColor} fontSize={12} tickFormatter={formatYAxis} />
            <Tooltip content={<CustomTooltip />} />
            <Legend wrapperStyle={{ color: textColor }} />
            <Line
              type="monotone"
              dataKey="revenue"
              name="Revenue"
              stroke={colors.revenue}
              strokeWidth={2}
              dot={{ r: 4 }}
            />
            {showProfit && (
              <Line
                type="monotone"
                dataKey="profit"
                name="Profit"
                stroke={colors.profit}
                strokeWidth={2}
                dot={{ r: 4 }}
              />
            )}
          </LineChart>
        );

      case 'composed':
      default:
        return (
          <ComposedChart {...chartProps}>
            <defs>
              <linearGradient id="revenueGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor={colors.revenue} stopOpacity={0.3} />
                <stop offset="95%" stopColor={colors.revenue} stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke={gridColor} />
            <XAxis dataKey="period" stroke={textColor} fontSize={12} />
            <YAxis yAxisId="left" stroke={textColor} fontSize={12} tickFormatter={formatYAxis} />
            {showMargin && (
              <YAxis
                yAxisId="right"
                orientation="right"
                stroke={textColor}
                fontSize={12}
                tickFormatter={(v) => `${v}%`}
              />
            )}
            <Tooltip content={<CustomTooltip />} />
            <Legend wrapperStyle={{ color: textColor }} />
            <Area
              yAxisId="left"
              type="monotone"
              dataKey="revenue"
              name="Revenue"
              stroke={colors.revenue}
              fill="url(#revenueGradient)"
              strokeWidth={2}
            />
            <Bar
              yAxisId="left"
              dataKey="cost"
              name="Cost"
              fill={colors.cost}
              radius={[4, 4, 0, 0]}
              opacity={0.8}
            />
            {showProfit && (
              <Line
                yAxisId="left"
                type="monotone"
                dataKey="profit"
                name="Profit"
                stroke={colors.profit}
                strokeWidth={3}
                dot={{ r: 5, fill: colors.profit }}
              />
            )}
            {showMargin && (
              <Line
                yAxisId="right"
                type="monotone"
                dataKey="margin"
                name="Margin"
                stroke={colors.margin}
                strokeWidth={2}
                strokeDasharray="5 5"
                dot={false}
              />
            )}
            {showForecast && (
              <Line
                yAxisId="left"
                type="monotone"
                dataKey="forecast"
                name="Forecast"
                stroke={colors.forecast}
                strokeWidth={2}
                strokeDasharray="8 4"
                dot={false}
              />
            )}
          </ComposedChart>
        );
    }
  };

  // Calculate summary metrics
  const summary = useMemo(() => {
    if (!data || data.length === 0) return null;

    const totalRevenue = data.reduce((sum, item) => sum + item.revenue, 0);
    const totalCost = data.reduce((sum, item) => sum + item.cost, 0);
    const totalProfit = totalRevenue - totalCost;
    const avgMargin = data.reduce((sum, item) => sum + item.margin, 0) / data.length;

    return {
      totalRevenue,
      totalCost,
      totalProfit,
      avgMargin,
    };
  }, [data]);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
      className={clsx('rounded-xl border border-gray-800 bg-gray-900/50 p-6', className)}
    >
      {/* Header */}
      <div className="mb-6">
        <h3 className="text-lg font-semibold text-white mb-4">Revenue Analytics</h3>

        {/* Summary Metrics */}
        {summary && (
          <div className="grid grid-cols-4 gap-4 mb-4">
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Total Revenue</div>
              <div className="text-xl font-bold text-blue-400">
                ${(summary.totalRevenue / 1000000).toFixed(2)}M
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Total Cost</div>
              <div className="text-xl font-bold text-red-400">
                ${(summary.totalCost / 1000000).toFixed(2)}M
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Net Profit</div>
              <div className="text-xl font-bold text-green-400">
                ${(summary.totalProfit / 1000000).toFixed(2)}M
              </div>
            </div>
            <div className="bg-gray-800/50 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Avg Margin</div>
              <div className="text-xl font-bold text-amber-400">
                {summary.avgMargin.toFixed(1)}%
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

export default RevenueChart;
