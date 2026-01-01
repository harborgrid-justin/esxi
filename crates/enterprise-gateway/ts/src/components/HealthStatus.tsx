/**
 * Enterprise API Gateway - Health Status Component
 */

import React from 'react';
import type { Upstream, HealthStatus as HealthStatusType } from '../types';

export interface HealthStatusProps {
  upstreams: Upstream[];
  healthStatus: Map<string, HealthStatusType>;
}

export const HealthStatus: React.FC<HealthStatusProps> = ({ upstreams, healthStatus }) => {
  return (
    <div className="health-status">
      <h2>Backend Health Status</h2>
      {upstreams.map((upstream) => (
        <div key={upstream.id} className="upstream-health">
          <h3>{upstream.name}</h3>
          <div className="targets-grid">
            {upstream.targets.map((target) => {
              const health = healthStatus.get(target.id);
              return (
                <div key={target.id} className={`target-card ${target.healthy ? 'healthy' : 'unhealthy'}`}>
                  <div className="status-indicator" />
                  <div>
                    <div className="target-url">{target.url}</div>
                    <div className="target-stats">
                      {health && (
                        <>
                          <span>Response: {health.responseTime}ms</span>
                          <span>Checks: {health.consecutiveSuccesses}/{health.consecutiveFailures}</span>
                        </>
                      )}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      ))}

      <style>{`
        .health-status {
          padding: 20px;
        }

        .upstream-health {
          margin-bottom: 30px;
        }

        .targets-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
          gap: 15px;
        }

        .target-card {
          background: white;
          padding: 15px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
          display: flex;
          align-items: center;
          gap: 10px;
        }

        .status-indicator {
          width: 12px;
          height: 12px;
          border-radius: 50%;
          flex-shrink: 0;
        }

        .target-card.healthy .status-indicator {
          background: #28a745;
        }

        .target-card.unhealthy .status-indicator {
          background: #dc3545;
        }

        .target-url {
          font-weight: 600;
          margin-bottom: 5px;
        }

        .target-stats {
          font-size: 12px;
          color: #666;
          display: flex;
          gap: 10px;
        }
      `}</style>
    </div>
  );
};
