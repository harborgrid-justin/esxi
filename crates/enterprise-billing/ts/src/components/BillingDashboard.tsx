/**
 * Billing Dashboard - Overview of billing and subscription status
 */

import React from 'react';
import { Subscription, Invoice, Plan, UsageSummary } from '../types';
import { Decimal } from 'decimal.js';
import { format } from 'date-fns';

export interface BillingDashboardProps {
  subscription?: Subscription;
  plan?: Plan;
  upcomingInvoice?: Invoice;
  usageSummary?: UsageSummary;
  onManageSubscription: () => void;
  onViewInvoices: () => void;
  onUpdatePayment: () => void;
}

export const BillingDashboard: React.FC<BillingDashboardProps> = ({
  subscription,
  plan,
  upcomingInvoice,
  usageSummary,
  onManageSubscription,
  onViewInvoices,
  onUpdatePayment,
}) => {
  if (!subscription || !plan) {
    return (
      <div className="billing-dashboard">
        <div className="no-subscription">
          <h2>No Active Subscription</h2>
          <p>Subscribe to a plan to get started.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="billing-dashboard">
      <h1>Billing Dashboard</h1>

      <div className="dashboard-grid">
        <div className="card current-plan">
          <h3>Current Plan</h3>
          <div className="plan-info">
            <div className="plan-name">{plan.name}</div>
            <div className="plan-price">${plan.amount.toFixed(2)}/month</div>
            <div className="plan-status status-{subscription.status}">
              {subscription.status.toUpperCase()}
            </div>
          </div>
          <button className="btn-outline" onClick={onManageSubscription}>
            Manage Subscription
          </button>
        </div>

        <div className="card billing-cycle">
          <h3>Billing Cycle</h3>
          <div className="cycle-info">
            <div className="date-range">
              {format(subscription.currentPeriodStart, 'MMM d')} -{' '}
              {format(subscription.currentPeriodEnd, 'MMM d, yyyy')}
            </div>
            <div className="next-billing">
              Next billing: {format(subscription.currentPeriodEnd, 'MMM d, yyyy')}
            </div>
          </div>
        </div>

        {upcomingInvoice && (
          <div className="card upcoming-invoice">
            <h3>Upcoming Invoice</h3>
            <div className="invoice-amount">${upcomingInvoice.total.toFixed(2)}</div>
            <div className="invoice-date">
              Due {format(upcomingInvoice.dueDate, 'MMM d, yyyy')}
            </div>
            <button className="btn-link" onClick={onViewInvoices}>
              View all invoices
            </button>
          </div>
        )}

        {usageSummary && (
          <div className="card usage-summary">
            <h3>Current Usage</h3>
            <div className="usage-items">
              {usageSummary.components.map((component) => (
                <div key={component.meteredComponentId} className="usage-item">
                  <span>{component.componentName}</span>
                  <span>
                    {component.totalUsage} {component.unit}
                  </span>
                </div>
              ))}
            </div>
            <div className="estimated-cost">
              Estimated: ${usageSummary.estimatedCost.toFixed(2)}
            </div>
          </div>
        )}
      </div>

      <div className="quick-actions">
        <h3>Quick Actions</h3>
        <div className="action-buttons">
          <button onClick={onUpdatePayment}>Update Payment Method</button>
          <button onClick={onViewInvoices}>View Invoices</button>
          <button onClick={onManageSubscription}>Change Plan</button>
        </div>
      </div>

      <style jsx>{`
        .billing-dashboard {
          padding: 2rem;
          max-width: 1200px;
          margin: 0 auto;
        }

        h1 {
          margin-bottom: 2rem;
        }

        .dashboard-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: 1.5rem;
          margin-bottom: 2rem;
        }

        .card {
          background: white;
          padding: 1.5rem;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .card h3 {
          margin: 0 0 1rem 0;
          font-size: 1.1rem;
          color: #666;
        }

        .plan-name {
          font-size: 1.5rem;
          font-weight: 700;
          margin-bottom: 0.5rem;
        }

        .plan-price {
          font-size: 1.25rem;
          color: #007bff;
          margin-bottom: 0.5rem;
        }

        .plan-status {
          display: inline-block;
          padding: 0.25rem 0.75rem;
          border-radius: 12px;
          font-size: 0.85rem;
          font-weight: 600;
          margin-bottom: 1rem;
        }

        .status-active {
          background: #d4edda;
          color: #155724;
        }

        .status-trial {
          background: #d1ecf1;
          color: #0c5460;
        }

        .invoice-amount {
          font-size: 2rem;
          font-weight: 700;
          margin-bottom: 0.5rem;
        }

        .usage-items {
          margin-bottom: 1rem;
        }

        .usage-item {
          display: flex;
          justify-content: space-between;
          padding: 0.5rem 0;
          border-bottom: 1px solid #f0f0f0;
        }

        .estimated-cost {
          font-weight: 700;
          color: #007bff;
          margin-top: 1rem;
        }

        .action-buttons {
          display: flex;
          gap: 1rem;
          flex-wrap: wrap;
        }

        .action-buttons button {
          padding: 0.75rem 1.5rem;
          background: white;
          border: 2px solid #007bff;
          color: #007bff;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }

        .action-buttons button:hover {
          background: #007bff;
          color: white;
        }

        .btn-outline,
        .btn-link {
          padding: 0.5rem 1rem;
          border: 1px solid #007bff;
          background: transparent;
          color: #007bff;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }

        .btn-link {
          border: none;
          padding: 0;
        }
      `}</style>
    </div>
  );
};
