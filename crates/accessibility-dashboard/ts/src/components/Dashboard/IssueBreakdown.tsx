/**
 * Issue Breakdown Component
 * Displays detailed issue analysis by category and severity
 */

import React from 'react';
import { IssueDistribution } from '../Charts/IssueDistribution';
import type { ComplianceMetrics } from '../../types';

export interface IssueBreakdownProps {
  metrics: ComplianceMetrics | null;
  isLoading?: boolean;
}

export const IssueBreakdown: React.FC<IssueBreakdownProps> = ({
  metrics,
  isLoading = false,
}) => {
  if (isLoading) {
    return (
      <div className="animate-pulse grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="h-96 bg-gray-200 rounded-lg" />
        <div className="h-96 bg-gray-200 rounded-lg" />
      </div>
    );
  }

  if (!metrics) {
    return (
      <div className="text-center py-12 bg-gray-50 rounded-lg border border-gray-200">
        <p className="text-gray-500">No issue data available</p>
      </div>
    );
  }

  const { issues } = metrics;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
        <h2 className="text-lg font-bold text-gray-900">Issue Analysis</h2>
        <p className="text-sm text-gray-600 mt-1">
          Detailed breakdown of {issues.total} accessibility{' '}
          {issues.total === 1 ? 'issue' : 'issues'} detected
        </p>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* By Category */}
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <IssueDistribution
            data={issues.byCategory}
            type="category"
            chartType="doughnut"
            title="Issues by WCAG Principle"
          />

          <div className="mt-6 pt-6 border-t border-gray-200">
            <h4 className="text-sm font-semibold text-gray-900 mb-3">
              WCAG 2.1 Principles
            </h4>
            <ul className="space-y-2 text-xs text-gray-600">
              <li>
                <strong className="text-gray-900">Perceivable:</strong> Information
                must be presentable to users
              </li>
              <li>
                <strong className="text-gray-900">Operable:</strong> UI components
                must be operable
              </li>
              <li>
                <strong className="text-gray-900">Understandable:</strong>{' '}
                Information must be understandable
              </li>
              <li>
                <strong className="text-gray-900">Robust:</strong> Content must be
                robust enough for assistive technologies
              </li>
            </ul>
          </div>
        </div>

        {/* By Severity */}
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <IssueDistribution
            data={issues.bySeverity}
            type="severity"
            chartType="doughnut"
            title="Issues by Severity Level"
          />

          <div className="mt-6 pt-6 border-t border-gray-200">
            <h4 className="text-sm font-semibold text-gray-900 mb-3">
              Severity Definitions
            </h4>
            <ul className="space-y-2 text-xs text-gray-600">
              <li>
                <strong className="text-red-600">Critical:</strong> Blocks access
                for users with disabilities
              </li>
              <li>
                <strong className="text-orange-600">Serious:</strong> Creates
                significant barriers to accessibility
              </li>
              <li>
                <strong className="text-amber-600">Moderate:</strong> May cause
                difficulty for some users
              </li>
              <li>
                <strong className="text-blue-600">Minor:</strong> Minor
                inconvenience or best practice
              </li>
            </ul>
          </div>
        </div>
      </div>

      {/* Issue Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <h4 className="text-sm font-medium text-gray-500 mb-2">
            Most Common Category
          </h4>
          <div className="text-2xl font-bold text-gray-900 capitalize">
            {issues.byCategory.reduce((max, cat) =>
              cat.count > max.count ? cat : max
            ).category}
          </div>
          <p className="text-sm text-gray-600 mt-1">
            {issues.byCategory.reduce((max, cat) =>
              cat.count > max.count ? cat : max
            ).count}{' '}
            issues (
            {issues.byCategory
              .reduce((max, cat) => (cat.count > max.count ? cat : max))
              .percentage.toFixed(1)}
            %)
          </p>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <h4 className="text-sm font-medium text-gray-500 mb-2">
            Critical Issues
          </h4>
          <div className="text-2xl font-bold text-red-600">
            {issues.bySeverity.find((s) => s.severity === 'critical')?.count || 0}
          </div>
          <p className="text-sm text-gray-600 mt-1">
            Require immediate attention
          </p>
        </div>

        <div className="bg-white rounded-lg border border-gray-200 shadow-sm p-6">
          <h4 className="text-sm font-medium text-gray-500 mb-2">
            Total Categories
          </h4>
          <div className="text-2xl font-bold text-gray-900">
            {issues.byCategory.filter((cat) => cat.count > 0).length}
          </div>
          <p className="text-sm text-gray-600 mt-1">
            of 4 WCAG principles affected
          </p>
        </div>
      </div>
    </div>
  );
};
