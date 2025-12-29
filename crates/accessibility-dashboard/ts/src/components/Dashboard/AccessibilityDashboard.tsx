/**
 * Accessibility Dashboard - Main Component
 * Enterprise WCAG Compliance Dashboard
 */

import React, { useState } from 'react';
import { ComplianceOverview } from './ComplianceOverview';
import { IssueBreakdown } from './IssueBreakdown';
import { TrendAnalysis } from './TrendAnalysis';
import { IssueList } from '../Widgets/IssueList';
import { PageRanking } from '../Widgets/PageRanking';
import { WCAGLevelFilter } from '../Filters/WCAGLevelFilter';
import { SeverityFilter } from '../Filters/SeverityFilter';
import { DateRangeFilter } from '../Filters/DateRangeFilter';
import { DashboardProvider, useDashboardContext } from '../../context/DashboardContext';
import { useCompliance } from '../../hooks/useCompliance';
import { useIssues } from '../../hooks/useIssues';
import type {
  AccessibilityIssue,
  TrendDataPoint,
  HeatmapDataPoint,
} from '../../types';

export interface AccessibilityDashboardProps {
  issues: AccessibilityIssue[];
  trendData?: TrendDataPoint[];
  heatmapData?: HeatmapDataPoint[];
  onIssueClick?: (issue: AccessibilityIssue) => void;
  onRefresh?: () => Promise<void>;
}

const DashboardContent: React.FC<AccessibilityDashboardProps> = ({
  issues,
  trendData = [],
  heatmapData = [],
  onIssueClick,
  onRefresh,
}) => {
  const { viewMode, setViewMode, filters, resetFilters } = useDashboardContext();
  const [showFilters, setShowFilters] = useState(false);

  const { metrics, pageCompliance, isLoading, refresh } = useCompliance(issues, {
    autoRefresh: false,
  });

  const {
    sortedIssues,
    issueCount,
    criticalCount,
    updateIssue,
  } = useIssues({ initialIssues: issues });

  const handleRefresh = async () => {
    await refresh();
    await onRefresh?.();
  };

  const hasActiveFilters =
    filters.wcagLevels.length < 3 ||
    filters.severities.length < 4 ||
    filters.dateRange.start !== null ||
    filters.dateRange.end !== null ||
    filters.searchQuery !== '';

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900">
                Accessibility Dashboard
              </h1>
              <p className="text-sm text-gray-600 mt-1">
                WCAG 2.1 Compliance Monitoring
              </p>
            </div>

            <div className="flex items-center gap-3">
              <button
                onClick={() => setShowFilters(!showFilters)}
                className="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                aria-expanded={showFilters}
                aria-controls="filters-panel"
              >
                <svg
                  className="w-5 h-5 mr-2"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                  />
                </svg>
                Filters
                {hasActiveFilters && (
                  <span className="ml-2 inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                    Active
                  </span>
                )}
              </button>

              <button
                onClick={handleRefresh}
                disabled={isLoading}
                className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50"
                aria-label="Refresh dashboard data"
              >
                <svg
                  className={`w-5 h-5 mr-2 ${isLoading ? 'animate-spin' : ''}`}
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                  />
                </svg>
                Refresh
              </button>
            </div>
          </div>

          {/* Summary Bar */}
          <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-gray-900">
                {issueCount}
              </div>
              <div className="text-xs text-gray-500">Total Issues</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-600">
                {criticalCount}
              </div>
              <div className="text-xs text-gray-500">Critical</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-gray-900">
                {pageCompliance.length}
              </div>
              <div className="text-xs text-gray-500">Pages</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">
                {metrics?.currentScore.overall || 0}%
              </div>
              <div className="text-xs text-gray-500">Compliance</div>
            </div>
          </div>
        </div>
      </header>

      {/* Filters Panel */}
      {showFilters && (
        <div
          id="filters-panel"
          className="bg-gray-100 border-b border-gray-200"
          role="region"
          aria-label="Filters"
        >
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold text-gray-900">Filters</h2>
              {hasActiveFilters && (
                <button
                  onClick={resetFilters}
                  className="text-sm text-blue-600 hover:text-blue-800 font-medium"
                >
                  Reset All Filters
                </button>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <WCAGLevelFilter />
              <SeverityFilter />
              <DateRangeFilter />
            </div>
          </div>
        </div>
      )}

      {/* View Mode Tabs */}
      <div className="bg-white border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <nav className="-mb-px flex gap-8" aria-label="Dashboard views">
            {[
              { value: 'overview', label: 'Overview' },
              { value: 'detailed', label: 'Detailed Analysis' },
              { value: 'trends', label: 'Trends' },
            ].map((tab) => (
              <button
                key={tab.value}
                onClick={() => setViewMode(tab.value as any)}
                className={`py-4 px-1 border-b-2 font-medium text-sm ${
                  viewMode === tab.value
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
                aria-current={viewMode === tab.value ? 'page' : undefined}
              >
                {tab.label}
              </button>
            ))}
          </nav>
        </div>
      </div>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {viewMode === 'overview' && (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="lg:col-span-2">
              <ComplianceOverview metrics={metrics} isLoading={isLoading} />
            </div>
            <div className="space-y-6">
              <PageRanking
                pages={pageCompliance}
                maxItems={5}
                showScore
              />
            </div>
          </div>
        )}

        {viewMode === 'detailed' && (
          <div className="space-y-6">
            <IssueBreakdown metrics={metrics} isLoading={isLoading} />
            <IssueList
              issues={sortedIssues}
              onIssueClick={onIssueClick}
              onStatusChange={(id, status) => updateIssue(id, { status })}
              showPagination
              itemsPerPage={20}
            />
          </div>
        )}

        {viewMode === 'trends' && (
          <TrendAnalysis
            trendData={trendData}
            heatmapData={heatmapData}
            isLoading={isLoading}
          />
        )}
      </main>
    </div>
  );
};

/**
 * Main Dashboard Component with Provider
 */
export const AccessibilityDashboard: React.FC<AccessibilityDashboardProps> = (
  props
) => {
  return (
    <DashboardProvider>
      <DashboardContent {...props} />
    </DashboardProvider>
  );
};
