/**
 * Node Palette - Available workflow nodes for drag-and-drop
 */

import React from 'react';
import { ActionType } from '../../types';

export interface NodePaletteProps {
  onNodeSelect?: (nodeType: string, actionType?: ActionType) => void;
}

export const NodePalette: React.FC<NodePaletteProps> = ({ onNodeSelect }) => {
  const nodeCategories = [
    {
      name: 'Actions',
      nodes: [
        { type: 'action', label: 'HTTP Request', actionType: ActionType.HTTP, icon: '🌐' },
        { type: 'action', label: 'Email', actionType: ActionType.EMAIL, icon: '📧' },
        { type: 'action', label: 'Notification', actionType: ActionType.NOTIFICATION, icon: '🔔' },
        { type: 'action', label: 'Database', actionType: ActionType.DATABASE, icon: '💾' },
        { type: 'action', label: 'Transform', actionType: ActionType.TRANSFORM, icon: '🔄' },
        { type: 'action', label: 'Approval', actionType: ActionType.APPROVAL, icon: '✅' }
      ]
    },
    {
      name: 'Control Flow',
      nodes: [
        { type: 'condition', label: 'Condition', icon: '🔀' },
        { type: 'parallel', label: 'Parallel', icon: '⚡' },
        { type: 'loop', label: 'Loop', icon: '🔁' },
        { type: 'wait', label: 'Wait', icon: '⏸️' }
      ]
    },
    {
      name: 'Advanced',
      nodes: [
        { type: 'subworkflow', label: 'Subworkflow', icon: '📦' }
      ]
    }
  ];

  const handleDragStart = (event: React.DragEvent, node: any) => {
    event.dataTransfer.setData('application/reactflow', JSON.stringify(node));
    event.dataTransfer.effectAllowed = 'move';
  };

  return (
    <div style={styles.container}>
      <div style={styles.header}>
        <h3 style={styles.title}>Node Palette</h3>
      </div>

      {nodeCategories.map((category) => (
        <div key={category.name} style={styles.category}>
          <h4 style={styles.categoryTitle}>{category.name}</h4>

          <div style={styles.nodeList}>
            {category.nodes.map((node, index) => (
              <div
                key={`${node.type}-${index}`}
                draggable
                onDragStart={(e) => handleDragStart(e, node)}
                onClick={() => onNodeSelect?.(node.type, node.actionType)}
                style={styles.node}
              >
                <span style={styles.nodeIcon}>{node.icon}</span>
                <span style={styles.nodeLabel}>{node.label}</span>
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    width: '250px',
    height: '100%',
    backgroundColor: '#f5f5f5',
    borderRight: '1px solid #ddd',
    overflowY: 'auto'
  },
  header: {
    padding: '16px',
    borderBottom: '1px solid #ddd',
    backgroundColor: '#fff'
  },
  title: {
    margin: 0,
    fontSize: '16px',
    fontWeight: 600
  },
  category: {
    padding: '16px',
    borderBottom: '1px solid #ddd'
  },
  categoryTitle: {
    margin: '0 0 12px 0',
    fontSize: '14px',
    fontWeight: 500,
    color: '#666'
  },
  nodeList: {
    display: 'flex',
    flexDirection: 'column',
    gap: '8px'
  },
  node: {
    padding: '12px',
    backgroundColor: '#fff',
    border: '1px solid #ddd',
    borderRadius: '6px',
    cursor: 'grab',
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    transition: 'all 0.2s',
    userSelect: 'none'
  },
  nodeIcon: {
    fontSize: '20px'
  },
  nodeLabel: {
    fontSize: '14px',
    fontWeight: 500
  }
};
