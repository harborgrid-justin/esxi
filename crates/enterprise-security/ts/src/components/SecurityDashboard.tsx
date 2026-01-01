/**
 * Security Dashboard - Security Overview Component
 * Real-time security metrics and threat monitoring
 */

import React, { useState, useEffect } from 'react';
import { threatDetection } from '../services/ThreatDetection';
import { vulnerabilityScanner } from '../services/VulnerabilityScanner';
import { incidentResponse } from '../services/IncidentResponse';

export interface SecurityDashboardProps {
  refreshInterval?: number;
}

export const SecurityDashboard: React.FC<SecurityDashboardProps> = ({ refreshInterval = 30000 }) => {
  const [activeThreats, setActiveThreats] = useState(0);
  const [openVulnerabilities, setOpenVulnerabilities] = useState(0);
  const [openIncidents, setOpenIncidents] = useState(0);
  const [lastUpdated, setLastUpdated] = useState(new Date());

  useEffect(() => {
    const updateMetrics = () => {
      setActiveThreats(threatDetection.getActiveThreats().length);
      setOpenVulnerabilities(vulnerabilityScanner.getOpenVulnerabilities().length);
      setOpenIncidents(incidentResponse.getOpenIncidents().length);
      setLastUpdated(new Date());
    };

    updateMetrics();
    const interval = setInterval(updateMetrics, refreshInterval);

    return () => clearInterval(interval);
  }, [refreshInterval]);

  return (
    <div className="security-dashboard">
      <div className="dashboard-header">
        <h2>Security Dashboard</h2>
        <span className="last-updated">
          Last updated: {lastUpdated.toLocaleTimeString()}
        </span>
      </div>

      <div className="metrics-grid">
        <div className={`metric-card ${activeThreats > 0 ? 'alert' : 'safe'}`}>
          <h3>Active Threats</h3>
          <div className="metric-value">{activeThreats}</div>
        </div>

        <div className={`metric-card ${openVulnerabilities > 0 ? 'warning' : 'safe'}`}>
          <h3>Open Vulnerabilities</h3>
          <div className="metric-value">{openVulnerabilities}</div>
        </div>

        <div className={`metric-card ${openIncidents > 0 ? 'info' : 'safe'}`}>
          <h3>Open Incidents</h3>
          <div className="metric-value">{openIncidents}</div>
        </div>
      </div>

      <style>{`
        .security-dashboard {
          padding: 24px;
          background: #fff;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        .dashboard-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }
        .metrics-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
        }
        .metric-card {
          padding: 20px;
          border-radius: 8px;
          text-align: center;
        }
        .metric-card.safe {
          background: #d4edda;
          border: 1px solid #c3e6cb;
        }
        .metric-card.alert {
          background: #f8d7da;
          border: 1px solid #f5c6cb;
        }
        .metric-card.warning {
          background: #fff3cd;
          border: 1px solid #ffeaa7;
        }
        .metric-card.info {
          background: #d1ecf1;
          border: 1px solid #bee5eb;
        }
        .metric-value {
          font-size: 36px;
          font-weight: bold;
          margin-top: 8px;
        }
      `}</style>
    </div>
  );
};
