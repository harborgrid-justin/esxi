import { useEffect, useState } from 'react';
import { useMapStore } from '@/stores/mapStore';
import { useFeatures } from '@/hooks/useFeatures';
import type { Feature, Geometry } from '@/types';

export function DrawingTools() {
  const { map, activeTool } = useMapStore();
  const [drawingFeatures, setDrawingFeatures] = useState<Feature[]>([]);
  const [currentDrawing, setCurrentDrawing] = useState<{
    type: 'Point' | 'LineString' | 'Polygon';
    coordinates: any[];
  } | null>(null);

  useEffect(() => {
    if (!map || !activeTool) return;

    let clickHandler: (e: maplibregl.MapMouseEvent) => void;

    if (activeTool === 'draw-point') {
      clickHandler = (e: maplibregl.MapMouseEvent) => {
        const { lng, lat } = e.lngLat;
        const point: Feature = {
          id: `point-${Date.now()}`,
          type: 'Feature',
          geometry: {
            type: 'Point',
            coordinates: [lng, lat],
          },
          properties: {
            createdAt: new Date().toISOString(),
          },
        };

        setDrawingFeatures((prev) => [...prev, point]);
        addFeatureToMap(point);
      };

      map.on('click', clickHandler);
    } else if (activeTool === 'draw-line') {
      const coordinates: [number, number][] = [];

      clickHandler = (e: maplibregl.MapMouseEvent) => {
        const { lng, lat } = e.lngLat;
        coordinates.push([lng, lat]);

        setCurrentDrawing({
          type: 'LineString',
          coordinates: [...coordinates],
        });

        if (coordinates.length >= 2) {
          updateDrawingLayer();
        }
      };

      map.on('click', clickHandler);

      // Finish drawing on double-click
      const dblClickHandler = () => {
        if (coordinates.length >= 2) {
          const line: Feature = {
            id: `line-${Date.now()}`,
            type: 'Feature',
            geometry: {
              type: 'LineString',
              coordinates,
            },
            properties: {
              createdAt: new Date().toISOString(),
            },
          };

          setDrawingFeatures((prev) => [...prev, line]);
          addFeatureToMap(line);
          coordinates.length = 0;
          setCurrentDrawing(null);
        }
      };

      map.on('dblclick', dblClickHandler);
    } else if (activeTool === 'draw-polygon') {
      const coordinates: [number, number][] = [];

      clickHandler = (e: maplibregl.MapMouseEvent) => {
        const { lng, lat } = e.lngLat;
        coordinates.push([lng, lat]);

        setCurrentDrawing({
          type: 'Polygon',
          coordinates: [[...coordinates]],
        });

        if (coordinates.length >= 3) {
          updateDrawingLayer();
        }
      };

      map.on('click', clickHandler);

      // Finish drawing on double-click
      const dblClickHandler = () => {
        if (coordinates.length >= 3) {
          // Close the polygon
          coordinates.push(coordinates[0]);

          const polygon: Feature = {
            id: `polygon-${Date.now()}`,
            type: 'Feature',
            geometry: {
              type: 'Polygon',
              coordinates: [coordinates],
            },
            properties: {
              createdAt: new Date().toISOString(),
            },
          };

          setDrawingFeatures((prev) => [...prev, polygon]);
          addFeatureToMap(polygon);
          coordinates.length = 0;
          setCurrentDrawing(null);
        }
      };

      map.on('dblclick', dblClickHandler);
    }

    return () => {
      if (clickHandler) {
        map.off('click', clickHandler);
      }
    };
  }, [map, activeTool]);

  const addFeatureToMap = (feature: Feature) => {
    if (!map) return;

    if (!map.getSource('drawn-features')) {
      map.addSource('drawn-features', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      });

      map.addLayer({
        id: 'drawn-features-fill',
        type: 'fill',
        source: 'drawn-features',
        paint: {
          'fill-color': '#0080ff',
          'fill-opacity': 0.4,
        },
        filter: ['==', '$type', 'Polygon'],
      });

      map.addLayer({
        id: 'drawn-features-line',
        type: 'line',
        source: 'drawn-features',
        paint: {
          'line-color': '#0080ff',
          'line-width': 2,
        },
      });

      map.addLayer({
        id: 'drawn-features-point',
        type: 'circle',
        source: 'drawn-features',
        paint: {
          'circle-radius': 6,
          'circle-color': '#0080ff',
        },
        filter: ['==', '$type', 'Point'],
      });
    }

    const source = map.getSource('drawn-features');
    if (source.type === 'geojson') {
      source.setData({
        type: 'FeatureCollection',
        features: [...drawingFeatures, feature],
      });
    }
  };

  const updateDrawingLayer = () => {
    if (!map || !currentDrawing) return;

    if (!map.getSource('current-drawing')) {
      map.addSource('current-drawing', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      });

      map.addLayer({
        id: 'current-drawing-line',
        type: 'line',
        source: 'current-drawing',
        paint: {
          'line-color': '#ff8000',
          'line-width': 2,
          'line-dasharray': [2, 2],
        },
      });
    }

    const feature: Feature = {
      id: 'temp',
      type: 'Feature',
      geometry: currentDrawing as Geometry,
      properties: {},
    };

    const source = map.getSource('current-drawing');
    if (source.type === 'geojson') {
      source.setData({
        type: 'FeatureCollection',
        features: [feature],
      });
    }
  };

  return null; // This is a headless component
}
