/**
 * Subscription Manager - Manage subscription settings
 */

import React, { useState } from 'react';
import { Subscription, Plan, SubscriptionStatus } from '../types';
import { format } from 'date-fns';

export interface SubscriptionManagerProps {
  subscription: Subscription;
  plan: Plan;
  availablePlans: Plan[];
  onChangePlan: (newPlan: Plan) => void;
  onUpdateQuantity: (quantity: number) => void;
  onCancelSubscription: () => void;
  onReactivateSubscription: () => void;
}

export const SubscriptionManager: React.FC<SubscriptionManagerProps> = ({
  subscription,
  plan,
  availablePlans,
  onChangePlan,
  onUpdateQuantity,
  onCancelSubscription,
  onReactivateSubscription,
}) => {
  const [quantity, setQuantity] = useState(subscription.quantity);
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);

  const handleQuantityUpdate = () => {
    if (quantity !== subscription.quantity && quantity > 0) {
      onUpdateQuantity(quantity);
    }
  };

  const isCanceled = subscription.status === SubscriptionStatus.CANCELED;
  const canModify = !isCanceled;

  return (
    <div className="subscription-manager">
      <h2>Subscription Settings</h2>

      <div className="settings-grid">
        <div className="setting-card">
          <h3>Current Plan</h3>
          <div className="plan-info">
            <div className="plan-name">{plan.name}</div>
            <div className="plan-price">${plan.amount.toFixed(2)}/month</div>
          </div>

          {canModify && (
            <div className="available-plans">
              <h4>Switch to a different plan</h4>
              <div className="plan-options">
                {availablePlans
                  .filter((p) => p.id !== plan.id)
                  .map((p) => (
                    <div key={p.id} className="plan-option">
                      <div>
                        <div className="option-name">{p.name}</div>
                        <div className="option-price">
                          ${p.amount.toFixed(2)}/month
                        </div>
                      </div>
                      <button
                        className="btn-sm"
                        onClick={() => onChangePlan(p)}
                      >
                        Switch
                      </button>
                    </div>
                  ))}
              </div>
            </div>
          )}
        </div>

        {plan.pricingModel === 'per_seat' && canModify && (
          <div className="setting-card">
            <h3>Quantity</h3>
            <div className="quantity-controls">
              <div className="quantity-input">
                <button onClick={() => setQuantity(Math.max(1, quantity - 1))}>
                  -
                </button>
                <input
                  type="number"
                  min="1"
                  value={quantity}
                  onChange={(e) => setQuantity(parseInt(e.target.value) || 1)}
                />
                <button onClick={() => setQuantity(quantity + 1)}>+</button>
              </div>
              {quantity !== subscription.quantity && (
                <button className="btn-primary" onClick={handleQuantityUpdate}>
                  Update Quantity
                </button>
              )}
            </div>
            <div className="quantity-info">
              Current: {subscription.quantity} seat(s)
            </div>
          </div>
        )}

        <div className="setting-card">
          <h3>Billing Cycle</h3>
          <div className="cycle-info">
            <div className="info-row">
              <span>Current Period:</span>
              <span>
                {format(subscription.currentPeriodStart, 'MMM d')} -{' '}
                {format(subscription.currentPeriodEnd, 'MMM d, yyyy')}
              </span>
            </div>
            {subscription.trialEnd && (
              <div className="info-row">
                <span>Trial Ends:</span>
                <span>{format(subscription.trialEnd, 'MMM d, yyyy')}</span>
              </div>
            )}
            <div className="info-row">
              <span>Status:</span>
              <span className={`status status-${subscription.status}`}>
                {subscription.status.toUpperCase()}
              </span>
            </div>
          </div>
        </div>

        <div className="setting-card danger-zone">
          <h3>Danger Zone</h3>
          {isCanceled ? (
            <div className="canceled-info">
              <p>Your subscription has been canceled.</p>
              <button
                className="btn-primary"
                onClick={onReactivateSubscription}
              >
                Reactivate Subscription
              </button>
            </div>
          ) : (
            <>
              {!showCancelConfirm ? (
                <button
                  className="btn-danger"
                  onClick={() => setShowCancelConfirm(true)}
                >
                  Cancel Subscription
                </button>
              ) : (
                <div className="cancel-confirm">
                  <p>Are you sure you want to cancel your subscription?</p>
                  <div className="button-group">
                    <button
                      className="btn-danger"
                      onClick={() => {
                        onCancelSubscription();
                        setShowCancelConfirm(false);
                      }}
                    >
                      Yes, Cancel
                    </button>
                    <button
                      className="btn-secondary"
                      onClick={() => setShowCancelConfirm(false)}
                    >
                      Nevermind
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>

      <style jsx>{`
        .subscription-manager {
          padding: 2rem;
          max-width: 1200px;
          margin: 0 auto;
        }

        h2 {
          margin-bottom: 2rem;
        }

        .settings-grid {
          display: grid;
          gap: 1.5rem;
        }

        .setting-card {
          background: white;
          padding: 1.5rem;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .setting-card h3 {
          margin: 0 0 1rem 0;
        }

        .plan-name {
          font-size: 1.25rem;
          font-weight: 700;
        }

        .plan-price {
          color: #007bff;
          font-size: 1.1rem;
          margin-top: 0.5rem;
        }

        .available-plans h4 {
          margin: 1.5rem 0 1rem 0;
          font-size: 1rem;
        }

        .plan-options {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .plan-option {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem;
          border: 1px solid #e0e0e0;
          border-radius: 4px;
        }

        .option-name {
          font-weight: 600;
        }

        .option-price {
          color: #666;
          font-size: 0.875rem;
        }

        .quantity-controls {
          display: flex;
          gap: 1rem;
          align-items: center;
        }

        .quantity-input {
          display: flex;
          gap: 0.5rem;
        }

        .quantity-input button {
          width: 2.5rem;
          height: 2.5rem;
          border: 1px solid #ddd;
          background: white;
          cursor: pointer;
          border-radius: 4px;
          font-size: 1.25rem;
        }

        .quantity-input input {
          width: 4rem;
          text-align: center;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-size: 1rem;
        }

        .quantity-info {
          margin-top: 1rem;
          color: #666;
        }

        .info-row {
          display: flex;
          justify-content: space-between;
          padding: 0.75rem 0;
          border-bottom: 1px solid #f0f0f0;
        }

        .status {
          padding: 0.25rem 0.75rem;
          border-radius: 12px;
          font-size: 0.85rem;
          font-weight: 600;
        }

        .status-active {
          background: #d4edda;
          color: #155724;
        }

        .status-trial {
          background: #d1ecf1;
          color: #0c5460;
        }

        .status-canceled {
          background: #f8d7da;
          color: #721c24;
        }

        .danger-zone {
          border: 2px solid #dc3545;
        }

        .btn-danger {
          padding: 0.75rem 1.5rem;
          background: #dc3545;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }

        .cancel-confirm {
          padding: 1rem;
          background: #f8d7da;
          border-radius: 4px;
        }

        .button-group {
          display: flex;
          gap: 1rem;
          margin-top: 1rem;
        }

        .btn-primary,
        .btn-secondary,
        .btn-sm {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }

        .btn-primary {
          background: #007bff;
          color: white;
        }

        .btn-secondary {
          background: #6c757d;
          color: white;
        }

        .btn-sm {
          padding: 0.5rem 1rem;
          background: #007bff;
          color: white;
        }
      `}</style>
    </div>
  );
};
