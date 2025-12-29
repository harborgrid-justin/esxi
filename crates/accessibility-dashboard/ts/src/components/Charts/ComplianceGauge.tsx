/**
 * Compliance Gauge Chart
 * Circular gauge displaying compliance percentage
 */

import React, { useEffect, useRef } from 'react';
import { Chart as ChartJS, ArcElement, Tooltip, Legend } from 'chart.js';
import { Doughnut } from 'react-chartjs-2';
import { getScoreColor } from '../../utils/calculations';

ChartJS.register(ArcElement, Tooltip, Legend);

export interface ComplianceGaugeProps {
  score: number; // 0-100
  title?: string;
  subtitle?: string;
  size?: number;
  showLabel?: boolean;
  className?: string;
}

export const ComplianceGauge: React.FC<ComplianceGaugeProps> = ({
  score,
  title = 'Compliance Score',
  subtitle,
  size = 200,
  showLabel = true,
  className,
}) => {
  const chartRef = useRef(null);
  const scoreColor = getScoreColor(score);

  const data = {
    labels: ['Compliance', 'Remaining'],
    datasets: [
      {
        data: [score, 100 - score],
        backgroundColor: [scoreColor, '#e5e7eb'],
        borderWidth: 0,
        circumference: 180,
        rotation: 270,
      },
    ],
  };

  const options = {
    responsive: true,
    maintainAspectRatio: true,
    cutout: '75%',
    plugins: {
      legend: {
        display: false,
      },
      tooltip: {
        enabled: true,
        callbacks: {
          label: (context: any) => {
            const label = context.label || '';
            const value = context.parsed || 0;
            return `${label}: ${value.toFixed(1)}%`;
          },
        },
      },
    },
  };

  return (
    <div className={className} role="img" aria-label={`${title}: ${score}%`}>
      {title && (
        <h3 className="text-sm font-semibold text-gray-900 mb-2 text-center">
          {title}
        </h3>
      )}

      <div className="relative" style={{ width: size, margin: '0 auto' }}>
        <Doughnut ref={chartRef} data={data} options={options} />

        {showLabel && (
          <div
            className="absolute inset-0 flex flex-col items-center justify-center"
            style={{ top: '20%' }}
          >
            <div
              className="text-4xl font-bold"
              style={{ color: scoreColor }}
              aria-hidden="true"
            >
              {score}%
            </div>
            {subtitle && (
              <div className="text-xs text-gray-500 mt-1">{subtitle}</div>
            )}
          </div>
        )}
      </div>

      <div className="mt-4 text-center">
        <div className="inline-flex items-center gap-2 text-sm">
          <span
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: scoreColor }}
            aria-hidden="true"
          />
          <span className="text-gray-700">
            {score >= 90 && 'Excellent'}
            {score >= 70 && score < 90 && 'Good'}
            {score >= 50 && score < 70 && 'Fair'}
            {score < 50 && 'Needs Improvement'}
          </span>
        </div>
      </div>
    </div>
  );
};
