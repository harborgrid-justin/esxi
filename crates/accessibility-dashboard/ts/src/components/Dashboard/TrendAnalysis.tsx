/**
 * Trend Analysis Component
 * Displays historical compliance trends and patterns
 */

import React from 'react';
import { TrendLineChart } from '../Charts/TrendLineChart';
import { HeatmapCalendar } from '../Charts/HeatmapCalendar';
import type { TrendDataPoint, HeatmapDataPoint } from '../../types';

export interface TrendAnalysisProps {
  trendData: TrendDataPoint[];
  heatmapData: HeatmapDataPoint[];
  isLoading?: boolean;
}

export const TrendAnalysis: React.FC<TrendAnalysisProps> = ({
  trendData,
  heatmapData,
  isLoading = false,
}) => {
  if (isLoading) {
    return (
      <div className="animate-pulse space-y-6">
        <div className="h-96 bg-gray-200 rounded-lg" />
        <div className="h-64 bg-gray-200 rounded-lg" />
      </div>
    );
  }

  if (trendData.length === 0 && heatmapData.length === 0) {
    return (
      <div className="text-center py-12 bg-gray-50 rounded-lg border border-gray-200">
        <p className="text-gray-500">No trend data available</p>
      </div>
    );
  }

  // Calculate trend statistics
  const calculateTrendStats = () => {
    if (trendData.length < 2) {
      return {
        avgScore: 0,
        scoreChange: 0,
        avgIssues: 0,
        issueChange: 0,
      };
    }

    const recentData = trendData.slice(-7); // Last 7 data points
    const avgScore =
      recentData.reduce((sum, point) => sum + point.score, 0) / recentData.length;

    const firstScore = trendData[0].score;
    const lastScore = trendData[trendData.length - 1].score;
    const scoreChange = lastScore - firstScore;

    const avgIssues =
      recentData.reduce((sum, point) => sum + point.issueCount, 0) /
      recentData.length;

    const firstIssues = trendData[0].issueCount;
    const lastIssues = trendData[trendData.length - 1].issueCount;
    const issueChange = lastIssues - firstIssues;

    return {
      avgScore: Math.round(avgScore),
      scoreChange: Math.round(scoreChange),
      avgIssues: Math.round(avgIssues),
      issueChange,
    };
  };

  const stats = calculateTrendStats();

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
        <h2 className="text-lg font-bold text-gray-900">Trend Analysis</h2>
        <p className="text-sm text-gray-600 mt-1">
          Historical compliance trends and patterns over time
        </p>
      </div>

      {/* Trend Statistics */}
      {trendData.length >= 2 && (
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-4">
            <h4 className="text-xs font-medium text-gray-500 mb-1">
              7-Day Avg Score
            </h4>
            <div className="text-2xl font-bold text-gray-900">
              {stats.avgScore}%
            </div>
          </div>

          <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-4">
            <h4 className="text-xs font-medium text-gray-500 mb-1">
              Score Change
            </h4>
            <div
              className={`text-2xl font-bold ${
                stats.scoreChange >= 0 ? 'text-green-600' : 'text-red-600'
              }`}
            >
              {stats.scoreChange >= 0 ? '+' : ''}
              {stats.scoreChange}%
            </div>
          </div>

          <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-4">
            <h4 className="text-xs font-medium text-gray-500 mb-1">
              7-Day Avg Issues
            </h4>
            <div className="text-2xl font-bold text-gray-900">
              {stats.avgIssues}
            </div>
          </div>

          <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-4">
            <h4 className="text-xs font-medium text-gray-500 mb-1">
              Issue Change
            </h4>
            <div
              className={`text-2xl font-bold ${
                stats.issueChange <= 0 ? 'text-green-600' : 'text-red-600'
              }`}
            >
              {stats.issueChange >= 0 ? '+' : ''}
              {stats.issueChange}
            </div>
          </div>
        </div>
      )}

      {/* Line Chart */}
      {trendData.length > 0 && (
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <TrendLineChart
            data={trendData}
            title="Compliance Score Trend"
            showIssueCount
            showCriticalCount
            height={350}
          />

          <div className="mt-6 pt-6 border-t border-gray-200 text-sm text-gray-600">
            <h4 className="font-semibold text-gray-900 mb-2">
              Trend Interpretation
            </h4>
            <ul className="list-disc list-inside space-y-1">
              <li>
                Blue line shows overall compliance score over time (0-100%)
              </li>
              <li>
                Orange line indicates total number of issues detected
              </li>
              <li>
                Red line highlights critical issues requiring immediate attention
              </li>
              <li>
                Upward compliance trend indicates improving accessibility
              </li>
            </ul>
          </div>
        </div>
      )}

      {/* Heatmap Calendar */}
      {heatmapData.length > 0 && (
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <HeatmapCalendar
            data={heatmapData}
            weeks={12}
            title="Compliance Activity Heatmap"
          />

          <div className="mt-6 pt-6 border-t border-gray-200 text-sm text-gray-600">
            <h4 className="font-semibold text-gray-900 mb-2">
              Reading the Heatmap
            </h4>
            <p>
              Each cell represents a day. Darker green indicates higher compliance
              scores. Hover over cells to see detailed information including
              compliance percentage and issue count for that day.
            </p>
          </div>
        </div>
      )}

      {/* Insights */}
      {trendData.length >= 2 && (
        <div className="bg-blue-50 rounded-lg border border-blue-200 p-6">
          <h4 className="text-sm font-semibold text-blue-900 mb-2">
            Key Insights
          </h4>
          <ul className="space-y-2 text-sm text-blue-800">
            {stats.scoreChange > 0 && (
              <li>
                ✓ Compliance score has improved by {stats.scoreChange}% over the
                tracked period
              </li>
            )}
            {stats.scoreChange < 0 && (
              <li>
                ⚠ Compliance score has decreased by {Math.abs(stats.scoreChange)}%
                over the tracked period
              </li>
            )}
            {stats.issueChange < 0 && (
              <li>
                ✓ Issue count has decreased by {Math.abs(stats.issueChange)},
                indicating progress
              </li>
            )}
            {stats.issueChange > 0 && (
              <li>
                ⚠ Issue count has increased by {stats.issueChange}, requiring
                attention
              </li>
            )}
            <li>
              Average compliance score over the last 7 periods: {stats.avgScore}%
            </li>
          </ul>
        </div>
      )}
    </div>
  );
};
