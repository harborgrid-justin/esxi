/**
 * Highlight accessibility issues on the page
 */

import React, { useEffect, useState } from 'react';
import type { AccessibilityIssue } from '../../types';

export interface IssueHighlighterProps {
  issues: AccessibilityIssue[];
  selectedIssue: AccessibilityIssue | null;
  onIssueSelect: (issue: AccessibilityIssue) => void;
  enabled?: boolean;
  className?: string;
}

export const IssueHighlighter: React.FC<IssueHighlighterProps> = ({
  issues,
  selectedIssue,
  onIssueSelect,
  enabled = true,
  className = '',
}) => {
  const [highlightedElements, setHighlightedElements] = useState<Map<Element, HTMLDivElement>>(new Map());

  useEffect(() => {
    if (!enabled) {
      // Remove all highlights
      highlightedElements.forEach((overlay) => overlay.remove());
      setHighlightedElements(new Map());
      return;
    }

    // Clear existing highlights
    highlightedElements.forEach((overlay) => overlay.remove());
    const newHighlights = new Map<Element, HTMLDivElement>();

    // Add highlights for each issue
    issues.forEach((issue) => {
      const element = issue.node.element;
      const rect = element.getBoundingClientRect();

      const overlay = document.createElement('div');
      overlay.className = `a11y-issue-overlay a11y-issue-${issue.severity}`;
      overlay.style.cssText = `
        position: absolute;
        top: ${window.scrollY + rect.top}px;
        left: ${window.scrollX + rect.left}px;
        width: ${rect.width}px;
        height: ${rect.height}px;
        pointer-events: all;
        z-index: 999999;
        cursor: pointer;
      `;

      overlay.addEventListener('click', () => {
        onIssueSelect(issue);
      });

      // Add badge with issue count
      const badge = document.createElement('div');
      badge.className = `a11y-issue-badge a11y-issue-badge-${issue.severity}`;
      badge.textContent = '!';
      badge.style.cssText = `
        position: absolute;
        top: -8px;
        right: -8px;
        width: 20px;
        height: 20px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        font-weight: bold;
        color: white;
      `;

      overlay.appendChild(badge);
      document.body.appendChild(overlay);
      newHighlights.set(element, overlay);
    });

    setHighlightedElements(newHighlights);

    // Update highlights on scroll/resize
    const updatePositions = () => {
      issues.forEach((issue) => {
        const element = issue.node.element;
        const overlay = newHighlights.get(element);
        if (overlay) {
          const rect = element.getBoundingClientRect();
          overlay.style.top = `${window.scrollY + rect.top}px`;
          overlay.style.left = `${window.scrollX + rect.left}px`;
          overlay.style.width = `${rect.width}px`;
          overlay.style.height = `${rect.height}px`;
        }
      });
    };

    window.addEventListener('scroll', updatePositions);
    window.addEventListener('resize', updatePositions);

    return () => {
      window.removeEventListener('scroll', updatePositions);
      window.removeEventListener('resize', updatePositions);
      newHighlights.forEach((overlay) => overlay.remove());
    };
  }, [issues, enabled, onIssueSelect]);

  // Highlight selected issue
  useEffect(() => {
    if (selectedIssue) {
      const overlay = highlightedElements.get(selectedIssue.node.element);
      if (overlay) {
        overlay.classList.add('a11y-issue-selected');
        // Scroll into view
        overlay.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
    } else {
      highlightedElements.forEach((overlay) => {
        overlay.classList.remove('a11y-issue-selected');
      });
    }
  }, [selectedIssue, highlightedElements]);

  return (
    <>
      <div className={`issue-highlighter-control ${className}`}>
        <div className="control-header">
          <h4>Issue Highlighting</h4>
          <div className="issue-counts">
            <span className="count critical">
              {issues.filter(i => i.severity === 'critical').length} Critical
            </span>
            <span className="count serious">
              {issues.filter(i => i.severity === 'serious').length} Serious
            </span>
            <span className="count moderate">
              {issues.filter(i => i.severity === 'moderate').length} Moderate
            </span>
            <span className="count minor">
              {issues.filter(i => i.severity === 'minor').length} Minor
            </span>
          </div>
        </div>

        <div className="issue-list">
          {issues.map((issue) => (
            <div
              key={issue.id}
              className={`issue-item ${selectedIssue?.id === issue.id ? 'selected' : ''} issue-${issue.severity}`}
              onClick={() => onIssueSelect(issue)}
            >
              <div className="issue-severity-indicator" />
              <div className="issue-content">
                <div className="issue-type">{issue.type.replace(/-/g, ' ')}</div>
                <div className="issue-description">{issue.description}</div>
              </div>
            </div>
          ))}
        </div>
      </div>

      <style>{`
        .a11y-issue-overlay {
          border: 2px solid;
          background: rgba(255, 0, 0, 0.1);
          box-shadow: 0 0 0 4px rgba(255, 0, 0, 0.1);
          transition: all 0.2s;
        }

        .a11y-issue-overlay:hover {
          background: rgba(255, 0, 0, 0.2);
        }

        .a11y-issue-overlay.a11y-issue-selected {
          background: rgba(33, 150, 243, 0.2);
          border-color: #2196f3;
          box-shadow: 0 0 0 4px rgba(33, 150, 243, 0.3);
        }

        .a11y-issue-critical {
          border-color: #dc3545;
        }

        .a11y-issue-serious {
          border-color: #fd7e14;
        }

        .a11y-issue-moderate {
          border-color: #ffc107;
        }

        .a11y-issue-minor {
          border-color: #17a2b8;
        }

        .a11y-issue-badge-critical {
          background: #dc3545;
        }

        .a11y-issue-badge-serious {
          background: #fd7e14;
        }

        .a11y-issue-badge-moderate {
          background: #ffc107;
        }

        .a11y-issue-badge-minor {
          background: #17a2b8;
        }

        .issue-highlighter-control {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
          max-height: 500px;
          display: flex;
          flex-direction: column;
        }

        .control-header {
          padding: 15px;
          border-bottom: 1px solid #e0e0e0;
          background: #f8f9fa;
        }

        .control-header h4 {
          margin: 0 0 10px 0;
          font-size: 14px;
          text-transform: uppercase;
          color: #666;
        }

        .issue-counts {
          display: flex;
          gap: 12px;
          flex-wrap: wrap;
        }

        .count {
          padding: 4px 10px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
        }

        .count.critical {
          background: #f8d7da;
          color: #721c24;
        }

        .count.serious {
          background: #ffe5d0;
          color: #8a4a00;
        }

        .count.moderate {
          background: #fff3cd;
          color: #856404;
        }

        .count.minor {
          background: #d1ecf1;
          color: #0c5460;
        }

        .issue-list {
          overflow-y: auto;
          flex: 1;
        }

        .issue-item {
          display: flex;
          gap: 10px;
          padding: 12px 15px;
          cursor: pointer;
          border-bottom: 1px solid #f0f0f0;
          transition: background 0.2s;
        }

        .issue-item:hover {
          background: #f8f9fa;
        }

        .issue-item.selected {
          background: #e3f2fd;
        }

        .issue-severity-indicator {
          width: 4px;
          border-radius: 2px;
          flex-shrink: 0;
        }

        .issue-item.issue-critical .issue-severity-indicator {
          background: #dc3545;
        }

        .issue-item.issue-serious .issue-severity-indicator {
          background: #fd7e14;
        }

        .issue-item.issue-moderate .issue-severity-indicator {
          background: #ffc107;
        }

        .issue-item.issue-minor .issue-severity-indicator {
          background: #17a2b8;
        }

        .issue-content {
          flex: 1;
          min-width: 0;
        }

        .issue-type {
          font-weight: 600;
          font-size: 12px;
          text-transform: capitalize;
          margin-bottom: 4px;
        }

        .issue-description {
          font-size: 12px;
          color: #666;
          line-height: 1.4;
        }
      `}</style>
    </>
  );
};
