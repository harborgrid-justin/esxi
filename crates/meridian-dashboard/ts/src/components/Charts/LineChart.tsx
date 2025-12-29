/**
 * LineChart - Time series line chart component
 */

import React from 'react';
import {
  LineChart as RechartsLineChart,
  Line,
  AreaChart as RechartsAreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { ChartOptions } from '../../types';

export interface LineChartProps {
  data: any[];
  xKey: string;
  yKeys: string[];
  options?: ChartOptions;
  type?: 'line' | 'area';
}

const DEFAULT_COLORS = [
  '#1976d2',
  '#4caf50',
  '#ff9800',
  '#e91e63',
  '#9c27b0',
  '#00bcd4',
];

export const LineChart: React.FC<LineChartProps> = ({
  data,
  xKey,
  yKeys,
  options = {
    legend: true,
    grid: true,
    tooltip: true,
    animation: true,
    stacked: false,
  },
  type = 'line',
}) => {
  const colors = options.colors || DEFAULT_COLORS;
  const ChartComponent = type === 'area' ? RechartsAreaChart : RechartsLineChart;

  return (
    <ResponsiveContainer width="100%" height="100%">
      <ChartComponent data={data}>
        {options.grid && <CartesianGrid strokeDasharray="3 3" />}
        <XAxis dataKey={xKey} />
        <YAxis />
        {options.tooltip && <Tooltip />}
        {options.legend && <Legend />}
        {type === 'area'
          ? yKeys.map((key, index) => (
              <Area
                key={key}
                type="monotone"
                dataKey={key}
                stackId={options.stacked ? '1' : undefined}
                stroke={colors[index % colors.length]}
                fill={colors[index % colors.length]}
                fillOpacity={0.6}
                isAnimationActive={options.animation}
              />
            ))
          : yKeys.map((key, index) => (
              <Line
                key={key}
                type="monotone"
                dataKey={key}
                stroke={colors[index % colors.length]}
                strokeWidth={2}
                dot={{ r: 4 }}
                isAnimationActive={options.animation}
              />
            ))}
      </ChartComponent>
    </ResponsiveContainer>
  );
};
