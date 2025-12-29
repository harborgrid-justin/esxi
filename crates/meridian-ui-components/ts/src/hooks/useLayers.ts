/**
 * Layer Management Hook
 * Provides layer state and control methods
 * @module @meridian/ui-components/hooks
 */

import { useCallback, useMemo } from 'react';
import { useLayerStore } from '../context/MapContext';
import type { LayerConfig, LayerStyle } from '../types';

/**
 * Hook for layer management
 */
export const useLayers = () => {
  const layers = useLayerStore((state) => state.layers);
  const activeLayerId = useLayerStore((state) => state.activeLayerId);
  const addLayer = useLayerStore((state) => state.addLayer);
  const removeLayer = useLayerStore((state) => state.removeLayer);
  const updateLayer = useLayerStore((state) => state.updateLayer);
  const toggleLayerVisibility = useLayerStore(
    (state) => state.toggleLayerVisibility
  );
  const setActiveLayer = useLayerStore((state) => state.setActiveLayer);
  const reorderLayers = useLayerStore((state) => state.reorderLayers);
  const clearLayers = useLayerStore((state) => state.clearLayers);

  /**
   * Get active layer
   */
  const activeLayer = useMemo(
    () => layers.find((l) => l.id === activeLayerId) || null,
    [layers, activeLayerId]
  );

  /**
   * Get visible layers
   */
  const visibleLayers = useMemo(
    () => layers.filter((l) => l.visible),
    [layers]
  );

  /**
   * Get layers sorted by z-index
   */
  const sortedLayers = useMemo(
    () =>
      [...layers].sort((a, b) => (a.style.zIndex || 0) - (b.style.zIndex || 0)),
    [layers]
  );

  /**
   * Add new layer
   */
  const createLayer = useCallback(
    (layer: Omit<LayerConfig, 'id'>) => {
      const id = `layer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      const newLayer: LayerConfig = {
        ...layer,
        id,
      };
      addLayer(newLayer);
      return newLayer;
    },
    [addLayer]
  );

  /**
   * Get layer by ID
   */
  const getLayer = useCallback(
    (layerId: string): LayerConfig | undefined => {
      return layers.find((l) => l.id === layerId);
    },
    [layers]
  );

  /**
   * Check if layer exists
   */
  const hasLayer = useCallback(
    (layerId: string): boolean => {
      return layers.some((l) => l.id === layerId);
    },
    [layers]
  );

  /**
   * Get layers by type
   */
  const getLayersByType = useCallback(
    (type: LayerConfig['type']) => {
      return layers.filter((l) => l.type === type);
    },
    [layers]
  );

  /**
   * Update layer name
   */
  const updateLayerName = useCallback(
    (layerId: string, name: string) => {
      updateLayer(layerId, { name });
    },
    [updateLayer]
  );

  /**
   * Update layer style
   */
  const updateLayerStyle = useCallback(
    (layerId: string, style: Partial<LayerStyle>) => {
      const layer = getLayer(layerId);
      if (layer) {
        updateLayer(layerId, {
          style: { ...layer.style, ...style },
        });
      }
    },
    [getLayer, updateLayer]
  );

  /**
   * Update layer opacity
   */
  const updateLayerOpacity = useCallback(
    (layerId: string, opacity: number) => {
      updateLayer(layerId, { opacity: Math.max(0, Math.min(1, opacity)) });
    },
    [updateLayer]
  );

  /**
   * Set layer visibility
   */
  const setLayerVisibility = useCallback(
    (layerId: string, visible: boolean) => {
      updateLayer(layerId, { visible });
    },
    [updateLayer]
  );

  /**
   * Show layer
   */
  const showLayer = useCallback(
    (layerId: string) => {
      setLayerVisibility(layerId, true);
    },
    [setLayerVisibility]
  );

  /**
   * Hide layer
   */
  const hideLayer = useCallback(
    (layerId: string) => {
      setLayerVisibility(layerId, false);
    },
    [setLayerVisibility]
  );

  /**
   * Show all layers
   */
  const showAllLayers = useCallback(() => {
    layers.forEach((layer) => {
      if (!layer.visible) {
        setLayerVisibility(layer.id, true);
      }
    });
  }, [layers, setLayerVisibility]);

  /**
   * Hide all layers
   */
  const hideAllLayers = useCallback(() => {
    layers.forEach((layer) => {
      if (layer.visible) {
        setLayerVisibility(layer.id, false);
      }
    });
  }, [layers, setLayerVisibility]);

  /**
   * Move layer up in z-order
   */
  const moveLayerUp = useCallback(
    (layerId: string) => {
      const layer = getLayer(layerId);
      if (layer && layer.style.zIndex !== undefined) {
        updateLayerStyle(layerId, { zIndex: layer.style.zIndex + 1 });
      }
    },
    [getLayer, updateLayerStyle]
  );

  /**
   * Move layer down in z-order
   */
  const moveLayerDown = useCallback(
    (layerId: string) => {
      const layer = getLayer(layerId);
      if (layer && layer.style.zIndex !== undefined) {
        updateLayerStyle(layerId, { zIndex: layer.style.zIndex - 1 });
      }
    },
    [getLayer, updateLayerStyle]
  );

  /**
   * Move layer to top
   */
  const moveLayerToTop = useCallback(
    (layerId: string) => {
      const maxZIndex = Math.max(...layers.map((l) => l.style.zIndex || 0));
      updateLayerStyle(layerId, { zIndex: maxZIndex + 1 });
    },
    [layers, updateLayerStyle]
  );

  /**
   * Move layer to bottom
   */
  const moveLayerToBottom = useCallback(
    (layerId: string) => {
      const minZIndex = Math.min(...layers.map((l) => l.style.zIndex || 0));
      updateLayerStyle(layerId, { zIndex: minZIndex - 1 });
    },
    [layers, updateLayerStyle]
  );

  /**
   * Duplicate layer
   */
  const duplicateLayer = useCallback(
    (layerId: string) => {
      const layer = getLayer(layerId);
      if (layer) {
        const newLayer = {
          ...layer,
          name: `${layer.name} (Copy)`,
        };
        return createLayer(newLayer);
      }
      return null;
    },
    [getLayer, createLayer]
  );

  /**
   * Get layer count
   */
  const layerCount = useMemo(() => layers.length, [layers]);

  /**
   * Get visible layer count
   */
  const visibleLayerCount = useMemo(
    () => visibleLayers.length,
    [visibleLayers]
  );

  /**
   * Check if any layers exist
   */
  const hasLayers = useMemo(() => layers.length > 0, [layers]);

  /**
   * Batch update layers
   */
  const batchUpdateLayers = useCallback(
    (updates: Array<{ layerId: string; updates: Partial<LayerConfig> }>) => {
      updates.forEach(({ layerId, updates: layerUpdates }) => {
        updateLayer(layerId, layerUpdates);
      });
    },
    [updateLayer]
  );

  /**
   * Import layers from configuration
   */
  const importLayers = useCallback(
    (layerConfigs: LayerConfig[]) => {
      clearLayers();
      layerConfigs.forEach((config) => {
        addLayer(config);
      });
    },
    [addLayer, clearLayers]
  );

  /**
   * Export layers configuration
   */
  const exportLayers = useCallback(() => {
    return JSON.parse(JSON.stringify(layers));
  }, [layers]);

  return {
    // State
    layers,
    activeLayerId,
    activeLayer,
    visibleLayers,
    sortedLayers,
    layerCount,
    visibleLayerCount,
    hasLayers,

    // Layer CRUD
    createLayer,
    addLayer,
    removeLayer,
    updateLayer,
    getLayer,
    hasLayer,
    getLayersByType,
    duplicateLayer,

    // Layer properties
    updateLayerName,
    updateLayerStyle,
    updateLayerOpacity,

    // Visibility
    toggleLayerVisibility,
    setLayerVisibility,
    showLayer,
    hideLayer,
    showAllLayers,
    hideAllLayers,

    // Layer order
    setActiveLayer,
    reorderLayers,
    moveLayerUp,
    moveLayerDown,
    moveLayerToTop,
    moveLayerToBottom,

    // Batch operations
    batchUpdateLayers,
    clearLayers,

    // Import/Export
    importLayers,
    exportLayers,
  };
};

export default useLayers;
