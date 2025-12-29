/**
 * BarChart - Bar/column chart component
 */

import React from 'react';
import {
  BarChart as RechartsBarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { ChartOptions } from '../../types';

export interface BarChartProps {
  data: any[];
  xKey: string;
  yKeys: string[];
  options?: ChartOptions;
  type?: 'bar' | 'column';
}

const DEFAULT_COLORS = [
  '#1976d2',
  '#4caf50',
  '#ff9800',
  '#e91e63',
  '#9c27b0',
  '#00bcd4',
];

export const BarChart: React.FC<BarChartProps> = ({
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
  type = 'column',
}) => {
  const colors = options.colors || DEFAULT_COLORS;
  const layout = type === 'bar' ? 'horizontal' : 'vertical';

  return (
    <ResponsiveContainer width="100%" height="100%">
      <RechartsBarChart
        data={data}
        layout={layout === 'horizontal' ? 'horizontal' : 'vertical'}
      >
        {options.grid && <CartesianGrid strokeDasharray="3 3" />}
        {layout === 'horizontal' ? (
          <>
            <XAxis type="number" />
            <YAxis dataKey={xKey} type="category" />
          </>
        ) : (
          <>
            <XAxis dataKey={xKey} />
            <YAxis />
          </>
        )}
        {options.tooltip && <Tooltip />}
        {options.legend && <Legend />}
        {yKeys.map((key, index) => (
          <Bar
            key={key}
            dataKey={key}
            stackId={options.stacked ? '1' : undefined}
            fill={colors[index % colors.length]}
            isAnimationActive={options.animation}
          />
        ))}
      </RechartsBarChart>
    </ResponsiveContainer>
  );
};
