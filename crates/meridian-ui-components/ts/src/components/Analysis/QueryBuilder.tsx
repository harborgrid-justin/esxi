/**
 * QueryBuilder Component
 * Spatial and attribute query builder
 * @module @meridian/ui-components/Analysis
 */

import React, { useState } from 'react';
import { useLayers } from '../../hooks/useLayers';
import type { QueryFilter, SpatialRelation, GeometryType } from '../../types';

export interface QueryBuilderProps {
  /** Callback when query is executed */
  onQueryExecute?: (filter: QueryFilter) => void;
  /** Callback when panel is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Query builder for spatial and attribute queries
 */
export const QueryBuilder: React.FC<QueryBuilderProps> = ({
  onQueryExecute,
  onClose,
  className = '',
}) => {
  const { layers } = useLayers();
  const [layerId, setLayerId] = useState('');
  const [geometryType, setGeometryType] = useState<GeometryType | ''>('');
  const [spatialRelation, setSpatialRelation] = useState<SpatialRelation>('intersects');
  const [attributes, setAttributes] = useState<Array<{ key: string; value: string }>>([]);

  const handleAddAttribute = () => {
    setAttributes([...attributes, { key: '', value: '' }]);
  };

  const handleRemoveAttribute = (index: number) => {
    setAttributes(attributes.filter((_, i) => i !== index));
  };

  const handleAttributeChange = (
    index: number,
    field: 'key' | 'value',
    value: string
  ) => {
    const newAttributes = [...attributes];
    newAttributes[index][field] = value;
    setAttributes(newAttributes);
  };

  const handleExecuteQuery = () => {
    const filter: QueryFilter = {
      layerId: layerId || undefined,
      geometryType: geometryType || undefined,
      spatialRelation,
      attributes: attributes.reduce(
        (acc, attr) => {
          if (attr.key && attr.value) {
            acc[attr.key] = attr.value;
          }
          return acc;
        },
        {} as Record<string, unknown>
      ),
    };

    onQueryExecute?.(filter);
  };

  const handleClearQuery = () => {
    setLayerId('');
    setGeometryType('');
    setSpatialRelation('intersects');
    setAttributes([]);
  };

  return (
    <div
      className={`meridian-query-builder bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="region"
      aria-label="Query builder"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Query Builder</h3>
        {onClose && (
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
            aria-label="Close panel"
          >
            <svg
              className="w-5 h-5"
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
        )}
      </div>

      {/* Description */}
      <p className="text-sm text-gray-600 mb-4">
        Build queries to filter and find features based on spatial and attribute criteria.
      </p>

      <div className="space-y-4">
        {/* Layer filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Layer
          </label>
          <select
            value={layerId}
            onChange={(e) => setLayerId(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            aria-label="Select layer"
          >
            <option value="">All Layers</option>
            {layers.map((layer) => (
              <option key={layer.id} value={layer.id}>
                {layer.name}
              </option>
            ))}
          </select>
        </div>

        {/* Geometry type filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Geometry Type
          </label>
          <select
            value={geometryType}
            onChange={(e) => setGeometryType(e.target.value as GeometryType | '')}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            aria-label="Select geometry type"
          >
            <option value="">Any Type</option>
            <option value="Point">Point</option>
            <option value="LineString">Line</option>
            <option value="Polygon">Polygon</option>
            <option value="MultiPoint">Multi-Point</option>
            <option value="MultiLineString">Multi-Line</option>
            <option value="MultiPolygon">Multi-Polygon</option>
          </select>
        </div>

        {/* Spatial relation */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Spatial Relation
          </label>
          <select
            value={spatialRelation}
            onChange={(e) => setSpatialRelation(e.target.value as SpatialRelation)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            aria-label="Select spatial relation"
          >
            <option value="intersects">Intersects</option>
            <option value="contains">Contains</option>
            <option value="within">Within</option>
            <option value="touches">Touches</option>
            <option value="crosses">Crosses</option>
            <option value="overlaps">Overlaps</option>
            <option value="disjoint">Disjoint</option>
          </select>
        </div>

        {/* Attribute filters */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <label className="block text-sm font-medium text-gray-700">
              Attribute Filters
            </label>
            <button
              onClick={handleAddAttribute}
              className="text-sm text-blue-600 hover:text-blue-700 font-medium"
            >
              + Add Filter
            </button>
          </div>

          {attributes.length === 0 ? (
            <div className="text-sm text-gray-500 italic p-3 bg-gray-50 rounded-lg">
              No attribute filters added
            </div>
          ) : (
            <div className="space-y-2">
              {attributes.map((attr, index) => (
                <div key={index} className="flex gap-2">
                  <input
                    type="text"
                    value={attr.key}
                    onChange={(e) =>
                      handleAttributeChange(index, 'key', e.target.value)
                    }
                    placeholder="Attribute name"
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
                    aria-label={`Attribute name ${index + 1}`}
                  />
                  <input
                    type="text"
                    value={attr.value}
                    onChange={(e) =>
                      handleAttributeChange(index, 'value', e.target.value)
                    }
                    placeholder="Value"
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm"
                    aria-label={`Attribute value ${index + 1}`}
                  />
                  <button
                    onClick={() => handleRemoveAttribute(index)}
                    className="w-9 h-9 flex items-center justify-center text-red-600 hover:bg-red-50 rounded transition-colors"
                    aria-label="Remove filter"
                  >
                    <svg
                      className="w-5 h-5"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                      />
                    </svg>
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Query summary */}
        <div className="p-3 bg-gray-50 rounded-lg border border-gray-200">
          <div className="text-sm font-medium text-gray-700 mb-2">
            Query Summary
          </div>
          <div className="text-xs text-gray-600 space-y-1">
            <div>
              Layer:{' '}
              <span className="font-medium">
                {layerId
                  ? layers.find((l) => l.id === layerId)?.name
                  : 'All layers'}
              </span>
            </div>
            <div>
              Geometry:{' '}
              <span className="font-medium">
                {geometryType || 'Any type'}
              </span>
            </div>
            <div>
              Relation:{' '}
              <span className="font-medium capitalize">
                {spatialRelation}
              </span>
            </div>
            <div>
              Filters:{' '}
              <span className="font-medium">
                {attributes.filter((a) => a.key && a.value).length}
              </span>
            </div>
          </div>
        </div>

        {/* Actions */}
        <div className="space-y-2">
          <button
            onClick={handleExecuteQuery}
            className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Execute Query
          </button>
          <button
            onClick={handleClearQuery}
            className="w-full px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
          >
            Clear All
          </button>
        </div>

        {/* Help */}
        <div className="p-3 bg-blue-50 rounded-lg">
          <p className="text-xs text-blue-900">
            <strong>Spatial relations:</strong>
          </p>
          <ul className="text-xs text-blue-800 mt-1 ml-4 list-disc space-y-0.5">
            <li>Intersects: Features that overlap or touch</li>
            <li>Contains: Features completely inside</li>
            <li>Within: Features completely outside</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default QueryBuilder;
