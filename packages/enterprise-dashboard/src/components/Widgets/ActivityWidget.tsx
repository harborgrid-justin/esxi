/**
 * Activity Widget Component
 * Real-time activity feed display
 */

import React, { useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import clsx from 'clsx';
import { formatDistanceToNow, format } from 'date-fns';
import type { ActivityLogEntry } from '../../types';

export interface ActivityWidgetProps {
  activities: ActivityLogEntry[];
  maxDisplay?: number;
  filterTypes?: ActivityLogEntry['type'][];
  className?: string;
  compact?: boolean;
  showFilters?: boolean;
}

const ACTIVITY_TYPE_CONFIG: Record<
  ActivityLogEntry['type'],
  { color: string; bg: string; icon: string; label: string }
> = {
  user: {
    color: 'text-blue-400',
    bg: 'bg-blue-500/10',
    icon: '👤',
    label: 'User',
  },
  system: {
    color: 'text-green-400',
    bg: 'bg-green-500/10',
    icon: '⚙️',
    label: 'System',
  },
  security: {
    color: 'text-red-400',
    bg: 'bg-red-500/10',
    icon: '🔒',
    label: 'Security',
  },
  deployment: {
    color: 'text-purple-400',
    bg: 'bg-purple-500/10',
    icon: '🚀',
    label: 'Deployment',
  },
  configuration: {
    color: 'text-amber-400',
    bg: 'bg-amber-500/10',
    icon: '🔧',
    label: 'Config',
  },
};

const SEVERITY_COLORS = {
  info: 'text-gray-400',
  warning: 'text-yellow-400',
  error: 'text-red-400',
};

export const ActivityWidget: React.FC<ActivityWidgetProps> = ({
  activities,
  maxDisplay = 20,
  filterTypes,
  className,
  compact = false,
  showFilters = true,
}) => {
  const [selectedTypes, setSelectedTypes] = React.useState<Set<ActivityLogEntry['type']>>(
    new Set(filterTypes || ['user', 'system', 'security', 'deployment', 'configuration'])
  );

  // Filter and sort activities
  const filteredActivities = useMemo(() => {
    return activities
      .filter((activity) => selectedTypes.has(activity.type))
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, maxDisplay);
  }, [activities, selectedTypes, maxDisplay]);

  // Count by type
  const typeCounts = useMemo(() => {
    const counts: Record<ActivityLogEntry['type'], number> = {
      user: 0,
      system: 0,
      security: 0,
      deployment: 0,
      configuration: 0,
    };

    activities.forEach((activity) => {
      counts[activity.type] += 1;
    });

    return counts;
  }, [activities]);

  const toggleType = (type: ActivityLogEntry['type']) => {
    setSelectedTypes((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(type)) {
        newSet.delete(type);
      } else {
        newSet.add(type);
      }
      return newSet;
    });
  };

  return (
    <div className={clsx('rounded-xl border border-gray-800 bg-gray-900/50 p-6', className)}>
      {/* Header */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Activity Feed</h3>
          <span className="text-sm text-gray-500">
            {filteredActivities.length} of {activities.length}
          </span>
        </div>

        {/* Type Filters */}
        {showFilters && (
          <div className="flex flex-wrap gap-2">
            {(Object.keys(ACTIVITY_TYPE_CONFIG) as ActivityLogEntry['type'][]).map((type) => {
              const config = ACTIVITY_TYPE_CONFIG[type];
              const count = typeCounts[type];
              const isSelected = selectedTypes.has(type);

              return (
                <button
                  key={type}
                  onClick={() => toggleType(type)}
                  className={clsx(
                    'flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs font-medium transition-all',
                    isSelected
                      ? `${config.bg} ${config.color} border border-current`
                      : 'bg-gray-800/50 text-gray-500 border border-transparent hover:bg-gray-800'
                  )}
                >
                  <span>{config.icon}</span>
                  <span>{config.label}</span>
                  <span className="ml-1 opacity-60">({count})</span>
                </button>
              );
            })}
          </div>
        )}
      </div>

      {/* Activity List */}
      <div className="space-y-2 max-h-[600px] overflow-y-auto custom-scrollbar">
        <AnimatePresence mode="popLayout">
          {filteredActivities.length === 0 ? (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="text-center py-8 text-gray-500"
            >
              <div className="text-4xl mb-2">📋</div>
              <div className="text-sm">No activities to display</div>
            </motion.div>
          ) : (
            filteredActivities.map((activity) => (
              <ActivityItem
                key={activity.id}
                activity={activity}
                compact={compact}
              />
            ))
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};

/**
 * Individual Activity Item
 */
const ActivityItem: React.FC<{
  activity: ActivityLogEntry;
  compact?: boolean;
}> = ({ activity, compact }) => {
  const typeConfig = ACTIVITY_TYPE_CONFIG[activity.type];
  const severityColor = activity.severity ? SEVERITY_COLORS[activity.severity] : 'text-gray-400';

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 10 }}
      layout
      className={clsx(
        'border border-gray-700 rounded-lg p-3 hover:bg-gray-800/30 transition-all',
        typeConfig.bg
      )}
    >
      <div className="flex items-start gap-3">
        {/* Icon */}
        <span className="text-xl flex-shrink-0">{typeConfig.icon}</span>

        {/* Content */}
        <div className="flex-1 min-w-0">
          {/* Header */}
          <div className="flex items-start justify-between gap-2 mb-1">
            <div className="flex items-center gap-2 flex-wrap">
              <span className={clsx('text-xs font-semibold', typeConfig.color)}>
                {typeConfig.label}
              </span>
              <span className="text-xs text-gray-500">•</span>
              <span className="text-xs text-gray-500">
                {formatDistanceToNow(new Date(activity.timestamp), { addSuffix: true })}
              </span>
              {activity.severity && activity.severity !== 'info' && (
                <>
                  <span className="text-xs text-gray-500">•</span>
                  <span className={clsx('text-xs font-semibold uppercase', severityColor)}>
                    {activity.severity}
                  </span>
                </>
              )}
            </div>
            {!compact && (
              <span className="text-xs text-gray-600 font-mono">
                {format(new Date(activity.timestamp), 'HH:mm:ss')}
              </span>
            )}
          </div>

          {/* Action & Actor */}
          <div className="mb-1">
            <span className="text-sm text-gray-300">
              <span className="font-semibold text-white">{activity.actor.name}</span>
              <span className="text-gray-500 mx-1">•</span>
              <span>{activity.action}</span>
            </span>
          </div>

          {/* Description */}
          <p className={clsx('text-sm text-gray-400', compact && 'line-clamp-1')}>
            {activity.description}
          </p>

          {/* Resource */}
          {!compact && activity.resource && (
            <div className="mt-2 flex items-center gap-2 text-xs text-gray-500">
              <span className="px-2 py-0.5 bg-gray-800/50 rounded">
                {activity.resource.type}
              </span>
              <span>→</span>
              <span className="font-mono">{activity.resource.name}</span>
            </div>
          )}

          {/* Location & IP */}
          {!compact && (activity.location || activity.ipAddress) && (
            <div className="mt-2 flex items-center gap-3 text-xs text-gray-600">
              {activity.location && (
                <span className="flex items-center gap-1">
                  📍 {activity.location}
                </span>
              )}
              {activity.ipAddress && (
                <span className="font-mono">{activity.ipAddress}</span>
              )}
            </div>
          )}
        </div>
      </div>
    </motion.div>
  );
};

