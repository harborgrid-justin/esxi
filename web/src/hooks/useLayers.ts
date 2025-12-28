import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { layersApi } from '@/api/layers';
import { useMapStore } from '@/stores/mapStore';
import type { Layer, FeatureCollection } from '@/types';

export function useLayers() {
  const queryClient = useQueryClient();
  const { map, setLayers, addLayer, removeLayer, updateLayer } = useMapStore();

  const { data: layers, isLoading, error } = useQuery({
    queryKey: ['layers'],
    queryFn: layersApi.getLayers,
    onSuccess: (data) => {
      setLayers(data);
    },
  });

  const createLayerMutation = useMutation({
    mutationFn: layersApi.createLayer,
    onSuccess: (newLayer) => {
      queryClient.invalidateQueries({ queryKey: ['layers'] });
      addLayer(newLayer);
    },
  });

  const updateLayerMutation = useMutation({
    mutationFn: ({ id, updates }: { id: string; updates: Partial<Layer> }) =>
      layersApi.updateLayer(id, updates),
    onSuccess: (updatedLayer) => {
      queryClient.invalidateQueries({ queryKey: ['layers'] });
      updateLayer(updatedLayer.id, updatedLayer);
    },
  });

  const deleteLayerMutation = useMutation({
    mutationFn: layersApi.deleteLayer,
    onSuccess: (_, layerId) => {
      queryClient.invalidateQueries({ queryKey: ['layers'] });
      removeLayer(layerId);

      // Remove from map
      if (map && map.getLayer(layerId)) {
        map.removeLayer(layerId);
        if (map.getSource(layerId)) {
          map.removeSource(layerId);
        }
      }
    },
  });

  const addLayerToMap = (layer: Layer, features?: FeatureCollection) => {
    if (!map) return;

    // Add source
    if (!map.getSource(layer.id)) {
      if (layer.source.type === 'geojson') {
        map.addSource(layer.id, {
          type: 'geojson',
          data: features || layer.source.data || { type: 'FeatureCollection', features: [] },
        });
      } else if (layer.source.type === 'vector') {
        map.addSource(layer.id, {
          type: 'vector',
          tiles: layer.source.tiles,
          bounds: layer.source.bounds,
        });
      } else if (layer.source.type === 'raster') {
        map.addSource(layer.id, {
          type: 'raster',
          tiles: layer.source.tiles,
          tileSize: 256,
        });
      }
    }

    // Add layer
    if (!map.getLayer(layer.id)) {
      const layerConfig: maplibregl.LayerSpecification = {
        id: layer.id,
        source: layer.id,
        type: layer.type === 'vector' ? 'fill' : 'raster',
        paint: {},
      };

      if (layer.type === 'vector' && layer.style) {
        layerConfig.paint = {
          'fill-color': layer.style.fillColor || '#3388ff',
          'fill-opacity': layer.style.fillOpacity || 0.4,
        };
      }

      map.addLayer(layerConfig);
    }

    // Set visibility and opacity
    map.setLayoutProperty(layer.id, 'visibility', layer.visible ? 'visible' : 'none');
    if (layer.type === 'vector') {
      map.setPaintProperty(layer.id, 'fill-opacity', layer.opacity);
    } else {
      map.setPaintProperty(layer.id, 'raster-opacity', layer.opacity);
    }
  };

  const removeLayerFromMap = (layerId: string) => {
    if (!map) return;

    if (map.getLayer(layerId)) {
      map.removeLayer(layerId);
    }
    if (map.getSource(layerId)) {
      map.removeSource(layerId);
    }
  };

  return {
    layers,
    isLoading,
    error,
    createLayer: createLayerMutation.mutate,
    updateLayer: updateLayerMutation.mutate,
    deleteLayer: deleteLayerMutation.mutate,
    addLayerToMap,
    removeLayerFromMap,
  };
}
