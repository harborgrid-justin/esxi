/**
 * Selection Management Hook
 * Provides feature selection state and control methods
 * @module @meridian/ui-components/hooks
 */

import { useCallback, useMemo } from 'react';
import { useMapStore } from '../context/MapContext';
import type { Feature, SelectionMode } from '../types';

/**
 * Hook for selection management
 */
export const useSelection = () => {
  const selectedFeatures = useMapStore((state) => state.selectedFeatures);
  const hoveredFeature = useMapStore((state) => state.hoveredFeature);
  const selectFeature = useMapStore((state) => state.selectFeature);
  const deselectFeature = useMapStore((state) => state.deselectFeature);
  const clearSelection = useMapStore((state) => state.clearSelection);
  const setHoveredFeature = useMapStore((state) => state.setHoveredFeature);

  /**
   * Get selected feature IDs
   */
  const selectedIds = useMemo(
    () => selectedFeatures.map((f) => f.id).filter((id): id is string | number => id !== undefined),
    [selectedFeatures]
  );

  /**
   * Get selection count
   */
  const selectionCount = useMemo(
    () => selectedFeatures.length,
    [selectedFeatures]
  );

  /**
   * Check if feature is selected
   */
  const isFeatureSelected = useCallback(
    (featureId: string | number): boolean => {
      return selectedFeatures.some((f) => f.id === featureId);
    },
    [selectedFeatures]
  );

  /**
   * Check if feature is hovered
   */
  const isFeatureHovered = useCallback(
    (featureId: string | number): boolean => {
      return hoveredFeature?.id === featureId;
    },
    [hoveredFeature]
  );

  /**
   * Select single feature
   */
  const selectSingle = useCallback(
    (feature: Feature) => {
      selectFeature(feature, false);
    },
    [selectFeature]
  );

  /**
   * Toggle feature selection
   */
  const toggleFeature = useCallback(
    (feature: Feature) => {
      selectFeature(feature, true);
    },
    [selectFeature]
  );

  /**
   * Select multiple features
   */
  const selectMultiple = useCallback(
    (features: Feature[]) => {
      clearSelection();
      features.forEach((feature) => {
        selectFeature(feature, true);
      });
    },
    [clearSelection, selectFeature]
  );

  /**
   * Add feature to selection
   */
  const addToSelection = useCallback(
    (feature: Feature) => {
      if (!isFeatureSelected(feature.id!)) {
        selectFeature(feature, true);
      }
    },
    [isFeatureSelected, selectFeature]
  );

  /**
   * Remove feature from selection
   */
  const removeFromSelection = useCallback(
    (featureId: string | number) => {
      deselectFeature(featureId);
    },
    [deselectFeature]
  );

  /**
   * Select all features
   */
  const selectAll = useCallback(
    (features: Feature[]) => {
      selectMultiple(features);
    },
    [selectMultiple]
  );

  /**
   * Invert selection
   */
  const invertSelection = useCallback(
    (allFeatures: Feature[]) => {
      const newSelection = allFeatures.filter(
        (f) => !isFeatureSelected(f.id!)
      );
      selectMultiple(newSelection);
    },
    [isFeatureSelected, selectMultiple]
  );

  /**
   * Select by attribute
   */
  const selectByAttribute = useCallback(
    (
      features: Feature[],
      attributeName: string,
      attributeValue: unknown
    ) => {
      const matching = features.filter(
        (f) => f.properties[attributeName] === attributeValue
      );
      selectMultiple(matching);
    },
    [selectMultiple]
  );

  /**
   * Select by condition
   */
  const selectByCondition = useCallback(
    (features: Feature[], condition: (feature: Feature) => boolean) => {
      const matching = features.filter(condition);
      selectMultiple(matching);
    },
    [selectMultiple]
  );

  /**
   * Get first selected feature
   */
  const firstSelected = useMemo(
    () => selectedFeatures[0] || null,
    [selectedFeatures]
  );

  /**
   * Get last selected feature
   */
  const lastSelected = useMemo(
    () => selectedFeatures[selectedFeatures.length - 1] || null,
    [selectedFeatures]
  );

  /**
   * Check if has selection
   */
  const hasSelection = useMemo(
    () => selectedFeatures.length > 0,
    [selectedFeatures]
  );

  /**
   * Check if single selection
   */
  const isSingleSelection = useMemo(
    () => selectedFeatures.length === 1,
    [selectedFeatures]
  );

  /**
   * Check if multiple selection
   */
  const isMultipleSelection = useMemo(
    () => selectedFeatures.length > 1,
    [selectedFeatures]
  );

  /**
   * Get selection bounds
   */
  const getSelectionBounds = useCallback(():
    | [number, number, number, number]
    | null => {
    if (selectedFeatures.length === 0) return null;

    let minLon = Infinity;
    let minLat = Infinity;
    let maxLon = -Infinity;
    let maxLat = -Infinity;

    selectedFeatures.forEach((feature) => {
      const coords = extractCoordinates(feature.geometry.coordinates);
      coords.forEach(([lon, lat]) => {
        minLon = Math.min(minLon, lon);
        minLat = Math.min(minLat, lat);
        maxLon = Math.max(maxLon, lon);
        maxLat = Math.max(maxLat, lat);
      });
    });

    return [minLon, minLat, maxLon, maxLat];
  }, [selectedFeatures]);

  /**
   * Export selected features as GeoJSON
   */
  const exportSelection = useCallback(() => {
    return {
      type: 'FeatureCollection' as const,
      features: selectedFeatures,
    };
  }, [selectedFeatures]);

  /**
   * Get selected attributes
   */
  const getSelectedAttributes = useCallback(
    (attributeName: string): unknown[] => {
      return selectedFeatures
        .map((f) => f.properties[attributeName])
        .filter((v) => v !== undefined);
    },
    [selectedFeatures]
  );

  /**
   * Get unique attribute values from selection
   */
  const getUniqueAttributeValues = useCallback(
    (attributeName: string): unknown[] => {
      const values = getSelectedAttributes(attributeName);
      return Array.from(new Set(values));
    },
    [getSelectedAttributes]
  );

  return {
    // State
    selectedFeatures,
    hoveredFeature,
    selectedIds,
    selectionCount,
    hasSelection,
    isSingleSelection,
    isMultipleSelection,
    firstSelected,
    lastSelected,

    // Selection operations
    selectSingle,
    selectMultiple,
    toggleFeature,
    addToSelection,
    removeFromSelection,
    clearSelection,
    selectAll,
    invertSelection,
    selectByAttribute,
    selectByCondition,

    // Hover
    setHoveredFeature,

    // Checks
    isFeatureSelected,
    isFeatureHovered,

    // Utilities
    getSelectionBounds,
    exportSelection,
    getSelectedAttributes,
    getUniqueAttributeValues,
  };
};

/**
 * Helper function to extract coordinates from geometry
 */
function extractCoordinates(
  coords: number[] | number[][] | number[][][]
): number[][] {
  if (typeof coords[0] === 'number') {
    return [coords as number[]];
  }

  if (Array.isArray(coords[0]) && typeof coords[0][0] === 'number') {
    return coords as number[][];
  }

  // Flatten nested arrays
  const result: number[][] = [];
  function flatten(arr: unknown): void {
    if (Array.isArray(arr)) {
      if (typeof arr[0] === 'number') {
        result.push(arr as number[]);
      } else {
        arr.forEach(flatten);
      }
    }
  }
  flatten(coords);
  return result;
}

export default useSelection;
