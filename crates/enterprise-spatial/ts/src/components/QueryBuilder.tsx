/**
 * Query Builder Component
 * Build and execute spatial queries
 */

import React, { useState } from 'react';
import { SpatialQuery, SpatialRelationship } from '../types';

export interface QueryBuilderProps {
  onExecute?: (query: SpatialQuery) => void;
  availableFields?: string[];
}

export const QueryBuilder: React.FC<QueryBuilderProps> = ({
  onExecute,
  availableFields = [],
}) => {
  const [spatialRel, setSpatialRel] = useState<SpatialRelationship>('intersects');
  const [whereClause, setWhereClause] = useState('');
  const [selectedFields, setSelectedFields] = useState<string[]>([]);
  const [orderBy, setOrderBy] = useState<string[]>([]);
  const [limit, setLimit] = useState<number>(100);
  const [returnGeometry, setReturnGeometry] = useState(true);

  const handleExecute = () => {
    const query: SpatialQuery = {
      spatialRel,
      where: whereClause || undefined,
      fields: selectedFields.length > 0 ? selectedFields : undefined,
      orderBy: orderBy.length > 0 ? orderBy : undefined,
      limit,
      returnGeometry,
    };

    onExecute?.(query);
  };

  const toggleField = (field: string) => {
    setSelectedFields((prev) =>
      prev.includes(field) ? prev.filter((f) => f !== field) : [...prev, field]
    );
  };

  return (
    <div className="query-builder">
      <div className="query-header">
        <h3>Query Builder</h3>
      </div>

      <div className="query-section">
        <label>Spatial Relationship:</label>
        <select
          value={spatialRel}
          onChange={(e) => setSpatialRel(e.target.value as SpatialRelationship)}
        >
          <option value="intersects">Intersects</option>
          <option value="contains">Contains</option>
          <option value="within">Within</option>
          <option value="overlaps">Overlaps</option>
          <option value="touches">Touches</option>
          <option value="crosses">Crosses</option>
          <option value="disjoint">Disjoint</option>
          <option value="equals">Equals</option>
        </select>
      </div>

      <div className="query-section">
        <label>WHERE Clause:</label>
        <textarea
          placeholder="e.g., population > 100000"
          value={whereClause}
          onChange={(e) => setWhereClause(e.target.value)}
          rows={3}
        />
      </div>

      {availableFields.length > 0 && (
        <div className="query-section">
          <label>Select Fields:</label>
          <div className="field-list">
            {availableFields.map((field) => (
              <label key={field} className="field-item">
                <input
                  type="checkbox"
                  checked={selectedFields.includes(field)}
                  onChange={() => toggleField(field)}
                />
                {field}
              </label>
            ))}
          </div>
        </div>
      )}

      <div className="query-section">
        <label>Limit:</label>
        <input
          type="number"
          value={limit}
          onChange={(e) => setLimit(parseInt(e.target.value))}
          min="1"
        />
      </div>

      <div className="query-section">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={returnGeometry}
            onChange={(e) => setReturnGeometry(e.target.checked)}
          />
          Return Geometry
        </label>
      </div>

      <div className="query-actions">
        <button className="btn-execute" onClick={handleExecute}>
          Execute Query
        </button>
      </div>

      <style jsx>{`
        .query-builder {
          width: 350px;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .query-header {
          padding: 15px;
          border-bottom: 1px solid #ddd;
        }

        .query-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .query-section {
          padding: 15px;
          border-bottom: 1px solid #eee;
        }

        .query-section label {
          display: block;
          margin-bottom: 8px;
          font-weight: bold;
          font-size: 13px;
        }

        .query-section select,
        .query-section input[type="number"],
        .query-section textarea {
          width: 100%;
          padding: 8px;
          border: 1px solid #ddd;
          border-radius: 3px;
          font-size: 13px;
        }

        .query-section textarea {
          font-family: monospace;
          resize: vertical;
        }

        .field-list {
          max-height: 200px;
          overflow-y: auto;
          border: 1px solid #ddd;
          border-radius: 3px;
          padding: 8px;
        }

        .field-item {
          display: flex;
          align-items: center;
          padding: 5px;
          cursor: pointer;
          font-weight: normal;
        }

        .field-item:hover {
          background: #f0f0f0;
        }

        .field-item input {
          margin-right: 8px;
        }

        .checkbox-label {
          display: flex !important;
          align-items: center;
          cursor: pointer;
          font-weight: normal !important;
        }

        .checkbox-label input {
          margin-right: 8px;
        }

        .query-actions {
          padding: 15px;
        }

        .btn-execute {
          width: 100%;
          padding: 12px;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          font-size: 14px;
          font-weight: bold;
          cursor: pointer;
        }

        .btn-execute:hover {
          background: #0056b3;
        }
      `}</style>
    </div>
  );
};
