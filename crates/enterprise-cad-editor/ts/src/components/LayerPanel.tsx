/**
 * Layer Panel - Layer Management UI
 * Manage layers, visibility, locking, and ordering
 */

import React from 'react';
import { Layer } from '../types';

export interface LayerPanelProps {
  layers: Layer[];
  activeLayerId?: string;
  onLayerSelect?: (layerId: string) => void;
  onLayerToggleVisibility?: (layerId: string) => void;
  onLayerToggleLock?: (layerId: string) => void;
  onLayerAdd?: () => void;
  onLayerDelete?: (layerId: string) => void;
  onLayerReorder?: (fromIndex: number, toIndex: number) => void;
}

export const LayerPanel: React.FC<LayerPanelProps> = ({
  layers,
  activeLayerId,
  onLayerSelect,
  onLayerToggleVisibility,
  onLayerToggleLock,
  onLayerAdd,
  onLayerDelete,
  onLayerReorder
}) => {
  const sortedLayers = [...layers].sort((a, b) => b.order - a.order);

  return (
    <div style={{ width: 250, background: '#f5f5f5', borderLeft: '1px solid #ccc' }}>
      <div style={{ padding: 10, borderBottom: '1px solid #ccc', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h3 style={{ margin: 0, fontSize: 14 }}>Layers</h3>
        <button onClick={onLayerAdd} style={{ padding: '4px 8px', fontSize: 12 }}>
          + Add
        </button>
      </div>

      <div style={{ overflowY: 'auto', maxHeight: 'calc(100% - 50px)' }}>
        {sortedLayers.map((layer) => (
          <div
            key={layer.id}
            onClick={() => onLayerSelect?.(layer.id)}
            style={{
              padding: '8px 10px',
              borderBottom: '1px solid #e0e0e0',
              cursor: 'pointer',
              background: layer.id === activeLayerId ? '#e3f2fd' : 'white',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between'
            }}
          >
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', gap: 8 }}>
              {layer.color && (
                <div
                  style={{
                    width: 16,
                    height: 16,
                    background: layer.color,
                    border: '1px solid #ccc',
                    borderRadius: 2
                  }}
                />
              )}
              <span style={{ fontSize: 13, fontWeight: layer.id === activeLayerId ? 600 : 400 }}>
                {layer.name}
              </span>
            </div>

            <div style={{ display: 'flex', gap: 4 }}>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onLayerToggleVisibility?.(layer.id);
                }}
                style={{
                  padding: '2px 6px',
                  fontSize: 11,
                  background: 'transparent',
                  border: 'none',
                  cursor: 'pointer'
                }}
                title={layer.visible ? 'Hide' : 'Show'}
              >
                {layer.visible ? '👁' : '👁‍🗨'}
              </button>

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onLayerToggleLock?.(layer.id);
                }}
                style={{
                  padding: '2px 6px',
                  fontSize: 11,
                  background: 'transparent',
                  border: 'none',
                  cursor: 'pointer'
                }}
                title={layer.locked ? 'Unlock' : 'Lock'}
              >
                {layer.locked ? '🔒' : '🔓'}
              </button>

              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onLayerDelete?.(layer.id);
                }}
                style={{
                  padding: '2px 6px',
                  fontSize: 11,
                  background: 'transparent',
                  border: 'none',
                  cursor: 'pointer',
                  color: '#d32f2f'
                }}
                title="Delete"
              >
                ×
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
