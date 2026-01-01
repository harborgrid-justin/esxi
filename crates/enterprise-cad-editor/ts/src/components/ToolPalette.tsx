/**
 * Tool Palette - Tool Selection UI
 * Select and configure CAD tools
 */

import React from 'react';
import { ToolType } from '../types';

export interface ToolPaletteProps {
  activeTool: ToolType;
  onToolSelect: (tool: ToolType) => void;
  orientation?: 'horizontal' | 'vertical';
}

const tools: Array<{ type: ToolType; icon: string; label: string }> = [
  { type: ToolType.Select, icon: '↖', label: 'Select' },
  { type: ToolType.Pen, icon: '✏', label: 'Pen' },
  { type: ToolType.Line, icon: '/', label: 'Line' },
  { type: ToolType.Rectangle, icon: '▭', label: 'Rectangle' },
  { type: ToolType.Circle, icon: '○', label: 'Circle' },
  { type: ToolType.Ellipse, icon: '⬭', label: 'Ellipse' },
  { type: ToolType.Polygon, icon: '⬡', label: 'Polygon' },
  { type: ToolType.Pan, icon: '✋', label: 'Pan' },
  { type: ToolType.Zoom, icon: '🔍', label: 'Zoom' },
  { type: ToolType.Measure, icon: '📏', label: 'Measure' },
  { type: ToolType.Dimension, icon: '📐', label: 'Dimension' },
  { type: ToolType.Boolean, icon: '∪', label: 'Boolean' }
];

export const ToolPalette: React.FC<ToolPaletteProps> = ({
  activeTool,
  onToolSelect,
  orientation = 'vertical'
}) => {
  const isVertical = orientation === 'vertical';

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: isVertical ? 'column' : 'row',
        gap: 4,
        padding: 8,
        background: '#f5f5f5',
        borderRight: isVertical ? '1px solid #ccc' : 'none',
        borderBottom: !isVertical ? '1px solid #ccc' : 'none'
      }}
    >
      {tools.map((tool) => (
        <button
          key={tool.type}
          onClick={() => onToolSelect(tool.type)}
          title={tool.label}
          style={{
            width: 40,
            height: 40,
            padding: 0,
            border: '1px solid #ccc',
            borderRadius: 4,
            background: activeTool === tool.type ? '#2196f3' : 'white',
            color: activeTool === tool.type ? 'white' : '#333',
            cursor: 'pointer',
            fontSize: 18,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            transition: 'all 0.2s'
          }}
          onMouseEnter={(e) => {
            if (activeTool !== tool.type) {
              e.currentTarget.style.background = '#e3f2fd';
            }
          }}
          onMouseLeave={(e) => {
            if (activeTool !== tool.type) {
              e.currentTarget.style.background = 'white';
            }
          }}
        >
          {tool.icon}
        </button>
      ))}
    </div>
  );
};
