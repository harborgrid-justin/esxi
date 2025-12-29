/**
 * MeasureTool Component
 * Distance and area measurement tool
 * @module @meridian/ui-components/Tools
 */

import React, { useState, useCallback } from 'react';
import { useMap } from '../../hooks/useMap';
import type { Coordinate, MeasurementType, MeasurementResult } from '../../types';

export interface MeasureToolProps {
  /** Initial measurement type */
  initialType?: MeasurementType;
  /** Callback when measurement is complete */
  onMeasurementComplete?: (result: MeasurementResult) => void;
  /** Callback when tool is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Tool for measuring distances and areas on the map
 */
export const MeasureTool: React.FC<MeasureToolProps> = ({
  initialType = 'distance',
  onMeasurementComplete,
  onClose,
  className = '',
}) => {
  const { setInteractionMode, setCursor } = useMap();
  const [measurementType, setMeasurementType] =
    useState<MeasurementType>(initialType);
  const [points, setPoints] = useState<Coordinate[]>([]);
  const [isActive, setIsActive] = useState(false);
  const [result, setResult] = useState<MeasurementResult | null>(null);

  /**
   * Start measurement
   */
  const startMeasurement = useCallback(() => {
    setIsActive(true);
    setPoints([]);
    setResult(null);
    setInteractionMode('measure');
    setCursor('crosshair');
  }, [setInteractionMode, setCursor]);

  /**
   * Stop measurement
   */
  const stopMeasurement = useCallback(() => {
    setIsActive(false);
    setInteractionMode('pan');
    setCursor('default');
  }, [setInteractionMode, setCursor]);

  /**
   * Add point to measurement
   */
  const addPoint = useCallback(
    (coord: Coordinate) => {
      const newPoints = [...points, coord];
      setPoints(newPoints);

      // Calculate measurement
      if (measurementType === 'distance' && newPoints.length >= 2) {
        const distance = calculateDistance(newPoints);
        const measureResult: MeasurementResult = {
          type: 'distance',
          value: distance,
          unit: 'kilometers',
        };
        setResult(measureResult);
        onMeasurementComplete?.(measureResult);
      } else if (measurementType === 'area' && newPoints.length >= 3) {
        const area = calculateArea(newPoints);
        const measureResult: MeasurementResult = {
          type: 'area',
          value: area,
          unit: 'square kilometers',
        };
        setResult(measureResult);
        onMeasurementComplete?.(measureResult);
      }
    },
    [points, measurementType, onMeasurementComplete]
  );

  /**
   * Clear measurement
   */
  const clearMeasurement = useCallback(() => {
    setPoints([]);
    setResult(null);
  }, []);

  /**
   * Finish measurement
   */
  const finishMeasurement = useCallback(() => {
    stopMeasurement();
  }, [stopMeasurement]);

  return (
    <div
      className={`meridian-measure-tool bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="region"
      aria-label="Measurement tool"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Measure</h3>
        {onClose && (
          <button
            onClick={onClose}
            className="w-8 h-8 flex items-center justify-center hover:bg-gray-100 rounded transition-colors"
            aria-label="Close tool"
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

      {/* Measurement type selector */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Measurement Type
        </label>
        <div className="grid grid-cols-2 gap-2">
          <button
            onClick={() => {
              setMeasurementType('distance');
              clearMeasurement();
            }}
            className={`px-4 py-2 rounded-lg border transition-colors ${
              measurementType === 'distance'
                ? 'bg-blue-600 text-white border-blue-600'
                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
            }`}
            aria-pressed={measurementType === 'distance'}
          >
            Distance
          </button>
          <button
            onClick={() => {
              setMeasurementType('area');
              clearMeasurement();
            }}
            className={`px-4 py-2 rounded-lg border transition-colors ${
              measurementType === 'area'
                ? 'bg-blue-600 text-white border-blue-600'
                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
            }`}
            aria-pressed={measurementType === 'area'}
          >
            Area
          </button>
        </div>
      </div>

      {/* Controls */}
      <div className="mb-4 space-y-2">
        {!isActive ? (
          <button
            onClick={startMeasurement}
            className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Start Measuring
          </button>
        ) : (
          <>
            <button
              onClick={finishMeasurement}
              className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
            >
              Finish
            </button>
            <button
              onClick={clearMeasurement}
              className="w-full px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
            >
              Clear
            </button>
          </>
        )}
      </div>

      {/* Instructions */}
      <div className="mb-4 p-3 bg-blue-50 rounded-lg">
        <p className="text-sm text-blue-900">
          {measurementType === 'distance'
            ? 'Click on the map to add points. Distance will be calculated along the line.'
            : 'Click on the map to add points. Area will be calculated when you have at least 3 points.'}
        </p>
      </div>

      {/* Results */}
      {result && (
        <div className="p-4 bg-green-50 rounded-lg border border-green-200">
          <div className="text-sm text-green-900 font-medium mb-1">
            {result.type === 'distance' ? 'Total Distance' : 'Total Area'}
          </div>
          <div className="text-2xl font-bold text-green-900">
            {formatMeasurement(result.value, result.unit)}
          </div>
          {points.length > 0 && (
            <div className="text-xs text-green-700 mt-2">
              {points.length} point{points.length !== 1 ? 's' : ''} added
            </div>
          )}
        </div>
      )}

      {/* Point list */}
      {points.length > 0 && (
        <div className="mt-4">
          <div className="text-sm font-medium text-gray-700 mb-2">
            Points ({points.length})
          </div>
          <div className="max-h-32 overflow-y-auto space-y-1">
            {points.map((point, index) => (
              <div
                key={index}
                className="text-xs text-gray-600 font-mono bg-gray-50 px-2 py-1 rounded"
              >
                {index + 1}. {point.lon.toFixed(6)}, {point.lat.toFixed(6)}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * Calculate distance between points using Haversine formula
 */
function calculateDistance(points: Coordinate[]): number {
  if (points.length < 2) return 0;

  const EARTH_RADIUS_KM = 6371;
  let totalDistance = 0;

  for (let i = 0; i < points.length - 1; i++) {
    const p1 = points[i];
    const p2 = points[i + 1];

    const lat1Rad = (p1.lat * Math.PI) / 180;
    const lat2Rad = (p2.lat * Math.PI) / 180;
    const deltaLat = ((p2.lat - p1.lat) * Math.PI) / 180;
    const deltaLon = ((p2.lon - p1.lon) * Math.PI) / 180;

    const a =
      Math.sin(deltaLat / 2) * Math.sin(deltaLat / 2) +
      Math.cos(lat1Rad) *
        Math.cos(lat2Rad) *
        Math.sin(deltaLon / 2) *
        Math.sin(deltaLon / 2);

    const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
    totalDistance += EARTH_RADIUS_KM * c;
  }

  return totalDistance;
}

/**
 * Calculate area using Shoelace formula (simplified)
 */
function calculateArea(points: Coordinate[]): number {
  if (points.length < 3) return 0;

  let area = 0;
  for (let i = 0; i < points.length; i++) {
    const j = (i + 1) % points.length;
    area += points[i].lon * points[j].lat;
    area -= points[j].lon * points[i].lat;
  }

  return Math.abs(area / 2) * 12345; // Simplified conversion to km²
}

/**
 * Format measurement value
 */
function formatMeasurement(value: number, unit: string): string {
  if (unit.includes('kilometer')) {
    if (value < 1) {
      return `${(value * 1000).toFixed(2)} m${unit.includes('square') ? '²' : ''}`;
    }
    return `${value.toFixed(2)} km${unit.includes('square') ? '²' : ''}`;
  }
  return `${value.toFixed(2)} ${unit}`;
}

export default MeasureTool;
