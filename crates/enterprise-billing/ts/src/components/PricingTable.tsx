/**
 * Pricing Table - Display plan comparison and pricing
 */

import React from 'react';
import { Plan, PlanInterval, PricingModel } from '../types';
import { Decimal } from 'decimal.js';

export interface PricingTableProps {
  plans: Plan[];
  currentPlanId?: string;
  onSelectPlan: (plan: Plan) => void;
  highlightPlanId?: string;
  showAnnualSavings?: boolean;
  currency?: string;
}

export const PricingTable: React.FC<PricingTableProps> = ({
  plans,
  currentPlanId,
  onSelectPlan,
  highlightPlanId,
  showAnnualSavings = true,
  currency = 'USD',
}) => {
  const formatPrice = (amount: Decimal, interval: PlanInterval): string => {
    const price = amount.toFixed(2);

    switch (interval) {
      case PlanInterval.MONTHLY:
        return `$${price}/month`;
      case PlanInterval.QUARTERLY:
        return `$${price}/quarter`;
      case PlanInterval.YEARLY:
        return `$${price}/year`;
      case PlanInterval.LIFETIME:
        return `$${price} one-time`;
      default:
        return `$${price}`;
    }
  };

  const calculateMonthlySavings = (plan: Plan): string | null => {
    if (!showAnnualSavings || plan.interval !== PlanInterval.YEARLY) {
      return null;
    }

    const monthlyEquivalent = plan.amount.dividedBy(12);
    const monthlyPlans = plans.filter(
      (p) => p.interval === PlanInterval.MONTHLY && p.name === plan.name
    );

    if (monthlyPlans.length === 0) return null;

    const monthlyPlan = monthlyPlans[0];
    const savings = monthlyPlan.amount.times(12).minus(plan.amount);
    const savingsPercent = savings.dividedBy(monthlyPlan.amount.times(12)).times(100);

    return `Save ${savingsPercent.toFixed(0)}% ($${savings.toFixed(2)}/year)`;
  };

  return (
    <div className="pricing-table">
      <div className="pricing-grid">
        {plans.map((plan) => {
          const isCurrentPlan = plan.id === currentPlanId;
          const isHighlighted = plan.id === highlightPlanId;
          const savings = calculateMonthlySavings(plan);

          return (
            <div
              key={plan.id}
              className={`pricing-card ${isHighlighted ? 'highlighted' : ''} ${
                isCurrentPlan ? 'current' : ''
              }`}
            >
              {isHighlighted && (
                <div className="badge-popular">Most Popular</div>
              )}

              <div className="plan-header">
                <h3 className="plan-name">{plan.name}</h3>
                <p className="plan-description">{plan.description}</p>
              </div>

              <div className="plan-pricing">
                <div className="price">
                  {plan.pricingModel === PricingModel.FLAT_RATE ||
                  plan.pricingModel === PricingModel.PER_SEAT ? (
                    <>
                      <span className="amount">
                        {formatPrice(plan.amount, plan.interval)}
                      </span>
                      {plan.pricingModel === PricingModel.PER_SEAT && (
                        <span className="per-seat">/seat</span>
                      )}
                    </>
                  ) : (
                    <span className="amount">Custom Pricing</span>
                  )}
                </div>

                {savings && <div className="savings">{savings}</div>}

                {plan.trialDays && plan.trialDays > 0 && (
                  <div className="trial-info">{plan.trialDays} day free trial</div>
                )}
              </div>

              <div className="plan-features">
                <ul>
                  {plan.features.map((feature) => (
                    <li key={feature.id} className={feature.enabled ? '' : 'disabled'}>
                      <span className="feature-icon">
                        {feature.enabled ? '✓' : '×'}
                      </span>
                      <span className="feature-name">{feature.name}</span>
                      {feature.quota !== undefined && !feature.unlimited && (
                        <span className="feature-quota">({feature.quota})</span>
                      )}
                      {feature.unlimited && (
                        <span className="feature-unlimited">(Unlimited)</span>
                      )}
                    </li>
                  ))}
                </ul>
              </div>

              {plan.meteredComponents && plan.meteredComponents.length > 0 && (
                <div className="metered-components">
                  <h4>Usage-Based Pricing</h4>
                  <ul>
                    {plan.meteredComponents.map((component) => (
                      <li key={component.id}>
                        {component.name}:{' '}
                        {component.unitAmount
                          ? `$${component.unitAmount.toFixed(4)}/${component.unit}`
                          : 'Tiered pricing'}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              <div className="plan-action">
                {isCurrentPlan ? (
                  <button className="btn-current" disabled>
                    Current Plan
                  </button>
                ) : (
                  <button
                    className="btn-select"
                    onClick={() => onSelectPlan(plan)}
                  >
                    {currentPlanId ? 'Switch to this plan' : 'Get Started'}
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      <style jsx>{`
        .pricing-table {
          padding: 2rem;
          max-width: 1200px;
          margin: 0 auto;
        }

        .pricing-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 2rem;
        }

        .pricing-card {
          border: 2px solid #e0e0e0;
          border-radius: 8px;
          padding: 2rem;
          background: white;
          position: relative;
          transition: all 0.3s ease;
        }

        .pricing-card:hover {
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          transform: translateY(-4px);
        }

        .pricing-card.highlighted {
          border-color: #007bff;
          box-shadow: 0 0 20px rgba(0, 123, 255, 0.2);
        }

        .pricing-card.current {
          border-color: #28a745;
        }

        .badge-popular {
          position: absolute;
          top: -12px;
          left: 50%;
          transform: translateX(-50%);
          background: #007bff;
          color: white;
          padding: 4px 16px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 600;
        }

        .plan-header {
          margin-bottom: 1.5rem;
        }

        .plan-name {
          font-size: 1.5rem;
          font-weight: 700;
          margin: 0 0 0.5rem 0;
        }

        .plan-description {
          color: #666;
          margin: 0;
        }

        .plan-pricing {
          margin-bottom: 2rem;
          padding-bottom: 2rem;
          border-bottom: 1px solid #e0e0e0;
        }

        .price {
          margin-bottom: 0.5rem;
        }

        .amount {
          font-size: 2rem;
          font-weight: 700;
          color: #333;
        }

        .per-seat {
          font-size: 1rem;
          color: #666;
          margin-left: 0.5rem;
        }

        .savings {
          color: #28a745;
          font-weight: 600;
          font-size: 0.9rem;
        }

        .trial-info {
          color: #007bff;
          font-weight: 600;
          font-size: 0.9rem;
          margin-top: 0.5rem;
        }

        .plan-features ul {
          list-style: none;
          padding: 0;
          margin: 0 0 2rem 0;
        }

        .plan-features li {
          padding: 0.5rem 0;
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .plan-features li.disabled {
          color: #999;
          text-decoration: line-through;
        }

        .feature-icon {
          font-weight: 700;
          color: #28a745;
        }

        .plan-features li.disabled .feature-icon {
          color: #dc3545;
        }

        .feature-quota,
        .feature-unlimited {
          margin-left: auto;
          font-size: 0.85rem;
          color: #666;
        }

        .metered-components {
          margin-bottom: 2rem;
          padding: 1rem;
          background: #f8f9fa;
          border-radius: 4px;
        }

        .metered-components h4 {
          margin: 0 0 1rem 0;
          font-size: 1rem;
        }

        .metered-components ul {
          list-style: none;
          padding: 0;
          margin: 0;
          font-size: 0.9rem;
          color: #666;
        }

        .metered-components li {
          padding: 0.25rem 0;
        }

        .plan-action {
          margin-top: auto;
        }

        .btn-select,
        .btn-current {
          width: 100%;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 4px;
          font-size: 1rem;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .btn-select {
          background: #007bff;
          color: white;
        }

        .btn-select:hover {
          background: #0056b3;
        }

        .btn-current {
          background: #28a745;
          color: white;
          cursor: default;
        }

        .pricing-card.highlighted .btn-select {
          background: #007bff;
          box-shadow: 0 4px 8px rgba(0, 123, 255, 0.3);
        }
      `}</style>
    </div>
  );
};
