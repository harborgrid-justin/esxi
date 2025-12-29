/**
 * Usage Metrics Component
 * Display tenant usage and resource consumption
 */

import React from 'react';
import { useTenant } from '../../hooks/useTenant';
import { usePermissions } from '../../hooks/usePermissions';
import TenantService from '../../services/TenantService';

interface UsageMetricsProps {
  tenantService: TenantService;
  tenantId?: string;
  className?: string;
}

export const UsageMetrics: React.FC<UsageMetricsProps> = ({
  tenantService,
  tenantId,
  className,
}) => {
  const { tenant, usageMetrics, isLoadingMetrics, refetchMetrics } = useTenant(
    tenantService,
    tenantId
  );
  const { can } = usePermissions();

  const canViewBilling = can.viewBilling();

  if (!canViewBilling) {
    return (
      <div className={className} role="alert">
        <p>You do not have permission to view usage metrics</p>
      </div>
    );
  }

  if (isLoadingMetrics) {
    return (
      <div className={className} role="status" aria-label="Loading usage metrics">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!tenant || !usageMetrics) {
    return (
      <div className={className} role="alert">
        <p>Unable to load usage metrics</p>
      </div>
    );
  }

  const calculatePercentage = (current: number, max: number): number => {
    if (max === 0) return 0;
    return Math.min(100, (current / max) * 100);
  };

  const getUsageColor = (percentage: number): string => {
    if (percentage >= 90) return 'danger';
    if (percentage >= 75) return 'warning';
    return 'success';
  };

  const userPercentage = calculatePercentage(
    usageMetrics.currentUsers,
    tenant.maxUsers
  );
  const orgPercentage = calculatePercentage(
    usageMetrics.currentOrganizations,
    tenant.maxOrganizations
  );

  return (
    <div className={className}>
      <header className="metrics-header">
        <div className="header-content">
          <h1>Usage Metrics</h1>
          <p className="subtitle">Current usage and resource consumption</p>
        </div>
        <button
          type="button"
          onClick={() => refetchMetrics()}
          className="btn btn-secondary"
          aria-label="Refresh metrics"
        >
          Refresh
        </button>
      </header>

      <div className="metrics-grid">
        {/* User Usage */}
        <section className="metric-card" aria-labelledby="user-usage-heading">
          <h2 id="user-usage-heading">User Usage</h2>
          <div className="metric-content">
            <div className="metric-value">
              <span className="current">{usageMetrics.currentUsers}</span>
              <span className="separator">/</span>
              <span className="max">{tenant.maxUsers}</span>
            </div>
            <div className="metric-label">Active Users</div>
            <div
              className="progress-bar"
              role="progressbar"
              aria-valuenow={usageMetrics.currentUsers}
              aria-valuemin={0}
              aria-valuemax={tenant.maxUsers}
              aria-label={`User usage: ${usageMetrics.currentUsers} of ${tenant.maxUsers}`}
            >
              <div
                className={`progress-fill ${getUsageColor(userPercentage)}`}
                style={{ width: `${userPercentage}%` }}
              />
            </div>
            <div className="metric-percentage">
              {userPercentage.toFixed(1)}% used
            </div>
          </div>
        </section>

        {/* Organization Usage */}
        <section className="metric-card" aria-labelledby="org-usage-heading">
          <h2 id="org-usage-heading">Organization Usage</h2>
          <div className="metric-content">
            <div className="metric-value">
              <span className="current">{usageMetrics.currentOrganizations}</span>
              <span className="separator">/</span>
              <span className="max">{tenant.maxOrganizations}</span>
            </div>
            <div className="metric-label">Active Organizations</div>
            <div
              className="progress-bar"
              role="progressbar"
              aria-valuenow={usageMetrics.currentOrganizations}
              aria-valuemin={0}
              aria-valuemax={tenant.maxOrganizations}
              aria-label={`Organization usage: ${usageMetrics.currentOrganizations} of ${tenant.maxOrganizations}`}
            >
              <div
                className={`progress-fill ${getUsageColor(orgPercentage)}`}
                style={{ width: `${orgPercentage}%` }}
              />
            </div>
            <div className="metric-percentage">
              {orgPercentage.toFixed(1)}% used
            </div>
          </div>
        </section>

        {/* Storage Usage */}
        <section className="metric-card" aria-labelledby="storage-usage-heading">
          <h2 id="storage-usage-heading">Storage Usage</h2>
          <div className="metric-content">
            <div className="metric-value">
              <span className="current">
                {usageMetrics.storageUsedGB.toFixed(2)}
              </span>
              <span className="unit">GB</span>
            </div>
            <div className="metric-label">Used Storage</div>
          </div>
        </section>

        {/* API Calls */}
        <section className="metric-card" aria-labelledby="api-usage-heading">
          <h2 id="api-usage-heading">API Usage</h2>
          <div className="metric-content">
            <div className="metric-value">
              <span className="current">
                {usageMetrics.apiCallsThisMonth.toLocaleString()}
              </span>
            </div>
            <div className="metric-label">API Calls This Month</div>
          </div>
        </section>

        {/* Bandwidth Usage */}
        <section className="metric-card" aria-labelledby="bandwidth-usage-heading">
          <h2 id="bandwidth-usage-heading">Bandwidth Usage</h2>
          <div className="metric-content">
            <div className="metric-value">
              <span className="current">
                {usageMetrics.bandwidthUsedGB.toFixed(2)}
              </span>
              <span className="unit">GB</span>
            </div>
            <div className="metric-label">Bandwidth This Month</div>
          </div>
        </section>

        {/* Custom Metrics */}
        {usageMetrics.customMetrics &&
          Object.keys(usageMetrics.customMetrics).length > 0 && (
            <section className="metric-card full-width" aria-labelledby="custom-metrics-heading">
              <h2 id="custom-metrics-heading">Additional Metrics</h2>
              <div className="custom-metrics-list">
                {Object.entries(usageMetrics.customMetrics).map(([key, value]) => (
                  <div key={key} className="custom-metric">
                    <span className="metric-name">
                      {key.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase())}
                    </span>
                    <span className="metric-value">{value.toLocaleString()}</span>
                  </div>
                ))}
              </div>
            </section>
          )}
      </div>

      {/* Warnings */}
      {(userPercentage >= 90 || orgPercentage >= 90) && (
        <div className="usage-warnings" role="alert">
          <h3>Usage Warnings</h3>
          <ul>
            {userPercentage >= 90 && (
              <li>
                User limit almost reached ({usageMetrics.currentUsers}/
                {tenant.maxUsers}). Consider upgrading your plan.
              </li>
            )}
            {orgPercentage >= 90 && (
              <li>
                Organization limit almost reached (
                {usageMetrics.currentOrganizations}/{tenant.maxOrganizations}).
                Consider upgrading your plan.
              </li>
            )}
          </ul>
        </div>
      )}
    </div>
  );
};

export default UsageMetrics;
