/**
 * ARIA Tree Component
 * Visualizes the ARIA accessibility tree
 */

import React, { useState } from 'react';
import { ARIATreeNode } from '../../types';
import { buildARIATree } from '../../utils/ariaUtils';

export interface ARIATreeProps {
  target?: HTMLElement | Document;
}

const TreeNode: React.FC<{ node: ARIATreeNode; depth: number }> = ({ node, depth }) => {
  const [expanded, setExpanded] = useState(depth < 2);

  const hasChildren = node.children.length > 0;
  const indent = depth * 20;

  return (
    <div className="tree-node">
      <div className="tree-node__content" style={{ paddingLeft: `${indent}px` }}>
        {hasChildren && (
          <button
            className="tree-node__toggle"
            onClick={() => setExpanded(!expanded)}
            aria-expanded={expanded}
          >
            {expanded ? '▼' : '▶'}
          </button>
        )}

        <span className="tree-node__element">&lt;{node.element}&gt;</span>

        {node.role && <span className="tree-node__role">{node.role}</span>}

        {node.accessibleName && (
          <span className="tree-node__name" title={node.accessibleName}>
            "{node.accessibleName}"
          </span>
        )}

        {Object.keys(node.attributes).length > 0 && (
          <span className="tree-node__attr-count">
            {Object.keys(node.attributes).length} attributes
          </span>
        )}
      </div>

      {expanded && hasChildren && (
        <div className="tree-node__children">
          {node.children.map((child, index) => (
            <TreeNode key={index} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
};

export const ARIATree: React.FC<ARIATreeProps> = ({ target }) => {
  if (!target) {
    return <div className="empty-state">No target specified.</div>;
  }

  const tree = buildARIATree(target);

  return (
    <div className="aria-tree">
      <h3>Accessibility Tree</h3>
      <div className="tree-container">
        <TreeNode node={tree} depth={0} />
      </div>
    </div>
  );
};
