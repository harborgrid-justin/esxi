/**
 * Usage Metrics - Visualize usage and quotas
 */

import React from 'react';
import { QuotaUsage, UsageSummary } from '../types';
import { format, differenceInDays } from 'date-fns';

export interface UsageMetricsProps {
  quotas: QuotaUsage[];
  usageSummary?: UsageSummary;
}

export const UsageMetrics: React.FC<UsageMetricsProps> = ({
  quotas,
  usageSummary,
}) => {
  const getUsagePercentage = (quota: QuotaUsage): number => {
    if (quota.unlimited) return 0;
    return (quota.used / quota.limit) * 100;
  };

  const getStatusColor = (percentage: number): string => {
    if (percentage >= 90) return '#dc3545';
    if (percentage >= 75) return '#ffc107';
    return '#28a745';
  };

  const daysRemaining = usageSummary
    ? differenceInDays(usageSummary.periodEnd, new Date())
    : 0;

  return (
    <div className="usage-metrics">
      <div className="metrics-header">
        <h2>Usage & Quotas</h2>
        {usageSummary && (
          <div className="period-info">
            {format(usageSummary.periodStart, 'MMM d')} -{' '}
            {format(usageSummary.periodEnd, 'MMM d, yyyy')}
            <span className="days-remaining">
              {daysRemaining} days remaining
            </span>
          </div>
        )}
      </div>

      <div className="quotas-grid">
        {quotas.map((quota) => {
          const percentage = getUsagePercentage(quota);
          const color = getStatusColor(percentage);

          return (
            <div key={quota.featureId} className="quota-card">
              <div className="quota-header">
                <h3>{quota.featureId}</h3>
                {quota.unlimited ? (
                  <span className="unlimited-badge">Unlimited</span>
                ) : (
                  <span className="quota-value">
                    {quota.used.toLocaleString()} / {quota.limit.toLocaleString()}
                  </span>
                )}
              </div>

              {!quota.unlimited && (
                <>
                  <div className="progress-bar">
                    <div
                      className="progress-fill"
                      style={{
                        width: `${Math.min(percentage, 100)}%`,
                        backgroundColor: color,
                      }}
                    />
                  </div>
                  <div className="quota-percentage" style={{ color }}>
                    {percentage.toFixed(1)}% used
                  </div>
                </>
              )}
            </div>
          );
        })}
      </div>

      {usageSummary && usageSummary.components.length > 0 && (
        <div className="metered-usage">
          <h3>Metered Usage</h3>
          <div className="usage-table">
            <table>
              <thead>
                <tr>
                  <th>Service</th>
                  <th>Usage</th>
                  <th>Cost</th>
                </tr>
              </thead>
              <tbody>
                {usageSummary.components.map((component) => (
                  <tr key={component.meteredComponentId}>
                    <td>{component.componentName}</td>
                    <td>
                      {component.totalUsage.toLocaleString()} {component.unit}
                    </td>
                    <td>${component.cost.toFixed(2)}</td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr>
                  <td colSpan={2}>Estimated Total</td>
                  <td>${usageSummary.estimatedCost.toFixed(2)}</td>
                </tr>
              </tfoot>
            </table>
          </div>
        </div>
      )}

      <style jsx>{`
        .usage-metrics {
          padding: 2rem;
          max-width: 1200px;
          margin: 0 auto;
        }

        .metrics-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .period-info {
          display: flex;
          gap: 1rem;
          align-items: center;
        }

        .days-remaining {
          padding: 0.25rem 0.75rem;
          background: #e7f3ff;
          border-radius: 12px;
          font-size: 0.875rem;
          font-weight: 600;
          color: #007bff;
        }

        .quotas-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: 1.5rem;
          margin-bottom: 2rem;
        }

        .quota-card {
          background: white;
          padding: 1.5rem;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .quota-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
        }

        .quota-header h3 {
          margin: 0;
          font-size: 1rem;
        }

        .quota-value {
          font-weight: 700;
          color: #333;
        }

        .unlimited-badge {
          padding: 0.25rem 0.75rem;
          background: #28a745;
          color: white;
          border-radius: 12px;
          font-size: 0.75rem;
          font-weight: 600;
        }

        .progress-bar {
          height: 8px;
          background: #f0f0f0;
          border-radius: 4px;
          overflow: hidden;
          margin-bottom: 0.5rem;
        }

        .progress-fill {
          height: 100%;
          transition: width 0.3s ease;
        }

        .quota-percentage {
          font-size: 0.875rem;
          font-weight: 600;
        }

        .metered-usage {
          background: white;
          padding: 1.5rem;
          border-radius: 8px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .metered-usage h3 {
          margin: 0 0 1rem 0;
        }

        table {
          width: 100%;
          border-collapse: collapse;
        }

        th,
        td {
          padding: 0.75rem;
          text-align: left;
          border-bottom: 1px solid #f0f0f0;
        }

        th {
          font-weight: 600;
          color: #666;
        }

        tfoot td {
          font-weight: 700;
          border-top: 2px solid #333;
        }
      `}</style>
    </div>
  );
};
