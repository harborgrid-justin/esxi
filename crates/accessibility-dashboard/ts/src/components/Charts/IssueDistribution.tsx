/**
 * Issue Distribution Chart
 * Pie/Donut chart showing issue breakdown by category or severity
 */

import React from 'react';
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js';
import { Pie, Doughnut } from 'react-chartjs-2';
import type {
  CategoryBreakdown,
  SeverityBreakdown,
  IssueCategory,
  IssueSeverity,
} from '../../types';
import { getCategoryColor, getSeverityColor } from '../../utils/calculations';

ChartJS.register(ArcElement, Tooltip, Legend);

export interface IssueDistributionProps {
  data: CategoryBreakdown[] | SeverityBreakdown[];
  type: 'category' | 'severity';
  chartType?: 'pie' | 'doughnut';
  title?: string;
  showLegend?: boolean;
  className?: string;
}

export const IssueDistribution: React.FC<IssueDistributionProps> = ({
  data,
  type,
  chartType = 'doughnut',
  title,
  showLegend = true,
  className,
}) => {
  const labels =
    type === 'category'
      ? (data as CategoryBreakdown[]).map(
          (item) => item.category.charAt(0).toUpperCase() + item.category.slice(1)
        )
      : (data as SeverityBreakdown[]).map(
          (item) => item.severity.charAt(0).toUpperCase() + item.severity.slice(1)
        );

  const values = data.map((item) => item.count);

  const colors =
    type === 'category'
      ? (data as CategoryBreakdown[]).map((item) =>
          getCategoryColor(item.category)
        )
      : (data as SeverityBreakdown[]).map((item) =>
          getSeverityColor(item.severity)
        );

  const chartData = {
    labels,
    datasets: [
      {
        label: type === 'category' ? 'Issues by Category' : 'Issues by Severity',
        data: values,
        backgroundColor: colors,
        borderColor: '#ffffff',
        borderWidth: 2,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: true,
    plugins: {
      legend: {
        display: showLegend,
        position: 'bottom' as const,
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
          label: (context: any) => {
            const label = context.label || '';
            const value = context.parsed || 0;
            const total = context.dataset.data.reduce(
              (acc: number, val: number) => acc + val,
              0
            );
            const percentage = total > 0 ? ((value / total) * 100).toFixed(1) : '0.0';
            return `${label}: ${value} (${percentage}%)`;
          },
        },
      },
    },
  };

  const ChartComponent = chartType === 'pie' ? Pie : Doughnut;

  const totalIssues = values.reduce((sum, val) => sum + val, 0);

  return (
    <div className={className}>
      {title && (
        <h3 className="text-sm font-semibold text-gray-900 mb-4 text-center">
          {title}
        </h3>
      )}

      {totalIssues > 0 ? (
        <div>
          <ChartComponent data={chartData} options={options} />

          <div className="mt-4 space-y-2">
            {data.map((item, index) => {
              const percentage = item.percentage.toFixed(1);
              const isCategoryBreakdown = 'category' in item;
              const key = isCategoryBreakdown
                ? (item as CategoryBreakdown).category
                : (item as SeverityBreakdown).severity;

              return (
                <div
                  key={key}
                  className="flex items-center justify-between text-sm"
                >
                  <div className="flex items-center gap-2">
                    <span
                      className="w-3 h-3 rounded-full flex-shrink-0"
                      style={{ backgroundColor: colors[index] }}
                      aria-hidden="true"
                    />
                    <span className="text-gray-700">{labels[index]}</span>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="font-medium text-gray-900">
                      {item.count}
                    </span>
                    <span className="text-gray-500 text-xs w-12 text-right">
                      {percentage}%
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ) : (
        <div className="text-center py-8 text-gray-500 text-sm">
          No data available
        </div>
      )}
    </div>
  );
};
