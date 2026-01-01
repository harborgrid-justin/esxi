/**
 * Compliance Report - Compliance Status Dashboard
 * View compliance framework status and controls
 */

import React, { useState } from 'react';
import { soc2Compliance } from '../compliance/SOC2Compliance';
import { hipaaCompliance } from '../compliance/HIPAACompliance';
import { gdprCompliance } from '../compliance/GDPRCompliance';
import { pciDSSCompliance } from '../compliance/PCIDSSCompliance';
import { ComplianceFramework } from '../types';

export const ComplianceReport: React.FC = () => {
  const [framework, setFramework] = useState<ComplianceFramework>(ComplianceFramework.SOC2);

  const getFrameworkData = () => {
    switch (framework) {
      case ComplianceFramework.SOC2:
        return {
          controls: soc2Compliance.getAllControls(),
          summary: soc2Compliance.getSummary(),
        };
      case ComplianceFramework.HIPAA:
        return {
          controls: hipaaCompliance.getAllControls(),
          summary: null,
        };
      case ComplianceFramework.GDPR:
        return {
          controls: gdprCompliance.getAllControls(),
          summary: null,
        };
      case ComplianceFramework.PCI_DSS:
        return {
          controls: pciDSSCompliance.getAllControls(),
          summary: null,
        };
      default:
        return { controls: [], summary: null };
    }
  };

  const data = getFrameworkData();

  return (
    <div className="compliance-report">
      <div className="report-header">
        <h2>Compliance Dashboard</h2>
        <select value={framework} onChange={(e) => setFramework(e.target.value as ComplianceFramework)}>
          <option value={ComplianceFramework.SOC2}>SOC 2</option>
          <option value={ComplianceFramework.HIPAA}>HIPAA</option>
          <option value={ComplianceFramework.GDPR}>GDPR</option>
          <option value={ComplianceFramework.PCI_DSS}>PCI DSS</option>
        </select>
      </div>

      {data.summary && (
        <div className="summary-cards">
          <div className="summary-card">
            <h3>Compliance Rate</h3>
            <div className="metric">{(data.summary.complianceRate * 100).toFixed(1)}%</div>
          </div>
          <div className="summary-card">
            <h3>Compliant</h3>
            <div className="metric success">{data.summary.compliant}</div>
          </div>
          <div className="summary-card">
            <h3>Non-Compliant</h3>
            <div className="metric error">{data.summary.nonCompliant}</div>
          </div>
          <div className="summary-card">
            <h3>Under Review</h3>
            <div className="metric warning">{data.summary.underReview}</div>
          </div>
        </div>
      )}

      <div className="controls-list">
        <h3>Controls ({data.controls.length})</h3>
        <table>
          <thead>
            <tr>
              <th>Code</th>
              <th>Title</th>
              <th>Category</th>
              <th>Status</th>
            </tr>
          </thead>
          <tbody>
            {data.controls.map((control) => (
              <tr key={control.id}>
                <td><code>{control.code}</code></td>
                <td>{control.title}</td>
                <td>{control.category}</td>
                <td><span className={`status-${control.status.toLowerCase()}`}>{control.status}</span></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <style>{`
        .compliance-report {
          background: #fff;
          padding: 24px;
          border-radius: 8px;
        }
        .report-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }
        .summary-cards {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 32px;
        }
        .summary-card {
          padding: 20px;
          background: #f5f5f5;
          border-radius: 8px;
          text-align: center;
        }
        .metric {
          font-size: 32px;
          font-weight: bold;
          margin-top: 8px;
        }
        .metric.success { color: #4caf50; }
        .metric.error { color: #f44336; }
        .metric.warning { color: #ff9800; }
        table {
          width: 100%;
          border-collapse: collapse;
        }
        th, td {
          padding: 12px;
          text-align: left;
          border-bottom: 1px solid #e0e0e0;
        }
        code {
          background: #f5f5f5;
          padding: 2px 6px;
          border-radius: 3px;
        }
      `}</style>
    </div>
  );
};
