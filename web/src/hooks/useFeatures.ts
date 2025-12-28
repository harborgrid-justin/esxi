import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { layersApi } from '@/api/layers';
import { useMapStore } from '@/stores/mapStore';
import type { Feature, FeatureCollection } from '@/types';

export function useFeatures(layerId: string, bbox?: [number, number, number, number]) {
  const queryClient = useQueryClient();
  const { map, setSelectedFeatures, clearSelection } = useMapStore();

  const { data: features, isLoading, error } = useQuery({
    queryKey: ['features', layerId, bbox],
    queryFn: () => layersApi.getFeatures(layerId, bbox),
    enabled: !!layerId,
  });

  const createFeatureMutation = useMutation({
    mutationFn: (feature: Omit<Feature, 'id'>) => layersApi.createFeature(layerId, feature),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['features', layerId] });

      // Update map source
      if (map && map.getSource(layerId)) {
        const source = map.getSource(layerId);
        if (source.type === 'geojson') {
          queryClient.fetchQuery({ queryKey: ['features', layerId] }).then((data) => {
            source.setData(data as FeatureCollection);
          });
        }
      }
    },
  });

  const updateFeatureMutation = useMutation({
    mutationFn: ({ featureId, updates }: { featureId: string; updates: Partial<Feature> }) =>
      layersApi.updateFeature(layerId, featureId, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['features', layerId] });

      // Update map source
      if (map && map.getSource(layerId)) {
        const source = map.getSource(layerId);
        if (source.type === 'geojson') {
          queryClient.fetchQuery({ queryKey: ['features', layerId] }).then((data) => {
            source.setData(data as FeatureCollection);
          });
        }
      }
    },
  });

  const deleteFeatureMutation = useMutation({
    mutationFn: (featureId: string) => layersApi.deleteFeature(layerId, featureId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['features', layerId] });

      // Update map source
      if (map && map.getSource(layerId)) {
        const source = map.getSource(layerId);
        if (source.type === 'geojson') {
          queryClient.fetchQuery({ queryKey: ['features', layerId] }).then((data) => {
            source.setData(data as FeatureCollection);
          });
        }
      }
    },
  });

  const selectFeatureOnMap = (feature: Feature) => {
    if (!map) return;

    setSelectedFeatures([feature]);

    // Highlight feature on map
    if (!map.getSource('selected-features')) {
      map.addSource('selected-features', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [feature] },
      });

      map.addLayer({
        id: 'selected-features-fill',
        type: 'fill',
        source: 'selected-features',
        paint: {
          'fill-color': '#ff0000',
          'fill-opacity': 0.3,
        },
      });

      map.addLayer({
        id: 'selected-features-outline',
        type: 'line',
        source: 'selected-features',
        paint: {
          'line-color': '#ff0000',
          'line-width': 2,
        },
      });
    } else {
      const source = map.getSource('selected-features');
      if (source.type === 'geojson') {
        source.setData({ type: 'FeatureCollection', features: [feature] });
      }
    }
  };

  const clearFeatureSelection = () => {
    if (!map) return;

    clearSelection();

    if (map.getSource('selected-features')) {
      const source = map.getSource('selected-features');
      if (source.type === 'geojson') {
        source.setData({ type: 'FeatureCollection', features: [] });
      }
    }
  };

  return {
    features,
    isLoading,
    error,
    createFeature: createFeatureMutation.mutate,
    updateFeature: updateFeatureMutation.mutate,
    deleteFeature: deleteFeatureMutation.mutate,
    selectFeature: selectFeatureOnMap,
    clearSelection: clearFeatureSelection,
  };
}
