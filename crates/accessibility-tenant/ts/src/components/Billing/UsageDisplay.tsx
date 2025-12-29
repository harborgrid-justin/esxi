/**
 * Usage Display Component
 * Display current usage statistics
 */

import React from 'react';
import { UsageMetrics } from '../../types';

interface UsageDisplayProps {
  usage: UsageMetrics;
  className?: string;
}

export const UsageDisplay: React.FC<UsageDisplayProps> = ({ usage, className }) => {
  return (
    <div className={className}>
      <header>
        <h2>Current Usage</h2>
      </header>

      <div className="usage-grid">
        <div className="usage-card">
          <h3>Users</h3>
          <div className="usage-value">{usage.currentUsers.toLocaleString()}</div>
        </div>

        <div className="usage-card">
          <h3>Organizations</h3>
          <div className="usage-value">{usage.currentOrganizations.toLocaleString()}</div>
        </div>

        <div className="usage-card">
          <h3>Storage</h3>
          <div className="usage-value">{usage.storageUsedGB.toFixed(2)} GB</div>
        </div>

        <div className="usage-card">
          <h3>API Calls</h3>
          <div className="usage-value">{usage.apiCallsThisMonth.toLocaleString()}</div>
          <p className="usage-label">This Month</p>
        </div>

        <div className="usage-card">
          <h3>Bandwidth</h3>
          <div className="usage-value">{usage.bandwidthUsedGB.toFixed(2)} GB</div>
          <p className="usage-label">This Month</p>
        </div>
      </div>
    </div>
  );
};

export default UsageDisplay;
