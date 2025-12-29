/**
 * PDF Tag Tree Viewer Component
 * Displays PDF tag structure in a hierarchical tree
 */

import React, { useState } from 'react';
import type { TagStructure } from '../../types/index.js';

export interface TagTreeViewerProps {
  tagTree: TagStructure;
  className?: string;
}

export function TagTreeViewer({ tagTree, className = '' }: TagTreeViewerProps): JSX.Element {
  return (
    <div className={`tag-tree-viewer ${className}`}>
      <h3>PDF Tag Tree</h3>
      <div className="tag-tree-viewer__tree">
        <TagNode node={tagTree} depth={0} />
      </div>
    </div>
  );
}

interface TagNodeProps {
  node: TagStructure;
  depth: number;
}

function TagNode({ node, depth }: TagNodeProps): JSX.Element {
  const [isExpanded, setIsExpanded] = useState(depth < 3);
  const hasChildren = node.children && node.children.length > 0;

  return (
    <div className="tag-node" style={{ paddingLeft: `${depth * 16}px` }}>
      <div className="tag-node__header">
        {hasChildren && (
          <button
            className="tag-node__toggle"
            onClick={() => setIsExpanded(!isExpanded)}
            aria-label={isExpanded ? 'Collapse' : 'Expand'}
          >
            {isExpanded ? '▼' : '▶'}
          </button>
        )}
        <span className="tag-node__type">&lt;{node.type}&gt;</span>
        {node.alt && <span className="tag-node__alt">Alt: {node.alt}</span>}
        {node.actualText && <span className="tag-node__text">Text: {node.actualText}</span>}
      </div>
      {isExpanded && hasChildren && (
        <div className="tag-node__children">
          {node.children!.map((child, i) => (
            <TagNode key={i} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}
