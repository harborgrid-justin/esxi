/**
 * Compliance Overview Component
 * Displays overall compliance metrics and KPIs
 */

import React from 'react';
import { ScoreCard } from '../Widgets/ScoreCard';
import { ComplianceGauge } from '../Charts/ComplianceGauge';
import type { ComplianceMetrics } from '../../types';
import { formatPercentage } from '../../utils/calculations';

export interface ComplianceOverviewProps {
  metrics: ComplianceMetrics | null;
  isLoading?: boolean;
}

export const ComplianceOverview: React.FC<ComplianceOverviewProps> = ({
  metrics,
  isLoading = false,
}) => {
  if (isLoading) {
    return (
      <div className="animate-pulse space-y-4">
        <div className="h-64 bg-gray-200 rounded-lg" />
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="h-32 bg-gray-200 rounded-lg" />
          <div className="h-32 bg-gray-200 rounded-lg" />
          <div className="h-32 bg-gray-200 rounded-lg" />
        </div>
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className="text-center py-12 bg-gray-50 rounded-lg border border-gray-200">
        <p className="text-gray-500">No compliance data available</p>
      </div>
    );
  }

  const { currentScore, change, trend, issues, pages } = metrics;

  const getTrendProps = () => {
    if (trend === 'improving') {
      return {
        trend: 'up' as const,
        trendValue: `+${formatPercentage(Math.abs(change))}`,
      };
    }
    if (trend === 'declining') {
      return {
        trend: 'down' as const,
        trendValue: `-${formatPercentage(Math.abs(change))}`,
      };
    }
    return {
      trend: 'neutral' as const,
      trendValue: 'No change',
    };
  };

  const trendProps = getTrendProps();

  return (
    <div className="space-y-6">
      {/* Main Gauge */}
      <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
        <ComplianceGauge
          score={currentScore.overall}
          title="Overall WCAG Compliance"
          subtitle={`Level ${currentScore.level} Conformance`}
          size={240}
        />

        <div className="mt-6 grid grid-cols-3 gap-4 pt-6 border-t border-gray-200">
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {currentScore.passedTests}
            </div>
            <div className="text-xs text-gray-500 mt-1">Passed Tests</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-red-600">
              {currentScore.failedTests}
            </div>
            <div className="text-xs text-gray-500 mt-1">Failed Tests</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-amber-600">
              {currentScore.warningTests}
            </div>
            <div className="text-xs text-gray-500 mt-1">Warnings</div>
          </div>
        </div>
      </div>

      {/* KPI Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <ScoreCard
          title="Compliance Rate"
          value={currentScore.overall}
          isPercentage
          {...trendProps}
          subtitle="vs. previous period"
        />

        <ScoreCard
          title="Total Issues"
          value={issues.total}
          subtitle={`${
            issues.bySeverity.find((s) => s.severity === 'critical')?.count || 0
          } critical`}
          variant={issues.total > 50 ? 'danger' : 'default'}
        />

        <ScoreCard
          title="Pages Analyzed"
          value={pages.total}
          subtitle={`${pages.compliant} compliant (${formatPercentage(
            pages.total > 0 ? (pages.compliant / pages.total) * 100 : 0
          )})`}
        />

        <ScoreCard
          title="Average Page Score"
          value={pages.avgScore}
          isPercentage
          subtitle={`${pages.nonCompliant} pages need work`}
        />
      </div>

      {/* Issue Breakdown Summary */}
      <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
        <h3 className="text-sm font-semibold text-gray-900 mb-4">
          Issue Summary by Category
        </h3>

        <div className="space-y-3">
          {issues.byCategory.map((category) => {
            const percentage = formatPercentage(category.percentage);

            return (
              <div key={category.category}>
                <div className="flex items-center justify-between text-sm mb-1">
                  <span className="font-medium text-gray-700 capitalize">
                    {category.category}
                  </span>
                  <span className="text-gray-900">
                    {category.count} issues ({percentage})
                  </span>
                </div>
                <div className="w-full bg-gray-200 rounded-full h-2">
                  <div
                    className="bg-blue-500 h-2 rounded-full transition-all"
                    style={{ width: `${category.percentage}%` }}
                    role="progressbar"
                    aria-valuenow={category.percentage}
                    aria-valuemin={0}
                    aria-valuemax={100}
                    aria-label={`${category.category}: ${percentage} of total issues`}
                  />
                </div>
                {category.criticalCount > 0 && (
                  <p className="text-xs text-red-600 mt-1">
                    {category.criticalCount} critical{' '}
                    {category.criticalCount === 1 ? 'issue' : 'issues'}
                  </p>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
