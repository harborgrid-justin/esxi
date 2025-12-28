import { useEffect, useRef } from 'react';
import { useMap } from '@/hooks/useMap';
import { useLayers } from '@/hooks/useLayers';
import { useMapStore } from '@/stores/mapStore';

export function MapContainer() {
  const mapContainerRef = useRef<HTMLDivElement>(null);
  const { layers } = useLayers();
  const { addLayerToMap } = useLayers();
  const map = useMapStore((state) => state.map);

  useMap({
    container: 'map',
    center: [-98.5795, 39.8283], // Center of US
    zoom: 4,
  });

  // Add layers to map when they change
  useEffect(() => {
    if (!map || !layers) return;

    layers.forEach((layer) => {
      if (layer.visible) {
        addLayerToMap(layer);
      }
    });
  }, [map, layers, addLayerToMap]);

  return (
    <div className="relative w-full h-full">
      <div
        id="map"
        ref={mapContainerRef}
        className="absolute inset-0 w-full h-full"
        style={{ cursor: 'default' }}
      />
    </div>
  );
}
