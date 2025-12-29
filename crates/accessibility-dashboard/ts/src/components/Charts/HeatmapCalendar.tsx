/**
 * Heatmap Calendar Chart
 * GitHub-style activity heatmap for compliance scores
 */

import React, { useMemo } from 'react';
import {
  format,
  startOfWeek,
  endOfWeek,
  eachDayOfInterval,
  isSameDay,
  subWeeks,
} from 'date-fns';
import type { HeatmapDataPoint } from '../../types';
import { getScoreColor } from '../../utils/calculations';

export interface HeatmapCalendarProps {
  data: HeatmapDataPoint[];
  weeks?: number;
  title?: string;
  className?: string;
}

export const HeatmapCalendar: React.FC<HeatmapCalendarProps> = ({
  data,
  weeks = 12,
  title = 'Compliance Activity',
  className,
}) => {
  const heatmapData = useMemo(() => {
    const endDate = new Date();
    const startDate = subWeeks(endDate, weeks);

    const weekStart = startOfWeek(startDate, { weekStartsOn: 0 });
    const weekEnd = endOfWeek(endDate, { weekStartsOn: 0 });

    const allDays = eachDayOfInterval({ start: weekStart, end: weekEnd });

    return allDays.map((day) => {
      const dataPoint = data.find((point) => isSameDay(point.date, day));

      return {
        date: day,
        value: dataPoint?.value || 0,
        issueCount: dataPoint?.issueCount || 0,
      };
    });
  }, [data, weeks]);

  // Group days by week
  const weekGroups = useMemo(() => {
    const groups: HeatmapDataPoint[][] = [];
    let currentWeek: HeatmapDataPoint[] = [];

    heatmapData.forEach((day, index) => {
      currentWeek.push(day);

      if ((index + 1) % 7 === 0) {
        groups.push(currentWeek);
        currentWeek = [];
      }
    });

    if (currentWeek.length > 0) {
      groups.push(currentWeek);
    }

    return groups;
  }, [heatmapData]);

  const getIntensityClass = (value: number): string => {
    if (value === 0) return 'bg-gray-100';
    if (value >= 90) return 'bg-green-500';
    if (value >= 70) return 'bg-green-300';
    if (value >= 50) return 'bg-amber-300';
    return 'bg-red-300';
  };

  const dayLabels = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

  return (
    <div className={className}>
      {title && (
        <h3 className="text-sm font-semibold text-gray-900 mb-4">
          {title}
        </h3>
      )}

      <div className="overflow-x-auto">
        <div className="inline-flex flex-col gap-1">
          {/* Day labels */}
          <div className="flex gap-1">
            <div className="w-8" />
            {weekGroups[0] && weekGroups[0].map((day, dayIndex) => (
              <div
                key={dayIndex}
                className="text-xs text-gray-500 text-center"
                style={{ width: 14 }}
              >
                {dayIndex % 2 === 0 ? dayLabels[dayIndex].charAt(0) : ''}
              </div>
            ))}
          </div>

          {/* Heatmap grid */}
          <div className="flex gap-1">
            {/* Week labels */}
            <div className="flex flex-col gap-1 justify-around text-xs text-gray-500">
              {weekGroups.map((_, weekIndex) =>
                weekIndex % 4 === 0 ? (
                  <div key={weekIndex} className="h-3.5 flex items-center">
                    {format(weekGroups[weekIndex][0].date, 'MMM')}
                  </div>
                ) : (
                  <div key={weekIndex} className="h-3.5" />
                )
              )}
            </div>

            {/* Days grid */}
            {weekGroups.map((week, weekIndex) => (
              <div key={weekIndex} className="flex flex-col gap-1">
                {week.map((day, dayIndex) => {
                  const intensityClass = getIntensityClass(day.value);
                  const tooltipId = `day-${weekIndex}-${dayIndex}`;

                  return (
                    <div
                      key={dayIndex}
                      className={`w-3.5 h-3.5 rounded-sm border border-gray-200 hover:ring-2 hover:ring-blue-500 transition-all cursor-pointer ${intensityClass}`}
                      title={`${format(day.date, 'MMM d, yyyy')}: ${
                        day.value
                      }% compliance (${day.issueCount} issues)`}
                      role="img"
                      aria-label={`${format(day.date, 'MMM d, yyyy')}: ${
                        day.value
                      }% compliance score with ${day.issueCount} issues`}
                      tabIndex={0}
                    />
                  );
                })}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Legend */}
      <div className="mt-4 flex items-center justify-end gap-2 text-xs text-gray-600">
        <span>Less</span>
        <div className="flex gap-1">
          <div
            className="w-3.5 h-3.5 rounded-sm bg-gray-100 border border-gray-200"
            aria-hidden="true"
          />
          <div
            className="w-3.5 h-3.5 rounded-sm bg-red-300 border border-gray-200"
            aria-hidden="true"
          />
          <div
            className="w-3.5 h-3.5 rounded-sm bg-amber-300 border border-gray-200"
            aria-hidden="true"
          />
          <div
            className="w-3.5 h-3.5 rounded-sm bg-green-300 border border-gray-200"
            aria-hidden="true"
          />
          <div
            className="w-3.5 h-3.5 rounded-sm bg-green-500 border border-gray-200"
            aria-hidden="true"
          />
        </div>
        <span>More</span>
      </div>

      <div className="mt-2 text-xs text-gray-500 text-center">
        Showing compliance scores for the last {weeks} weeks
      </div>
    </div>
  );
};
