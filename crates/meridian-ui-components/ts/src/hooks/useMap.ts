/**
 * Map Management Hook
 * Provides map state and control methods
 * @module @meridian/ui-components/hooks
 */

import { useCallback, useEffect, useState } from 'react';
import { useMapContext, useMapStore } from '../context/MapContext';
import type { MapViewState, Coordinate, MapInteractionMode } from '../types';

/**
 * Hook for map management
 */
export const useMap = () => {
  const context = useMapContext();
  const viewState = useMapStore((state) => state.viewState);
  const interactionMode = useMapStore((state) => state.interactionMode);
  const cursor = useMapStore((state) => state.cursor);
  const loading = useMapStore((state) => state.loading);
  const error = useMapStore((state) => state.error);
  const setViewState = useMapStore((state) => state.setViewState);
  const setInteractionMode = useMapStore((state) => state.setInteractionMode);
  const setCursor = useMapStore((state) => state.setCursor);
  const setLoading = useMapStore((state) => state.setLoading);
  const setError = useMapStore((state) => state.setError);
  const reset = useMapStore((state) => state.reset);

  const [isReady, setIsReady] = useState(false);

  /**
   * Set map center
   */
  const setCenter = useCallback(
    (center: Coordinate) => {
      setViewState({ center });
    },
    [setViewState]
  );

  /**
   * Set map zoom level
   */
  const setZoom = useCallback(
    (zoom: number) => {
      setViewState({ zoom: Math.max(0, Math.min(22, zoom)) });
    },
    [setViewState]
  );

  /**
   * Zoom in by one level
   */
  const zoomIn = useCallback(() => {
    setViewState({ zoom: Math.min(viewState.zoom + 1, 22) });
  }, [viewState.zoom, setViewState]);

  /**
   * Zoom out by one level
   */
  const zoomOut = useCallback(() => {
    setViewState({ zoom: Math.max(viewState.zoom - 1, 0) });
  }, [viewState.zoom, setViewState]);

  /**
   * Set map rotation
   */
  const setRotation = useCallback(
    (rotation: number) => {
      setViewState({ rotation: rotation % 360 });
    },
    [setViewState]
  );

  /**
   * Reset rotation to north
   */
  const resetRotation = useCallback(() => {
    setViewState({ rotation: 0 });
  }, [setViewState]);

  /**
   * Set map pitch
   */
  const setPitch = useCallback(
    (pitch: number) => {
      setViewState({ pitch: Math.max(0, Math.min(60, pitch)) });
    },
    [setViewState]
  );

  /**
   * Fit map to bounds
   */
  const fitBounds = useCallback(
    (
      bounds: [number, number, number, number],
      options?: { padding?: number; duration?: number }
    ) => {
      context.fitBounds(bounds);
    },
    [context]
  );

  /**
   * Fly to location with animation
   */
  const flyTo = useCallback(
    (center: Coordinate, zoom: number, duration?: number) => {
      context.flyTo(center, zoom);
    },
    [context]
  );

  /**
   * Pan map by pixel offset
   */
  const panBy = useCallback(
    (dx: number, dy: number) => {
      // Implementation depends on map library
      // This is a simplified version
      const zoomFactor = Math.pow(2, viewState.zoom);
      const lonDelta = (dx / zoomFactor) * 0.01;
      const latDelta = (dy / zoomFactor) * 0.01;

      setViewState({
        center: {
          lon: viewState.center.lon + lonDelta,
          lat: viewState.center.lat - latDelta,
        },
      });
    },
    [viewState, setViewState]
  );

  /**
   * Get current bounds
   */
  const getBounds = useCallback((): [number, number, number, number] => {
    // Simplified bounds calculation
    const zoomFactor = Math.pow(2, -viewState.zoom);
    const extent = 180 * zoomFactor;

    return [
      viewState.center.lon - extent,
      viewState.center.lat - extent / 2,
      viewState.center.lon + extent,
      viewState.center.lat + extent / 2,
    ];
  }, [viewState]);

  /**
   * Convert screen coordinates to geographic coordinates
   */
  const screenToCoordinate = useCallback(
    (x: number, y: number): Coordinate => {
      // Simplified conversion (needs proper implementation with map library)
      const bounds = getBounds();
      const [minLon, minLat, maxLon, maxLat] = bounds;

      // Assuming map container is full viewport
      const width = window.innerWidth;
      const height = window.innerHeight;

      const lon = minLon + (x / width) * (maxLon - minLon);
      const lat = maxLat - (y / height) * (maxLat - minLat);

      return { lon, lat };
    },
    [getBounds]
  );

  /**
   * Convert geographic coordinates to screen coordinates
   */
  const coordinateToScreen = useCallback(
    (coord: Coordinate): [number, number] => {
      // Simplified conversion (needs proper implementation with map library)
      const bounds = getBounds();
      const [minLon, minLat, maxLon, maxLat] = bounds;

      const width = window.innerWidth;
      const height = window.innerHeight;

      const x = ((coord.lon - minLon) / (maxLon - minLon)) * width;
      const y = ((maxLat - coord.lat) / (maxLat - minLat)) * height;

      return [x, y];
    },
    [getBounds]
  );

  /**
   * Update full view state
   */
  const updateViewState = useCallback(
    (updates: Partial<MapViewState>) => {
      setViewState(updates);
    },
    [setViewState]
  );

  /**
   * Reset map to initial state
   */
  const resetMap = useCallback(() => {
    reset();
  }, [reset]);

  /**
   * Get map instance
   */
  const getMapInstance = useCallback(() => {
    return context.getMapInstance();
  }, [context]);

  /**
   * Mark map as ready
   */
  useEffect(() => {
    if (context.mapRef.current) {
      setIsReady(true);
    }
  }, [context.mapRef]);

  return {
    // State
    viewState,
    interactionMode,
    cursor,
    loading,
    error,
    isReady,
    mapRef: context.mapRef,

    // View control
    setCenter,
    setZoom,
    zoomIn,
    zoomOut,
    setRotation,
    resetRotation,
    setPitch,
    fitBounds,
    flyTo,
    panBy,
    updateViewState,

    // Interaction
    setInteractionMode,
    setCursor,

    // State management
    setLoading,
    setError,
    resetMap,

    // Utility
    getBounds,
    screenToCoordinate,
    coordinateToScreen,
    getMapInstance,
  };
};

export default useMap;
