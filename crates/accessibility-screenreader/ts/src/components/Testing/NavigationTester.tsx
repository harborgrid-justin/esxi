/**
 * Test landmark and heading navigation
 */

import React from 'react';
import type { AccessibilityNode, LandmarkStructure, HeadingStructure } from '../../types';

export interface NavigationTesterProps {
  landmarks: LandmarkStructure;
  headings: HeadingStructure;
  onNodeSelect: (node: AccessibilityNode) => void;
  className?: string;
}

export const NavigationTester: React.FC<NavigationTesterProps> = ({
  landmarks,
  headings,
  onNodeSelect,
  className = '',
}) => {
  return (
    <div className={`navigation-tester ${className}`}>
      <div className="tester-section">
        <div className="section-header">
          <h3>Landmark Navigation</h3>
          <div className="score-badge" data-score={landmarks.score >= 80 ? 'good' : landmarks.score >= 60 ? 'fair' : 'poor'}>
            {landmarks.score}/100
          </div>
        </div>

        {landmarks.landmarks.length === 0 ? (
          <div className="empty-state">No landmarks found on page</div>
        ) : (
          <div className="landmark-list">
            {landmarks.landmarks.map((landmark, index) => (
              <div key={index} className="landmark-item" onClick={() => onNodeSelect(landmark.node)}>
                <div className="landmark-role">{landmark.role}</div>
                <div className="landmark-label">{landmark.label || '(no label)'}</div>
                {landmark.missingLabel && (
                  <span className="badge warning">Missing Label</span>
                )}
                {landmark.duplicateLabels && (
                  <span className="badge warning">Duplicate Label</span>
                )}
              </div>
            ))}
          </div>
        )}

        {landmarks.issues.length > 0 && (
          <div className="issues-list">
            <h4>Issues ({landmarks.issues.length})</h4>
            {landmarks.issues.map((issue, index) => (
              <div key={index} className={`issue issue-${issue.severity}`}>
                <div className="issue-type">{issue.type.replace(/-/g, ' ')}</div>
                <div className="issue-description">{issue.description}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="tester-section">
        <div className="section-header">
          <h3>Heading Navigation</h3>
          <div className="score-badge" data-score={headings.score >= 80 ? 'good' : headings.score >= 60 ? 'fair' : 'poor'}>
            {headings.score}/100
          </div>
        </div>

        {headings.headings.length === 0 ? (
          <div className="empty-state">No headings found on page</div>
        ) : (
          <div className="heading-list">
            {headings.headings.map((heading, index) => (
              <div
                key={index}
                className={`heading-item ${heading.skipped ? 'skipped' : ''} ${heading.empty ? 'empty' : ''}`}
                style={{ paddingLeft: `${(heading.level - 1) * 20}px` }}
                onClick={() => onNodeSelect(heading.node)}
              >
                <div className="heading-level">H{heading.level}</div>
                <div className="heading-text">{heading.text || '(empty)'}</div>
                {heading.skipped && (
                  <span className="badge warning">Skipped Level</span>
                )}
                {heading.empty && (
                  <span className="badge error">Empty</span>
                )}
              </div>
            ))}
          </div>
        )}

        {headings.issues.length > 0 && (
          <div className="issues-list">
            <h4>Issues ({headings.issues.length})</h4>
            {headings.issues.map((issue, index) => (
              <div key={index} className={`issue issue-${issue.severity}`}>
                <div className="issue-type">{issue.type.replace(/-/g, ' ')}</div>
                <div className="issue-description">{issue.description}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .navigation-tester {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 20px;
        }

        .tester-section {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
          padding: 20px;
        }

        .section-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 15px;
          padding-bottom: 10px;
          border-bottom: 2px solid #e0e0e0;
        }

        .section-header h3 {
          margin: 0;
          font-size: 16px;
        }

        .score-badge {
          padding: 4px 12px;
          border-radius: 4px;
          font-weight: bold;
          font-size: 14px;
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

        .landmark-list,
        .heading-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .landmark-item,
        .heading-item {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 10px;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .landmark-item:hover,
        .heading-item:hover {
          background: #f0f0f0;
          border-color: #007bff;
        }

        .heading-item.skipped {
          background: #fff3cd;
          border-color: #ffc107;
        }

        .heading-item.empty {
          background: #f8d7da;
          border-color: #dc3545;
        }

        .landmark-role,
        .heading-level {
          font-weight: bold;
          color: #9c27b0;
          min-width: 80px;
        }

        .landmark-label,
        .heading-text {
          flex: 1;
        }

        .badge {
          padding: 2px 8px;
          border-radius: 3px;
          font-size: 11px;
          font-weight: 500;
        }

        .badge.warning {
          background: #fff3cd;
          color: #856404;
        }

        .badge.error {
          background: #f8d7da;
          color: #721c24;
        }

        .issues-list {
          margin-top: 20px;
          padding-top: 20px;
          border-top: 1px solid #e0e0e0;
        }

        .issues-list h4 {
          margin: 0 0 10px 0;
          font-size: 14px;
          color: #666;
        }

        .issue {
          padding: 10px;
          margin-bottom: 8px;
          border-left: 3px solid;
          background: #f8f9fa;
          border-radius: 4px;
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

        .issue-type {
          font-weight: 600;
          font-size: 12px;
          text-transform: capitalize;
          margin-bottom: 4px;
        }

        .issue-description {
          font-size: 12px;
          color: #666;
        }
      `}</style>
    </div>
  );
};
