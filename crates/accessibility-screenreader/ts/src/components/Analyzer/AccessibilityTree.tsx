/**
 * Visual accessibility tree display
 */

import React, { useState } from 'react';
import type { AccessibilityNode } from '../../types';

export interface AccessibilityTreeProps {
  tree: AccessibilityNode;
  currentNode: AccessibilityNode | null;
  onNodeSelect: (node: AccessibilityNode) => void;
  className?: string;
}

export const AccessibilityTree: React.FC<AccessibilityTreeProps> = ({
  tree,
  currentNode,
  onNodeSelect,
  className = '',
}) => {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set([tree.id]));

  const toggleExpanded = (nodeId: string) => {
    setExpandedNodes(prev => {
      const next = new Set(prev);
      if (next.has(nodeId)) {
        next.delete(nodeId);
      } else {
        next.add(nodeId);
      }
      return next;
    });
  };

  const renderNode = (node: AccessibilityNode, level: number = 0): JSX.Element => {
    const isExpanded = expandedNodes.has(node.id);
    const isCurrent = currentNode?.id === node.id;
    const hasChildren = node.children.length > 0;

    return (
      <div key={node.id} className="tree-node-wrapper">
        <div
          className={`tree-node ${isCurrent ? 'current' : ''} ${node.hidden ? 'hidden' : ''}`}
          style={{ paddingLeft: `${level * 20}px` }}
          onClick={() => onNodeSelect(node)}
        >
          {hasChildren && (
            <button
              className="expand-button"
              onClick={(e) => {
                e.stopPropagation();
                toggleExpanded(node.id);
              }}
            >
              {isExpanded ? '▼' : '▶'}
            </button>
          )}
          {!hasChildren && <span className="expand-spacer" />}

          <span className="node-role">{node.role}</span>
          {node.name && <span className="node-name">"{node.name}"</span>}
          {node.level && <span className="node-level">L{node.level}</span>}

          <div className="node-badges">
            {node.focusable && <span className="badge focusable">focusable</span>}
            {node.disabled && <span className="badge disabled">disabled</span>}
            {node.hidden && <span className="badge hidden">hidden</span>}
            {node.required && <span className="badge required">required</span>}
            {node.invalid && <span className="badge invalid">invalid</span>}
          </div>
        </div>

        {hasChildren && isExpanded && (
          <div className="tree-children">
            {node.children.map(child => renderNode(child, level + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className={`accessibility-tree ${className}`}>
      <div className="tree-header">
        <h3>Accessibility Tree</h3>
        <div className="tree-actions">
          <button onClick={() => setExpandedNodes(new Set([tree.id]))}>
            Collapse All
          </button>
          <button onClick={() => {
            const allIds = new Set<string>();
            const collectIds = (node: AccessibilityNode) => {
              allIds.add(node.id);
              node.children.forEach(collectIds);
            };
            collectIds(tree);
            setExpandedNodes(allIds);
          }}>
            Expand All
          </button>
        </div>
      </div>

      <div className="tree-content">
        {renderNode(tree)}
      </div>

      <style>{`
        .accessibility-tree {
          border: 1px solid #e0e0e0;
          border-radius: 4px;
          background: white;
        }

        .tree-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 15px;
          border-bottom: 1px solid #e0e0e0;
          background: #f8f9fa;
        }

        .tree-header h3 {
          margin: 0;
          font-size: 16px;
          color: #333;
        }

        .tree-actions {
          display: flex;
          gap: 10px;
        }

        .tree-actions button {
          padding: 6px 12px;
          font-size: 12px;
          background: white;
          color: #007bff;
          border: 1px solid #007bff;
        }

        .tree-actions button:hover {
          background: #007bff;
          color: white;
        }

        .tree-content {
          padding: 10px;
          max-height: 600px;
          overflow-y: auto;
        }

        .tree-node {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 8px;
          cursor: pointer;
          border-radius: 4px;
          transition: background 0.2s;
          font-family: 'Monaco', 'Menlo', monospace;
          font-size: 13px;
        }

        .tree-node:hover {
          background: #f0f0f0;
        }

        .tree-node.current {
          background: #e3f2fd;
          border: 2px solid #2196f3;
        }

        .tree-node.hidden {
          opacity: 0.5;
        }

        .expand-button {
          background: none;
          border: none;
          padding: 0;
          width: 16px;
          height: 16px;
          cursor: pointer;
          color: #666;
          font-size: 10px;
        }

        .expand-spacer {
          width: 16px;
        }

        .node-role {
          color: #9c27b0;
          font-weight: 600;
        }

        .node-name {
          color: #333;
        }

        .node-level {
          color: #666;
          font-size: 11px;
        }

        .node-badges {
          display: flex;
          gap: 4px;
          margin-left: auto;
        }

        .badge {
          padding: 2px 6px;
          border-radius: 3px;
          font-size: 10px;
          font-weight: 500;
        }

        .badge.focusable {
          background: #e3f2fd;
          color: #1976d2;
        }

        .badge.disabled {
          background: #ffebee;
          color: #c62828;
        }

        .badge.hidden {
          background: #f5f5f5;
          color: #757575;
        }

        .badge.required {
          background: #fff3e0;
          color: #e65100;
        }

        .badge.invalid {
          background: #ffebee;
          color: #d32f2f;
        }
      `}</style>
    </div>
  );
};
