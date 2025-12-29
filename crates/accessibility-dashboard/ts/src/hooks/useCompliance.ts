/**
 * useCompliance Hook
 * Manages compliance data fetching and calculations
 */

import { useState, useEffect, useMemo } from 'react';
import type {
  ComplianceScore,
  ComplianceMetrics,
  PageCompliance,
  AccessibilityIssue,
  WCAGLevel,
} from '../types';
import {
  calculateComplianceScore,
  calculateWeightedScore,
  calculateCategoryBreakdown,
  calculateSeverityBreakdown,
  calculateTrend,
  calculateChange,
  calculateLevelCompliance,
  calculateAverageScore,
} from '../utils/calculations';
import { useDashboardContext } from '../context/DashboardContext';

export interface UseComplianceOptions {
  autoRefresh?: boolean;
  refreshInterval?: number; // in milliseconds
}

export interface UseComplianceReturn {
  metrics: ComplianceMetrics | null;
  pageCompliance: PageCompliance[];
  isLoading: boolean;
  error: Error | null;
  refresh: () => Promise<void>;
}

/**
 * Hook for managing compliance data and metrics
 */
export function useCompliance(
  issues: AccessibilityIssue[],
  options: UseComplianceOptions = {}
): UseComplianceReturn {
  const { autoRefresh = false, refreshInterval = 60000 } = options;
  const { filters } = useDashboardContext();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [previousScore, setPreviousScore] = useState<ComplianceScore | null>(
    null
  );

  // Filter issues based on dashboard filters
  const filteredIssues = useMemo(() => {
    return issues.filter((issue) => {
      // Filter by WCAG level
      if (!filters.wcagLevels.includes(issue.criterion.level)) {
        return false;
      }

      // Filter by severity
      if (!filters.severities.includes(issue.severity)) {
        return false;
      }

      // Filter by category
      if (!filters.categories.includes(issue.criterion.category)) {
        return false;
      }

      // Filter by status
      if (!filters.statuses.includes(issue.status)) {
        return false;
      }

      // Filter by date range
      if (filters.dateRange.start && issue.detectedAt < filters.dateRange.start) {
        return false;
      }

      if (filters.dateRange.end && issue.detectedAt > filters.dateRange.end) {
        return false;
      }

      // Filter by search query
      if (filters.searchQuery) {
        const query = filters.searchQuery.toLowerCase();
        return (
          issue.description.toLowerCase().includes(query) ||
          issue.pageUrl.toLowerCase().includes(query) ||
          issue.element.toLowerCase().includes(query) ||
          issue.criterion.name.toLowerCase().includes(query)
        );
      }

      return true;
    });
  }, [issues, filters]);

  // Calculate metrics from filtered issues
  const metrics = useMemo<ComplianceMetrics | null>(() => {
    if (filteredIssues.length === 0) {
      // Return perfect score if no issues
      const perfectScore = calculateComplianceScore(100, 100, 0, 0, 'AA');

      return {
        currentScore: perfectScore,
        previousScore: previousScore || perfectScore,
        change: 0,
        trend: 'stable',
        issues: {
          total: 0,
          byCategory: calculateCategoryBreakdown([]),
          bySeverity: calculateSeverityBreakdown([]),
        },
        pages: {
          total: 0,
          compliant: 0,
          nonCompliant: 0,
          avgScore: 100,
        },
      };
    }

    // Calculate current score
    const openIssues = filteredIssues.filter((i) => i.status === 'open');
    const resolvedIssues = filteredIssues.filter((i) => i.status === 'resolved');

    const totalTests = filteredIssues.length;
    const passedTests = resolvedIssues.length;
    const failedTests = openIssues.length;
    const warningTests = filteredIssues.filter(
      (i) => i.severity === 'minor' || i.severity === 'moderate'
    ).length;

    const currentScore = calculateComplianceScore(
      totalTests,
      passedTests,
      failedTests,
      warningTests,
      'AA'
    );

    // Use weighted score for overall metric
    const weightedScore = calculateWeightedScore(filteredIssues);
    currentScore.overall = weightedScore;

    // Calculate change and trend
    const change = previousScore
      ? calculateChange(currentScore.overall, previousScore.overall)
      : 0;

    const trend = previousScore
      ? calculateTrend(currentScore.overall, previousScore.overall)
      : 'stable';

    // Calculate page statistics
    const pageUrls = [...new Set(filteredIssues.map((i) => i.pageUrl))];
    const pageScores = pageUrls.map((url) => {
      const pageIssues = filteredIssues.filter((i) => i.pageUrl === url);
      return calculateWeightedScore(pageIssues);
    });

    const avgScore = calculateAverageScore(pageScores);
    const compliantPages = pageScores.filter((score) => score >= 90).length;

    return {
      currentScore,
      previousScore: previousScore || currentScore,
      change,
      trend,
      issues: {
        total: filteredIssues.length,
        byCategory: calculateCategoryBreakdown(filteredIssues),
        bySeverity: calculateSeverityBreakdown(filteredIssues),
      },
      pages: {
        total: pageUrls.length,
        compliant: compliantPages,
        nonCompliant: pageUrls.length - compliantPages,
        avgScore,
      },
    };
  }, [filteredIssues, previousScore]);

  // Calculate page compliance
  const pageCompliance = useMemo<PageCompliance[]>(() => {
    const pageUrls = [...new Set(filteredIssues.map((i) => i.pageUrl))];

    return pageUrls
      .map((url) => {
        const pageIssues = filteredIssues.filter((i) => i.pageUrl === url);
        const criticalCount = pageIssues.filter(
          (i) => i.severity === 'critical'
        ).length;

        const openIssues = pageIssues.filter((i) => i.status === 'open');
        const resolvedIssues = pageIssues.filter((i) => i.status === 'resolved');

        const score = calculateComplianceScore(
          pageIssues.length,
          resolvedIssues.length,
          openIssues.length,
          0,
          'AA'
        );

        score.overall = calculateWeightedScore(pageIssues);

        const lastScanned =
          pageIssues.length > 0
            ? new Date(
                Math.max(...pageIssues.map((i) => i.detectedAt.getTime()))
              )
            : new Date();

        // Extract title from URL (basic implementation)
        const title =
          url.split('/').pop()?.replace(/-/g, ' ') || 'Unknown Page';

        return {
          url,
          title,
          score,
          issueCount: pageIssues.length,
          criticalIssues: criticalCount,
          lastScanned,
        };
      })
      .sort((a, b) => b.issueCount - a.issueCount);
  }, [filteredIssues]);

  // Refresh function
  const refresh = async (): Promise<void> => {
    setIsLoading(true);
    setError(null);

    try {
      // Store current score as previous for trend calculation
      if (metrics) {
        setPreviousScore(metrics.currentScore);
      }

      // In a real implementation, this would fetch fresh data
      await new Promise((resolve) => setTimeout(resolve, 500));
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to refresh data'));
    } finally {
      setIsLoading(false);
    }
  };

  // Auto-refresh effect
  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      refresh();
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval]);

  return {
    metrics,
    pageCompliance,
    isLoading,
    error,
    refresh,
  };
}
