/**
 * Property Inspector - Shape Property Editor
 * Edit properties of selected shapes
 */

import React from 'react';
import { Shape, ShapeStyle } from '../types';

export interface PropertyInspectorProps {
  selectedShapes: Shape[];
  onPropertyChange: (shapeId: string, property: string, value: any) => void;
}

export const PropertyInspector: React.FC<PropertyInspectorProps> = ({
  selectedShapes,
  onPropertyChange
}) => {
  if (selectedShapes.length === 0) {
    return (
      <div style={{ padding: 20, textAlign: 'center', color: '#999' }}>
        No shapes selected
      </div>
    );
  }

  const shape = selectedShapes[0];
  const style = shape.style;

  return (
    <div style={{ width: 250, background: '#f5f5f5', borderLeft: '1px solid #ccc', padding: 16 }}>
      <h3 style={{ margin: '0 0 16px 0', fontSize: 14 }}>
        Properties
        {selectedShapes.length > 1 && ` (${selectedShapes.length} selected)`}
      </h3>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        {/* Fill */}
        <div>
          <label style={{ display: 'block', fontSize: 12, marginBottom: 4, fontWeight: 600 }}>
            Fill
          </label>
          <input
            type="color"
            value={typeof style.fill === 'string' ? style.fill : '#000000'}
            onChange={(e) => onPropertyChange(shape.id, 'style.fill', e.target.value)}
            style={{ width: '100%', height: 32, border: '1px solid #ccc', borderRadius: 4 }}
          />
        </div>

        {/* Fill Opacity */}
        <div>
          <label style={{ display: 'block', fontSize: 12, marginBottom: 4, fontWeight: 600 }}>
            Fill Opacity: {((style.fillOpacity ?? 1) * 100).toFixed(0)}%
          </label>
          <input
            type="range"
            min="0"
            max="100"
            value={(style.fillOpacity ?? 1) * 100}
            onChange={(e) => onPropertyChange(shape.id, 'style.fillOpacity', parseInt(e.target.value) / 100)}
            style={{ width: '100%' }}
          />
        </div>

        {/* Stroke */}
        <div>
          <label style={{ display: 'block', fontSize: 12, marginBottom: 4, fontWeight: 600 }}>
            Stroke
          </label>
          <input
            type="color"
            value={typeof style.stroke === 'string' ? style.stroke : '#000000'}
            onChange={(e) => onPropertyChange(shape.id, 'style.stroke', e.target.value)}
            style={{ width: '100%', height: 32, border: '1px solid #ccc', borderRadius: 4 }}
          />
        </div>

        {/* Stroke Width */}
        <div>
          <label style={{ display: 'block', fontSize: 12, marginBottom: 4, fontWeight: 600 }}>
            Stroke Width: {style.strokeWidth ?? 1}px
          </label>
          <input
            type="range"
            min="0"
            max="20"
            step="0.5"
            value={style.strokeWidth ?? 1}
            onChange={(e) => onPropertyChange(shape.id, 'style.strokeWidth', parseFloat(e.target.value))}
            style={{ width: '100%' }}
          />
        </div>

        {/* Opacity */}
        <div>
          <label style={{ display: 'block', fontSize: 12, marginBottom: 4, fontWeight: 600 }}>
            Opacity: {((style.opacity ?? 1) * 100).toFixed(0)}%
          </label>
          <input
            type="range"
            min="0"
            max="100"
            value={(style.opacity ?? 1) * 100}
            onChange={(e) => onPropertyChange(shape.id, 'style.opacity', parseInt(e.target.value) / 100)}
            style={{ width: '100%' }}
          />
        </div>

        {/* Locked */}
        <div>
          <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12 }}>
            <input
              type="checkbox"
              checked={shape.locked}
              onChange={(e) => onPropertyChange(shape.id, 'locked', e.target.checked)}
            />
            Locked
          </label>
        </div>

        {/* Visible */}
        <div>
          <label style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12 }}>
            <input
              type="checkbox"
              checked={shape.visible}
              onChange={(e) => onPropertyChange(shape.id, 'visible', e.target.checked)}
            />
            Visible
          </label>
        </div>
      </div>
    </div>
  );
};
