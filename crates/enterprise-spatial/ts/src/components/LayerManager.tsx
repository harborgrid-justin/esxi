/**
 * Layer Manager Component
 * Manage map layers and their visibility
 */

import React, { useState } from 'react';
import { Layer } from '../types';

export interface LayerManagerProps {
  layers: Layer[];
  onLayerToggle?: (layerId: string, visible: boolean) => void;
  onLayerReorder?: (layers: Layer[]) => void;
  onLayerRemove?: (layerId: string) => void;
  onLayerOpacityChange?: (layerId: string, opacity: number) => void;
}

export const LayerManager: React.FC<LayerManagerProps> = ({
  layers,
  onLayerToggle,
  onLayerReorder,
  onLayerRemove,
  onLayerOpacityChange,
}) => {
  const [expanded, setExpanded] = useState<Set<string>>(new Set());

  const toggleExpanded = (layerId: string) => {
    const newExpanded = new Set(expanded);
    if (newExpanded.has(layerId)) {
      newExpanded.delete(layerId);
    } else {
      newExpanded.add(layerId);
    }
    setExpanded(newExpanded);
  };

  const handleToggle = (layerId: string) => {
    const layer = layers.find((l) => l.id === layerId);
    if (layer && onLayerToggle) {
      onLayerToggle(layerId, !layer.visible);
    }
  };

  const handleOpacityChange = (layerId: string, opacity: number) => {
    onLayerOpacityChange?.(layerId, opacity);
  };

  const handleRemove = (layerId: string) => {
    if (confirm('Remove this layer?')) {
      onLayerRemove?.(layerId);
    }
  };

  return (
    <div className="layer-manager">
      <div className="layer-manager-header">
        <h3>Layers</h3>
        <span className="layer-count">{layers.length}</span>
      </div>

      <div className="layer-list">
        {layers.map((layer) => (
          <div key={layer.id} className="layer-item">
            <div className="layer-header">
              <input
                type="checkbox"
                checked={layer.visible}
                onChange={() => handleToggle(layer.id)}
              />
              <button
                className="expand-button"
                onClick={() => toggleExpanded(layer.id)}
              >
                {expanded.has(layer.id) ? '▼' : '▶'}
              </button>
              <span className="layer-name">{layer.name}</span>
              <span className="layer-type">{layer.type}</span>
              <button
                className="remove-button"
                onClick={() => handleRemove(layer.id)}
              >
                ×
              </button>
            </div>

            {expanded.has(layer.id) && (
              <div className="layer-details">
                <div className="layer-control">
                  <label>Opacity:</label>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.1"
                    value={layer.opacity}
                    onChange={(e) =>
                      handleOpacityChange(layer.id, parseFloat(e.target.value))
                    }
                  />
                  <span>{Math.round(layer.opacity * 100)}%</span>
                </div>

                {layer.bounds && (
                  <div className="layer-info">
                    <strong>Bounds:</strong>
                    <div className="bounds-info">
                      <div>X: [{layer.bounds.minX.toFixed(2)}, {layer.bounds.maxX.toFixed(2)}]</div>
                      <div>Y: [{layer.bounds.minY.toFixed(2)}, {layer.bounds.maxY.toFixed(2)}]</div>
                    </div>
                  </div>
                )}

                {layer.metadata && (
                  <div className="layer-info">
                    <strong>Metadata:</strong>
                    <pre>{JSON.stringify(layer.metadata, null, 2)}</pre>
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>

      <style jsx>{`
        .layer-manager {
          width: 300px;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .layer-manager-header {
          padding: 15px;
          border-bottom: 1px solid #ddd;
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .layer-manager-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .layer-count {
          background: #007bff;
          color: white;
          padding: 2px 8px;
          border-radius: 12px;
          font-size: 12px;
        }

        .layer-list {
          max-height: 500px;
          overflow-y: auto;
        }

        .layer-item {
          border-bottom: 1px solid #eee;
        }

        .layer-header {
          padding: 10px;
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .layer-header input[type="checkbox"] {
          cursor: pointer;
        }

        .expand-button {
          background: none;
          border: none;
          cursor: pointer;
          font-size: 12px;
          padding: 0;
          width: 20px;
        }

        .layer-name {
          flex: 1;
          font-weight: 500;
        }

        .layer-type {
          font-size: 11px;
          background: #f0f0f0;
          padding: 2px 6px;
          border-radius: 3px;
        }

        .remove-button {
          background: none;
          border: none;
          color: #dc3545;
          font-size: 20px;
          cursor: pointer;
          padding: 0;
          width: 20px;
        }

        .remove-button:hover {
          color: #a02020;
        }

        .layer-details {
          padding: 10px;
          background: #f9f9f9;
          border-top: 1px solid #eee;
        }

        .layer-control {
          margin-bottom: 10px;
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .layer-control label {
          font-size: 12px;
          font-weight: bold;
          min-width: 60px;
        }

        .layer-control input[type="range"] {
          flex: 1;
        }

        .layer-control span {
          font-size: 12px;
          min-width: 35px;
        }

        .layer-info {
          margin-bottom: 10px;
          font-size: 12px;
        }

        .layer-info strong {
          display: block;
          margin-bottom: 5px;
        }

        .bounds-info {
          padding-left: 10px;
        }

        .layer-info pre {
          margin: 5px 0 0 10px;
          font-size: 11px;
          max-height: 150px;
          overflow-y: auto;
        }
      `}</style>
    </div>
  );
};
