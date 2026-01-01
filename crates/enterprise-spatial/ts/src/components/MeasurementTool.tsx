/**
 * Measurement Tool Component
 * Measure distances and areas on the map
 */

import React, { useState } from 'react';
import { Position, Polygon } from '../types';
import { GeometryFactory } from '../geometry/GeometryFactory';

export interface MeasurementToolProps {
  onMeasure?: (result: MeasurementResult) => void;
}

export interface MeasurementResult {
  type: 'distance' | 'area' | 'perimeter';
  value: number;
  units: string;
  positions: Position[];
}

export const MeasurementTool: React.FC<MeasurementToolProps> = ({ onMeasure }) => {
  const [mode, setMode] = useState<'distance' | 'area'>('distance');
  const [positions, setPositions] = useState<Position[]>([]);
  const [units, setUnits] = useState<'metric' | 'imperial'>('metric');
  const [result, setResult] = useState<MeasurementResult | null>(null);

  const handleAddPoint = () => {
    // In a real implementation, this would be triggered by map clicks
    const newPoint: Position = [
      Math.random() * 360 - 180,
      Math.random() * 180 - 90,
    ];
    const newPositions = [...positions, newPoint];
    setPositions(newPositions);

    if (mode === 'distance' && newPositions.length >= 2) {
      calculateDistance(newPositions);
    } else if (mode === 'area' && newPositions.length >= 3) {
      calculateArea(newPositions);
    }
  };

  const calculateDistance = (points: Position[]) => {
    let totalDistance = 0;

    for (let i = 0; i < points.length - 1; i++) {
      totalDistance += GeometryFactory.haversineDistance(points[i], points[i + 1]);
    }

    const distanceUnits = units === 'metric' ? 'meters' : 'feet';
    const finalDistance = units === 'metric' ? totalDistance : totalDistance * 3.28084;

    const measureResult: MeasurementResult = {
      type: 'distance',
      value: finalDistance,
      units: distanceUnits,
      positions: points,
    };

    setResult(measureResult);
    onMeasure?.(measureResult);
  };

  const calculateArea = (points: Position[]) => {
    const closedPoints = [...points, points[0]];
    const polygon = GeometryFactory.createPolygon([closedPoints]);
    const area = GeometryFactory.getArea(polygon);

    const areaUnits = units === 'metric' ? 'sq meters' : 'sq feet';
    const finalArea = units === 'metric' ? area : area * 10.7639;

    const measureResult: MeasurementResult = {
      type: 'area',
      value: finalArea,
      units: areaUnits,
      positions: points,
    };

    setResult(measureResult);
    onMeasure?.(measureResult);
  };

  const handleClear = () => {
    setPositions([]);
    setResult(null);
  };

  const formatValue = (value: number): string => {
    if (value > 1000000) {
      return `${(value / 1000000).toFixed(2)} M`;
    } else if (value > 1000) {
      return `${(value / 1000).toFixed(2)} K`;
    }
    return value.toFixed(2);
  };

  return (
    <div className="measurement-tool">
      <div className="tool-header">
        <h3>Measurement Tool</h3>
      </div>

      <div className="tool-controls">
        <div className="control-group">
          <label>Mode:</label>
          <select value={mode} onChange={(e) => setMode(e.target.value as any)}>
            <option value="distance">Distance</option>
            <option value="area">Area</option>
          </select>
        </div>

        <div className="control-group">
          <label>Units:</label>
          <select value={units} onChange={(e) => setUnits(e.target.value as any)}>
            <option value="metric">Metric</option>
            <option value="imperial">Imperial</option>
          </select>
        </div>

        <div className="button-group">
          <button onClick={handleAddPoint}>Add Point</button>
          <button onClick={handleClear}>Clear</button>
        </div>
      </div>

      {positions.length > 0 && (
        <div className="points-list">
          <h4>Points ({positions.length})</h4>
          <div className="points">
            {positions.map((pos, index) => (
              <div key={index} className="point-item">
                <span className="point-number">{index + 1}</span>
                <span className="coordinates">
                  {pos[0].toFixed(6)}, {pos[1].toFixed(6)}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {result && (
        <div className="measurement-result">
          <div className="result-type">{result.type.toUpperCase()}</div>
          <div className="result-value">
            {formatValue(result.value)} {result.units}
          </div>
          {result.type === 'distance' && positions.length > 1 && (
            <div className="result-segments">
              {positions.map((pos, index) => {
                if (index === 0) return null;
                const segmentDist = GeometryFactory.haversineDistance(
                  positions[index - 1],
                  pos
                );
                const finalDist =
                  units === 'metric' ? segmentDist : segmentDist * 3.28084;
                return (
                  <div key={index} className="segment">
                    Segment {index}: {formatValue(finalDist)} {result.units}
                  </div>
                );
              })}
            </div>
          )}
        </div>
      )}

      <style jsx>{`
        .measurement-tool {
          width: 300px;
          background: white;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .tool-header {
          padding: 15px;
          border-bottom: 1px solid #ddd;
        }

        .tool-header h3 {
          margin: 0;
          font-size: 18px;
        }

        .tool-controls {
          padding: 15px;
          border-bottom: 1px solid #eee;
        }

        .control-group {
          margin-bottom: 10px;
        }

        .control-group label {
          display: block;
          margin-bottom: 5px;
          font-weight: bold;
          font-size: 13px;
        }

        .control-group select {
          width: 100%;
          padding: 8px;
          border: 1px solid #ddd;
          border-radius: 3px;
        }

        .button-group {
          display: flex;
          gap: 10px;
          margin-top: 15px;
        }

        .button-group button {
          flex: 1;
          padding: 10px;
          border: none;
          border-radius: 3px;
          cursor: pointer;
          font-weight: bold;
        }

        .button-group button:first-child {
          background: #007bff;
          color: white;
        }

        .button-group button:first-child:hover {
          background: #0056b3;
        }

        .button-group button:last-child {
          background: #6c757d;
          color: white;
        }

        .button-group button:last-child:hover {
          background: #545b62;
        }

        .points-list {
          padding: 15px;
          border-bottom: 1px solid #eee;
        }

        .points-list h4 {
          margin: 0 0 10px 0;
          font-size: 14px;
        }

        .points {
          max-height: 200px;
          overflow-y: auto;
        }

        .point-item {
          padding: 8px;
          background: #f9f9f9;
          margin-bottom: 5px;
          border-radius: 3px;
          display: flex;
          gap: 10px;
          font-size: 13px;
        }

        .point-number {
          background: #007bff;
          color: white;
          width: 24px;
          height: 24px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          font-weight: bold;
          font-size: 11px;
        }

        .coordinates {
          flex: 1;
          font-family: monospace;
        }

        .measurement-result {
          padding: 15px;
          background: #e3f2fd;
        }

        .result-type {
          font-size: 11px;
          text-transform: uppercase;
          color: #666;
          margin-bottom: 5px;
        }

        .result-value {
          font-size: 24px;
          font-weight: bold;
          color: #007bff;
          margin-bottom: 10px;
        }

        .result-segments {
          font-size: 12px;
        }

        .segment {
          padding: 5px;
          background: white;
          margin-bottom: 3px;
          border-radius: 2px;
        }
      `}</style>
    </div>
  );
};
