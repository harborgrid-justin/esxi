/**
 * PieChart - Pie/donut chart component
 */

import React from 'react';
import {
  PieChart as RechartsPieChart,
  Pie,
  Cell,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import { ChartOptions } from '../../types';

export interface PieChartProps {
  data: any[];
  nameKey: string;
  valueKey: string;
  options?: ChartOptions;
  type?: 'pie' | 'donut';
}

const DEFAULT_COLORS = [
  '#1976d2',
  '#4caf50',
  '#ff9800',
  '#e91e63',
  '#9c27b0',
  '#00bcd4',
  '#795548',
  '#607d8b',
];

export const PieChart: React.FC<PieChartProps> = ({
  data,
  nameKey,
  valueKey,
  options = {
    legend: true,
    grid: false,
    tooltip: true,
    animation: true,
    stacked: false,
  },
  type = 'pie',
}) => {
  const colors = options.colors || DEFAULT_COLORS;
  const innerRadius = type === 'donut' ? '50%' : 0;

  return (
    <ResponsiveContainer width="100%" height="100%">
      <RechartsPieChart>
        <Pie
          data={data}
          cx="50%"
          cy="50%"
          labelLine={false}
          label={({ name, percent }) =>
            `${name}: ${(percent * 100).toFixed(0)}%`
          }
          outerRadius="80%"
          innerRadius={innerRadius}
          fill="#8884d8"
          dataKey={valueKey}
          nameKey={nameKey}
          isAnimationActive={options.animation}
        >
          {data.map((entry, index) => (
            <Cell key={`cell-${index}`} fill={colors[index % colors.length]} />
          ))}
        </Pie>
        {options.tooltip && <Tooltip />}
        {options.legend && <Legend />}
      </RechartsPieChart>
    </ResponsiveContainer>
  );
};
