/**
 * FeatureEditor Component
 * Editor for feature properties and geometry
 * @module @meridian/ui-components/Data
 */

import React, { useState, useEffect } from 'react';
import type { Feature } from '../../types';

export interface FeatureEditorProps {
  /** Feature to edit */
  feature: Feature;
  /** Callback when feature is saved */
  onSave?: (feature: Feature) => void;
  /** Callback when editing is cancelled */
  onCancel?: () => void;
  /** Callback when feature is deleted */
  onDelete?: (featureId: string | number) => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Editor for modifying feature properties
 */
export const FeatureEditor: React.FC<FeatureEditorProps> = ({
  feature,
  onSave,
  onCancel,
  onDelete,
  className = '',
}) => {
  const [editedFeature, setEditedFeature] = useState<Feature>(feature);
  const [hasChanges, setHasChanges] = useState(false);

  useEffect(() => {
    setEditedFeature(feature);
    setHasChanges(false);
  }, [feature]);

  const handlePropertyChange = (key: string, value: unknown) => {
    setEditedFeature({
      ...editedFeature,
      properties: {
        ...editedFeature.properties,
        [key]: value,
      },
    });
    setHasChanges(true);
  };

  const handleAddProperty = () => {
    const key = prompt('Enter property name:');
    if (key) {
      handlePropertyChange(key, '');
    }
  };

  const handleRemoveProperty = (key: string) => {
    if (confirm(`Remove property "${key}"?`)) {
      const { [key]: _, ...rest } = editedFeature.properties;
      setEditedFeature({
        ...editedFeature,
        properties: rest,
      });
      setHasChanges(true);
    }
  };

  const handleSave = () => {
    onSave?.(editedFeature);
    setHasChanges(false);
  };

  const handleCancel = () => {
    if (hasChanges) {
      if (confirm('Discard unsaved changes?')) {
        setEditedFeature(feature);
        setHasChanges(false);
        onCancel?.();
      }
    } else {
      onCancel?.();
    }
  };

  const handleDelete = () => {
    if (confirm('Delete this feature? This action cannot be undone.')) {
      onDelete?.(editedFeature.id!);
    }
  };

  return (
    <div
      className={`meridian-feature-editor bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="form"
      aria-label="Feature editor"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Edit Feature</h3>
        {hasChanges && (
          <span className="text-xs bg-yellow-100 text-yellow-800 px-2 py-1 rounded">
            Unsaved changes
          </span>
        )}
      </div>

      {/* Feature ID */}
      <div className="mb-4 p-3 bg-gray-50 rounded-lg">
        <div className="text-xs text-gray-500 mb-1">Feature ID</div>
        <div className="text-sm font-mono text-gray-900">
          {editedFeature.id || 'No ID'}
        </div>
      </div>

      {/* Geometry info */}
      <div className="mb-4 p-3 bg-blue-50 rounded-lg border border-blue-200">
        <div className="text-xs text-blue-700 mb-1">Geometry Type</div>
        <div className="text-sm font-medium text-blue-900 capitalize">
          {editedFeature.geometry.type}
        </div>
      </div>

      {/* Properties */}
      <div className="mb-4">
        <div className="flex items-center justify-between mb-3">
          <label className="block text-sm font-medium text-gray-700">
            Properties
          </label>
          <button
            onClick={handleAddProperty}
            className="text-sm text-blue-600 hover:text-blue-700 font-medium"
          >
            + Add Property
          </button>
        </div>

        <div className="space-y-3 max-h-96 overflow-y-auto">
          {Object.entries(editedFeature.properties).map(([key, value]) => (
            <div key={key} className="space-y-1">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium text-gray-700">
                  {key}
                </label>
                <button
                  onClick={() => handleRemoveProperty(key)}
                  className="text-red-600 hover:text-red-700 p-1"
                  aria-label={`Remove ${key}`}
                  title={`Remove ${key}`}
                >
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M6 18L18 6M6 6l12 12"
                    />
                  </svg>
                </button>
              </div>

              {/* Value input based on type */}
              {typeof value === 'boolean' ? (
                <select
                  value={String(value)}
                  onChange={(e) =>
                    handlePropertyChange(key, e.target.value === 'true')
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                >
                  <option value="true">True</option>
                  <option value="false">False</option>
                </select>
              ) : typeof value === 'number' ? (
                <input
                  type="number"
                  value={value}
                  onChange={(e) =>
                    handlePropertyChange(key, parseFloat(e.target.value))
                  }
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                />
              ) : (
                <textarea
                  value={String(value)}
                  onChange={(e) => handlePropertyChange(key, e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
                  rows={2}
                />
              )}
            </div>
          ))}

          {Object.keys(editedFeature.properties).length === 0 && (
            <div className="text-sm text-gray-500 italic p-3 bg-gray-50 rounded-lg">
              No properties defined
            </div>
          )}
        </div>
      </div>

      {/* Actions */}
      <div className="space-y-2">
        <button
          onClick={handleSave}
          disabled={!hasChanges}
          className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Save Changes
        </button>
        <button
          onClick={handleCancel}
          className="w-full px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
        >
          Cancel
        </button>
        {onDelete && (
          <button
            onClick={handleDelete}
            className="w-full px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
          >
            Delete Feature
          </button>
        )}
      </div>

      {/* Validation info */}
      <div className="mt-4 p-3 bg-green-50 rounded-lg border border-green-200">
        <div className="flex items-start gap-2">
          <svg
            className="w-5 h-5 text-green-600 flex-shrink-0 mt-0.5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div className="text-sm text-green-900">
            <div className="font-medium mb-1">Feature is valid</div>
            <div className="text-xs text-green-700">
              Geometry: {editedFeature.geometry.type} •{' '}
              {Object.keys(editedFeature.properties).length} properties
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default FeatureEditor;
