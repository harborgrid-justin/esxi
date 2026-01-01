/**
 * Checkout Flow - Subscription checkout process
 */

import React, { useState } from 'react';
import { Plan, Tenant, PaymentMethod, ProrationPreview } from '../types';
import { Decimal } from 'decimal.js';

export interface CheckoutFlowProps {
  plan: Plan;
  tenant?: Tenant;
  prorationPreview?: ProrationPreview;
  onComplete: (data: CheckoutData) => Promise<void>;
  onCancel: () => void;
}

export interface CheckoutData {
  planId: string;
  quantity: number;
  paymentMethodId?: string;
  billingEmail: string;
  billingAddress?: {
    line1: string;
    city: string;
    postalCode: string;
    country: string;
  };
}

export const CheckoutFlow: React.FC<CheckoutFlowProps> = ({
  plan,
  tenant,
  prorationPreview,
  onComplete,
  onCancel,
}) => {
  const [step, setStep] = useState<'details' | 'payment' | 'confirm'>('details');
  const [quantity, setQuantity] = useState(1);
  const [billingEmail, setBillingEmail] = useState(tenant?.billingEmail || '');
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const calculateTotal = (): Decimal => {
    if (prorationPreview) {
      return prorationPreview.immediateCharge;
    }
    return plan.amount.times(quantity);
  };

  const handleSubmit = async () => {
    setProcessing(true);
    setError(null);

    try {
      await onComplete({
        planId: plan.id,
        quantity,
        billingEmail,
      });
    } catch (err: any) {
      setError(err.message);
      setProcessing(false);
    }
  };

  return (
    <div className="checkout-flow">
      <div className="checkout-header">
        <h2>Complete Your Purchase</h2>
        <div className="steps">
          <div className={`step ${step === 'details' ? 'active' : 'completed'}`}>
            1. Details
          </div>
          <div className={`step ${step === 'payment' ? 'active' : ''}`}>
            2. Payment
          </div>
          <div className={`step ${step === 'confirm' ? 'active' : ''}`}>
            3. Confirm
          </div>
        </div>
      </div>

      <div className="checkout-content">
        <div className="checkout-main">
          {step === 'details' && (
            <div className="step-details">
              <h3>Subscription Details</h3>

              <div className="form-group">
                <label>Billing Email</label>
                <input
                  type="email"
                  value={billingEmail}
                  onChange={(e) => setBillingEmail(e.target.value)}
                  placeholder="billing@company.com"
                  required
                />
              </div>

              {plan.pricingModel === 'per_seat' && (
                <div className="form-group">
                  <label>Number of Seats</label>
                  <input
                    type="number"
                    min="1"
                    value={quantity}
                    onChange={(e) => setQuantity(parseInt(e.target.value) || 1)}
                  />
                </div>
              )}

              {plan.trialDays && plan.trialDays > 0 && (
                <div className="trial-notice">
                  You will receive a {plan.trialDays} day free trial. Your card
                  will not be charged until the trial ends.
                </div>
              )}

              <button
                className="btn-primary"
                onClick={() => setStep('payment')}
                disabled={!billingEmail}
              >
                Continue to Payment
              </button>
            </div>
          )}

          {step === 'payment' && (
            <div className="step-payment">
              <h3>Payment Method</h3>

              <div className="payment-methods">
                <div className="payment-option">
                  <input type="radio" name="payment" id="card" defaultChecked />
                  <label htmlFor="card">Credit/Debit Card</label>
                </div>

                <div className="card-form">
                  <div className="form-group">
                    <label>Card Number</label>
                    <input type="text" placeholder="4242 4242 4242 4242" />
                  </div>

                  <div className="form-row">
                    <div className="form-group">
                      <label>Expiry</label>
                      <input type="text" placeholder="MM/YY" />
                    </div>
                    <div className="form-group">
                      <label>CVC</label>
                      <input type="text" placeholder="123" />
                    </div>
                  </div>
                </div>
              </div>

              <div className="button-group">
                <button className="btn-secondary" onClick={() => setStep('details')}>
                  Back
                </button>
                <button className="btn-primary" onClick={() => setStep('confirm')}>
                  Continue
                </button>
              </div>
            </div>
          )}

          {step === 'confirm' && (
            <div className="step-confirm">
              <h3>Confirm Your Order</h3>

              <div className="order-summary">
                <div className="summary-item">
                  <span>Plan</span>
                  <span>{plan.name}</span>
                </div>
                <div className="summary-item">
                  <span>Quantity</span>
                  <span>{quantity}</span>
                </div>
                <div className="summary-item total">
                  <span>Total Due Today</span>
                  <span>${calculateTotal().toFixed(2)}</span>
                </div>
              </div>

              {error && <div className="error-message">{error}</div>}

              <div className="button-group">
                <button className="btn-secondary" onClick={() => setStep('payment')}>
                  Back
                </button>
                <button
                  className="btn-primary"
                  onClick={handleSubmit}
                  disabled={processing}
                >
                  {processing ? 'Processing...' : 'Complete Purchase'}
                </button>
              </div>
            </div>
          )}
        </div>

        <div className="checkout-sidebar">
          <div className="order-summary-card">
            <h4>Order Summary</h4>
            <div className="summary-details">
              <div className="item">
                <span>{plan.name}</span>
                <span>${plan.amount.toFixed(2)}</span>
              </div>
              {quantity > 1 && (
                <div className="item">
                  <span>× {quantity}</span>
                  <span>${plan.amount.times(quantity).toFixed(2)}</span>
                </div>
              )}
              {prorationPreview && (
                <>
                  <div className="item credit">
                    <span>Proration Credit</span>
                    <span>-${prorationPreview.creditApplied.toFixed(2)}</span>
                  </div>
                </>
              )}
              <div className="divider"></div>
              <div className="item total">
                <span>Total</span>
                <span>${calculateTotal().toFixed(2)}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <style jsx>{`
        .checkout-flow {
          max-width: 1000px;
          margin: 0 auto;
          padding: 2rem;
        }

        .checkout-header {
          margin-bottom: 2rem;
        }

        .checkout-header h2 {
          margin: 0 0 1rem 0;
        }

        .steps {
          display: flex;
          gap: 1rem;
        }

        .step {
          padding: 0.5rem 1rem;
          background: #f0f0f0;
          border-radius: 4px;
          font-weight: 600;
        }

        .step.active {
          background: #007bff;
          color: white;
        }

        .step.completed {
          background: #28a745;
          color: white;
        }

        .checkout-content {
          display: grid;
          grid-template-columns: 2fr 1fr;
          gap: 2rem;
        }

        .checkout-main {
          background: white;
          padding: 2rem;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .checkout-sidebar {
          position: sticky;
          top: 2rem;
          height: fit-content;
        }

        .order-summary-card {
          background: white;
          padding: 1.5rem;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 600;
        }

        .form-group input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-size: 1rem;
        }

        .form-row {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 1rem;
        }

        .trial-notice {
          padding: 1rem;
          background: #e7f3ff;
          border-left: 4px solid #007bff;
          margin-bottom: 1.5rem;
        }

        .button-group {
          display: flex;
          gap: 1rem;
          margin-top: 2rem;
        }

        .btn-primary,
        .btn-secondary {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 4px;
          font-size: 1rem;
          font-weight: 600;
          cursor: pointer;
        }

        .btn-primary {
          background: #007bff;
          color: white;
          flex: 1;
        }

        .btn-primary:hover:not(:disabled) {
          background: #0056b3;
        }

        .btn-primary:disabled {
          background: #ccc;
          cursor: not-allowed;
        }

        .btn-secondary {
          background: #6c757d;
          color: white;
        }

        .order-summary .summary-item,
        .summary-details .item {
          display: flex;
          justify-content: space-between;
          padding: 0.75rem 0;
        }

        .summary-item.total,
        .item.total {
          border-top: 2px solid #333;
          font-weight: 700;
          font-size: 1.25rem;
          margin-top: 1rem;
          padding-top: 1rem;
        }

        .item.credit {
          color: #28a745;
        }

        .divider {
          border-top: 1px solid #ddd;
          margin: 1rem 0;
        }

        .error-message {
          padding: 1rem;
          background: #f8d7da;
          color: #721c24;
          border-radius: 4px;
          margin-bottom: 1rem;
        }

        @media (max-width: 768px) {
          .checkout-content {
            grid-template-columns: 1fr;
          }

          .checkout-sidebar {
            position: static;
          }
        }
      `}</style>
    </div>
  );
};