/**
 * Compact Activity Timeline
 */
export const ActivityTimeline: React.FC<{
  activities: ActivityLogEntry[];
  maxDisplay?: number;
  className?: string;
}> = ({ activities, maxDisplay = 10, className }) => {
  const recentActivities = useMemo(() => {
    return activities
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, maxDisplay);
  }, [activities, maxDisplay]);

  return (
    <div className={clsx('space-y-0', className)}>
      {recentActivities.map((activity, index) => {
        const typeConfig = ACTIVITY_TYPE_CONFIG[activity.type];
        const isLast = index === recentActivities.length - 1;

        return (
          <div key={activity.id} className="flex gap-3">
            {/* Timeline */}
            <div className="flex flex-col items-center">
              <div className={clsx('w-8 h-8 rounded-full flex items-center justify-center', typeConfig.bg)}>
                <span className="text-sm">{typeConfig.icon}</span>
              </div>
              {!isLast && <div className="w-0.5 flex-1 bg-gray-700 my-1" />}
            </div>

            {/* Content */}
            <div className="flex-1 pb-4">
              <div className="text-xs text-gray-500 mb-1">
                {formatDistanceToNow(new Date(activity.timestamp), { addSuffix: true })}
              </div>
              <div className="text-sm text-white font-medium mb-1">
                {activity.action}
              </div>
              <div className="text-xs text-gray-400">
                by {activity.actor.name}
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
};

export default ActivityWidget;
