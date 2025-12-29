/**
 * Issue List Component
 * Displays list of accessibility issues with filtering and sorting
 */

import React, { useState, useMemo } from 'react';
import type { AccessibilityIssue, AccessibilitySeverity } from '../../types/index.js';

export interface IssueListProps {
  issues: AccessibilityIssue[];
  className?: string;
}

export function IssueList({ issues, className = '' }: IssueListProps): JSX.Element {
  const [filterSeverity, setFilterSeverity] = useState<AccessibilitySeverity | 'all'>('all');
  const [sortBy, setSortBy] = useState<'severity' | 'type' | 'page'>('severity');
  const [expandedIssues, setExpandedIssues] = useState<Set<string>>(new Set());

  const filteredAndSortedIssues = useMemo(() => {
    let filtered = issues;

    // Filter by severity
    if (filterSeverity !== 'all') {
      filtered = filtered.filter(issue => issue.severity === filterSeverity);
    }

    // Sort
    return [...filtered].sort((a, b) => {
      if (sortBy === 'severity') {
        const severityOrder = { critical: 0, error: 1, warning: 2, info: 3 };
        return severityOrder[a.severity] - severityOrder[b.severity];
      } else if (sortBy === 'type') {
        return a.type.localeCompare(b.type);
      } else if (sortBy === 'page') {
        return (a.pageNumber || 0) - (b.pageNumber || 0);
      }
      return 0;
    });
  }, [issues, filterSeverity, sortBy]);

  const toggleIssue = (issueId: string) => {
    setExpandedIssues(prev => {
      const next = new Set(prev);
      if (next.has(issueId)) {
        next.delete(issueId);
      } else {
        next.add(issueId);
      }
      return next;
    });
  };

  const severityCounts = useMemo(() => ({
    critical: issues.filter(i => i.severity === 'critical').length,
    error: issues.filter(i => i.severity === 'error').length,
    warning: issues.filter(i => i.severity === 'warning').length,
    info: issues.filter(i => i.severity === 'info').length
  }), [issues]);

  return (
    <div className={`issue-list ${className}`}>
      <div className="issue-list__controls">
        <div className="issue-list__filters">
          <label htmlFor="severity-filter">Filter by severity:</label>
          <select
            id="severity-filter"
            value={filterSeverity}
            onChange={(e) => setFilterSeverity(e.target.value as AccessibilitySeverity | 'all')}
          >
            <option value="all">All ({issues.length})</option>
            <option value="critical">Critical ({severityCounts.critical})</option>
            <option value="error">Error ({severityCounts.error})</option>
            <option value="warning">Warning ({severityCounts.warning})</option>
            <option value="info">Info ({severityCounts.info})</option>
          </select>
        </div>

        <div className="issue-list__sort">
          <label htmlFor="sort-by">Sort by:</label>
          <select
            id="sort-by"
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as 'severity' | 'type' | 'page')}
          >
            <option value="severity">Severity</option>
            <option value="type">Type</option>
            <option value="page">Page</option>
          </select>
        </div>
      </div>

      {filteredAndSortedIssues.length === 0 ? (
        <div className="issue-list__empty">
          <p>No issues found matching the selected criteria.</p>
        </div>
      ) : (
        <div className="issue-list__items">
          {filteredAndSortedIssues.map((issue) => (
            <IssueItem
              key={issue.id}
              issue={issue}
              expanded={expandedIssues.has(issue.id)}
              onToggle={() => toggleIssue(issue.id)}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface IssueItemProps {
  issue: AccessibilityIssue;
  expanded: boolean;
  onToggle: () => void;
}

function IssueItem({ issue, expanded, onToggle }: IssueItemProps): JSX.Element {
  return (
    <div className={`issue-item issue-item--${issue.severity}`}>
      <button
        className="issue-item__header"
        onClick={onToggle}
        aria-expanded={expanded}
        aria-controls={`issue-${issue.id}`}
      >
        <div className="issue-item__severity">
          <SeverityBadge severity={issue.severity} />
        </div>
        <div className="issue-item__title">
          <h4>{issue.title}</h4>
          {issue.pageNumber && (
            <span className="issue-item__page">Page {issue.pageNumber}</span>
          )}
        </div>
        <div className="issue-item__expand" aria-hidden="true">
          {expanded ? '−' : '+'}
        </div>
      </button>

      {expanded && (
        <div id={`issue-${issue.id}`} className="issue-item__content">
          <p className="issue-item__description">{issue.description}</p>

          {issue.wcagCriteria && issue.wcagCriteria.length > 0 && (
            <div className="issue-item__wcag">
              <strong>WCAG Criteria:</strong>{' '}
              {issue.wcagCriteria.join(', ')} ({issue.wcagLevel})
            </div>
          )}

          {issue.location && (
            <div className="issue-item__location">
              <strong>Location:</strong>{' '}
              {issue.location.element || issue.location.xpath || 'N/A'}
            </div>
          )}

          {issue.remediation && (
            <div className="issue-item__remediation">
              <h5>How to Fix:</h5>
              <p><strong>{issue.remediation.action}</strong></p>
              <p>{issue.remediation.description}</p>
              {issue.remediation.steps.length > 0 && (
                <ol>
                  {issue.remediation.steps.map((step, index) => (
                    <li key={index}>{step}</li>
                  ))}
                </ol>
              )}
              {issue.remediation.codeExample && (
                <pre className="issue-item__code">
                  <code>{issue.remediation.codeExample}</code>
                </pre>
              )}
              <div className="issue-item__meta">
                <span>Effort: {issue.remediation.estimatedEffort}</span>
                {issue.remediation.toolsRequired && issue.remediation.toolsRequired.length > 0 && (
                  <span>Tools: {issue.remediation.toolsRequired.join(', ')}</span>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface SeverityBadgeProps {
  severity: AccessibilitySeverity;
}

function SeverityBadge({ severity }: SeverityBadgeProps): JSX.Element {
  const labels = {
    critical: 'Critical',
    error: 'Error',
    warning: 'Warning',
    info: 'Info'
  };

  const icons = {
    critical: '⛔',
    error: '❌',
    warning: '⚠️',
    info: 'ℹ️'
  };

  return (
    <span className={`severity-badge severity-badge--${severity}`}>
      <span className="severity-badge__icon" aria-hidden="true">
        {icons[severity]}
      </span>
      <span className="severity-badge__label">{labels[severity]}</span>
    </span>
  );
}
