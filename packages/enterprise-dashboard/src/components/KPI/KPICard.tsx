/**
 * KPI Card Component
 * Displays key performance indicator with trend and sparkline
 */

import React from 'react';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import type { KPIMetric } from '../../types';
import { KPITrend } from './KPITrend';

export interface KPICardProps {
  metric: KPIMetric;
  className?: string;
  onClick?: () => void;
  animated?: boolean;
  showSparkline?: boolean;
  size?: 'small' | 'medium' | 'large';
}

export const KPICard: React.FC<KPICardProps> = ({
  metric,
  className,
  onClick,
  animated = true,
  showSparkline = true,
  size = 'medium',
}) => {
  const {
    label,
    value,
    previousValue,
    unit = '',
    format = 'number',
    trend,
    trendValue,
    sparklineData,
    target,
    status = 'healthy',
    description,
    icon,
    color,
  } = metric;

  // Format value based on type
  const formatValue = (val: number | string): string => {
    if (typeof val === 'string') return val;

    switch (format) {
      case 'currency':
        return new Intl.NumberFormat('en-US', {
          style: 'currency',
          currency: 'USD',
          minimumFractionDigits: 0,
          maximumFractionDigits: 2,
        }).format(val);
      case 'percentage':
        return `${val.toFixed(1)}%`;
      case 'bytes':
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        let size = val;
        let unitIndex = 0;
        while (size >= 1024 && unitIndex < units.length - 1) {
          size /= 1024;
          unitIndex++;
        }
        return `${size.toFixed(1)} ${units[unitIndex]}`;
      case 'duration':
        if (val < 1000) return `${val}ms`;
        if (val < 60000) return `${(val / 1000).toFixed(1)}s`;
        if (val < 3600000) return `${(val / 60000).toFixed(1)}m`;
        return `${(val / 3600000).toFixed(1)}h`;
      default:
        return new Intl.NumberFormat('en-US').format(val);
    }
  };

  // Status colors
  const statusColors = {
    healthy: 'bg-green-500/10 border-green-500/20 text-green-400',
    warning: 'bg-yellow-500/10 border-yellow-500/20 text-yellow-400',
    critical: 'bg-red-500/10 border-red-500/20 text-red-400',
  };

  const sizeClasses = {
    small: 'p-4',
    medium: 'p-6',
    large: 'p-8',
  };

  const cardContent = (
    <div
      className={clsx(
        'relative overflow-hidden rounded-xl border bg-gradient-to-br transition-all duration-300',
        statusColors[status],
        sizeClasses[size],
        onClick && 'cursor-pointer hover:shadow-lg hover:scale-105',
        className
      )}
      onClick={onClick}
      style={color ? { borderColor: `${color}40` } : undefined}
    >
      {/* Background Pattern */}
      <div className="absolute inset-0 opacity-5">
        <div className="absolute inset-0 bg-grid-pattern" />
      </div>

      {/* Content */}
      <div className="relative z-10">
        {/* Header */}
        <div className="flex items-start justify-between mb-4">
          <div className="flex-1">
            <div className="flex items-center gap-2 mb-1">
              {icon && (
                <span className="text-2xl" role="img" aria-label={label}>
                  {icon}
                </span>
              )}
              <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide">
                {label}
              </h3>
            </div>
            {description && (
              <p className="text-xs text-gray-500 mt-1">{description}</p>
            )}
          </div>

          {trend && trendValue !== undefined && (
            <KPITrend direction={trend} value={trendValue} />
          )}
        </div>

        {/* Value */}
        <div className="mb-4">
          <div className="flex items-baseline gap-2">
            <span
              className={clsx(
                'font-bold',
                size === 'small' && 'text-2xl',
                size === 'medium' && 'text-3xl',
                size === 'large' && 'text-4xl'
              )}
              style={color ? { color } : undefined}
            >
              {formatValue(value)}
            </span>
            {unit && (
              <span className="text-sm text-gray-500 font-medium">{unit}</span>
            )}
          </div>

          {previousValue !== undefined && (
            <div className="text-xs text-gray-500 mt-1">
              Previous: {formatValue(previousValue)}
            </div>
          )}
        </div>

        {/* Target Progress */}
        {target !== undefined && typeof value === 'number' && (
          <div className="mb-4">
            <div className="flex items-center justify-between text-xs text-gray-500 mb-1">
              <span>Progress to target</span>
              <span>{((value / target) * 100).toFixed(0)}%</span>
            </div>
            <div className="h-1.5 bg-gray-800 rounded-full overflow-hidden">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: `${Math.min((value / target) * 100, 100)}%` }}
                transition={{ duration: 1, ease: 'easeOut' }}
                className="h-full bg-gradient-to-r from-blue-500 to-cyan-500"
                style={color ? { background: color } : undefined}
              />
            </div>
          </div>
        )}

        {/* Sparkline */}
        {showSparkline && sparklineData && sparklineData.length > 0 && (
          <div className="mt-4">
            <svg
              width="100%"
              height="40"
              className="opacity-60"
              preserveAspectRatio="none"
            >
              <Sparkline data={sparklineData} color={color || '#3b82f6'} />
            </svg>
          </div>
        )}
      </div>
    </div>
  );

  if (animated) {
    return (
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        {cardContent}
      </motion.div>
    );
  }

  return cardContent;
};

/**
 * Sparkline Component
 */
const Sparkline: React.FC<{ data: number[]; color: string }> = ({ data, color }) => {
  if (data.length < 2) return null;

  const max = Math.max(...data);
  const min = Math.min(...data);
  const range = max - min || 1;

  const points = data.map((value, index) => {
    const x = (index / (data.length - 1)) * 100;
    const y = 40 - ((value - min) / range) * 35;
    return `${x},${y}`;
  }).join(' ');

  return (
    <>
      {/* Area fill */}
      <polygon
        points={`0,40 ${points} 100,40`}
        fill={color}
        fillOpacity="0.2"
      />
      {/* Line */}
      <polyline
        points={points}
        fill="none"
        stroke={color}
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </>
  );
};

export default KPICard;
