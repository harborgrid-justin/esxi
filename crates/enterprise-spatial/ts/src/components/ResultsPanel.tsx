/**
 * Results Panel Component
 * Display analysis and query results
 */

import React, { useState } from 'react';
import { Feature } from '../types';

export interface ResultsPanelProps {
  results: Feature[];
  title?: string;
  onFeatureClick?: (feature: Feature) => void;
  onExport?: (format: 'geojson' | 'csv') => void;
}

export const ResultsPanel: React.FC<ResultsPanelProps> = ({
  results,
  title = 'Results',
  onFeatureClick,
  onExport,
}) => {
  const [viewMode, setViewMode] = useState<'table' | 'list' | 'json'>('table');
  const [selectedFeature, setSelectedFeature] = useState<Feature | null>(null);

  const handleFeatureClick = (feature: Feature) => {
    setSelectedFeature(feature);
    onFeatureClick?.(feature);
  };

  const getPropertyKeys = (): string[] => {
    if (results.length === 0) return [];
    const keys = new Set<string>();
    results.forEach((result) => {
      Object.keys(result.properties || {}).forEach((key) => keys.add(key));
    });
    return Array.from(keys);
  };

  return (
    <div className="results-panel">
      <div className="results-header">
        <div className="header-left">
          <h3>{title}</h3>
          <span className="result-count">
            {results.length} {results.length === 1 ? 'result' : 'results'}
          </span>
        </div>

        <div className="header-controls">
          <select value={viewMode} onChange={(e) => setViewMode(e.target.value as any)}>
            <option value="table">Table</option>
            <option value="list">List</option>
            <option value="json">JSON</option>
          </select>

          {onExport && (
            <div className="export-buttons">
              <button onClick={() => onExport('geojson')}>Export GeoJSON</button>
              <button onClick={() => onExport('csv')}>Export CSV</button>
            </div>
          )}
        </div>
      </div>

      <div className="results-content">
        {results.length === 0 ? (
          <div className="no-results">
            <p>No results to display</p>
          </div>
        ) : viewMode === 'table' ? (
          <div className="table-view">
            <table>
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Type</th>
                  {getPropertyKeys().map((key) => (
                    <th key={key}>{key}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {results.map((result, index) => (
                  <tr
                    key={index}
                    onClick={() => handleFeatureClick(result)}
                    className={selectedFeature === result ? 'selected' : ''}
                  >
                    <td>{result.id || index}</td>
                    <td>{result.geometry.type}</td>
                    {getPropertyKeys().map((key) => (
                      <td key={key}>
                        {typeof result.properties?.[key] === 'object'
                          ? JSON.stringify(result.properties[key])
                          : String(result.properties?.[key] || '')}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : viewMode === 'list' ? (
          <div className="list-view">
            {results.map((result, index) => (
              <div
                key={index}
                className={`list-item ${selectedFeature === result ? 'selected' : ''}`}
                onClick={() => handleFeatureClick(result)}
              >
                <div className="item-header">
                  <span className="item-id">#{result.id || index}</span>
                  <span className="item-type">{result.geometry.type}</span>
                </div>
                <div className="item-properties">
                  {Object.entries(result.properties || {}).map(([key, value]) => (
                    <div key={key} className="property">
                      <span className="prop-key">{key}:</span>
                      <span className="prop-value">
                        {typeof value === 'object' ? JSON.stringify(value) : String(value)}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="json-view">
            <pre>{JSON.stringify({ type: 'FeatureCollection', features: results }, null, 2)}</pre>
          </div>
        )}
      </div>

      <style jsx>{`
        .results-panel {
          width: 100%;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
          display: flex;
          flex-direction: column;
          max-height: 600px;
        }

        .results-header {
          padding: 15px;
          border-bottom: 1px solid #ddd;
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .header-left {
          display: flex;
          align-items: center;
          gap: 10px;
        }

        .header-left h3 {
          margin: 0;
          font-size: 18px;
        }

        .result-count {
          background: #007bff;
          color: white;
          padding: 2px 8px;
          border-radius: 12px;
          font-size: 12px;
        }

        .header-controls {
          display: flex;
          gap: 10px;
          align-items: center;
        }

        .header-controls select {
          padding: 5px 10px;
          border: 1px solid #ddd;
          border-radius: 3px;
        }

        .export-buttons {
          display: flex;
          gap: 5px;
        }

        .export-buttons button {
          padding: 5px 10px;
          background: #28a745;
          color: white;
          border: none;
          border-radius: 3px;
          cursor: pointer;
          font-size: 12px;
        }

        .export-buttons button:hover {
          background: #218838;
        }

        .results-content {
          flex: 1;
          overflow: auto;
        }

        .no-results {
          padding: 40px;
          text-align: center;
          color: #999;
        }

        .table-view {
          overflow-x: auto;
        }

        table {
          width: 100%;
          border-collapse: collapse;
        }

        th {
          background: #f5f5f5;
          padding: 10px;
          text-align: left;
          font-weight: bold;
          border-bottom: 2px solid #ddd;
          position: sticky;
          top: 0;
        }

        td {
          padding: 10px;
          border-bottom: 1px solid #eee;
        }

        tr:hover {
          background: #f9f9f9;
        }

        tr.selected {
          background: #e3f2fd;
        }

        tr {
          cursor: pointer;
        }

        .list-view {
          padding: 10px;
        }

        .list-item {
          background: #f9f9f9;
          padding: 15px;
          margin-bottom: 10px;
          border-radius: 4px;
          cursor: pointer;
        }

        .list-item:hover {
          background: #f0f0f0;
        }

        .list-item.selected {
          background: #e3f2fd;
          border: 2px solid #007bff;
        }

        .item-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 10px;
          font-weight: bold;
        }

        .item-id {
          color: #666;
        }

        .item-type {
          background: #007bff;
          color: white;
          padding: 2px 8px;
          border-radius: 3px;
          font-size: 11px;
        }

        .item-properties {
          font-size: 13px;
        }

        .property {
          margin-bottom: 5px;
        }

        .prop-key {
          font-weight: bold;
          margin-right: 5px;
        }

        .prop-value {
          color: #555;
        }

        .json-view {
          padding: 15px;
        }

        .json-view pre {
          margin: 0;
          font-size: 12px;
          line-height: 1.5;
        }
      `}</style>
    </div>
  );
};
