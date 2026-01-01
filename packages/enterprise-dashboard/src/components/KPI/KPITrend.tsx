/**
 * KPI Trend Component
 * Displays trend indicator with percentage change
 */

import React from 'react';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import type { TrendDirection } from '../../types';

export interface KPITrendProps {
  direction: TrendDirection;
  value: number;
  className?: string;
  showValue?: boolean;
  size?: 'small' | 'medium' | 'large';
  animated?: boolean;
}

export const KPITrend: React.FC<KPITrendProps> = ({
  direction,
  value,
  className,
  showValue = true,
  size = 'medium',
  animated = true,
}) => {
  const isPositive = direction === 'up';
  const isNegative = direction === 'down';
  const isStable = direction === 'stable';

  const sizeClasses = {
    small: 'text-xs px-2 py-1',
    medium: 'text-sm px-2.5 py-1.5',
    large: 'text-base px-3 py-2',
  };

  const iconSizes = {
    small: 'w-3 h-3',
    medium: 'w-4 h-4',
    large: 'w-5 h-5',
  };

  const formatValue = (val: number): string => {
    const absValue = Math.abs(val);
    if (absValue >= 1000) {
      return `${(absValue / 1000).toFixed(1)}k`;
    }
    return absValue.toFixed(1);
  };

  const trendContent = (
    <div
      className={clsx(
        'inline-flex items-center gap-1.5 rounded-full font-semibold transition-all duration-300',
        sizeClasses[size],
        {
          'bg-green-500/20 text-green-400': isPositive,
          'bg-red-500/20 text-red-400': isNegative,
          'bg-gray-500/20 text-gray-400': isStable,
        },
        className
      )}
    >
      {/* Trend Icon */}
      <span className={clsx('flex items-center justify-center', iconSizes[size])}>
        {isPositive && (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="3"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="18 9 12 3 6 9" />
            <line x1="12" y1="3" x2="12" y2="21" />
          </svg>
        )}
        {isNegative && (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="3"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="6 15 12 21 18 15" />
            <line x1="12" y1="21" x2="12" y2="3" />
          </svg>
        )}
        {isStable && (
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="3"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        )}
      </span>

      {/* Trend Value */}
      {showValue && (
        <span className="tabular-nums">
          {!isStable && (isPositive ? '+' : '-')}
          {formatValue(value)}%
        </span>
      )}
    </div>
  );

  if (animated) {
    return (
      <motion.div
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.3, type: 'spring', stiffness: 200 }}
      >
        {trendContent}
      </motion.div>
    );
  }

  return trendContent;
};

/**
 * Compact Trend Indicator (Icon Only)
 */
export const CompactTrend: React.FC<{
  direction: TrendDirection;
  className?: string;
}> = ({ direction, className }) => {
  return (
    <span
      className={clsx(
        'inline-flex items-center justify-center w-5 h-5',
        {
          'text-green-400': direction === 'up',
          'text-red-400': direction === 'down',
          'text-gray-400': direction === 'stable',
        },
        className
      )}
    >
      {direction === 'up' && '↑'}
      {direction === 'down' && '↓'}
      {direction === 'stable' && '→'}
    </span>
  );
};

/**
 * Trend Badge with Color Coding
 */
export const TrendBadge: React.FC<{
  direction: TrendDirection;
  value: number;
  label?: string;
  className?: string;
}> = ({ direction, value, label, className }) => {
  const isGood = direction === 'up';
  const isBad = direction === 'down';

  return (
    <div
      className={clsx(
        'inline-flex items-center gap-2 px-3 py-2 rounded-lg border',
        {
          'bg-green-500/10 border-green-500/30 text-green-400': isGood,
          'bg-red-500/10 border-red-500/30 text-red-400': isBad,
          'bg-gray-500/10 border-gray-500/30 text-gray-400': direction === 'stable',
        },
        className
      )}
    >
      <KPITrend direction={direction} value={value} size="small" animated={false} />
      {label && <span className="text-sm font-medium">{label}</span>}
    </div>
  );
};

export default KPITrend;
