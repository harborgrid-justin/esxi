/**
 * Quota Widget Component
 * Usage quotas and limits visualization
 */

import React, { useMemo } from 'react';
import { motion } from 'framer-motion';
import clsx from 'clsx';
import { format, formatDistanceToNow } from 'date-fns';
import type { QuotaUsage } from '../../types';

export interface QuotaWidgetProps {
  quotas: QuotaUsage[];
  showAll?: boolean;
  warningThreshold?: number;
  criticalThreshold?: number;
  className?: string;
  compact?: boolean;
}

const CATEGORY_CONFIG: Record<
  QuotaUsage['category'],
  { color: string; icon: string; label: string }
> = {
  compute: { color: '#3b82f6', icon: '💻', label: 'Compute' },
  storage: { color: '#10b981', icon: '💾', label: 'Storage' },
  network: { color: '#8b5cf6', icon: '🌐', label: 'Network' },
  api: { color: '#f59e0b', icon: '🔌', label: 'API' },
  users: { color: '#ec4899', icon: '👥', label: 'Users' },
  custom: { color: '#6b7280', icon: '⚡', label: 'Custom' },
};

export const QuotaWidget: React.FC<QuotaWidgetProps> = ({
  quotas,
  showAll = false,
  warningThreshold = 75,
  criticalThreshold = 90,
  className,
  compact = false,
}) => {
  // Sort and filter quotas
  const displayQuotas = useMemo(() => {
    const sorted = [...quotas].sort((a, b) => b.percentage - a.percentage);
    if (showAll) return sorted;
    // Show only quotas above warning threshold
    return sorted.filter((q) => q.percentage >= warningThreshold);
  }, [quotas, showAll, warningThreshold]);

  // Count by status
  const counts = useMemo(() => {
    return {
      critical: quotas.filter((q) => q.percentage >= criticalThreshold).length,
      warning: quotas.filter(
        (q) => q.percentage >= warningThreshold && q.percentage < criticalThreshold
      ).length,
      normal: quotas.filter((q) => q.percentage < warningThreshold).length,
      total: quotas.length,
    };
  }, [quotas, warningThreshold, criticalThreshold]);

  // Get status color
  const getStatusColor = (percentage: number): string => {
    if (percentage >= criticalThreshold) return 'text-red-400';
    if (percentage >= warningThreshold) return 'text-yellow-400';
    return 'text-green-400';
  };

  const getStatusBg = (percentage: number): string => {
    if (percentage >= criticalThreshold) return 'bg-red-500/20';
    if (percentage >= warningThreshold) return 'bg-yellow-500/20';
    return 'bg-green-500/20';
  };

  return (
    <div className={clsx('rounded-xl border border-gray-800 bg-gray-900/50 p-6', className)}>
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Usage Quotas</h3>
          <div className="flex items-center gap-2">
            {counts.critical > 0 && (
              <span className="px-2 py-1 rounded-full bg-red-500/20 text-red-400 text-xs font-semibold">
                {counts.critical} Critical
              </span>
            )}
            {counts.warning > 0 && (
              <span className="px-2 py-1 rounded-full bg-yellow-500/20 text-yellow-400 text-xs font-semibold">
                {counts.warning} Warning
              </span>
            )}
          </div>
        </div>

        {/* Summary */}
        {!compact && (
          <div className="grid grid-cols-3 gap-3 mb-4">
            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Critical</div>
              <div className="text-2xl font-bold text-red-400">{counts.critical}</div>
            </div>
            <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Warning</div>
              <div className="text-2xl font-bold text-yellow-400">{counts.warning}</div>
            </div>
            <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-3">
              <div className="text-xs text-gray-400 mb-1">Normal</div>
              <div className="text-2xl font-bold text-green-400">{counts.normal}</div>
            </div>
          </div>
        )}
      </div>

      {/* Quota List */}
      <div className="space-y-4 max-h-[600px] overflow-y-auto custom-scrollbar">
        {displayQuotas.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <div className="text-4xl mb-2">✓</div>
            <div className="text-sm">All quotas are healthy</div>
          </div>
        ) : (
          displayQuotas.map((quota) => (
            <QuotaItem
              key={quota.id}
              quota={quota}
              warningThreshold={warningThreshold}
              criticalThreshold={criticalThreshold}
              compact={compact}
            />
          ))
        )}
      </div>
    </div>
  );
};

/**
 * Individual Quota Item
 */
