import React from 'react';
import type { HealthStatus } from '../../types';
import { HEALTH_COLORS } from '../../types';

export interface StatusIndicatorProps {
  health: HealthStatus;
  isConnected: boolean;
}

/**
 * System health and connection status indicator
 */
export const StatusIndicator: React.FC<StatusIndicatorProps> = ({ health, isConnected }) => {
  const getStatusLabel = () => {
    if (!isConnected) return 'Disconnected';
    return health.charAt(0).toUpperCase() + health.slice(1);
  };

  const getStatusColor = () => {
    if (!isConnected) return '#6b7280';
    return HEALTH_COLORS[health];
  };

  return (
    <div className="status-indicator">
      <div className="connection-status">
        <div className={`connection-dot ${isConnected ? 'connected' : 'disconnected'}`} />
        <span className="connection-label">{isConnected ? 'Connected' : 'Disconnected'}</span>
      </div>

      <div className="health-status">
        <div
          className="health-badge"
          style={{
            backgroundColor: getStatusColor(),
          }}
        >
          {getStatusLabel()}
        </div>
      </div>

      <style>{`
        .status-indicator {
          display: flex;
          align-items: center;
          gap: 16px;
        }

        .connection-status {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .connection-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          animation: pulse 2s ease-in-out infinite;
        }

        .connection-dot.connected {
          background: #10b981;
        }

        .connection-dot.disconnected {
          background: #6b7280;
          animation: none;
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        .connection-label {
          font-size: 14px;
          font-weight: 500;
          color: #6b7280;
        }

        .health-badge {
          padding: 6px 16px;
          border-radius: 16px;
          font-size: 13px;
          font-weight: 600;
          color: white;
          text-transform: capitalize;
        }
      `}</style>
    </div>
  );
};
