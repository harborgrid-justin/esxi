/**
 * Alert Widget Component
 * Real-time alerts display and management
 */

import React, { useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import clsx from 'clsx';
import { formatDistanceToNow } from 'date-fns';
import type { Alert, AlertSeverity } from '../../types';

export interface AlertWidgetProps {
  alerts: Alert[];
  maxDisplay?: number;
  onAcknowledge?: (alertId: string) => void;
  onResolve?: (alertId: string) => void;
  onEscalate?: (alertId: string) => void;
  className?: string;
  compact?: boolean;
}

const SEVERITY_CONFIG: Record<
  AlertSeverity,
  { color: string; bg: string; icon: string; label: string }
> = {
  critical: {
    color: 'text-red-400',
    bg: 'bg-red-500/20',
    icon: '🔴',
    label: 'Critical',
  },
  high: {
    color: 'text-orange-400',
    bg: 'bg-orange-500/20',
    icon: '🟠',
    label: 'High',
  },
  medium: {
    color: 'text-yellow-400',
    bg: 'bg-yellow-500/20',
    icon: '🟡',
    label: 'Medium',
  },
  low: {
    color: 'text-blue-400',
    bg: 'bg-blue-500/20',
    icon: '🔵',
    label: 'Low',
  },
  info: {
    color: 'text-gray-400',
    bg: 'bg-gray-500/20',
    icon: 'ℹ️',
    label: 'Info',
  },
};

export const AlertWidget: React.FC<AlertWidgetProps> = ({
  alerts,
  maxDisplay = 10,
  onAcknowledge,
  onResolve,
  onEscalate,
  className,
  compact = false,
}) => {
  // Group alerts by severity
  const groupedAlerts = useMemo(() => {
    const groups: Record<AlertSeverity, Alert[]> = {
      critical: [],
      high: [],
      medium: [],
      low: [],
      info: [],
    };

    alerts.forEach((alert) => {
      if (alert.status === 'active') {
        groups[alert.severity].push(alert);
      }
    });

    return groups;
  }, [alerts]);

  // Count by severity
  const counts = useMemo(() => {
    return {
      critical: groupedAlerts.critical.length,
      high: groupedAlerts.high.length,
      medium: groupedAlerts.medium.length,
      low: groupedAlerts.low.length,
      info: groupedAlerts.info.length,
      total: alerts.filter((a) => a.status === 'active').length,
    };
  }, [groupedAlerts, alerts]);

  // Display alerts (sorted by severity and time)
  const displayAlerts = useMemo(() => {
    const severityOrder: AlertSeverity[] = ['critical', 'high', 'medium', 'low', 'info'];
    const sorted: Alert[] = [];

    severityOrder.forEach((severity) => {
      const severityAlerts = groupedAlerts[severity].sort(
        (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      );
      sorted.push(...severityAlerts);
    });

    return sorted.slice(0, maxDisplay);
  }, [groupedAlerts, maxDisplay]);

  return (
    <div className={clsx('rounded-xl border border-gray-800 bg-gray-900/50 p-6', className)}>
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Active Alerts</h3>
          <div className="flex items-center gap-2">
            {counts.critical > 0 && (
              <span className="px-2 py-1 rounded-full bg-red-500/20 text-red-400 text-xs font-semibold">
                {counts.critical} Critical
              </span>
            )}
            {counts.high > 0 && (
              <span className="px-2 py-1 rounded-full bg-orange-500/20 text-orange-400 text-xs font-semibold">
                {counts.high} High
              </span>
            )}
          </div>
        </div>

        {/* Severity Summary */}
        {!compact && (
          <div className="grid grid-cols-5 gap-2 mb-4">
            {(['critical', 'high', 'medium', 'low', 'info'] as AlertSeverity[]).map((severity) => {
              const config = SEVERITY_CONFIG[severity];
              const count = counts[severity];

              return (
                <div
                  key={severity}
                  className={clsx(
                    'flex items-center justify-between rounded-lg p-2 border',
                    config.bg,
                    count > 0 ? 'border-current' : 'border-gray-700'
                  )}
                >
                  <span className="text-xs font-medium text-gray-400">{config.label}</span>
                  <span className={clsx('text-sm font-bold', count > 0 ? config.color : 'text-gray-500')}>
                    {count}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Alert List */}
      <div className="space-y-3 max-h-[600px] overflow-y-auto custom-scrollbar">
        <AnimatePresence mode="popLayout">
          {displayAlerts.length === 0 ? (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="text-center py-8 text-gray-500"
            >
              <div className="text-4xl mb-2">✓</div>
              <div className="text-sm">No active alerts</div>
            </motion.div>
          ) : (
            displayAlerts.map((alert) => (
              <AlertItem
                key={alert.id}
                alert={alert}
                onAcknowledge={onAcknowledge}
                onResolve={onResolve}
                onEscalate={onEscalate}
                compact={compact}
              />
            ))
          )}
        </AnimatePresence>
      </div>

      {/* Footer */}
      {displayAlerts.length > 0 && alerts.length > maxDisplay && (
        <div className="mt-4 text-center text-sm text-gray-500">
          Showing {displayAlerts.length} of {counts.total} active alerts
        </div>
      )}
    </div>
  );
};

/**
 * Individual Alert Item
 */
const AlertItem: React.FC<{
  alert: Alert;
  onAcknowledge?: (id: string) => void;
  onResolve?: (id: string) => void;
  onEscalate?: (id: string) => void;
  compact?: boolean;
}> = ({ alert, onAcknowledge, onResolve, onEscalate, compact }) => {
  const config = SEVERITY_CONFIG[alert.severity];

  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      layout
      className={clsx(
        'border rounded-lg p-4 transition-all duration-200',
        config.bg,
        'border-gray-700 hover:border-gray-600',
        alert.status === 'acknowledged' && 'opacity-60'
      )}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-start gap-3 flex-1">
          <span className="text-2xl flex-shrink-0">{config.icon}</span>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <span className={clsx('text-xs font-semibold uppercase', config.color)}>
                {config.label}
              </span>
              <span className="text-xs text-gray-500">
                {formatDistanceToNow(new Date(alert.timestamp), { addSuffix: true })}
              </span>
            </div>
            <h4 className="font-semibold text-white mb-1">{alert.title}</h4>
            {!compact && (
              <p className="text-sm text-gray-400 line-clamp-2">{alert.message}</p>
            )}
          </div>
        </div>
      </div>

      {/* Metadata */}
      {!compact && (
        <div className="flex items-center gap-4 mb-3 text-xs text-gray-500">
          <span>Source: {alert.source}</span>
          {alert.impact && (
            <>
              <span>•</span>
              <span>Impact: {alert.impact.users.toLocaleString()} users</span>
            </>
          )}
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2">
        {alert.status === 'active' && onAcknowledge && (
          <button
            onClick={() => onAcknowledge(alert.id)}
            className="px-3 py-1.5 text-xs font-medium bg-blue-500/20 text-blue-400 rounded hover:bg-blue-500/30 transition-colors"
          >
            Acknowledge
          </button>
        )}
        {onResolve && (
          <button
            onClick={() => onResolve(alert.id)}
            className="px-3 py-1.5 text-xs font-medium bg-green-500/20 text-green-400 rounded hover:bg-green-500/30 transition-colors"
          >
            Resolve
          </button>
        )}
        {alert.severity === 'critical' && onEscalate && (
          <button
            onClick={() => onEscalate(alert.id)}
            className="px-3 py-1.5 text-xs font-medium bg-red-500/20 text-red-400 rounded hover:bg-red-500/30 transition-colors"
          >
            Escalate
          </button>
        )}
      </div>
    </motion.div>
  );
};

export default AlertWidget;
