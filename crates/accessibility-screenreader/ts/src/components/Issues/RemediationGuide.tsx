/**
 * Step-by-step remediation guide for accessibility issues
 */

import React from 'react';
import type { AccessibilityIssue } from '../../types';

export interface RemediationGuideProps {
  issues: AccessibilityIssue[];
  className?: string;
}

interface RemediationStep {
  title: string;
  description: string;
  codeExample?: string;
}

export const RemediationGuide: React.FC<RemediationGuideProps> = ({
  issues,
  className = '',
}) => {
  const getRemediationSteps = (issue: AccessibilityIssue): RemediationStep[] => {
    // Generate context-specific remediation steps
    const steps: RemediationStep[] = [];

    switch (issue.type) {
      case 'missing-label':
        steps.push({
          title: 'Add a label element',
          description: 'Associate a <label> element with the form field using the for attribute',
          codeExample: `<label for="username">Username:</label>\n<input type="text" id="username" name="username">`,
        });
        steps.push({
          title: 'Or use aria-label',
          description: 'Add an aria-label attribute directly to the element',
          codeExample: `<input type="text" aria-label="Username" name="username">`,
        });
        break;

      case 'missing-main':
        steps.push({
          title: 'Add a main landmark',
          description: 'Wrap the main content area with a <main> element',
          codeExample: `<main>\n  <!-- Main page content -->\n</main>`,
        });
        break;

      case 'skipped-level':
        steps.push({
          title: 'Fix heading hierarchy',
          description: 'Ensure headings increase by only one level at a time',
          codeExample: `<!-- Bad -->\n<h1>Title</h1>\n<h3>Subtitle</h3>\n\n<!-- Good -->\n<h1>Title</h1>\n<h2>Subtitle</h2>`,
        });
        break;

      case 'placeholder-as-label':
        steps.push({
          title: 'Replace placeholder with label',
          description: 'Use a visible label instead of relying on placeholder text',
          codeExample: `<label for="email">Email Address</label>\n<input type="email" id="email" placeholder="you@example.com">`,
        });
        break;

      default:
        steps.push({
          title: 'Review the issue',
          description: issue.remediation,
        });
    }

    return steps;
  };

  const prioritizedIssues = [...issues].sort((a, b) => {
    const severityOrder = { critical: 0, serious: 1, moderate: 2, minor: 3 };
    return severityOrder[a.severity] - severityOrder[b.severity];
  });

  return (
    <div className={`remediation-guide ${className}`}>
      <div className="guide-header">
        <h3>Remediation Guide</h3>
        <div className="issue-summary">
          {issues.length} issue{issues.length !== 1 ? 's' : ''} to fix
        </div>
      </div>

      {prioritizedIssues.length === 0 ? (
        <div className="no-issues">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <polyline points="22 4 12 14.01 9 11.01" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
          <h4>No Issues Found!</h4>
          <p>Your page has excellent screen reader compatibility.</p>
        </div>
      ) : (
        <div className="issues-list">
          {prioritizedIssues.map((issue, index) => {
            const steps = getRemediationSteps(issue);

            return (
              <div key={issue.id} className="remediation-item">
                <div className="item-header">
                  <div className="item-number">{index + 1}</div>
                  <div className="item-info">
                    <div className="item-title">{issue.type.replace(/-/g, ' ')}</div>
                    <div className={`item-severity severity-${issue.severity}`}>
                      {issue.severity}
                    </div>
                  </div>
                </div>

                <div className="item-description">
                  {issue.description}
                </div>

                <div className="remediation-steps">
                  <h5>Steps to Fix:</h5>
                  {steps.map((step, stepIndex) => (
                    <div key={stepIndex} className="step">
                      <div className="step-number">{stepIndex + 1}</div>
                      <div className="step-content">
                        <div className="step-title">{step.title}</div>
                        <div className="step-description">{step.description}</div>
                        {step.codeExample && (
                          <pre className="step-code">{step.codeExample}</pre>
                        )}
                      </div>
                    </div>
                  ))}
                </div>

                <div className="item-footer">
                  <div className="wcag-refs">
                    <strong>WCAG:</strong> {issue.wcagCriteria.join(', ')}
                  </div>
                  <div className="sr-refs">
                    <strong>Affects:</strong> {issue.screenReadersAffected.join(', ')}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}

      <style>{`
        .remediation-guide {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
        }

        .guide-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px;
          border-bottom: 2px solid #e0e0e0;
          background: #f8f9fa;
        }

        .guide-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .issue-summary {
          padding: 6px 12px;
          background: #e3f2fd;
          color: #1976d2;
          border-radius: 4px;
          font-size: 13px;
          font-weight: 500;
        }

        .no-issues {
          text-align: center;
          padding: 60px 20px;
          color: #28a745;
        }

        .no-issues svg {
          margin-bottom: 20px;
        }

        .no-issues h4 {
          margin: 0 0 10px 0;
          font-size: 20px;
          color: #333;
        }

        .no-issues p {
          margin: 0;
          color: #666;
        }

        .issues-list {
          padding: 20px;
        }

        .remediation-item {
          padding: 20px;
          margin-bottom: 20px;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          background: #fafafa;
        }

        .remediation-item:last-child {
          margin-bottom: 0;
        }

        .item-header {
          display: flex;
          gap: 15px;
          margin-bottom: 15px;
        }

        .item-number {
          width: 40px;
          height: 40px;
          border-radius: 50%;
          background: #007bff;
          color: white;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: bold;
          font-size: 18px;
          flex-shrink: 0;
        }

        .item-info {
          flex: 1;
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .item-title {
          font-size: 18px;
          font-weight: 600;
          text-transform: capitalize;
        }

        .item-severity {
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

        .item-description {
          padding: 15px;
          background: white;
          border-left: 4px solid #007bff;
          border-radius: 4px;
          margin-bottom: 20px;
        }

        .remediation-steps {
          margin-bottom: 20px;
        }

        .remediation-steps h5 {
          margin: 0 0 15px 0;
          font-size: 14px;
          color: #666;
          text-transform: uppercase;
        }

        .step {
          display: flex;
          gap: 15px;
          margin-bottom: 15px;
          padding: 15px;
          background: white;
          border-radius: 4px;
        }

        .step:last-child {
          margin-bottom: 0;
        }

        .step-number {
          width: 28px;
          height: 28px;
          border-radius: 50%;
          background: #e3f2fd;
          color: #1976d2;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: 600;
          font-size: 14px;
          flex-shrink: 0;
        }

        .step-content {
          flex: 1;
        }

        .step-title {
          font-weight: 600;
          margin-bottom: 6px;
        }

        .step-description {
          color: #666;
          margin-bottom: 10px;
          line-height: 1.5;
        }

        .step-code {
          padding: 12px;
          background: #f8f9fa;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          overflow-x: auto;
          font-family: 'Monaco', 'Menlo', monospace;
          font-size: 12px;
          line-height: 1.5;
          margin: 0;
        }

        .item-footer {
          display: flex;
          justify-content: space-between;
          padding-top: 15px;
          border-top: 1px solid #e0e0e0;
          font-size: 13px;
          color: #666;
        }

        .wcag-refs strong,
        .sr-refs strong {
          color: #333;
        }
      `}</style>
    </div>
  );
};
