/**
 * MapWidget - Embedded map visualization widget
 */

import React, { useEffect, useRef } from 'react';
import { Widget, MapWidgetConfig } from '../../types';
import { useDataSource } from '../../hooks/useDataSource';

export interface MapWidgetProps {
  widget: Widget;
}

export const MapWidget: React.FC<MapWidgetProps> = ({ widget }) => {
  const mapRef = useRef<HTMLDivElement>(null);
  const { data, loading, error } = useDataSource(widget.data_source);

  const config = widget.config as MapWidgetConfig;

  useEffect(() => {
    if (!mapRef.current) return;

    // Initialize map (placeholder - integrate with actual map library)
    // Example: Leaflet, Mapbox, Google Maps, etc.
    console.log('Initializing map with config:', config);

    // Cleanup
    return () => {
      console.log('Cleaning up map');
    };
  }, [config]);

  if (loading) {
    return (
      <div className="map-loading">
        <div className="spinner"></div>
        <p>Loading map...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="map-error">
        <p>Error loading map: {error.message}</p>
      </div>
    );
  }

  return (
    <div className="map-widget">
      <div ref={mapRef} className="map-container">
        {/* Map will be rendered here */}
        <div className="map-placeholder">
          <p>Map View</p>
          <p className="map-info">
            Center: [{config.center[0]}, {config.center[1]}]
          </p>
          <p className="map-info">Zoom: {config.zoom}</p>
          <p className="map-info">Layers: {config.layers.join(', ')}</p>
        </div>
      </div>

      <style jsx>{`
        .map-widget {
          width: 100%;
          height: 100%;
          position: relative;
        }

        .map-container {
          width: 100%;
          height: 100%;
          border-radius: 4px;
          overflow: hidden;
        }

        .map-placeholder {
          width: 100%;
          height: 100%;
          background: #e8f4f8;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          font-size: 14px;
          color: #666;
        }

        .map-info {
          margin: 4px 0;
          font-size: 12px;
        }

        .map-loading,
        .map-error {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: #666;
        }

        .spinner {
          border: 3px solid #f3f3f3;
          border-top: 3px solid #1976d2;
          border-radius: 50%;
          width: 40px;
          height: 40px;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% {
            transform: rotate(0deg);
          }
          100% {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </div>
  );
};
