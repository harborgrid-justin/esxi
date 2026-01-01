/**
 * Payment Method Manager - Manage payment methods
 */

import React, { useState } from 'react';
import { PaymentMethod, PaymentMethodType } from '../types';

export interface PaymentMethodManagerProps {
  paymentMethods: PaymentMethod[];
  onAddPaymentMethod: () => void;
  onSetDefault: (paymentMethod: PaymentMethod) => void;
  onRemove: (paymentMethod: PaymentMethod) => void;
}

export const PaymentMethodManager: React.FC<PaymentMethodManagerProps> = ({
  paymentMethods,
  onAddPaymentMethod,
  onSetDefault,
  onRemove,
}) => {
  const getCardBrandIcon = (brand: string): string => {
    switch (brand.toLowerCase()) {
      case 'visa':
        return '💳';
      case 'mastercard':
        return '💳';
      case 'amex':
        return '💳';
      default:
        return '💳';
    }
  };

  const formatPaymentMethod = (pm: PaymentMethod): string => {
    if (pm.card) {
      return `${pm.card.brand} •••• ${pm.card.last4}`;
    }
    if (pm.bankAccount) {
      return `${pm.bankAccount.bankName} •••• ${pm.bankAccount.last4}`;
    }
    if (pm.paypal) {
      return pm.paypal.email;
    }
    return 'Unknown';
  };

  const getExpiryInfo = (pm: PaymentMethod): string | null => {
    if (pm.card) {
      return `Expires ${pm.card.expiryMonth}/${pm.card.expiryYear}`;
    }
    return null;
  };

  return (
    <div className="payment-method-manager">
      <div className="header">
        <h2>Payment Methods</h2>
        <button className="btn-add" onClick={onAddPaymentMethod}>
          + Add Payment Method
        </button>
      </div>

      <div className="payment-methods">
        {paymentMethods.length === 0 ? (
          <div className="no-methods">
            <p>No payment methods added yet.</p>
            <button className="btn-primary" onClick={onAddPaymentMethod}>
              Add Your First Payment Method
            </button>
          </div>
        ) : (
          paymentMethods.map((pm) => (
            <div key={pm.id} className={`payment-method ${pm.isDefault ? 'default' : ''}`}>
              <div className="pm-icon">
                {pm.card && getCardBrandIcon(pm.card.brand)}
                {pm.type === PaymentMethodType.BANK_ACCOUNT && '🏦'}
                {pm.type === PaymentMethodType.PAYPAL && 'P'}
              </div>

              <div className="pm-details">
                <div className="pm-name">{formatPaymentMethod(pm)}</div>
                {getExpiryInfo(pm) && (
                  <div className="pm-expiry">{getExpiryInfo(pm)}</div>
                )}
                {pm.isDefault && (
                  <div className="default-badge">Default</div>
                )}
              </div>

              <div className="pm-actions">
                {!pm.isDefault && (
                  <button
                    className="btn-link"
                    onClick={() => onSetDefault(pm)}
                  >
                    Set as Default
                  </button>
                )}
                <button
                  className="btn-link danger"
                  onClick={() => onRemove(pm)}
                >
                  Remove
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      <style jsx>{`
        .payment-method-manager {
          padding: 2rem;
          max-width: 800px;
          margin: 0 auto;
        }

        .header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .btn-add {
          padding: 0.75rem 1.5rem;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }

        .payment-methods {
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }

        .payment-method {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1.5rem;
          background: white;
          border: 2px solid #e0e0e0;
          border-radius: 8px;
        }

        .payment-method.default {
          border-color: #007bff;
        }

        .pm-icon {
          font-size: 2rem;
          width: 3rem;
          height: 3rem;
          display: flex;
          align-items: center;
          justify-content: center;
          background: #f8f9fa;
          border-radius: 8px;
        }

        .pm-details {
          flex: 1;
        }

        .pm-name {
          font-weight: 600;
          margin-bottom: 0.25rem;
        }

        .pm-expiry {
          font-size: 0.875rem;
          color: #666;
        }

        .default-badge {
          display: inline-block;
          padding: 0.25rem 0.75rem;
          background: #007bff;
          color: white;
          border-radius: 12px;
          font-size: 0.75rem;
          font-weight: 600;
          margin-top: 0.5rem;
        }

        .pm-actions {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .btn-link {
          background: none;
          border: none;
          color: #007bff;
          cursor: pointer;
          font-weight: 600;
          padding: 0.25rem;
        }

        .btn-link.danger {
          color: #dc3545;
        }

        .no-methods {
          text-align: center;
          padding: 3rem;
        }

        .btn-primary {
          padding: 0.75rem 1.5rem;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
};
