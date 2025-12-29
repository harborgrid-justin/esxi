/**
 * DrawTool Component
 * Drawing tools for points, lines, and polygons
 * @module @meridian/ui-components/Tools
 */

import React, { useState, useCallback } from 'react';
import { useMap } from '../../hooks/useMap';
import type { DrawingToolType, Coordinate, Feature } from '../../types';

export interface DrawToolProps {
  /** Initial drawing tool type */
  initialType?: DrawingToolType;
  /** Callback when drawing is complete */
  onDrawComplete?: (feature: Feature) => void;
  /** Callback when tool is closed */
  onClose?: () => void;
  /** Custom CSS class */
  className?: string;
}

/**
 * Tool for drawing geometric features on the map
 */
export const DrawTool: React.FC<DrawToolProps> = ({
  initialType = 'polygon',
  onDrawComplete,
  onClose,
  className = '',
}) => {
  const { setInteractionMode, setCursor } = useMap();
  const [drawType, setDrawType] = useState<DrawingToolType>(initialType);
  const [isActive, setIsActive] = useState(false);
  const [points, setPoints] = useState<Coordinate[]>([]);
  const [properties, setProperties] = useState<Record<string, string>>({
    name: '',
    description: '',
  });

  /**
   * Start drawing
   */
  const startDrawing = useCallback(() => {
    setIsActive(true);
    setPoints([]);
    setInteractionMode('draw');
    setCursor('crosshair');
  }, [setInteractionMode, setCursor]);

  /**
   * Stop drawing
   */
  const stopDrawing = useCallback(() => {
    setIsActive(false);
    setInteractionMode('pan');
    setCursor('default');
  }, [setInteractionMode, setCursor]);

  /**
   * Add point
   */
  const addPoint = useCallback(
    (coord: Coordinate) => {
      setPoints([...points, coord]);
    },
    [points]
  );

  /**
   * Remove last point
   */
  const removeLastPoint = useCallback(() => {
    setPoints(points.slice(0, -1));
  }, [points]);

  /**
   * Clear drawing
   */
  const clearDrawing = useCallback(() => {
    setPoints([]);
  }, []);

  /**
   * Finish drawing
   */
  const finishDrawing = useCallback(() => {
    if (points.length === 0) return;

    let feature: Feature | null = null;

    switch (drawType) {
      case 'point':
        if (points.length >= 1) {
          feature = {
            type: 'Feature',
            id: Date.now(),
            geometry: {
              type: 'Point',
              coordinates: [points[0].lon, points[0].lat],
            },
            properties: { ...properties },
          };
        }
        break;

      case 'line':
        if (points.length >= 2) {
          feature = {
            type: 'Feature',
            id: Date.now(),
            geometry: {
              type: 'LineString',
              coordinates: points.map((p) => [p.lon, p.lat]),
            },
            properties: { ...properties },
          };
        }
        break;

      case 'polygon':
        if (points.length >= 3) {
          const coords = points.map((p) => [p.lon, p.lat]);
          coords.push(coords[0]); // Close the polygon
          feature = {
            type: 'Feature',
            id: Date.now(),
            geometry: {
              type: 'Polygon',
              coordinates: [coords],
            },
            properties: { ...properties },
          };
        }
        break;

      case 'circle':
        if (points.length === 2) {
          // Create circle as polygon approximation
          const center = points[0];
          const edge = points[1];
          const radius = Math.sqrt(
            Math.pow(edge.lon - center.lon, 2) +
              Math.pow(edge.lat - center.lat, 2)
          );
          const circlePoints = generateCircle(center, radius, 32);
          feature = {
            type: 'Feature',
            id: Date.now(),
            geometry: {
              type: 'Polygon',
              coordinates: [circlePoints.map((p) => [p.lon, p.lat])],
            },
            properties: { ...properties, radius },
          };
        }
        break;

      case 'rectangle':
        if (points.length === 2) {
          const [p1, p2] = points;
          const coords = [
            [p1.lon, p1.lat],
            [p2.lon, p1.lat],
            [p2.lon, p2.lat],
            [p1.lon, p2.lat],
            [p1.lon, p1.lat],
          ];
          feature = {
            type: 'Feature',
            id: Date.now(),
            geometry: {
              type: 'Polygon',
              coordinates: [coords],
            },
            properties: { ...properties },
          };
        }
        break;
    }

    if (feature) {
      onDrawComplete?.(feature);
      clearDrawing();
      stopDrawing();
    }
  }, [points, drawType, properties, onDrawComplete, clearDrawing, stopDrawing]);

  const handlePropertyChange = (key: string, value: string) => {
    setProperties({ ...properties, [key]: value });
  };

  const getMinPoints = (): number => {
    switch (drawType) {
      case 'point':
        return 1;
      case 'line':
      case 'circle':
      case 'rectangle':
        return 2;
      case 'polygon':
        return 3;
      default:
        return 1;
    }
  };

  const canFinish = points.length >= getMinPoints();

  return (
    <div
      className={`meridian-draw-tool bg-white rounded-lg shadow-lg p-4 ${className}`}
      role="region"
      aria-label="Drawing tool"
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-semibold text-gray-900">Draw</h3>
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

      {/* Drawing type selector */}
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Draw Type
        </label>
        <div className="grid grid-cols-3 gap-2">
          {(['point', 'line', 'polygon', 'circle', 'rectangle'] as DrawingToolType[]).map(
            (type) => (
              <button
                key={type}
                onClick={() => {
                  setDrawType(type);
                  clearDrawing();
                }}
                disabled={isActive}
                className={`px-3 py-2 rounded-lg border text-sm transition-colors ${
                  drawType === type
                    ? 'bg-blue-600 text-white border-blue-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50 disabled:opacity-50'
                }`}
                aria-pressed={drawType === type}
              >
                {type.charAt(0).toUpperCase() + type.slice(1)}
              </button>
            )
          )}
        </div>
      </div>

      {/* Properties */}
      <div className="mb-4 space-y-3">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Name
          </label>
          <input
            type="text"
            value={properties.name}
            onChange={(e) => handlePropertyChange('name', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            placeholder="Feature name"
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Description
          </label>
          <textarea
            value={properties.description}
            onChange={(e) => handlePropertyChange('description', e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm"
            rows={2}
            placeholder="Feature description"
          />
        </div>
      </div>

      {/* Controls */}
      <div className="space-y-2">
        {!isActive ? (
          <button
            onClick={startDrawing}
            className="w-full px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            Start Drawing
          </button>
        ) : (
          <>
            <button
              onClick={finishDrawing}
              disabled={!canFinish}
              className="w-full px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Finish ({points.length}/{getMinPoints()})
            </button>
            {points.length > 0 && (
              <button
                onClick={removeLastPoint}
                className="w-full px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 transition-colors"
              >
                Undo Last Point
              </button>
            )}
            <button
              onClick={() => {
                clearDrawing();
                stopDrawing();
              }}
              className="w-full px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors"
            >
              Cancel
            </button>
          </>
        )}
      </div>

      {/* Instructions */}
      <div className="mt-4 p-3 bg-blue-50 rounded-lg">
        <p className="text-sm text-blue-900">
          {getInstructions(drawType, points.length)}
        </p>
      </div>

      {/* Point list */}
      {points.length > 0 && (
        <div className="mt-4">
          <div className="text-sm font-medium text-gray-700 mb-2">
            Points ({points.length})
          </div>
          <div className="max-h-24 overflow-y-auto space-y-1">
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
 * Generate circle points
 */
function generateCircle(
  center: Coordinate,
  radius: number,
  segments: number
): Coordinate[] {
  const points: Coordinate[] = [];
  for (let i = 0; i < segments; i++) {
    const angle = (i / segments) * 2 * Math.PI;
    points.push({
      lon: center.lon + radius * Math.cos(angle),
      lat: center.lat + radius * Math.sin(angle),
    });
  }
  points.push(points[0]); // Close the circle
  return points;
}

/**
 * Get instructions based on draw type
 */
function getInstructions(type: DrawingToolType, pointCount: number): string {
  switch (type) {
    case 'point':
      return 'Click on the map to place a point.';
    case 'line':
      return pointCount < 2
        ? 'Click to add points along the line. Need at least 2 points.'
        : 'Click to add more points or finish to complete the line.';
    case 'polygon':
      return pointCount < 3
        ? 'Click to add vertices. Need at least 3 points to form a polygon.'
        : 'Click to add more vertices or finish to complete the polygon.';
    case 'circle':
      return pointCount === 0
        ? 'Click for circle center.'
        : 'Click to set the circle radius.';
    case 'rectangle':
      return pointCount === 0
        ? 'Click for first corner.'
        : 'Click for opposite corner.';
    default:
      return 'Click on the map to draw.';
  }
}

export default DrawTool;
