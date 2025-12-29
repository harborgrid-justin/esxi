/**
 * Score Card Widget
 * Displays KPI metrics with accessible design
 */

import React from 'react';
import clsx from 'clsx';
import { getScoreColor } from '../../utils/calculations';

export interface ScoreCardProps {
  title: string;
  value: number | string;
  subtitle?: string;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  icon?: React.ReactNode;
  variant?: 'default' | 'primary' | 'success' | 'warning' | 'danger';
  isPercentage?: boolean;
  className?: string;
}

export const ScoreCard: React.FC<ScoreCardProps> = ({
  title,
  value,
  subtitle,
  trend,
  trendValue,
  icon,
  variant = 'default',
  isPercentage = false,
  className,
}) => {
  const displayValue =
    typeof value === 'number' && isPercentage ? `${value}%` : value;

  const scoreColor =
    typeof value === 'number' && isPercentage
      ? getScoreColor(value)
      : undefined;

  const trendIcon = trend === 'up' ? '↑' : trend === 'down' ? '↓' : '→';
  const trendColor =
    trend === 'up'
      ? 'text-green-600'
      : trend === 'down'
      ? 'text-red-600'
      : 'text-gray-600';

  const variantStyles = {
    default: 'bg-white border-gray-200',
    primary: 'bg-blue-50 border-blue-200',
    success: 'bg-green-50 border-green-200',
    warning: 'bg-amber-50 border-amber-200',
    danger: 'bg-red-50 border-red-200',
  };

  return (
    <div
      className={clsx(
        'border rounded-lg p-6 shadow-sm',
        variantStyles[variant],
        className
      )}
      role="article"
      aria-labelledby="score-card-title"
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <h3
            id="score-card-title"
            className="text-sm font-medium text-gray-600 mb-1"
          >
            {title}
          </h3>
          <div className="flex items-baseline gap-2">
            <p
              className="text-3xl font-bold"
              style={scoreColor ? { color: scoreColor } : undefined}
            >
              {displayValue}
            </p>
            {trendValue && (
              <span
                className={clsx('text-sm font-medium', trendColor)}
                aria-label={`Trend: ${trend}`}
              >
                <span aria-hidden="true">{trendIcon}</span> {trendValue}
              </span>
            )}
          </div>
          {subtitle && (
            <p className="mt-1 text-sm text-gray-500">{subtitle}</p>
          )}
        </div>
        {icon && (
          <div
            className="ml-4 flex-shrink-0 text-gray-400"
            aria-hidden="true"
          >
            {icon}
          </div>
        )}
      </div>
    </div>
  );
};
