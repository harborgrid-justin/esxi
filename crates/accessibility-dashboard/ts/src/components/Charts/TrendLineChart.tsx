/**
 * Trend Line Chart
 * Line chart displaying compliance trends over time
 */

import React from 'react';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js';
import { Line } from 'react-chartjs-2';
import { format } from 'date-fns';
import type { TrendDataPoint } from '../../types';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

export interface TrendLineChartProps {
  data: TrendDataPoint[];
  title?: string;
  showIssueCount?: boolean;
  showCriticalCount?: boolean;
  height?: number;
  className?: string;
}

export const TrendLineChart: React.FC<TrendLineChartProps> = ({
  data,
  title = 'Compliance Trend',
  showIssueCount = true,
  showCriticalCount = true,
  height = 300,
  className,
}) => {
  const labels = data.map((point) => format(point.date, 'MMM d'));

  const datasets = [
    {
      label: 'Compliance Score',
      data: data.map((point) => point.score),
      borderColor: '#3b82f6',
      backgroundColor: 'rgba(59, 130, 246, 0.1)',
      borderWidth: 2,
      fill: true,
      tension: 0.4,
      pointRadius: 4,
      pointHoverRadius: 6,
      yAxisID: 'y',
    },
  ];

  if (showIssueCount) {
    datasets.push({
      label: 'Total Issues',
      data: data.map((point) => point.issueCount),
      borderColor: '#f59e0b',
      backgroundColor: 'rgba(245, 158, 11, 0.1)',
      borderWidth: 2,
      fill: false,
      tension: 0.4,
      pointRadius: 4,
      pointHoverRadius: 6,
      yAxisID: 'y1',
    });
  }

  if (showCriticalCount) {
    datasets.push({
      label: 'Critical Issues',
      data: data.map((point) => point.criticalCount),
      borderColor: '#ef4444',
      backgroundColor: 'rgba(239, 68, 68, 0.1)',
      borderWidth: 2,
      fill: false,
      tension: 0.4,
      pointRadius: 4,
      pointHoverRadius: 6,
      yAxisID: 'y1',
    });
  }

  const chartData = {
    labels,
    datasets,
  };

  const options = {
    responsive: true,
    maintainAspectRatio: false,
    interaction: {
      mode: 'index' as const,
      intersect: false,
    },
    plugins: {
      legend: {
        display: true,
        position: 'top' as const,
        labels: {
          padding: 15,
          font: {
            size: 12,
          },
          color: '#374151',
          usePointStyle: true,
          pointStyle: 'circle',
        },
      },
      tooltip: {
        enabled: true,
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        padding: 12,
        titleFont: {
          size: 14,
        },
        bodyFont: {
          size: 13,
        },
        callbacks: {
          title: (context: any) => {
            const index = context[0].dataIndex;
            return format(data[index].date, 'MMMM d, yyyy');
          },
          label: (context: any) => {
            const label = context.dataset.label || '';
            const value = context.parsed.y;

            if (label === 'Compliance Score') {
              return `${label}: ${value}%`;
            }
            return `${label}: ${value}`;
          },
        },
      },
    },
    scales: {
      x: {
        grid: {
          display: false,
        },
        ticks: {
          color: '#6b7280',
          font: {
            size: 11,
          },
        },
      },
      y: {
        type: 'linear' as const,
        display: true,
        position: 'left' as const,
        title: {
          display: true,
          text: 'Compliance Score (%)',
          color: '#374151',
          font: {
            size: 12,
            weight: 'bold',
          },
        },
        min: 0,
        max: 100,
        grid: {
          color: 'rgba(0, 0, 0, 0.05)',
        },
        ticks: {
          color: '#6b7280',
          font: {
            size: 11,
          },
          callback: (value: any) => `${value}%`,
        },
      },
      y1: {
        type: 'linear' as const,
        display: showIssueCount || showCriticalCount,
        position: 'right' as const,
        title: {
          display: true,
          text: 'Issue Count',
          color: '#374151',
          font: {
            size: 12,
            weight: 'bold',
          },
        },
        grid: {
          drawOnChartArea: false,
        },
        ticks: {
          color: '#6b7280',
          font: {
            size: 11,
          },
        },
      },
    },
  };

  if (data.length === 0) {
    return (
      <div className={className}>
        {title && (
          <h3 className="text-sm font-semibold text-gray-900 mb-4">
            {title}
          </h3>
        )}
        <div className="flex items-center justify-center h-64 bg-gray-50 rounded-lg border border-gray-200">
          <p className="text-gray-500 text-sm">No trend data available</p>
        </div>
      </div>
    );
  }

  return (
    <div className={className}>
      {title && (
        <h3 className="text-sm font-semibold text-gray-900 mb-4">
          {title}
        </h3>
      )}
      <div style={{ height }}>
        <Line data={chartData} options={options} />
      </div>
    </div>
  );
};
