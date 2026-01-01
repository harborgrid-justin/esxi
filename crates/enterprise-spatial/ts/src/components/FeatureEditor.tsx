/**
 * Feature Editor Component
 * Edit feature geometries and properties
 */

import React, { useState } from 'react';
import { Feature, Geometry } from '../types';

export interface FeatureEditorProps {
  feature: Feature | null;
  onSave?: (feature: Feature) => void;
  onCancel?: () => void;
}

export const FeatureEditor: React.FC<FeatureEditorProps> = ({
  feature,
  onSave,
  onCancel,
}) => {
  const [editedFeature, setEditedFeature] = useState<Feature | null>(feature);
  const [propertyKey, setPropertyKey] = useState('');
  const [propertyValue, setPropertyValue] = useState('');

  if (!editedFeature) {
    return (
      <div className="feature-editor empty">
        <p>No feature selected</p>
      </div>
    );
  }

  const handlePropertyAdd = () => {
    if (!propertyKey) return;

    setEditedFeature({
      ...editedFeature,
      properties: {
        ...editedFeature.properties,
        [propertyKey]: propertyValue,
      },
    });

    setPropertyKey('');
    setPropertyValue('');
  };

  const handlePropertyRemove = (key: string) => {
    const newProperties = { ...editedFeature.properties };
    delete newProperties[key];

    setEditedFeature({
      ...editedFeature,
      properties: newProperties,
    });
  };

  const handleSave = () => {
    if (editedFeature && onSave) {
      onSave(editedFeature);
    }
  };

  return (
    <div className="feature-editor">
      <div className="editor-header">
        <h3>Feature Editor</h3>
        <span className="feature-id">ID: {editedFeature.id || 'New'}</span>
      </div>

      <div className="editor-section">
        <h4>Geometry</h4>
        <div className="geometry-info">
          <div className="info-row">
            <span className="label">Type:</span>
            <span className="value">{editedFeature.geometry.type}</span>
          </div>
          <div className="geometry-json">
            <pre>{JSON.stringify(editedFeature.geometry, null, 2)}</pre>
          </div>
        </div>
      </div>

      <div className="editor-section">
        <h4>Properties</h4>
        <div className="properties-list">
          {Object.entries(editedFeature.properties || {}).map(([key, value]) => (
            <div key={key} className="property-item">
              <span className="property-key">{key}:</span>
              <span className="property-value">
                {typeof value === 'object' ? JSON.stringify(value) : String(value)}
              </span>
              <button
                className="remove-property"
                onClick={() => handlePropertyRemove(key)}
              >
                ×
              </button>
            </div>
          ))}
        </div>

        <div className="add-property">
          <input
            type="text"
            placeholder="Property name"
            value={propertyKey}
            onChange={(e) => setPropertyKey(e.target.value)}
          />
          <input
            type="text"
            placeholder="Value"
            value={propertyValue}
            onChange={(e) => setPropertyValue(e.target.value)}
          />
          <button onClick={handlePropertyAdd}>Add</button>
        </div>
      </div>

      <div className="editor-actions">
        <button className="btn-primary" onClick={handleSave}>
          Save
        </button>
        <button className="btn-secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>

      <style jsx>{`
        .feature-editor {
          width: 400px;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .feature-editor.empty {
          padding: 40px;
          text-align: center;
          color: #999;
        }

        .editor-header {
          padding: 15px;
          border-bottom: 1px solid #ddd;
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .editor-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .feature-id {
          font-size: 12px;
          color: #666;
        }

        .editor-section {
          padding: 15px;
          border-bottom: 1px solid #eee;
        }

        .editor-section h4 {
          margin: 0 0 10px 0;
          font-size: 14px;
        }

        .geometry-info .info-row {
          display: flex;
          margin-bottom: 10px;
        }

        .info-row .label {
          font-weight: bold;
          min-width: 60px;
        }

        .geometry-json {
          background: #f9f9f9;
          padding: 10px;
          border-radius: 4px;
          max-height: 200px;
          overflow-y: auto;
        }

        .geometry-json pre {
          margin: 0;
          font-size: 12px;
        }

        .properties-list {
          max-height: 300px;
          overflow-y: auto;
          margin-bottom: 10px;
        }

        .property-item {
          display: flex;
          align-items: center;
          padding: 8px;
          background: #f9f9f9;
          margin-bottom: 5px;
          border-radius: 3px;
        }

        .property-key {
          font-weight: bold;
          min-width: 100px;
          font-size: 13px;
        }

        .property-value {
          flex: 1;
          font-size: 13px;
          word-break: break-all;
        }

        .remove-property {
          background: none;
          border: none;
          color: #dc3545;
          font-size: 20px;
          cursor: pointer;
          padding: 0;
          width: 25px;
        }

        .add-property {
          display: flex;
          gap: 5px;
        }

        .add-property input {
          flex: 1;
          padding: 8px;
          border: 1px solid #ddd;
          border-radius: 3px;
          font-size: 13px;
        }

        .add-property button {
          padding: 8px 15px;
          background: #28a745;
          color: white;
          border: none;
          border-radius: 3px;
          cursor: pointer;
        }

        .add-property button:hover {
          background: #218838;
        }

        .editor-actions {
          padding: 15px;
          display: flex;
          gap: 10px;
        }

        .editor-actions button {
          flex: 1;
          padding: 10px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: bold;
        }

        .btn-primary {
          background: #007bff;
          color: white;
        }

        .btn-primary:hover {
          background: #0056b3;
        }

        .btn-secondary {
          background: #6c757d;
          color: white;
        }

        .btn-secondary:hover {
          background: #545b62;
        }
      `}</style>
    </div>
  );
};
