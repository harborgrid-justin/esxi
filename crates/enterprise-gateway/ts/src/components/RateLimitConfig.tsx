/**
 * Enterprise API Gateway - Rate Limit Configuration Component
 */

import React, { useState } from 'react';
import type { RateLimit, RateLimitAlgorithm } from '../types';

export interface RateLimitConfigProps {
  rateLimits: RateLimit[];
  onAdd?: (limit: Omit<RateLimit, 'id'>) => void;
  onUpdate?: (id: string, updates: Partial<RateLimit>) => void;
  onDelete?: (id: string) => void;
}

export const RateLimitConfig: React.FC<RateLimitConfigProps> = ({
  rateLimits,
  onAdd,
  onUpdate,
  onDelete,
}) => {
  const [showForm, setShowForm] = useState(false);
  const [formData, setFormData] = useState({
    name: '',
    algorithm: 'token-bucket' as RateLimitAlgorithm,
    limit: 100,
    window: 60000,
    scope: 'global' as const,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onAdd?.({ ...formData, enabled: true });
    setFormData({ name: '', algorithm: 'token-bucket', limit: 100, window: 60000, scope: 'global' });
    setShowForm(false);
  };

  return (
    <div className="rate-limit-config">
      <div className="header">
        <h2>Rate Limiting</h2>
        <button onClick={() => setShowForm(!showForm)}>
          {showForm ? 'Cancel' : '+ Add Limit'}
        </button>
      </div>

      {showForm && (
        <form onSubmit={handleSubmit} className="limit-form">
          <input
            placeholder="Name"
            value={formData.name}
            onChange={(e) => setFormData({ ...formData, name: e.target.value })}
            required
          />
          <select
            value={formData.algorithm}
            onChange={(e) => setFormData({ ...formData, algorithm: e.target.value as RateLimitAlgorithm })}
          >
            <option value="token-bucket">Token Bucket</option>
            <option value="sliding-window">Sliding Window</option>
            <option value="fixed-window">Fixed Window</option>
            <option value="adaptive">Adaptive</option>
          </select>
          <input
            type="number"
            placeholder="Limit"
            value={formData.limit}
            onChange={(e) => setFormData({ ...formData, limit: Number(e.target.value) })}
            required
          />
          <input
            type="number"
            placeholder="Window (ms)"
            value={formData.window}
            onChange={(e) => setFormData({ ...formData, window: Number(e.target.value) })}
            required
          />
          <button type="submit">Create</button>
        </form>
      )}

      <div className="limits-list">
        {rateLimits.map((limit) => (
          <div key={limit.id} className="limit-card">
            <h3>{limit.name}</h3>
            <p>Algorithm: {limit.algorithm}</p>
            <p>Limit: {limit.limit} requests per {limit.window}ms</p>
            <p>Scope: {limit.scope}</p>
            <button onClick={() => onUpdate?.(limit.id, { enabled: !limit.enabled })}>
              {limit.enabled ? 'Disable' : 'Enable'}
            </button>
            <button onClick={() => onDelete?.(limit.id)}>Delete</button>
          </div>
        ))}
      </div>
    </div>
  );
};
