/**
 * Map Context Provider
 * Provides map state and functionality to child components
 * @module @meridian/ui-components/context
 */

import React, { createContext, useContext, useCallback, useRef, useEffect } from 'react';
import { create } from 'zustand';
import type {
  MapViewState,
  MapInteractionMode,
  Feature,
  LayerConfig,
  MapEvent,
  MapEventHandlers,
} from '../types';

// ============================================================================
// Store Definitions
// ============================================================================

/**
 * Map store using Zustand
 */
interface MapStoreState {
  viewState: MapViewState;
  interactionMode: MapInteractionMode;
  selectedFeatures: Feature[];
  hoveredFeature: Feature | null;
  cursor: string;
  loading: boolean;
  error: string | null;

  // Actions
  setViewState: (viewState: Partial<MapViewState>) => void;
  setInteractionMode: (mode: MapInteractionMode) => void;
  selectFeature: (feature: Feature, multiSelect?: boolean) => void;
  deselectFeature: (featureId: string | number) => void;
  clearSelection: () => void;
  setHoveredFeature: (feature: Feature | null) => void;
  setCursor: (cursor: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const useMapStore = create<MapStoreState>((set) => ({
  viewState: {
    center: { lon: 0, lat: 0 },
    zoom: 2,
    rotation: 0,
    pitch: 0,
  },
  interactionMode: 'pan',
  selectedFeatures: [],
  hoveredFeature: null,
  cursor: 'default',
  loading: false,
  error: null,

  setViewState: (updates) =>
    set((state) => ({
      viewState: { ...state.viewState, ...updates },
    })),

  setInteractionMode: (mode) =>
    set({ interactionMode: mode }),

  selectFeature: (feature, multiSelect = false) =>
    set((state) => {
      if (multiSelect) {
        const exists = state.selectedFeatures.some((f) => f.id === feature.id);
        if (exists) {
          return {
            selectedFeatures: state.selectedFeatures.filter(
              (f) => f.id !== feature.id
            ),
          };
        }
        return {
          selectedFeatures: [...state.selectedFeatures, feature],
        };
      }
      return { selectedFeatures: [feature] };
    }),

  deselectFeature: (featureId) =>
    set((state) => ({
      selectedFeatures: state.selectedFeatures.filter(
        (f) => f.id !== featureId
      ),
    })),

  clearSelection: () => set({ selectedFeatures: [] }),

  setHoveredFeature: (feature) => set({ hoveredFeature: feature }),

  setCursor: (cursor) => set({ cursor }),

  setLoading: (loading) => set({ loading }),

  setError: (error) => set({ error }),

  reset: () =>
    set({
      viewState: {
        center: { lon: 0, lat: 0 },
        zoom: 2,
        rotation: 0,
        pitch: 0,
      },
      interactionMode: 'pan',
      selectedFeatures: [],
      hoveredFeature: null,
      cursor: 'default',
      loading: false,
      error: null,
    }),
}));

/**
 * Layer store using Zustand
 */
interface LayerStoreState {
  layers: LayerConfig[];
  activeLayerId: string | null;

  // Actions
  addLayer: (layer: LayerConfig) => void;
  removeLayer: (layerId: string) => void;
  updateLayer: (layerId: string, updates: Partial<LayerConfig>) => void;
  toggleLayerVisibility: (layerId: string) => void;
  setActiveLayer: (layerId: string | null) => void;
  reorderLayers: (layerIds: string[]) => void;
  clearLayers: () => void;
}

const useLayerStore = create<LayerStoreState>((set) => ({
  layers: [],
  activeLayerId: null,

  addLayer: (layer) =>
    set((state) => ({
      layers: [...state.layers, layer],
    })),

  removeLayer: (layerId) =>
    set((state) => ({
      layers: state.layers.filter((l) => l.id !== layerId),
      activeLayerId:
        state.activeLayerId === layerId ? null : state.activeLayerId,
    })),

  updateLayer: (layerId, updates) =>
    set((state) => ({
      layers: state.layers.map((l) =>
        l.id === layerId ? { ...l, ...updates } : l
      ),
    })),

  toggleLayerVisibility: (layerId) =>
    set((state) => ({
      layers: state.layers.map((l) =>
        l.id === layerId ? { ...l, visible: !l.visible } : l
      ),
    })),

  setActiveLayer: (layerId) => set({ activeLayerId: layerId }),

  reorderLayers: (layerIds) =>
    set((state) => {
      const layerMap = new Map(state.layers.map((l) => [l.id, l]));
      return {
        layers: layerIds.map((id) => layerMap.get(id)!).filter(Boolean),
      };
    }),

  clearLayers: () => set({ layers: [], activeLayerId: null }),
}));

// ============================================================================
// Context Definition
// ============================================================================

interface MapContextValue {
  // Stores
  mapStore: typeof useMapStore;
  layerStore: typeof useLayerStore;

  // Map container ref
  mapRef: React.RefObject<HTMLDivElement>;

  // Event handlers
  eventHandlers: MapEventHandlers;
  registerEventHandler: (
    event: keyof MapEventHandlers,
    handler: (event: MapEvent) => void
  ) => void;
  unregisterEventHandler: (event: keyof MapEventHandlers) => void;

  // Utility methods
  fitBounds: (bounds: [number, number, number, number]) => void;
  flyTo: (center: { lon: number; lat: number }, zoom: number) => void;
  getMapInstance: () => unknown; // Map instance type depends on implementation
}

const MapContext = createContext<MapContextValue | null>(null);

// ============================================================================
// Provider Component
// ============================================================================

export interface MapProviderProps {
  children: React.ReactNode;
  initialViewState?: Partial<MapViewState>;
  onMapLoad?: () => void;
}

/**
 * Map Provider Component
 * Wraps the application to provide map context
 */
export const MapProvider: React.FC<MapProviderProps> = ({
  children,
  initialViewState,
  onMapLoad,
}) => {
  const mapRef = useRef<HTMLDivElement>(null);
  const eventHandlersRef = useRef<MapEventHandlers>({});
  const mapInstanceRef = useRef<unknown>(null);

  // Initialize view state
  const setViewState = useMapStore((state) => state.setViewState);

  useEffect(() => {
    if (initialViewState) {
      setViewState(initialViewState);
    }
  }, [initialViewState, setViewState]);

  // Event handler registration
  const registerEventHandler = useCallback(
    (event: keyof MapEventHandlers, handler: (event: MapEvent) => void) => {
      eventHandlersRef.current[event] = handler;
    },
    []
  );

  const unregisterEventHandler = useCallback(
    (event: keyof MapEventHandlers) => {
      delete eventHandlersRef.current[event];
    },
    []
  );

  // Utility methods
  const fitBounds = useCallback(
    (bounds: [number, number, number, number]) => {
      // Calculate center and zoom from bounds
      const [minLon, minLat, maxLon, maxLat] = bounds;
      const centerLon = (minLon + maxLon) / 2;
      const centerLat = (minLat + maxLat) / 2;

      // Simple zoom calculation (can be improved)
      const lonDiff = Math.abs(maxLon - minLon);
      const latDiff = Math.abs(maxLat - minLat);
      const maxDiff = Math.max(lonDiff, latDiff);
      const zoom = Math.log2(360 / maxDiff);

      setViewState({
        center: { lon: centerLon, lat: centerLat },
        zoom: Math.min(Math.max(zoom, 0), 22),
      });
    },
    [setViewState]
  );

  const flyTo = useCallback(
    (center: { lon: number; lat: number }, zoom: number) => {
      // Animate to position (implementation depends on map library)
      setViewState({ center, zoom });
    },
    [setViewState]
  );

  const getMapInstance = useCallback(() => {
    return mapInstanceRef.current;
  }, []);

  const contextValue: MapContextValue = {
    mapStore: useMapStore,
    layerStore: useLayerStore,
    mapRef,
    eventHandlers: eventHandlersRef.current,
    registerEventHandler,
    unregisterEventHandler,
    fitBounds,
    flyTo,
    getMapInstance,
  };

  return (
    <MapContext.Provider value={contextValue}>{children}</MapContext.Provider>
  );
};

// ============================================================================
// Hook
// ============================================================================

/**
 * Use Map Context Hook
 * Provides access to map context
 */
export const useMapContext = (): MapContextValue => {
  const context = useContext(MapContext);
  if (!context) {
    throw new Error('useMapContext must be used within MapProvider');
  }
  return context;
};

/**
 * Export stores for direct access
 */
export { useMapStore, useLayerStore };
