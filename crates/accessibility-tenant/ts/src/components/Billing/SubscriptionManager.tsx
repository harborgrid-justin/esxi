/**
 * Subscription Manager Component
 * Manage subscription tier and billing cycle
 */

import React, { useState, useEffect } from 'react';
import { Subscription, SubscriptionTier, SubscriptionStatus } from '../../types';
import BillingService from '../../services/BillingService';
import { usePermissions } from '../../hooks/usePermissions';

interface SubscriptionManagerProps {
  billingService: BillingService;
  tenantId: string;
  className?: string;
}

export const SubscriptionManager: React.FC<SubscriptionManagerProps> = ({
  billingService,
  tenantId,
  className,
}) => {
  const [subscription, setSubscription] = useState<Subscription | null>(null);
  const [plans, setPlans] = useState<any[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedTier, setSelectedTier] = useState<SubscriptionTier | null>(null);
  const [billingCycle, setBillingCycle] = useState<'monthly' | 'yearly'>('monthly');
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string>('');
  const { can } = usePermissions();

  const canViewBilling = can.viewBilling();

  useEffect(() => {
    loadData();
  }, [tenantId]);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [subResponse, plansResponse] = await Promise.all([
        billingService.getSubscription(tenantId),
        billingService.getPricingPlans(),
      ]);

      if (subResponse.success && subResponse.data) {
        setSubscription(subResponse.data);
      }
      if (plansResponse.success && plansResponse.data) {
        setPlans(plansResponse.data);
      }
    } catch (err) {
      setError('Failed to load subscription data');
    } finally {
      setIsLoading(false);
    }
  };

  const handleUpdateTier = async () => {
    if (!selectedTier) return;

    setIsUpdating(true);
    setError('');

    try {
      const response = await billingService.updateSubscriptionTier(
        tenantId,
        selectedTier,
        billingCycle
      );

      if (response.success && response.data) {
        setSubscription(response.data);
        setSelectedTier(null);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update subscription');
    } finally {
      setIsUpdating(false);
    }
  };

  const handleCancelSubscription = async () => {
    if (!confirm('Are you sure you want to cancel your subscription?')) {
      return;
    }

    setIsUpdating(true);
    try {
      const response = await billingService.cancelSubscription(tenantId);
      if (response.success && response.data) {
        setSubscription(response.data);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel subscription');
    } finally {
      setIsUpdating(false);
    }
  };

  if (!canViewBilling) {
    return (
      <div className={className} role="alert">
        <p>You do not have permission to view billing information</p>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className={className} role="status">
        <div className="loading-spinner">Loading...</div>
      </div>
    );
  }

  return (
    <div className={className}>
      <header>
        <h1>Subscription Management</h1>
        <p className="subtitle">Manage your subscription tier and billing</p>
      </header>

      {subscription && (
        <section className="current-subscription">
          <h2>Current Subscription</h2>
          <div className="subscription-card">
            <div className="tier-info">
              <h3>{subscription.tier}</h3>
              <span className={`status-badge ${subscription.status.toLowerCase()}`}>
                {subscription.status}
              </span>
            </div>
            <div className="subscription-details">
              <p>
                <strong>Current Period:</strong>{' '}
                {new Date(subscription.currentPeriodStart).toLocaleDateString()} -{' '}
                {new Date(subscription.currentPeriodEnd).toLocaleDateString()}
              </p>
              {subscription.pricing && (
                <p>
                  <strong>Price:</strong> ${subscription.pricing.basePriceMonthly}/month
                </p>
              )}
            </div>
            {subscription.status === SubscriptionStatus.ACTIVE && (
              <button
                type="button"
                onClick={handleCancelSubscription}
                disabled={isUpdating}
                className="btn btn-danger"
              >
                Cancel Subscription
              </button>
            )}
          </div>
        </section>
      )}

      <section className="plans-section">
        <h2>Available Plans</h2>

        <div className="billing-cycle-toggle">
          <button
            type="button"
            className={`toggle-btn ${billingCycle === 'monthly' ? 'active' : ''}`}
            onClick={() => setBillingCycle('monthly')}
          >
            Monthly
          </button>
          <button
            type="button"
            className={`toggle-btn ${billingCycle === 'yearly' ? 'active' : ''}`}
            onClick={() => setBillingCycle('yearly')}
          >
            Yearly (Save 20%)
          </button>
        </div>

        <div className="plans-grid">
          {plans.map((plan) => (
            <article
              key={plan.tier}
              className={`plan-card ${selectedTier === plan.tier ? 'selected' : ''} ${
                subscription?.tier === plan.tier ? 'current' : ''
              }`}
            >
              <header>
                <h3>{plan.name}</h3>
                <div className="price">
                  <span className="amount">
                    ${billingCycle === 'monthly' ? plan.pricing.monthly : plan.pricing.yearly}
                  </span>
                  <span className="period">/{billingCycle === 'monthly' ? 'month' : 'year'}</span>
                </div>
              </header>
              <p className="plan-description">{plan.description}</p>
              <ul className="features-list">
                {plan.features.map((feature: string, index: number) => (
                  <li key={index}>{feature}</li>
                ))}
              </ul>
              <div className="limits-info">
                <p>Up to {plan.limits.users} users</p>
                <p>Up to {plan.limits.organizations} organizations</p>
                <p>{plan.limits.storage}GB storage</p>
              </div>
              <button
                type="button"
                onClick={() => setSelectedTier(plan.tier)}
                disabled={subscription?.tier === plan.tier}
                className="btn btn-primary"
              >
                {subscription?.tier === plan.tier ? 'Current Plan' : 'Select Plan'}
              </button>
            </article>
          ))}
        </div>
      </section>

      {selectedTier && selectedTier !== subscription?.tier && (
        <div className="confirmation-dialog">
          <p>Update subscription to {selectedTier}?</p>
          <div className="dialog-actions">
            <button
              type="button"
              onClick={handleUpdateTier}
              disabled={isUpdating}
              className="btn btn-primary"
            >
              {isUpdating ? 'Updating...' : 'Confirm Update'}
            </button>
            <button
              type="button"
              onClick={() => setSelectedTier(null)}
              disabled={isUpdating}
              className="btn btn-secondary"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {error && (
        <div className="error-message" role="alert">
          {error}
        </div>
      )}
    </div>
  );
};

export default SubscriptionManager;