const QuotaItem: React.FC<{
  quota: QuotaUsage;
  warningThreshold: number;
  criticalThreshold: number;
  compact?: boolean;
}> = ({ quota, warningThreshold, criticalThreshold, compact }) => {
  const config = CATEGORY_CONFIG[quota.category];
  const isCritical = quota.percentage >= criticalThreshold;
  const isWarning = quota.percentage >= warningThreshold && !isCritical;

  const getProgressColor = (): string => {
    if (isCritical) return '#ef4444';
    if (isWarning) return '#f59e0b';
    return '#10b981';
  };

  const formatValue = (value: number): string => {
    if (quota.unit === 'bytes') {
      const units = ['B', 'KB', 'MB', 'GB', 'TB'];
      let size = value;
      let unitIndex = 0;
      while (size >= 1024 && unitIndex < units.length - 1) {
        size /= 1024;
        unitIndex++;
      }
      return `${size.toFixed(2)} ${units[unitIndex]}`;
    }
    return `${value.toLocaleString()} ${quota.unit}`;
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className={clsx(
        'border rounded-lg p-4',
        isCritical && 'border-red-500/30 bg-red-500/5',
        isWarning && 'border-yellow-500/30 bg-yellow-500/5',
        !isCritical && !isWarning && 'border-gray-700 bg-gray-800/20'
      )}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{config.icon}</span>
          <div>
            <div className="flex items-center gap-2">
              <h4 className="font-semibold text-white">{quota.name}</h4>
              <span
                className="text-xs px-2 py-0.5 rounded-full"
                style={{ backgroundColor: `${config.color}20`, color: config.color }}
              >
                {config.label}
              </span>
            </div>
            {!compact && quota.trend && (
              <div className="text-xs text-gray-500 mt-1 flex items-center gap-1">
                {quota.trend === 'up' && '📈 Increasing'}
                {quota.trend === 'down' && '📉 Decreasing'}
                {quota.trend === 'stable' && '➡️ Stable'}
              </div>
            )}
          </div>
        </div>
        <div className="text-right">
          <div
            className={clsx(
              'text-2xl font-bold',
              isCritical && 'text-red-400',
              isWarning && 'text-yellow-400',
              !isCritical && !isWarning && 'text-green-400'
            )}
          >
            {quota.percentage.toFixed(1)}%
          </div>
        </div>
      </div>

      {/* Progress Bar */}
      <div className="mb-3">
        <div className="h-3 bg-gray-800 rounded-full overflow-hidden">
          <motion.div
            initial={{ width: 0 }}
            animate={{ width: `${Math.min(quota.percentage, 100)}%` }}
            transition={{ duration: 1, ease: 'easeOut' }}
            className="h-full relative"
            style={{ backgroundColor: getProgressColor() }}
          >
            {/* Animated shimmer effect */}
            <div
              className="absolute inset-0 opacity-30"
              style={{
                background: 'linear-gradient(90deg, transparent, white, transparent)',
                animation: 'shimmer 2s infinite',
              }}
            />
          </motion.div>
        </div>
      </div>

      {/* Usage Details */}
      <div className="flex items-center justify-between text-sm mb-2">
        <span className="text-gray-400">
          {formatValue(quota.current)} / {formatValue(quota.limit)}
        </span>
        {quota.overage && quota.overage > 0 && (
          <span className="text-red-400 font-semibold">
            Overage: {formatValue(quota.overage)}
          </span>
        )}
      </div>

      {/* Warnings & Forecast */}
      {!compact && (
        <>
          {quota.warnings && quota.warnings.length > 0 && (
            <div className="mb-2 space-y-1">
              {quota.warnings.map((warning, index) => (
                <div key={index} className="text-xs text-yellow-400 flex items-start gap-2">
                  <span>⚠️</span>
                  <span>{warning.message}</span>
                </div>
              ))}
            </div>
          )}

          {quota.forecast && (
            <div className="text-xs text-gray-500 flex items-center justify-between pt-2 border-t border-gray-700">
              <span>Estimated exhaustion:</span>
              <span className="font-semibold text-orange-400">
                {formatDistanceToNow(new Date(quota.forecast.exhaustionDate), { addSuffix: true })}
              </span>
            </div>
          )}

          {quota.resetDate && (
            <div className="text-xs text-gray-500 flex items-center justify-between pt-2 border-t border-gray-700 mt-2">
              <span>Resets:</span>
              <span className="font-semibold">
                {formatDistanceToNow(new Date(quota.resetDate), { addSuffix: true })}
              </span>
            </div>
          )}

          {quota.overageCost && quota.overageCost > 0 && (
            <div className="text-xs flex items-center justify-between pt-2 border-t border-gray-700 mt-2">
              <span className="text-gray-500">Overage cost:</span>
              <span className="font-semibold text-red-400">
                ${quota.overageCost.toFixed(2)}
              </span>
            </div>
          )}
        </>
      )}
    </motion.div>
  );
};

/**
 * Compact Quota Summary
 */
export const QuotaSummary: React.FC<{
  quotas: QuotaUsage[];
  className?: string;
}> = ({ quotas, className }) => {
  const categories = useMemo(() => {
    const grouped: Record<QuotaUsage['category'], QuotaUsage[]> = {
      compute: [],
      storage: [],
      network: [],
      api: [],
      users: [],
      custom: [],
    };

    quotas.forEach((quota) => {
      grouped[quota.category].push(quota);
    });

    return Object.entries(grouped)
      .filter(([_, items]) => items.length > 0)
      .map(([category, items]) => {
        const avgPercentage = items.reduce((sum, q) => sum + q.percentage, 0) / items.length;
        return {
          category: category as QuotaUsage['category'],
          count: items.length,
          avgPercentage,
          maxPercentage: Math.max(...items.map((q) => q.percentage)),
        };
      });
  }, [quotas]);

  return (
    <div className={clsx('grid grid-cols-2 gap-3', className)}>
      {categories.map(({ category, count, avgPercentage, maxPercentage }) => {
        const config = CATEGORY_CONFIG[category];
        return (
          <div key={category} className="bg-gray-800/50 rounded-lg p-3 border border-gray-700">
            <div className="flex items-center gap-2 mb-2">
              <span className="text-xl">{config.icon}</span>
              <div className="flex-1">
                <div className="text-xs text-gray-400">{config.label}</div>
                <div className="text-sm font-semibold text-white">{count} quotas</div>
              </div>
            </div>
            <div className="h-2 bg-gray-700 rounded-full overflow-hidden mb-1">
              <div
                className="h-full"
                style={{
                  width: `${Math.min(maxPercentage, 100)}%`,
                  backgroundColor: config.color,
                }}
              />
            </div>
            <div className="text-xs text-gray-500">
              Avg: {avgPercentage.toFixed(0)}% • Max: {maxPercentage.toFixed(0)}%
            </div>
          </div>
        );
      })}
    </div>
  );
};

export default QuotaWidget;
