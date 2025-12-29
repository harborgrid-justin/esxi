/**
 * ReportBuilder - Report creation and configuration interface
 */

import React, { useState } from 'react';
import { Report, ReportFormat, ReportSchedule, Dashboard } from '../../types';
import { v4 as uuidv4 } from 'uuid';

export interface ReportBuilderProps {
  dashboard: Dashboard;
  onSave?: (report: Partial<Report>) => void;
  onCancel?: () => void;
}

export const ReportBuilder: React.FC<ReportBuilderProps> = ({
  dashboard,
  onSave,
  onCancel,
}) => {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [format, setFormat] = useState<ReportFormat>('pdf');
  const [recipients, setRecipients] = useState<string[]>(['']);
  const [scheduleEnabled, setScheduleEnabled] = useState(false);
  const [cronExpression, setCronExpression] = useState('0 9 * * 1');
  const [timezone, setTimezone] = useState('UTC');

  const handleAddRecipient = () => {
    setRecipients([...recipients, '']);
  };

  const handleRemoveRecipient = (index: number) => {
    setRecipients(recipients.filter((_, i) => i !== index));
  };

  const handleRecipientChange = (index: number, value: string) => {
    const updated = [...recipients];
    updated[index] = value;
    setRecipients(updated);
  };

  const handleSave = () => {
    const schedule: ReportSchedule | undefined = scheduleEnabled
      ? {
          cron: cronExpression,
          timezone,
          enabled: true,
        }
      : undefined;

    const report: Partial<Report> = {
      name,
      description: description || undefined,
      dashboard_id: dashboard.id,
      format,
      schedule,
      recipients: recipients.filter((r) => r.trim() !== ''),
    };

    if (onSave) {
      onSave(report);
    }
  };

  const isValid = name.trim() !== '' && recipients.some((r) => r.trim() !== '');

  return (
    <div className="report-builder">
      <h2>Create Report</h2>

      <div className="form-group">
        <label htmlFor="name">Report Name *</label>
        <input
          id="name"
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Enter report name"
        />
      </div>

      <div className="form-group">
        <label htmlFor="description">Description</label>
        <textarea
          id="description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Optional description"
          rows={3}
        />
      </div>

      <div className="form-group">
        <label htmlFor="format">Format *</label>
        <select
          id="format"
          value={format}
          onChange={(e) => setFormat(e.target.value as ReportFormat)}
        >
          <option value="pdf">PDF</option>
          <option value="excel">Excel</option>
          <option value="csv">CSV</option>
          <option value="json">JSON</option>
        </select>
      </div>

      <div className="form-group">
        <label>Recipients *</label>
        {recipients.map((recipient, index) => (
          <div key={index} className="recipient-row">
            <input
              type="email"
              value={recipient}
              onChange={(e) => handleRecipientChange(index, e.target.value)}
              placeholder="email@example.com"
            />
            {recipients.length > 1 && (
              <button
                type="button"
                onClick={() => handleRemoveRecipient(index)}
                className="btn-remove"
              >
                Remove
              </button>
            )}
          </div>
        ))}
        <button type="button" onClick={handleAddRecipient} className="btn-add">
          + Add Recipient
        </button>
      </div>

      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={scheduleEnabled}
            onChange={(e) => setScheduleEnabled(e.target.checked)}
          />
          <span>Schedule Report</span>
        </label>
      </div>

      {scheduleEnabled && (
        <>
          <div className="form-group">
            <label htmlFor="cron">Cron Expression</label>
            <input
              id="cron"
              type="text"
              value={cronExpression}
              onChange={(e) => setCronExpression(e.target.value)}
              placeholder="0 9 * * 1"
            />
            <small>Example: 0 9 * * 1 (Every Monday at 9 AM)</small>
          </div>

          <div className="form-group">
            <label htmlFor="timezone">Timezone</label>
            <input
              id="timezone"
              type="text"
              value={timezone}
              onChange={(e) => setTimezone(e.target.value)}
              placeholder="UTC"
            />
          </div>
        </>
      )}

      <div className="actions">
        {onCancel && (
          <button type="button" onClick={onCancel} className="btn-cancel">
            Cancel
          </button>
        )}
        <button
          type="button"
          onClick={handleSave}
          disabled={!isValid}
          className="btn-save"
        >
          Create Report
        </button>
      </div>

      <style jsx>{`
        .report-builder {
          padding: 20px;
          background: white;
          border-radius: 8px;
          max-width: 600px;
        }

        h2 {
          margin: 0 0 20px 0;
          font-size: 24px;
          color: #333;
        }

        .form-group {
          margin-bottom: 20px;
        }

        .form-group label {
          display: block;
          margin-bottom: 8px;
          font-weight: 600;
          color: #333;
        }

        .form-group label input[type='checkbox'] {
          margin-right: 8px;
        }

        .form-group input,
        .form-group textarea,
        .form-group select {
          width: 100%;
          padding: 10px 12px;
          border: 1px solid #ddd;
          border-radius: 4px;
          font-size: 14px;
        }

        .form-group small {
          display: block;
          margin-top: 4px;
          font-size: 12px;
          color: #666;
        }

        .recipient-row {
          display: flex;
          gap: 8px;
          margin-bottom: 8px;
        }

        .recipient-row input {
          flex: 1;
        }

        .btn-remove {
          padding: 8px 16px;
          background: #f44336;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .btn-add {
          padding: 8px 16px;
          background: #4caf50;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          margin-top: 8px;
        }

        .actions {
          display: flex;
          gap: 12px;
          justify-content: flex-end;
          margin-top: 24px;
          padding-top: 24px;
          border-top: 1px solid #e0e0e0;
        }

        .btn-cancel {
          padding: 10px 20px;
          background: white;
          color: #333;
          border: 1px solid #ddd;
          border-radius: 4px;
          cursor: pointer;
        }

        .btn-save {
          padding: 10px 20px;
          background: #1976d2;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .btn-save:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};
