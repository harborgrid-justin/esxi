/**
 * Spatial Analyzer Component
 * Main UI for spatial analysis operations
 */

import React, { useState, useCallback } from 'react';
import { Feature, Geometry, BufferOptions, SimplificationOptions } from '../types';
import { BufferAnalysis } from '../geometry/BufferAnalysis';
import { SimplificationEngine } from '../geometry/SimplificationEngine';
import { OverlayAnalysis } from '../geometry/OverlayAnalysis';
import { ProximityAnalysis } from '../analysis/ProximityAnalysis';

export interface SpatialAnalyzerProps {
  features: Feature[];
  onResult?: (result: Feature[]) => void;
  onError?: (error: Error) => void;
}

export const SpatialAnalyzer: React.FC<SpatialAnalyzerProps> = ({
  features,
  onResult,
  onError,
}) => {
  const [operation, setOperation] = useState<string>('buffer');
  const [distance, setDistance] = useState<number>(100);
  const [units, setUnits] = useState<'meters' | 'kilometers' | 'feet' | 'miles'>('meters');
  const [tolerance, setTolerance] = useState<number>(10);
  const [selectedFeatures, setSelectedFeatures] = useState<Feature[]>([]);
  const [results, setResults] = useState<Feature[]>([]);
  const [loading, setLoading] = useState(false);

  const handleAnalysis = useCallback(async () => {
    if (selectedFeatures.length === 0) {
      onError?.(new Error('No features selected'));
      return;
    }

    setLoading(true);

    try {
      let resultFeatures: Feature[] = [];

      switch (operation) {
        case 'buffer':
          resultFeatures = selectedFeatures.map((feature) => {
            const buffered = BufferAnalysis.buffer(feature.geometry, {
              distance,
              units,
            });
            return {
              type: 'Feature' as const,
              geometry: buffered,
              properties: { ...feature.properties, buffered: true },
            };
          });
          break;

        case 'simplify':
          resultFeatures = selectedFeatures.map((feature) => {
            const simplified = SimplificationEngine.simplify(feature.geometry, {
              tolerance,
              highQuality: true,
            });
            return {
              type: 'Feature' as const,
              geometry: simplified,
              properties: { ...feature.properties, simplified: true },
            };
          });
          break;

        case 'union':
          if (selectedFeatures.length >= 2) {
            const unioned = OverlayAnalysis.mergePolygons(
              ...selectedFeatures.map((f) => f.geometry as any)
            );
            resultFeatures = [
              {
                type: 'Feature' as const,
                geometry: unioned,
                properties: { operation: 'union' },
              },
            ];
          }
          break;

        case 'nearest':
          if (selectedFeatures.length >= 1 && features.length > 1) {
            const nearest = ProximityAnalysis.nearest(
              selectedFeatures[0].geometry,
              features.filter((f) => !selectedFeatures.includes(f)),
              { limit: 5 }
            );
            resultFeatures = nearest;
          }
          break;

        default:
          throw new Error(`Unknown operation: ${operation}`);
      }

      setResults(resultFeatures);
      onResult?.(resultFeatures);
    } catch (error) {
      onError?.(error as Error);
    } finally {
      setLoading(false);
    }
  }, [operation, distance, units, tolerance, selectedFeatures, features, onResult, onError]);

  const toggleFeatureSelection = (feature: Feature) => {
    setSelectedFeatures((prev) =>
      prev.includes(feature)
        ? prev.filter((f) => f !== feature)
        : [...prev, feature]
    );
  };

  return (
    <div className="spatial-analyzer">
      <div className="analyzer-header">
        <h2>Spatial Analyzer</h2>
        <p>Select features and choose an analysis operation</p>
      </div>

      <div className="analyzer-controls">
        <div className="control-group">
          <label>Operation:</label>
          <select value={operation} onChange={(e) => setOperation(e.target.value)}>
            <option value="buffer">Buffer</option>
            <option value="simplify">Simplify</option>
            <option value="union">Union</option>
            <option value="intersect">Intersect</option>
            <option value="difference">Difference</option>
            <option value="nearest">Find Nearest</option>
            <option value="distance">Calculate Distance</option>
          </select>
        </div>

        {operation === 'buffer' && (
          <>
            <div className="control-group">
              <label>Distance:</label>
              <input
                type="number"
                value={distance}
                onChange={(e) => setDistance(parseFloat(e.target.value))}
              />
            </div>
            <div className="control-group">
              <label>Units:</label>
              <select value={units} onChange={(e) => setUnits(e.target.value as any)}>
                <option value="meters">Meters</option>
                <option value="kilometers">Kilometers</option>
                <option value="feet">Feet</option>
                <option value="miles">Miles</option>
              </select>
            </div>
          </>
        )}

        {operation === 'simplify' && (
          <div className="control-group">
            <label>Tolerance:</label>
            <input
              type="number"
              value={tolerance}
              onChange={(e) => setTolerance(parseFloat(e.target.value))}
            />
          </div>
        )}

        <button onClick={handleAnalysis} disabled={loading || selectedFeatures.length === 0}>
          {loading ? 'Processing...' : 'Run Analysis'}
        </button>
      </div>

      <div className="feature-list">
        <h3>Features ({features.length})</h3>
        <div className="features">
          {features.map((feature, index) => (
            <div
              key={index}
              className={`feature-item ${selectedFeatures.includes(feature) ? 'selected' : ''}`}
              onClick={() => toggleFeatureSelection(feature)}
            >
              <input
                type="checkbox"
                checked={selectedFeatures.includes(feature)}
                onChange={() => {}}
              />
              <span>
                {feature.properties?.name || `Feature ${index + 1}`} ({feature.geometry.type})
              </span>
            </div>
          ))}
        </div>
      </div>

      {results.length > 0 && (
        <div className="results">
          <h3>Results ({results.length})</h3>
          <div className="result-items">
            {results.map((result, index) => (
              <div key={index} className="result-item">
                <span>{result.geometry.type}</span>
                <pre>{JSON.stringify(result.properties, null, 2)}</pre>
              </div>
            ))}
          </div>
        </div>
      )}

      <style jsx>{`
        .spatial-analyzer {
          padding: 20px;
          max-width: 800px;
        }

        .analyzer-header {
          margin-bottom: 20px;
        }

        .analyzer-header h2 {
          margin: 0 0 5px 0;
          font-size: 24px;
        }

        .analyzer-header p {
          margin: 0;
          color: #666;
        }

        .analyzer-controls {
          background: #f5f5f5;
          padding: 15px;
          border-radius: 4px;
          margin-bottom: 20px;
        }

        .control-group {
          margin-bottom: 10px;
        }

        .control-group label {
          display: inline-block;
          width: 100px;
          font-weight: bold;
        }

        .control-group input,
        .control-group select {
          padding: 5px;
          border: 1px solid #ddd;
          border-radius: 3px;
        }

        button {
          background: #007bff;
          color: white;
          border: none;
          padding: 10px 20px;
          border-radius: 4px;
          cursor: pointer;
          margin-top: 10px;
        }

        button:hover:not(:disabled) {
          background: #0056b3;
        }

        button:disabled {
          background: #ccc;
          cursor: not-allowed;
        }

        .feature-list {
          margin-bottom: 20px;
        }

        .feature-list h3 {
          margin-bottom: 10px;
        }

        .features {
          max-height: 300px;
          overflow-y: auto;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .feature-item {
          padding: 10px;
          border-bottom: 1px solid #eee;
          cursor: pointer;
          display: flex;
          align-items: center;
        }

        .feature-item:hover {
          background: #f0f0f0;
        }

        .feature-item.selected {
          background: #e3f2fd;
        }

        .feature-item input {
          margin-right: 10px;
        }

        .results {
          border-top: 2px solid #ddd;
          padding-top: 20px;
        }

        .result-items {
          max-height: 400px;
          overflow-y: auto;
        }

        .result-item {
          background: #f9f9f9;
          padding: 10px;
          margin-bottom: 10px;
          border-radius: 4px;
        }

        .result-item pre {
          margin: 5px 0 0 0;
          font-size: 12px;
        }
      `}</style>
    </div>
  );
};
