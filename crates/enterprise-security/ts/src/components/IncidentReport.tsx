/**
 * Incident Report - Security Incident Reporting UI
 * Report and track security incidents
 */

import React, { useState } from 'react';
import { incidentResponse } from '../services/IncidentResponse';
import { IncidentSeverity } from '../types';

export const IncidentReport: React.FC = () => {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [severity, setSeverity] = useState<IncidentSeverity>(IncidentSeverity.MEDIUM);
  const [category, setCategory] = useState('');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await incidentResponse.createIncident({
      title,
      description,
      severity,
      category,
      reportedBy: 'current-user',
    });
    // Reset form
    setTitle('');
    setDescription('');
    setSeverity(IncidentSeverity.MEDIUM);
    setCategory('');
  };

  return (
    <div className="incident-report">
      <h2>Report Security Incident</h2>
      <form onSubmit={handleSubmit}>
        <div className="form-group">
          <label>Title</label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            required
          />
        </div>

        <div className="form-group">
          <label>Description</label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            rows={4}
            required
          />
        </div>

        <div className="form-group">
          <label>Severity</label>
          <select value={severity} onChange={(e) => setSeverity(e.target.value as IncidentSeverity)}>
            <option value={IncidentSeverity.LOW}>Low</option>
            <option value={IncidentSeverity.MEDIUM}>Medium</option>
            <option value={IncidentSeverity.HIGH}>High</option>
            <option value={IncidentSeverity.CRITICAL}>Critical</option>
          </select>
        </div>

        <div className="form-group">
          <label>Category</label>
          <input
            type="text"
            value={category}
            onChange={(e) => setCategory(e.target.value)}
            placeholder="e.g., Data Breach, Malware, etc."
            required
          />
        </div>

        <button type="submit">Submit Incident</button>
      </form>

      <style>{`
        .incident-report {
          background: #fff;
          padding: 24px;
          border-radius: 8px;
          max-width: 600px;
        }
        .form-group {
          margin-bottom: 20px;
        }
        .form-group label {
          display: block;
          margin-bottom: 8px;
          font-weight: 600;
        }
        .form-group input,
        .form-group textarea,
        .form-group select {
          width: 100%;
          padding: 10px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }
        button {
          padding: 12px 24px;
          background: #f44336;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-weight: 600;
        }
        button:hover {
          background: #d32f2f;
        }
      `}</style>
    </div>
  );
};
