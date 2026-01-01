/**
 * Enterprise API Gateway - Route Manager Component
 *
 * Manage API routes and their configuration
 */

import React, { useState } from 'react';
import type { Route, Upstream, HTTPMethod } from '../types';

export interface RouteManagerProps {
  routes: Route[];
  upstreams: Upstream[];
  onAddRoute?: (route: Omit<Route, 'id' | 'createdAt' | 'updatedAt'>) => void;
  onUpdateRoute?: (id: string, updates: Partial<Route>) => void;
  onDeleteRoute?: (id: string) => void;
}

export const RouteManager: React.FC<RouteManagerProps> = ({
  routes,
  upstreams,
  onAddRoute,
  onUpdateRoute,
  onDeleteRoute,
}) => {
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingRoute, setEditingRoute] = useState<string | null>(null);
  const [formData, setFormData] = useState({
    name: '',
    methods: [] as HTTPMethod[],
    paths: '',
    matchType: 'exact' as const,
    upstreamId: '',
    enabled: true,
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!onAddRoute) return;

    const upstream = upstreams.find((u) => u.id === formData.upstreamId);
    if (!upstream) return;

    onAddRoute({
      name: formData.name,
      methods: formData.methods,
      paths: formData.paths.split('\n').filter(Boolean),
      matchType: formData.matchType,
      upstream,
      enabled: formData.enabled,
    });

    // Reset form
    setFormData({
      name: '',
      methods: [],
      paths: '',
      matchType: 'exact',
      upstreamId: '',
      enabled: true,
    });
    setShowAddForm(false);
  };

  const toggleMethod = (method: HTTPMethod) => {
    setFormData((prev) => ({
      ...prev,
      methods: prev.methods.includes(method)
        ? prev.methods.filter((m) => m !== method)
        : [...prev.methods, method],
    }));
  };

  return (
    <div className="route-manager">
      <div className="manager-header">
        <h2>Routes</h2>
        <button onClick={() => setShowAddForm(!showAddForm)}>
          {showAddForm ? 'Cancel' : '+ Add Route'}
        </button>
      </div>

      {showAddForm && (
        <div className="add-route-form">
          <h3>Add New Route</h3>
          <form onSubmit={handleSubmit}>
            <div className="form-group">
              <label>Route Name</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
              />
            </div>

            <div className="form-group">
              <label>HTTP Methods</label>
              <div className="method-checkboxes">
                {(['GET', 'POST', 'PUT', 'PATCH', 'DELETE'] as HTTPMethod[]).map((method) => (
                  <label key={method}>
                    <input
                      type="checkbox"
                      checked={formData.methods.includes(method)}
                      onChange={() => toggleMethod(method)}
                    />
                    {method}
                  </label>
                ))}
              </div>
            </div>

            <div className="form-group">
              <label>Paths (one per line)</label>
              <textarea
                value={formData.paths}
                onChange={(e) => setFormData({ ...formData, paths: e.target.value })}
                placeholder="/api/users&#10;/api/posts"
                rows={3}
                required
              />
            </div>

            <div className="form-group">
              <label>Match Type</label>
              <select
                value={formData.matchType}
                onChange={(e) => setFormData({ ...formData, matchType: e.target.value as any })}
              >
                <option value="exact">Exact Match</option>
                <option value="prefix">Prefix Match</option>
                <option value="regex">Regex Match</option>
              </select>
            </div>

            <div className="form-group">
              <label>Upstream</label>
              <select
                value={formData.upstreamId}
                onChange={(e) => setFormData({ ...formData, upstreamId: e.target.value })}
                required
              >
                <option value="">Select upstream...</option>
                {upstreams.map((upstream) => (
                  <option key={upstream.id} value={upstream.id}>
                    {upstream.name}
                  </option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label>
                <input
                  type="checkbox"
                  checked={formData.enabled}
                  onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                />
                Enabled
              </label>
            </div>

            <button type="submit">Create Route</button>
          </form>
        </div>
      )}

      <div className="routes-list">
        {routes.map((route) => (
          <div key={route.id} className="route-card">
            <div className="route-header">
              <h3>{route.name}</h3>
              <div className="route-actions">
                <button
                  className={route.enabled ? 'enabled' : 'disabled'}
                  onClick={() => onUpdateRoute?.(route.id, { enabled: !route.enabled })}
                >
                  {route.enabled ? '✓ Enabled' : '✗ Disabled'}
                </button>
                <button onClick={() => onDeleteRoute?.(route.id)}>Delete</button>
              </div>
            </div>
            <div className="route-details">
              <div className="detail-row">
                <span className="label">Methods:</span>
                <span className="value">{route.methods.join(', ')}</span>
              </div>
              <div className="detail-row">
                <span className="label">Paths:</span>
                <span className="value">{route.paths.join(', ')}</span>
              </div>
              <div className="detail-row">
                <span className="label">Match Type:</span>
                <span className="value">{route.matchType}</span>
              </div>
              <div className="detail-row">
                <span className="label">Upstream:</span>
                <span className="value">{route.upstream.name}</span>
              </div>
            </div>
          </div>
        ))}
      </div>

      <style>{`
        .route-manager {
          padding: 20px;
        }

        .manager-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 20px;
        }

        .manager-header button {
          padding: 10px 20px;
          background: #007bff;
          color: white;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .add-route-form {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
          margin-bottom: 20px;
        }

        .form-group {
          margin-bottom: 15px;
        }

        .form-group label {
          display: block;
          margin-bottom: 5px;
          font-weight: 600;
        }

        .form-group input[type="text"],
        .form-group textarea,
        .form-group select {
          width: 100%;
          padding: 8px;
          border: 1px solid #ddd;
          border-radius: 4px;
        }

        .method-checkboxes {
          display: flex;
          gap: 15px;
        }

        .method-checkboxes label {
          display: flex;
          align-items: center;
          gap: 5px;
        }

        .routes-list {
          display: grid;
          gap: 15px;
        }

        .route-card {
          background: white;
          padding: 20px;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }

        .route-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 15px;
          padding-bottom: 15px;
          border-bottom: 1px solid #eee;
        }

        .route-header h3 {
          margin: 0;
        }

        .route-actions {
          display: flex;
          gap: 10px;
        }

        .route-actions button {
          padding: 6px 12px;
          border: none;
          border-radius: 4px;
          cursor: pointer;
        }

        .route-actions button.enabled {
          background: #28a745;
          color: white;
        }

        .route-actions button.disabled {
          background: #6c757d;
          color: white;
        }

        .route-details {
          display: grid;
          gap: 10px;
        }

        .detail-row {
          display: flex;
          gap: 10px;
        }

        .detail-row .label {
          font-weight: 600;
          color: #666;
          min-width: 100px;
        }

        .detail-row .value {
          color: #1a1a1a;
        }
      `}</style>
    </div>
  );
};
