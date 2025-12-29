/**
 * Reading order visualization
 */

import React from 'react';
import type { ReadingOrder, AccessibilityNode } from '../../types';

export interface ReadingOrderViewerProps {
  readingOrder: ReadingOrder;
  onNodeSelect: (node: AccessibilityNode) => void;
  className?: string;
}

export const ReadingOrderViewer: React.FC<ReadingOrderViewerProps> = ({
  readingOrder,
  onNodeSelect,
  className = '',
}) => {
  const { items, issues, score } = readingOrder;

  return (
    <div className={`reading-order-viewer ${className}`}>
      <div className="viewer-header">
        <h3>Reading Order Analysis</h3>
        <div className="score-badge" data-score={score >= 80 ? 'good' : score >= 60 ? 'fair' : 'poor'}>
          Score: {score}/100
        </div>
      </div>

      {issues.length > 0 && (
        <div className="issues-section">
          <h4>Issues Found</h4>
          {issues.map((issue, index) => (
            <div key={index} className={`issue issue-${issue.severity}`}>
              <div className="issue-header">
                <span className="issue-type">{issue.type.replace(/-/g, ' ')}</span>
                <span className="issue-severity">{issue.severity}</span>
              </div>
              <p className="issue-description">{issue.description}</p>
              <p className="issue-remediation"><strong>Fix:</strong> {issue.remediation}</p>
              <div className="affected-items">
                <strong>Affected elements ({issue.items.length}):</strong>
                <ul>
                  {issue.items.slice(0, 5).map((item, idx) => (
                    <li key={idx} onClick={() => onNodeSelect(item.node)}>
                      {item.node.name || '[unnamed]'} - Deviation: {item.deviation}
                    </li>
                  ))}
                  {issue.items.length > 5 && (
                    <li className="more">...and {issue.items.length - 5} more</li>
                  )}
                </ul>
              </div>
            </div>
          ))}
        </div>
      )}

      <div className="reading-sequence">
        <h4>Reading Sequence ({items.length} items)</h4>
        <div className="sequence-grid">
          {items.map((item, index) => (
            <div
              key={item.node.id}
              className={`sequence-item ${item.isOutOfOrder ? 'out-of-order' : ''}`}
              onClick={() => onNodeSelect(item.node)}
              title={`Visual: (${Math.round(item.visualPosition.x)}, ${Math.round(item.visualPosition.y)}), Deviation: ${item.deviation}`}
            >
              <div className="item-order">{index + 1}</div>
              <div className="item-content">
                <div className="item-role">{item.node.role}</div>
                <div className="item-name">{item.node.name || '[unnamed]'}</div>
              </div>
              {item.isOutOfOrder && (
                <div className="item-warning" title={`Deviation: ${item.deviation}`}>
                  ⚠
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      <style>{`
        .reading-order-viewer {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
          padding: 20px;
        }

        .viewer-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
          padding-bottom: 10px;
          border-bottom: 2px solid #e0e0e0;
        }

        .viewer-header h3 {
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

        .issues-section {
          margin-bottom: 30px;
        }

        .issues-section h4 {
          margin: 0 0 15px 0;
          font-size: 16px;
        }

        .issue {
          padding: 15px;
          border-left: 4px solid;
          margin-bottom: 15px;
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

        .issue-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 10px;
        }

        .issue-type {
          font-weight: 600;
          text-transform: capitalize;
        }

        .issue-severity {
          padding: 2px 8px;
          border-radius: 3px;
          font-size: 12px;
          font-weight: 500;
          background: rgba(0, 0, 0, 0.1);
        }

        .issue-description {
          margin: 10px 0;
        }

        .issue-remediation {
          margin: 10px 0;
          color: #666;
        }

        .affected-items ul {
          margin: 10px 0;
          padding-left: 20px;
        }

        .affected-items li {
          cursor: pointer;
          color: #007bff;
          margin: 5px 0;
        }

        .affected-items li:hover {
          text-decoration: underline;
        }

        .affected-items li.more {
          color: #666;
          cursor: default;
        }

        .reading-sequence h4 {
          margin: 0 0 15px 0;
          font-size: 16px;
        }

        .sequence-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 10px;
        }

        .sequence-item {
          display: flex;
          align-items: center;
          gap: 10px;
          padding: 10px;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .sequence-item:hover {
          background: #f0f0f0;
          border-color: #007bff;
        }

        .sequence-item.out-of-order {
          background: #fff3cd;
          border-color: #ffc107;
        }

        .item-order {
          font-size: 18px;
          font-weight: bold;
          color: #007bff;
          min-width: 30px;
        }

        .item-content {
          flex: 1;
          min-width: 0;
        }

        .item-role {
          font-size: 11px;
          color: #666;
          text-transform: uppercase;
        }

        .item-name {
          font-size: 13px;
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .item-warning {
          font-size: 20px;
          color: #ffc107;
        }
      `}</style>
    </div>
  );
};
