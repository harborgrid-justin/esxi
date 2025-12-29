/**
 * Detailed information about a specific accessibility issue
 */

import React from 'react';
import type { AccessibilityIssue } from '../../types';

export interface IssueDetailsProps {
  issue: AccessibilityIssue | null;
  onClose?: () => void;
  className?: string;
}

export const IssueDetails: React.FC<IssueDetailsProps> = ({
  issue,
  onClose,
  className = '',
}) => {
  if (!issue) {
    return (
      <div className={`issue-details empty ${className}`}>
        <p>Select an issue to see details</p>

        <style>{`
          .issue-details.empty {
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 40px;
            border: 1px solid #e0e0e0;
            border-radius: 4px;
            background: white;
            color: #999;
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className={`issue-details ${className}`}>
      <div className="details-header">
        <div className="header-content">
          <div className={`severity-badge severity-${issue.severity}`}>
            {issue.severity}
          </div>
          <h3>{issue.type.replace(/-/g, ' ')}</h3>
        </div>
        {onClose && (
          <button className="close-button" onClick={onClose} aria-label="Close">
            ×
          </button>
        )}
      </div>

      <div className="details-content">
        <section className="detail-section">
          <h4>Description</h4>
          <p>{issue.description}</p>
        </section>

        <section className="detail-section">
          <h4>How to Fix</h4>
          <p>{issue.remediation}</p>
        </section>

        {issue.codeExample && (
          <section className="detail-section">
            <h4>Current Code</h4>
            <pre className="code-block bad">{issue.codeExample}</pre>
          </section>
        )}

        {issue.fixedExample && (
          <section className="detail-section">
            <h4>Fixed Code</h4>
            <pre className="code-block good">{issue.fixedExample}</pre>
          </section>
        )}

        <section className="detail-section">
          <h4>WCAG Criteria</h4>
          <div className="wcag-list">
            {issue.wcagCriteria.map((criterion) => (
              <div key={criterion} className="wcag-item">
                <span className="wcag-number">{criterion}</span>
                <a
                  href={`https://www.w3.org/WAI/WCAG21/Understanding/${criterion.replace(/\./g, '')}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="wcag-link"
                >
                  View Guideline
                </a>
              </div>
            ))}
          </div>
        </section>

        <section className="detail-section">
          <h4>Screen Readers Affected</h4>
          <div className="sr-list">
            {issue.screenReadersAffected.map((sr) => (
              <span key={sr} className="sr-badge">
                {sr}
              </span>
            ))}
          </div>
        </section>

        <section className="detail-section">
          <h4>Element Information</h4>
          <div className="element-info">
            <div className="info-row">
              <label>Role:</label>
              <span className="role-value">{issue.node.role}</span>
            </div>
            {issue.node.name && (
              <div className="info-row">
                <label>Accessible Name:</label>
                <span>{issue.node.name}</span>
              </div>
            )}
            {issue.node.description && (
              <div className="info-row">
                <label>Description:</label>
                <span>{issue.node.description}</span>
              </div>
            )}
            <div className="info-row">
              <label>Tag Name:</label>
              <span>{issue.node.element.tagName.toLowerCase()}</span>
            </div>
            {issue.node.element.className && (
              <div className="info-row">
                <label>Class:</label>
                <span className="class-value">{issue.node.element.className}</span>
              </div>
            )}
            {issue.node.element.id && (
              <div className="info-row">
                <label>ID:</label>
                <span className="id-value">#{issue.node.element.id}</span>
              </div>
            )}
          </div>
        </section>
      </div>

      <style>{`
        .issue-details {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
          display: flex;
          flex-direction: column;
          max-height: 600px;
        }

        .details-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          padding: 20px;
          border-bottom: 2px solid #e0e0e0;
          background: #f8f9fa;
        }

        .header-content {
          flex: 1;
        }

        .header-content h3 {
          margin: 10px 0 0 0;
          font-size: 20px;
          text-transform: capitalize;
        }

        .severity-badge {
          display: inline-block;
          padding: 4px 12px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
          text-transform: uppercase;
        }

        .severity-critical {
          background: #f8d7da;
          color: #721c24;
        }

        .severity-serious {
          background: #ffe5d0;
          color: #8a4a00;
        }

        .severity-moderate {
          background: #fff3cd;
          color: #856404;
        }

        .severity-minor {
          background: #d1ecf1;
          color: #0c5460;
        }

        .close-button {
          background: none;
          border: none;
          font-size: 32px;
          line-height: 1;
          color: #666;
          cursor: pointer;
          padding: 0;
          width: 32px;
          height: 32px;
        }

        .close-button:hover {
          color: #333;
        }

        .details-content {
          overflow-y: auto;
          flex: 1;
          padding: 20px;
        }

        .detail-section {
          margin-bottom: 24px;
        }

        .detail-section:last-child {
          margin-bottom: 0;
        }

        .detail-section h4 {
          margin: 0 0 12px 0;
          font-size: 14px;
          color: #666;
          text-transform: uppercase;
        }

        .detail-section p {
          margin: 0;
          line-height: 1.6;
          color: #333;
        }

        .code-block {
          padding: 15px;
          border-radius: 4px;
          overflow-x: auto;
          font-family: 'Monaco', 'Menlo', monospace;
          font-size: 13px;
          line-height: 1.5;
          margin: 0;
        }

        .code-block.bad {
          background: #ffebee;
          border-left: 4px solid #dc3545;
          color: #c62828;
        }

        .code-block.good {
          background: #e8f5e9;
          border-left: 4px solid #28a745;
          color: #2e7d32;
        }

        .wcag-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .wcag-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 10px;
          background: #f8f9fa;
          border-radius: 4px;
        }

        .wcag-number {
          font-weight: 600;
          color: #007bff;
        }

        .wcag-link {
          color: #007bff;
          text-decoration: none;
          font-size: 13px;
        }

        .wcag-link:hover {
          text-decoration: underline;
        }

        .sr-list {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }

        .sr-badge {
          padding: 6px 12px;
          background: #e3f2fd;
          color: #1976d2;
          border-radius: 4px;
          font-size: 13px;
          font-weight: 500;
        }

        .element-info {
          background: #f8f9fa;
          padding: 15px;
          border-radius: 4px;
        }

        .info-row {
          display: flex;
          gap: 10px;
          margin-bottom: 8px;
        }

        .info-row:last-child {
          margin-bottom: 0;
        }

        .info-row label {
          font-weight: 600;
          color: #666;
          min-width: 140px;
        }

        .info-row span {
          flex: 1;
          color: #333;
        }

        .role-value {
          color: #9c27b0;
          font-weight: 600;
        }

        .class-value {
          font-family: monospace;
        }

        .id-value {
          font-family: monospace;
          color: #007bff;
        }
      `}</style>
    </div>
  );
};
