/**
 * Issue List Widget
 * Filterable, sortable list of accessibility issues
 */

import React, { useState, useId } from 'react';
import { format } from 'date-fns';
import clsx from 'clsx';
import type { AccessibilityIssue, IssueSeverity, IssueStatus } from '../../types';
import { getSeverityColor } from '../../utils/calculations';

export interface IssueListProps {
  issues: AccessibilityIssue[];
  onIssueClick?: (issue: AccessibilityIssue) => void;
  onStatusChange?: (issueId: string, status: IssueStatus) => void;
  maxHeight?: string;
  showPagination?: boolean;
  itemsPerPage?: number;
}

const SEVERITY_LABELS: Record<IssueSeverity, string> = {
  critical: 'Critical',
  serious: 'Serious',
  moderate: 'Moderate',
  minor: 'Minor',
};

const STATUS_LABELS: Record<IssueStatus, string> = {
  open: 'Open',
  'in-progress': 'In Progress',
  resolved: 'Resolved',
  'wont-fix': "Won't Fix",
};

export const IssueList: React.FC<IssueListProps> = ({
  issues,
  onIssueClick,
  onStatusChange,
  maxHeight = '600px',
  showPagination = true,
  itemsPerPage = 10,
}) => {
  const [currentPage, setCurrentPage] = useState(1);
  const listId = useId();

  const totalPages = Math.ceil(issues.length / itemsPerPage);
  const startIndex = (currentPage - 1) * itemsPerPage;
  const endIndex = startIndex + itemsPerPage;
  const paginatedIssues = showPagination
    ? issues.slice(startIndex, endIndex)
    : issues;

  const handleStatusChange = (
    issueId: string,
    newStatus: IssueStatus,
    e: React.MouseEvent
  ) => {
    e.stopPropagation();
    onStatusChange?.(issueId, newStatus);
  };

  if (issues.length === 0) {
    return (
      <div
        className="text-center py-12 bg-gray-50 rounded-lg border border-gray-200"
        role="status"
      >
        <p className="text-gray-500 text-sm">No issues found</p>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 shadow-sm">
      <div
        className="overflow-auto"
        style={{ maxHeight }}
        role="region"
        aria-labelledby={`${listId}-heading`}
        tabIndex={0}
      >
        <h3 id={`${listId}-heading`} className="sr-only">
          Accessibility Issues List
        </h3>
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50 sticky top-0">
            <tr>
              <th
                scope="col"
                className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Issue
              </th>
              <th
                scope="col"
                className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Severity
              </th>
              <th
                scope="col"
                className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Status
              </th>
              <th
                scope="col"
                className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Page
              </th>
              <th
                scope="col"
                className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Detected
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {paginatedIssues.map((issue) => {
              const severityColor = getSeverityColor(issue.severity);

              return (
                <tr
                  key={issue.id}
                  onClick={() => onIssueClick?.(issue)}
                  className={clsx(
                    'hover:bg-gray-50 transition-colors',
                    onIssueClick && 'cursor-pointer'
                  )}
                  tabIndex={onIssueClick ? 0 : undefined}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      onIssueClick?.(issue);
                    }
                  }}
                  role={onIssueClick ? 'button' : undefined}
                >
                  <td className="px-4 py-4">
                    <div className="text-sm">
                      <div className="font-medium text-gray-900">
                        {issue.criterion.code} - {issue.criterion.name}
                      </div>
                      <div className="text-gray-500 truncate max-w-md">
                        {issue.description}
                      </div>
                      <div className="text-xs text-gray-400 mt-1">
                        {issue.element}
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-4 whitespace-nowrap">
                    <span
                      className="inline-flex items-center gap-2 px-2.5 py-0.5 rounded-full text-xs font-medium text-white"
                      style={{ backgroundColor: severityColor }}
                    >
                      <span
                        className="w-2 h-2 rounded-full bg-white"
                        aria-hidden="true"
                      />
                      {SEVERITY_LABELS[issue.severity]}
                    </span>
                  </td>
                  <td className="px-4 py-4 whitespace-nowrap">
                    {onStatusChange ? (
                      <select
                        value={issue.status}
                        onChange={(e) =>
                          handleStatusChange(
                            issue.id,
                            e.target.value as IssueStatus,
                            e as unknown as React.MouseEvent
                          )
                        }
                        onClick={(e) => e.stopPropagation()}
                        className="text-sm border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                        aria-label={`Change status for ${issue.criterion.name}`}
                      >
                        {Object.entries(STATUS_LABELS).map(([value, label]) => (
                          <option key={value} value={value}>
                            {label}
                          </option>
                        ))}
                      </select>
                    ) : (
                      <span className="text-sm text-gray-900">
                        {STATUS_LABELS[issue.status]}
                      </span>
                    )}
                  </td>
                  <td className="px-4 py-4">
                    <div className="text-sm text-gray-900 truncate max-w-xs">
                      {issue.pageUrl}
                    </div>
                  </td>
                  <td className="px-4 py-4 whitespace-nowrap text-sm text-gray-500">
                    {format(issue.detectedAt, 'MMM d, yyyy')}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {showPagination && totalPages > 1 && (
        <div
          className="px-4 py-3 border-t border-gray-200 bg-gray-50 flex items-center justify-between"
          role="navigation"
          aria-label="Pagination"
        >
          <div className="text-sm text-gray-700">
            Showing {startIndex + 1} to {Math.min(endIndex, issues.length)} of{' '}
            {issues.length} issues
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
              disabled={currentPage === 1}
              className="px-3 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-500"
              aria-label="Previous page"
            >
              Previous
            </button>
            <span className="px-3 py-1 text-sm text-gray-700">
              Page {currentPage} of {totalPages}
            </span>
            <button
              onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
              disabled={currentPage === totalPages}
              className="px-3 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-blue-500"
              aria-label="Next page"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
