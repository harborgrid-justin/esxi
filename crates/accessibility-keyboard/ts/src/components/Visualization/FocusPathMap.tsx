/**
 * Focus Path Map Component
 * Visualizes the path of focus navigation through the page
 */

import React, { useState, useEffect, useCallback } from 'react';
import { FocusPath, FocusEvent } from '../../types';
import { useFocusTracking } from '../../hooks/useFocusTracking';

export interface FocusPathMapProps {
  show: boolean;
  autoTrack?: boolean;
  maxPathLength?: number;
  onClose?: () => void;
}

export const FocusPathMap: React.FC<FocusPathMapProps> = ({
  show,
  autoTrack = true,
  maxPathLength = 20,
  onClose,
}) => {
  const { focusHistory, isTracking, startTracking, stopTracking, clearHistory } =
    useFocusTracking();
  const [pathLines, setPathLines] = useState<JSX.Element[]>([]);
  const [pathMarkers, setPathMarkers] = useState<JSX.Element[]>([]);

  useEffect(() => {
    if (show && autoTrack && !isTracking) {
      startTracking();
    }
  }, [show, autoTrack]);

  useEffect(() => {
    if (show && focusHistory.length > 0) {
      renderFocusPath();
    }
  }, [show, focusHistory]);

  const renderFocusPath = () => {
    const recentFocusEvents = focusHistory
      .filter((event) => event.type === 'focus')
      .slice(-maxPathLength);

    if (recentFocusEvents.length < 2) {
      setPathLines([]);
      setPathMarkers([]);
      return;
    }

    // Create path lines
    const lines: JSX.Element[] = [];
    for (let i = 0; i < recentFocusEvents.length - 1; i++) {
      const from = recentFocusEvents[i].element.getBoundingClientRect();
      const to = recentFocusEvents[i + 1].element.getBoundingClientRect();

      const fromX = from.left + from.width / 2;
      const fromY = from.top + from.height / 2;
      const toX = to.left + to.width / 2;
      const toY = to.top + to.height / 2;

      const length = Math.sqrt(Math.pow(toX - fromX, 2) + Math.pow(toY - fromY, 2));
      const angle = Math.atan2(toY - fromY, toX - fromX) * (180 / Math.PI);

      const opacity = 0.3 + (i / recentFocusEvents.length) * 0.7;
      const color = recentFocusEvents[i].triggeredBy === 'keyboard' ? '#007bff' : '#28a745';

      lines.push(
        <div
          key={`line-${i}`}
          style={{
            position: 'fixed',
            top: `${fromY}px`,
            left: `${fromX}px`,
            width: `${length}px`,
            height: '3px',
            backgroundColor: color,
            transformOrigin: '0 50%',
            transform: `rotate(${angle}deg)`,
            opacity,
            zIndex: 999990 + i,
            pointerEvents: 'none',
            transition: 'all 0.3s ease',
          }}
        >
          {/* Arrow head */}
          <div
            style={{
              position: 'absolute',
              right: '-6px',
              top: '-3px',
              width: 0,
              height: 0,
              borderLeft: '6px solid ' + color,
              borderTop: '4.5px solid transparent',
              borderBottom: '4.5px solid transparent',
              opacity,
            }}
          />
        </div>
      );
    }

    // Create markers
    const markers = recentFocusEvents.map((event, index) => {
      const rect = event.element.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const isLatest = index === recentFocusEvents.length - 1;
      const size = isLatest ? 24 : 16;
      const color = event.triggeredBy === 'keyboard' ? '#007bff' : '#28a745';

      return (
        <div
          key={`marker-${index}`}
          style={{
            position: 'fixed',
            top: `${centerY - size / 2}px`,
            left: `${centerX - size / 2}px`,
            width: `${size}px`,
            height: `${size}px`,
            backgroundColor: color,
            borderRadius: '50%',
            border: '2px solid white',
            boxShadow: '0 2px 4px rgba(0,0,0,0.3)',
            zIndex: 999995 + index,
            pointerEvents: 'none',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
            fontSize: isLatest ? '12px' : '9px',
            fontWeight: 700,
            transition: 'all 0.3s ease',
          }}
        >
          {index + 1}
        </div>
      );
    });

    setPathLines(lines);
    setPathMarkers(markers);
  };

  const handleClear = () => {
    clearHistory();
    setPathLines([]);
    setPathMarkers([]);
  };

  if (!show) return null;

  return (
    <>
      {/* Semi-transparent background */}
      <div
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'rgba(0, 0, 0, 0.2)',
          zIndex: 999989,
          backdropFilter: 'blur(1px)',
          pointerEvents: 'none',
        }}
      />

      {/* Path lines */}
      {pathLines}

      {/* Path markers */}
      {pathMarkers}

      {/* Control panel */}
      <div
        style={{
          position: 'fixed',
          top: '20px',
          left: '20px',
          backgroundColor: 'white',
          padding: '16px',
          borderRadius: '8px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.15)',
          zIndex: 999999,
          minWidth: '280px',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '12px' }}>
          <h4 style={{ margin: 0, fontSize: '16px' }}>Focus Path Map</h4>
          <button
            onClick={onClose}
            style={{
              border: 'none',
              background: 'none',
              fontSize: '20px',
              cursor: 'pointer',
              padding: 0,
              lineHeight: 1,
            }}
          >
            ×
          </button>
        </div>

        <div style={{ fontSize: '13px', color: '#6c757d', marginBottom: '12px' }}>
          {isTracking ? 'Tracking focus navigation...' : 'Tracking paused'}
        </div>

        <div style={{ fontSize: '12px', marginBottom: '12px' }}>
          <div style={{ marginBottom: '4px' }}>
            Path length: {focusHistory.filter((e) => e.type === 'focus').length}
          </div>
          <div>Showing last {Math.min(maxPathLength, focusHistory.length)} focus events</div>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '12px' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '16px',
                height: '16px',
                backgroundColor: '#007bff',
                borderRadius: '50%',
                border: '2px solid white',
              }}
            />
            <span style={{ fontSize: '12px' }}>Keyboard focus</span>
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <div
              style={{
                width: '16px',
                height: '16px',
                backgroundColor: '#28a745',
                borderRadius: '50%',
                border: '2px solid white',
              }}
            />
            <span style={{ fontSize: '12px' }}>Mouse focus</span>
          </div>
        </div>

        <div style={{ display: 'flex', gap: '8px', paddingTop: '12px', borderTop: '1px solid #dee2e6' }}>
          <button
            onClick={isTracking ? stopTracking : startTracking}
            style={{
              flex: 1,
              padding: '8px',
              backgroundColor: isTracking ? '#dc3545' : '#28a745',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '13px',
            }}
          >
            {isTracking ? 'Pause' : 'Resume'}
          </button>
          <button
            onClick={handleClear}
            style={{
              flex: 1,
              padding: '8px',
              backgroundColor: '#6c757d',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
              fontSize: '13px',
            }}
          >
            Clear
          </button>
        </div>
      </div>
    </>
  );
};
