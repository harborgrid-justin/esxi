import { useEffect, useRef } from 'react';
import maplibregl from 'maplibre-gl';
import type { Map as MapLibreMap } from 'maplibre-gl';
import { useMapStore } from '@/stores/mapStore';
import 'maplibre-gl/dist/maplibre-gl.css';

export interface UseMapOptions {
  container: string;
  style?: string | object;
  center?: [number, number];
  zoom?: number;
  bearing?: number;
  pitch?: number;
}

export function useMap(options: UseMapOptions) {
  const mapRef = useRef<MapLibreMap | null>(null);
  const { setMap, setMapState } = useMapStore();

  useEffect(() => {
    if (!mapRef.current) {
      const map = new maplibregl.Map({
        container: options.container,
        style: options.style || {
          version: 8,
          sources: {
            osm: {
              type: 'raster',
              tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
              tileSize: 256,
              attribution: '&copy; OpenStreetMap contributors',
            },
          },
          layers: [
            {
              id: 'osm',
              type: 'raster',
              source: 'osm',
            },
          ],
        },
        center: options.center || [0, 0],
        zoom: options.zoom || 2,
        bearing: options.bearing || 0,
        pitch: options.pitch || 0,
      });

      map.addControl(new maplibregl.NavigationControl(), 'top-right');
      map.addControl(new maplibregl.ScaleControl(), 'bottom-left');

      map.on('load', () => {
        console.log('Map loaded');
        setMap(map);
        mapRef.current = map;
      });

      map.on('move', () => {
        const center = map.getCenter();
        setMapState({
          center: [center.lng, center.lat],
          zoom: map.getZoom(),
          bearing: map.getBearing(),
          pitch: map.getPitch(),
        });
      });

      map.on('error', (e) => {
        console.error('Map error:', e);
      });
    }

    return () => {
      if (mapRef.current) {
        mapRef.current.remove();
        mapRef.current = null;
        setMap(null);
      }
    };
  }, [options.container, setMap, setMapState]);

  return mapRef.current;
}
