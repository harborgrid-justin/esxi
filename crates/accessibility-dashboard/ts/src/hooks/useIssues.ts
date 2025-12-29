/**
 * useIssues Hook
 * Manages accessibility issue data and filtering
 */

import { useState, useEffect, useMemo, useCallback } from 'react';
import type {
  AccessibilityIssue,
  IssueSeverity,
  IssueStatus,
  IssueCategory,
  WCAGLevel,
} from '../types';
import { useDashboardContext } from '../context/DashboardContext';

export interface UseIssuesOptions {
  initialIssues?: AccessibilityIssue[];
  sortBy?: 'severity' | 'date' | 'page' | 'status';
  sortOrder?: 'asc' | 'desc';
}

export interface UseIssuesReturn {
  issues: AccessibilityIssue[];
  filteredIssues: AccessibilityIssue[];
  sortedIssues: AccessibilityIssue[];
  issueCount: number;
  criticalCount: number;
  openCount: number;
  isLoading: boolean;
  error: Error | null;
  updateIssue: (issueId: string, updates: Partial<AccessibilityIssue>) => void;
  deleteIssue: (issueId: string) => void;
  bulkUpdateStatus: (issueIds: string[], status: IssueStatus) => void;
  setSortBy: (sortBy: UseIssuesOptions['sortBy']) => void;
  setSortOrder: (sortOrder: UseIssuesOptions['sortOrder']) => void;
}

// Severity order for sorting
const SEVERITY_ORDER: Record<IssueSeverity, number> = {
  critical: 4,
  serious: 3,
  moderate: 2,
  minor: 1,
};

/**
 * Hook for managing accessibility issues
 */
export function useIssues(
  options: UseIssuesOptions = {}
): UseIssuesReturn {
  const {
    initialIssues = [],
    sortBy: initialSortBy = 'severity',
    sortOrder: initialSortOrder = 'desc',
  } = options;

  const { filters } = useDashboardContext();

  const [issues, setIssues] = useState<AccessibilityIssue[]>(initialIssues);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [sortBy, setSortBy] = useState<UseIssuesOptions['sortBy']>(initialSortBy);
  const [sortOrder, setSortOrder] = useState<UseIssuesOptions['sortOrder']>(
    initialSortOrder
  );

  // Update issues when initial data changes
  useEffect(() => {
    setIssues(initialIssues);
  }, [initialIssues]);

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
          issue.criterion.name.toLowerCase().includes(query) ||
          issue.criterion.code.toLowerCase().includes(query)
        );
      }

      return true;
    });
  }, [issues, filters]);

  // Sort filtered issues
  const sortedIssues = useMemo(() => {
    const sorted = [...filteredIssues];

    sorted.sort((a, b) => {
      let comparison = 0;

      switch (sortBy) {
        case 'severity':
          comparison =
            SEVERITY_ORDER[a.severity] - SEVERITY_ORDER[b.severity];
          break;

        case 'date':
          comparison =
            a.detectedAt.getTime() - b.detectedAt.getTime();
          break;

        case 'page':
          comparison = a.pageUrl.localeCompare(b.pageUrl);
          break;

        case 'status':
          comparison = a.status.localeCompare(b.status);
          break;

        default:
          comparison = 0;
      }

      return sortOrder === 'asc' ? comparison : -comparison;
    });

    return sorted;
  }, [filteredIssues, sortBy, sortOrder]);

  // Calculate counts
  const issueCount = filteredIssues.length;
  const criticalCount = filteredIssues.filter(
    (issue) => issue.severity === 'critical'
  ).length;
  const openCount = filteredIssues.filter(
    (issue) => issue.status === 'open'
  ).length;

  // Update a single issue
  const updateIssue = useCallback(
    (issueId: string, updates: Partial<AccessibilityIssue>) => {
      setIssues((prevIssues) =>
        prevIssues.map((issue) =>
          issue.id === issueId
            ? { ...issue, ...updates, updatedAt: new Date() }
            : issue
        )
      );
    },
    []
  );

  // Delete an issue
  const deleteIssue = useCallback((issueId: string) => {
    setIssues((prevIssues) => prevIssues.filter((issue) => issue.id !== issueId));
  }, []);

  // Bulk update status
  const bulkUpdateStatus = useCallback(
    (issueIds: string[], status: IssueStatus) => {
      setIssues((prevIssues) =>
        prevIssues.map((issue) =>
          issueIds.includes(issue.id)
            ? { ...issue, status, updatedAt: new Date() }
            : issue
        )
      );
    },
    []
  );

  return {
    issues,
    filteredIssues,
    sortedIssues,
    issueCount,
    criticalCount,
    openCount,
    isLoading,
    error,
    updateIssue,
    deleteIssue,
    bulkUpdateStatus,
    setSortBy,
    setSortOrder,
  };
}
