/**
 * ScatterChart - Scatter plot component
 */

import React from 'react';
import {
  ScatterChart as RechartsScatterChart,
  Scatter,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ZAxis,
} from 'recharts';
import { ChartOptions } from '../../types';

export interface ScatterChartProps {
  data: any[];
  xKey: string;
  yKey: string;
  options?: ChartOptions;
}

const DEFAULT_COLOR = '#1976d2';

export const ScatterChart: React.FC<ScatterChartProps> = ({
  data,
  xKey,
  yKey,
  options = {
    legend: true,
    grid: true,
    tooltip: true,
    animation: true,
    stacked: false,
  },
}) => {
  const color = options.colors?.[0] || DEFAULT_COLOR;

  return (
    <ResponsiveContainer width="100%" height="100%">
      <RechartsScatterChart>
        {options.grid && <CartesianGrid strokeDasharray="3 3" />}
        <XAxis dataKey={xKey} name={xKey} />
        <YAxis dataKey={yKey} name={yKey} />
        <ZAxis range={[60, 400]} />
        {options.tooltip && <Tooltip cursor={{ strokeDasharray: '3 3' }} />}
        {options.legend && <Legend />}
        <Scatter
          name={`${xKey} vs ${yKey}`}
          data={data}
          fill={color}
          isAnimationActive={options.animation}
        />
      </RechartsScatterChart>
    </ResponsiveContainer>
  );
};
