/**
 * Structure Viewer Component
 * Displays document tag structure in a tree view
 */

import React, { useState } from 'react';
import type { TagStructure } from '../../types/index.js';

export interface StructureViewerProps {
  structure: TagStructure;
  className?: string;
}

export function StructureViewer({ structure, className = '' }: StructureViewerProps): JSX.Element {
  return (
    <div className={`structure-viewer ${className}`}>
      <div className="structure-viewer__header">
        <h3>Document Structure</h3>
        <p>Explore the semantic structure of your document</p>
      </div>
      <div className="structure-viewer__tree">
        <StructureNode node={structure} level={0} />
      </div>
    </div>
  );
}

interface StructureNodeProps {
  node: TagStructure;
  level: number;
}

function StructureNode({ node, level }: StructureNodeProps): JSX.Element {
  const [expanded, setExpanded] = useState(level < 2); // Expand first 2 levels by default
  const hasChildren = node.children && node.children.length > 0;

  return (
    <div className="structure-node" style={{ marginLeft: `${level * 20}px` }}>
      <div className="structure-node__header">
        {hasChildren && (
          <button
            className="structure-node__toggle"
            onClick={() => setExpanded(!expanded)}
            aria-expanded={expanded}
            aria-label={expanded ? 'Collapse' : 'Expand'}
          >
            {expanded ? '▼' : '▶'}
          </button>
        )}
        <div className="structure-node__info">
          <span className="structure-node__type">{node.type}</span>
          {node.role && node.role !== node.type && (
            <span className="structure-node__role">({node.role})</span>
          )}
          {node.title && (
            <span className="structure-node__title">"{node.title}"</span>
          )}
          {node.alt && (
            <span className="structure-node__alt" title="Alternative text">
              Alt: "{node.alt}"
            </span>
          )}
          {node.isArtifact && (
            <span className="structure-node__artifact">Artifact</span>
          )}
        </div>
      </div>

      {expanded && hasChildren && (
        <div className="structure-node__children">
          {node.children!.map((child, index) => (
            <StructureNode key={child.id || index} node={child} level={level + 1} />
          ))}
        </div>
      )}
    </div>
  );
}
