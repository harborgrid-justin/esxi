/**
 * ChartWidget - Chart visualization widget
 */

import React from 'react';
import { Widget, ChartWidgetConfig } from '../../types';
import { useDataSource } from '../../hooks/useDataSource';
import { LineChart } from '../Charts/LineChart';
import { BarChart } from '../Charts/BarChart';
import { PieChart } from '../Charts/PieChart';
import { ScatterChart } from '../Charts/ScatterChart';
import { HeatmapChart } from '../Charts/HeatmapChart';

export interface ChartWidgetProps {
  widget: Widget;
}

export const ChartWidget: React.FC<ChartWidgetProps> = ({ widget }) => {
  const { data, loading, error } = useDataSource(widget.data_source);
  const config = widget.config as ChartWidgetConfig;

  if (loading) {
    return (
      <div className="chart-loading">
        <div className="spinner"></div>
        <p>Loading chart...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="chart-error">
        <p>Error loading chart: {error.message}</p>
      </div>
    );
  }

  const renderChart = () => {
    const chartData = Array.isArray(data) ? data : [];

    switch (config.chart_type) {
      case 'line':
      case 'area':
        return (
          <LineChart
            data={chartData}
            xKey={config.x_axis}
            yKeys={config.y_axis}
            options={config.options}
            type={config.chart_type}
          />
        );
      case 'bar':
      case 'column':
        return (
          <BarChart
            data={chartData}
            xKey={config.x_axis}
            yKeys={config.y_axis}
            options={config.options}
            type={config.chart_type}
          />
        );
      case 'pie':
      case 'donut':
        return (
          <PieChart
            data={chartData}
            nameKey={config.x_axis}
            valueKey={config.y_axis[0]}
            options={config.options}
            type={config.chart_type}
          />
        );
      case 'scatter':
        return (
          <ScatterChart
            data={chartData}
            xKey={config.x_axis}
            yKey={config.y_axis[0]}
            options={config.options}
          />
        );
      case 'heatmap':
        return (
          <HeatmapChart
            data={chartData}
            xKey={config.x_axis}
            yKey={config.y_axis[0]}
            options={config.options}
          />
        );
      default:
        return <div>Unsupported chart type: {config.chart_type}</div>;
    }
  };

  return (
    <div className="chart-widget">
      {renderChart()}

      <style jsx>{`
        .chart-widget {
          width: 100%;
          height: 100%;
          min-height: 200px;
        }

        .chart-loading,
        .chart-error {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: #666;
        }

        .spinner {
          border: 3px solid #f3f3f3;
          border-top: 3px solid #1976d2;
          border-radius: 50%;
          width: 40px;
          height: 40px;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% {
            transform: rotate(0deg);
          }
          100% {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </div>
  );
};
