import { create } from 'zustand';
import type { Map as MapLibreMap } from 'maplibre-gl';
import type { Layer, Feature, MapTool, MapState } from '@/types';

interface MapStore {
  // Map instance
  map: MapLibreMap | null;
  setMap: (map: MapLibreMap | null) => void;

  // Map state
  mapState: MapState;
  setMapState: (state: Partial<MapState>) => void;

  // Layers
  layers: Layer[];
  setLayers: (layers: Layer[]) => void;
  addLayer: (layer: Layer) => void;
  removeLayer: (layerId: string) => void;
  updateLayer: (layerId: string, updates: Partial<Layer>) => void;
  toggleLayerVisibility: (layerId: string) => void;
  setLayerOpacity: (layerId: string, opacity: number) => void;

  // Selected features
  selectedFeatures: Feature[];
  setSelectedFeatures: (features: Feature[]) => void;
  addSelectedFeature: (feature: Feature) => void;
  removeSelectedFeature: (featureId: string) => void;
  clearSelection: () => void;

  // Active tool
  activeTool: MapTool | null;
  setActiveTool: (tool: MapTool | null) => void;

  // UI state
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
  toggleSidebar: () => void;

  sidebarTab: 'layers' | 'properties' | 'analysis';
  setSidebarTab: (tab: 'layers' | 'properties' | 'analysis') => void;

  layerPanelOpen: boolean;
  setLayerPanelOpen: (open: boolean) => void;
  toggleLayerPanel: () => void;

  // Loading & error states
  loading: boolean;
  setLoading: (loading: boolean) => void;

  error: string | null;
  setError: (error: string | null) => void;
}

export const useMapStore = create<MapStore>((set) => ({
  // Map instance
  map: null,
  setMap: (map) => set({ map }),

  // Map state
  mapState: {
    center: [0, 0],
    zoom: 2,
    bearing: 0,
    pitch: 0,
  },
  setMapState: (state) =>
    set((prev) => ({
      mapState: { ...prev.mapState, ...state },
    })),

  // Layers
  layers: [],
  setLayers: (layers) => set({ layers }),
  addLayer: (layer) =>
    set((state) => ({
      layers: [...state.layers, layer],
    })),
  removeLayer: (layerId) =>
    set((state) => ({
      layers: state.layers.filter((l) => l.id !== layerId),
    })),
  updateLayer: (layerId, updates) =>
    set((state) => ({
      layers: state.layers.map((l) => (l.id === layerId ? { ...l, ...updates } : l)),
    })),
  toggleLayerVisibility: (layerId) =>
    set((state) => ({
      layers: state.layers.map((l) => (l.id === layerId ? { ...l, visible: !l.visible } : l)),
    })),
  setLayerOpacity: (layerId, opacity) =>
    set((state) => ({
      layers: state.layers.map((l) => (l.id === layerId ? { ...l, opacity } : l)),
    })),

  // Selected features
  selectedFeatures: [],
  setSelectedFeatures: (features) => set({ selectedFeatures: features }),
  addSelectedFeature: (feature) =>
    set((state) => ({
      selectedFeatures: [...state.selectedFeatures, feature],
    })),
  removeSelectedFeature: (featureId) =>
    set((state) => ({
      selectedFeatures: state.selectedFeatures.filter((f) => f.id !== featureId),
    })),
  clearSelection: () => set({ selectedFeatures: [] }),

  // Active tool
  activeTool: null,
  setActiveTool: (tool) => set({ activeTool: tool }),

  // UI state
  sidebarOpen: true,
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),

  sidebarTab: 'layers',
  setSidebarTab: (tab) => set({ sidebarTab: tab }),

  layerPanelOpen: false,
  setLayerPanelOpen: (open) => set({ layerPanelOpen: open }),
  toggleLayerPanel: () => set((state) => ({ layerPanelOpen: !state.layerPanelOpen })),

  // Loading & error states
  loading: false,
  setLoading: (loading) => set({ loading }),

  error: null,
  setError: (error) => set({ error }),
}));
