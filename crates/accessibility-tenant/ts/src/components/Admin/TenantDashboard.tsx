/**
 * Tenant Dashboard Component
 * Main admin dashboard for tenant overview
 */

import React from 'react';
import { useTenantContext } from '../../context/TenantContext';
import { usePermissions } from '../../hooks/usePermissions';
import { SubscriptionStatus, SubscriptionTier } from '../../types';

interface TenantDashboardProps {
  className?: string;
}

export const TenantDashboard: React.FC<TenantDashboardProps> = ({ className }) => {
  const { tenant, user, isLoading } = useTenantContext();
  const { can } = usePermissions();

  if (isLoading) {
    return (
      <div className={className} role="status" aria-label="Loading tenant dashboard">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  if (!tenant || !user) {
    return (
      <div className={className} role="alert">
        <p>Unable to load tenant information</p>
      </div>
    );
  }

  const canManageTenant = can.manageTenant();

  const getSubscriptionStatusColor = (status: SubscriptionStatus): string => {
    switch (status) {
      case SubscriptionStatus.ACTIVE:
        return 'status-active';
      case SubscriptionStatus.TRIAL:
        return 'status-trial';
      case SubscriptionStatus.SUSPENDED:
        return 'status-suspended';
      case SubscriptionStatus.CANCELED:
      case SubscriptionStatus.EXPIRED:
        return 'status-expired';
      default:
        return 'status-default';
    }
  };

  const getTierDisplayName = (tier: SubscriptionTier): string => {
    switch (tier) {
      case SubscriptionTier.FREE:
        return 'Free';
      case SubscriptionTier.STARTER:
        return 'Starter';
      case SubscriptionTier.PROFESSIONAL:
        return 'Professional';
      case SubscriptionTier.ENTERPRISE:
        return 'Enterprise';
      case SubscriptionTier.CUSTOM:
        return 'Custom';
      default:
        return tier;
    }
  };

  return (
    <div className={className}>
      <header className="dashboard-header">
        <h1>Tenant Dashboard</h1>
        <p className="subtitle">Welcome back, {user.displayName}</p>
      </header>

      <div className="dashboard-grid">
        {/* Tenant Overview Card */}
        <section className="dashboard-card" aria-labelledby="tenant-overview-heading">
          <h2 id="tenant-overview-heading">Tenant Overview</h2>
          <div className="card-content">
            <div className="info-row">
              <span className="label">Name:</span>
              <span className="value">{tenant.name}</span>
            </div>
            <div className="info-row">
              <span className="label">Slug:</span>
              <span className="value">{tenant.slug}</span>
            </div>
            {tenant.domain && (
              <div className="info-row">
                <span className="label">Domain:</span>
                <span className="value">{tenant.domain}</span>
              </div>
            )}
            {tenant.customDomain && (
              <div className="info-row">
                <span className="label">Custom Domain:</span>
                <span className="value">{tenant.customDomain}</span>
              </div>
            )}
            <div className="info-row">
              <span className="label">Created:</span>
              <span className="value">
                {new Date(tenant.createdAt).toLocaleDateString()}
              </span>
            </div>
          </div>
        </section>

        {/* Subscription Card */}
        <section className="dashboard-card" aria-labelledby="subscription-heading">
          <h2 id="subscription-heading">Subscription</h2>
          <div className="card-content">
            <div className="info-row">
              <span className="label">Tier:</span>
              <span className="value tier-badge">
                {getTierDisplayName(tenant.subscriptionTier)}
              </span>
            </div>
            <div className="info-row">
              <span className="label">Status:</span>
              <span className={`value status-badge ${getSubscriptionStatusColor(tenant.subscriptionStatus)}`}>
                {tenant.subscriptionStatus}
              </span>
            </div>
            {tenant.trialEndsAt && (
              <div className="info-row">
                <span className="label">Trial Ends:</span>
                <span className="value">
                  {new Date(tenant.trialEndsAt).toLocaleDateString()}
                </span>
              </div>
            )}
            {tenant.subscriptionEndsAt && (
              <div className="info-row">
                <span className="label">Subscription Ends:</span>
                <span className="value">
                  {new Date(tenant.subscriptionEndsAt).toLocaleDateString()}
                </span>
              </div>
            )}
          </div>
        </section>

        {/* Usage Limits Card */}
        <section className="dashboard-card" aria-labelledby="usage-limits-heading">
          <h2 id="usage-limits-heading">Usage Limits</h2>
          <div className="card-content">
            <div className="info-row">
              <span className="label">Max Organizations:</span>
              <span className="value">{tenant.maxOrganizations}</span>
            </div>
            <div className="info-row">
              <span className="label">Max Users:</span>
              <span className="value">{tenant.maxUsers}</span>
            </div>
          </div>
        </section>

        {/* Features Card */}
        <section className="dashboard-card" aria-labelledby="features-heading">
          <h2 id="features-heading">Enabled Features</h2>
          <div className="card-content">
            {tenant.features.length > 0 ? (
              <ul className="feature-list">
                {tenant.features.map((feature, index) => (
                  <li key={index} className="feature-item">
                    <span className="feature-icon" aria-hidden="true">✓</span>
                    {feature}
                  </li>
                ))}
              </ul>
            ) : (
              <p>No additional features enabled</p>
            )}
          </div>
        </section>

        {/* Settings Summary Card */}
        <section className="dashboard-card" aria-labelledby="settings-heading">
          <h2 id="settings-heading">Settings Summary</h2>
          <div className="card-content">
            <div className="info-row">
              <span className="label">Custom Domains:</span>
              <span className="value">
                {tenant.settings.allowCustomDomains ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div className="info-row">
              <span className="label">SSO Configuration:</span>
              <span className="value">
                {tenant.settings.allowSSOConfiguration ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div className="info-row">
              <span className="label">User Registration:</span>
              <span className="value">
                {tenant.settings.allowUserRegistration ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div className="info-row">
              <span className="label">Session Timeout:</span>
              <span className="value">{tenant.settings.sessionTimeout} minutes</span>
            </div>
          </div>
        </section>

        {/* Quick Actions Card */}
        {canManageTenant && (
          <section className="dashboard-card" aria-labelledby="actions-heading">
            <h2 id="actions-heading">Quick Actions</h2>
            <div className="card-content">
              <div className="action-buttons">
                <button
                  type="button"
                  className="btn btn-primary"
                  aria-label="Manage tenant settings"
                >
                  Manage Settings
                </button>
                <button
                  type="button"
                  className="btn btn-secondary"
                  aria-label="View organizations"
                >
                  View Organizations
                </button>
                <button
                  type="button"
                  className="btn btn-secondary"
                  aria-label="Manage users"
                >
                  Manage Users
                </button>
                <button
                  type="button"
                  className="btn btn-secondary"
                  aria-label="View billing"
                >
                  View Billing
                </button>
              </div>
            </div>
          </section>
        )}
      </div>
    </div>
  );
};

export default TenantDashboard;
