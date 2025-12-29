/**
 * Test form field announcements
 */

import React from 'react';
import type { FormStructure, AccessibilityNode } from '../../types';

export interface FormTesterProps {
  formStructure: FormStructure;
  onNodeSelect: (node: AccessibilityNode) => void;
  className?: string;
}

export const FormTester: React.FC<FormTesterProps> = ({
  formStructure,
  onNodeSelect,
  className = '',
}) => {
  const { fields, issues, score } = formStructure;

  const getLabelMethodBadge = (method: string) => {
    const classes: Record<string, string> = {
      'label-element': 'good',
      'aria-label': 'good',
      'aria-labelledby': 'good',
      'title': 'warning',
      'placeholder': 'error',
      'none': 'error',
    };

    return classes[method] || 'warning';
  };

  return (
    <div className={`form-tester ${className}`}>
      <div className="tester-header">
        <h3>Form Accessibility</h3>
        <div className="score-badge" data-score={score >= 80 ? 'good' : score >= 60 ? 'fair' : 'poor'}>
          {score}/100
        </div>
      </div>

      {fields.length === 0 ? (
        <div className="empty-state">No form fields found on page</div>
      ) : (
        <>
          <div className="fields-list">
            <h4>Form Fields ({fields.length})</h4>
            {fields.map((field, index) => (
              <div key={index} className="field-item" onClick={() => onNodeSelect(field.node)}>
                <div className="field-header">
                  <div className="field-role">{field.node.role}</div>
                  <div className="field-label">{field.label || '(no label)'}</div>
                  <div className={`label-method badge-${getLabelMethodBadge(field.labelMethod)}`}>
                    {field.labelMethod.replace(/-/g, ' ')}
                  </div>
                </div>

                <div className="field-announcement">
                  <strong>Announcement:</strong> {field.announcement.text}
                </div>

                {field.instructions && (
                  <div className="field-detail">
                    <strong>Instructions:</strong> {field.instructions}
                  </div>
                )}

                {field.errorMessage && (
                  <div className="field-detail error">
                    <strong>Error:</strong> {field.errorMessage}
                  </div>
                )}

                {field.groupLabel && (
                  <div className="field-detail">
                    <strong>Group:</strong> {field.groupLabel}
                  </div>
                )}

                <div className="field-states">
                  {field.node.required && <span className="state-badge required">Required</span>}
                  {field.node.disabled && <span className="state-badge disabled">Disabled</span>}
                  {field.node.readonly && <span className="state-badge readonly">Readonly</span>}
                  {field.node.invalid && <span className="state-badge invalid">Invalid</span>}
                </div>
              </div>
            ))}
          </div>

          {issues.length > 0 && (
            <div className="issues-section">
              <h4>Issues ({issues.length})</h4>
              {issues.map((issue, index) => (
                <div key={index} className={`issue issue-${issue.severity}`} onClick={() => onNodeSelect(issue.field.node)}>
                  <div className="issue-header">
                    <span className="issue-type">{issue.type.replace(/-/g, ' ')}</span>
                    <span className="issue-severity">{issue.severity}</span>
                  </div>
                  <div className="issue-description">{issue.description}</div>
                  <div className="issue-remediation">
                    <strong>Fix:</strong> {issue.remediation}
                  </div>
                </div>
              ))}
            </div>
          )}
        </>
      )}

      <style>{`
        .form-tester {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
          padding: 20px;
        }

        .tester-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          padding-bottom: 10px;
          border-bottom: 2px solid #e0e0e0;
        }

        .tester-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .score-badge {
          padding: 8px 16px;
          border-radius: 4px;
          font-weight: bold;
        }

        .score-badge[data-score="good"] {
          background: #d4edda;
          color: #155724;
        }

        .score-badge[data-score="fair"] {
          background: #fff3cd;
          color: #856404;
        }

        .score-badge[data-score="poor"] {
          background: #f8d7da;
          color: #721c24;
        }

        .empty-state {
          text-align: center;
          padding: 40px 20px;
          color: #999;
        }

        .fields-list h4,
        .issues-section h4 {
          margin: 0 0 15px 0;
          font-size: 14px;
          color: #666;
          text-transform: uppercase;
        }

        .field-item {
          padding: 15px;
          margin-bottom: 12px;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .field-item:hover {
          background: #f8f9fa;
          border-color: #007bff;
        }

        .field-header {
          display: flex;
          align-items: center;
          gap: 10px;
          margin-bottom: 10px;
        }

        .field-role {
          font-weight: bold;
          color: #9c27b0;
          min-width: 100px;
        }

        .field-label {
          flex: 1;
          font-weight: 500;
        }

        .label-method {
          padding: 4px 10px;
          border-radius: 4px;
          font-size: 11px;
          font-weight: 500;
          text-transform: capitalize;
        }

        .badge-good {
          background: #d4edda;
          color: #155724;
        }

        .badge-warning {
          background: #fff3cd;
          color: #856404;
        }

        .badge-error {
          background: #f8d7da;
          color: #721c24;
        }

        .field-announcement {
          padding: 10px;
          background: #f0f7ff;
          border-left: 3px solid #2196f3;
          border-radius: 4px;
          margin-bottom: 8px;
          font-size: 13px;
        }

        .field-detail {
          font-size: 13px;
          margin: 8px 0;
          color: #666;
        }

        .field-detail.error {
          color: #dc3545;
        }

        .field-states {
          display: flex;
          gap: 6px;
          margin-top: 10px;
        }

        .state-badge {
          padding: 3px 8px;
          border-radius: 3px;
          font-size: 11px;
          font-weight: 500;
        }

        .state-badge.required {
          background: #fff3e0;
          color: #e65100;
        }

        .state-badge.disabled {
          background: #ffebee;
          color: #c62828;
        }

        .state-badge.readonly {
          background: #f5f5f5;
          color: #757575;
        }

        .state-badge.invalid {
          background: #ffebee;
          color: #d32f2f;
        }

        .issues-section {
          margin-top: 30px;
          padding-top: 20px;
          border-top: 1px solid #e0e0e0;
        }

        .issue {
          padding: 15px;
          margin-bottom: 12px;
          border-left: 4px solid;
          background: #f8f9fa;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .issue:hover {
          background: #f0f0f0;
        }

        .issue-critical {
          border-left-color: #dc3545;
        }

        .issue-serious {
          border-left-color: #fd7e14;
        }

        .issue-moderate {
          border-left-color: #ffc107;
        }

        .issue-minor {
          border-left-color: #17a2b8;
        }

        .issue-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 8px;
        }

        .issue-type {
          font-weight: 600;
          text-transform: capitalize;
        }

        .issue-severity {
          padding: 2px 8px;
          border-radius: 3px;
          font-size: 11px;
          font-weight: 500;
          background: rgba(0, 0, 0, 0.1);
        }

        .issue-description {
          margin: 8px 0;
          font-size: 13px;
        }

        .issue-remediation {
          margin-top: 8px;
          font-size: 13px;
          color: #666;
        }
      `}</style>
    </div>
  );
};
